use std::collections::HashMap;

use saphyr::Yaml;

use crate::saves::session::SavedSession;

#[derive(Debug)]
pub struct AurorasAdditionsSave {
    /// A collection of [SavedSession]s related to the SID of the level they're for
    pub sessions_per_level: HashMap<(String, String), SavedSession>,
    /// Saved modded sessions for legacy mods
    ///
    /// Each session is paired with the SID of the level it is for
    ///
    /// Apparently saved in yaml format
    pub mod_sessions_per_level: HashMap<(String, String), Yaml>,
    /// Saved modded sessions for mods that support the async ser/de api
    ///
    /// Each SID has a collection of mods that saved their sessions and the saved mod sessions
    ///
    /// Stored mod sessions are base64 encoded
    pub mod_sessions_per_level_binary: HashMap<(String, String), HashMap<String, String>>,
    /// Saved last music volume
    pub music_volume_memory: u8,
}
