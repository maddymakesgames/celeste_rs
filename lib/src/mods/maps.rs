use std::collections::HashMap;

use celeste_rs_macros::FromYaml;

use crate::{
    maps::elements::MapRoot,
    saves::session::CoreModes,
    utils::{YamlString, num::Float},
};

pub struct BundledMap {
    pub meta: Option<MapMeta>,
    pub altsides_meta: Option<AltSidesMeta>,
    pub map: MapRoot,
}

#[derive(FromYaml)]
pub struct MapMeta {
    #[name = "Parent"]
    pub parent: Option<String>,

    #[name = "Icon"]
    pub icon: Option<String>,

    #[name = "Interlude"]
    pub interlude: Option<bool>,

    #[name = "CassetteCheckpointIndex"]
    pub cassette_checkpoint_idx: Option<i32>,

    #[name = "TitleBaseColor"]
    pub title_base_color: Option<String>,
    #[name = "TitleAccentColor"]
    pub title_accent_color: Option<String>,
    #[name = "TitleTextColor"]
    pub title_text_color: Option<String>,

    #[name = "IntroType"]
    pub intro_type: Option<IntroTypes>,

    #[name = "Dreaming"]
    pub dreaming: Option<bool>,

    #[name = "ColorGrade"]
    pub color_grade: Option<String>,

    #[name = "Wipe"]
    pub wipe: Option<String>,

    #[name = "DarknessAlpha"]
    pub darkness_alpha: Option<f32>,
    #[name = "BloomBase"]
    pub bloom_base: Option<f32>,
    #[name = "BloomStrength"]
    pub bloom_strength: Option<f32>,

    #[name = "JumpThru"]
    pub jump_thru: Option<String>,

    #[name = "CoreMode"]
    pub core_mode: Option<CoreModes>,

    #[name = "CassetteNoteColor"]
    pub cassette_note_color: Option<String>,
    #[name = "CassetteSong"]
    pub cassette_song: Option<String>,

    #[name = "PostcardSoundID"]
    pub postcard_sound_id: Option<String>,

    #[name = "ForegroundTiles"]
    pub foreground_tiles: Option<String>,
    #[name = "BackgroundTiles"]
    pub background_tile: Option<String>,
    #[name = "AnimatedTiles"]
    pub animated_tiles: Option<String>,
    #[name = "Sprites"]
    pub sprites: Option<String>,
    #[name = "Portraits"]
    pub portraits: Option<String>,

    #[name = "OverrideASideMeta"]
    pub override_a_side_meta: Option<bool>,

    #[name = "Modes"]
    pub map_meta_mode_properties: Option<Vec<MapMetaModeProperty>>,

    #[name = "Mountain"]
    pub mountain: Option<MountainData>,

    #[name = "CompleteScreen"]
    pub complete_screen: Option<CompleteScreen>,
    #[name = "LoadingVignetteScreen"]
    pub loding_vignette_screen: Option<CompleteScreen>,

    #[name = "LoadingVignetteText"]
    pub loading_vignette_text: Option<TextVignette>,

    #[name = "CassetteModifier"]
    pub cassette_modifier: Option<CassetteModifier>,
}

#[derive(FromYaml)]
pub struct MapMetaModeProperty {
    #[name = "AudioState"]
    pub audio_state: MapMetaAudioState,
    #[name = "Checkpoints"]
    pub checkpoints: Option<Vec<MapMetaCheckpointData>>,
    #[name = "IgnoreLevelAudioLayerData"]
    pub ignore_level_audio_layer_data: Option<bool>,
    #[name = "Inventory"]
    pub inventory: String,
    #[name = "Path"]
    pub path: Option<String>,
    #[name = "PoemID"]
    pub poem_id: Option<String>,

    #[name = "StartLevel"]
    pub start_level: Option<String>,
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
    pub background_music_params: Option<HashMap<String, Float>>,
    #[name = "FogColors"]
    pub fog_colors: Option<Vec<YamlString>>,
    #[name = "StarFogColor"]
    pub star_fog_color: Option<YamlString>,
    #[name = "StarStreamColors"]
    pub star_stream_colors: Option<Vec<YamlString>>,
    #[name = "StarBeltColors1"]
    pub star_belt_colors1: Option<Vec<YamlString>>,
    #[name = "StarBeltColors2"]
    pub start_belt_colors2: Option<Vec<YamlString>>,
    #[name = "Idle"]
    pub idle: Option<MountainPosition>,
    #[name = "Select"]
    pub select: Option<MountainPosition>,
    #[name = "Zoom"]
    pub zoom: Option<MountainPosition>,
    #[name = "Cursor"]
    pub cursor: Option<[Float; 3]>,
    #[name = "State"]
    pub state: Option<u32>,
    #[name = "Rotate"]
    pub rotate: Option<bool>,
    #[name = "ShowCore"]
    pub show_core: Option<bool>,
    #[name = "ShowSnow"]
    pub show_snow: Option<bool>,
}

#[derive(FromYaml)]
pub struct CompleteScreen {
    #[name = "Atlas"]
    pub atlas: String,
    #[name = "Start"]
    pub start: Option<[Float; 2]>,
    #[name = "Center"]
    pub center: Option<[Float; 2]>,
    #[name = "Offset"]
    pub offset: Option<[Float; 2]>,
    #[name = "Layers"]
    pub layers: Vec<CompleteScreenLayer>,
    #[name = "MusicBySide"]
    pub music_by_side: Option<Vec<String>>,
    #[name = "Title"]
    pub title: Option<CompleteScreenTitle>,
}

#[derive(FromYaml)]
pub struct CompleteScreenLayer {
    #[name = "Type"]
    pub kind: String,
    #[name = "Images"]
    pub images: Option<Vec<String>>,
    #[name = "Position"]
    pub position: Option<[Float; 2]>,
    #[name = "Scroll"]
    pub scroll: Option<[Float; 1]>,
    #[name = "FrameRate"]
    pub frame_rate: Option<Float>,
    #[name = "Alpha"]
    pub alpha: Option<f32>,
    #[name = "Speed"]
    pub speed: Option<[f32; 2]>,
    #[name = "Scale"]
    pub scale: Option<Float>,
    #[name = "Loop"]
    pub loop_frame: Option<bool>,
}

#[derive(FromYaml)]
pub struct CompleteScreenTitle {
    #[name = "ASide"]
    pub a_side: String,
    #[name = "BSide"]
    pub b_side: Option<String>,
    #[name = "CSide"]
    pub c_side: Option<String>,
    #[name = "FullClear"]
    pub full_clear: Option<String>,
}

#[derive(FromYaml)]
pub struct TextVignette {
    #[name = "Dialog"]
    pub dialog: String,
    #[name = "Audio"]
    pub audio: Option<String>,
    #[name = "InitialDelay"]
    pub initial_delay: Option<f32>,
    #[name = "FinalDelay"]
    pub final_delay: Option<f32>,
    #[name = "SnowDirection"]
    pub snow_direction: Option<[f32; 2]>,
}

#[derive(FromYaml)]
pub struct CassetteModifier {
    #[name = "TempoMul"]
    pub tempo_mult: Option<i32>,
    #[name = "LeadBeats"]
    pub lead_beats: Option<i32>,
    #[name = "BeatsPerTick"]
    pub beats_per_tick: Option<i32>,
    #[name = "TicksPerSwap"]
    pub ticks_per_swap: Option<i32>,
    #[name = "Blocks"]
    pub blocks: Option<i32>,
    #[name = "BeatsMax"]
    pub beats_max: Option<i32>,
    #[name = "BeatIndexOffset"]
    pub beat_index_offset: Option<i32>,
    #[name = "ActiveDuringTransitions"]
    pub active_during_transitions: Option<bool>,
    #[name = "OldBehavior"]
    pub old_behavior: Option<bool>,
}

#[derive(FromYaml)]
pub struct MountainPosition {
    #[name = "Position"]
    pub position: [Float; 3],
    #[name = "Target"]
    pub target: [Float; 3],
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
