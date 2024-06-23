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
    fn name() -> &'static str
    where Self: Sized {
        "Style"
    }

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
    fn name() -> &'static str
    where Self: Sized {
        "Backgrounds"
    }

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
    fn name() -> &'static str
    where Self: Sized {
        "Foregrounds"
    }

    fn from_raw(parser: MapParser) -> Result<Self, MapElementParsingError> {
        Ok(Self {
            parallax_elements: parser.parse_all_elements()?,
            snow_fg: parser.parse_element::<SnowFG>().is_ok(),
        })
    }

    fn to_raw(&self, encoder: &mut MapEncoder) {
        encoder.children(&self.parallax_elements);

        if self.snow_fg {
            encoder.child(&SnowFG);
        }
    }
}

#[derive(Debug, Clone)]
pub struct Parallax {
    pub texture: ResolvableString,
    pub x: Float,
    pub y: Float,
    pub scroll_x: Float,
    pub scroll_y: Float,
    pub loopx: bool,
    pub loopy: bool,
}

impl MapElement for Parallax {
    fn name() -> &'static str
    where Self: Sized {
        "parallax"
    }

    fn from_raw(parser: MapParser) -> Result<Self, MapElementParsingError> {
        Ok(Self {
            texture: parser.get_attribute("texture")?,
            x: parser.get_attribute("x")?,
            y: parser.get_attribute("y")?,
            scroll_x: parser.get_attribute("scrollx")?,
            scroll_y: parser.get_attribute("scrolly")?,
            loopx: parser.get_attribute("loopx")?,
            loopy: parser.get_attribute("loopy")?,
        })
    }

    fn to_raw(&self, encoder: &mut MapEncoder) {
        encoder.attribute("texture", self.texture.clone());
        encoder.attribute("x", self.x);
        encoder.attribute("y", self.y);
        encoder.attribute("scrollx", self.scroll_x);
        encoder.attribute("scrolly", self.scroll_y);
        encoder.attribute("loopx", self.loopx);
        encoder.attribute("loopy", self.loopy);
    }
}

#[derive(Debug, Clone)]
pub struct SnowBG;

impl MapElement for SnowBG {
    fn name() -> &'static str
    where Self: Sized {
        "snowBg"
    }

    fn from_raw(_parser: MapParser) -> Result<Self, MapElementParsingError> {
        Ok(Self)
    }

    fn to_raw(&self, _encoder: &mut MapEncoder) {}
}

#[derive(Debug, Clone)]
pub struct SnowFG;

impl MapElement for SnowFG {
    fn name() -> &'static str
    where Self: Sized {
        "snowFg"
    }

    fn from_raw(_parser: MapParser) -> Result<Self, MapElementParsingError> {
        Ok(Self)
    }

    fn to_raw(&self, _encoder: &mut MapEncoder) {}
}
