//! Implements reading and writing of Celeste's map format
//! along with providing helper structs for all the map elements seen in the vanilla game
use std::{
    any::Any,
    collections::HashMap,
    fmt::{Debug, Display},
    io::{Read, Write},
};

pub mod elements;
pub mod encoder;
pub mod lookup;
pub mod parser;
pub mod reader;
pub mod var_types;
pub mod writer;
use elements::*;
pub use lookup::*;

use crate::maps::{
    encoder::MapEncoder,
    entities::{Entity, MapEntity},
    parser::{ElementParser, ElementParserImpl, MapElementParsingError, MapParser},
    reader::{MapReadError, MapReader},
    triggers::{MapTrigger, Trigger},
    var_types::EncodedVar,
    writer::{MapWriteError, MapWriter},
};

#[derive(Debug, Clone)]
/// The raw structure of an element in the map binary
///
/// All elements get parsed to this before being parsed into proper structs.
pub struct RawMapElement {
    pub name: ResolvableString,
    pub attributes: Vec<MapAttribute>,
    pub children: Vec<RawMapElement>,
}

#[derive(Debug, Clone)]
/// An attribute attached to a map element
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
/// The raw format of the map binary, is parsed into a [MapRoot]
pub struct RawMap {
    pub name: String,
    pub lookup_table: LookupTable,
    pub root_element: RawMapElement,
}

impl RawMap {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, MapReadError> {
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

    fn write<T: Write>(&self, writer: &mut MapWriter<T>) -> Result<(), MapWriteError> {
        writer.write_string("CELESTE MAP")?;

        writer.write_string(&self.name)?;
        writer.write_lookup_table(&self.lookup_table)?;
        writer.write_element(&self.root_element)?;
        Ok(())
    }

    fn to_bytes(&self) -> Result<Vec<u8>, MapWriteError> {
        let mut buf = Vec::new();
        self.to_writer(&mut buf)?;

        Ok(buf)
    }

    fn to_writer(&self, writer: impl Write) -> Result<(), MapWriteError> {
        let mut map_writer = MapWriter::new(writer);
        self.write(&mut map_writer)
    }

    /// Resolve all the [ResolvableString]s stored in the map
    ///
    /// This should be called directly after reading in the map file,
    /// as modifications can change the lookup table without updating the indicies.
    ///
    /// Note: not needed if using a [MapManager]
    pub fn resolve_strings(&mut self) {
        self.root_element.resolve_strings(&self.lookup_table);
    }

    /// Converts all the [ResolvableString]s to indicies in the lookup table.
    ///
    /// This should be done only right before you serialize.
    /// The indicies change on pretty much any change to the map data.
    ///
    /// Note: not needed if using a [MapManager]
    pub fn unresolve_strings(&mut self) {
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
    fn to_string(&self, depth: u8, lookup_table: &LookupTable) -> String {
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
        if let ResolvableString::String(s) = &self.name {
            lookup_table.index_string(s);
        }

        for attr in &self.attributes {
            if let ResolvableString::String(s) = &attr.name {
                lookup_table.index_string(s);
            }

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
    fn to_string(&self, lookup_table: &LookupTable) -> String {
        format!(
            "{}: {}",
            self.name.to_string(lookup_table),
            self.value.to_string(lookup_table)
        )
    }
}

/// A trait to represent an element of a map.
pub trait MapElement: Any + Debug {
    /// The name of the element in the map binary
    const NAME: &'static str;

    fn from_raw(parser: MapParser) -> Result<Self, MapElementParsingError>
    where Self: Sized;
    fn to_raw(&self, encoder: &mut MapEncoder);
}

/// An object-safe version of [MapElement]
pub trait ErasedMapElement: Any + Debug {
    fn name(&self) -> &str;
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

/// A dynamic element, if a parser for the element was registered it will be parsed into that struct, otherwise it is a [RawMapElement]
///
/// You can check what the element is with [ErasedMapElement::name], and check if the element is parsed using [Any::type_id]
pub type DynMapElement = Box<dyn ErasedMapElement>;

impl ErasedMapElement for RawMapElement {
    fn name(&self) -> &str {
        self.name.as_str().unwrap_or("UNRESOLVED STRING")
    }

    fn from_raw(parser: MapParser) -> Result<Self, MapElementParsingError>
    where Self: Sized {
        Ok(parser.raw.clone())
    }

    fn to_raw(&self, encoder: &mut MapEncoder) {
        encoder.from_raw(self)
    }
}


impl ErasedMapElement for DynMapElement {
    fn name(&self) -> &str {
        self.as_ref().name()
    }

    fn from_raw(_parser: MapParser) -> Result<Self, MapElementParsingError>
    where Self: Sized {
        Err(MapElementParsingError::custom(
            "can't use trait object for from_raw",
        ))
    }

    fn to_raw(&self, encoder: &mut MapEncoder) {
        (**self).to_raw(encoder)
    }
}

/// A manager struct that can read and write celeste maps.
pub struct MapManager {
    map: RawMap,
    parsers: HashMap<&'static str, Box<dyn ElementParserImpl>>,
}

impl MapManager {
    /// Creates a new `MapManager` reading in a celeste map from the passed reader
    pub fn new(mut reader: impl Read) -> Result<Self, MapReadError> {
        let mut buf = Vec::new();

        reader.read_to_end(&mut buf)?;

        let mut raw = RawMap::from_bytes(&buf)?;

        let parsers = HashMap::new();

        raw.resolve_strings();

        Ok(MapManager { map: raw, parsers })
    }

    /// Parse the map passed in the constructor using any registered parsers when needed
    pub fn parse_map(&self) -> Result<MapRoot, MapElementParsingError> {
        let parser = MapParser {
            verbose_debug: false,
            lookup: &self.map.lookup_table,
            raw: &self.map.root_element,
            parsers: &self.parsers,
        };

        parser.parse_self::<MapRoot>()
    }

    /// Same as [parse_map](Self::parse_map) but when compiled with `debug_assertions` will
    /// print debug information about the parser
    pub fn verbose_parse(&self) -> Result<MapRoot, MapElementParsingError> {
        let parser = MapParser {
            verbose_debug: cfg!(debug_assertions),
            lookup: &self.map.lookup_table,
            raw: &self.map.root_element,
            parsers: &self.parsers,
        };

        parser.parse_self::<MapRoot>()
    }

    /// Encode the map data back into a [RawMap]. The raw map is stored in the manager itself.
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

    /// Allows the `MapManager` to parse a new type of [MapElement].
    ///
    /// All [MapElement] implementations in this crate can be added via [MapManager::default_parsers].
    ///
    /// This is used exclusively for parsing [DynMapElement]s
    /// during calls to [MapParser::parse_any_element] any calls to [MapParser::parse_element] or [MapParser::parse_all_elements] will work even if a parser is never registered.
    ///
    /// This generally should not be called with any `Option<T>`, as that will conflict with the parser for `T`.
    /// There is no place in the vanilla elements where having a [DynMapElement] be optional makes sense and so
    /// no `Option<T>` impls are registered by default.
    pub fn add_parser<T: MapElement>(&mut self) {
        self.parsers
            .insert(T::NAME, Box::new(ElementParser::<T>::new()));
    }

    /// Allows the `MapManager` to parse a new type of [Entity]
    ///
    /// Acts the same as (add_parser)[MapManager::add_parser] but for entities
    pub fn add_entity_parser<T: Entity>(&mut self) {
        self.add_parser::<MapEntity<T>>();
    }

    /// Gets a reference to the [RawMap] stored in the manager.
    ///
    /// This is initialized in the constructor and modified in [encode_map](Self::encode_map)
    pub fn map(&self) -> &RawMap {
        &self.map
    }

    /// Allows the `MapManager` to parse a new type of [Trigger]
    ///
    /// Acts the same as (add_parser)[MapManager::add_parser] but for triggers
    pub fn add_trigger_parser<T: Trigger>(&mut self) {
        self.add_parser::<MapTrigger<T>>();
    }

    /// Writes the stored map data as binary into the provided writer
    pub fn write_map(&self, writer: &mut impl Write) -> Result<(), MapWriteError> {
        self.map.to_writer(writer)
    }

    /// Writes the stored map to binary and returns the bytes
    pub fn map_bytes(&self) -> Result<Vec<u8>, MapWriteError> {
        self.map.to_bytes()
    }
}
