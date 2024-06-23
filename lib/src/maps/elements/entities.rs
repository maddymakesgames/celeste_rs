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
    const NAME: &'static str;

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
    const NAME: &'static str = "node";

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
    const NAME: &'static str = T::NAME;

    fn from_raw(_parser: MapParser) -> Result<Self, MapElementParsingError>
    where Self: Sized {
        Ok(T::default())
    }

    fn to_raw(&self, _encoder: &mut MapEncoder) {}
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

            impl UnitEntity for $struct_name {
                const NAME: &'static str = $name;
            }
        )*
    };
}

macro_rules! entities {
    ($($premade_structs: ident),*$(($struct_name: ident, $name: literal, $field_data: tt $($node_data: tt)?)),*) => {
        pub fn add_entity_parsers(mm: &mut MapManager) {
            $(
                mm.add_entity_parser::<$struct_name>();
            )*
            $(
                mm.add_entity_parser::<$premade_structs>();
            )*
            add_unit_entity_parsers(mm);
        }

        $(
            entities!{p $struct_name, $name, $field_data $($node_data)?}
        )*
    };

    (p $struct_name: ident, $name: literal, [$($field_name: ident, $field_bin_name: literal, $field_type: ty),*]) => {
        #[derive(Debug)]
        pub struct $struct_name {
            $(pub $field_name: $field_type),*
        }

        impl Entity for $struct_name {
            const NAME: &'static str = $name;

            fn from_raw(parser: MapParser) -> Result<Self, MapElementParsingError>
            where Self: Sized {
                Ok(Self {
                    $($field_name: parser.get_attribute($field_bin_name)?),*
                })
            }

            fn to_raw(&self, encoder: &mut MapEncoder) {
                $(
                    encoder.attribute($field_bin_name, self.$field_name.clone());
                )*
            }
        }
    };

    (p $struct_name: ident, $name: literal, [$($field_name: ident, $field_bin_name: literal, $field_type: ty),*] ($node_name: ident)) => {
        #[derive(Debug)]
        pub struct $struct_name {
            $(pub $field_name: $field_type,)*
            $node_name: Node
        }

        impl Entity for $struct_name {
            const NAME: &'static str = $name;

            fn from_raw(parser: MapParser) -> Result<Self, MapElementParsingError>
            where Self: Sized {
                Ok(Self {
                    $node_name: parser.parse_element()?,
                    $($field_name: parser.get_attribute($field_bin_name)?),*
                })
            }

            fn to_raw(&self, encoder: &mut MapEncoder) {
                $(
                    encoder.attribute($field_bin_name, self.$field_name.clone());
                )*
                encoder.child(&self.$node_name);
            }
        }
    };

    (p $struct_name: ident, $name: literal, [$($field_name: ident, $field_bin_name: literal, $field_type: ty),*] [$node_name: ident]) => {
        #[derive(Debug)]
        pub struct $struct_name {
            $(pub $field_name: $field_type),*
            $node_name: Vec<Node>
        }

        impl Entity for $struct_name {
            const NAME: &'static str = $name;

            fn from_raw(parser: MapParser) -> Result<Self, MapElementParsingError>
            where Self: Sized {
                Ok(Self {
                    $node_name: parser.parse_all_elements()?,
                    $($field_name: parser.get_attribute($field_bin_name)?),*
                })
            }

            fn to_raw(&self, encoder: &mut MapEncoder) {
                $(
                    encoder.attribute($field_bin_name, self.$field_name.clone());
                )*
                encoder.children(&self.$node_name);
            }
        }
    };
}

entities! {
    FallingBlock
    (
        SpikesUp, "spikesUp",
        [kind, "type", ResolvableString]
    ),
    (
        SpikesDown, "spikesDown",
        [kind, "type", ResolvableString]
    ),
    (
        SpikesLeft, "spikesLeft",
        [kind, "type", ResolvableString]
    ),
    (
        SpikesRight, "spikesRight",
        [kind, "type", ResolvableString]
    ),
    (
        JumpThru, "jumpThru",
        [texture, "texture", ResolvableString]
    ),
    (
        ZipMover, "zipMover",
        []
        (target)
    ),
    (
        Wire, "wire",
        [above, "above", bool]
        (to)
    ),
    (
        Strawberry, "strawberry",
        [
            winged, "winged", bool,
            checkpoint_id, "checkpointID", Integer,
            order, "order", Integer
        ]
    ),
    (
        Lightbeam, "lightbeam",
        [
            rotation, "rotation", Integer,
            flag, "flag", ResolvableString
        ]
    ),
    (
        Cassette, "cassette",
        []
        [bubble_points]
    ),
    (
        CassetteBlock, "cassetteBlock",
        [
            index, "index", Integer,
            finished_state, "finishedState", bool
        ]
    ),
    (
        DashBlock, "dashBlock",
        [
            permanent, "permanent", bool,
            tile_type, "tiletype", Integer,
            blend_in, "blendin", bool,
            can_dash, "canDash", bool
        ]
    ),
    (
        Bonfire, "bonfire",
        [
            mode, "mode", ResolvableString
        ]
    ),
    (
        NPC, "npc",
        [
            npc, "npc", ResolvableString
        ]
    ),
    (
        CoverupWall, "coverupWall",
        [
            tile_type, "tiletype", Integer
        ]
    ),
    (
        Memorial, "memorial",
        [
            dreaming, "dreaming", bool
        ]
    ),
    (
        BirdForsakenCityGem, "birdForsakenCityGem",
        []
        [nodes]
    )
}

unit_entities! {
    Player, "player",
    GoldenBerry, "goldenBerry",
    CrumbleBlock, "crumbleBlock",
    Refill, "refill",
    Checkpoint, "checkpoint",
    WingedGoldenStrawberry, "memorialTextController",
    FlutterBird, "flutterbird"
}

#[derive(Debug)]
pub struct FallingBlock {
    pub tile_type: Integer,
    pub behind: bool,
    pub climb_fall: Option<bool>,
}

impl Entity for FallingBlock {
    const NAME: &'static str = "fallingBlock";

    fn from_raw(parser: MapParser) -> Result<Self, MapElementParsingError>
    where Self: Sized {
        Ok(Self {
            tile_type: parser.get_attribute("tiletype")?,
            behind: parser.get_attribute("behind")?,
            climb_fall: parser.get_optional_attribute("climbFall"),
        })
    }

    fn to_raw(&self, encoder: &mut MapEncoder) {
        encoder.attribute("tiletype", self.tile_type);
        encoder.attribute("behind", self.behind);
        encoder.optional_attribute("climbFall", &self.climb_fall);
    }
}

#[derive(Debug)]
pub struct FakeWall {
    pub tile_type: Integer,
    pub play_transition_reveal: Option<bool>,
}

impl Entity for FakeWall {
    const NAME: &'static str = "fakeWall";

    fn from_raw(parser: MapParser) -> Result<Self, MapElementParsingError>
    where Self: Sized {
        Ok(Self {
            tile_type: parser.get_attribute("tiletype")?,
            play_transition_reveal: parser.get_optional_attribute("playTransitionReveal"),
        })
    }

    fn to_raw(&self, encoder: &mut MapEncoder) {
        encoder.attribute("tiletype", self.tile_type);
        encoder.optional_attribute("playTransitionReveal", &self.play_transition_reveal);
    }
}

#[derive(Debug)]
pub struct Spring {
    pub player_can_use: Option<bool>,
}

impl Entity for Spring {
    const NAME: &'static str = "spring";

    fn from_raw(parser: MapParser) -> Result<Self, MapElementParsingError>
    where Self: Sized {
        Ok(Self {
            player_can_use: parser.get_optional_attribute("playerCanUse"),
        })
    }

    fn to_raw(&self, encoder: &mut MapEncoder) {
        encoder.optional_attribute("playerCanUse", &self.player_can_use);
    }
}
