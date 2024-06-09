use std::{error::Error, fmt::Display, ops::Index};

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

    pub fn read_u16(&mut self) -> Result<u16, MapReadError> {
        Ok((self.read_byte()? as u16) | ((self.read_byte()? as u16) << 8))
    }

    pub fn read_u32(&mut self) -> Result<u32, MapReadError> {
        Ok((self.read_byte()? as u32)
            | (self.read_byte()? as u32) << 8
            | (self.read_byte()? as u32) << 16
            | (self.read_byte()? as u32) << 24)
    }

    pub fn read_u64(&mut self) -> Result<u64, MapReadError> {
        Ok((self.read_byte()? as u64)
            | (self.read_byte()? as u64) << 8
            | (self.read_byte()? as u64) << 16
            | (self.read_byte()? as u64) << 24
            | (self.read_byte()? as u64) << 32
            | (self.read_byte()? as u64) << 40
            | (self.read_byte()? as u64) << 48
            | (self.read_byte()? as u64) << 56)
    }

    pub fn read_bool(&mut self) -> Result<bool, MapReadError> {
        let byte = self.read_byte()?;

        if byte > 1 {
            Err(MapReadError::InvalidBoolPattern(byte))
        } else {
            Ok(byte == 1)
        }
    }

    pub fn read_f32(&mut self) -> Result<f32, MapReadError> {
        Ok(f32::from_bits(self.read_u32()?))
    }

    pub fn read_f64(&mut self) -> Result<f64, MapReadError> {
        Ok(f64::from_bits(self.read_u64()?))
    }

    pub fn read_char(&mut self) -> Result<char, MapReadError> {
        Ok(self.read_byte()? as char)
    }

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

    pub fn read_string(&mut self) -> Result<String, MapReadError> {
        let length = self.read_varint()?;

        let mut buf = String::with_capacity(length as usize);

        for _ in 0 .. length {
            buf.push(self.read_char()?)
        }

        Ok(buf)
    }

    pub fn read_length_encoded_string(&mut self) -> Result<String, MapReadError> {
        let bytes = self.read_u16()?;

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
            2 => EncodedVar::Short(self.read_u16()?),
            3 => EncodedVar::Int(self.read_u32()?),
            4 => EncodedVar::Float(self.read_f32()?),
            5 => EncodedVar::LookupIndex(self.read_lookup_index()?),
            6 => EncodedVar::String(self.read_string()?),
            7 => EncodedVar::LengthEncodedString(self.read_length_encoded_string()?),
            8 => EncodedVar::Long(self.read_u64()?),
            9 => EncodedVar::Double(self.read_f64()?),
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
        let name = self.read_lookup_index()?;
        let attr_count = self.read_byte()?;
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
        let name = self.read_lookup_index()?;
        let value = self.read_encoded_var()?;

        Ok(MapAttribute { name, value })
    }
}

#[derive(Debug)]
pub enum EncodedVar {
    Bool(bool),
    Byte(u8),
    Short(u16),
    Int(u32),
    Float(f32),
    LookupIndex(LookupIndex),
    String(String),
    LengthEncodedString(String),
    Long(u64),
    Double(f64),
}

impl EncodedVar {
    fn to_string(&self, lookup_table: &LookupTable) -> String {
        match &self {
            EncodedVar::Bool(b) => b.to_string(),
            EncodedVar::Byte(b) => b.to_string(),
            EncodedVar::Short(s) => s.to_string(),
            EncodedVar::Int(i) => i.to_string(),
            EncodedVar::Float(f) => f.to_string(),
            EncodedVar::LookupIndex(i) => lookup_table[*i].clone(),
            EncodedVar::String(s) => s.clone(),
            EncodedVar::LengthEncodedString(s) => s.clone(),
            EncodedVar::Long(l) => l.to_string(),
            EncodedVar::Double(d) => d.to_string(),
        }
    }
}

#[derive(Debug)]
pub enum MapReadError {
    EndOfBuffer,
    InvalidBoolPattern(u8),
    InvalidVarint(u32, u8),
    InvalidEncodedVarType(u8),
    InvalidHeader(String),
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
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LookupIndex(u16);

#[derive(Debug)]
pub struct LookupTable {
    lookup_strings: Vec<String>,
    strings_to_add: Vec<String>,
}

impl LookupTable {
    pub fn new() -> LookupTable {
        LookupTable {
            lookup_strings: Vec::new(),
            strings_to_add: Vec::new(),
        }
    }

    pub fn from_vec(vec: Vec<String>) -> LookupTable {
        LookupTable {
            lookup_strings: vec,
            strings_to_add: Vec::new(),
        }
    }

    pub fn lookup_contains(&self, str: impl AsRef<str>) -> bool {
        let str = str.as_ref();
        self.lookup_strings.binary_search_by(|a| str.cmp(a)).is_ok()
    }

    pub fn add_string(&mut self, str: impl AsRef<str>) {
        let str = str.as_ref();
        if !self.lookup_contains(str) {
            if let Some(index) = self.strings_to_add.iter().position(|s| s == str) {
                if let Err(idx) = self.lookup_strings.binary_search_by(|a| str.cmp(a)) {
                    let str = self.strings_to_add.swap_remove(index);
                    self.lookup_strings.insert(idx, str);
                }
            } else {
                self.strings_to_add.push(str.to_owned());
            }
        }
    }

    /// Resolves a string to it's lookup position if it exists
    ///
    /// Any indecies returned from this become invalid if [add_string](Self::add_string) is run again
    pub fn resolve_string(&self, str: impl AsRef<str>) -> Option<LookupIndex> {
        let str = str.as_ref();
        if self.lookup_contains(str) {
            self.lookup_strings
                .iter()
                .position(|s| s == str)
                .map(|i| LookupIndex(i as u16))
        } else {
            None
        }
    }

    fn to_string(&self, depth: u8) -> String {
        let mut buf = String::new();
        buf.push('[');

        for str in &self.lookup_strings {
            buf.push('\n');
            for _ in 0 .. depth + 1 {
                buf.push('\t');
            }
            buf.push('"');
            buf.push_str(str);
            buf.push('"');
        }

        if !self.lookup_strings.is_empty() {
            buf.push('\n');
            for _ in 0 .. depth {
                buf.push('\t');
            }
        }

        buf.push(']');

        buf
    }

    pub fn get(&self, index: LookupIndex) -> Option<&String> {
        self.lookup_strings.get(index.0 as usize)
    }
}

impl Index<LookupIndex> for LookupTable {
    type Output = String;

    fn index(&self, index: LookupIndex) -> &Self::Output {
        &self.lookup_strings[index.0 as usize]
    }
}

impl Default for LookupTable {
    fn default() -> Self {
        Self::new()
    }
}


#[derive(Debug)]
pub struct RawMapElement {
    pub name: LookupIndex,
    pub attributes: Vec<MapAttribute>,
    pub children: Vec<RawMapElement>,
}

#[derive(Debug)]
pub struct MapAttribute {
    pub name: LookupIndex,
    pub value: EncodedVar,
}

#[derive(Debug)]
pub struct RawMap {
    pub name: String,
    pub lookup_table: LookupTable,
    pub root_element: RawMapElement,
}

impl RawMap {
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, MapReadError> {
        let mut reader = MapReader::new(bytes);

        let check_string = reader.read_string()?;

        if check_string != "CELESTE MAP" {
            return Err(MapReadError::InvalidHeader(check_string));
        }

        let name = reader.read_string()?;
        let lookup_table = reader.read_lookup_table()?;
        let root_element = reader.read_element()?;

        Ok(Self {
            name,
            lookup_table,
            root_element,
        })
    }
}

impl Display for RawMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {{\n\tlookup_table: {}\n\troot_element: {}\n}}",
            &self.name,
            self.lookup_table.to_string(2),
            self.root_element.to_string(2, &self.lookup_table)
        )
    }
}

impl RawMapElement {
    pub fn to_string(&self, depth: u8, lookup_table: &LookupTable) -> String {
        let mut buf = String::new();

        buf.push_str(&lookup_table[self.name]);
        buf.push_str(" {\n");

        for _ in 0 .. depth {
            buf.push('\t');
        }

        buf.push_str("attributes: [");

        for attr in &self.attributes {
            buf.push('\n');
            for _ in 0 .. depth + 1 {
                buf.push('\t');
            }
            buf.push_str(&attr.to_string(lookup_table));
        }

        if !self.attributes.is_empty() {
            buf.push('\n');
            for _ in 0 .. depth {
                buf.push('\t');
            }
        }

        buf.push_str("],\n");
        for _ in 0 .. depth {
            buf.push('\t');
        }

        buf.push_str("children: [");

        for child in &self.children {
            buf.push('\n');
            for _ in 0 .. depth {
                buf.push('\t');
            }

            buf.push_str(&child.to_string(depth + 1, lookup_table));
        }

        if !self.children.is_empty() {
            buf.push('\n');
            for _ in 0 .. depth {
                buf.push('\t');
            }
        }

        buf.push_str("]\n");
        for _ in 0 .. depth - 1 {
            buf.push('\t');
        }
        buf.push('}');

        buf
    }
}

impl MapAttribute {
    pub fn to_string(&self, lookup_table: &LookupTable) -> String {
        format!(
            "{}: {}",
            lookup_table[self.name],
            self.value.to_string(lookup_table)
        )
    }
}
