use crate::maps::{
    var_types::{Float, Integer},
    DynMapElement,
    EncodedVar,
    MapElement,
    MapElementParsingError,
    MapEncoder,
    MapParser,
    ResolvableString,
};

#[derive(Debug)]
pub struct Levels {
    pub levels: Vec<Level>,
}

impl MapElement for Levels {
    const NAME: &'static str = "levels";

    fn from_raw(parser: MapParser) -> Result<Self, MapElementParsingError>
    where Self: Sized {
        Ok(Self {
            levels: parser.parse_all_elements()?,
        })
    }

    fn to_raw(&self, encoder: &mut MapEncoder) {
        for level in &self.levels {
            encoder.child(level)
        }
    }
}

#[derive(Debug)]
pub struct Level {
    pub name: ResolvableString,
    pub x: Float,
    pub y: Float,
    pub width: Integer,
    pub height: Integer,
    pub wind_pattern: ResolvableString,
    pub dark: bool,
    pub camera_offset_x: Integer,
    pub camera_offset_y: Integer,
    pub alt_music: ResolvableString,
    pub music: ResolvableString,
    pub music_layer_1: bool,
    pub music_layer_2: bool,
    pub music_layer_3: bool,
    pub music_layer_4: bool,
    pub music_progress: Option<ResolvableString>,
    pub ambience: Option<ResolvableString>,
    pub ambience_progress: Option<ResolvableString>,
    pub underwater: bool,
    pub space: Option<bool>,
    pub disable_down_transition: Option<bool>,
    pub whisper: Option<bool>,
    pub delay_alt_music_fade: Option<bool>,
    pub c: Integer,
    pub entities: Entities,
    pub solids: Solids,
    pub triggers: Triggers,
    pub fg_tiles: FGTiles,
    pub fg_decals: FGDecals,
    pub bg_tiles: BGTiles,
    pub bg_decals: BGDecals,
}

impl MapElement for Level {
    const NAME: &'static str = "level";

    fn from_raw(parser: MapParser) -> Result<Self, MapElementParsingError> {
        Ok(Self {
            name: parser.get_attribute("name")?,
            x: parser.get_attribute("x")?,
            y: parser.get_attribute("y")?,
            width: parser.get_attribute("width")?,
            height: parser.get_attribute("height")?,
            wind_pattern: parser.get_attribute("windPattern")?,
            dark: parser.get_attribute("dark")?,
            camera_offset_x: parser.get_attribute("cameraOffsetX")?,
            camera_offset_y: parser.get_attribute("cameraOffsetY")?,
            alt_music: parser.get_attribute("alt_music")?,
            music: parser.get_attribute("music")?,
            music_layer_1: parser.get_attribute("musicLayer1")?,
            music_layer_2: parser.get_attribute("musicLayer2")?,
            music_layer_3: parser.get_attribute("musicLayer3")?,
            music_layer_4: parser.get_attribute("musicLayer4")?,
            music_progress: parser.get_optional_attribute("musicProgress"),
            ambience: parser.get_optional_attribute("ambience"),
            ambience_progress: parser.get_optional_attribute("ambienceProgress"),
            underwater: parser.get_attribute("underwater")?,
            space: parser.get_optional_attribute("space"),
            disable_down_transition: parser.get_optional_attribute("disableDownTransition"),
            whisper: parser.get_optional_attribute("whisper"),
            delay_alt_music_fade: parser.get_optional_attribute("delayAltMusicFade"),
            c: parser.get_attribute("c")?,
            entities: parser.parse_element()?,
            solids: parser.parse_element()?,
            triggers: parser.parse_element()?,
            fg_tiles: parser.parse_element()?,
            fg_decals: parser.parse_element()?,
            bg_tiles: parser.parse_element()?,
            bg_decals: parser.parse_element()?,
        })
    }

    fn to_raw(&self, encoder: &mut MapEncoder) {
        encoder.attribute("name", self.name.clone());
        encoder.attribute("width", self.width);
        encoder.attribute("height", self.height);
        encoder.attribute("windPattern", self.wind_pattern.clone());
        encoder.attribute("dark", self.dark);
        encoder.attribute("cameraOffsetX", self.camera_offset_x);
        encoder.attribute("cameraOffsetY", self.camera_offset_y);
        encoder.attribute("alt_music", self.alt_music.clone());
        encoder.attribute("music", self.music.clone());
        encoder.attribute("musicLayer1", self.music_layer_1);
        encoder.attribute("musicLayer2", self.music_layer_2);
        encoder.attribute("musicLayer3", self.music_layer_3);
        encoder.attribute("musicLayer4", self.music_layer_4);
        if let Some(progress) = &self.music_progress {
            encoder.attribute("musicProgress", progress.clone());
        }
        encoder.optional_attribute("ambience", &self.ambience);
        if let Some(progress) = &self.ambience_progress {
            encoder.attribute("musicProgress", progress.clone());
        }
        encoder.attribute("underwater", self.underwater);
        encoder.optional_attribute("space", &self.space);
        encoder.optional_attribute("disableDownTransition", &self.disable_down_transition);
        if let Some(whisper) = self.whisper {
            encoder.attribute("whisper", whisper);
        }

        if let Some(alt_music_fade) = self.delay_alt_music_fade {
            encoder.attribute("delayAltMusicFade", alt_music_fade);
        }
        encoder.attribute("x", self.x);
        encoder.attribute("y", self.y);
        encoder.attribute("c", self.c);
        encoder.child(&self.entities);
        encoder.child(&self.solids);
        encoder.child(&self.triggers);
        encoder.child(&self.fg_decals);
        encoder.child(&self.fg_tiles);
        encoder.child(&self.bg_decals);
        encoder.child(&self.bg_tiles);
    }
}


#[derive(Debug)]
pub struct Triggers {
    pub offset_x: Float,
    pub offset_y: Float,
    pub triggers: Vec<DynMapElement>,
}

impl MapElement for Triggers {
    const NAME: &'static str = "triggers";

    fn from_raw(parser: MapParser) -> Result<Self, MapElementParsingError>
    where Self: Sized {
        Ok(Self {
            offset_x: parser.get_attribute("offsetX")?,
            offset_y: parser.get_attribute("offsetY")?,
            triggers: parser.parse_any_element()?,
        })
    }

    fn to_raw(&self, encoder: &mut MapEncoder) {
        encoder.attribute("offsetX", self.offset_x);
        encoder.attribute("offsetY", self.offset_y);
        encoder.children(&self.triggers);
    }
}

#[derive(Debug)]
pub struct FGTiles {
    pub offset_x: Float,
    pub offset_y: Float,
    pub tileset: ResolvableString,
    pub export_mode: Integer,
}

impl MapElement for FGTiles {
    const NAME: &'static str = "fgtiles";

    fn from_raw(parser: MapParser) -> Result<Self, MapElementParsingError>
    where Self: Sized {
        Ok(Self {
            offset_x: parser.get_attribute("offsetX")?,
            offset_y: parser.get_attribute("offsetY")?,
            tileset: parser.get_attribute("tileset")?,
            export_mode: parser.get_attribute("exportMode")?,
        })
    }

    fn to_raw(&self, encoder: &mut MapEncoder) {
        encoder.attribute("offsetX", self.offset_x);
        encoder.attribute("offsetY", self.offset_y);
        encoder.attribute("tileset", self.tileset.clone());
        encoder.attribute("exportMode", self.export_mode);
    }
}

#[derive(Debug)]
pub struct BGTiles {
    pub offset_x: Float,
    pub offset_y: Float,
    pub tileset: ResolvableString,
    pub export_mode: Integer,
}

impl MapElement for BGTiles {
    const NAME: &'static str = "bgtiles";

    fn from_raw(parser: MapParser) -> Result<Self, MapElementParsingError>
    where Self: Sized {
        Ok(Self {
            offset_x: parser.get_attribute("offsetX")?,
            offset_y: parser.get_attribute("offsetY")?,
            tileset: parser.get_attribute("tileset")?,
            export_mode: parser.get_attribute("exportMode")?,
        })
    }

    fn to_raw(&self, encoder: &mut MapEncoder) {
        encoder.attribute("offsetX", self.offset_x);
        encoder.attribute("offsetY", self.offset_y);
        encoder.attribute("tileset", self.tileset.clone());
        encoder.attribute("exportMode", self.export_mode);
    }
}

#[derive(Debug)]
pub struct BGDecals {
    pub offset_x: Float,
    pub offset_y: Float,
    pub decals: Vec<Decal>,
}

impl MapElement for BGDecals {
    const NAME: &'static str = "bgdecals";

    fn from_raw(parser: MapParser) -> Result<Self, MapElementParsingError>
    where Self: Sized {
        Ok(Self {
            offset_x: parser.get_attribute("offsetX")?,
            offset_y: parser.get_attribute("offsetY")?,
            decals: parser.parse_all_elements()?,
        })
    }

    fn to_raw(&self, encoder: &mut MapEncoder) {
        encoder.attribute("offsetX", self.offset_x);
        encoder.attribute("offsetY", self.offset_y);
        encoder.children(&self.decals);
    }
}


#[derive(Debug)]
pub struct FGDecals {
    pub offset_x: Float,
    pub offset_y: Float,
    pub decals: Vec<Decal>,
}

impl MapElement for FGDecals {
    const NAME: &'static str = "fgdecals";

    fn from_raw(parser: MapParser) -> Result<Self, MapElementParsingError>
    where Self: Sized {
        Ok(Self {
            offset_x: parser.get_attribute("offsetX")?,
            offset_y: parser.get_attribute("offsetY")?,
            decals: parser.parse_all_elements()?,
        })
    }

    fn to_raw(&self, encoder: &mut MapEncoder) {
        encoder.attribute("offsetX", self.offset_x);
        encoder.attribute("offsetY", self.offset_y);
        encoder.children(&self.decals);
    }
}


#[derive(Clone, Debug)]
pub struct Decal {
    pub x: Float,
    pub y: Float,
    pub scale_x: Float,
    pub scale_y: Float,
    pub rotation: Option<Float>,
    pub texture: ResolvableString,
}

impl MapElement for Decal {
    const NAME: &'static str = "decal";

    fn from_raw(parser: MapParser) -> Result<Self, MapElementParsingError>
    where Self: Sized {
        Ok(Self {
            x: parser.get_attribute("x")?,
            y: parser.get_attribute("y")?,
            scale_x: parser.get_attribute("scaleX")?,
            scale_y: parser.get_attribute("scaleY")?,
            rotation: parser.get_optional_attribute("rotation"),
            texture: parser.get_attribute("texture")?,
        })
    }

    fn to_raw(&self, encoder: &mut MapEncoder) {
        encoder.attribute("x", self.x);
        encoder.attribute("y", self.y);
        encoder.attribute("scaleX", self.scale_x);
        encoder.attribute("scaleY", self.scale_y);
        encoder.optional_attribute("rotation", &self.rotation);
        encoder.attribute("texture", self.texture.clone());
    }
}

#[derive(Debug)]
pub struct Background {
    pub offset_x: Float,
    pub offset_y: Float,
    pub inner_text: String,
}

impl MapElement for Background {
    const NAME: &'static str = "bg";

    fn from_raw(parser: MapParser) -> Result<Self, MapElementParsingError>
    where Self: Sized {
        Ok(Self {
            offset_x: parser.get_attribute("offsetX")?,
            offset_y: parser.get_attribute("offsetY")?,
            inner_text: parser.get_attribute("innerText")?,
        })
    }

    fn to_raw(&self, encoder: &mut MapEncoder) {
        encoder.attribute("offsetX", self.offset_x);
        encoder.attribute("offsetY", self.offset_y);
        encoder.attribute("innerText", EncodedVar::new_rle_str(&self.inner_text));
    }
}

#[derive(Debug)]
pub struct ObjTiles {
    pub offset_x: Float,
    pub offset_y: Float,
    pub tileset: ResolvableString,
    pub export_mode: Integer,
    pub inner_text: Option<String>,
}

impl MapElement for ObjTiles {
    const NAME: &'static str = "objtiles";

    fn from_raw(parser: MapParser) -> Result<Self, MapElementParsingError>
    where Self: Sized {
        Ok(Self {
            offset_x: parser.get_attribute("offsetX")?,
            offset_y: parser.get_attribute("offsetY")?,
            tileset: parser.get_attribute("tileset")?,
            export_mode: parser.get_attribute("exportMode")?,
            inner_text: parser.get_optional_attribute("innerText"),
        })
    }

    fn to_raw(&self, encoder: &mut MapEncoder) {
        encoder.attribute("offsetX", self.offset_x);
        encoder.attribute("offsetY", self.offset_y);
        encoder.attribute("tileset", self.tileset.clone());
        encoder.optional_attribute(
            "innerText",
            &self.inner_text.as_ref().map(EncodedVar::new_rle_str),
        );
    }
}

#[derive(Clone, Debug)]
pub struct Solids {
    pub offset_x: Float,
    pub offset_y: Float,
    pub inner_text: String,
}

impl MapElement for Solids {
    const NAME: &'static str = "solids";

    fn from_raw(parser: MapParser) -> Result<Self, MapElementParsingError>
    where Self: Sized {
        Ok(Self {
            offset_x: parser.get_attribute("offsetX")?,
            offset_y: parser.get_attribute("offsetY")?,
            inner_text: parser.get_attribute("innerText")?,
        })
    }

    fn to_raw(&self, encoder: &mut MapEncoder) {
        encoder.attribute("offsetX", self.offset_x);
        encoder.attribute("offsetY", self.offset_y);
        encoder.attribute("innerText", EncodedVar::new_rle_str(&self.inner_text));
    }
}

#[derive(Debug)]
pub struct Entities {
    pub offset_x: Float,
    pub offset_y: Float,
    pub entities: Vec<DynMapElement>,
}

impl MapElement for Entities {
    const NAME: &'static str = "entities";

    fn from_raw(parser: MapParser) -> Result<Self, MapElementParsingError>
    where Self: Sized {
        Ok(Self {
            offset_x: parser.get_attribute("offsetX")?,
            offset_y: parser.get_attribute("offsetY")?,
            entities: parser.parse_any_element()?,
        })
    }

    fn to_raw(&self, encoder: &mut MapEncoder) {
        encoder.attribute("offsetX", self.offset_x);
        encoder.attribute("offsetY", self.offset_y);

        encoder.children(&self.entities)
    }
}
