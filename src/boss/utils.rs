use anyhow::{Context, Result as AnyResult};
use std::{fs, path::Path};
use yy_typings::utils::TrailingCommaUtility;

pub fn serialize(absolute_path: &Path, data: &impl serde::Serialize) -> AnyResult<()> {
    let data = serde_json::to_string_pretty(data)?;
    fs::write(absolute_path, data)?;
    Ok(())
}

pub fn deserialize<T>(path: &Path, tcu: Option<&TrailingCommaUtility>) -> AnyResult<T>
where
    for<'de> T: serde::Deserialize<'de>,
{
    let file_string =
        fs::read_to_string(path).with_context(|| format!("path given: {:?}", path))?;
    let data = if let Some(tcu) = tcu {
        serde_json::from_str(&tcu.clear_trailing_comma(&file_string))
    } else {
        serde_json::from_str(&file_string)
    }?;

    Ok(data)
}
