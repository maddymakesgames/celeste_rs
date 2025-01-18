use std::collections::HashMap;

use celeste_rs_macros::FromYaml;

use crate::{maps::elements::MapRoot, saves::session::CoreModes};

pub struct BundledMap {
    pub meta: MapMeta,
    pub altsides_meta: Option<AltSidesMeta>,
    pub map: MapRoot,
}

#[derive(FromYaml)]
pub struct MapMeta {
    #[name = "Parent"]
    pub parent: String,

    #[name = "Icon"]
    pub icon: String,

    #[name = "Interlude"]
    pub interlude: Option<bool>,

    #[name = "CassetteCheckpointIndex"]
    pub cassette_checkpoint_idx: Option<i32>,

    #[name = "TitleBaseColor"]
    pub title_base_color: String,
    #[name = "TitleAccentColor"]
    pub title_accent_color: String,
    #[name = "TitleTextColor"]
    pub title_text_color: String,

    #[name = "IntroType"]
    pub intro_type: Option<IntroTypes>,

    #[name = "Dreaming"]
    pub dreaming: Option<bool>,

    #[name = "ColorGrade"]
    pub color_grade: String,

    #[name = "Wipe"]
    pub wipe: String,

    #[name = "DarknessAlpha"]
    pub darkness_alpha: Option<f32>,
    #[name = "BloomBase"]
    pub bloom_base: Option<f32>,
    #[name = "BloomStrength"]
    pub bloom_strength: Option<f32>,

    #[name = "JumpThru"]
    pub jump_thru: String,

    #[name = "CoreMode"]
    pub core_mode: Option<CoreModes>,

    #[name = "CassetteNoteColor"]
    pub cassette_note_color: String,
    #[name = "CassetteSong"]
    pub cassette_song: String,

    #[name = "PostcardSoundID"]
    pub postcard_sound_id: String,

    #[name = "ForegroundTiles"]
    pub foreground_tiles: String,
    #[name = "BackgroundTiles"]
    pub background_tile: String,
    #[name = "AnimatedTiles"]
    pub animated_tiles: String,
    #[name = "Sprites"]
    pub sprites: String,
    #[name = "Portraits"]
    pub portraits: String,

    #[name = "OverrideASideMeta"]
    pub override_a_side_meta: Option<bool>,

    #[name = "Modes"]
    pub map_meta_mode_properties: Vec<MapMetaModeProperty>,

    #[name = "Mountain"]
    pub mountain: MountainData,

    #[name = "CompleteScreen"]
    pub complete_screen: CompleteScreen,
    #[name = "LoadingVignetteScreen"]
    pub loding_vignette_screen: CompleteScreen,

    #[name = "LoadingVignetteText"]
    pub loading_vignette_text: TextVignette,

    #[name = "CassetteModifier"]
    pub cassette_modifier: CassetteModifier,
}

#[derive(FromYaml)]
pub struct MapMetaModeProperty {
    #[name = "AudioState"]
    pub audio_state: MapMetaAudioState,
    #[name = "Checkpoints"]
    pub checkpoints: Vec<MapMetaCheckpointData>,
    #[name = "IgnoreLevelAudioLayerData"]
    pub ignore_level_audio_layer_data: Option<bool>,
    #[name = "Inventory"]
    pub inventory: String,
    #[name = "Path"]
    pub path: String,
    #[name = "PoemID"]
    pub poem_id: String,

    #[name = "StartLevel"]
    pub start_level: String,
    #[name = "HeartIsEnd"]
    pub heart_is_end: Option<bool>,
    #[name = "SeekerSlowdown"]
    pub seeker_slowdown: Option<bool>,
    #[name = "TheoInBubble"]
    pub theo_in_bubble: Option<bool>,
}

#[derive(FromYaml)]
pub struct MapMetaAudioState {
    #[name = "Music"]
    pub music: String,
    #[name = "Ambience"]
    pub ambience: String,
}

#[derive(FromYaml)]
pub struct MapMetaCheckpointData {
    #[name = "Level"]
    pub level: String,
    #[name = "Name"]
    pub name: String,
    #[name = "Dreaming"]
    pub dreaming: Option<bool>,
    #[name = "Inventory"]
    pub inventory: String,
    #[name = "AudioState"]
    pub audio_state: MapMetaAudioState,
    #[name = "Flags"]
    pub flags: Vec<String>,
    #[name = "CoreMode"]
    pub core_mode: Option<CoreModes>,
}

#[derive(FromYaml)]
pub struct MountainData {
    #[name = "MountainModelDirectory"]
    pub mountain_model_directory: Option<String>,
    #[name = "MountainTextureDirectory"]
    pub mountain_texture_directory: Option<String>,
    #[name = "BackgroundMusic"]
    pub background_music: Option<String>,
    #[name = "BackgroundAmbience"]
    pub background_ambience: Option<String>,
    #[name = "BackgroundMusicParams"]
    pub background_music_params: Option<HashMap<String, f32>>,
    #[name = "FogColors"]
    pub fog_colors: Option<Vec<String>>,
    #[name = "StarFogColor"]
    pub star_fog_color: Option<String>,
    #[name = "StarStreamColors"]
    pub star_stream_colors: Vec<u32>,
    #[name = "StarBeltColors1"]
    pub star_belt_colors1: Vec<u32>,
    #[name = "StarBeltColors2"]
    pub start_belt_colors2: Vec<u32>,
    #[name = "Idle"]
    pub idle: Option<MountainPosition>,
    #[name = "Select"]
    pub select: Option<MountainPosition>,
    #[name = "Zoom"]
    pub zoom: Option<MountainPosition>,
    #[name = "Cursor"]
    pub cursor: Option<[f32; 3]>,
    #[name = "State"]
    pub state: u32,
    #[name = "Rotate"]
    pub rotate: bool,
    #[name = "ShowCore"]
    pub show_core: bool,
    #[name = "ShowSnow"]
    pub show_snow: bool,
}

#[derive(FromYaml)]
pub struct CompleteScreen {
    #[name = "Atlas"]
    pub atlas: String,
    #[name = "Start"]
    pub start: [f32; 2],
    #[name = "Center"]
    pub center: [f32; 2],
    #[name = "Offset"]
    pub offset: [f32; 2],
    #[name = "Layers"]
    pub layers: Vec<CompleteScreenLayer>,
    #[name = "MusicBySide"]
    pub music_by_side: Vec<String>,
    #[name = "Title"]
    pub title: CompleteScreenTitle,
}

#[derive(FromYaml)]
pub struct CompleteScreenLayer {
    #[name = "Type"]
    pub kind: String,
    #[name = "Images"]
    pub images: Vec<String>,
    #[name = "Position"]
    pub position: [f32; 2],
    #[name = "Scroll"]
    pub scroll: [f32; 2],
    #[name = "FrameRate"]
    pub frame_rate: f32,
    #[name = "Alpha"]
    pub alpha: f32,
    #[name = "Speed"]
    pub speed: [f32; 2],
    #[name = "Scale"]
    pub scale: f32,
    #[name = "Loop"]
    pub loop_frame: bool,
}

#[derive(FromYaml)]
pub struct CompleteScreenTitle {
    #[name = "ASide"]
    pub a_side: String,
    #[name = "BSide"]
    pub b_side: String,
    #[name = "CSide"]
    pub c_side: String,
    #[name = "FullClear"]
    pub full_clear: String,
}

#[derive(FromYaml)]
pub struct TextVignette {
    #[name = "Dialog"]
    pub dialog: String,
    #[name = "Audio"]
    pub audio: String,
    #[name = "InitialDelay"]
    pub initial_delay: f32,
    #[name = "FinalDelay"]
    pub final_delay: f32,
    #[name = "SnowDirection"]
    pub snow_direction: [f32; 2],
}

#[derive(FromYaml)]
pub struct CassetteModifier {
    #[name = "TempoMul"]
    pub tempo_mult: i32,
    #[name = "LeadBeats"]
    pub lead_beats: i32,
    #[name = "BeatsPerTick"]
    pub beats_per_tick: i32,
    #[name = "TicksPerSwap"]
    pub ticks_per_swap: i32,
    #[name = "Blocks"]
    pub blocks: i32,
    #[name = "BeatsMax"]
    pub beats_max: i32,
    #[name = "BeatIndexOffset"]
    pub beat_index_offset: i32,
    #[name = "ActiveDuringTransitions"]
    pub active_during_transitions: bool,
    #[name = "OldBehavior"]
    pub old_behavior: bool,
}

#[derive(FromYaml)]
pub struct MountainPosition {
    #[name = "Position"]
    pub position: [f64; 3],
    #[name = "Target"]
    pub target: [f64; 3],
}

pub struct AltSidesMeta {}

#[derive(FromYaml)]
pub enum IntroTypes {
    Transition,
    Respawn,
    WalkInRight,
    WalkInLeft,
    Jump,
    WakeUp,
    Fall,
    TempleMirrorVoid,
    None,
    ThinkForABit,
}
