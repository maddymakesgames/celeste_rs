use std::{
    any::Any,
    collections::HashMap,
    fmt::{Debug, Display},
    io::Read,
};

pub mod elements;
pub mod encoder;
pub mod lookup;
pub mod parser;
pub mod reader;
pub mod var_types;
use elements::*;
pub use lookup::*;

use crate::maps::{
    encoder::MapEncoder,
    entities::{Entity, MapEntity},
    parser::{ElementParser, ElementParserImpl, MapElementParsingError, MapParser},
    reader::{MapReadError, MapReader},
    var_types::EncodedVar,
};

#[derive(Debug, Clone)]
pub struct RawMapElement {
    pub name: ResolvableString,
    pub attributes: Vec<MapAttribute>,
    pub children: Vec<RawMapElement>,
}

#[derive(Debug, Clone)]
pub struct MapAttribute {
    pub name: ResolvableString,
    pub value: EncodedVar,
}

impl MapAttribute {
    pub fn new(name: ResolvableString, value: impl Into<EncodedVar>) -> Self {
        MapAttribute {
            name,
            value: value.into(),
        }
    }
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

    fn resolve_strings(&mut self) {
        self.root_element.resolve_strings(&self.lookup_table);
    }

    fn unresolve_strings(&mut self) {
        self.root_element
            .add_attr_value_strs(&mut self.lookup_table);
        self.root_element.unresolve_strings(&mut self.lookup_table);
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

        buf.push_str(self.name.to_string(lookup_table));
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

    fn resolve_strings(&mut self, lookup_table: &LookupTable) {
        self.name.resolve(lookup_table);

        for attr in &mut self.attributes {
            attr.name.resolve(lookup_table);

            if let EncodedVar::LookupIndex(i) = attr.value {
                attr.value = lookup_table[i].clone().into()
            }
        }

        for child in &mut self.children {
            child.resolve_strings(lookup_table)
        }
    }

    fn add_attr_value_strs(&self, lookup_table: &mut LookupTable) {
        for attr in &self.attributes {
            if let EncodedVar::String(str) = &attr.value {
                lookup_table.add_string(str);
            }
        }

        for child in &self.children {
            child.add_attr_value_strs(lookup_table);
        }
    }

    fn unresolve_strings(&mut self, lookup_table: &mut LookupTable) {
        self.name.to_index(lookup_table);

        for attr in &mut self.attributes {
            attr.name.to_index(lookup_table);

            if let EncodedVar::String(str) = &attr.value {
                if let Some(idx) = lookup_table.lookup_string(str) {
                    attr.value = EncodedVar::LookupIndex(idx);
                }
            }
        }

        for child in &mut self.children {
            child.unresolve_strings(lookup_table);
        }
    }
}

impl MapAttribute {
    pub fn to_string(&self, lookup_table: &LookupTable) -> String {
        format!(
            "{}: {}",
            self.name.to_string(lookup_table),
            self.value.to_string(lookup_table)
        )
    }
}

pub trait MapElement: Any + Debug {
    const NAME: &'static str;

    fn from_raw(parser: MapParser) -> Result<Self, MapElementParsingError>
    where Self: Sized;
    fn to_raw(&self, encoder: &mut MapEncoder);
}

pub trait ErasedMapElement: Any + Debug {
    fn name(&self) -> &'static str;
    fn from_raw(parser: MapParser) -> Result<Self, MapElementParsingError>
    where Self: Sized;
    fn to_raw(&self, encoder: &mut MapEncoder);
}

impl<T: MapElement> ErasedMapElement for T {
    fn name(&self) -> &'static str {
        T::NAME
    }

    fn from_raw(parser: MapParser) -> Result<Self, MapElementParsingError>
    where Self: Sized {
        T::from_raw(parser)
    }

    fn to_raw(&self, encoder: &mut MapEncoder) {
        self.to_raw(encoder)
    }
}

pub type DynMapElement = Box<dyn ErasedMapElement>;

impl ErasedMapElement for RawMapElement {
    fn name(&self) -> &'static str {
        ""
    }

    fn from_raw(parser: MapParser) -> Result<Self, MapElementParsingError>
    where Self: Sized {
        Ok(parser.raw.clone())
    }

    fn to_raw(&self, encoder: &mut MapEncoder) {
        encoder.from_raw(self)
    }
}

pub struct MapManager {
    map: RawMap,
    parsers: HashMap<&'static str, Box<dyn ElementParserImpl>>,
}

impl MapManager {
    pub fn new(mut reader: impl Read) -> Result<Self, MapReadError> {
        let mut buf = Vec::new();

        reader.read_to_end(&mut buf)?;

        let mut raw = RawMap::from_bytes(buf)?;

        let parsers = HashMap::new();

        raw.resolve_strings();

        Ok(MapManager { map: raw, parsers })
    }

    pub fn parse_map(&self) -> Result<MapRoot, MapElementParsingError> {
        let parser = MapParser {
            verbose_debug: false,
            lookup: &self.map.lookup_table,
            raw: &self.map.root_element,
            parsers: &self.parsers,
        };

        parser.parse_self::<MapRoot>()
    }

    pub fn verbose_parse(&self) -> Result<MapRoot, MapElementParsingError> {
        let parser = MapParser {
            verbose_debug: cfg!(debug_assertions),
            lookup: &self.map.lookup_table,
            raw: &self.map.root_element,
            parsers: &self.parsers,
        };

        parser.parse_self::<MapRoot>()
    }

    pub fn encode_map(&mut self, name: impl ToString, root: &MapRoot) {
        let mut lookup = LookupTable::new();

        let root_name = lookup.index_string(MapRoot::NAME);

        let mut encoder = MapEncoder {
            lookup: &mut lookup,
            element_name: root_name,
            children: Vec::new(),
            attrs: Vec::new(),
        };

        MapElement::to_raw(root, &mut encoder);

        self.map.root_element = encoder.resolve();
        self.map.name = name.to_string();
        self.map.lookup_table = lookup;

        self.map.unresolve_strings();
    }

    pub fn add_parser<T: MapElement>(&mut self) {
        self.parsers
            .insert(T::NAME, Box::new(ElementParser::<T>::new()));
    }

    pub fn add_entity_parser<T: Entity>(&mut self) {
        self.add_parser::<MapEntity<T>>();
    }

    pub fn map(&self) -> &RawMap {
        &self.map
    }
}
