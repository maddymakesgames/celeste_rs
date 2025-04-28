use std::{ffi::OsStr, fmt::Write, fs::OpenOptions, io::Read, path::Path};

use crate::{
    saves::mods::{auroras_additions::AurorasAdditionsSave, collab_utils2::CollabsUtils2Save, *},
    utils::{FromYaml, YamlParseError, YamlWriteError},
};
use anyhow::{Result, anyhow};
use saphyr::{LoadError, LoadableYamlNode, YAMLDecodingTrap, Yaml, YamlDecoder, YamlOwned};

mod auroras_additions;
mod collab_utils2;

fn check_yaml_file<'a>(
    file_type: &'static str,
    global: bool,
    path: &'a Path,
) -> Result<(u8, &'a str)> {
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

    Ok((file_index, mod_name))
}

impl ParsedModSave {
    pub fn get_file_name(&self, file_index: u8) -> String {
        format!("{file_index}-modsave-{}.celeste", match self {
            ParsedModSave::AurorasAdditions(_) => AurorasAdditionsSave::MOD_NAME,
            ParsedModSave::CollabUtils2(_) => CollabsUtils2Save::MOD_NAME,
            ParsedModSave::Unknown(DynYamlDoc(mod_name, _)) => mod_name,
        })
    }

    pub fn parse_from_path(path: impl AsRef<Path>) -> Result<(u8, Self)> {
        let path = path.as_ref();

        let file = OpenOptions::new().read(true).write(false).open(path)?;


        Self::from_reader_and_path(path, file)
    }

    pub fn from_reader_and_path(path: impl AsRef<Path>, reader: impl Read) -> Result<(u8, Self)> {
        let (file_index, mod_name) = check_yaml_file("modsave", false, path.as_ref())?;

        Ok((file_index, match mod_name {
            AurorasAdditionsSave::MOD_NAME =>
                Self::AurorasAdditions(AurorasAdditionsSave::parse_from_reader(reader)?),
            CollabsUtils2Save::MOD_NAME =>
                Self::CollabUtils2(CollabsUtils2Save::parse_from_reader(reader)?),
            _ => Self::Unknown(DynYamlDoc::parse_from_reader_and_mod_name(
                reader, mod_name,
            )?),
        }))
    }

    pub fn to_writer(&self, writer: &mut impl Write) -> Result<(), YamlWriteError> {
        match self {
            ParsedModSave::AurorasAdditions(a) => a.to_writer(writer),
            ParsedModSave::CollabUtils2(c) => c.to_writer(writer),
            ParsedModSave::Unknown(doc) => doc.to_writer(writer),
        }
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

        let file = OpenOptions::new().read(true).write(false).open(path)?;


        Self::from_reader_and_path(path, file)
    }

    pub fn from_reader_and_path(path: impl AsRef<Path>, reader: impl Read) -> Result<(u8, Self)> {
        let (file_index, mod_name) = check_yaml_file("modsession", false, path.as_ref())?;


        #[allow(clippy::match_single_binding)]
        Ok((file_index, match mod_name {
            _ => Self::Unknown(DynYamlDoc::parse_from_reader_and_mod_name(
                reader, mod_name,
            )?),
        }))
    }

    pub fn to_writer(&self, writer: &mut impl Write) -> Result<(), YamlWriteError> {
        match self {
            ParsedModSession::Unknown(doc) => doc.to_writer(writer),
        }
    }
}

impl ParsedModSetting {
    pub fn get_file_name(&self) -> String {
        format!("modsettings-{}.celeste", match self {
            ParsedModSetting::Unknown(DynYamlDoc(mod_name, _)) => mod_name,
        })
    }

    pub fn parse_from_path(path: impl AsRef<Path>) -> Result<(u8, Self)> {
        let path = path.as_ref();

        let file = OpenOptions::new().read(true).write(false).open(path)?;


        Self::from_reader_and_path(path, file)
    }

    pub fn from_reader_and_path(path: impl AsRef<Path>, reader: impl Read) -> Result<(u8, Self)> {
        let (file_index, mod_name) = check_yaml_file("modsettings", true, path.as_ref())?;


        #[allow(clippy::match_single_binding)]
        Ok((file_index, match mod_name {
            _ => Self::Unknown(DynYamlDoc::parse_from_reader_and_mod_name(
                reader, mod_name,
            )?),
        }))
    }

    pub fn to_writer(&self, writer: &mut impl Write) -> Result<(), YamlWriteError> {
        match self {
            ParsedModSetting::Unknown(doc) => doc.to_writer(writer),
        }
    }
}

impl DynYamlDoc {
    pub fn parse_from_str_and_mod_name(str: &str, mod_name: &str) -> Result<Self> {
        let yaml = Yaml::load_from_str(str)?;
        Ok(Self(
            mod_name.to_owned(),
            YamlOwned::from_bare_yaml(yaml[0].clone()),
        ))
    }

    pub fn parse_from_reader_and_mod_name(reader: impl Read, mod_name: &str) -> Result<Self> {
        let mut yaml = YamlDecoder::read(reader);
        yaml.encoding_trap(YAMLDecodingTrap::Strict);
        let yaml = yaml.decode();

        match yaml {
            Ok(y) => Ok(Self(
                mod_name.to_owned(),
                YamlOwned::from_bare_yaml(y[0].clone()),
            )),
            Err(e) => match e {
                LoadError::IO(e) => Err(Box::new(e).into()),
                LoadError::Scan(e) => Err(Box::new(e).into()),
                LoadError::Decode(e) => Err(anyhow::format_err!(e)),
            },
        }
    }
}

impl FromYaml for DynYamlDoc {
    fn parse_from_yaml(_yaml: &Yaml) -> Result<Self, YamlParseError> {
        unimplemented!(
            "Don't call YamlFile::parse_from_yaml on DynYamlDoc, use one of the DynYamlDoc \
             methods."
        )
    }

    fn to_yaml(&self) -> Result<Yaml, YamlWriteError> {
        Ok((&self.1).into())
    }
}

impl ModFile for DynYamlDoc {
    const MOD_NAME: &'static str = "";
}

impl ModSave for DynYamlDoc {}
impl ModSession for DynYamlDoc {}
impl ModSettings for DynYamlDoc {}
