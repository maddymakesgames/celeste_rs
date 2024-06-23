use std::{any::Any, collections::HashMap, error::Error, fmt::Display, marker::PhantomData};

use crate::maps::{
    encoder::MapEncoder,
    var_types::{EncodedVar, EncodedVarError},
    LookupIndex,
    LookupTable,
    MapElement,
    RawMapElement,
};

pub struct MapParser<'a> {
    pub(crate) verbose_debug: bool,
    pub(crate) lookup: &'a LookupTable,
    pub(crate) raw: &'a RawMapElement,
    pub(crate) parsers: &'a HashMap<&'static str, Box<dyn ElementParserImpl>>,
}

impl<'a> MapParser<'a> {
    pub(crate) fn parse_self<T: MapElement>(self) -> Result<T, MapElementParsingError> {
        T::from_raw(self)
    }

    pub fn parse_element<T: MapElement>(&self) -> Result<T, MapElementParsingError> {
        if self.verbose_debug {
            println!("{}", T::name());
        }
        for element in &self.raw.children {
            if self.lookup[element.name] == T::name() {
                return T::from_raw(MapParser {
                    verbose_debug: self.verbose_debug,
                    lookup: self.lookup,
                    raw: element,
                    parsers: self.parsers,
                });
            }
        }

        Err(MapElementParsingError::NoMatchingElementFound {
            expected: T::name(),
            found: self.lookup[self.raw.name].clone(),
        })
    }

    pub fn parse_all_elements<T: MapElement>(&self) -> Result<Vec<T>, MapElementParsingError> {
        let len = self.raw.children.len();

        if self.verbose_debug {
            println!("Vec<{}>", T::name());
        }

        self.raw
            .children
            .iter()
            .filter(|r| self.lookup[r.name] == T::name())
            .map(|r| {
                T::from_raw(MapParser {
                    verbose_debug: self.verbose_debug,
                    lookup: self.lookup,
                    raw: r,
                    parsers: self.parsers,
                })
            })
            .try_fold(
                Vec::with_capacity(len),
                |mut vec, t| -> Result<Vec<T>, MapElementParsingError> {
                    vec.push(t?);

                    Ok(vec)
                },
            )
    }

    pub fn parse_any_element(&self) -> Result<Vec<Box<dyn MapElement>>, MapElementParsingError> {
        let parsed_elements = self.raw.children.iter().map(|raw| {
            if let Some(parser) = self.parsers.get(self.lookup[raw.name].as_str()) {
                if self.verbose_debug {
                    println!("{}", parser.element_name());
                }

                parser.element_from_raw(MapParser {
                    verbose_debug: self.verbose_debug,
                    lookup: self.lookup,
                    raw,
                    parsers: self.parsers,
                })
            } else {
                Ok(Box::new(raw.clone()) as Box<dyn MapElement>)
            }
        });

        // Preallocate only enough for 1/4 of the possible errors
        // child lists can be long so we don't really want to preallocate the entire
        // length twice
        let mut errors = Vec::with_capacity(self.raw.children.len() / 4);
        let mut parsed = Vec::with_capacity(self.raw.children.len());

        for element in parsed_elements {
            match element {
                Ok(p) => parsed.push(p),
                Err(e) => errors.push(e),
            }
        }

        if errors.is_empty() {
            Ok(parsed)
        } else {
            Err(MapElementParsingError::MultiError { errors })
        }
    }

    pub fn get_attribute<'b, T: TryFrom<&'b EncodedVar, Error = EncodedVarError>>(
        &'b self,
        str: &'static str,
    ) -> Result<T, MapElementParsingError> {
        if self.verbose_debug {
            println!("Attr({str})");
        }

        match self
            .raw
            .attributes
            .iter()
            .find_map(|a| {
                if self.lookup[a.name] == str {
                    Some(&a.value)
                } else {
                    None
                }
            })
            .ok_or(MapElementParsingError::attribute_missing(str))
        {
            Ok(t) => T::try_from(t).map_err(MapElementParsingError::EncodedVarError),
            Err(e) => Err(e),
        }
    }

    pub fn get_attribute_raw(
        &self,
        str: &'static str,
    ) -> Result<&EncodedVar, MapElementParsingError> {
        if self.verbose_debug {
            println!("Attr({str})");
        }

        self.raw
            .attributes
            .iter()
            .find_map(|a| {
                if self.lookup[a.name] == str {
                    Some(&a.value)
                } else {
                    None
                }
            })
            .ok_or(MapElementParsingError::attribute_missing(str))
    }

    pub fn get_optional_attribute<'b, T: TryFrom<&'b EncodedVar, Error = EncodedVarError>>(
        &'b self,
        str: &'static str,
    ) -> Option<T> {
        self.get_attribute(str).ok()
    }
}


#[derive(Debug)]
pub enum MapElementParsingError {
    AttributeMissing {
        name: &'static str,
    },
    NoMatchingElementFound {
        expected: &'static str,
        found: String,
    },
    NameMismatch {
        expected: &'static str,
        found: String,
    },
    EncodedVarError(EncodedVarError),
    MultiError {
        errors: Vec<MapElementParsingError>,
    },
    Custom(Box<dyn Error>),
}

impl MapElementParsingError {
    pub fn attribute_missing(name: &'static str) -> MapElementParsingError {
        MapElementParsingError::AttributeMissing { name }
    }

    pub fn name_mismatch(
        expected: &'static str,
        name: LookupIndex,
        lookup_table: &LookupTable,
    ) -> Self {
        MapElementParsingError::NameMismatch {
            expected,
            found: lookup_table[name].clone(),
        }
    }

    pub fn custom(err: impl Into<Box<dyn Error>>) -> Self {
        MapElementParsingError::Custom(err.into())
    }
}

impl Display for MapElementParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MapElementParsingError::NameMismatch { expected, found } => write!(
                f,
                "Found attribute of wrong type, expected \"{expected}\" found \"{found}\""
            ),
            MapElementParsingError::EncodedVarError(e) => Display::fmt(e, f),
            MapElementParsingError::Custom(e) => Display::fmt(e, f),
            MapElementParsingError::NoMatchingElementFound { expected, found } => write!(
                f,
                "No element of type \"{}\" was found in the children of a \"{}\" element",
                expected, found
            ),
            MapElementParsingError::AttributeMissing { name } =>
                write!(f, "Missing attribute \"{name}\""),
            MapElementParsingError::MultiError { errors } => {
                writeln!(f, "Found multiple errors parsing map elements:")?;

                for error in errors {
                    writeln!(f, "\t{error}")?;
                }

                Ok(())
            }
        }
    }
}

impl Error for MapElementParsingError {}

impl From<EncodedVarError> for MapElementParsingError {
    fn from(value: EncodedVarError) -> Self {
        MapElementParsingError::EncodedVarError(value)
    }
}


pub trait ElementParserImpl: Any {
    fn element_name(&self) -> &'static str;
    fn element_from_raw(
        &self,
        parser: MapParser,
    ) -> Result<Box<dyn MapElement>, MapElementParsingError>;
    fn element_to_raw(&self, element: &dyn MapElement, encoder: &mut MapEncoder);
}

pub trait MapElementParser: Any {
    type Element: MapElement;
}

pub struct ElementParser<T>(PhantomData<T>);

impl<T> ElementParser<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T> Default for ElementParser<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: MapElement> ElementParserImpl for ElementParser<T> {
    fn element_name(&self) -> &'static str {
        T::name()
    }

    fn element_from_raw(
        &self,
        parser: MapParser,
    ) -> Result<Box<dyn MapElement>, MapElementParsingError> {
        T::from_raw(parser).map(|e| Box::new(e) as Box<dyn MapElement>)
    }

    fn element_to_raw(&self, element: &dyn MapElement, encoder: &mut MapEncoder) {
        let element = unsafe { &*(element as *const dyn MapElement as *const T) };
        element.to_raw(encoder)
    }
}

impl MapElement for Box<dyn MapElement> {
    fn name() -> &'static str
    where Self: Sized {
        ""
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
