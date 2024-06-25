use celeste_rs_macros::MapElement;

use crate::maps::{
    var_types::{Float, Integer},
    DynMapElement,
    MapElement,
    ResolvableString,
};

#[derive(Debug, MapElement)]
#[name = "levels"]
pub struct Levels {
    #[child]
    pub levels: Vec<Level>,
}

#[derive(Debug, MapElement)]
#[name = "level"]
pub struct Level {
    #[name = "name"]
    pub name: ResolvableString,
    #[name = "x"]
    pub x: Float,
    #[name = "y"]
    pub y: Float,
    #[name = "width"]
    pub width: Integer,
    #[name = "height"]
    pub height: Integer,
    #[name = "windPattern"]
    pub wind_pattern: ResolvableString,
    #[name = "dark"]
    pub dark: bool,
    #[name = "cameraOffsetX"]
    pub camera_offset_x: Integer,
    #[name = "cameraOffsetY"]
    pub camera_offset_y: Integer,
    #[name = "alt_music"]
    pub alt_music: ResolvableString,
    #[name = "music"]
    pub music: ResolvableString,
    #[name = "musicLayer1"]
    pub music_layer_1: bool,
    #[name = "musicLayer2"]
    pub music_layer_2: bool,
    #[name = "musicLayer3"]
    pub music_layer_3: bool,
    #[name = "musicLayer4"]
    pub music_layer_4: bool,
    #[name = "musicProgress"]
    pub music_progress: Option<ResolvableString>,
    #[name = "ambience"]
    pub ambience: Option<ResolvableString>,
    #[name = "ambienceProgress"]
    pub ambience_progress: Option<ResolvableString>,
    #[name = "underwater"]
    pub underwater: bool,
    #[name = "space"]
    pub space: Option<bool>,
    #[name = "disableDownTransition"]
    pub disable_down_transition: Option<bool>,
    #[name = "whisper"]
    pub whisper: Option<bool>,
    #[name = "delayAltMusicFade"]
    pub delay_alt_music_fade: Option<bool>,
    #[name = "enforceDashNumber"]
    pub enforce_dash_number: Option<Integer>,
    #[name = "c"]
    pub c: Integer,
    #[child]
    pub entities: Entities,
    #[child]
    pub solids: Solids,
    #[child]
    pub triggers: Triggers,
    #[child]
    pub fg_tiles: FGTiles,
    #[child]
    pub fg_decals: FGDecals,
    #[child]
    pub bg_tiles: BGTiles,
    #[child]
    pub bg_decals: BGDecals,
    #[child]
    pub bg: Background,
    #[child]
    pub objtiles: ObjTiles,
}
#[derive(Debug, MapElement)]
#[name = "triggers"]
pub struct Triggers {
    #[name = "offsetX"]
    pub offset_x: Float,
    #[name = "offsetY"]
    pub offset_y: Float,
    #[dyn_child]
    pub triggers: Vec<DynMapElement>,
}

#[derive(Debug, MapElement)]
#[name = "fgtiles"]
pub struct FGTiles {
    #[name = "offsetX"]
    pub offset_x: Float,
    #[name = "offsetY"]
    pub offset_y: Float,
    #[name = "tileset"]
    pub tileset: ResolvableString,
    #[name = "exportMode"]
    pub export_mode: Integer,
    #[name = "innerText"]
    pub inner_text: Option<String>,
}

#[derive(Debug, MapElement)]
#[name = "bgtiles"]
pub struct BGTiles {
    #[name = "offsetX"]
    pub offset_x: Float,
    #[name = "offsetY"]
    pub offset_y: Float,
    #[name = "tileset"]
    pub tileset: ResolvableString,
    #[name = "exportMode"]
    pub export_mode: Integer,
}

#[derive(Debug, MapElement)]
#[name = "bgdecals"]
pub struct BGDecals {
    #[name = "offsetX"]
    pub offset_x: Float,
    #[name = "offsetY"]
    pub offset_y: Float,
    #[child]
    pub decals: Vec<Decal>,
}


#[derive(Debug, MapElement)]
#[name = "fgdecals"]
pub struct FGDecals {
    #[name = "offsetX"]
    pub offset_x: Float,
    #[name = "offsetY"]
    pub offset_y: Float,
    #[child]
    pub decals: Vec<Decal>,
}

#[derive(Clone, Debug, MapElement)]
#[name = "decal"]
pub struct Decal {
    #[name = "x"]
    pub x: Float,
    #[name = "y"]
    pub y: Float,
    #[name = "scaleX"]
    pub scale_x: Float,
    #[name = "scaleY"]
    pub scale_y: Float,
    #[name = "rotation"]
    pub rotation: Option<Float>,
    #[name = "texture"]
    pub texture: ResolvableString,
}

#[derive(Debug, MapElement)]
#[name = "bg"]
pub struct Background {
    #[name = "offsetX"]
    pub offset_x: Float,
    #[name = "offsetY"]
    pub offset_y: Float,
    #[name = "innerText"]
    pub inner_text: String,
}

#[derive(Debug, MapElement)]
#[name = "objtiles"]
pub struct ObjTiles {
    #[name = "offsetX"]
    pub offset_x: Float,
    #[name = "offsetY"]
    pub offset_y: Float,
    #[name = "tileset"]
    pub tileset: ResolvableString,
    #[name = "exportMode"]
    pub export_mode: Integer,
    #[name = "innerText"]
    pub inner_text: Option<String>,
}

#[derive(Debug, MapElement)]
#[name = "solids"]
pub struct Solids {
    #[name = "offsetX"]
    pub offset_x: Float,
    #[name = "offsetY"]
    pub offset_y: Float,
    #[name = "innerTex"]
    pub inner_text: String,
}

#[derive(Debug, MapElement)]
#[name = "entities"]
pub struct Entities {
    #[name = "offsetX"]
    pub offset_x: Float,
    #[name = "offsetY"]
    pub offset_y: Float,
    #[dyn_child]
    pub entities: Vec<DynMapElement>,
}
