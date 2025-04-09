use celeste_rs_macros::Entity;

use crate::maps::{
    MapElement,
    MapManager,
    ResolvableString,
    elements::Node,
    encoder::MapEncoder,
    parser::{MapElementParsingError, MapParser},
    var_types::{Character, Float, Integer},
};

use std::{any::Any, fmt::Debug};

#[derive(Debug)]
/// The full struct of an [Entity] implementaiton
///
/// These fields are on every entity element and are
/// useful even when we cannot resolve the entity itself
pub struct MapEntity<T: Entity> {
    pub id: Integer,
    pub x: Float,
    pub y: Float,
    pub width: Option<Integer>,
    pub height: Option<Integer>,
    pub origin_x: Float,
    pub origin_y: Float,
    pub entity: T,
}

impl<T: Entity> MapElement for MapEntity<T> {
    const NAME: &'static str = T::NAME;

    fn from_raw(parser: MapParser) -> Result<Self, MapElementParsingError>
    where Self: Sized {
        Ok(Self {
            id: parser.get_attribute("id")?,
            x: parser.get_attribute("x")?,
            y: parser.get_attribute("y")?,
            width: parser.get_optional_attribute("width")?,
            height: parser.get_optional_attribute("height")?,
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

/// Represents an entity, anything that can be stored in [Entities](super::Entities).
pub trait Entity: Debug + Any {
    /// The name of the entity in the binary file
    const NAME: &'static str;

    fn from_raw(parser: MapParser) -> Result<Self, MapElementParsingError>
    where Self: Sized;
    fn to_raw(&self, encoder: &mut MapEncoder);
}


macro_rules! unit_entities {
    ($($struct_name: ident, $name: literal),*) => {
        pub(super) fn add_unit_entity_parsers(mm: &mut MapManager) {
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

/// Adds parsers for all the vanilla entities to the [MapManager]
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
    mm.add_entity_parser::<Tentacles>();
    mm.add_entity_parser::<Glider>();
    mm.add_entity_parser::<FlingBirdIntro>();
    mm.add_entity_parser::<ExitBlock>();
    mm.add_entity_parser::<TempleCrackedBlock>();
    mm.add_entity_parser::<Clothesline>();
    mm.add_entity_parser::<RedBlocks>();
    mm.add_entity_parser::<Door>();
    mm.add_entity_parser::<DashSwitchV>();
    mm.add_entity_parser::<Spinner>();
    mm.add_entity_parser::<ConditionBlock>();
    mm.add_entity_parser::<FloatySpaceBlock>();
    mm.add_entity_parser::<SoundSource>();
    mm.add_entity_parser::<LockBlock>();
    mm.add_entity_parser::<Seeker>();
    mm.add_entity_parser::<DashSwitchH>();
    mm.add_entity_parser::<SummitCheckpoint>();
    mm.add_entity_parser::<SummitGem>();
    mm.add_entity_parser::<SummitGemManager>();
    mm.add_entity_parser::<FireBall>();
    mm.add_entity_parser::<FlingBird>();
    mm.add_entity_parser::<CoreModeToggle>();
    mm.add_entity_parser::<HeartGemDoor>();
    mm.add_entity_parser::<Eyebomb>();
    mm.add_entity_parser::<TempleGate>();
    mm.add_entity_parser::<PlaybackTutorial>();
    mm.add_entity_parser::<StarJumpBlock>();
    mm.add_entity_parser::<SeekerStatue>();
    mm.add_entity_parser::<FinalBoss>();
    mm.add_entity_parser::<Cloud>();
    mm.add_entity_parser::<BigWaterfall>();
    mm.add_entity_parser::<WallSpringLeft>();
    mm.add_entity_parser::<CoreMessage>();
    mm.add_entity_parser::<IntroCrusher>();
    mm.add_entity_parser::<Key>();
    mm.add_entity_parser::<HaHaHa>();
    mm.add_entity_parser::<BadelineBoost>();
    mm.add_entity_parser::<WallSpringRight>();
    mm.add_entity_parser::<CrumbleWallOnRumble>();
    mm.add_entity_parser::<RisingLava>();
    mm.add_entity_parser::<GreenBlocks>();
    mm.add_entity_parser::<LightningBlock>();
    mm.add_entity_parser::<BirdPath>();
    mm.add_entity_parser::<CutsceneNode>();
    mm.add_entity_parser::<ClutterDoor>();
    mm.add_entity_parser::<BigSpinner>();
    mm.add_entity_parser::<CliffsideFlag>();
    mm.add_entity_parser::<RidgeGate>();
    mm.add_entity_parser::<SwapBlock>();
    mm.add_entity_parser::<MovingPlatform>();
    mm.add_entity_parser::<SwitchGate>();
    mm.add_entity_parser::<BlackGem>();
    mm.add_entity_parser::<SummitBackgroundManager>();
    mm.add_entity_parser::<TempleMirror>();
    mm.add_entity_parser::<MoveBlock>();
    mm.add_entity_parser::<DreamBlock>();
    mm.add_entity_parser::<WallBooster>();
    mm.add_entity_parser::<Water>();
    mm.add_entity_parser::<Lightning>();
    mm.add_entity_parser::<MoonCreature>();
    mm.add_entity_parser::<FinalBossMovingBlock>();
    mm.add_entity_parser::<RotateSpinner>();
    mm.add_entity_parser::<Booster>();
    mm.add_entity_parser::<Bird>();
    mm.add_entity_parser::<ReflectionHeartStatue>();
    mm.add_entity_parser::<YellowBlocks>();
    mm.add_entity_parser::<TowerViewer>();
    mm.add_entity_parser::<Cobweb>();
    mm.add_entity_parser::<InfiniteStar>();
    mm.add_entity_parser::<Torch>();
    mm.add_entity_parser::<ColorSwitch>();
    mm.add_entity_parser::<CliffFlag>();
    mm.add_entity_parser::<Lamp>();
    mm.add_entity_parser::<PowerSourceNumber>();
    mm.add_entity_parser::<Bridge>();
    mm.add_entity_parser::<CrushBlock>();
    mm.add_entity_parser::<TrackSpinner>();
    mm.add_entity_parser::<Gondola>();

    add_unit_entity_parsers(mm);
}

use vanilla_entities::*;

#[allow(missing_docs)]
pub mod vanilla_entities {
    use super::*;


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
        pub bubble_points: Vec<Node>,
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
        pub tile_type: Character,
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
        pub tile_type: Character,
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
        pub nodes: Vec<Node>,
    }

    #[derive(Debug, Entity)]
    #[name = "fallingBlock"]
    pub struct FallingBlock {
        #[name = "tiletype"]
        pub tile_type: Character,
        #[name = "behind"]
        pub behind: Option<bool>,
        #[name = "climbFall"]
        pub climb_fall: Option<bool>,
    }

    #[derive(Debug, Entity)]
    #[name = "fakeWall"]
    pub struct FakeWall {
        #[name = "tiletype"]
        pub tile_type: Character,
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
        FlutterBird, "flutterbird",
        Plateau, "plateau",
        Payphone, "payphone",
        WavedashMachine, "wavedashmachine",
        PlaybackBillboard, "playbackBillboard",
        SinkingPlatform, "sinkingPlatform",
        PlayerSeeker, "playerSeeker",
        PICOConsole, "picoconsole",
        DarkChaser, "darkChaser",
        ResortRoofEnding, "resortRoofEnding",
        FireBarrier, "fireBarrier",
        TriggerSpikesUp, "triggerSpikesUp",
        TriggerSpikesDown, "triggerSpikesDown",
        IceBlock, "iceBlock",
        Waterfall, "waterfall",
        BounceBlock, "bounceBlock",
        ResortLantern, "resortLantern",
        KillBox, "killbox",
        TouchSwitch, "touchSwitch",
        ClutterCabinet, "clutterCabinet",
        TheoCrystal, "theoCrystal",
        BlockField, "blockField",
        InvisibleBarrier, "invisibleBarrier",
        TempleMirrorPortal, "templeMirrorPortal",
        TriggerSpikesRight, "triggerSpikesRight",
        BridgeFixed, "bridgeFixed",
        IntroCar, "introCar",
        ForegroundDebris, "foregroundDebris",
        OshiroDoor, "oshirodoor",
        TheoCrystalHoldingBarrier, "theoCrystalHoldingBarrier",
        TempleEye, "templeEye",
        TheoCrystalPedestal, "theoCrystalPedestal",
        FloatingDebris, "floatingDebris",
        TriggerSpikesLeft, "triggerSpikesLeft",
        KevinsPC, "kevins_pc",
        SandwichLava, "sandwichLava",
        Trapdoor, "trapdoor",
        SummitCloud, "summitcloud",
        FriendlyGhost, "friendlyGhost",
        FinalBossFallingBlock, "finalBossFallingBlock",
        DreamMirror, "dreammirror",
        SeekerBarrier, "seekerBarrier",
        TempleBigEyeball, "templeBigEyeball",
        ResortMirror, "resortmirror",
        WhiteBlock, "whiteblock",
        HangingLamp, "hanginglamp"
    }

    #[derive(Debug, Entity)]
    #[name = "tentacles"]
    pub struct Tentacles {
        #[name = "fear_distance"]
        pub fear_distance: ResolvableString,
        #[name = "slide_until"]
        pub slide_until: u8,
        #[node]
        pub node: Node,
    }

    #[derive(Debug, Entity)]
    #[name = "glider"]
    pub struct Glider {
        #[name = "bubble"]
        pub bubble: bool,
        #[name = "tutorial"]
        pub tutorial: bool,
    }

    #[derive(Debug, Entity)]
    #[name = "flingBirdIntro"]
    pub struct FlingBirdIntro {
        #[name = "crashes"]
        pub crashes: bool,
        #[node]
        pub node: Node,
    }

    #[derive(Debug, Entity)]
    #[name = "exitBlock"]
    pub struct ExitBlock {
        #[name = "tileType"]
        pub tiletype: Character,
    }

    #[derive(Debug, Entity)]
    #[name = "templeCrackedBlock"]
    pub struct TempleCrackedBlock {
        #[name = "persistent"]
        pub persistent: bool,
    }

    #[derive(Debug, Entity)]
    #[name = "clothesline"]
    pub struct Clothesline {
        #[node]
        pub node: Node,
    }

    #[derive(Debug, Entity)]
    #[name = "redBlocks"]
    pub struct RedBlocks {
        #[name = "inverted"]
        pub inverted: bool,
    }

    #[derive(Debug, Entity)]
    #[name = "door"]
    pub struct Door {
        #[name = "type"]
        pub kind: ResolvableString,
    }

    #[derive(Debug, Entity)]
    #[name = "dashSwitchV"]
    pub struct DashSwitchV {
        #[name = "ceiling"]
        pub ceiling: bool,
        #[name = "persistent"]
        pub persistent: bool,
        #[name = "sprite"]
        pub sprite: ResolvableString,
        #[name = "allGates"]
        pub allgates: Option<bool>,
    }

    #[derive(Debug, Entity)]
    #[name = "spinner"]
    pub struct Spinner {
        #[name = "attachToSolid"]
        pub attachtosolid: bool,
    }

    #[derive(Debug, Entity)]
    #[name = "conditionBlock"]
    pub struct ConditionBlock {
        #[name = "tileType"]
        pub tiletype: ResolvableString,
        #[name = "condition"]
        pub condition: ResolvableString,
        #[name = "conditionID"]
        pub conditionid: ResolvableString,
    }

    #[derive(Debug, Entity)]
    #[name = "floatySpaceBlock"]
    pub struct FloatySpaceBlock {
        #[name = "tiletype"]
        pub tiletype: Character,
        #[name = "disableSpawnOffset"]
        pub disablespawnoffset: bool,
    }

    #[derive(Debug, Entity)]
    #[name = "soundSource"]
    pub struct SoundSource {
        #[name = "sound"]
        pub sound: ResolvableString,
    }

    #[derive(Debug, Entity)]
    #[name = "lockBlock"]
    pub struct LockBlock {
        #[name = "stepMusicProgress"]
        pub stepmusicprogress: bool,
        #[name = "sprite"]
        pub sprite: ResolvableString,
        #[name = "unlock_sfx"]
        pub unlock_sfx: Option<ResolvableString>,
    }

    #[derive(Debug, Entity)]
    #[name = "seeker"]
    pub struct Seeker {
        #[node]
        pub node: Option<Node>,
    }

    #[derive(Debug, Entity)]
    #[name = "dashSwitchH"]
    pub struct DashSwitchH {
        #[name = "leftSide"]
        pub leftside: bool,
        #[name = "persistent"]
        pub persistent: bool,
        #[name = "sprite"]
        pub sprite: ResolvableString,
        #[name = "allGates"]
        pub allgates: Option<bool>,
    }

    #[derive(Debug, Entity)]
    #[name = "summitcheckpoint"]
    pub struct SummitCheckpoint {
        #[name = "number"]
        pub number: u8,
    }


    #[derive(Debug, Entity)]
    #[name = "summitgem"]
    pub struct SummitGem {
        #[name = "gem"]
        pub gem: u8,
    }

    #[derive(Debug, Entity)]
    #[name = "summitGemManager"]
    pub struct SummitGemManager {
        #[node]
        pub node: Node,
    }

    #[derive(Debug, Entity)]
    #[name = "fireBall"]
    pub struct FireBall {
        #[name = "amount"]
        pub amount: u8,
        #[name = "offset"]
        pub offset: Float,
        #[name = "speed"]
        pub speed: Float,
        #[name = "notCoreMode"]
        pub notcoremode: Option<bool>,
        #[node]
        pub node: Node,
    }

    #[derive(Debug, Entity)]
    #[name = "flingBird"]
    pub struct FlingBird {
        #[name = "waiting"]
        pub waiting: bool,
        #[node]
        pub node: Node,
    }

    #[derive(Debug, Entity)]
    #[name = "coreModeToggle"]
    pub struct CoreModeToggle {
        #[name = "onlyFire"]
        pub onlyfire: bool,
        #[name = "onlyIce"]
        pub onlyice: bool,
        #[name = "persistent"]
        pub persistent: bool,
    }

    #[derive(Debug, Entity)]
    #[name = "heartGemDoor"]
    pub struct HeartGemDoor {
        #[name = "requires"]
        pub requires: u8,
        #[name = "startHidden"]
        pub starthidden: Option<bool>,
        #[node]
        pub node: Node,
    }

    #[derive(Debug, Entity)]
    #[name = "eyebomb"]
    pub struct Eyebomb {
        #[name = "right"]
        pub right: bool,
    }

    #[derive(Debug, Entity)]
    #[name = "templeGate"]
    pub struct TempleGate {
        #[name = "type"]
        pub kind: ResolvableString,
        #[name = "sprite"]
        pub sprite: ResolvableString,
    }

    #[derive(Debug, Entity)]
    #[name = "playbackTutorial"]
    pub struct PlaybackTutorial {
        #[name = "tutorial"]
        pub tutorial: ResolvableString,
        #[node]
        pub node: Node,
    }

    #[derive(Debug, Entity)]
    #[name = "starJumpBlock"]
    pub struct StarJumpBlock {
        #[name = "sinks"]
        pub sinks: bool,
    }

    #[derive(Debug, Entity)]
    #[name = "seekerStatue"]
    pub struct SeekerStatue {
        #[name = "hatch"]
        pub hatch: ResolvableString,
        #[node]
        pub node: Node,
    }

    #[derive(Debug, Entity)]
    #[name = "finalBoss"]
    pub struct FinalBoss {
        #[name = "patternIndex"]
        pub patternindex: u8,
        #[name = "cameraPastY"]
        pub camerapasty: u8,
        #[name = "dialog"]
        pub dialog: bool,
        #[name = "startHit"]
        pub starthit: bool,
        #[name = "cameraLockY"]
        pub cameralocky: bool,
        #[node]
        pub node: Node,
    }

    #[derive(Debug, Entity)]
    #[name = "cloud"]
    pub struct Cloud {
        #[name = "fragile"]
        pub fragile: bool,
    }

    #[derive(Debug, Entity)]
    #[name = "bigWaterfall"]
    pub struct BigWaterfall {
        #[name = "layer"]
        pub layer: ResolvableString,
    }

    #[derive(Debug, Entity)]
    #[name = "wallSpringLeft"]
    pub struct WallSpringLeft {
        #[name = "playerCanUse"]
        pub playercanuse: Option<bool>,
    }


    #[derive(Debug, Entity)]
    #[name = "coreMessage"]
    pub struct CoreMessage {
        #[name = "line"]
        pub line: u8,
    }

    #[derive(Debug, Entity)]
    #[name = "introCrusher"]
    pub struct IntroCrusher {
        #[node]
        pub node: Node,
    }

    #[derive(Debug, Entity)]
    #[name = "key"]
    pub struct Key {
        #[node]
        pub node: Option<Node>,
    }

    #[derive(Debug, Entity)]
    #[name = "hahaha"]
    pub struct HaHaHa {
        #[name = "ifset"]
        pub ifset: ResolvableString,
        #[name = "triggerLaughSfx"]
        pub triggerlaughsfx: bool,
        #[node]
        pub node: Node,
    }

    #[derive(Debug, Entity)]
    #[name = "badelineBoost"]
    pub struct BadelineBoost {
        #[name = "lockCamera"]
        pub lockcamera: bool,
        #[name = "canSkip"]
        pub canskip: Option<bool>,
        #[name = "finalCh9Boost"]
        pub finalch9boost: Option<bool>,
        #[name = "finalCh9GoldenBoost"]
        pub finalch9goldenboost: Option<bool>,
        #[name = "finalCh9Dialog"]
        pub finalch9dialog: Option<bool>,
        #[node]
        pub node: Option<Node>,
    }

    #[derive(Debug, Entity)]
    #[name = "wallSpringRight"]
    pub struct WallSpringRight {
        #[name = "playerCanUse"]
        pub playercanuse: Option<bool>,
    }

    #[derive(Debug, Entity)]
    #[name = "crumbleWallOnRumble"]
    pub struct CrumbleWallOnRumble {
        #[name = "blendin"]
        pub blendin: bool,
        #[name = "persistent"]
        pub persistent: bool,
    }

    #[derive(Debug, Entity)]
    #[name = "risingLava"]
    pub struct RisingLava {
        #[name = "intro"]
        pub intro: bool,
    }

    #[derive(Debug, Entity)]
    #[name = "greenBlocks"]
    pub struct GreenBlocks {
        #[name = "inverted"]
        pub inverted: bool,
    }

    #[derive(Debug, Entity)]
    #[name = "lightningBlock"]
    pub struct LightningBlock {
        #[name = "flag"]
        pub flag: bool,
        #[name = "music"]
        pub music: ResolvableString,
        #[name = "music_progress"]
        pub music_progress: Integer,
        #[name = "music_session"]
        pub music_session: Option<bool>,
        #[name = "flipX"]
        pub flipx: bool,
    }

    #[derive(Debug, Entity)]
    #[name = "birdPath"]
    pub struct BirdPath {
        #[name = "only_once"]
        pub only_once: bool,
        #[name = "onlyIfLeft"]
        pub onlyifleft: bool,
        #[name = "speedMult"]
        pub speedmult: Option<Float>,
        #[node]
        pub node: Node,
    }

    #[derive(Debug, Entity)]
    #[name = "cutsceneNode"]
    pub struct CutsceneNode {
        #[name = "nodeName"]
        pub nodename: ResolvableString,
    }

    #[derive(Debug, Entity)]
    #[name = "clutterDoor"]
    pub struct ClutterDoor {
        #[name = "type"]
        pub kind: ResolvableString,
    }

    #[derive(Debug, Entity)]
    #[name = "bigSpinner"]
    pub struct BigSpinner {
        #[node]
        pub node: Option<Node>,
    }

    #[derive(Debug, Entity)]
    #[name = "cliffside_flag"]
    pub struct CliffsideFlag {
        #[name = "index"]
        pub index: u8,
    }

    #[derive(Debug, Entity)]
    #[name = "ridgeGate"]
    pub struct RidgeGate {
        #[name = "strawberries"]
        pub strawberries: ResolvableString,
        #[name = "keys"]
        pub keys: ResolvableString,
        #[node]
        pub node: Option<Node>,
    }

    #[derive(Debug, Entity)]
    #[name = "swapBlock"]
    pub struct SwapBlock {
        #[name = "theme"]
        pub theme: Option<ResolvableString>,
        #[node]
        pub node: Node,
    }

    #[derive(Debug, Entity)]
    #[name = "movingPlatform"]
    pub struct MovingPlatform {
        #[node]
        pub node: Node,
    }

    #[derive(Debug, Entity)]
    #[name = "switchGate"]
    pub struct SwitchGate {
        #[name = "persistent"]
        pub persistent: bool,
        #[name = "sprite"]
        pub sprite: Option<ResolvableString>,
        #[node]
        pub node: Node,
    }

    #[derive(Debug, Entity)]
    #[name = "blackGem"]
    pub struct BlackGem {
        #[name = "removeCameraTriggers"]
        pub removecameratriggers: Option<bool>,
        #[name = "fake"]
        pub fake: Option<bool>,
    }

    #[derive(Debug, Entity)]
    #[name = "SummitBackgroundManager"]
    pub struct SummitBackgroundManager {
        #[name = "index"]
        pub index: Integer,
        #[name = "intro_launch"]
        pub intro_launch: bool,
        #[name = "cutscene"]
        pub cutscene: Option<ResolvableString>,
        #[name = "dark"]
        pub dark: Option<bool>,
        #[name = "ambience"]
        pub ambience: Option<ResolvableString>,
    }

    #[derive(Debug, Entity)]
    #[name = "templeMirror"]
    pub struct TempleMirror {
        #[name = "reflectX"]
        pub reflectx: Integer,
        #[name = "reflectY"]
        pub reflecty: u8,
    }

    #[derive(Debug, Entity)]
    #[name = "moveBlock"]
    pub struct MoveBlock {
        #[name = "direction"]
        pub direction: ResolvableString,
        #[name = "canSteer"]
        pub cansteer: bool,
        #[name = "fast"]
        pub fast: bool,
    }

    #[derive(Debug, Entity)]
    #[name = "dreamBlock"]
    pub struct DreamBlock {
        #[name = "fastMoving"]
        pub fastmoving: Option<bool>,
        #[name = "oneUse"]
        pub oneuse: Option<bool>,
        #[name = "below"]
        pub below: Option<bool>,
        #[node]
        pub node: Option<Node>,
    }

    #[derive(Debug, Entity)]
    #[name = "wallBooster"]
    pub struct WallBooster {
        #[name = "left"]
        pub left: bool,
        #[name = "notCoreMode"]
        pub notcoremode: Option<bool>,
    }

    #[derive(Debug, Entity)]
    #[name = "water"]
    pub struct Water {
        #[name = "steamy"]
        pub steamy: bool,
        #[name = "hasBottom"]
        pub hasbottom: bool,
    }

    #[derive(Debug, Entity)]
    #[name = "lightning"]
    pub struct Lightning {
        #[name = "perLevel"]
        pub perlevel: bool,
        #[name = "moveTime"]
        pub movetime: Float,
        #[node]
        pub node: Option<Node>,
    }

    #[derive(Debug, Entity)]
    #[name = "moonCreature"]
    pub struct MoonCreature {
        #[name = "number"]
        pub number: u8,
    }

    #[derive(Debug, Entity)]
    #[name = "finalBossMovingBlock"]
    pub struct FinalBossMovingBlock {
        #[name = "nodeIndex"]
        pub nodeindex: u8,
        #[node]
        pub node: Node,
    }

    #[derive(Debug, Entity)]
    #[name = "rotateSpinner"]
    pub struct RotateSpinner {
        #[name = "clockwise"]
        pub clockwise: bool,
        #[node]
        pub node: Node,
    }

    #[derive(Debug, Entity)]
    #[name = "booster"]
    pub struct Booster {
        #[name = "red"]
        pub red: bool,
        #[name = "ch9_hub_booster"]
        pub ch9_hub_booster: Option<bool>,
    }

    #[derive(Debug, Entity)]
    #[name = "bird"]
    pub struct Bird {
        #[name = "mode"]
        pub mode: ResolvableString,
        #[name = "onlyOnce"]
        pub onlyonce: Option<bool>,
        #[name = "onlyIfPlayerLeft"]
        pub onlyifplayerleft: Option<bool>,
        #[node]
        pub node: Option<Node>,
    }

    #[derive(Debug, Entity)]
    #[name = "reflectionHeartStatue"]
    pub struct ReflectionHeartStatue {
        #[node]
        pub node: Node,
    }

    #[derive(Debug, Entity)]
    #[name = "yellowBlocks"]
    pub struct YellowBlocks {
        #[name = "inverted"]
        pub inverted: bool,
    }

    #[derive(Debug, Entity)]
    #[name = "towerviewer"]
    pub struct TowerViewer {
        #[name = "summit"]
        pub summit: Option<bool>,
        #[name = "onlyY"]
        pub onlyy: Option<bool>,
        #[node]
        pub node: Option<Node>,
    }

    #[derive(Debug, Entity)]
    #[name = "cobweb"]
    pub struct Cobweb {
        #[node]
        pub node: Node,
    }

    #[derive(Debug, Entity)]
    #[name = "infiniteStar"]
    pub struct InfiniteStar {
        #[name = "shielded"]
        pub shielded: bool,
        #[name = "singleUse"]
        pub singleuse: bool,
    }

    #[derive(Debug, Entity)]
    #[name = "torch"]
    pub struct Torch {
        #[name = "startLit"]
        pub startlit: bool,
    }

    #[derive(Debug, Entity)]
    #[name = "colorSwitch"]
    pub struct ColorSwitch {
        #[name = "type"]
        pub kind: ResolvableString,
    }

    #[derive(Debug, Entity)]
    #[name = "cliffflag"]
    pub struct CliffFlag {
        #[node]
        pub node: Node,
    }

    #[derive(Debug, Entity)]
    #[name = "lamp"]
    pub struct Lamp {
        #[name = "broken"]
        pub broken: bool,
    }

    #[derive(Debug, Entity)]
    #[name = "powerSourceNumber"]
    pub struct PowerSourceNumber {
        #[name = "number"]
        pub number: u8,
        #[name = "strawberries"]
        pub strawberries: ResolvableString,
        #[name = "keys"]
        pub keys: ResolvableString,
    }

    #[derive(Debug, Entity)]
    #[name = "bridge"]
    pub struct Bridge {
        #[node]
        pub node: Node,
    }


    #[derive(Debug, Entity)]
    #[name = "crushBlock"]
    pub struct CrushBlock {
        #[name = "axes"]
        pub axes: ResolvableString,
        #[name = "chillout"]
        pub chillout: bool,
    }


    #[derive(Debug, Entity)]
    #[name = "trackSpinner"]
    pub struct TrackSpinner {
        #[name = "startCenter"]
        pub startcenter: bool,
        #[name = "speed"]
        pub speed: ResolvableString,
        #[node]
        pub node: Node,
    }

    #[derive(Debug, Entity)]
    #[name = "gondola"]
    pub struct Gondola {
        #[name = "active"]
        pub active: bool,
        #[node]
        pub node: Node,
    }
}
