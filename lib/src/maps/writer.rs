use std::{error::Error, fmt::Display};

use crate::maps::{
    var_types::EncodedVar,
    LookupIndex,
    LookupTable,
    MapAttribute,
    RawMapElement,
    ResolvableString,
};

// TODO: this would probably be better if we
// modified a writer directly instead of writing to a vec
//
// Leaving that for later though since it'll be somewhat hard
#[derive(Default)]
pub struct MapWriter {
    buf: Vec<u8>,
}

impl MapWriter {
    pub fn new() -> Self {
        MapWriter::default()
    }

    pub fn write_byte(&mut self, byte: u8) {
        self.buf.push(byte);
    }

    pub fn write_short(&mut self, short: i16) {
        self.buf.extend_from_slice(&short.to_le_bytes());
    }

    pub fn write_int(&mut self, int: i32) {
        self.buf.extend_from_slice(&int.to_le_bytes());
    }

    pub fn write_long(&mut self, long: i64) {
        self.buf.extend_from_slice(&long.to_le_bytes());
    }

    pub fn write_bool(&mut self, bool: bool) {
        self.buf.push(bool as u8)
    }

    pub fn write_float(&mut self, float: f32) {
        self.buf.extend_from_slice(&float.to_le_bytes());
    }

    pub fn write_double(&mut self, double: f64) {
        self.buf.extend_from_slice(&double.to_le_bytes());
    }

    pub fn write_char(&mut self, char: char) {
        // Char can only be ascii so this works
        self.buf.push(char as u8);
    }

    pub fn write_varint(&mut self, mut int: u32) {
        let mut next_byte: u8;
        let mut buf = [0u8; 5];
        let mut i = 0;

        while i < 5 {
            next_byte = (int & 0x7F) as u8;
            int >>= 7;
            if int != 0 {
                next_byte |= 0x80;
            }
            buf[i] = next_byte;
            i += 1;

            if int == 0 {
                break;
            }
        }

        self.buf.extend_from_slice(&buf[.. i]);
    }

    pub fn write_string(&mut self, str: &str) {
        self.write_varint(str.len() as u32);

        for c in str.chars() {
            self.write_char(c);
        }
    }

    pub fn write_length_encoded_string(&mut self, str: &str) {
        let mut buf = Vec::with_capacity(str.len());

        if str.is_empty() {
            self.write_short(0);
            return;
        }

        let mut char_iter = str.chars();

        let mut cur_char = char_iter.next().unwrap();
        let mut cur_char_count = 1;

        for c in str.chars() {
            if c != cur_char || cur_char_count == 255 {
                buf.push(cur_char_count);
                buf.push(c as u8);

                cur_char = c;
                cur_char_count = 1;
            } else {
                cur_char_count += 1;
            }
        }

        self.write_short(buf.len() as i16);
        self.buf.extend(&buf);
    }

    pub fn write_encoded_var(&mut self, var: &EncodedVar) {
        match var {
            EncodedVar::Bool(b) => {
                self.write_byte(0);
                self.write_bool(*b);
            }
            EncodedVar::Byte(b) => {
                self.write_byte(1);
                self.write_byte(*b);
            }
            EncodedVar::Short(s) => {
                self.write_byte(2);
                self.write_short(*s);
            }
            EncodedVar::Int(i) => {
                self.write_byte(3);
                self.write_int(*i);
            }
            EncodedVar::Float(f) => {
                self.write_byte(4);
                self.write_float(*f);
            }
            EncodedVar::LookupIndex(i) => {
                self.write_byte(5);
                self.write_lookup_index(*i);
            }
            EncodedVar::String(s) => {
                self.write_byte(6);
                self.write_string(s);
            }
            EncodedVar::LengthEncodedString(s) => {
                self.write_byte(7);
                self.write_length_encoded_string(s);
            }
        }
    }

    pub fn write_lookup_table(&mut self, table: &LookupTable) {
        self.write_short(table.lookup_strings.len() as i16);

        for str in &table.lookup_strings {
            self.write_string(str);
        }
    }

    pub fn write_lookup_index(&mut self, idx: LookupIndex) {
        self.buf.extend_from_slice(&idx.0.to_le_bytes());
    }

    pub fn write_element(&mut self, element: &RawMapElement) -> Result<(), MapWriteError> {
        if let ResolvableString::LookupIndex(idx) = &element.name {
            self.write_lookup_index(*idx);
        } else {
            return Err(MapWriteError::ResolvedString);
        }

        self.write_byte(element.attributes.len() as u8);

        for attr in &element.attributes {
            self.write_attribute(attr)?;
        }

        self.write_short(element.children.len() as i16);

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

    pub fn bytes(self) -> Vec<u8> {
        self.buf
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
