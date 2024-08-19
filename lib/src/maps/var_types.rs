use std::{
    error::Error,
    fmt::{Debug, Display},
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
};

use crate::maps::{LookupIndex, LookupTable, ResolvableString};

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
            EncodedVar::LengthEncodedString(s) => s.clone(),
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

#[derive(Clone, Copy)]
pub enum Integer {
    U8(u8),
    I16(i16),
    I32(i32),
}

impl Debug for Integer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::U8(arg0) => write!(f, "{arg0}_u8"),
            Self::I16(arg0) => write!(f, "{arg0}_i16"),
            Self::I32(arg0) => write!(f, "{arg0}_i32"),
        }
    }
}

impl From<u8> for Integer {
    fn from(value: u8) -> Self {
        Integer::U8(value)
    }
}

impl From<i16> for Integer {
    fn from(value: i16) -> Self {
        Integer::I16(value)
    }
}

impl From<i32> for Integer {
    fn from(value: i32) -> Self {
        Integer::I32(value)
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
        }
    }
}

impl Add for Integer {
    type Output = Integer;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Integer::U8(b1), Integer::U8(b2)) => Integer::U8(b1 + b2),
            (Integer::U8(b1), Integer::I16(s2)) => Integer::I16(b1 as i16 + s2),
            (Integer::U8(b1), Integer::I32(i2)) => Integer::I32(b1 as i32 + i2),
            (Integer::I16(s1), Integer::U8(b2)) => Integer::I16(s1 + b2 as i16),
            (Integer::I16(s1), Integer::I16(s2)) => Integer::I16(s1 + s2),
            (Integer::I16(s1), Integer::I32(i2)) => Integer::I32(s1 as i32 + i2),
            (Integer::I32(i1), Integer::U8(b2)) => Integer::I32(i1 + b2 as i32),
            (Integer::I32(i1), Integer::I16(s2)) => Integer::I32(i1 + s2 as i32),
            (Integer::I32(i1), Integer::I32(i2)) => Integer::I32(i1 + i2),
        }
    }
}

impl AddAssign for Integer {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for Integer {
    type Output = Integer;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Integer::U8(b1), Integer::U8(b2)) => Integer::U8(b1 - b2),
            (Integer::U8(b1), Integer::I16(s2)) => Integer::I16(b1 as i16 - s2),
            (Integer::U8(b1), Integer::I32(i2)) => Integer::I32(b1 as i32 - i2),
            (Integer::I16(s1), Integer::U8(b2)) => Integer::I16(s1 - b2 as i16),
            (Integer::I16(s1), Integer::I16(s2)) => Integer::I16(s1 - s2),
            (Integer::I16(s1), Integer::I32(i2)) => Integer::I32(s1 as i32 - i2),
            (Integer::I32(i1), Integer::U8(b2)) => Integer::I32(i1 - b2 as i32),
            (Integer::I32(i1), Integer::I16(s2)) => Integer::I32(i1 - s2 as i32),
            (Integer::I32(i1), Integer::I32(i2)) => Integer::I32(i1 - i2),
        }
    }
}

impl SubAssign for Integer {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Mul for Integer {
    type Output = Integer;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Integer::U8(b1), Integer::U8(b2)) => Integer::U8(b1 * b2),
            (Integer::U8(b1), Integer::I16(s2)) => Integer::I16(b1 as i16 * s2),
            (Integer::U8(b1), Integer::I32(i2)) => Integer::I32(b1 as i32 * i2),
            (Integer::I16(s1), Integer::U8(b2)) => Integer::I16(s1 * b2 as i16),
            (Integer::I16(s1), Integer::I16(s2)) => Integer::I16(s1 * s2),
            (Integer::I16(s1), Integer::I32(i2)) => Integer::I32(s1 as i32 * i2),
            (Integer::I32(i1), Integer::U8(b2)) => Integer::I32(i1 * b2 as i32),
            (Integer::I32(i1), Integer::I16(s2)) => Integer::I32(i1 * s2 as i32),
            (Integer::I32(i1), Integer::I32(i2)) => Integer::I32(i1 * i2),
        }
    }
}

impl MulAssign for Integer {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl Div for Integer {
    type Output = Integer;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Integer::U8(b1), Integer::U8(b2)) => Integer::U8(b1 / b2),
            (Integer::U8(b1), Integer::I16(s2)) => Integer::I16(b1 as i16 / s2),
            (Integer::U8(b1), Integer::I32(i2)) => Integer::I32(b1 as i32 / i2),
            (Integer::I16(s1), Integer::U8(b2)) => Integer::I16(s1 / b2 as i16),
            (Integer::I16(s1), Integer::I16(s2)) => Integer::I16(s1 / s2),
            (Integer::I16(s1), Integer::I32(i2)) => Integer::I32(s1 as i32 / i2),
            (Integer::I32(i1), Integer::U8(b2)) => Integer::I32(i1 / b2 as i32),
            (Integer::I32(i1), Integer::I16(s2)) => Integer::I32(i1 / s2 as i32),
            (Integer::I32(i1), Integer::I32(i2)) => Integer::I32(i1 / i2),
        }
    }
}

impl DivAssign for Integer {
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}


#[derive(Clone, Copy)]
pub enum Float {
    U8(u8),
    I16(i16),
    I32(i32),
    F32(f32),
}

impl Float {
    pub fn as_f32(&self) -> f32 {
        match *self {
            Float::U8(b) => b as f32,
            Float::I16(s) => s as f32,
            Float::I32(i) => i as f32,
            Float::F32(f) => f,
        }
    }
}

impl Debug for Float {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::U8(arg0) => write!(f, "{arg0}_u8"),
            Self::I16(arg0) => write!(f, "{arg0}_i16"),
            Self::I32(arg0) => write!(f, "{arg0}_i32"),
            Self::F32(arg0) => write!(f, "{arg0}_f32"),
        }
    }
}

impl From<u8> for Float {
    fn from(value: u8) -> Self {
        Float::U8(value)
    }
}

impl From<i16> for Float {
    fn from(value: i16) -> Self {
        Float::I16(value)
    }
}

impl From<i32> for Float {
    fn from(value: i32) -> Self {
        Float::I32(value)
    }
}

impl From<Integer> for Float {
    fn from(value: Integer) -> Self {
        match value {
            Integer::U8(b) => Float::U8(b),
            Integer::I16(s) => Float::I16(s),
            Integer::I32(i) => Float::I32(i),
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
        }
    }
}

impl Add for Float {
    type Output = Float;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Float::U8(b1), Float::U8(b2)) => Float::U8(b1 + b2),
            (Float::U8(b1), Float::I16(s2)) => Float::I16(b1 as i16 + s2),
            (Float::U8(b1), Float::I32(i2)) => Float::I32(b1 as i32 + i2),
            (Float::U8(b1), Float::F32(f2)) => Float::F32(b1 as f32 + f2),
            (Float::I16(s1), Float::U8(b2)) => Float::I16(s1 + b2 as i16),
            (Float::I16(s1), Float::I16(s2)) => Float::I16(s1 + s2),
            (Float::I16(s1), Float::I32(i2)) => Float::I32(s1 as i32 + i2),
            (Float::I16(s1), Float::F32(f2)) => Float::F32(s1 as f32 + f2),
            (Float::I32(i1), Float::U8(b2)) => Float::I32(i1 + b2 as i32),
            (Float::I32(i1), Float::I16(s2)) => Float::I32(i1 + s2 as i32),
            (Float::I32(i1), Float::I32(i2)) => Float::I32(i1 + i2),
            (Float::I32(i1), Float::F32(f2)) => Float::F32(i1 as f32 + f2),
            (Float::F32(f1), Float::U8(b2)) => Float::F32(f1 + b2 as f32),
            (Float::F32(f1), Float::I16(s2)) => Float::F32(f1 + s2 as f32),
            (Float::F32(f1), Float::I32(i2)) => Float::F32(f1 + i2 as f32),
            (Float::F32(f1), Float::F32(f2)) => Float::F32(f1 + f2),
        }
    }
}

impl AddAssign for Float {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Add<Integer> for Float {
    type Output = Float;

    fn add(self, rhs: Integer) -> Self::Output {
        let rhs: Float = rhs.into();
        self.add(rhs)
    }
}

impl Add<Float> for Integer {
    type Output = Float;

    fn add(self, rhs: Float) -> Self::Output {
        let this: Float = self.into();
        this.add(rhs)
    }
}

impl AddAssign<Integer> for Float {
    fn add_assign(&mut self, rhs: Integer) {
        *self = *self + rhs;
    }
}


impl Sub for Float {
    type Output = Float;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Float::U8(b1), Float::U8(b2)) => Float::U8(b1 - b2),
            (Float::U8(b1), Float::I16(s2)) => Float::I16(b1 as i16 - s2),
            (Float::U8(b1), Float::I32(i2)) => Float::I32(b1 as i32 - i2),
            (Float::U8(b1), Float::F32(f2)) => Float::F32(b1 as f32 - f2),
            (Float::I16(s1), Float::U8(b2)) => Float::I16(s1 - b2 as i16),
            (Float::I16(s1), Float::I16(s2)) => Float::I16(s1 - s2),
            (Float::I16(s1), Float::I32(i2)) => Float::I32(s1 as i32 - i2),
            (Float::I16(s1), Float::F32(f2)) => Float::F32(s1 as f32 - f2),
            (Float::I32(i1), Float::U8(b2)) => Float::I32(i1 - b2 as i32),
            (Float::I32(i1), Float::I16(s2)) => Float::I32(i1 - s2 as i32),
            (Float::I32(i1), Float::I32(i2)) => Float::I32(i1 - i2),
            (Float::I32(i1), Float::F32(f2)) => Float::F32(i1 as f32 - f2),
            (Float::F32(f1), Float::U8(b2)) => Float::F32(f1 - b2 as f32),
            (Float::F32(f1), Float::I16(s2)) => Float::F32(f1 - s2 as f32),
            (Float::F32(f1), Float::I32(i2)) => Float::F32(f1 - i2 as f32),
            (Float::F32(f1), Float::F32(f2)) => Float::F32(f1 - f2),
        }
    }
}

impl SubAssign for Float {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}


impl Sub<Integer> for Float {
    type Output = Float;

    fn sub(self, rhs: Integer) -> Self::Output {
        let rhs: Float = rhs.into();
        self.sub(rhs)
    }
}

impl Sub<Float> for Integer {
    type Output = Float;

    fn sub(self, rhs: Float) -> Self::Output {
        let this: Float = self.into();
        this.sub(rhs)
    }
}

impl SubAssign<Integer> for Float {
    fn sub_assign(&mut self, rhs: Integer) {
        *self = *self - rhs;
    }
}


impl Mul for Float {
    type Output = Float;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Float::U8(b1), Float::U8(b2)) => Float::U8(b1 * b2),
            (Float::U8(b1), Float::I16(s2)) => Float::I16(b1 as i16 * s2),
            (Float::U8(b1), Float::I32(i2)) => Float::I32(b1 as i32 * i2),
            (Float::U8(b1), Float::F32(f2)) => Float::F32(b1 as f32 * f2),
            (Float::I16(s1), Float::U8(b2)) => Float::I16(s1 * b2 as i16),
            (Float::I16(s1), Float::I16(s2)) => Float::I16(s1 * s2),
            (Float::I16(s1), Float::I32(i2)) => Float::I32(s1 as i32 * i2),
            (Float::I16(s1), Float::F32(f2)) => Float::F32(s1 as f32 * f2),
            (Float::I32(i1), Float::U8(b2)) => Float::I32(i1 * b2 as i32),
            (Float::I32(i1), Float::I16(s2)) => Float::I32(i1 * s2 as i32),
            (Float::I32(i1), Float::I32(i2)) => Float::I32(i1 * i2),
            (Float::I32(i1), Float::F32(f2)) => Float::F32(i1 as f32 * f2),
            (Float::F32(f1), Float::U8(b2)) => Float::F32(f1 * b2 as f32),
            (Float::F32(f1), Float::I16(s2)) => Float::F32(f1 * s2 as f32),
            (Float::F32(f1), Float::I32(i2)) => Float::F32(f1 * i2 as f32),
            (Float::F32(f1), Float::F32(f2)) => Float::F32(f1 * f2),
        }
    }
}

impl MulAssign for Float {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}


impl Mul<Integer> for Float {
    type Output = Float;

    fn mul(self, rhs: Integer) -> Self::Output {
        let rhs: Float = rhs.into();
        self.mul(rhs)
    }
}

impl Mul<Float> for Integer {
    type Output = Float;

    fn mul(self, rhs: Float) -> Self::Output {
        let this: Float = self.into();
        this.mul(rhs)
    }
}

impl MulAssign<Integer> for Float {
    fn mul_assign(&mut self, rhs: Integer) {
        *self = *self * rhs;
    }
}


impl Div for Float {
    type Output = Float;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Float::U8(b1), Float::U8(b2)) => Float::U8(b1 / b2),
            (Float::U8(b1), Float::I16(s2)) => Float::I16(b1 as i16 / s2),
            (Float::U8(b1), Float::I32(i2)) => Float::I32(b1 as i32 / i2),
            (Float::U8(b1), Float::F32(f2)) => Float::F32(b1 as f32 / f2),
            (Float::I16(s1), Float::U8(b2)) => Float::I16(s1 / b2 as i16),
            (Float::I16(s1), Float::I16(s2)) => Float::I16(s1 / s2),
            (Float::I16(s1), Float::I32(i2)) => Float::I32(s1 as i32 / i2),
            (Float::I16(s1), Float::F32(f2)) => Float::F32(s1 as f32 / f2),
            (Float::I32(i1), Float::U8(b2)) => Float::I32(i1 / b2 as i32),
            (Float::I32(i1), Float::I16(s2)) => Float::I32(i1 / s2 as i32),
            (Float::I32(i1), Float::I32(i2)) => Float::I32(i1 / i2),
            (Float::I32(i1), Float::F32(f2)) => Float::F32(i1 as f32 / f2),
            (Float::F32(f1), Float::U8(b2)) => Float::F32(f1 / b2 as f32),
            (Float::F32(f1), Float::I16(s2)) => Float::F32(f1 / s2 as f32),
            (Float::F32(f1), Float::I32(i2)) => Float::F32(f1 / i2 as f32),
            (Float::F32(f1), Float::F32(f2)) => Float::F32(f1 / f2),
        }
    }
}

impl DivAssign for Float {
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}

impl Div<Integer> for Float {
    type Output = Float;

    fn div(self, rhs: Integer) -> Self::Output {
        let rhs: Float = rhs.into();
        self.div(rhs)
    }
}

impl Div<Float> for Integer {
    type Output = Float;

    fn div(self, rhs: Float) -> Self::Output {
        let this: Float = self.into();
        this.div(rhs)
    }
}

impl DivAssign<Integer> for Float {
    fn div_assign(&mut self, rhs: Integer) {
        *self = *self / rhs;
    }
}

#[derive(Clone)]
pub enum Character {
    String(ResolvableString),
    Byte(u8),
}

impl Character {
    /// Returns whether or not the [Character] is valid as a [char]
    pub fn verify(&self, lookup_table: &LookupTable) -> bool {
        match self {
            Character::String(s) => s.to_string(&lookup_table).len() == 1,
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

    /// Resolves the [ResolvableString] if the [Character] is [Character::String]
    pub fn resolve(&mut self, lookup_table: &LookupTable) {
        if let Character::String(str) = self {
            str.resolve(lookup_table)
        }
    }

    /// Unresolves the [ResolvableString] if the [Character] is [Character::String]
    pub fn unresolve(&mut self, lookup_table: &LookupTable) {
        if let Character::String(str) = self {
            str.to_index(lookup_table)
        }
    }

    /// Converts the [Character] to a [char] if it would be valid before resolution.
    ///
    /// If you have already called [Character::resolve] this is okay to use
    ///
    /// When compiling in debug mode, will return `None` if the string is more than one character long
    pub fn static_as_char(&self) -> Option<char> {
        match self {
            Character::String(s) => {
                #[cfg(debug_assertions)]
                if !self.static_verify() {
                    return None;
                }


                s.as_str().map(|str| str.chars().next()).flatten()
            }
            Character::Byte(b) => Some(*b as char),
        }
    }

    /// Converts the [Character] to a [char] if it would be valid.
    ///
    /// In release mode, this only returns `None` if the string is empty.
    ///
    /// When compiling in debug mode, will return `None` if the string is more than one character long

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
