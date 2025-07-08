use std::{
    error::Error,
    fmt::{Debug, Display},
};

use crate::{
    maps::{LookupIndex, LookupTable, ResolvableString},
    utils::num::{Float, Integer},
};

#[derive(Debug, Clone)]
pub enum EncodedVar {
    Bool(bool),
    Byte(u8),
    Short(i16),
    Int(i32),
    Float(f32),
    LookupIndex(LookupIndex),
    String(String),
    LengthEncodedString(String),
}

impl EncodedVar {
    pub fn to_string(&self, lookup_table: &LookupTable) -> String {
        match &self {
            EncodedVar::Bool(b) => b.to_string(),
            EncodedVar::Byte(b) => format!("{b}_u8"),
            EncodedVar::Short(s) => format!("{s}_i16"),
            EncodedVar::Int(i) => format!("{i}_i32"),
            EncodedVar::Float(f) => format!("{f}_f32"),
            EncodedVar::LookupIndex(i) => lookup_table[*i].clone(),
            EncodedVar::String(s) => s.clone(),
            EncodedVar::LengthEncodedString(s) => format!("RLE {s}"),
        }
    }

    pub fn kind(&self) -> &'static str {
        match &self {
            EncodedVar::Bool(_) => "bool",
            EncodedVar::Byte(_) => "byte",
            EncodedVar::Short(_) => "short",
            EncodedVar::Int(_) => "int",
            EncodedVar::Float(_) => "float",
            EncodedVar::LookupIndex(_) => "lookup index",
            EncodedVar::String(_) => "string",
            EncodedVar::LengthEncodedString(_) => "rle string",
        }
    }

    pub fn bool(&self) -> Result<bool, EncodedVarError> {
        if let EncodedVar::Bool(val) = self {
            Ok(*val)
        } else {
            Err(EncodedVarError::new("bool", self.kind()))
        }
    }

    pub fn u8(&self) -> Result<u8, EncodedVarError> {
        if let EncodedVar::Byte(val) = self {
            Ok(*val)
        } else {
            Err(EncodedVarError::new("u8", self.kind()))
        }
    }

    pub fn i16(&self) -> Result<i16, EncodedVarError> {
        if let EncodedVar::Short(val) = self {
            Ok(*val)
        } else {
            Err(EncodedVarError::new("i16", self.kind()))
        }
    }

    pub fn i32(&self) -> Result<i32, EncodedVarError> {
        if let EncodedVar::Int(val) = self {
            Ok(*val)
        } else {
            Err(EncodedVarError::new("i32", self.kind()))
        }
    }

    pub fn f32(&self) -> Result<f32, EncodedVarError> {
        if let EncodedVar::Float(val) = self {
            Ok(*val)
        } else {
            Err(EncodedVarError::new("f32", self.kind()))
        }
    }

    pub fn lookup_index(&self) -> Result<LookupIndex, EncodedVarError> {
        if let EncodedVar::LookupIndex(val) = self {
            Ok(*val)
        } else {
            Err(EncodedVarError::new("lookup index", self.kind()))
        }
    }

    pub fn string(&self) -> Result<String, EncodedVarError> {
        if let EncodedVar::String(val) | EncodedVar::LengthEncodedString(val) = self {
            Ok(val.clone())
        } else {
            Err(EncodedVarError::new("string", self.kind()))
        }
    }

    pub fn index_string(&self) -> Result<ResolvableString, EncodedVarError> {
        match self {
            EncodedVar::String(s) => Ok(ResolvableString::String(s.clone())),
            EncodedVar::LookupIndex(i) => Ok(ResolvableString::LookupIndex(*i)),
            _ => Err(EncodedVarError::new("indexed string", self.kind())),
        }
    }

    pub fn float(&self) -> Result<Float, EncodedVarError> {
        Ok(match self {
            EncodedVar::Byte(b) => Float::U8(*b),
            EncodedVar::Short(s) => Float::I16(*s),
            EncodedVar::Int(i) => Float::I32(*i),
            EncodedVar::Float(f) => Float::F32(*f),
            _ => return Err(EncodedVarError::new("float", self.kind())),
        })
    }

    pub fn int(&self) -> Result<Integer, EncodedVarError> {
        Ok(match self {
            EncodedVar::Byte(b) => Integer::U8(*b),
            EncodedVar::Short(s) => Integer::I16(*s),
            EncodedVar::Int(i) => Integer::I32(*i),
            _ => return Err(EncodedVarError::new("integer", self.kind())),
        })
    }

    pub fn char(&self) -> Result<Character, EncodedVarError> {
        Ok(match self {
            EncodedVar::Byte(b) => Character::Byte(*b),
            EncodedVar::String(s) => Character::String(ResolvableString::String(s.clone())),
            EncodedVar::LookupIndex(i) => Character::String(ResolvableString::LookupIndex(*i)),
            _ => return Err(EncodedVarError::new("character", self.kind())),
        })
    }

    pub fn new_rle_str(str: impl AsRef<str>) -> EncodedVar {
        EncodedVar::LengthEncodedString(str.as_ref().to_owned())
    }
}

#[derive(Debug)]
pub struct EncodedVarError {
    expected: &'static str,
    found: &'static str,
}

impl EncodedVarError {
    fn new(expected: &'static str, found: &'static str) -> Self {
        EncodedVarError { expected, found }
    }
}

impl Display for EncodedVarError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Error trying to parse encoded var into a type, wanted a '{}', found '{}'",
            self.expected, self.found
        )
    }
}

impl Error for EncodedVarError {}


impl From<u8> for EncodedVar {
    fn from(value: u8) -> Self {
        EncodedVar::Byte(value)
    }
}

impl<'a> TryFrom<&'a EncodedVar> for u8 {
    type Error = EncodedVarError;

    fn try_from(value: &'a EncodedVar) -> Result<Self, Self::Error> {
        value.u8()
    }
}

impl From<i16> for EncodedVar {
    fn from(value: i16) -> Self {
        EncodedVar::Short(value)
    }
}

impl<'a> TryFrom<&'a EncodedVar> for i16 {
    type Error = EncodedVarError;

    fn try_from(value: &'a EncodedVar) -> Result<Self, Self::Error> {
        value.i16()
    }
}

impl From<i32> for EncodedVar {
    fn from(value: i32) -> Self {
        EncodedVar::Int(value)
    }
}

impl<'a> TryFrom<&'a EncodedVar> for i32 {
    type Error = EncodedVarError;

    fn try_from(value: &'a EncodedVar) -> Result<Self, Self::Error> {
        value.i32()
    }
}

impl From<f32> for EncodedVar {
    fn from(value: f32) -> Self {
        EncodedVar::Float(value)
    }
}

impl<'a> TryFrom<&'a EncodedVar> for f32 {
    type Error = EncodedVarError;

    fn try_from(value: &'a EncodedVar) -> Result<Self, Self::Error> {
        value.f32()
    }
}

impl From<bool> for EncodedVar {
    fn from(value: bool) -> Self {
        EncodedVar::Bool(value)
    }
}

impl<'a> TryFrom<&'a EncodedVar> for bool {
    type Error = EncodedVarError;

    fn try_from(value: &'a EncodedVar) -> Result<Self, Self::Error> {
        value.bool()
    }
}

impl From<ResolvableString> for EncodedVar {
    fn from(value: ResolvableString) -> Self {
        match value {
            ResolvableString::LookupIndex(l) => EncodedVar::LookupIndex(l),
            ResolvableString::String(s) => EncodedVar::String(s),
        }
    }
}

impl From<String> for EncodedVar {
    fn from(value: String) -> Self {
        EncodedVar::String(value)
    }
}

impl<'a> TryFrom<&'a EncodedVar> for String {
    type Error = EncodedVarError;

    fn try_from(value: &'a EncodedVar) -> Result<Self, Self::Error> {
        value.string()
    }
}

impl<'a> TryFrom<&'a EncodedVar> for &'a str {
    type Error = EncodedVarError;

    fn try_from(value: &'a EncodedVar) -> Result<Self, Self::Error> {
        if let EncodedVar::String(val) | EncodedVar::LengthEncodedString(val) = value {
            Ok(val)
        } else {
            Err(EncodedVarError::new("string", value.kind()))
        }
    }
}

impl<'a> TryFrom<&'a EncodedVar> for ResolvableString {
    type Error = EncodedVarError;

    fn try_from(value: &'a EncodedVar) -> Result<Self, Self::Error> {
        value.index_string()
    }
}

#[derive(Clone)]
/// A character
///
/// Implementation wise it is possible for the `String` variant to not be length one.
/// Running [verify](Self::verify) will verify that the string is valid to be used as a byte.<br>
/// Realistically these should always be characters as long as the map was made correctly but
/// we wanted to offer a way to verify that the character was valid.
pub enum Character {
    String(ResolvableString),
    Byte(u8),
}

impl Character {
    /// Returns whether or not the [Character] is valid as a [char]
    pub fn verify(&self, lookup_table: &LookupTable) -> bool {
        match self {
            Character::String(s) => s.to_string(lookup_table).len() == 1,
            Character::Byte(_) => true,
        }
    }

    /// Returns whether or not the [Character] is valid as a [char] before resolution
    fn static_verify(&self) -> bool {
        match self {
            Character::String(s) => match s {
                ResolvableString::LookupIndex(_) => false,
                ResolvableString::String(s) => s.len() == 1,
            },
            Character::Byte(_) => true,
        }
    }

    /// Resolves the [ResolvableString] if the [Character] is a [String](Character::String)
    pub fn resolve(&mut self, lookup_table: &LookupTable) {
        if let Character::String(str) = self {
            str.resolve(lookup_table)
        }
    }

    /// Unresolves the [ResolvableString] if the [Character] is a [String](Character::String)
    pub fn unresolve(&mut self, lookup_table: &LookupTable) {
        if let Character::String(str) = self {
            str.to_index(lookup_table)
        }
    }

    /// Converts the [Character] to a [char] if it would be valid before resolution.
    ///
    /// If you have already called [Character::resolve] this is okay to use
    ///
    /// When compiling in debug mode, this will return `None`
    /// if the string is more than one character long
    pub fn static_as_char(&self) -> Option<char> {
        match self {
            Character::String(s) => {
                #[cfg(debug_assertions)]
                if !self.static_verify() {
                    return None;
                }


                s.as_str().and_then(|str| str.chars().next())
            }
            Character::Byte(b) => Some(*b as char),
        }
    }

    /// Converts the [Character] to a [char] if it would be valid.
    ///
    /// In release mode, this only returns `None` if the string is empty.
    ///
    /// When compiling in debug mode, this will also return `None`
    /// if the string is more than one character long
    pub fn as_char(&self, lookup_table: &LookupTable) -> Option<char> {
        match self {
            Character::String(s) => {
                let str = s.to_string(lookup_table);

                #[cfg(debug_assertions)]
                if str.len() != 1 {
                    return None;
                }

                str.chars().next()
            }
            Character::Byte(b) => Some(*b as char),
        }
    }
}

impl Debug for Character {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(arg0) => Debug::fmt(arg0, f),
            Self::Byte(arg0) => write!(f, "{arg0}_u8"),
        }
    }
}

impl From<u8> for Character {
    fn from(value: u8) -> Self {
        Self::Byte(value)
    }
}

impl From<LookupIndex> for Character {
    fn from(value: LookupIndex) -> Self {
        Self::String(ResolvableString::LookupIndex(value))
    }
}

impl From<String> for Character {
    fn from(value: String) -> Self {
        Self::String(ResolvableString::String(value))
    }
}

impl<'a> TryFrom<&'a EncodedVar> for Character {
    type Error = EncodedVarError;

    fn try_from(value: &'a EncodedVar) -> Result<Self, Self::Error> {
        value.char()
    }
}

impl From<Character> for EncodedVar {
    fn from(value: Character) -> Self {
        match value {
            Character::String(s) => EncodedVar::from(s),
            Character::Byte(b) => EncodedVar::Byte(b),
        }
    }
}

impl<'a> TryFrom<&'a EncodedVar> for Integer {
    type Error = EncodedVarError;

    fn try_from(value: &'a EncodedVar) -> Result<Self, Self::Error> {
        value.int()
    }
}

impl From<Integer> for EncodedVar {
    fn from(value: Integer) -> Self {
        match value {
            Integer::U8(b) => EncodedVar::Byte(b),
            Integer::I16(s) => EncodedVar::Short(s),
            Integer::I32(i) => EncodedVar::Int(i),
            // Truncate because map format doesn't support i64s
            Integer::I64(l) => EncodedVar::Int(l as i32),
        }
    }
}

impl<'a> TryFrom<&'a EncodedVar> for Float {
    type Error = EncodedVarError;

    fn try_from(value: &'a EncodedVar) -> Result<Self, Self::Error> {
        value.float()
    }
}

impl From<Float> for EncodedVar {
    fn from(value: Float) -> Self {
        match value {
            Float::U8(b) => EncodedVar::Byte(b),
            Float::I16(s) => EncodedVar::Short(s),
            Float::I32(i) => EncodedVar::Int(i),
            Float::F32(f) => EncodedVar::Float(f),
            // Truncate because map format doesn't support them
            Float::I64(l) => EncodedVar::Int(l as i32),
            Float::F64(d) => EncodedVar::Float(d as f32),
        }
    }
}
