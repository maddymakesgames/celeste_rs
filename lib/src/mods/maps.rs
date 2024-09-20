use std::collections::HashMap;

use crate::{maps::elements::MapRoot, saves::session::CoreModes};

pub struct BundledMap {
    pub meta: MapMeta,
    pub altsides_meta: Option<AltSidesMeta>,
    pub map: MapRoot,
}

pub struct MapMeta {
    pub parent: String,

    pub icon: String,

    pub interlude: Option<bool>,

    pub cassete_checkpoint_idx: Option<i32>,

    pub title_base_color: String,
    pub title_accent_color: String,
    pub title_text_color: String,

    pub intro_type: Option<IntroTypes>,

    pub dreaming: Option<bool>,

    pub color_grade: String,

    pub wipe: String,

    pub darkness_alpha: Option<f32>,
    pub bloom_base: Option<f32>,
    pub bloom_strength: Option<f32>,

    pub jump_thru: String,

    pub core_mode: Option<CoreModes>,

    pub cassette_note_color: String,
    pub cassette_song: String,

    pub poscard_sound_id: String,

    pub foreground_tiles: String,
    pub background_tile: String,
    pub animated_tiles: String,
    pub sprites: String,
    pub portraits: String,

    pub override_a_side_meta: Option<bool>,

    pub map_meta_mode_properties: Vec<MapMetaModeProperty>,

    pub mountain: MountainData,

    pub complete_screen: CompleteScreen,
    pub loding_vignette_screen: CompleteScreen,

    pub loading_vignette_text: TextVignette,

    pub cassette_modifier: CassetteModifier,
}

pub struct MapMetaModeProperty {
    pub audio_state: MapMetaAudioState,
    pub checkpoints: Vec<MapMetaCheckpointData>,
    pub ignore_level_audio_layer_data: Option<bool>,
    pub inventory: String,
    pub path: String,
    pub poem_id: String,

    pub start_level: String,
    pub heart_is_end: Option<bool>,
    pub seeker_slowdown: Option<bool>,
    pub theo_in_bubble: Option<bool>,
}

pub struct MapMetaAudioState {
    pub music: String,
    pub ambience: String,
}

pub struct MapMetaCheckpointData {
    pub level: String,
    pub name: String,
    pub dreaming: Option<bool>,
    pub inventory: String,
    pub audio_state: MapMetaAudioState,
    pub flags: Vec<String>,
    pub core_mode: Option<CoreModes>,
}

pub struct MountainData {
    pub mountain_model_directory: Option<String>,
    pub mountain_texture_directory: Option<String>,
    pub background_music: Option<String>,
    pub background_ambience: Option<String>,
    pub background_music_params: Option<HashMap<String, f32>>,
    pub fog_colors: Option<Vec<String>>,
    pub star_fog_color: Option<String>,
    pub star_stream_colors: Vec<u32>,
    pub star_belt_colors1: Vec<u32>,
    pub start_belt_colors2: Vec<u32>,
    pub idle: Option<MountainPosition>,
    pub select: Option<MountainPosition>,
    pub zoom: Option<MountainPosition>,
    pub cursor: Option<[f32; 3]>,
    pub state: u32,
    pub rotate: bool,
    pub show_core: bool,
    pub show_snow: bool,
}

pub struct CompleteScreen {
    pub atlas: String,
    pub start: [f32; 2],
    pub center: [f32; 2],
    pub offset: [f32; 2],
    pub layers: Vec<CompleteScreenLayer>,
    pub music_by_side: Vec<String>,
    pub title: CompleteScreenTitle,
}

pub struct CompleteScreenLayer {
    pub kind: String,
    pub images: Vec<String>,
    pub position: [f32; 2],
    pub scroll: [f32; 2],
    pub frame_rate: f32,
    pub alpha: f32,
    pub speed: [f32; 2],
    pub scale: f32,
    pub loop_frame: bool,
}

pub struct CompleteScreenTitle {
    pub a_side: String,
    pub b_side: String,
    pub c_side: String,
    pub full_clear: String,
}

pub struct TextVignette {
    pub dialog: String,
    pub audio: String,
    pub initial_delay: f32,
    pub final_delay: f32,
    pub snow_direction: [f32; 2],
}

pub struct CassetteModifier {
    pub tempo_mult: f32,
    pub lead_beats: i32,
    pub beats_per_tick: i32,
    pub ticks_per_swap: i32,
    pub blocks: i32,
    pub beats_max: i32,
    pub beat_index_offset: i32,
    pub active_during_transitions: bool,
    pub old_behavior: bool,
}

pub struct MountainPosition {
    pub position: [f32; 3],
    pub target: [f32; 3],
}

pub struct AltSidesMeta {}

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
