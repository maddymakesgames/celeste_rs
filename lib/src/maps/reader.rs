use std::{
    error::Error,
    fmt::Display,
    ops::{Deref, DerefMut},
};

use crate::{
    maps::{
        var_types::EncodedVar,
        LookupIndex,
        LookupTable,
        MapAttribute,
        RawMapElement,
        ResolvableString,
    },
    utils::binary::{BinReadError, BinReader},
};

/// Helper to read data from map files
pub struct MapReader<'a> {
    reader: BinReader<'a>,
}

impl<'a> MapReader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        MapReader {
            reader: BinReader::new(bytes),
        }
    }

    pub fn read_encoded_var(&mut self) -> Result<EncodedVar, MapReadError> {
        let kind = self.read_u8()?;

        Ok(match kind {
            0 => EncodedVar::Bool(self.read_bool()?),
            1 => EncodedVar::Byte(self.read_u8()?),
            2 => EncodedVar::Short(self.read_i16()?),
            3 => EncodedVar::Int(self.read_i32()?),
            4 => EncodedVar::Float(self.read_float()?),
            5 => EncodedVar::LookupIndex(self.read_lookup_index()?),
            6 => EncodedVar::String(self.read_string()?),
            7 => EncodedVar::LengthEncodedString(self.read_length_encoded_string()?),
            _ => return Err(MapReadError::InvalidEncodedVarType(kind)),
        })
    }

    pub fn read_lookup_table(&mut self) -> Result<LookupTable, MapReadError> {
        let string_count = self.read_u16()?;

        let mut buf = Vec::with_capacity(string_count as usize);

        for _ in 0 .. string_count {
            buf.push(self.read_string()?)
        }

        Ok(LookupTable::from_vec(buf))
    }

    pub fn read_lookup_index(&mut self) -> Result<LookupIndex, MapReadError> {
        Ok(LookupIndex(self.read_u16()?))
    }

    pub fn read_element(&mut self) -> Result<RawMapElement, MapReadError> {
        let name = ResolvableString::LookupIndex(self.read_lookup_index()?);
        let attr_count = self.read_u8()?;
        let mut attributes = Vec::with_capacity(attr_count as usize);

        for _ in 0 .. attr_count {
            attributes.push(self.read_attribute()?);
        }

        let child_count = self.read_u16()?;
        let mut children = Vec::with_capacity(child_count as usize);

        for _ in 0 .. child_count {
            children.push(self.read_element()?);
        }

        Ok(RawMapElement {
            name,
            attributes,
            children,
        })
    }

    pub fn read_attribute(&mut self) -> Result<MapAttribute, MapReadError> {
        let name = ResolvableString::LookupIndex(self.read_lookup_index()?);
        let value = self.read_encoded_var()?;

        Ok(MapAttribute { name, value })
    }
}

impl<'a> Deref for MapReader<'a> {
    type Target = BinReader<'a>;

    fn deref(&self) -> &Self::Target {
        &self.reader
    }
}

impl<'a> DerefMut for MapReader<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.reader
    }
}

#[derive(Debug)]
pub enum MapReadError {
    InvalidEncodedVarType(u8),
    InvalidHeader(String),
    IOError(std::io::Error),
    BinError(BinReadError),
}

impl Error for MapReadError {}

impl Display for MapReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            MapReadError::BinError(e) => e.to_string(),
            MapReadError::InvalidEncodedVarType(kind) =>
                format!("Invalid encoded var type found: {kind}"),
            MapReadError::InvalidHeader(h) =>
                format!("Invalid file header found, expected \"CELESTE MAP\", found \"{h}\""),
            MapReadError::IOError(e) => e.to_string(),
        })
    }
}

impl From<std::io::Error> for MapReadError {
    fn from(value: std::io::Error) -> Self {
        Self::IOError(value)
    }
}

impl From<BinReadError> for MapReadError {
    fn from(value: BinReadError) -> Self {
        Self::BinError(value)
    }
}
