use std::{cmp::Ordering, error::Error, fmt::Display, num::ParseIntError, str::FromStr};

use saphyr::{Mapping, Scalar, Yaml};

use crate::utils::{FromYaml, YamlExt, YamlParseError, YamlWriteError};

#[derive(PartialEq, Eq, Clone, Copy)]
/// A (Semantic Versioning)[https://semver.org/]-respecting version number
///
/// Minor or patch being `None` means they should be treated as wildcards.
pub struct Version {
    pub major: u16,
    pub minor: Option<u16>,
    pub patch: Option<u16>,
}

impl Version {
    /// Checks if the [Version] matches some semver expression.
    ///
    /// Assumes `self` has no wildcards in it.<br>
    /// Useful for checking if a version matches a dependency.
    pub fn matches(&self, other: &Self) -> bool {
        self.major == other.major
            && (other.minor.is_none() || self.minor >= other.minor)
            && (other.patch.is_none() || self.patch >= other.patch)
    }
}

#[derive(Debug, Clone)]
pub enum VersionParseError {
    WildcardMajor,
    NoMajorVersion,
    NoMinorVersion,
    InvalidInt(ParseIntError),
}

impl Error for VersionParseError {}

impl Display for VersionParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VersionParseError::WildcardMajor => write!(
                f,
                "Version parse error: Wildcard not allowed for major version"
            ),
            VersionParseError::NoMajorVersion =>
                write!(f, "Version parse error: No major version found"),
            VersionParseError::NoMinorVersion =>
                write!(f, "Version parse error: No minor version found"),
            VersionParseError::InvalidInt(parse_int_error) =>
                write!(f, "Version parse error: {parse_int_error}"),
        }
    }
}

impl FromStr for Version {
    type Err = VersionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('.');

        let major = match parts.next() {
            Some("*") => return Err(VersionParseError::WildcardMajor),
            Some(str) => str.parse::<u16>().map_err(VersionParseError::InvalidInt)?,
            None => return Err(VersionParseError::NoMajorVersion),
        };

        let minor = match parts.next() {
            Some("*") => None,
            Some(str) => Some(str.parse::<u16>().map_err(VersionParseError::InvalidInt)?),
            None => return Err(VersionParseError::NoMinorVersion),
        };

        let patch = match parts.next() {
            Some("*") => None,
            Some(str) => Some(str.parse::<u16>().map_err(VersionParseError::InvalidInt)?),
            None => Some(0),
        };

        Ok(Version {
            major,
            minor,
            patch,
        })
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let star = "*".to_owned();
        write!(
            f,
            "{}.{}.{}",
            self.major,
            self.minor
                .as_ref()
                .map(u16::to_string)
                .unwrap_or(star.clone()),
            self.patch.as_ref().map(u16::to_string).unwrap_or(star)
        )
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => match self.minor.cmp(&other.minor) {
                Ordering::Equal => self.patch.cmp(&other.patch),
                c => c,
            },
            c => c,
        }
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct ModMeta {
    pub name: String,
    pub version: Version,
    pub dll: Option<String>,
    pub dependencies: Vec<(String, Option<Version>)>,
    pub optional_dependencies: Option<Vec<(String, Option<Version>)>>,
}

impl ModMeta {
    fn parse_name_version_from_yaml(
        yaml: &Yaml,
    ) -> Result<(String, Option<Version>), YamlParseError> {
        let name = yaml["Name"]
            .as_str()
            .map(ToString::to_string)
            .ok_or(YamlParseError::Custom(
                "everest.yaml mod definition found without a name".to_string(),
            ))?;
        let version = if let Some(y) = yaml.as_mapping_get("Version") {
            Some(match y {
                Yaml::Value(Scalar::FloatingPoint(f)) => Version::from_str(&f.to_string())
                    .map_err(|e| YamlParseError::Custom(e.to_string()))?,
                Yaml::Value(Scalar::String(s)) =>
                    Version::from_str(s).map_err(|e| YamlParseError::Custom(e.to_string()))?,
                _ => Err(YamlParseError::Custom(
                    "everest.yaml Version isn't a string or a float".to_owned(),
                ))?,
            })
        } else {
            None
        };

        Ok((name, version))
    }

    fn name_version_to_yaml(name: &str, version: &Option<Version>, hash: &mut Mapping) {
        hash.insert(
            Yaml::string("Name".to_owned()),
            Yaml::string(name.to_owned()),
        );
        if let Some(v) = version {
            hash.insert(
                Yaml::string("Version".to_owned()),
                Yaml::string(v.to_string()),
            );
        }
    }
}

impl FromYaml for ModMeta {
    fn parse_from_yaml(yaml: &saphyr::Yaml) -> Result<ModMeta, YamlParseError> {
        let (name, version) = ModMeta::parse_name_version_from_yaml(yaml)?;
        let version = version.ok_or(YamlParseError::Custom(format!(
            "Mod '{name}' is missing version in everest.yaml"
        )))?;

        let dll = yaml
            .as_mapping_get("dll")
            .and_then(Yaml::as_str)
            .map(ToOwned::to_owned);

        let dependencies = if yaml.contains_mapping_key("Dependencies") {
            yaml["Dependencies"]
                .as_vec()
                .ok_or(YamlParseError::Custom(
                    "Dependencies isn't a vec in everest.yaml".to_string(),
                ))?
                .iter()
                .map(ModMeta::parse_name_version_from_yaml)
                .collect::<Result<Vec<_>, _>>()?
        } else {
            Vec::new()
        };

        // OptionalDependencies is an optional field so we don't error if its None
        // I'm sure theres a better way to write this but I'm so tired
        let optional_dependencies = yaml
            .as_mapping_get("OptionalDependencies")
            .and_then(Yaml::as_vec)
            .map(|v| {
                v.iter()
                    .map(ModMeta::parse_name_version_from_yaml)
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()?;

        Ok(ModMeta {
            name,
            version,
            dll,
            dependencies,
            optional_dependencies,
        })
    }

    fn to_yaml(&self) -> Result<Yaml, YamlWriteError> {
        let mut hash = Mapping::new();
        ModMeta::name_version_to_yaml(&self.name, &Some(self.version), &mut hash);

        let mut dep_hash = Mapping::new();

        for (dep_name, dep_version) in &self.dependencies {
            ModMeta::name_version_to_yaml(dep_name, dep_version, &mut dep_hash);
        }

        hash.insert(
            Yaml::string("Dependencies".to_owned()),
            Yaml::hash(dep_hash),
        );

        let mut opt_dep_hash = Mapping::new();

        if let Some(optional_deps) = &self.optional_dependencies {
            for (dep_name, dep_version) in optional_deps {
                ModMeta::name_version_to_yaml(dep_name, dep_version, &mut opt_dep_hash);
            }
        }

        hash.insert(
            Yaml::string("OptionalDependencies".to_owned()),
            Yaml::hash(opt_dep_hash),
        );

        if let Some(dll) = &self.dll {
            hash.insert(Yaml::string("DLL".to_owned()), Yaml::string(dll.to_owned()));
        }

        Ok(Yaml::hash(hash))
    }
}
