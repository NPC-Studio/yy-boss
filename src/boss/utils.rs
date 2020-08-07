use std::{fs, path::Path};
use thiserror::Error;
use yy_typings::utils::TrailingCommaUtility;

#[derive(Debug, Error, serde::Serialize, serde::Deserialize)]
pub enum FileSerializationError {
    #[error("serde error message, {}", .0)]
    Serde(String),
    #[error("io error, {}", 0)]
    Io(String),
}

pub fn serialize(
    absolute_path: &Path,
    data: &impl serde::Serialize,
) -> Result<(), FileSerializationError> {
    let data = serde_json::to_string_pretty(data)
        .map_err(|e| FileSerializationError::Serde(e.to_string()))?;
    fs::write(absolute_path, data).map_err(|e| FileSerializationError::Io(e.to_string()))?;
    Ok(())
}

pub fn deserialize<T>(
    path: &Path,
    tcu: Option<&TrailingCommaUtility>,
) -> Result<T, FileSerializationError>
where
    for<'de> T: serde::Deserialize<'de>,
{
    let file_string =
        fs::read_to_string(path).map_err(|e| FileSerializationError::Io(e.to_string()))?;
    let data = if let Some(tcu) = tcu {
        serde_json::from_str(&tcu.clear_trailing_comma(&file_string))
    } else {
        serde_json::from_str(&file_string)
    }
    .map_err(|e| FileSerializationError::Serde(e.to_string()))?;

    Ok(data)
}
