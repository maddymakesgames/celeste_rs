use celeste_rs_macros::{Entity, MapElement};

use crate::maps::{
    encoder::MapEncoder,
    parser::{MapElementParsingError, MapParser},
    var_types::{Float, Integer},
    MapElement,
    MapManager,
    ResolvableString,
};

use std::{any::Any, fmt::Debug};

#[derive(Debug)]
pub struct MapEntity<T: Entity> {
    id: ResolvableString,
    x: Float,
    y: Float,
    width: Option<Integer>,
    height: Option<Integer>,
    origin_x: Float,
    origin_y: Float,
    entity: T,
}

impl<T: Entity> MapElement for MapEntity<T> {
    const NAME: &'static str = T::NAME;

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
        encoder.attribute("id", self.id.clone());
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
    const NAME: &'static str;

    fn from_raw(parser: MapParser) -> Result<Self, MapElementParsingError>
    where Self: Sized;
    fn to_raw(&self, encoder: &mut MapEncoder);
}

#[derive(Debug, MapElement)]
#[name = "node"]
pub struct Node {
    #[name = "x"]
    x: Float,
    #[name = "y"]
    y: Float,
}


macro_rules! unit_entities {
    ($($struct_name: ident, $name: literal),*) => {
        pub fn add_unit_entity_parsers(mm: &mut MapManager) {
            $(
                mm.add_entity_parser::<$struct_name>();
            )*
        }

        $(
            #[derive(Debug, Clone, Copy, Default)]
            pub struct $struct_name;

            impl Entity for $struct_name {
                const NAME: &'static str = $name;

                fn from_raw(_parser: MapParser) -> Result<Self, MapElementParsingError>
                where Self: Sized {
                    Ok(Self::default())
                }

                fn to_raw(&self, _encoder: &mut MapEncoder) {}
            }
        )*
    };
}

pub fn add_entity_parsers(mm: &mut MapManager) {
    mm.add_entity_parser::<SpikesUp>();
    mm.add_entity_parser::<SpikesDown>();
    mm.add_entity_parser::<SpikesLeft>();
    mm.add_entity_parser::<SpikesRight>();
    mm.add_entity_parser::<JumpThru>();
    mm.add_entity_parser::<Wire>();
    mm.add_entity_parser::<Strawberry>();
    mm.add_entity_parser::<Lightbeam>();
    mm.add_entity_parser::<Cassette>();
    mm.add_entity_parser::<CassetteBlock>();
    mm.add_entity_parser::<DashBlock>();
    mm.add_entity_parser::<Bonfire>();
    mm.add_entity_parser::<NPC>();
    mm.add_entity_parser::<CoverupWall>();
    mm.add_entity_parser::<Memorial>();
    mm.add_entity_parser::<BirdForsakenCityGem>();
    mm.add_entity_parser::<FallingBlock>();
    mm.add_entity_parser::<ZipMover>();
    mm.add_entity_parser::<FakeWall>();
    mm.add_entity_parser::<Spring>();
    mm.add_entity_parser::<Refill>();
    add_unit_entity_parsers(mm);
}
#[derive(Debug, Entity)]
#[name = "spikesUp"]
pub struct SpikesUp {
    #[name = "type"]
    pub kind: Option<ResolvableString>,
}
#[derive(Debug, Entity)]
#[name = "spikesDown"]
pub struct SpikesDown {
    #[name = "type"]
    pub kind: Option<ResolvableString>,
}

#[derive(Debug, Entity)]
#[name = "spikesLeft"]
pub struct SpikesLeft {
    #[name = "type"]
    pub kind: Option<ResolvableString>,
}

#[derive(Debug, Entity)]
#[name = "spikesRight"]
pub struct SpikesRight {
    #[name = "type"]
    pub kind: Option<ResolvableString>,
}


#[derive(Debug, Entity)]
#[name = "jumpThru"]
pub struct JumpThru {
    #[name = "texture"]
    pub texture: Option<ResolvableString>,
}

#[derive(Debug, Entity)]
#[name = "wire"]
pub struct Wire {
    #[name = "above"]
    pub above: bool,
    pub to: Node,
}

#[derive(Debug, Entity)]
#[name = "strawberry"]
pub struct Strawberry {
    #[name = "winged"]
    pub winged: bool,
    #[name = "checkpointID"]
    pub checkpoint_id: Integer,
    #[name = "order"]
    pub order: Option<Integer>,
}

#[derive(Debug, Entity)]
#[name = "lightbeam"]
pub struct Lightbeam {
    #[name = "rotation"]
    pub rotation: Integer,
    #[name = "flag"]
    pub flag: ResolvableString,
}

#[derive(Debug, Entity)]
#[name = "cassette"]
pub struct Cassette {
    #[node]
    bubble_points: Vec<Node>,
}

#[derive(Debug, Entity)]
#[name = "cassetteBlock"]
pub struct CassetteBlock {
    #[name = "index"]
    pub index: Integer,
    #[name = "finishedState"]
    pub finished_state: Option<bool>,
}

#[derive(Debug, Entity)]
#[name = "dashBlock"]
pub struct DashBlock {
    #[name = "permanent"]
    pub permanent: bool,
    #[name = "tiletype"]
    pub tile_type: Integer,
    #[name = "blendin"]
    pub blend_in: bool,
    #[name = "canDash"]
    pub can_dash: bool,
}
#[derive(Debug, Entity)]
#[name = "bonfire"]
pub struct Bonfire {
    #[name = "mode"]
    pub mode: ResolvableString,
}

#[derive(Debug, Entity)]
#[name = "npc"]
pub struct NPC {
    #[name = "npc"]
    pub npc: ResolvableString,
}

#[derive(Debug, Entity)]
#[name = "coverupWall"]
pub struct CoverupWall {
    #[name = "tiletype"]
    pub tile_type: Integer,
}

#[derive(Debug, Entity)]
#[name = "memorial"]
pub struct Memorial {
    #[name = "dreaming"]
    pub dreaming: bool,
}

#[derive(Debug, Entity)]
#[name = "birdForsakenCityGem"]
pub struct BirdForsakenCityGem {
    #[node]
    nodes: Vec<Node>,
}

#[derive(Debug, Entity)]
#[name = "fallingBlock"]
pub struct FallingBlock {
    #[name = "tiletype"]
    pub tile_type: Integer,
    #[name = "behind"]
    pub behind: bool,
    #[name = "climbFall"]
    pub climb_fall: Option<bool>,
}

#[derive(Debug, Entity)]
#[name = "fakeWall"]
pub struct FakeWall {
    #[name = "tiletype"]
    pub tile_type: ResolvableString,
    #[name = "playTransitionReveal"]
    pub play_transition_reveal: Option<bool>,
}

#[derive(Debug, Entity)]
#[name = "spring"]
pub struct Spring {
    #[name = "playerCanUse"]
    pub player_can_use: Option<bool>,
}

#[derive(Debug, Entity)]
#[name = "zipMover"]
pub struct ZipMover {
    #[name = "theme"]
    pub theme: Option<ResolvableString>,
    pub to: Node,
}

#[derive(Debug, Entity)]
#[name = "refill"]
pub struct Refill {
    #[name = "twoDash"]
    pub two_dash: Option<bool>,
    #[name = "oneUse"]
    pub one_use: Option<bool>,
}

unit_entities! {
    Player, "player",
    GoldenBerry, "goldenBerry",
    CrumbleBlock, "crumbleBlock",
    Checkpoint, "checkpoint",
    WingedGoldenStrawberry, "memorialTextController",
    FlutterBird, "flutterbird"
}
