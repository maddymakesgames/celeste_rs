use std::collections::{HashMap, HashSet};

use crate::{
    saves::{
        mods::{ModFile, ModSave, collab_utils2::CollabsUtils2Save},
        ops::XML_VERSION_HEADER,
        session::{RootSavedSession, SavedSession},
        util::FileTime,
    },
    utils::{
        FromYaml,
        YamlExt,
        YamlParseError,
        YamlWriteError,
        anyhow::{OptionOkOrIter, ResultMapIter},
    },
};

use saphyr::{LoadableYamlNode, Mapping, Yaml, YamlOwned};

impl ModSave for CollabsUtils2Save {}

impl ModFile for CollabsUtils2Save {
    const MOD_NAME: &'static str = "CollabUtils2";
}
impl FromYaml for CollabsUtils2Save {
    fn parse_from_yaml(yaml: &saphyr::Yaml) -> Result<CollabsUtils2Save, YamlParseError> {
        let mut sessions_per_level = HashMap::new();

        let sessions_per_level_map =
            yaml["SessionsPerLevel"]
                .as_mapping()
                .ok_or(YamlParseError::custom(
                    "CollabUtils2 save doesn't contain a SessionsPerLevel entry",
                ))?;


        for (sid, value) in sessions_per_level_map {
            let sid = sid.as_str().ok_or(YamlParseError::custom(
                "CollabUtils2 save SessionsPerLevel entry key isn't a string",
            ))?;

            let session = value.as_str().ok_or(YamlParseError::custom(
                "CollabUtils2 SessionsPerLevel entry doesn't have a string value",
            ))?;

            let session = quick_xml::de::from_str::<SavedSession>(session)
                .map_err(YamlParseError::custom_from_err)?;

            sessions_per_level.insert(sid.to_owned(), session);
        }

        let mut mod_sessions_per_level = HashMap::new();
        let mod_sessions_per_level_map =
            yaml["ModSessionsPerLevel"]
                .as_mapping()
                .ok_or(YamlParseError::custom(
                    "CollabUtils2 save doesn't have ModSessionsPerLevel entry",
                ))?;

        for (sid, value) in mod_sessions_per_level_map {
            let sid = sid.as_str().ok_or(YamlParseError::custom(
                "CollabUtils2 ModSessionsPerLevel entry doesn't have a string key",
            ))?;

            mod_sessions_per_level.insert(sid.to_owned(), YamlOwned::from_bare_yaml(value.clone()));
        }

        let mut mod_sessions_per_level_binary = HashMap::new();
        let mod_sessions_per_level_binary_map = yaml["ModSessionsPerLevelBinary"]
            .as_mapping()
            .ok_or(YamlParseError::custom(
                "CollabUtils2 save doesn't have a ModSessionsPerLevelBinary field",
            ))?;

        for (sid, value) in mod_sessions_per_level_binary_map {
            let sid = sid.as_str().ok_or(YamlParseError::custom(
                "CollabUtils2 ModSessionsPerLevelBinary entry doesn't have a string key",
            ))?;

            let mut mod_data = HashMap::new();

            for (mod_name, base64) in value.as_mapping().ok_or(YamlParseError::custom(
                "CollabUtils2 save ModSessionsPerLevelBinary doesn't have a list of mod sessions",
            ))? {
                let mod_name = mod_name.as_str().ok_or(YamlParseError::custom(
                    "CollabUtils2 save ModSessionsPerLevelBinary mod entry doesn't have a string \
                     key",
                ))?;

                let base64 = base64.as_str().ok_or(YamlParseError::custom(
                    "CollabUtils2 save ModSessionsPerLevelBinary mod entry doesn't have a string \
                     value",
                ))?;

                mod_data.insert(mod_name.to_owned(), base64.to_owned());
            }

            mod_sessions_per_level_binary.insert(sid.to_owned(), mod_data);
        }

        let mut visited_lobby_positions = HashMap::new();
        let visited_lobby_positions_map =
            yaml["VisitedLobbyPositions"]
                .as_mapping()
                .ok_or(YamlParseError::custom(
                    "CollabUtils2 save doesn't have a VisitedLobbyPositions field",
                ))?;

        for (sid, position) in visited_lobby_positions_map {
            let sid = sid.as_str().ok_or(YamlParseError::custom(
                "CollabUtils2 VisitedLobbyPositions entry doesn't have a string key",
            ))?;

            let position_code = position.as_str().ok_or(YamlParseError::custom(
                "CollabUtils2 VisitedLobbyPositions entry doesn't have a string value",
            ))?;

            visited_lobby_positions.insert(sid.to_owned(), position_code.to_owned());
        }

        let opened_mini_heart_doors = yaml["OpenedMiniHeartDoors"]
            .as_vec()
            .ok_or(YamlParseError::custom(
                "CollabUtils2 save doesn't have a OpenedMiniHeartDoors field",
            ))?
            .iter()
            .map(|y| {
                y.as_str().ok_or(YamlParseError::custom(
                    "CollabUtils2 OpenedMiniHeartDoors entry isn't a string",
                ))
            })
            .map_result(ToOwned::to_owned)
            .collect::<Result<HashSet<_>, _>>()?;

        let combined_rainbow_berries = yaml["CombinedRainbowBerries"]
            .as_vec()
            .ok_or(YamlParseError::custom(
                "CollabUtils2 save doesn't have a CombinedRainbowBerries field",
            ))?
            .iter()
            .map(Yaml::as_str)
            .ok_or(YamlParseError::custom(
                "CollabUtils2 CombinedRainbowBerries entry isn't a string",
            ))
            .map_result(ToOwned::to_owned)
            .collect::<Result<HashSet<_>, _>>()?;

        let speed_berry_pbs = yaml["SpeedBerryPBs"]
            .as_mapping()
            .ok_or(YamlParseError::custom(
                "CollabUtils2 save doesn't have a SpeedBerryPBs field",
            ))?
            .iter()
            .map(|(sid, val)| {
                (
                    sid.as_str().ok_or(YamlParseError::custom(
                        "CollabUtils2 SpeedBerryPBs entry doesn't have string key",
                    )),
                    val.as_integer().ok_or(YamlParseError::custom(
                        "CollabUtils2 SpeedBerryPBs entry doesn't have an integer value",
                    )),
                )
            })
            .map(|(s, t)| (s.map(ToOwned::to_owned), t.map(|t| FileTime(t as u64))))
            .map(|(a, b)| Ok::<_, YamlParseError>((a?, b?)))
            .collect::<Result<HashMap<_, _>, _>>()?;

        let speed_berry_option_message_shown = yaml["SpeedberryOptionMessageShown"]
            .as_bool()
            .ok_or(YamlParseError::custom(
                "CollabUtils2 save doesn't have SpeedberryOptionMessageShown field",
            ))?;

        let completed_warp_pedestal_sids = yaml["CompletedWarpPedestalSIDs"]
            .as_vec()
            .ok_or(YamlParseError::custom(
                "CollabUtils2 save doesn't have CompletedWarpPedestalSIDs field",
            ))?
            .iter()
            .map(Yaml::as_str)
            .ok_or(YamlParseError::custom(
                "CollabUtils2 CompletedWarpPedestalSIDs entry isn't a string",
            ))
            .map_result(ToOwned::to_owned)
            .collect::<Result<_, _>>()?;

        let reveal_map = yaml["RevealMap"].as_bool().ok_or(YamlParseError::custom(
            "CollabUtils2 save doesn't have RevealMap field",
        ))?;
        let pause_visiting_points =
            yaml["PauseVisitingPoints"]
                .as_bool()
                .ok_or(YamlParseError::custom(
                    "CollabUtils2 save doesn't have PauseVisitingPoints field",
                ))?;
        let show_visited_points =
            yaml["ShowVisitedPoints"]
                .as_bool()
                .ok_or(YamlParseError::custom(
                    "CollabUtils2 save doesn't have ShowVisitedPoints field",
                ))?;


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

    fn to_yaml(&self) -> Result<saphyr::Yaml, YamlWriteError> {
        let mut root = Mapping::new();

        let sessions_per_level = self
            .sessions_per_level
            .iter()
            .map(|(sid, session)| {
                Ok::<_, YamlWriteError>((
                    sid,
                    format!(
                        "{XML_VERSION_HEADER}{}",
                        quick_xml::se::to_string_with_root::<RootSavedSession>(
                            "Session",
                            &session.clone().into()
                        )
                        .map_err(YamlWriteError::custom_from_err)?
                    ),
                ))
            })
            .map_result(|(sid, session)| (Yaml::string(sid.clone()), Yaml::string(session)))
            .collect::<Result<Mapping, _>>()?;

        root.insert(
            Yaml::string("SessionsPerLevel".to_owned()),
            Yaml::hash(sessions_per_level),
        );

        let mod_sessions_per_level = self
            .mod_sessions_per_level
            .iter()
            .map(|(sid, session)| (Yaml::string(sid.clone()), session.into()))
            .collect::<Mapping>();

        root.insert(
            Yaml::string("ModSessionsPerLevel".to_owned()),
            Yaml::hash(mod_sessions_per_level),
        );

        let mod_sessions_per_level_binary = self
            .mod_sessions_per_level_binary
            .iter()
            .map(|(sid, session)| {
                (
                    Yaml::string(sid.clone()),
                    Yaml::hash(
                        session
                            .iter()
                            .map(|(mod_id, session)| {
                                (Yaml::string(mod_id.clone()), Yaml::string(session.clone()))
                            })
                            .collect::<Mapping>(),
                    ),
                )
            })
            .collect::<Mapping>();

        root.insert(
            Yaml::string("ModSessionsPerLevelBinary".to_owned()),
            Yaml::hash(mod_sessions_per_level_binary),
        );

        let visited_lobby_positions = self
            .visited_lobby_positions
            .iter()
            .map(|(sid, base64)| {
                (
                    Yaml::string(sid.to_owned()),
                    Yaml::string(base64.to_owned()),
                )
            })
            .collect::<Mapping>();

        root.insert(
            Yaml::string("VisitedLobbyPositions".to_owned()),
            Yaml::hash(visited_lobby_positions),
        );

        let opened_mini_heart_doors = self
            .opened_mini_heart_doors
            .iter()
            .cloned()
            .map(Yaml::string)
            .collect::<Vec<_>>();

        root.insert(
            Yaml::string("OpenedMiniHeartDoors".to_owned()),
            Yaml::seq(opened_mini_heart_doors),
        );

        let combined_rainbow_berries = self
            .combined_rainbow_berries
            .iter()
            .cloned()
            .map(Yaml::string)
            .collect::<Vec<_>>();

        root.insert(
            Yaml::string("CombinedRainbowBerries".to_owned()),
            Yaml::seq(combined_rainbow_berries),
        );

        let speed_berry_pbs = self
            .speed_berry_pbs
            .iter()
            .map(|(sid, time)| (Yaml::string(sid.clone()), Yaml::int(time.0 as i64)))
            .collect::<Mapping>();

        root.insert(
            Yaml::string("SpeedBerryPBs".to_owned()),
            Yaml::Mapping(speed_berry_pbs),
        );

        root.insert(
            Yaml::string("SpeedberryOptionMessageShown".to_owned()),
            Yaml::bool(self.speed_berry_option_message_shown),
        );

        let completed_warp_pedestal_sids = self
            .completed_warp_pedestal_sids
            .iter()
            .cloned()
            .map(Yaml::string)
            .collect::<Vec<_>>();

        root.insert(
            Yaml::string("CompletedWarpPedestalSIDs".to_owned()),
            Yaml::seq(completed_warp_pedestal_sids),
        );

        root.insert(
            Yaml::string("RevealMap".to_owned()),
            Yaml::bool(self.reveal_map),
        );

        root.insert(
            Yaml::string("PauseVisitingPoints".to_owned()),
            Yaml::bool(self.pause_visiting_points),
        );

        root.insert(
            Yaml::string("ShowVisitedPoints".to_owned()),
            Yaml::bool(self.show_visited_points),
        );

        Ok(Yaml::hash(root))
    }
}
