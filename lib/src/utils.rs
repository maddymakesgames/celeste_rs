use std::{fmt::Write, io::Read};

use saphyr::{YAMLDecodingTrap, Yaml, YamlDecoder, YamlEmitter, YamlLoader};

use crate::anyhow_utils::AnyhowOption;

pub trait YamlFile: Sized {
    fn parse_from_yaml(yaml: &Yaml) -> anyhow::Result<Self>;

    fn parse_from_str(str: &str) -> anyhow::Result<Self> {
        let yaml = YamlLoader::load_from_str(str)?;
        Self::parse_from_yaml(&yaml[0])
    }

    fn parse_from_reader(reader: impl Read) -> anyhow::Result<Self> {
        let yaml = YamlDecoder::read(reader)
            .encoding_trap(YAMLDecodingTrap::Strict)
            .decode();
        match yaml {
            Ok(y) => Self::parse_from_yaml(&y[0]),
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
        let mut emitter = YamlEmitter::new(writer);
        emitter.multiline_strings(true);
        Ok(emitter.dump(&yaml)?)
    }
}

impl<T: YamlFile> YamlFile for Vec<T> {
    fn parse_from_str(str: &str) -> anyhow::Result<Self> {
        YamlLoader::load_from_str(str)?
            .iter()
            .map(T::parse_from_yaml)
            .collect()
    }

    fn parse_from_reader(reader: impl Read) -> anyhow::Result<Self> {
        let yaml = YamlDecoder::read(reader)
            .encoding_trap(YAMLDecodingTrap::Strict)
            .decode();
        match yaml {
            Ok(y) => y.iter().map(T::parse_from_yaml).collect(),
            Err(e) => match e {
                saphyr::yaml::LoadError::IO(e) => Err(Box::new(e).into()),
                saphyr::yaml::LoadError::Scan(e) => Err(Box::new(e).into()),
                saphyr::yaml::LoadError::Decode(e) => Err(anyhow::format_err!(e)),
            },
        }
    }

    fn parse_from_yaml(yaml: &Yaml) -> anyhow::Result<Self> {
        yaml.as_vec()
            .anyhow("Yaml passed to Vec::parsed_from_yaml isn't an array.")?
            .iter()
            .map(T::parse_from_yaml)
            .collect()
    }

    fn to_yaml(&self) -> anyhow::Result<Yaml> {
        Ok(Yaml::Array(
            self.iter().map(T::to_yaml).collect::<Result<Vec<_>, _>>()?,
        ))
    }
}
