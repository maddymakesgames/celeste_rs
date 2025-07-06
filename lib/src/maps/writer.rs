use std::{
    error::Error,
    fmt::Display,
    io::Write,
    ops::{Deref, DerefMut},
};

use crate::{
    maps::{
        LookupIndex,
        LookupTable,
        MapAttribute,
        RawMapElement,
        ResolvableString,
        var_types::EncodedVar,
    },
    utils::binary::BinWriter,
};

pub struct MapWriter<T: Write> {
    writer: BinWriter<T>,
}

impl<T: Write> MapWriter<T> {
    pub fn new(writer: T) -> Self {
        MapWriter {
            writer: BinWriter::new(writer),
        }
    }

    pub fn write_encoded_var(&mut self, var: &EncodedVar) -> Result<(), MapWriteError> {
        match var {
            EncodedVar::Bool(b) => {
                self.write_u8(0)?;
                self.write_bool(*b)?;
            }
            EncodedVar::Byte(b) => {
                self.write_u8(1)?;
                self.write_u8(*b)?;
            }
            EncodedVar::Short(s) => {
                self.write_u8(2)?;
                self.write_i16(*s)?;
            }
            EncodedVar::Int(i) => {
                self.write_u8(3)?;
                self.write_i32(*i)?;
            }
            EncodedVar::Float(f) => {
                self.write_u8(4)?;
                self.write_f32(*f)?;
            }
            EncodedVar::LookupIndex(i) => {
                self.write_u8(5)?;
                self.write_lookup_index(*i)?;
            }
            EncodedVar::String(s) => {
                self.write_u8(6)?;
                self.write_string(s)?;
            }
            EncodedVar::LengthEncodedString(s) => {
                self.write_u8(7)?;
                self.write_length_encoded_string(s)?;
            }
        }
        Ok(())
    }

    pub fn write_lookup_table(&mut self, table: &LookupTable) -> Result<(), MapWriteError> {
        self.write_i16(table.lookup_strings.len() as i16)?;

        for str in &table.lookup_strings {
            self.write_string(str)?;
        }

        Ok(())
    }

    pub fn write_lookup_index(&mut self, idx: LookupIndex) -> Result<(), MapWriteError> {
        self.write_u16(idx.0)?;
        Ok(())
    }

    pub fn write_element(&mut self, element: &RawMapElement) -> Result<(), MapWriteError> {
        match &element.name {
            ResolvableString::LookupIndex(idx) => {
                self.write_lookup_index(*idx)?;
            }
            ResolvableString::String(s) => return Err(MapWriteError::ResolvedString(s.clone())),
        }

        self.write_u8(element.attributes.len() as u8)?;

        for attr in &element.attributes {
            self.write_attribute(attr)?;
        }

        self.write_i16(element.children.len() as i16)?;

        for child in &element.children {
            self.write_element(child)?;
        }

        Ok(())
    }

    pub fn write_attribute(&mut self, attr: &MapAttribute) -> Result<(), MapWriteError> {
        match &attr.name {
            ResolvableString::LookupIndex(idx) => {
                self.write_lookup_index(*idx)?;
            }
            ResolvableString::String(s) => return Err(MapWriteError::ResolvedString(s.clone())),
        }

        self.write_encoded_var(&attr.value)
    }
}

impl<T: Write> Deref for MapWriter<T> {
    type Target = BinWriter<T>;

    fn deref(&self) -> &Self::Target {
        &self.writer
    }
}

impl<T: Write> DerefMut for MapWriter<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.writer
    }
}

#[derive(Debug)]
pub enum MapWriteError {
    ResolvedString(String),
    IoError(std::io::Error),
}

impl Display for MapWriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MapWriteError::ResolvedString(str) => write!(
                f,
                "Tried to write map data with lookup strings still resolved. Resolved string: \
                 \"{str}\""
            ),
            MapWriteError::IoError(e) => Display::fmt(e, f),
        }
    }
}

impl Error for MapWriteError {}

impl From<MapWriteError> for std::io::Error {
    fn from(val: MapWriteError) -> std::io::Error {
        match val {
            MapWriteError::IoError(e) => e,
            c => std::io::Error::other(Box::new(c)),
        }
    }
}

impl From<std::io::Error> for MapWriteError {
    fn from(value: std::io::Error) -> Self {
        MapWriteError::IoError(value)
    }
}
