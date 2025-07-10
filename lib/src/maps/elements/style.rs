use celeste_rs_macros::MapElement;

use crate::{
    maps::{MapElement, MapElementParsingError, MapEncoder, MapParser, ResolvableString},
    utils::num::Float,
};

#[derive(Debug, MapElement)]
#[name = "Style"]
pub struct Styles {
    #[child]
    pub background: Backgrounds,
    #[child]
    pub foreground: Foregrounds,
}

#[derive(Debug)]
pub struct Backgrounds {
    pub parallax_elements: Vec<Parallax>,
    pub snow_bg: bool,
}

impl MapElement for Backgrounds {
    const NAME: &'static str = "Backgrounds";

    fn from_raw(parser: MapParser) -> Result<Self, MapElementParsingError> {
        Ok(Self {
            parallax_elements: parser.parse_all_elements()?,
            snow_bg: parser.parse_element::<SnowBG>().is_ok(),
        })
    }

    fn to_raw(&self, encoder: &mut MapEncoder) {
        encoder.children(&self.parallax_elements);

        if self.snow_bg {
            encoder.child(&SnowBG);
        }
    }
}

#[derive(Debug)]
pub struct Foregrounds {
    pub parallax_elements: Vec<Parallax>,
    pub snow_fg: bool,
}

impl MapElement for Foregrounds {
    const NAME: &'static str = "Foregrounds";

    fn from_raw(parser: MapParser) -> Result<Self, MapElementParsingError> {
        Ok(Self {
            parallax_elements: parser.parse_all_elements()?,
            snow_fg: parser.parse_element::<SnowFG>().is_ok(),
        })
    }

    fn to_raw(&self, encoder: &mut MapEncoder) {
        if self.snow_fg {
            encoder.child(&SnowFG);
        }

        encoder.children(&self.parallax_elements);
    }
}

#[derive(Debug, MapElement)]
#[name = "parallax"]
pub struct Parallax {
    #[name = "blendmode"]
    pub blend_mode: Option<ResolvableString>,
    #[name = "texture"]
    pub texture: ResolvableString,
    #[name = "x"]
    pub x: Option<Float>,
    #[name = "y"]
    pub y: Option<Float>,
    #[name = "scrollx"]
    pub scroll_x: Option<Float>,
    #[name = "scrolly"]
    pub scroll_y: Option<Float>,
    #[name = "loopx"]
    pub loopx: Option<bool>,
    #[name = "loopy"]
    pub loopy: Option<bool>,
    #[name = "speedx"]
    pub speed_x: Option<Float>,
    #[name = "speedy"]
    pub speed_y: Option<Float>,
    #[name = "color"]
    pub color: Option<ResolvableString>,
    #[name = "alpha"]
    pub alpha: Option<Float>,
}

#[derive(Debug, Clone, Copy, MapElement)]
#[name = "snowBg"]
pub struct SnowBG;

#[derive(Debug, Clone, Copy, MapElement)]
#[name = "snowFg"]
pub struct SnowFG;
