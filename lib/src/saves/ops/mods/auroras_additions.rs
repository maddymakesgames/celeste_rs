use std::collections::HashMap;

use anyhow::{anyhow, Result};
use saphyr::Yaml;

use crate::saves::{
    mods::auroras_additions::AurorasAdditionsSave,
    ops::mods::ModFile,
    session::SavedSession,
};


impl ModFile for AurorasAdditionsSave {
    const MOD_NAME: &'static str = "AurorasAdditions";

    fn parse_from_yaml(yaml: Yaml) -> Result<Self> {
        let mut sessions_per_level = HashMap::new();

        let sessions_per_level_map = yaml["SessionsPerLevel"].as_hash().ok_or(anyhow!(
            "Aurora's Additions save doesn't contain a SessionsPerLevel entry"
        ))?;


        for (sid, value) in sessions_per_level_map {
            let (sid, mode) = sid
                .as_str()
                .ok_or(anyhow!(
                    "Aurora's Additions save SessionsPerLevel entry key isn't a string"
                ))?
                .split_once(',')
                .ok_or(anyhow!(
                    "Aurora's Additions SessionsPerLevel entry doesn't have a properly formatted \
                     key"
                ))?;

            let session = value.as_str().ok_or(anyhow!(
                "Aurora's Additions SessionsPerLevel entry doesn't have a string value"
            ))?;

            let session = quick_xml::de::from_str::<SavedSession>(session)?;

            sessions_per_level.insert((sid.to_owned(), mode.to_owned()), session);
        }

        let mut mod_sessions_per_level = HashMap::new();
        let mod_sessions_per_level_map = yaml["ModSessionsPerLevel"].as_hash().ok_or(anyhow!(
            "Aurora's Additions save doesn't have ModSessionsPerLevel entry"
        ))?;

        for (sid_mode, value) in mod_sessions_per_level_map {
            let (sid, mode) = sid_mode
                .as_str()
                .ok_or(anyhow!(
                    "Aurora's Additions ModSessionsPerLevel entry doesn't have a string key"
                ))?
                .split_once(',')
                .ok_or(anyhow!(
                    "Aurora's Additions ModSessionsPerLevel entry has improperly formatted key"
                ))?;

            mod_sessions_per_level.insert((sid.to_owned(), mode.to_owned()), value.clone());
        }

        let mut mod_sessions_per_level_binary = HashMap::new();
        let mod_sessions_per_level_map = yaml["ModSessionsPerLevelBinary"].as_hash().ok_or(
            anyhow!("Aurora's Additions save doesn't have a ModSessionsPerLevelBinary field"),
        )?;

        for (sid_mode, value) in mod_sessions_per_level_map {
            let (sid, mode) = sid_mode
                .as_str()
                .ok_or(anyhow!(
                    "Aurora's Additions ModSessionsPerLevelBinary entry doesn't have a string key"
                ))?
                .split_once(',')
                .ok_or(anyhow!(
                    "Aurora's Additions ModSessionsPerLevelBinary entry has improperly formatted \
                     key"
                ))?;

            let mut mod_data = HashMap::new();

            for (mod_name, base64) in value.as_hash().ok_or(anyhow!(
                "Aurora's Additions save ModSessionsPerLevelBinary doesn't have a list of mod \
                 sessions"
            ))? {
                let mod_name = mod_name.as_str().ok_or(anyhow!(
                    "Aurora's Additions save ModSessionsPerLevelBinary mod entry doesn't have a \
                     string key"
                ))?;

                let base64 = base64.as_str().ok_or(anyhow!(
                    "Aurora's Additions save ModSessionsPerLevelBinary mod entry doesn't have a \
                     string value"
                ))?;

                mod_data.insert(mod_name.to_owned(), base64.to_owned());
            }

            mod_sessions_per_level_binary.insert((sid.to_owned(), mode.to_owned()), mod_data);
        }

        let music_volume_memory = yaml["MusicVolumeMemory"]
            .as_i64()
            .ok_or(anyhow!("Aurora's Additions MusicVolumeMemory isn't an int"))?;

        Ok(AurorasAdditionsSave {
            sessions_per_level,
            mod_sessions_per_level,
            mod_sessions_per_level_binary,
            music_volume_memory: music_volume_memory as u8,
        })
    }

    fn to_yaml(&self) -> Yaml {
        todo!()
    }
}
