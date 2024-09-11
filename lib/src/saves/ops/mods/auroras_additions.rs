use std::collections::HashMap;

use anyhow::{anyhow, Result};
use saphyr::{Hash, Yaml};

use crate::{
    anyhow_utils::ResultMapIter,
    saves::{
        mods::{auroras_additions::AurorasAdditionsSave, ModFile, ModSave},
        ops::XML_VERSION_HEADER,
        session::{RootSavedSession, SavedSession},
    },
    utils::YamlFile,
};

impl ModSave for AurorasAdditionsSave {}

impl ModFile for AurorasAdditionsSave {
    const MOD_NAME: &'static str = "AurorasAdditions";
}
impl YamlFile for AurorasAdditionsSave {
    fn parse_from_yaml(yaml: &Yaml) -> Result<Self> {
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

            sessions_per_level.insert((sid.to_owned(), mode.trim().to_owned()), session);
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

            mod_sessions_per_level.insert((sid.to_owned(), mode.trim().to_owned()), value.clone());
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

            mod_sessions_per_level_binary
                .insert((sid.to_owned(), mode.trim().to_owned()), mod_data);
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

    fn to_yaml(&self) -> Result<saphyr::Yaml> {
        let mut root = Hash::new();

        let sessions_per_level = self
            .sessions_per_level
            .iter()
            .map(|(sid_mode, session)| {
                Ok((
                    sid_mode,
                    format!(
                        "{XML_VERSION_HEADER}{}",
                        quick_xml::se::to_string_with_root::<RootSavedSession>(
                            "Session",
                            &session.clone().into(),
                        )?
                    ),
                ))
            })
            .map_result(|((sid, mode), session)| {
                (
                    Yaml::String(format!("{sid}, {mode}")),
                    Yaml::String(session),
                )
            })
            .collect::<Result<Hash>>()?;

        root.insert(
            Yaml::String("SessionsPerLevel".to_owned()),
            Yaml::Hash(sessions_per_level),
        );

        let mod_sessions_per_level = self
            .mod_sessions_per_level
            .iter()
            .map(|((sid, mode), session)| (Yaml::String(format!("{sid}, {mode}")), session.clone()))
            .collect::<Hash>();

        root.insert(
            Yaml::String("ModSessionsPerLevel".to_owned()),
            Yaml::Hash(mod_sessions_per_level),
        );

        let mod_sessions_per_level_binary = self
            .mod_sessions_per_level_binary
            .iter()
            .map(|((sid, mode), session)| {
                (
                    Yaml::String(format!("{sid}, {mode}")),
                    Yaml::Hash(
                        session
                            .iter()
                            .map(|(mod_id, session)| {
                                (Yaml::String(mod_id.clone()), Yaml::String(session.clone()))
                            })
                            .collect::<Hash>(),
                    ),
                )
            })
            .collect::<Hash>();

        root.insert(
            Yaml::String("ModSessionsPerLevelBinary".to_owned()),
            Yaml::Hash(mod_sessions_per_level_binary),
        );

        root.insert(
            Yaml::String("MusicVolumeMemory".to_owned()),
            Yaml::Integer(self.music_volume_memory as i64),
        );

        Ok(Yaml::Hash(root))
    }
}
