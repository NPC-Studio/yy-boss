use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
use thiserror::Error;
use yy_typings::utils::TrailingCommaUtility;

#[derive(Debug, Error, Serialize, Deserialize)]
#[serde(rename = "camelCase")]
pub enum FileSerializationError {
    #[error("serde error message, {}", .0)]
    Serde(String),
    #[error("io error, {}", 0)]
    Io(String),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum SerializationFormat {
    Json,
    Yaml,
}

impl SerializationFormat {
    pub fn file_ending(&self) -> &'static str {
        match self {
            SerializationFormat::Json => "json",
            SerializationFormat::Yaml => "yaml",
        }
    }

    pub fn serialize_and_write(
        &self,
        absolute_path: &Path,
        data: &impl serde::Serialize,
    ) -> Result<(), FileSerializationError> {
        match self {
            SerializationFormat::Json => serialize_json(absolute_path, data),
            SerializationFormat::Yaml => serialize_yaml(absolute_path, data),
        }
    }

    pub fn deserialize_and_read<T>(&self, path: &Path) -> Result<T, FileSerializationError>
    where
        for<'de> T: serde::Deserialize<'de>,
    {
        match self {
            SerializationFormat::Json => deserialize_json(path),
            SerializationFormat::Yaml => deserialize_yaml(path),
        }
    }

    pub fn serialize(
        &self,
        data: &impl serde::Serialize,
    ) -> Result<String, FileSerializationError> {
        match self {
            SerializationFormat::Json => serde_json::to_string_pretty(data)
                .map_err(|e| FileSerializationError::Serde(e.to_string())),
            SerializationFormat::Yaml => serde_yaml::to_string(data)
                .map_err(|e| FileSerializationError::Serde(e.to_string())),
        }
    }
}

impl Default for SerializationFormat {
    fn default() -> Self {
        Self::Json
    }
}

pub fn serialize_json(
    absolute_path: &Path,
    data: &impl serde::Serialize,
) -> Result<(), FileSerializationError> {
    let data = serde_json::to_string_pretty(data)
        .map_err(|e| FileSerializationError::Serde(e.to_string()))?;
    fs::write(absolute_path, data).map_err(|e| FileSerializationError::Io(e.to_string()))?;
    Ok(())
}

pub fn serialize_yaml(
    absolute_path: &Path,
    data: &impl serde::Serialize,
) -> Result<(), FileSerializationError> {
    let data =
        serde_yaml::to_string(data).map_err(|e| FileSerializationError::Serde(e.to_string()))?;
    fs::write(absolute_path, data).map_err(|e| FileSerializationError::Io(e.to_string()))?;
    Ok(())
}

pub fn deserialize_json<T>(path: &Path) -> Result<T, FileSerializationError>
where
    for<'de> T: serde::Deserialize<'de>,
{
    let file_string =
        fs::read_to_string(path).map_err(|e| FileSerializationError::Io(e.to_string()))?;
    let data = serde_json::from_str(&file_string)
        .map_err(|e| FileSerializationError::Serde(e.to_string()))?;

    Ok(data)
}

pub fn deserialize_yaml<T>(path: &Path) -> Result<T, FileSerializationError>
where
    for<'de> T: serde::Deserialize<'de>,
{
    let file_string =
        fs::read_to_string(path).map_err(|e| FileSerializationError::Io(e.to_string()))?;
    let data = serde_yaml::from_str(&file_string)
        .map_err(|e| FileSerializationError::Serde(e.to_string()))?;

    Ok(data)
}

pub fn deserialize_json_tc<T, P: AsRef<Path>>(
    path: P,
    tcu: &TrailingCommaUtility,
) -> Result<T, FileSerializationError>
where
    for<'de> T: serde::Deserialize<'de>,
{
    let file_string =
        fs::read_to_string(path).map_err(|e| FileSerializationError::Io(e.to_string()))?;
    let data = serde_json::from_str(&tcu.clear_trailing_comma(&file_string))
        .map_err(|e| FileSerializationError::Serde(e.to_string()))?;

    Ok(data)
}
