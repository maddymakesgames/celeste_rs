use std::{any::Any, fmt::Debug};

use celeste_rs_macros::Trigger;

use crate::maps::{
    encoder::MapEncoder,
    parser::{MapElementParsingError, MapParser},
    var_types::{Character, Float, Integer},
    MapElement,
    MapManager,
    Node,
    ResolvableString,
};

#[derive(Debug)]
pub struct MapTrigger<T: Trigger> {
    id: Integer,
    x: Float,
    y: Float,
    width: Option<Integer>,
    height: Option<Integer>,
    origin_x: Float,
    origin_y: Float,
    entity: T,
}

impl<T: Trigger> MapElement for MapTrigger<T> {
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


#[derive(Debug, Trigger, Default, Clone, Copy, PartialEq, Eq)]
#[name = "lookoutBlocker"]
pub struct LookoutBlocker;

#[derive(Debug, Trigger)]
#[name = "musicTrigger"]
pub struct MusicTrigger {
    #[name = "track"]
    track: ResolvableString,
    #[name = "resetFade"]
    resetfade: bool,
    #[name = "setInSession"]
    setinsession: bool,
    #[name = "resetOnLeave"]
    resetonleave: bool,
    #[name = "progress"]
    progress: u8,
}
#[derive(Debug, Trigger, Default, Clone, Copy, PartialEq, Eq)]
#[name = "birdPathTrigger"]
pub struct BirdPathTrigger;

#[derive(Debug, Trigger)]
#[name = "blackholeStrength"]
pub struct BlackholeStrength {
    #[name = "strength"]
    strength: ResolvableString,
}

#[derive(Debug, Trigger)]
#[name = "cameraOffsetTrigger"]
pub struct CameraOffsetTrigger {
    #[name = "cameraX"]
    camerax: Float,
    #[name = "cameraY"]
    cameray: Float,
}

#[derive(Debug, Trigger)]
#[name = "changeRespawnTrigger"]
pub struct ChangeRespawnTrigger {
    #[node]
    node: Option<Node>,
}

#[derive(Debug, Trigger)]
#[name = "minitextboxTrigger"]
pub struct MiniTextboxTrigger {
    #[name = "mode"]
    mode: ResolvableString,
    #[name = "dialog_id"]
    dialog_id: ResolvableString,
    #[name = "only_once"]
    only_once: bool,
    #[name = "death_count"]
    death_count: Integer,
}

#[derive(Debug, Trigger)]
#[name = "respawnTargetTrigger"]
pub struct RespawnTargetTrigger {
    #[node]
    node: Node,
}

#[derive(Debug, Trigger)]
#[name = "noRefillTrigger"]
pub struct NoRefillTrigger {
    #[name = "state"]
    state: bool,
}

#[derive(Debug, Trigger)]
#[name = "bloomFadeTrigger"]
pub struct BloomFadeTrigger {
    #[name = "bloomAddFrom"]
    bloomaddfrom: Float,
    #[name = "bloomAddTo"]
    bloomaddto: Float,
    #[name = "positionMode"]
    positionmode: ResolvableString,
}

#[derive(Debug, Trigger)]
#[name = "cameraTargetTrigger"]
pub struct CameraTargetTrigger {
    #[name = "lerpStrength"]
    lerpstrength: Float,
    #[name = "positionMode"]
    positionmode: ResolvableString,
    #[name = "xOnly"]
    xonly: bool,
    #[name = "yOnly"]
    yonly: bool,
    #[name = "deleteFlag"]
    deleteflag: Option<Character>,
    #[node]
    node: Node,
}

#[derive(Debug, Trigger)]
#[name = "musicFadeTrigger"]
pub struct MusicFadeTrigger {
    #[name = "direction"]
    direction: ResolvableString,
    #[name = "fadeA"]
    fadea: u8,
    #[name = "fadeB"]
    fadeb: u8,
    #[name = "parameter"]
    parameter: Option<ResolvableString>,
}
#[derive(Debug, Trigger, Default, Clone, Copy, PartialEq, Eq)]
#[name = "goldenBerryCollectTrigger"]
pub struct GoldenBerryCollectTrigger;

#[derive(Debug, Trigger)]
#[name = "cameraAdvanceTargetTrigger"]
pub struct CameraAdvanceTargetTrigger {
    #[name = "lerpStrengthX"]
    lerpstrengthx: u8,
    #[name = "lerpStrengthY"]
    lerpstrengthy: u8,
    #[name = "positionModeX"]
    positionmodex: ResolvableString,
    #[name = "positionModeY"]
    positionmodey: ResolvableString,
    #[name = "xOnly"]
    xonly: bool,
    #[name = "yOnly"]
    yonly: bool,
    #[node]
    node: Node,
}

#[derive(Debug, Trigger)]
#[name = "lightFadeTrigger"]
pub struct LightFadeTrigger {
    #[name = "lightAddFrom"]
    lightaddfrom: Float,
    #[name = "lightAddTo"]
    lightaddto: Float,
    #[name = "positionMode"]
    positionmode: ResolvableString,
}

#[derive(Debug, Trigger)]
#[name = "spawnFacingTrigger"]
pub struct SpawnFacingTrigger {
    #[name = "facing"]
    facing: ResolvableString,
}
#[derive(Debug, Trigger, Default, Clone, Copy, PartialEq, Eq)]
#[name = "windAttackTrigger"]
pub struct WindAttackTrigger;

#[derive(Debug, Trigger)]
#[name = "ambienceParamTrigger"]
pub struct AmbienceParamTrigger {
    #[name = "direction"]
    direction: ResolvableString,
    #[name = "parameter"]
    parameter: ResolvableString,
    #[name = "from"]
    from: Float,
    #[name = "to"]
    to: Float,
}

#[derive(Debug, Trigger)]
#[name = "eventTrigger"]
pub struct EventTrigger {
    #[name = "event"]
    event: ResolvableString,
    #[name = "onSpawn"]
    onspawn: Option<bool>,
}

#[derive(Debug, Trigger)]
#[name = "detachFollowersTrigger"]
pub struct DetachFollowersTrigger {
    #[name = "global"]
    global: bool,
    #[node]
    node: Node,
}

#[derive(Debug, Trigger, Default, Clone, Copy, PartialEq, Eq)]
#[name = "checkpointBlockerTrigger"]
pub struct CheckpointBlockerTrigger;

#[derive(Debug, Trigger)]
#[name = "altMusicTrigger"]
pub struct AltMusicTrigger {
    #[name = "track"]
    track: ResolvableString,
    #[name = "resetOnLeave"]
    resetonleave: bool,
}

#[derive(Debug, Trigger)]
#[name = "windTrigger"]
pub struct WindTrigger {
    #[name = "pattern"]
    pattern: ResolvableString,
}

#[derive(Debug, Trigger)]
#[name = "creditsTrigger"]
pub struct CreditsTrigger {
    #[name = "event"]
    event: ResolvableString,
}

#[derive(Debug, Trigger)]
#[name = "rumbleTrigger"]
pub struct RumbleTrigger {
    #[name = "manualTrigger"]
    manualtrigger: bool,
    #[name = "persistent"]
    persistent: bool,
    #[node]
    node: Node,
}

#[derive(Debug, Trigger)]
#[name = "oshiroTrigger"]
pub struct OshiroTrigger {
    #[name = "state"]
    state: bool,
}

#[derive(Debug, Trigger)]
#[name = "moonGlitchBackgroundTrigger"]
pub struct MoonGlitchBackgroundTrigger {
    #[name = "duration"]
    duration: ResolvableString,
    #[name = "stay"]
    stay: bool,
    #[name = "glitch"]
    glitch: bool,
}

#[derive(Debug, Trigger)]
#[name = "interactTrigger"]
pub struct InteractTrigger {
    #[name = "event"]
    event: ResolvableString,
    #[name = "event_2"]
    event_2: ResolvableString,
    #[name = "event_3"]
    event_3: ResolvableString,
    #[node]
    node: Option<Node>,
}
