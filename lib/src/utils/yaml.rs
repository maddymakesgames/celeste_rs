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
    Array,
    Hash as YamlHash,
    ScanError,
    YAMLDecodingTrap,
    Yaml,
    YamlDecoder,
    YamlEmitter,
    YamlLoader,
};

pub use saphyr;

pub trait YamlExt {
    fn try_as_bool(&self) -> Result<bool, YamlParseError>;
    fn try_as_i64(&self) -> Result<i64, YamlParseError>;
    fn try_as_f64(&self) -> Result<f64, YamlParseError>;
    fn try_as_str(&self) -> Result<&str, YamlParseError>;
    fn try_as_hash(&self) -> Result<&YamlHash, YamlParseError>;
    fn try_as_vec(&self) -> Result<&Array, YamlParseError>;
    fn try_as_mut_hash(&mut self) -> Result<&mut YamlHash, YamlParseError>;
    fn try_as_mut_vec(&mut self) -> Result<&mut Array, YamlParseError>;
    fn type_name(&self) -> &'static str;
}

impl YamlExt for Yaml {
    fn try_as_bool(&self) -> Result<bool, YamlParseError> {
        self.as_bool()
            .ok_or(YamlParseError::TypeMismatch("bool", self.type_name()))
    }

    fn try_as_i64(&self) -> Result<i64, YamlParseError> {
        self.as_i64()
            .ok_or(YamlParseError::TypeMismatch("i64", self.type_name()))
    }

    fn try_as_f64(&self) -> Result<f64, YamlParseError> {
        self.as_f64()
            .ok_or(YamlParseError::TypeMismatch("f64", self.type_name()))
    }

    fn try_as_str(&self) -> Result<&str, YamlParseError> {
        self.as_str()
            .ok_or(YamlParseError::TypeMismatch("String", self.type_name()))
    }

    fn try_as_hash(&self) -> Result<&YamlHash, YamlParseError> {
        self.as_hash()
            .ok_or(YamlParseError::TypeMismatch("Hash", self.type_name()))
    }

    fn try_as_vec(&self) -> Result<&Array, YamlParseError> {
        self.as_vec()
            .ok_or(YamlParseError::TypeMismatch("Vec", self.type_name()))
    }

    fn try_as_mut_hash(&mut self) -> Result<&mut YamlHash, YamlParseError> {
        let type_name = self.type_name();
        self.as_mut_hash()
            .ok_or(YamlParseError::TypeMismatch("Hash", type_name))
    }

    fn try_as_mut_vec(&mut self) -> Result<&mut Array, YamlParseError> {
        let type_name = self.type_name();
        self.as_mut_vec()
            .ok_or(YamlParseError::TypeMismatch("Vec", type_name))
    }

    fn type_name(&self) -> &'static str {
        yaml_type_name(self)
    }
}


pub trait HashExt {
    fn get_bool(&self, key: &Yaml) -> Option<Result<bool, YamlParseError>>;
    fn get_i64(&self, key: &Yaml) -> Option<Result<i64, YamlParseError>>;
    fn get_f64(&self, key: &Yaml) -> Option<Result<f64, YamlParseError>>;
    fn get_str(&self, key: &Yaml) -> Option<Result<&str, YamlParseError>>;
    fn get_hash(&self, key: &Yaml) -> Option<Result<&YamlHash, YamlParseError>>;
    fn get_vec(&self, key: &Yaml) -> Option<Result<&Array, YamlParseError>>;
    fn get_mut_hash(&mut self, key: &Yaml) -> Option<Result<&mut YamlHash, YamlParseError>>;
    fn get_mut_vec(&mut self, key: &Yaml) -> Option<Result<&mut Array, YamlParseError>>;
}

impl HashExt for YamlHash {
    fn get_bool(&self, key: &Yaml) -> Option<Result<bool, YamlParseError>> {
        self.get(key).map(YamlExt::try_as_bool)
    }

    fn get_i64(&self, key: &Yaml) -> Option<Result<i64, YamlParseError>> {
        self.get(key).map(YamlExt::try_as_i64)
    }

    fn get_f64(&self, key: &Yaml) -> Option<Result<f64, YamlParseError>> {
        self.get(key).map(YamlExt::try_as_f64)
    }

    fn get_str(&self, key: &Yaml) -> Option<Result<&str, YamlParseError>> {
        self.get(key).map(YamlExt::try_as_str)
    }

    fn get_hash(&self, key: &Yaml) -> Option<Result<&YamlHash, YamlParseError>> {
        self.get(key).map(YamlExt::try_as_hash)
    }

    fn get_vec(&self, key: &Yaml) -> Option<Result<&Array, YamlParseError>> {
        self.get(key).map(YamlExt::try_as_vec)
    }

    fn get_mut_hash(&mut self, key: &Yaml) -> Option<Result<&mut YamlHash, YamlParseError>> {
        self.get_mut(key).map(YamlExt::try_as_mut_hash)
    }

    fn get_mut_vec(&mut self, key: &Yaml) -> Option<Result<&mut Array, YamlParseError>> {
        self.get_mut(key).map(YamlExt::try_as_mut_vec)
    }
}


pub trait FromYaml: Sized {
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

impl<T: FromYaml> FromYaml for Vec<T> {
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
        yaml.try_as_vec()?.iter().map(T::parse_from_yaml).collect()
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

impl FromYaml for f64 {
    fn parse_from_yaml(yaml: &Yaml) -> Result<f64, YamlParseError> {
        yaml.try_as_f64()
    }

    fn to_yaml(&self) -> Result<Yaml, YamlWriteError> {
        Ok(Yaml::Real(self.to_string()))
    }
}

impl FromYaml for f32 {
    fn parse_from_yaml(yaml: &Yaml) -> Result<Self, YamlParseError> {
        f64::parse_from_yaml(yaml).map(|d| d as f32)
    }

    fn to_yaml(&self) -> Result<Yaml, YamlWriteError> {
        f64::to_yaml(&(*self as f64))
    }
}

impl FromYaml for i64 {
    fn parse_from_yaml(yaml: &Yaml) -> Result<Self, YamlParseError> {
        yaml.try_as_i64()
    }

    fn to_yaml(&self) -> Result<Yaml, YamlWriteError> {
        Ok(Yaml::Integer(*self))
    }
}

impl FromYaml for i32 {
    fn parse_from_yaml(yaml: &Yaml) -> Result<Self, YamlParseError> {
        i64::parse_from_yaml(yaml).map(|d| d as i32)
    }

    fn to_yaml(&self) -> Result<Yaml, YamlWriteError> {
        i64::to_yaml(&(*self as i64))
    }
}

impl FromYaml for String {
    fn parse_from_yaml(yaml: &Yaml) -> Result<Self, YamlParseError> {
        Ok(yaml.try_as_str()?.to_owned())
    }

    fn to_yaml(&self) -> Result<Yaml, YamlWriteError> {
        Ok(Yaml::String(self.clone()))
    }
}

impl FromYaml for bool {
    fn parse_from_yaml(yaml: &Yaml) -> Result<Self, YamlParseError> {
        yaml.try_as_bool()
    }

    fn to_yaml(&self) -> Result<Yaml, YamlWriteError> {
        Ok(Yaml::Boolean(*self))
    }
}

impl<K: FromYaml + Eq + Hash, V: FromYaml> FromYaml for HashMap<K, V> {
    fn parse_from_yaml(yaml: &Yaml) -> Result<Self, YamlParseError> {
        yaml.try_as_hash()?
            .into_iter()
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


impl<K: FromYaml + Ord, V: FromYaml> FromYaml for BTreeMap<K, V> {
    fn parse_from_yaml(yaml: &Yaml) -> Result<Self, YamlParseError> {
        yaml.try_as_hash()?
            .into_iter()
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

impl<T: FromYaml> FromYaml for Option<T> {
    fn parse_from_yaml(yaml: &Yaml) -> Result<Self, YamlParseError> {
        Ok(T::parse_from_yaml(yaml).ok())
    }

    fn to_yaml(&self) -> Result<Yaml, YamlWriteError> {
        if let Some(t) = self {
            t.to_yaml()
        } else {
            Ok(Yaml::Null)
        }
    }
}
