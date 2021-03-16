use super::metadata::{ConfigItem, Metadata};
use super::template::Template;
use crate::api::v1::Rune as ApiRune;
use crate::rune::error::Error;
use serde_derive::{Deserialize, Serialize};
use serde_yaml::{from_slice, to_vec};
use std::collections::HashMap;
use std::fs::read;
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use zip::result::ZipError;
use zip::write::FileOptions;
use zip::{ZipArchive, ZipWriter};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Rune {
    pub metadata: Metadata,
    pub template: Vec<Template>,
    pub transformers: Option<String>,
    pub react: Option<String>,
}

impl Rune {
    pub fn load<P: Into<PathBuf>>(path: P) -> Result<Self, Error> {
        let path = path.into();
        let metadata = from_slice(&read(path.join("metadata.yaml"))?)?;
        let template = from_slice(&read(path.join("rune.yaml"))?)?;
        let transformers = read(path.join("transformers.py"))
            .and_then(|bytes| Ok(String::from_utf8_lossy(&bytes).to_string()))
            .ok();
        let react = read(path.join("rune.py"))
            .and_then(|bytes| Ok(String::from_utf8_lossy(&bytes).to_string()))
            .ok();

        Ok(Self {
            metadata,
            template,
            transformers,
            react,
        })
    }

    pub fn zip(&self) -> Result<Vec<u8>, Error> {
        let buffer = Cursor::new(Vec::new());
        let mut writer = ZipWriter::new(buffer);

        writer.start_file_from_path(Path::new("metadata.yaml"), FileOptions::default())?;
        writer.write_all(&to_vec(&self.metadata)?)?;
        writer.start_file_from_path(Path::new("rune.yaml"), FileOptions::default())?;
        writer.write_all(&to_vec(&self.template)?)?;

        if let Some(tfs) = &self.transformers {
            writer.start_file_from_path(Path::new("transformers.py"), FileOptions::default())?;
            writer.write_all(tfs.as_bytes())?;
        }

        if let Some(react) = &self.react {
            writer.start_file_from_path(Path::new("rune.py"), FileOptions::default())?;
            writer.write_all(react.as_bytes())?;
        }

        let finished = writer.finish()?;
        Ok(finished.into_inner())
    }

    pub fn unzip(bytes: &[u8]) -> Result<Rune, Error> {
        let buffer = Cursor::new(bytes);
        let mut reader = ZipArchive::new(buffer).unwrap();

        let mut metadata = String::new();
        reader
            .by_name("metadata.yaml")?
            .read_to_string(&mut metadata)?;
        let metadata = from_slice(metadata.as_bytes())?;

        let mut template = String::new();
        reader.by_name("rune.yaml")?.read_to_string(&mut template)?;
        let template = from_slice(template.as_bytes())?;

        let transformers = match reader.by_name("transformers.py") {
            Ok(mut f) => {
                let mut buf = String::new();
                f.read_to_string(&mut buf)?;
                Ok(Some(buf))
            }
            Err(ZipError::FileNotFound) => Ok(None),
            Err(err) => Err(Error::ZipError(err)),
        }?;

        let react = match reader.by_name("rune.py") {
            Ok(mut f) => {
                let mut buf = String::new();
                f.read_to_string(&mut buf)?;
                Ok(Some(buf))
            }
            Err(ZipError::FileNotFound) => Ok(None),
            Err(err) => Err(Error::ZipError(err)),
        }?;

        let rune = Self {
            metadata,
            template,
            transformers,
            react,
        };
        Ok(rune)
    }
}

impl Into<ApiRune> for Rune {
    fn into(self) -> ApiRune {
        let mut state = HashMap::new();

        for (name, item) in &self.metadata.config {
            state.insert(
                name.clone(),
                match item {
                    ConfigItem::Boolean { default, .. } => Some(default.to_string()),
                    ConfigItem::Integer { default, .. } => Some(default.to_string()),
                    ConfigItem::String { default, .. } => Some(default.to_string()),
                    ConfigItem::Secret { .. } => None,
                    ConfigItem::Archive { .. } => None,
                },
            );
        }

        ApiRune {
            transformers: self.transformers.clone(),
            react: self.react.clone(),
            state,
        }
    }
}
