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
    utils::binary::BinWriter,
};

// TODO: this would probably be better if we
// modified a writer directly instead of writing to a vec
//
// Leaving that for later though since it'll be somewhat hard
#[derive(Default)]
pub struct MapWriter {
    writer: BinWriter,
}

impl MapWriter {
    pub fn new() -> Self {
        MapWriter::default()
    }

    pub fn finish(self) -> Vec<u8> {
        self.writer.finish()
    }

    pub fn write_encoded_var(&mut self, var: &EncodedVar) {
        match var {
            EncodedVar::Bool(b) => {
                self.write_u8(0);
                self.write_bool(*b);
            }
            EncodedVar::Byte(b) => {
                self.write_u8(1);
                self.write_u8(*b);
            }
            EncodedVar::Short(s) => {
                self.write_u8(2);
                self.write_i16(*s);
            }
            EncodedVar::Int(i) => {
                self.write_u8(3);
                self.write_i32(*i);
            }
            EncodedVar::Float(f) => {
                self.write_u8(4);
                self.write_f32(*f);
            }
            EncodedVar::LookupIndex(i) => {
                self.write_u8(5);
                self.write_lookup_index(*i);
            }
            EncodedVar::String(s) => {
                self.write_u8(6);
                self.write_string(s);
            }
            EncodedVar::LengthEncodedString(s) => {
                self.write_u8(7);
                self.write_length_encoded_string(s);
            }
        }
    }

    pub fn write_lookup_table(&mut self, table: &LookupTable) {
        self.write_i16(table.lookup_strings.len() as i16);

        for str in &table.lookup_strings {
            self.write_string(str);
        }
    }

    pub fn write_lookup_index(&mut self, idx: LookupIndex) {
        self.write_u16(idx.0);
    }

    pub fn write_element(&mut self, element: &RawMapElement) -> Result<(), MapWriteError> {
        if let ResolvableString::LookupIndex(idx) = &element.name {
            self.write_lookup_index(*idx);
        } else {
            return Err(MapWriteError::ResolvedString);
        }

        self.write_u8(element.attributes.len() as u8);

        for attr in &element.attributes {
            self.write_attribute(attr)?;
        }

        self.write_i16(element.children.len() as i16);

        for child in &element.children {
            self.write_element(child)?;
        }

        Ok(())
    }

    pub fn write_attribute(&mut self, attr: &MapAttribute) -> Result<(), MapWriteError> {
        if let ResolvableString::LookupIndex(idx) = &attr.name {
            self.write_lookup_index(*idx);
        } else {
            return Err(MapWriteError::ResolvedString);
        }

        self.write_encoded_var(&attr.value);
        Ok(())
    }
}

impl Deref for MapWriter {
    type Target = BinWriter;

    fn deref(&self) -> &Self::Target {
        &self.writer
    }
}

impl DerefMut for MapWriter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.writer
    }
}

#[derive(Debug)]
pub enum MapWriteError {
    ResolvedString,
}

impl Display for MapWriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MapWriteError::ResolvedString => write!(
                f,
                "Tried to write map data with lookup strings still resolved"
            ),
        }
    }
}

impl Error for MapWriteError {}

impl From<MapWriteError> for std::io::Error {
    fn from(val: MapWriteError) -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::Other, Box::new(val))
    }
}
