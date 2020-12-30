pub mod api;
pub mod error;
pub mod metadata;
pub mod template;

use crate::error::Error;
use crate::metadata::Metadata;
use crate::template::Template;
use serde_derive::{Deserialize, Serialize};
use serde_yaml::{from_slice, to_vec};
use std::fs::read;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use zip::write::FileOptions;
use zip::ZipWriter;

#[derive(Debug, Deserialize, Serialize)]
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
}
