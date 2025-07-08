use std::{
    fmt::Display,
    io::{Read, Write},
};

use crate::utils::binary::{BinReadError, BinReader, BinWriter};

/// A playback tutorial.
///
/// This is a list of frames that give the offset and actions
/// of the recorded player each frame.
///
/// This is used for the silhouette players in Farewell.
///
/// In mods, these are stored in the `Tutorials` directory
pub struct Playback {
    pub frames: Vec<PlaybackFrame>,
}

impl Playback {
    pub fn from_reader(mut reader: impl Read) -> Result<Self, PlaybackReadError> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;

        Self::from_bytes(&buf)
    }

    /// Reads a [Playback] from binary data
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, PlaybackReadError> {
        let mut reader = BinReader::new(bytes);

        let mut skip_scale = true;

        let header = reader.read_string();

        // We use is_ok_and because the varint length of the string could
        // be invalid and we don't want to cancel reading because of that
        if header.is_ok_and(|h| h == "TIMELINE") {
            skip_scale = reader.read_i32()? == 1;
        } else {
            reader.restart();
        }

        let frames = reader.read_i32()?;
        println!("frames: {frames}");
        let mut buf = Vec::with_capacity(frames as usize);

        for _ in 0 .. frames {
            buf.push(PlaybackFrame::from_reader(&mut reader, skip_scale)?);
        }

        Ok(Self { frames: buf })
    }

    fn write<T: Write>(&self, writer: &mut BinWriter<T>) -> std::io::Result<()> {
        if self.frames.is_empty() {
            return Ok(());
        }

        let first_frame = &self.frames[0];
        let base_timestamp = first_frame.timestamp;
        let base_position = first_frame.position;

        writer.write_string("TIMELINE")?;
        writer.write_i32(2)?;
        writer.write_i32(self.frames.len() as i32)?;

        for frame in &self.frames {
            frame.write(writer, base_position, base_timestamp)?;
        }
        Ok(())
    }

    /// Converts the [Playback] to binary data.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        let mut writer = BinWriter::new(&mut bytes);
        // There should be no write errors when writing to vec
        self.write(&mut writer).unwrap();
        bytes
    }

    /// Writes the [Playback] as binary to a writer.
    pub fn to_writer(&self, writer: impl Write) -> std::io::Result<()> {
        let mut bin_writer = BinWriter::new(writer);
        self.write(&mut bin_writer)
    }
}

/// A single frame of a [Playback]
pub struct PlaybackFrame {
    pub position: [f32; 2],
    pub timestamp: f32,
    pub animation: String,
    pub facing: PlaybackFacing,
    pub on_ground: bool,
    pub hair_color: [u8; 3],
    pub depth: i32,
    // Is always 0 but including it since technically its there
    pub sounds: i32,
    pub scale: [f32; 2],
    pub dash_direction: [f32; 2],
}

impl PlaybackFrame {
    fn from_reader(reader: &mut BinReader, skip_scale: bool) -> Result<Self, BinReadError> {
        let position = [reader.read_f32()?, reader.read_f32()?];
        let timestamp = reader.read_f32()?;
        let animation = reader.read_string()?;
        // TODO: make this an actual error
        let facing = PlaybackFacing::from_int(reader.read_i32()?).unwrap();
        let on_ground = reader.read_bool()?;
        let hair_color = [reader.read_u8()?, reader.read_u8()?, reader.read_u8()?];
        let depth = reader.read_i32()?;
        let sounds = 0;
        let scale;
        let dash_direction;

        if skip_scale {
            scale = [(facing as i32) as f32, 1.0];
            dash_direction = [0.0, 0.0];
        } else {
            scale = [reader.read_f32()?, reader.read_f32()?];
            dash_direction = [reader.read_f32()?, reader.read_f32()?];
        }

        Ok(PlaybackFrame {
            position,
            timestamp,
            animation,
            facing,
            on_ground,
            hair_color,
            depth,
            sounds,
            scale,
            dash_direction,
        })
    }

    fn write<T: Write>(
        &self,
        writer: &mut BinWriter<T>,
        base_pos: [f32; 2],
        base_timestamp: f32,
    ) -> std::io::Result<()> {
        writer.write_f32(self.position[0] - base_pos[0])?;
        writer.write_f32(self.position[1] - base_pos[1])?;
        writer.write_f32(self.timestamp - base_timestamp)?;
        writer.write_string(&self.animation)?;
        writer.write_i32(self.facing as i32)?;
        writer.write_bool(self.on_ground)?;
        writer.write_u8(self.hair_color[0])?;
        writer.write_u8(self.hair_color[1])?;
        writer.write_u8(self.hair_color[2])?;
        writer.write_i32(self.depth)?;
        writer.write_f32(self.scale[0])?;
        writer.write_f32(self.scale[1])?;
        writer.write_f32(self.dash_direction[0])?;
        writer.write_f32(self.dash_direction[1])?;
        Ok(())
    }
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaybackFacing {
    Left = -1,
    Right = 1,
}

impl PlaybackFacing {
    pub fn from_int(int: i32) -> Option<PlaybackFacing> {
        match int {
            -1 => Some(PlaybackFacing::Left),
            1 => Some(PlaybackFacing::Right),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum PlaybackReadError {
    BinError(BinReadError),
    IoError(std::io::Error),
}

impl Display for PlaybackReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlaybackReadError::BinError(bin_read_error) => bin_read_error.fmt(f),
            PlaybackReadError::IoError(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for PlaybackReadError {}

impl From<BinReadError> for PlaybackReadError {
    fn from(value: BinReadError) -> Self {
        PlaybackReadError::BinError(value)
    }
}

impl From<std::io::Error> for PlaybackReadError {
    fn from(value: std::io::Error) -> Self {
        PlaybackReadError::IoError(value)
    }
}
