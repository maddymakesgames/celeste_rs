use std::{
    ffi::OsStr,
    fs::{File, OpenOptions},
    io::Read,
    path::Path,
};

use crate::saves::mods::{auroras_additions::AurorasAdditionsSave, *};
use anyhow::{anyhow, Result};
use saphyr::{YAMLDecodingTrap, YamlDecoder, YamlLoader};

mod auroras_additions;


fn check_yaml_file<'a>(
    file_type: &'static str,
    global: bool,
    path: &'a Path,
) -> Result<(File, u8, &'a str)> {
    let file_name = path
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or(anyhow!("Invalid path to {file_type}: {path:?}"))?;


    if !file_name.contains(".celeste") {
        return Err(anyhow!("Given file is not a .celeste file: {file_name:?}"));
    }

    if !file_name.contains(file_type) {
        return Err(anyhow!("Given file is not a {file_type}: {file_name:}"));
    }

    if !file_name.contains('-') {
        return Err(anyhow!("Invalid {file_type} name: {file_name:?}"));
    }

    // Unwrap is safe because we already ensured that the file_name contains .celeste
    let file_stem = file_name.split('.').next().unwrap();

    let mut part_iter = file_stem.split('-');

    let file_index = if !global {
        // Unwrap is safe because we ensured that there is at least one - in the file name
        // TODO: redo error here to make it clear where it is coming from
        part_iter.next().unwrap().parse::<u8>()?
    } else {
        u8::MAX
    };

    // Unwrap is safe because of the same reason as above
    let _ = part_iter.next().unwrap();

    let mod_name = part_iter
        .next()
        .ok_or(anyhow!("Invalid {file_type} name: {file_name:?}"))?;

    let file = OpenOptions::new().read(true).write(false).open(path)?;

    Ok((file, file_index, mod_name))
}

impl ParsedModSave {
    pub fn get_file_name(&self, file_index: u8) -> String {
        format!("{file_index}-modsave-{}.celeste", match self {
            ParsedModSave::AurorasAdditions(_) => AurorasAdditionsSave::MOD_NAME,
            ParsedModSave::CollabUtils2(_) => todo!(),
            ParsedModSave::Unknown(DynYamlDoc(mod_name, _)) => mod_name,
        })
    }

    pub fn parse_from_path(path: impl AsRef<Path>) -> Result<(u8, Self)> {
        let path = path.as_ref();

        let (file, file_index, mod_name) = check_yaml_file("modsave", false, path)?;


        Ok((file_index, match mod_name {
            AurorasAdditionsSave::MOD_NAME =>
                Self::AurorasAdditions(AurorasAdditionsSave::parse_from_reader(file)?),
            _ => Self::Unknown(DynYamlDoc::parse_from_reader_and_mod_name(file, mod_name)?),
        }))
    }
}


impl ParsedModSession {
    pub fn get_file_name(&self, file_index: u8) -> String {
        format!("{file_index}-modsession-{}.celeste", match self {
            ParsedModSession::Unknown(DynYamlDoc(mod_name, _)) => mod_name,
        })
    }

    pub fn parse_from_path(path: impl AsRef<Path>) -> Result<(u8, Self)> {
        let path = path.as_ref();

        let (file, file_index, mod_name) = check_yaml_file("modsession", false, path)?;

        #[allow(clippy::match_single_binding)]
        Ok((file_index, match mod_name {
            _ => Self::Unknown(DynYamlDoc::parse_from_reader_and_mod_name(file, mod_name)?),
        }))
    }
}

impl ParsedModSetting {
    pub fn get_file_name(&self) -> String {
        format!("modsettings-{}.celeste", match self {
            ParsedModSetting::Unknown(DynYamlDoc(mod_name, _)) => mod_name,
        })
    }

    pub fn parse_from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        let (file, _, mod_name) = check_yaml_file("modsettings", true, path)?;

        #[allow(clippy::match_single_binding)]
        Ok(match mod_name {
            _ => Self::Unknown(DynYamlDoc::parse_from_reader_and_mod_name(file, mod_name)?),
        })
    }
}

impl DynYamlDoc {
    pub fn parse_from_str_and_mod_name(str: &str, mod_name: &str) -> Result<Self> {
        let yaml = YamlLoader::load_from_str(str)?;
        Ok(Self(mod_name.to_owned(), yaml[0].clone()))
    }

    pub fn parse_from_reader_and_mod_name(reader: impl Read, mod_name: &str) -> Result<Self> {
        let yaml = YamlDecoder::read(reader)
            .encoding_trap(YAMLDecodingTrap::Strict)
            .decode();
        match yaml {
            Ok(y) => Ok(Self(mod_name.to_owned(), y[0].clone())),
            Err(e) => match e {
                saphyr::yaml::LoadError::IO(e) => Err(Box::new(e).into()),
                saphyr::yaml::LoadError::Scan(e) => Err(Box::new(e).into()),
                saphyr::yaml::LoadError::Decode(e) => Err(anyhow::format_err!(e)),
            },
        }
    }
}
