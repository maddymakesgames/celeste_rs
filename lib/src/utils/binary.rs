use std::{error::Error, fmt::Display};

pub struct BinReader<'a> {
    contents: &'a [u8],
    cursor: usize,
}

impl<'a> BinReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        debug_assert!(!data.is_empty());
        BinReader {
            contents: data,
            cursor: 0,
        }
    }

    pub fn restart(&mut self) {
        self.cursor = 0;
    }

    pub fn seek(&mut self, position: usize) {
        debug_assert!(position < self.contents.len());
        self.cursor = position;
    }

    pub fn read_u8(&mut self) -> Result<u8, BinReadError> {
        match self.contents.get(self.cursor).copied() {
            Some(byte) => {
                self.cursor += 1;
                Ok(byte)
            }
            None => Err(BinReadError::EndOfBuffer),
        }
    }

    pub fn read_i8(&mut self) -> Result<i8, BinReadError> {
        Ok(self.read_u8()? as i8)
    }

    pub fn read_u16(&mut self) -> Result<u16, BinReadError> {
        Ok(u16::from_le_bytes([self.read_u8()?, self.read_u8()?]))
    }

    pub fn read_i16(&mut self) -> Result<i16, BinReadError> {
        Ok(self.read_u16()? as i16)
    }

    pub fn read_u32(&mut self) -> Result<u32, BinReadError> {
        Ok(u32::from_le_bytes([
            self.read_u8()?,
            self.read_u8()?,
            self.read_u8()?,
            self.read_u8()?,
        ]))
    }

    pub fn read_i32(&mut self) -> Result<i32, BinReadError> {
        Ok(self.read_u32()? as i32)
    }

    pub fn read_u64(&mut self) -> Result<u64, BinReadError> {
        Ok(u64::from_le_bytes([
            self.read_u8()?,
            self.read_u8()?,
            self.read_u8()?,
            self.read_u8()?,
            self.read_u8()?,
            self.read_u8()?,
            self.read_u8()?,
            self.read_u8()?,
        ]))
    }

    pub fn read_i64(&mut self) -> Result<i64, BinReadError> {
        Ok(self.read_u64()? as i64)
    }

    /// Reads a `bool` from the map data.
    ///
    /// Only 0 or 1 are valid bools, anything else returns an error
    pub fn read_bool(&mut self) -> Result<bool, BinReadError> {
        let byte = self.read_u8()?;

        if byte > 1 {
            Err(BinReadError::InvalidBoolPattern(byte))
        } else {
            Ok(byte == 1)
        }
    }

    pub fn read_float(&mut self) -> Result<f32, BinReadError> {
        Ok(f32::from_bits(self.read_u32()?))
    }

    pub fn read_double(&mut self) -> Result<f64, BinReadError> {
        Ok(f64::from_bits(self.read_u64()?))
    }

    pub fn read_char(&mut self) -> Result<char, BinReadError> {
        Ok(self.read_u8()? as char)
    }

    /// Reads a variable sized integer, maxing out at a `u32``
    pub fn read_varint(&mut self) -> Result<u32, BinReadError> {
        let mut result = 0u32;
        let mut by = 0;

        for i in 0 .. 5 {
            by = self.read_u8()?;
            result |= ((by & 0x7F) as u32) << (7 * i);

            if (by & 0x80) == 0 {
                return Ok(result);
            }
        }

        Err(BinReadError::InvalidVarint(result, by))
    }

    /// Reads a string, reading a varint to indicate length
    pub fn read_string(&mut self) -> Result<String, BinReadError> {
        let length = self.read_varint()?;

        let mut buf = String::with_capacity(length as usize);

        for _ in 0 .. length {
            buf.push(self.read_char()?)
        }

        Ok(buf)
    }

    /// Reads a [run length encoded](https://en.wikipedia.org/wiki/Run-length_encoding) string, with a i16 indicating length
    pub fn read_length_encoded_string(&mut self) -> Result<String, BinReadError> {
        let bytes = self.read_i16()?;

        // We know that the string length is at least half the byte count
        // so we can at least preallocate that
        let mut buf = String::with_capacity((bytes / 2) as usize);

        for _ in (0 .. bytes).step_by(2) {
            let repeat_count = self.read_u8()?;
            let char = self.read_char()?;

            for _ in 0 .. repeat_count {
                buf.push(char);
            }
        }

        Ok(buf)
    }
}

#[derive(Debug)]
pub enum BinReadError {
    EndOfBuffer,
    InvalidBoolPattern(u8),
    InvalidVarint(u32, u8),
}

impl Error for BinReadError {}

impl Display for BinReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            BinReadError::EndOfBuffer => "Map reader reached EOF while still reading".to_owned(),
            BinReadError::InvalidBoolPattern(bool) => format!("Improper bool patern found: {bool}"),
            BinReadError::InvalidVarint(..) => "Invalid variable-length integer found".to_owned(),
        })
    }
}

#[derive(Default)]
pub struct BinWriter {
    buf: Vec<u8>,
}

impl BinWriter {
    pub fn new() -> Self {
        BinWriter::default()
    }

    pub fn finish(self) -> Vec<u8> {
        self.buf
    }

    pub fn write_u8(&mut self, byte: u8) {
        self.buf.push(byte);
    }

    pub fn write_i8(&mut self, byte: i8) {
        self.buf.push(byte as u8);
    }

    pub fn write_u16(&mut self, short: u16) {
        self.buf.extend_from_slice(&short.to_le_bytes());
    }

    pub fn write_i16(&mut self, short: i16) {
        self.buf.extend_from_slice(&short.to_le_bytes());
    }

    pub fn write_u32(&mut self, int: u32) {
        self.buf.extend_from_slice(&int.to_le_bytes());
    }

    pub fn write_i32(&mut self, int: i32) {
        self.buf.extend_from_slice(&int.to_le_bytes());
    }

    pub fn write_u64(&mut self, long: u64) {
        self.buf.extend_from_slice(&long.to_le_bytes());
    }

    pub fn write_i64(&mut self, long: i64) {
        self.buf.extend_from_slice(&long.to_le_bytes());
    }

    pub fn write_bool(&mut self, bool: bool) {
        self.buf.push(bool as u8)
    }

    pub fn write_f32(&mut self, float: f32) {
        self.buf.extend_from_slice(&float.to_le_bytes());
    }

    pub fn write_f64(&mut self, double: f64) {
        self.buf.extend_from_slice(&double.to_le_bytes());
    }

    pub fn write_char(&mut self, char: char) {
        debug_assert!(char.is_ascii());
        // We only accept ascii chars so this is fine
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
            self.write_i16(0);
            return;
        }

        let mut char_iter = str.chars();

        let mut cur_char = char_iter.next().unwrap();
        let mut cur_char_count = 1;

        for c in char_iter {
            if c != cur_char || cur_char_count == 255 {
                buf.push(cur_char_count);
                buf.push(cur_char as u8);

                cur_char = c;
                cur_char_count = 1;
            } else {
                cur_char_count += 1;
            }
        }

        buf.push(cur_char_count);
        buf.push(cur_char as u8);

        self.write_i16(buf.len() as i16);
        self.buf.extend(&buf);
    }
}
