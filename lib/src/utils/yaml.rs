use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
    error::Error,
    fmt::{Debug, Display, Write},
    hash::Hash,
    io::Read,
    sync::Arc,
};

use saphyr::{
    EmitError,
    LoadError,
    LoadableYamlNode,
    Mapping,
    Scalar,
    ScanError,
    Sequence,
    YAMLDecodingTrap,
    Yaml,
    YamlDecoder,
    YamlEmitter,
};

pub use saphyr;

pub trait YamlExt<'a> {
    fn string(str: String) -> Self;
    fn str(str: &'a str) -> Self;
    fn int(int: i64) -> Self;
    fn float(float: f64) -> Self;
    fn bool(bool: bool) -> Self;
    fn hash(hash: Mapping<'a>) -> Self;
    fn seq(seq: Sequence<'a>) -> Self;
    fn try_as_bool(&self) -> Result<bool, YamlParseError>;
    fn try_as_i64(&self) -> Result<i64, YamlParseError>;
    fn try_as_f64(&self) -> Result<f64, YamlParseError>;
    fn try_as_str(&self) -> Result<&str, YamlParseError>;
    fn try_as_hash(&self) -> Result<&Mapping<'a>, YamlParseError>;
    fn try_as_vec(&self) -> Result<&Sequence<'a>, YamlParseError>;
    fn try_as_mut_hash(&mut self) -> Result<&mut Mapping<'a>, YamlParseError>;
    fn try_as_mut_vec(&mut self) -> Result<&mut Sequence<'a>, YamlParseError>;
    fn type_name(&self) -> &'static str;
}

impl<'a> YamlExt<'a> for Yaml<'a> {
    fn try_as_bool(&self) -> Result<bool, YamlParseError> {
        self.as_bool()
            .ok_or(YamlParseError::TypeMismatch("bool", self.type_name()))
    }

    fn try_as_i64(&self) -> Result<i64, YamlParseError> {
        self.as_integer()
            .ok_or(YamlParseError::TypeMismatch("i64", self.type_name()))
    }

    fn try_as_f64(&self) -> Result<f64, YamlParseError> {
        self.as_floating_point()
            .ok_or(YamlParseError::TypeMismatch("f64", self.type_name()))
    }

    fn try_as_str(&self) -> Result<&str, YamlParseError> {
        self.as_str()
            .ok_or(YamlParseError::TypeMismatch("String", self.type_name()))
    }

    fn try_as_hash(&self) -> Result<&Mapping<'a>, YamlParseError> {
        self.as_mapping()
            .ok_or(YamlParseError::TypeMismatch("Hash", self.type_name()))
    }

    fn try_as_vec(&self) -> Result<&Sequence<'a>, YamlParseError> {
        self.as_vec()
            .ok_or(YamlParseError::TypeMismatch("Vec", self.type_name()))
    }

    fn try_as_mut_hash(&mut self) -> Result<&mut Mapping<'a>, YamlParseError> {
        let type_name = self.type_name();
        self.as_mapping_mut()
            .ok_or(YamlParseError::TypeMismatch("Hash", type_name))
    }

    fn try_as_mut_vec(&mut self) -> Result<&mut Sequence<'a>, YamlParseError> {
        let type_name = self.type_name();
        self.as_sequence_mut()
            .ok_or(YamlParseError::TypeMismatch("Vec", type_name))
    }

    fn type_name(&self) -> &'static str {
        yaml_type_name(self)
    }

    fn string(str: String) -> Self {
        Yaml::Value(Scalar::String(str.into()))
    }

    fn str(str: &'a str) -> Self {
        Yaml::Value(Scalar::String(Cow::Borrowed(str)))
    }

    fn int(int: i64) -> Self {
        Yaml::Value(Scalar::Integer(int))
    }

    fn float(float: f64) -> Self {
        Yaml::Value(Scalar::FloatingPoint(float.into()))
    }

    fn bool(bool: bool) -> Self {
        Yaml::Value(Scalar::Boolean(bool))
    }

    fn hash(hash: Mapping<'a>) -> Self {
        Yaml::Mapping(hash)
    }

    fn seq(seq: Sequence<'a>) -> Self {
        Yaml::Sequence(seq)
    }
}


pub trait HashExt<'a> {
    fn get_bool(&self, key: &Yaml<'a>) -> Option<Result<bool, YamlParseError>>;
    fn get_i64(&self, key: &Yaml<'a>) -> Option<Result<i64, YamlParseError>>;
    fn get_f64(&self, key: &Yaml<'a>) -> Option<Result<f64, YamlParseError>>;
    fn get_str(&self, key: &Yaml<'a>) -> Option<Result<&str, YamlParseError>>;
    fn get_hash(&self, key: &Yaml<'a>) -> Option<Result<&Mapping<'a>, YamlParseError>>;
    fn get_vec(&self, key: &Yaml<'a>) -> Option<Result<&Sequence<'a>, YamlParseError>>;
    fn get_mut_hash(&mut self, key: &Yaml<'a>) -> Option<Result<&mut Mapping<'a>, YamlParseError>>;
    fn get_mut_vec(&mut self, key: &Yaml<'a>) -> Option<Result<&mut Sequence<'a>, YamlParseError>>;
}

impl<'a> HashExt<'a> for Mapping<'a> {
    fn get_bool(&self, key: &Yaml<'a>) -> Option<Result<bool, YamlParseError>> {
        self.get(key).map(YamlExt::try_as_bool)
    }

    fn get_i64(&self, key: &Yaml<'a>) -> Option<Result<i64, YamlParseError>> {
        self.get(key).map(YamlExt::try_as_i64)
    }

    fn get_f64(&self, key: &Yaml<'a>) -> Option<Result<f64, YamlParseError>> {
        self.get(key).map(YamlExt::try_as_f64)
    }

    fn get_str(&self, key: &Yaml<'a>) -> Option<Result<&str, YamlParseError>> {
        self.get(key).map(YamlExt::try_as_str)
    }

    fn get_hash(&self, key: &Yaml<'a>) -> Option<Result<&Mapping<'a>, YamlParseError>> {
        self.get(key).map(YamlExt::try_as_hash)
    }

    fn get_vec(&self, key: &Yaml<'a>) -> Option<Result<&Sequence<'a>, YamlParseError>> {
        self.get(key).map(YamlExt::try_as_vec)
    }

    fn get_mut_hash(&mut self, key: &Yaml<'a>) -> Option<Result<&mut Mapping<'a>, YamlParseError>> {
        self.get_mut(key).map(YamlExt::try_as_mut_hash)
    }

    fn get_mut_vec(&mut self, key: &Yaml<'a>) -> Option<Result<&mut Sequence<'a>, YamlParseError>> {
        self.get_mut(key).map(YamlExt::try_as_mut_vec)
    }
}


pub trait FromYaml: Sized {
    fn parse_from_yaml(yaml: &Yaml) -> Result<Self, YamlParseError>;

    fn parse_from_str(str: &str) -> Result<Self, YamlParseError> {
        let yaml = Yaml::load_from_str(str)?;
        Self::parse_from_yaml(&yaml[0])
    }

    fn parse_from_reader(reader: impl Read) -> Result<Self, YamlReadError> {
        let mut decoder = YamlDecoder::read(reader);
        decoder.encoding_trap(YAMLDecodingTrap::Strict);
        let yaml = decoder.decode();
        match yaml {
            Ok(y) => Self::parse_from_yaml(&y[0]).map_err(Into::into),
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
        Yaml::load_from_str(str)?
            .iter()
            .map(T::parse_from_yaml)
            .collect()
    }

    fn parse_from_reader(reader: impl Read) -> Result<Vec<T>, YamlReadError> {
        let mut decoder = YamlDecoder::read(reader);
        let yaml = decoder.encoding_trap(YAMLDecodingTrap::Strict).decode()?;

        yaml.iter()
            .map(T::parse_from_yaml)
            .map(|t| t.map_err(YamlReadError::ParseError))
            .collect()
    }

    fn parse_from_yaml(yaml: &Yaml) -> Result<Vec<T>, YamlParseError> {
        yaml.try_as_vec()?.iter().map(T::parse_from_yaml).collect()
    }

    fn to_yaml(&self) -> Result<Yaml, YamlWriteError> {
        Ok(Yaml::Sequence(
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
    UnknownVariant(&'static str, String),
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
                "SequenceOwned size mismatch when parsing yaml. SequenceOwned {name} expected \
                 length {expected} but got length {found}"
            ),
            YamlParseError::IoError(error) => Display::fmt(error, f),
            YamlParseError::ScanError(scan_error) => Display::fmt(scan_error, f),
            YamlParseError::Custom(str) => write!(f, "{str}"),
            YamlParseError::UnknownVariant(enum_name, found) => write!(
                f,
                "Found unknown enum variant \"{found}\" when parsing {enum_name}"
            ),
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
        Yaml::Value(Scalar::FloatingPoint(_)) => "f64",
        Yaml::Value(Scalar::Integer(_)) => "i64",
        Yaml::Value(Scalar::String(_)) => "String",
        Yaml::Value(Scalar::Boolean(_)) => "bool",
        Yaml::Sequence(_) => "Vec",
        Yaml::Mapping(_) => "Hash",
        Yaml::Alias(_) => "Alias",
        Yaml::Value(Scalar::Null) => "null",
        Yaml::BadValue => "BADVALUE",
        _ => "Unsupported",
    }
}

impl FromYaml for f64 {
    fn parse_from_yaml(yaml: &Yaml) -> Result<f64, YamlParseError> {
        yaml.try_as_f64()
    }

    fn to_yaml(&self) -> Result<Yaml, YamlWriteError> {
        Ok(Yaml::float(*self))
    }
}

impl FromYaml for f32 {
    fn parse_from_yaml(yaml: &Yaml) -> Result<Self, YamlParseError> {
        f64::parse_from_yaml(yaml).map(|d| d as f32)
    }

    fn to_yaml(&self) -> Result<Yaml, YamlWriteError> {
        Ok(Yaml::float(*self as f64))
    }
}

impl FromYaml for i64 {
    fn parse_from_yaml(yaml: &Yaml) -> Result<Self, YamlParseError> {
        yaml.try_as_i64()
    }

    fn to_yaml(&self) -> Result<Yaml, YamlWriteError> {
        Ok(Yaml::int(*self))
    }
}

impl FromYaml for String {
    fn parse_from_yaml(yaml: &Yaml) -> Result<Self, YamlParseError> {
        Ok(yaml.try_as_str()?.to_owned())
    }

    fn to_yaml(&self) -> Result<Yaml, YamlWriteError> {
        Ok(Yaml::string(self.clone()))
    }
}

impl FromYaml for bool {
    fn parse_from_yaml(yaml: &Yaml) -> Result<Self, YamlParseError> {
        yaml.try_as_bool()
    }

    fn to_yaml(&self) -> Result<Yaml, YamlWriteError> {
        Ok(Yaml::bool(*self))
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
        Ok(<saphyr::Yaml<'_> as YamlExt>::hash(
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
                .collect::<Result<Mapping, _>>()?,
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
        Ok(YamlExt::hash(
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
                .collect::<Result<Mapping, _>>()?,
        ))
    }
}

macro_rules! ints_yaml {
    ($($int: ty, $int_str: literal),*) => {
        $(
            impl FromYaml for $int {
                fn parse_from_yaml(yaml: &Yaml) -> Result<Self, YamlParseError> {
                    if let Ok(int) = yaml.try_as_i64()?.try_into() {
                        Ok(int)
                    } else {
                        Err(YamlParseError::custom(concat!("Could not fit the number parsed into an ", $int_str)))
                    }
                }

                fn to_yaml(&self) -> Result<Yaml, YamlWriteError> {
                    // Allow this, compiler should optimize out any unneeded branches
                    #[allow(irrefutable_let_patterns)]
                    if let Ok(int) = (*self).try_into() {
                        Ok(Yaml::int(int))
                    } else {
                        Err(YamlWriteError::custom(concat!("Could not fit an ", $int_str, " into a i64 when writing")))
                    }
                }
            }
        )*
    };
}

ints_yaml!(
    u8, "u8", u16, "u16", u32, "u32", u64, "u64", i8, "i8", i16, "i16", i32, "i32"
);
