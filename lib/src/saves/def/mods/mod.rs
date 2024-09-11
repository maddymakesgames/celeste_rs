//! Provides support for parsing arbitrary modded saves and sessions
//!
//! Pretty much all modsave or modsession files are in YAML so if we're parsing a session or save we haven't worked with before that is what we assume for parsing.
//!
//! [ModSave]s store arbitrary data that mods might need
//! [ModSession]s store arbitrary data that lasts for the current session in game.

pub mod auroras_additions;
pub mod collab_utils2;

use saphyr::Yaml;

use crate::{
    saves::mods::{auroras_additions::AurorasAdditionsSave, collab_utils2::CollabsUtils2Save},
    utils::YamlFile,
};

/// A mod related file<br>
/// Either a `*-modsave`, `*-modsession`, or `modsettings-*`.
///
/// This is built to parse YAML but mods can likely have
/// these files be in any format.<br>
/// To implement for a mod file that does not use YAML
/// make the `parse_from_yaml` body `unreachable!()`
/// and overwrite `parse_from_str` and `parse_from_reader`
pub trait ModFile: YamlFile + Sized {
    /// The unlocalized name of the mod the file is for.
    ///
    /// This is the third part of the file name and should be used to verify which file you're loading.
    const MOD_NAME: &'static str;
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

/// A generic YAML document that we cannot otherwise parse into a ModFile impl
///
/// String is mod name
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
