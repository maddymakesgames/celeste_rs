use std::{
    collections::{BTreeMap, HashMap},
    error::Error,
    fmt::{Debug, Display, Write},
    hash::Hash,
    io::Read,
    sync::Arc,
};

use saphyr::{
    emitter::EmitError,
    yaml::LoadError,
    ScanError,
    YAMLDecodingTrap,
    Yaml,
    YamlDecoder,
    YamlEmitter,
    YamlLoader,
};

pub use saphyr;

pub trait YamlFile: Sized {
    fn parse_from_yaml(yaml: &Yaml) -> Result<Self, YamlParseError>;

    fn parse_from_str(str: &str) -> Result<Self, YamlParseError> {
        let yaml = YamlLoader::load_from_str(str)?;
        Self::parse_from_yaml(&yaml[0])
    }

    fn parse_from_reader(reader: impl Read) -> Result<Self, YamlReadError> {
        let yaml = YamlDecoder::read(reader)
            .encoding_trap(YAMLDecodingTrap::Strict)
            .decode();
        match yaml {
            Ok(y) => Self::parse_from_yaml(&y[0]).map_err(YamlReadError::ParseError),
            Err(e) => Err(e.into()),
        }
    }

    fn to_yaml(&self) -> Result<Yaml, YamlWriteError>;

    fn to_writer(&self, writer: &mut impl Write) -> Result<(), YamlWriteError> {
        let yaml = self.to_yaml()?;
        let mut emitter = YamlEmitter::new(writer);
        emitter.multiline_strings(true);
        Ok(emitter.dump(&yaml)?)
    }
}

impl<T: YamlFile> YamlFile for Vec<T> {
    fn parse_from_str(str: &str) -> Result<Vec<T>, YamlParseError> {
        YamlLoader::load_from_str(str)?
            .iter()
            .map(T::parse_from_yaml)
            .collect()
    }

    fn parse_from_reader(reader: impl Read) -> Result<Vec<T>, YamlReadError> {
        let yaml = YamlDecoder::read(reader)
            .encoding_trap(YAMLDecodingTrap::Strict)
            .decode()?;

        yaml.iter()
            .map(T::parse_from_yaml)
            .map(|t| t.map_err(YamlReadError::ParseError))
            .collect()
    }

    fn parse_from_yaml(yaml: &Yaml) -> Result<Vec<T>, YamlParseError> {
        yaml.as_vec()
            .ok_or(YamlParseError::TypeMismatch(
                "Yaml passed to Vec::parsed_from_yaml isn't an array.",
                yaml_type_name(yaml),
            ))?
            .iter()
            .map(T::parse_from_yaml)
            .collect()
    }

    fn to_yaml(&self) -> Result<Yaml, YamlWriteError> {
        Ok(Yaml::Array(
            self.iter().map(T::to_yaml).collect::<Result<Vec<_>, _>>()?,
        ))
    }
}

#[derive(Debug)]
pub enum YamlReadError {
    LoadError(LoadError),
    ParseError(YamlParseError),
}

impl Display for YamlReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            YamlReadError::LoadError(load_error) => Debug::fmt(load_error, f),
            YamlReadError::ParseError(yaml_parse_error) => Display::fmt(yaml_parse_error, f),
        }
    }
}

impl Error for YamlReadError {}

impl From<LoadError> for YamlReadError {
    fn from(value: LoadError) -> Self {
        YamlReadError::LoadError(value)
    }
}

impl From<YamlParseError> for YamlReadError {
    fn from(value: YamlParseError) -> Self {
        YamlReadError::ParseError(value)
    }
}

#[derive(Debug, Clone)]
pub enum YamlParseError {
    TypeMismatch(&'static str, &'static str),
    ArrayLenMismatch(&'static str, usize, usize),
    ScanError(ScanError),
    IoError(Arc<std::io::Error>),
    Custom(String),
}

impl Display for YamlParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            YamlParseError::TypeMismatch(expected, found) => write!(
                f,
                "Type mismatch when parsing yaml, expected \"{expected}\", found \"{found}\""
            ),
            YamlParseError::ArrayLenMismatch(name, found, expected) => write!(
                f,
                "Array size mismatch when parsing yaml. Array {name} expected length {expected} \
                 but got length {found}"
            ),
            YamlParseError::IoError(error) => Display::fmt(error, f),
            YamlParseError::ScanError(scan_error) => Display::fmt(scan_error, f),
            YamlParseError::Custom(str) => write!(f, "{str}"),
        }
    }
}

impl Error for YamlParseError {}

impl YamlParseError {
    pub fn custom(str: &str) -> Self {
        Self::Custom(str.to_owned())
    }

    pub fn custom_from_err(e: impl ToString) -> Self {
        Self::Custom(e.to_string())
    }
}


impl From<ScanError> for YamlParseError {
    fn from(value: ScanError) -> Self {
        YamlParseError::ScanError(value)
    }
}

#[derive(Debug)]
pub enum YamlWriteError {
    Custom(String),
    EmitError(EmitError),
}

impl Display for YamlWriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            YamlWriteError::Custom(c) => f.write_str(c),
            YamlWriteError::EmitError(emit_error) => Display::fmt(emit_error, f),
        }
    }
}

impl Error for YamlWriteError {}

impl YamlWriteError {
    pub fn custom(str: &str) -> Self {
        Self::Custom(str.to_owned())
    }

    pub fn custom_from_err(e: impl ToString) -> Self {
        Self::Custom(e.to_string())
    }
}

impl From<EmitError> for YamlWriteError {
    fn from(value: EmitError) -> Self {
        YamlWriteError::EmitError(value)
    }
}

pub fn yaml_type_name(yaml: &Yaml) -> &'static str {
    match yaml {
        Yaml::Real(_) => "f64",
        Yaml::Integer(_) => "i64",
        Yaml::String(_) => "String",
        Yaml::Boolean(_) => "bool",
        Yaml::Array(_) => "Vec",
        Yaml::Hash(_) => "Hash",
        Yaml::Alias(_) => "Alias",
        Yaml::Null => "null",
        Yaml::BadValue => "BADVALUE",
    }
}

impl YamlFile for f64 {
    fn parse_from_yaml(yaml: &Yaml) -> Result<f64, YamlParseError> {
        yaml.as_f64()
            .ok_or(YamlParseError::TypeMismatch("f64", yaml_type_name(yaml)))
    }

    fn to_yaml(&self) -> Result<Yaml, YamlWriteError> {
        Ok(Yaml::Real(self.to_string()))
    }
}

impl YamlFile for f32 {
    fn parse_from_yaml(yaml: &Yaml) -> Result<Self, YamlParseError> {
        f64::parse_from_yaml(yaml).map(|d| d as f32)
    }

    fn to_yaml(&self) -> Result<Yaml, YamlWriteError> {
        f64::to_yaml(&(*self as f64))
    }
}

impl YamlFile for i64 {
    fn parse_from_yaml(yaml: &Yaml) -> Result<Self, YamlParseError> {
        yaml.as_i64()
            .ok_or(YamlParseError::TypeMismatch("i64", yaml_type_name(yaml)))
    }

    fn to_yaml(&self) -> Result<Yaml, YamlWriteError> {
        Ok(Yaml::Integer(*self))
    }
}

impl YamlFile for i32 {
    fn parse_from_yaml(yaml: &Yaml) -> Result<Self, YamlParseError> {
        i64::parse_from_yaml(yaml).map(|d| d as i32)
    }

    fn to_yaml(&self) -> Result<Yaml, YamlWriteError> {
        i64::to_yaml(&(*self as i64))
    }
}

impl YamlFile for String {
    fn parse_from_yaml(yaml: &Yaml) -> Result<Self, YamlParseError> {
        Ok(yaml
            .as_str()
            .ok_or(YamlParseError::TypeMismatch("String", yaml_type_name(yaml)))?
            .to_owned())
    }

    fn to_yaml(&self) -> Result<Yaml, YamlWriteError> {
        Ok(Yaml::String(self.clone()))
    }
}

impl YamlFile for bool {
    fn parse_from_yaml(yaml: &Yaml) -> Result<Self, YamlParseError> {
        yaml.as_bool()
            .ok_or(YamlParseError::TypeMismatch("bool", yaml_type_name(yaml)))
    }

    fn to_yaml(&self) -> Result<Yaml, YamlWriteError> {
        Ok(Yaml::Boolean(*self))
    }
}

impl<K: YamlFile + Eq + Hash, V: YamlFile> YamlFile for HashMap<K, V> {
    fn parse_from_yaml(yaml: &Yaml) -> Result<Self, YamlParseError> {
        yaml.as_hash()
            .ok_or(YamlParseError::TypeMismatch("Hash", yaml_type_name(yaml)))
            .map(|h| {
                h.into_iter()
                    .map(|(k, v)| {
                        let k = match K::parse_from_yaml(k) {
                            Ok(k) => k,
                            Err(e) => return Err(e),
                        };
                        let v = match V::parse_from_yaml(v) {
                            Ok(v) => v,
                            Err(e) => return Err(e),
                        };

                        Ok((k, v))
                    })
                    .collect::<Result<HashMap<K, V>, _>>()
            })?
    }

    fn to_yaml(&self) -> Result<Yaml, YamlWriteError> {
        Ok(Yaml::Hash(
            self.iter()
                .map(|(k, v)| {
                    let k = match k.to_yaml() {
                        Ok(k) => k,
                        Err(e) => return Err(e),
                    };
                    let v = match v.to_yaml() {
                        Ok(v) => v,
                        Err(e) => return Err(e),
                    };

                    Ok((k, v))
                })
                .collect::<Result<saphyr::Hash, _>>()?,
        ))
    }
}


impl<K: YamlFile + Ord, V: YamlFile> YamlFile for BTreeMap<K, V> {
    fn parse_from_yaml(yaml: &Yaml) -> Result<Self, YamlParseError> {
        yaml.as_hash()
            .ok_or(YamlParseError::TypeMismatch("Hash", yaml_type_name(yaml)))
            .map(|h| {
                h.into_iter()
                    .map(|(k, v)| {
                        let k = match K::parse_from_yaml(k) {
                            Ok(k) => k,
                            Err(e) => return Err(e),
                        };
                        let v = match V::parse_from_yaml(v) {
                            Ok(v) => v,
                            Err(e) => return Err(e),
                        };

                        Ok((k, v))
                    })
                    .collect::<Result<BTreeMap<K, V>, _>>()
            })?
    }

    fn to_yaml(&self) -> Result<Yaml, YamlWriteError> {
        Ok(Yaml::Hash(
            self.iter()
                .map(|(k, v)| {
                    let k = match k.to_yaml() {
                        Ok(k) => k,
                        Err(e) => return Err(e),
                    };
                    let v = match v.to_yaml() {
                        Ok(v) => v,
                        Err(e) => return Err(e),
                    };

                    Ok((k, v))
                })
                .collect::<Result<saphyr::Hash, _>>()?,
        ))
    }
}
