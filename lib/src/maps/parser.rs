use std::{any::Any, collections::HashMap, error::Error, fmt::Display, marker::PhantomData};

use crate::maps::{
    DynMapElement,
    ErasedMapElement,
    LookupIndex,
    LookupTable,
    MapElement,
    RawMapElement,
    encoder::MapEncoder,
    var_types::{EncodedVar, EncodedVarError},
};

/// Helper to parse [MapElement] implementors from [RawMapElement]s
pub struct MapParser<'a> {
    pub(crate) verbose_debug: bool,
    pub(crate) lookup: &'a LookupTable,
    pub(crate) raw: &'a RawMapElement,
    pub(crate) parsers: &'a HashMap<&'static str, Box<dyn ElementParserImpl>>,
}

impl MapParser<'_> {
    /// Parses `T` without forking, needed to parse the map root properly
    pub(crate) fn parse_self<T: MapElement>(self) -> Result<T, MapElementParsingError> {
        T::from_raw(self)
    }

    /// Attempts to parse a child element `T` from the current element
    pub fn parse_element<T: MapElement>(&self) -> Result<T, MapElementParsingError> {
        if self.verbose_debug {
            println!("{}", T::NAME);
        }
        for element in &self.raw.children {
            if element.name.to_string(self.lookup) == T::NAME {
                return T::from_raw(MapParser {
                    verbose_debug: self.verbose_debug,
                    lookup: self.lookup,
                    raw: element,
                    parsers: self.parsers,
                });
            }
        }

        Err(MapElementParsingError::NoMatchingElementFound {
            expected: T::NAME,
            found: self.raw.name.to_string(self.lookup).to_owned(),
        })
    }

    /// Attempts to parse as many child elements of type `T` as possible
    // TODO: make this return multierror instead of just ignoring all errors
    pub fn parse_all_elements<T: MapElement>(&self) -> Result<Vec<T>, MapElementParsingError> {
        let len = self.raw.children.len();

        if self.verbose_debug {
            println!("Vec<{}>", T::NAME);
        }

        self.raw
            .children
            .iter()
            .filter(|r| r.name.to_string(self.lookup) == T::NAME)
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

    /// Parses all the children of the current element as [DynMapElement]s
    ///
    /// Any elements found that don't have registered parsers will be kept as [RawMapElement]
    pub fn parse_any_element(&self) -> Result<Vec<DynMapElement>, MapElementParsingError> {
        let parsed_elements = self.raw.children.iter().map(|raw| {
            if let Some(parser) = self.parsers.get(raw.name.to_string(self.lookup)) {
                if self.verbose_debug {
                    println!("{}", parser.element_name());
                }

                parser
                    .element_from_raw(MapParser {
                        verbose_debug: self.verbose_debug,
                        lookup: self.lookup,
                        raw,
                        parsers: self.parsers,
                    })
                    .map_err(|e| (parser.element_name().to_owned(), e))
            } else {
                Ok(Box::new(raw.clone()) as DynMapElement)
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

    /// Attempts to parse a child with type `T`, if there is not one, returns `None`.
    ///
    /// Works the same as [parse_element](MapParser::parse_element), but returns `Ok(None)` instead of `NoMatchingElementFound` when no element of type `T` is found.
    pub fn parse_optional_element<T: MapElement>(
        &self,
    ) -> Result<Option<T>, MapElementParsingError> {
        for child in &self.raw.children {
            if child.name.to_string(self.lookup) == T::NAME {
                if self.verbose_debug {
                    println!("{}", T::NAME);
                }
                return T::from_raw(MapParser {
                    verbose_debug: self.verbose_debug,
                    lookup: self.lookup,
                    raw: child,
                    parsers: self.parsers,
                })
                .map(Some);
            }
        }

        Ok(None)
    }

    /// Get an attribute of type `T` with name `str`.
    ///
    /// If you want to accept any attribute type, use [get_attribute_raw](Self::get_attribute_raw) instead.
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
                if a.name.to_string(self.lookup) == str {
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

    /// Gets the [EncodedVar] of an attribute with name `str` on the current element.
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
                if a.name.to_string(self.lookup) == str {
                    Some(&a.value)
                } else {
                    None
                }
            })
            .ok_or(MapElementParsingError::attribute_missing(str))
    }

    /// Returns the value attached to the attribute with name `str` if it is there, otherwise returns `None`
    ///
    /// Works the same as [get_attribute](MapParser::get_attribute) but if there isn't an attribute with the name `str` it returns `Ok(None)` instead of `AttributeMissing`
    pub fn get_optional_attribute<'b, T: TryFrom<&'b EncodedVar, Error = EncodedVarError>>(
        &'b self,
        str: &'static str,
    ) -> Result<Option<T>, MapElementParsingError> {
        if self.verbose_debug {
            println!("Attr({str})");
        }

        match self.raw.attributes.iter().find_map(|a| {
            if a.name.to_string(self.lookup) == str {
                Some(&a.value)
            } else {
                None
            }
        }) {
            Some(t) => T::try_from(t)
                .map_err(MapElementParsingError::EncodedVarError)
                .map(Some),
            None => Ok(None),
        }
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
        errors: Vec<(String, MapElementParsingError)>,
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
                writeln!(f, "Found errors parsing dynamic map elements:")?;

                for (name, error) in errors {
                    writeln!(f, "\t{name}: {error}")?;
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

/// Represents something that can parse elements.
///
/// This shouldn't really be implemented by users, the only real use for this is
/// in [MapManager](super::MapManager) with [ElementParser]s.
pub trait ElementParserImpl: Any {
    fn element_name(&self) -> &'static str;
    fn element_from_raw(&self, parser: MapParser) -> Result<DynMapElement, MapElementParsingError>;
    fn element_to_raw(&self, element: &dyn ErasedMapElement, encoder: &mut MapEncoder);
}

/// A parser of [MapElement]s
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
        T::NAME
    }

    fn element_from_raw(&self, parser: MapParser) -> Result<DynMapElement, MapElementParsingError> {
        T::from_raw(parser).map(|e| Box::new(e) as DynMapElement)
    }

    fn element_to_raw(&self, element: &dyn ErasedMapElement, encoder: &mut MapEncoder) {
        let element = unsafe { &*(element as *const dyn ErasedMapElement as *const T) };
        element.to_raw(encoder)
    }
}
