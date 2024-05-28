//! Provides support for parsing arbitrary modded saves and sessions
//!
//! Pretty much all modsave or modsession files are in YAML so if we're parsing a session or save we haven't worked with before that is what we assume for parsing.
//!
//! [ModSave]s store arbitrary data that mods might need
//! [ModSession]s store arbitrary data that lasts for the current session in game.

pub mod auroras_additions;
pub mod collab_utils2;

use std::{fmt::Write, io::Read};

use saphyr::{YAMLDecodingTrap, Yaml, YamlDecoder, YamlEmitter, YamlLoader};

use crate::saves::mods::{
    auroras_additions::AurorasAdditionsSave,
    collab_utils2::CollabsUtils2Save,
};

/// A mod related file<br>
/// Either a `*-modsave`, `*-modsession`, or `modsettings-*`.
///
/// This is built to parse YAML but mods can likely have
/// these files be in any format.<br>
/// To implement for a mod file that does not use YAML
/// make the `parse_from_yaml` body `unreachable!()`
/// and overwrite `parse_from_str` and `parse_from_reader`
pub trait ModFile: Sized {
    /// The unlocalized name of the mod the file is for.
    ///
    /// This is the third part of the file name and should be used to verify which file you're loading.
    const MOD_NAME: &'static str;

    fn parse_from_yaml(yaml: Yaml) -> anyhow::Result<Self>;

    fn parse_from_str(str: &str) -> anyhow::Result<Self> {
        let yaml = YamlLoader::load_from_str(str)?;
        Self::parse_from_yaml(yaml[0].clone())
    }

    fn parse_from_reader(reader: impl Read) -> anyhow::Result<Self> {
        let yaml = YamlDecoder::read(reader)
            .encoding_trap(YAMLDecodingTrap::Strict)
            .decode();
        match yaml {
            Ok(y) => Self::parse_from_yaml(y[0].clone()),
            Err(e) => match e {
                saphyr::yaml::LoadError::IO(e) => Err(Box::new(e).into()),
                saphyr::yaml::LoadError::Scan(e) => Err(Box::new(e).into()),
                saphyr::yaml::LoadError::Decode(e) => Err(anyhow::format_err!(e)),
            },
        }
    }

    fn to_yaml(&self) -> anyhow::Result<Yaml>;

    fn to_writer(&self, writer: &mut impl Write) -> anyhow::Result<()> {
        let yaml = self.to_yaml()?;

        Ok(YamlEmitter::new(writer).dump(&yaml)?)
    }
}


/// A `*-modsave-*.celeste` file.
///
/// These save data across sessions
pub trait ModSave: ModFile {}

/// A `*-modsession-*.celeste` file.
///
/// These save data per-session. Usually data is more directly related to gameplay.
///
/// Mods like CollabUtils2 or Aurora's Additions saves these as part of a session.
pub trait ModSession: ModFile {}

/// A `modsettings-*.celeste` file
///
/// These store settings for each mod, and are shared among save files
pub trait ModSettings: ModFile {}


pub struct DynYamlDoc(pub String, pub Yaml);

#[allow(clippy::large_enum_variant)]
pub enum ParsedModSave {
    AurorasAdditions(AurorasAdditionsSave),
    CollabUtils2(CollabsUtils2Save),
    Unknown(DynYamlDoc),
}

#[allow(clippy::large_enum_variant)]
pub enum ParsedModSession {
    Unknown(DynYamlDoc),
}

#[allow(clippy::large_enum_variant)]
pub enum ParsedModSetting {
    Unknown(DynYamlDoc),
}
