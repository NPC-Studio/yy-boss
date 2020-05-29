use anyhow::Result as AnyResult;
use std::{fs, path::Path};

pub fn serialize(absolute_path: &Path, data: &impl serde::Serialize) -> AnyResult<()> {
    let data = serde_json::to_string_pretty(data)?;
    fs::write(absolute_path, data)?;
    Ok(())
}

// fn deserialize_json(path: &Path) -> Result<serde_json::Value> {
//     let file_string = fs::read_to_string(path)?;
//     let data = serde_json::from_str(&file_string)?;
//     Ok(data)
// }
