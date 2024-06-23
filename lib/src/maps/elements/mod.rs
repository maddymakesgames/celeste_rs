use crate::maps::{
    entities::add_entity_parsers,
    level::*,
    style::*,
    var_types::Integer,
    MapElement,
    MapElementParsingError,
    MapEncoder,
    MapManager,
    MapParser,
};

pub mod entities;
pub mod level;
pub mod style;

impl MapManager {
    pub fn default_parsers(&mut self) {
        self.add_parser::<MapRoot>();
        self.add_parser::<Filler>();
        self.add_parser::<Rect>();
        self.add_parser::<Levels>();
        self.add_parser::<Level>();
        self.add_parser::<Styles>();
        self.add_parser::<Triggers>();
        self.add_parser::<FGTiles>();
        self.add_parser::<BGTiles>();
        self.add_parser::<FGDecals>();
        self.add_parser::<BGDecals>();
        self.add_parser::<Decal>();
        self.add_parser::<Background>();
        self.add_parser::<ObjTiles>();
        self.add_parser::<Solids>();
        self.add_parser::<Entities>();
        self.add_parser::<Backgrounds>();
        self.add_parser::<Foregrounds>();
        self.add_parser::<Parallax>();
        self.add_parser::<SnowBG>();
        self.add_parser::<SnowFG>();
        add_entity_parsers(self);
    }
}

#[derive(Debug)]
pub struct MapRoot {
    pub filler: Filler,
    pub levels: Levels,
    pub style: Styles,
}

impl MapElement for MapRoot {
    fn name() -> &'static str
    where Self: Sized {
        "Map"
    }

    fn from_raw(parser: MapParser) -> Result<Self, MapElementParsingError> {
        let filler = parser.parse_element::<Filler>()?;
        let levels = parser.parse_element::<Levels>()?;
        let style = parser.parse_element::<Styles>()?;

        Ok(MapRoot {
            filler,
            levels,
            style,
        })
    }

    fn to_raw(&self, encoder: &mut MapEncoder) {
        encoder.child(&self.filler);
        encoder.child(&self.levels);
        encoder.child(&self.style);
    }
}

#[derive(Debug, Clone)]
pub struct Filler {
    pub filler: Vec<Rect>,
}

impl MapElement for Filler {
    fn name() -> &'static str
    where Self: Sized {
        "Filler"
    }

    fn from_raw(parser: MapParser) -> Result<Self, MapElementParsingError>
    where Self: Sized {
        Ok(Self {
            filler: parser.parse_all_elements()?,
        })
    }

    fn to_raw(&self, encoder: &mut MapEncoder) {
        for filler in &self.filler {
            encoder.child(filler)
        }
    }
}

#[derive(Debug, Clone)]
pub struct Rect {
    pub x: Integer,
    pub y: Integer,
    pub w: Integer,
    pub h: Integer,
}

impl MapElement for Rect {
    fn name() -> &'static str
    where Self: Sized {
        "rect"
    }

    fn from_raw(parser: MapParser) -> Result<Self, MapElementParsingError> {
        let x = parser.get_attribute("x")?;
        let y = parser.get_attribute("y")?;
        let w = parser.get_attribute("w")?;
        let h = parser.get_attribute("h")?;

        Ok(Rect { x, y, w, h })
    }

    fn to_raw(&self, encoder: &mut MapEncoder) {
        encoder.attribute("x", self.x);
        encoder.attribute("y", self.y);
        encoder.attribute("w", self.w);
        encoder.attribute("h", self.h);
    }
}
