use std::{error::Error, fmt::Display};

use crate::maps::{
    var_types::EncodedVar,
    LookupIndex,
    LookupTable,
    MapAttribute,
    RawMapElement,
    ResolvableString,
};

/// Helper to read data from map files
pub struct MapReader {
    map_data: Vec<u8>,
    cursor: usize,
}

impl MapReader {
    pub fn new(bytes: Vec<u8>) -> Self {
        MapReader {
            map_data: bytes,
            cursor: 0,
        }
    }

    pub fn read_byte(&mut self) -> Result<u8, MapReadError> {
        match self.map_data.get(self.cursor).copied() {
            Some(byte) => {
                self.cursor += 1;
                Ok(byte)
            }
            None => Err(MapReadError::EndOfBuffer),
        }
    }

    pub fn read_short(&mut self) -> Result<i16, MapReadError> {
        Ok(i16::from_le_bytes([self.read_byte()?, self.read_byte()?]))
    }

    pub fn read_int(&mut self) -> Result<i32, MapReadError> {
        Ok(i32::from_le_bytes([
            self.read_byte()?,
            self.read_byte()?,
            self.read_byte()?,
            self.read_byte()?,
        ]))
    }

    pub fn read_long(&mut self) -> Result<u64, MapReadError> {
        Ok(u64::from_le_bytes([
            self.read_byte()?,
            self.read_byte()?,
            self.read_byte()?,
            self.read_byte()?,
            self.read_byte()?,
            self.read_byte()?,
            self.read_byte()?,
            self.read_byte()?,
        ]))
    }

    /// Reads a `bool` from the map data.
    ///
    /// Only 0 or 1 are valid bools, anything else returns an error
    pub fn read_bool(&mut self) -> Result<bool, MapReadError> {
        let byte = self.read_byte()?;

        if byte > 1 {
            Err(MapReadError::InvalidBoolPattern(byte))
        } else {
            Ok(byte == 1)
        }
    }

    pub fn read_float(&mut self) -> Result<f32, MapReadError> {
        Ok(f32::from_bits(self.read_int()? as u32))
    }

    pub fn read_double(&mut self) -> Result<f64, MapReadError> {
        Ok(f64::from_bits(self.read_long()?))
    }

    pub fn read_char(&mut self) -> Result<char, MapReadError> {
        Ok(self.read_byte()? as char)
    }

    /// Reads a variable sized integer, maxing out at a `u32``
    pub fn read_varint(&mut self) -> Result<u32, MapReadError> {
        let mut result = 0;

        for shift in (0 .. 28).step_by(7) {
            let byte = self.read_byte()?;
            result |= ((byte as u32) & 0x7F) << shift;

            if byte <= 0x7F {
                return Ok(result);
            }
        }

        let byte = self.read_byte()?;
        if byte > 0b1111 {
            Err(MapReadError::InvalidVarint(result, byte))
        } else {
            result |= (byte as u32) << 28;
            Ok(result)
        }
    }

    /// Reads a string, reading a varint to indicate length
    pub fn read_string(&mut self) -> Result<String, MapReadError> {
        let length = self.read_varint()?;

        let mut buf = String::with_capacity(length as usize);

        for _ in 0 .. length {
            buf.push(self.read_char()?)
        }

        Ok(buf)
    }

    /// Reads a [run length encoded](https://en.wikipedia.org/wiki/Run-length_encoding) string, with a i16 indicating length
    pub fn read_length_encoded_string(&mut self) -> Result<String, MapReadError> {
        let bytes = self.read_short()?;

        // We know that the string length is at least half the byte count
        // so we can at least preallocate that
        let mut buf = String::with_capacity((bytes / 2) as usize);

        for _ in (0 .. bytes).step_by(2) {
            let repeat_count = self.read_byte()?;
            let char = self.read_char()?;

            for _ in 0 .. repeat_count {
                buf.push(char);
            }
        }

        Ok(buf)
    }

    pub fn read_encoded_var(&mut self) -> Result<EncodedVar, MapReadError> {
        let kind = self.read_byte()?;

        Ok(match kind {
            0 => EncodedVar::Bool(self.read_bool()?),
            1 => EncodedVar::Byte(self.read_byte()?),
            2 => EncodedVar::Short(self.read_short()?),
            3 => EncodedVar::Int(self.read_int()?),
            4 => EncodedVar::Float(self.read_float()?),
            5 => EncodedVar::LookupIndex(self.read_lookup_index()?),
            6 => EncodedVar::String(self.read_string()?),
            7 => EncodedVar::LengthEncodedString(self.read_length_encoded_string()?),
            _ => return Err(MapReadError::InvalidEncodedVarType(kind)),
        })
    }

    pub fn read_lookup_table(&mut self) -> Result<LookupTable, MapReadError> {
        let string_count = self.read_short()?;

        let mut buf = Vec::with_capacity(string_count as usize);

        for _ in 0 .. string_count {
            buf.push(self.read_string()?)
        }

        Ok(LookupTable::from_vec(buf))
    }

    pub fn read_lookup_index(&mut self) -> Result<LookupIndex, MapReadError> {
        Ok(LookupIndex(self.read_short()? as u16))
    }

    pub fn read_element(&mut self) -> Result<RawMapElement, MapReadError> {
        let name = ResolvableString::LookupIndex(self.read_lookup_index()?);
        let attr_count = self.read_byte()?;
        let mut attributes = Vec::with_capacity(attr_count as usize);

        for _ in 0 .. attr_count {
            attributes.push(self.read_attribute()?);
        }

        let child_count = self.read_short()?;
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

#[derive(Debug)]
pub enum MapReadError {
    EndOfBuffer,
    InvalidBoolPattern(u8),
    InvalidVarint(u32, u8),
    InvalidEncodedVarType(u8),
    InvalidHeader(String),
    IOError(std::io::Error),
}

impl Error for MapReadError {}

impl Display for MapReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            MapReadError::EndOfBuffer => "Map reader reached EOF while still reading".to_owned(),
            MapReadError::InvalidBoolPattern(bool) => format!("Improper bool patern found: {bool}"),
            MapReadError::InvalidVarint(..) => "Invalid variable-length integer found".to_owned(),
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
