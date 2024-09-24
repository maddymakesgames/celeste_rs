use std::{cmp::Ordering, fmt::Display};

use saphyr::{Hash, Yaml};

use crate::utils::{YamlFile, YamlParseError, YamlWriteError};

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
    pub dependencies: Vec<(String, Version)>,
    pub optional_dependencies: Option<Vec<(String, Version)>>,
}

impl ModMeta {
    fn parse_name_version_from_yaml(yaml: &Yaml) -> Result<(String, Version), YamlParseError> {
        let name = yaml["Name"]
            .as_str()
            .map(ToString::to_string)
            .ok_or(YamlParseError::Custom(
                "everest.yaml mod definition found without a name".to_string(),
            ))?;

        let mut version_parts = yaml["Version"]
            .as_str()
            .ok_or(YamlParseError::Custom(
                "everest.yaml mod definition found without a version".to_string(),
            ))?
            .split('.');

        let major = match version_parts.next() {
            Some("*") =>
                return Err(YamlParseError::Custom(
                    "mod version isn't allowed to have '*' for major number".to_string(),
                )),
            Some(str) => str
                .parse::<u16>()
                .map_err(|e| e.to_string())
                .map_err(YamlParseError::Custom)?,
            None =>
                return Err(YamlParseError::Custom(
                    "mod version found with no major version number".to_string(),
                )),
        };

        let minor = match version_parts.next() {
            Some("*") => None,
            Some(str) => Some(
                str.parse::<u16>()
                    .map_err(|e| e.to_string())
                    .map_err(YamlParseError::Custom)?,
            ),
            None =>
                return Err(YamlParseError::Custom(
                    "mod version found with no minor version number".to_string(),
                )),
        };

        let patch = match version_parts.next() {
            Some("*") => None,
            Some(str) => Some(
                str.parse::<u16>()
                    .map_err(|e| e.to_string())
                    .map_err(YamlParseError::Custom)?,
            ),
            None =>
                return Err(YamlParseError::Custom(
                    "mod version found with no patch version number".to_string(),
                )),
        };

        Ok((name, Version {
            major,
            minor,
            patch,
        }))
    }

    fn name_version_to_yaml(name: &str, version: &Version, hash: &mut Hash) {
        hash.insert(
            Yaml::String("Name".to_owned()),
            Yaml::String(name.to_owned()),
        );
        hash.insert(
            Yaml::String("Version".to_owned()),
            Yaml::String(version.to_string()),
        );
    }
}

impl YamlFile for ModMeta {
    fn parse_from_yaml(yaml: &saphyr::Yaml) -> Result<ModMeta, YamlParseError> {
        let (name, version) = ModMeta::parse_name_version_from_yaml(yaml)?;

        let dll = yaml["dll"].as_str().map(ToOwned::to_owned);

        let dependencies = yaml["Dependencies"]
            .as_vec()
            .ok_or(YamlParseError::Custom(
                "No dependencies declared in everest.yaml".to_string(),
            ))?
            .iter()
            .map(ModMeta::parse_name_version_from_yaml)
            .collect::<Result<Vec<_>, _>>()?;

        // OptionalDependencies is an optional field so we don't error if its None
        // I'm sure theres a better way to write this but I'm so tired
        let optional_dependencies = yaml["OptionalDependencies"]
            .as_vec()
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
        let mut hash = Hash::new();
        ModMeta::name_version_to_yaml(&self.name, &self.version, &mut hash);

        let mut dep_hash = Hash::new();

        for (dep_name, dep_version) in &self.dependencies {
            ModMeta::name_version_to_yaml(dep_name, dep_version, &mut dep_hash);
        }

        hash.insert(
            Yaml::String("Dependencies".to_owned()),
            Yaml::Hash(dep_hash),
        );

        let mut opt_dep_hash = Hash::new();

        if let Some(optional_deps) = &self.optional_dependencies {
            for (dep_name, dep_version) in optional_deps {
                ModMeta::name_version_to_yaml(dep_name, dep_version, &mut opt_dep_hash);
            }
        }

        hash.insert(
            Yaml::String("OptionalDependencies".to_owned()),
            Yaml::Hash(opt_dep_hash),
        );

        if let Some(dll) = &self.dll {
            hash.insert(Yaml::String("DLL".to_owned()), Yaml::String(dll.to_owned()));
        }

        Ok(Yaml::Hash(hash))
    }
}
