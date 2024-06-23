use crate::maps::{
    encoder::MapEncoder,
    parser::{MapElementParsingError, MapParser},
    var_types::{Float, Integer},
    MapElement,
    MapManager,
};

use std::{any::Any, fmt::Debug};

pub fn add_entity_parsers(_mm: &mut MapManager) {
    todo!()
}

#[derive(Debug)]
pub struct MapEntity<T: Entity> {
    id: Integer,
    x: Float,
    y: Float,
    width: Option<Integer>,
    height: Option<Integer>,
    origin_x: Float,
    origin_y: Float,
    entity: T,
}

impl<T: Entity> MapElement for MapEntity<T> {
    fn name() -> &'static str
    where Self: Sized {
        T::name()
    }

    fn from_raw(parser: MapParser) -> Result<Self, MapElementParsingError>
    where Self: Sized {
        Ok(Self {
            id: parser.get_attribute("id")?,
            x: parser.get_attribute("x")?,
            y: parser.get_attribute("y")?,
            width: parser.get_optional_attribute("width"),
            height: parser.get_optional_attribute("height"),
            origin_x: parser.get_attribute("originX")?,
            origin_y: parser.get_attribute("originY")?,
            entity: T::from_raw(parser)?,
        })
    }

    fn to_raw(&self, encoder: &mut MapEncoder) {
        encoder.attribute("id", self.id);
        encoder.attribute("x", self.x);
        encoder.attribute("y", self.y);
        encoder.optional_attribute("width", &self.width);
        encoder.optional_attribute("height", &self.height);
        encoder.attribute("originX", self.origin_x);
        encoder.attribute("originY", self.origin_y);
        self.entity.to_raw(encoder);
    }
}

pub trait Entity: Debug + Any {
    fn name() -> &'static str
    where Self: Sized;

    fn from_raw(parser: MapParser) -> Result<Self, MapElementParsingError>
    where Self: Sized;
    fn to_raw(&self, encoder: &mut MapEncoder);
}

#[derive(Debug)]
pub struct Node {
    x: Float,
    y: Float,
}

impl MapElement for Node {
    fn name() -> &'static str
    where Self: Sized {
        "node"
    }

    fn from_raw(parser: MapParser) -> Result<Self, MapElementParsingError>
    where Self: Sized {
        Ok(Self {
            x: parser.get_attribute("x")?,
            y: parser.get_attribute("y")?,
        })
    }

    fn to_raw(&self, encoder: &mut MapEncoder) {
        encoder.attribute("x", self.x);
        encoder.attribute("y", self.y);
    }
}

pub trait UnitEntity: Debug + Any + Copy + Default {
    const NAME: &'static str;
}

impl<T: UnitEntity> Entity for T {
    fn name() -> &'static str
    where Self: Sized {
        T::NAME
    }

    fn from_raw(_parser: MapParser) -> Result<Self, MapElementParsingError>
    where Self: Sized {
        Ok(T::default())
    }

    fn to_raw(&self, _encoder: &mut MapEncoder) {}
}
