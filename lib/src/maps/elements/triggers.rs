use std::{any::Any, fmt::Debug};

use celeste_rs_macros::Trigger;

use crate::{
    maps::{
        MapElement,
        MapManager,
        Node,
        ResolvableString,
        encoder::MapEncoder,
        parser::{MapElementParsingError, MapParser},
        var_types::Character,
    },
    utils::num::{Float, Integer},
};
#[derive(Debug)]
/// The metadata related to all triggers
pub struct MapTrigger<T: Trigger> {
    pub id: Integer,
    pub x: Float,
    pub y: Float,
    pub width: Option<Integer>,
    pub height: Option<Integer>,
    pub origin_x: Option<Float>,
    pub origin_y: Option<Float>,
    pub entity: T,
}

impl<T: Trigger> MapElement for MapTrigger<T> {
    const NAME: &'static str = T::NAME;

    fn from_raw(parser: MapParser) -> Result<Self, MapElementParsingError>
    where Self: Sized {
        Ok(Self {
            id: parser.get_attribute("id")?,
            x: parser.get_attribute("x")?,
            y: parser.get_attribute("y")?,
            width: parser.get_optional_attribute("width")?,
            height: parser.get_optional_attribute("height")?,
            origin_x: parser.get_optional_attribute("originX")?,
            origin_y: parser.get_optional_attribute("originY")?,
            entity: T::from_raw(parser)?,
        })
    }

    fn to_raw(&self, encoder: &mut MapEncoder) {
        encoder.attribute("id", self.id);
        encoder.attribute("x", self.x);
        encoder.attribute("y", self.y);
        encoder.optional_attribute("width", &self.width);
        encoder.optional_attribute("height", &self.height);
        encoder.optional_attribute("originX", &self.origin_x);
        encoder.optional_attribute("originY", &self.origin_y);
        self.entity.to_raw(encoder);
    }
}

pub trait Trigger: Debug + Any {
    const NAME: &'static str;

    fn from_raw(parser: MapParser) -> Result<Self, MapElementParsingError>
    where Self: Sized;
    fn to_raw(&self, encoder: &mut MapEncoder);
}


pub fn add_trigger_parsers(mm: &mut MapManager) {
    mm.add_trigger_parser::<LookoutBlocker>();
    mm.add_trigger_parser::<MusicTrigger>();
    mm.add_trigger_parser::<BirdPathTrigger>();
    mm.add_trigger_parser::<BlackholeStrength>();
    mm.add_trigger_parser::<CameraOffsetTrigger>();
    mm.add_trigger_parser::<ChangeRespawnTrigger>();
    mm.add_trigger_parser::<MiniTextboxTrigger>();
    mm.add_trigger_parser::<RespawnTargetTrigger>();
    mm.add_trigger_parser::<NoRefillTrigger>();
    mm.add_trigger_parser::<BloomFadeTrigger>();
    mm.add_trigger_parser::<CameraTargetTrigger>();
    mm.add_trigger_parser::<MusicFadeTrigger>();
    mm.add_trigger_parser::<GoldenBerryCollectTrigger>();
    mm.add_trigger_parser::<CameraAdvanceTargetTrigger>();
    mm.add_trigger_parser::<LightFadeTrigger>();
    mm.add_trigger_parser::<SpawnFacingTrigger>();
    mm.add_trigger_parser::<WindAttackTrigger>();
    mm.add_trigger_parser::<AmbienceParamTrigger>();
    mm.add_trigger_parser::<EventTrigger>();
    mm.add_trigger_parser::<DetachFollowersTrigger>();
    mm.add_trigger_parser::<CheckpointBlockerTrigger>();
    mm.add_trigger_parser::<AltMusicTrigger>();
    mm.add_trigger_parser::<WindTrigger>();
    mm.add_trigger_parser::<CreditsTrigger>();
    mm.add_trigger_parser::<RumbleTrigger>();
    mm.add_trigger_parser::<OshiroTrigger>();
    mm.add_trigger_parser::<MoonGlitchBackgroundTrigger>();
    mm.add_trigger_parser::<InteractTrigger>();
}

use vanilla_triggers::*;

#[allow(missing_docs)]
pub mod vanilla_triggers {

    use super::*;

    #[derive(Debug, Trigger, Default, Clone, Copy, PartialEq, Eq)]
    #[name = "lookoutBlocker"]
    pub struct LookoutBlocker;

    #[derive(Debug, Trigger)]
    #[name = "musicTrigger"]
    pub struct MusicTrigger {
        #[name = "track"]
        pub track: ResolvableString,
        #[name = "resetFade"]
        pub resetfade: Option<bool>,
        #[name = "setInSession"]
        pub setinsession: Option<bool>,
        #[name = "resetOnLeave"]
        pub resetonleave: bool,
        #[name = "progress"]
        pub progress: Option<u8>,
    }
    #[derive(Debug, Trigger, Default, Clone, Copy, PartialEq, Eq)]
    #[name = "birdPathTrigger"]
    pub struct BirdPathTrigger;

    #[derive(Debug, Trigger)]
    #[name = "blackholeStrength"]
    pub struct BlackholeStrength {
        #[name = "strength"]
        pub strength: ResolvableString,
    }

    #[derive(Debug, Trigger)]
    #[name = "cameraOffsetTrigger"]
    pub struct CameraOffsetTrigger {
        #[name = "cameraX"]
        pub camerax: Float,
        #[name = "cameraY"]
        pub cameray: Float,
    }

    #[derive(Debug, Trigger)]
    #[name = "changeRespawnTrigger"]
    pub struct ChangeRespawnTrigger {
        #[node]
        pub node: Option<Node>,
    }

    #[derive(Debug, Trigger)]
    #[name = "minitextboxTrigger"]
    pub struct MiniTextboxTrigger {
        #[name = "mode"]
        pub mode: ResolvableString,
        #[name = "dialog_id"]
        pub dialog_id: ResolvableString,
        #[name = "only_once"]
        pub only_once: bool,
        #[name = "death_count"]
        pub death_count: Integer,
    }

    #[derive(Debug, Trigger)]
    #[name = "respawnTargetTrigger"]
    pub struct RespawnTargetTrigger {
        #[node]
        pub node: Node,
    }

    #[derive(Debug, Trigger)]
    #[name = "noRefillTrigger"]
    pub struct NoRefillTrigger {
        #[name = "state"]
        pub state: bool,
    }

    #[derive(Debug, Trigger)]
    #[name = "bloomFadeTrigger"]
    pub struct BloomFadeTrigger {
        #[name = "bloomAddFrom"]
        pub bloomaddfrom: Float,
        #[name = "bloomAddTo"]
        pub bloomaddto: Float,
        #[name = "positionMode"]
        pub positionmode: ResolvableString,
    }

    #[derive(Debug, Trigger)]
    #[name = "cameraTargetTrigger"]
    pub struct CameraTargetTrigger {
        #[name = "lerpStrength"]
        pub lerpstrength: Float,
        #[name = "positionMode"]
        pub positionmode: ResolvableString,
        #[name = "xOnly"]
        pub xonly: bool,
        #[name = "yOnly"]
        pub yonly: bool,
        #[name = "deleteFlag"]
        pub deleteflag: Option<Character>,
        #[node]
        pub node: Node,
    }

    #[derive(Debug, Trigger)]
    #[name = "musicFadeTrigger"]
    pub struct MusicFadeTrigger {
        #[name = "direction"]
        pub direction: ResolvableString,
        #[name = "fadeA"]
        pub fadea: Float,
        #[name = "fadeB"]
        pub fadeb: Float,
        #[name = "parameter"]
        pub parameter: Option<ResolvableString>,
    }
    #[derive(Debug, Trigger, Default, Clone, Copy, PartialEq, Eq)]
    #[name = "goldenBerryCollectTrigger"]
    pub struct GoldenBerryCollectTrigger;

    #[derive(Debug, Trigger)]
    #[name = "cameraAdvanceTargetTrigger"]
    pub struct CameraAdvanceTargetTrigger {
        #[name = "lerpStrengthX"]
        pub lerpstrengthx: Float,
        #[name = "lerpStrengthY"]
        pub lerpstrengthy: Float,
        #[name = "positionModeX"]
        pub positionmodex: ResolvableString,
        #[name = "positionModeY"]
        pub positionmodey: ResolvableString,
        #[name = "xOnly"]
        pub xonly: bool,
        #[name = "yOnly"]
        pub yonly: bool,
        #[node]
        pub node: Node,
    }

    #[derive(Debug, Trigger)]
    #[name = "lightFadeTrigger"]
    pub struct LightFadeTrigger {
        #[name = "lightAddFrom"]
        pub lightaddfrom: Float,
        #[name = "lightAddTo"]
        pub lightaddto: Float,
        #[name = "positionMode"]
        pub positionmode: ResolvableString,
    }

    #[derive(Debug, Trigger)]
    #[name = "spawnFacingTrigger"]
    pub struct SpawnFacingTrigger {
        #[name = "facing"]
        pub facing: ResolvableString,
    }
    #[derive(Debug, Trigger, Default, Clone, Copy, PartialEq, Eq)]
    #[name = "windAttackTrigger"]
    pub struct WindAttackTrigger;

    #[derive(Debug, Trigger)]
    #[name = "ambienceParamTrigger"]
    pub struct AmbienceParamTrigger {
        #[name = "direction"]
        pub direction: ResolvableString,
        #[name = "parameter"]
        pub parameter: ResolvableString,
        #[name = "from"]
        pub from: Float,
        #[name = "to"]
        pub to: Float,
    }

    #[derive(Debug, Trigger)]
    #[name = "eventTrigger"]
    pub struct EventTrigger {
        #[name = "event"]
        pub event: ResolvableString,
        #[name = "onSpawn"]
        pub onspawn: Option<bool>,
    }

    #[derive(Debug, Trigger)]
    #[name = "detachFollowersTrigger"]
    pub struct DetachFollowersTrigger {
        #[name = "global"]
        pub global: bool,
        #[node]
        pub node: Node,
    }

    #[derive(Debug, Trigger, Default, Clone, Copy, PartialEq, Eq)]
    #[name = "checkpointBlockerTrigger"]
    pub struct CheckpointBlockerTrigger;

    #[derive(Debug, Trigger)]
    #[name = "altMusicTrigger"]
    pub struct AltMusicTrigger {
        #[name = "track"]
        pub track: ResolvableString,
        #[name = "resetOnLeave"]
        pub resetonleave: bool,
    }

    #[derive(Debug, Trigger)]
    #[name = "windTrigger"]
    pub struct WindTrigger {
        #[name = "pattern"]
        pub pattern: ResolvableString,
    }

    #[derive(Debug, Trigger)]
    #[name = "creditsTrigger"]
    pub struct CreditsTrigger {
        #[name = "event"]
        pub event: ResolvableString,
    }

    #[derive(Debug, Trigger)]
    #[name = "rumbleTrigger"]
    pub struct RumbleTrigger {
        #[name = "manualTrigger"]
        pub manualtrigger: bool,
        #[name = "persistent"]
        pub persistent: bool,
        #[node]
        pub node: Node,
    }

    #[derive(Debug, Trigger)]
    #[name = "oshiroTrigger"]
    pub struct OshiroTrigger {
        #[name = "state"]
        pub state: bool,
    }

    #[derive(Debug, Trigger)]
    #[name = "moonGlitchBackgroundTrigger"]
    pub struct MoonGlitchBackgroundTrigger {
        #[name = "duration"]
        pub duration: ResolvableString,
        #[name = "stay"]
        pub stay: bool,
        #[name = "glitch"]
        pub glitch: bool,
    }

    #[derive(Debug, Trigger)]
    #[name = "interactTrigger"]
    pub struct InteractTrigger {
        #[name = "event"]
        pub event: ResolvableString,
        #[name = "event_2"]
        pub event_2: ResolvableString,
        #[name = "event_3"]
        pub event_3: ResolvableString,
        #[node]
        pub node: Option<Node>,
    }
}
