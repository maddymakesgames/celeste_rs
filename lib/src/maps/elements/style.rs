use crate::maps::{
    var_types::Float,
    MapElement,
    MapElementParsingError,
    MapEncoder,
    MapParser,
    ResolvableString,
};

#[derive(Debug)]
pub struct Styles {
    pub background: Backgrounds,
    pub foreground: Foregrounds,
}

impl MapElement for Styles {
    const NAME: &'static str = "Style";

    fn from_raw(parser: MapParser) -> Result<Self, MapElementParsingError>
    where Self: Sized {
        Ok(Self {
            background: parser.parse_element()?,
            foreground: parser.parse_element()?,
        })
    }

    fn to_raw(&self, encoder: &mut MapEncoder) {
        encoder.child(&self.background);
        encoder.child(&self.foreground);
    }
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

#[derive(Debug, Clone)]
pub struct Parallax {
    pub blend_mode: Option<ResolvableString>,
    pub texture: ResolvableString,
    pub x: Float,
    pub y: Float,
    pub scroll_x: Float,
    pub scroll_y: Float,
    pub loopx: bool,
    pub loopy: bool,
    pub speed_x: Option<Float>,
    pub speed_y: Option<Float>,
    pub color: Option<ResolvableString>,
    pub alpha: Option<Float>,
}

impl MapElement for Parallax {
    const NAME: &'static str = "parallax";

    fn from_raw(parser: MapParser) -> Result<Self, MapElementParsingError> {
        Ok(Self {
            blend_mode: parser.get_optional_attribute("blendmode"),
            texture: parser.get_attribute("texture")?,
            x: parser.get_attribute("x")?,
            y: parser.get_attribute("y")?,
            scroll_x: parser.get_attribute("scrollx")?,
            scroll_y: parser.get_attribute("scrolly")?,
            loopx: parser.get_attribute("loopx")?,
            loopy: parser.get_attribute("loopy")?,
            speed_x: parser.get_optional_attribute("speedx"),
            speed_y: parser.get_optional_attribute("speedy"),
            color: parser.get_optional_attribute("color"),
            alpha: parser.get_optional_attribute("alpha"),
        })
    }

    fn to_raw(&self, encoder: &mut MapEncoder) {
        encoder.optional_attribute("blendmode", &self.blend_mode);
        encoder.attribute("texture", self.texture.clone());
        encoder.attribute("x", self.x);
        encoder.attribute("y", self.y);
        encoder.attribute("scrollx", self.scroll_x);
        encoder.attribute("scrolly", self.scroll_y);
        encoder.attribute("loopx", self.loopx);
        encoder.attribute("loopy", self.loopy);
        encoder.optional_attribute("speedx", &self.speed_x);
        encoder.optional_attribute("speedy", &self.speed_y);
        encoder.optional_attribute("color", &self.color);
        encoder.optional_attribute("alpha", &self.alpha);
    }
}

#[derive(Debug, Clone)]
pub struct SnowBG;

impl MapElement for SnowBG {
    const NAME: &'static str = "snowBg";

    fn from_raw(_parser: MapParser) -> Result<Self, MapElementParsingError> {
        Ok(Self)
    }

    fn to_raw(&self, _encoder: &mut MapEncoder) {}
}

#[derive(Debug, Clone)]
pub struct SnowFG;

impl MapElement for SnowFG {
    const NAME: &'static str = "snowFg";

    fn from_raw(_parser: MapParser) -> Result<Self, MapElementParsingError> {
        Ok(Self)
    }

    fn to_raw(&self, _encoder: &mut MapEncoder) {}
}
