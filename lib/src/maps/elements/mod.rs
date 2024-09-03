use celeste_rs_macros::MapElement;

use crate::maps::{
    entities::add_entity_parsers,
    level::*,
    style::*,
    triggers::add_trigger_parsers,
    var_types::{Float, Integer},
    MapElement,
    MapManager,
};

pub mod entities;
pub mod level;
pub mod style;
pub mod triggers;

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
        add_trigger_parsers(self);
    }
}

#[derive(Debug, MapElement)]
#[name = "Map"]
pub struct MapRoot {
    #[child]
    pub filler: Filler,
    #[child]
    pub levels: Levels,
    #[child]
    pub style: Styles,
}

#[derive(Debug, MapElement)]
#[name = "Filler"]
pub struct Filler {
    #[child]
    pub filler: Vec<Rect>,
}

#[derive(Debug, Clone, Copy, MapElement)]
#[name = "rect"]
pub struct Rect {
    #[name = "x"]
    pub x: Integer,
    #[name = "y"]
    pub y: Integer,
    #[name = "w"]
    pub w: Integer,
    #[name = "h"]
    pub h: Integer,
}

#[derive(Debug, MapElement)]
#[name = "node"]
pub struct Node {
    #[name = "x"]
    pub x: Float,
    #[name = "y"]
    pub y: Float,
}
