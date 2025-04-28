use std::collections::{HashMap, HashSet};

use saphyr::YamlOwned;

use crate::saves::{session::SavedSession, util::FileTime};

/// The [ModSave](crate::saves::mods::ModSave) for [Collab Utils 2](https://gamebanana.com/mods/53704)
pub struct CollabsUtils2Save {
    /// A collection of [SavedSession]s related to the SID of the level they're for
    pub sessions_per_level: HashMap<String, SavedSession>,
    /// Saved modded sessions for legacy mods
    ///
    /// Each session is paired with the SID of the level it is for
    ///
    /// Apparently saved in yaml format
    pub mod_sessions_per_level: HashMap<String, YamlOwned>,
    /// Saved modded sessions for mods that support the async ser/de api
    ///
    /// Each SID has a collection of mods that saved their sessions and the saved mod sessions
    ///
    /// Stored mod sessions are base64 encoded
    pub mod_sessions_per_level_binary: HashMap<String, HashMap<String, String>>,
    /// Lobby SIDs and a base64 encoded list of locations visited.
    pub visited_lobby_positions: HashMap<String, String>,
    /// Set of opened mini-heart doors
    pub opened_mini_heart_doors: HashSet<String>,
    /// Set of rainbow berries that have had their forming animation play
    pub combined_rainbow_berries: HashSet<String>,
    /// Collection of level SIDs adn the speed berry time for that level
    pub speed_berry_pbs: HashMap<String, FileTime>,
    /// Whether the speed berry ui notice has been shown on this save
    pub speed_berry_option_message_shown: bool,
    /// List of warp pedestals that have completed their fill animation
    pub completed_warp_pedestal_sids: Vec<String>,
    /// Whether the map should be forcibly revealed
    pub reveal_map: bool,
    /// Whether collab utils should pause visiting points
    pub pause_visiting_points: bool,
    /// Whetehr collab utils should visualize visited points
    pub show_visited_points: bool,
}
