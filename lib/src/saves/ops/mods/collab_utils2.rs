use std::collections::{HashMap, HashSet};

use crate::{
    anyhow_utils::{AnyhowIter, AnyhowOption, ResultMapIter},
    saves::{
        mods::{collab_utils2::CollabsUtils2Save, ModFile, ModSave},
        ops::XML_VERSION_HEADER,
        session::{RootSavedSession, SavedSession},
        util::FileTime,
    },
};

use anyhow::{anyhow, Error, Result};
use saphyr::{Hash, Yaml};

impl ModSave for CollabsUtils2Save {}

impl ModFile for CollabsUtils2Save {
    const MOD_NAME: &'static str = "CollabUtils2";

    fn parse_from_yaml(yaml: saphyr::Yaml) -> anyhow::Result<Self> {
        let mut sessions_per_level = HashMap::new();

        let sessions_per_level_map = yaml["SessionsPerLevel"].as_hash().ok_or(anyhow!(
            "CollabUtils2 save doesn't contain a SessionsPerLevel entry"
        ))?;


        for (sid, value) in sessions_per_level_map {
            let sid = sid.as_str().ok_or(anyhow!(
                "CollabUtils2 save SessionsPerLevel entry key isn't a string"
            ))?;

            let session = value.as_str().ok_or(anyhow!(
                "CollabUtils2 SessionsPerLevel entry doesn't have a string value"
            ))?;

            let session = quick_xml::de::from_str::<SavedSession>(session)?;

            sessions_per_level.insert(sid.to_owned(), session);
        }

        let mut mod_sessions_per_level = HashMap::new();
        let mod_sessions_per_level_map = yaml["ModSessionsPerLevel"].as_hash().ok_or(anyhow!(
            "CollabUtils2 save doesn't have ModSessionsPerLevel entry"
        ))?;

        for (sid, value) in mod_sessions_per_level_map {
            let sid = sid.as_str().ok_or(anyhow!(
                "CollabUtils2 ModSessionsPerLevel entry doesn't have a string key"
            ))?;

            mod_sessions_per_level.insert(sid.to_owned(), value.clone());
        }

        let mut mod_sessions_per_level_binary = HashMap::new();
        let mod_sessions_per_level_binary_map = yaml["ModSessionsPerLevelBinary"].as_hash().ok_or(
            anyhow!("CollabUtils2 save doesn't have a ModSessionsPerLevelBinary field"),
        )?;

        for (sid, value) in mod_sessions_per_level_binary_map {
            let sid = sid.as_str().ok_or(anyhow!(
                "CollabUtils2 ModSessionsPerLevelBinary entry doesn't have a string key"
            ))?;

            let mut mod_data = HashMap::new();

            for (mod_name, base64) in value.as_hash().ok_or(anyhow!(
                "CollabUtils2 save ModSessionsPerLevelBinary doesn't have a list of mod sessions"
            ))? {
                let mod_name = mod_name.as_str().ok_or(anyhow!(
                    "CollabUtils2 save ModSessionsPerLevelBinary mod entry doesn't have a string \
                     key"
                ))?;

                let base64 = base64.as_str().ok_or(anyhow!(
                    "CollabUtils2 save ModSessionsPerLevelBinary mod entry doesn't have a string \
                     value"
                ))?;

                mod_data.insert(mod_name.to_owned(), base64.to_owned());
            }

            mod_sessions_per_level_binary.insert(sid.to_owned(), mod_data);
        }

        let mut visited_lobby_positions = HashMap::new();
        let visited_lobby_positions_map = yaml["VisitedLobbyPositions"].as_hash().ok_or(
            anyhow!("CollabUtils2 save doesn't have a VisitedLobbyPositions field"),
        )?;

        for (sid, position) in visited_lobby_positions_map {
            let sid = sid.as_str().ok_or(anyhow!(
                "CollabUtils2 VisitedLobbyPositions entry doesn't have a string key"
            ))?;

            let position_code = position.as_str().ok_or(anyhow!(
                "CollabUtils2 VisitedLobbyPositions entry doesn't have a string value"
            ))?;

            visited_lobby_positions.insert(sid.to_owned(), position_code.to_owned());
        }

        let opened_mini_heart_doors = yaml["OpenedMiniHeartDoors"]
            .as_vec()
            .anyhow("CollabUtils2 save doesn't have a OpenedMiniHeartDoors field")?
            .iter()
            .map(Yaml::as_str)
            .anyhow("CollabUtils2 OpenedMiniHeartDoors entry isn't a string")
            .map_result(ToOwned::to_owned)
            .collect::<Result<HashSet<_>, _>>()?;

        let combined_rainbow_berries = yaml["CombinedRainbowBerries"]
            .as_vec()
            .anyhow("CollabUtils2 save doesn't have a CombinedRainbowBerries field")?
            .iter()
            .map(Yaml::as_str)
            .anyhow("CollabUtils2 CombinedRainbowBerries entry isn't a string")
            .map_result(ToOwned::to_owned)
            .collect::<Result<HashSet<_>>>()?;

        let speed_berry_pbs = yaml["SpeedBerryPBs"]
            .as_hash()
            .anyhow("CollabUtils2 save doesn't have a SpeedBerryPBs field")?
            .iter()
            .map(|(sid, val)| {
                (
                    sid.as_str()
                        .anyhow("CollabUtils2 SpeedBerryPBs entry doesn't have string key"),
                    val.as_i64()
                        .anyhow("CollabUtils2 SpeedBerryPBs entry doesn't have an integer value"),
                )
            })
            .map(|(s, t)| (s.map(ToOwned::to_owned), t.map(|t| FileTime(t as u64))))
            .map(|(a, b)| Ok((a?, b?)))
            .collect::<Result<HashMap<_, _>, Error>>()?;

        let speed_berry_option_message_shown = yaml["SpeedberryOptionMessageShown"]
            .as_bool()
            .anyhow("CollabUtils2 save doesn't have SpeedberryOptionMessageShown field")?;

        let completed_warp_pedestal_sids = yaml["CompletedWarpPedestalSIDs"]
            .as_vec()
            .anyhow("CollabUtils2 save doesn't have CompletedWarpPedestalSIDs field")?
            .iter()
            .map(Yaml::as_str)
            .anyhow("CollabUtils2 CompletedWarpPedestalSIDs entry isn't a string")
            .map_result(ToOwned::to_owned)
            .collect::<Result<_>>()?;

        let reveal_map = yaml["RevealMap"]
            .as_bool()
            .anyhow("CollabUtils2 save doesn't have RevealMap field")?;
        let pause_visiting_points = yaml["PauseVisitingPoints"]
            .as_bool()
            .anyhow("CollabUtils2 save doesn't have PauseVisitingPoints field")?;
        let show_visited_points = yaml["ShowVisitedPoints"]
            .as_bool()
            .anyhow("CollabUtils2 save doesn't have ShowVisitedPoints field")?;


        Ok(CollabsUtils2Save {
            sessions_per_level,
            mod_sessions_per_level,
            mod_sessions_per_level_binary,
            visited_lobby_positions,
            opened_mini_heart_doors,
            combined_rainbow_berries,
            speed_berry_pbs,
            speed_berry_option_message_shown,
            completed_warp_pedestal_sids,
            reveal_map,
            pause_visiting_points,
            show_visited_points,
        })
    }

    fn to_yaml(&self) -> Result<saphyr::Yaml> {
        let mut root = Hash::new();

        let sessions_per_level = self
            .sessions_per_level
            .iter()
            .map(|(sid, session)| {
                Ok((
                    sid,
                    format!(
                        "{XML_VERSION_HEADER}{}",
                        quick_xml::se::to_string_with_root::<RootSavedSession>(
                            "Session",
                            &session.clone().into()
                        )?
                    ),
                ))
            })
            .map_result(|(sid, session)| (Yaml::String(sid.clone()), Yaml::String(session)))
            .collect::<Result<Hash>>()?;

        root.insert(
            Yaml::String("SessionsPerLevel".to_owned()),
            Yaml::Hash(sessions_per_level),
        );

        let mod_sessions_per_level = self
            .mod_sessions_per_level
            .iter()
            .map(|(sid, session)| (Yaml::String(sid.clone()), session.clone()))
            .collect::<Hash>();

        root.insert(
            Yaml::String("ModSessionsPerLevel".to_owned()),
            Yaml::Hash(mod_sessions_per_level),
        );

        let mod_sessions_per_level_binary = self
            .mod_sessions_per_level_binary
            .iter()
            .map(|(sid, session)| {
                (
                    Yaml::String(sid.clone()),
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

        let visited_lobby_positions = self
            .visited_lobby_positions
            .iter()
            .map(|(sid, base64)| {
                (
                    Yaml::String(sid.to_owned()),
                    Yaml::String(base64.to_owned()),
                )
            })
            .collect::<Hash>();

        root.insert(
            Yaml::String("VisitedLobbyPositions".to_owned()),
            Yaml::Hash(visited_lobby_positions),
        );

        let opened_mini_heart_doors = self
            .opened_mini_heart_doors
            .iter()
            .cloned()
            .map(Yaml::String)
            .collect::<Vec<_>>();

        root.insert(
            Yaml::String("OpenedMiniHeartDoors".to_owned()),
            Yaml::Array(opened_mini_heart_doors),
        );

        let combined_rainbow_berries = self
            .combined_rainbow_berries
            .iter()
            .cloned()
            .map(Yaml::String)
            .collect::<Vec<_>>();

        root.insert(
            Yaml::String("CombinedRainbowBerries".to_owned()),
            Yaml::Array(combined_rainbow_berries),
        );

        let speed_berry_pbs = self
            .speed_berry_pbs
            .iter()
            .map(|(sid, time)| (Yaml::String(sid.clone()), Yaml::Integer(time.0 as i64)))
            .collect::<Hash>();

        root.insert(
            Yaml::String("SpeedBerryPBs".to_owned()),
            Yaml::Hash(speed_berry_pbs),
        );

        root.insert(
            Yaml::String("SpeedberryOptionMessageShown".to_owned()),
            Yaml::Boolean(self.speed_berry_option_message_shown),
        );

        let completed_warp_pedestal_sids = self
            .completed_warp_pedestal_sids
            .iter()
            .cloned()
            .map(Yaml::String)
            .collect::<Vec<_>>();

        root.insert(
            Yaml::String("CompletedWarpPedestalSIDs".to_owned()),
            Yaml::Array(completed_warp_pedestal_sids),
        );

        root.insert(
            Yaml::String("RevealMap".to_owned()),
            Yaml::Boolean(self.reveal_map),
        );

        root.insert(
            Yaml::String("PauseVisitingPoints".to_owned()),
            Yaml::Boolean(self.pause_visiting_points),
        );

        root.insert(
            Yaml::String("ShowVisitedPoints".to_owned()),
            Yaml::Boolean(self.show_visited_points),
        );

        Ok(Yaml::Hash(root))
    }
}
