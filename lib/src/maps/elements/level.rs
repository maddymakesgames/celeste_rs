use celeste_rs_macros::MapElement;

use crate::maps::{
    DynMapElement,
    MapElement,
    ResolvableString,
    var_types::{Float, Integer},
};

use super::entities::DynEntity;

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
    #[name = "width"]
    pub width: Integer,
    #[name = "height"]
    pub height: Integer,
    #[name = "windPattern"]
    pub wind_pattern: Option<ResolvableString>,
    #[name = "dark"]
    pub dark: Option<bool>,
    #[name = "cameraOffsetX"]
    pub camera_offset_x: Option<Float>,
    #[name = "cameraOffsetY"]
    pub camera_offset_y: Option<Float>,
    #[name = "alt_music"]
    pub alt_music: Option<ResolvableString>,
    #[name = "music"]
    pub music: Option<ResolvableString>,
    #[name = "musicLayer1"]
    pub music_layer_1: Option<bool>,
    #[name = "musicLayer2"]
    pub music_layer_2: Option<bool>,
    #[name = "musicLayer3"]
    pub music_layer_3: Option<bool>,
    #[name = "musicLayer4"]
    pub music_layer_4: Option<bool>,
    #[name = "musicProgress"]
    pub music_progress: Option<ResolvableString>,
    #[name = "ambience"]
    pub ambience: Option<ResolvableString>,
    #[name = "ambienceProgress"]
    pub ambience_progress: Option<ResolvableString>,
    #[name = "underwater"]
    pub underwater: Option<bool>,
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
    #[name = "x"]
    pub x: Float,
    #[name = "y"]
    pub y: Float,
    #[name = "c"]
    pub c: Integer,
    #[child]
    pub triggers: Option<Triggers>,
    #[child]
    pub fg_tiles: Option<FGTiles>,
    #[child]
    pub fg_decals: Option<FGDecals>,
    #[child]
    pub solids: Solids,
    #[child]
    pub entities: Option<Entities>,
    #[child]
    pub bg_tiles: Option<BGTiles>,
    #[child]
    pub bg_decals: Option<BGDecals>,
    #[child]
    pub bg: Background,
    #[child]
    pub objtiles: Option<ObjTiles>,
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
    pub offset_x: Option<Float>,
    #[name = "offsetY"]
    pub offset_y: Option<Float>,
    #[name = "innerText"]
    #[rle]
    pub inner_text: Option<String>,
}

#[derive(Debug, MapElement)]
#[name = "objtiles"]
pub struct ObjTiles {
    #[name = "offsetX"]
    pub offset_x: Option<Float>,
    #[name = "offsetY"]
    pub offset_y: Option<Float>,
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
    pub offset_x: Option<Float>,
    #[name = "offsetY"]
    pub offset_y: Option<Float>,
    #[name = "innerText"]
    #[rle]
    pub inner_text: Option<String>,
}

#[derive(Debug, MapElement)]
#[name = "entities"]
pub struct Entities {
    #[name = "offsetX"]
    pub offset_x: Option<Float>,
    #[name = "offsetY"]
    pub offset_y: Option<Float>,
    #[dyn_entities]
    pub entities: Vec<DynEntity>,
}
