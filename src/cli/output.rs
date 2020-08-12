use serde::{Deserialize, Serialize};
use thiserror::Error;
use yy_boss::{FolderGraphError, Resource, SerializedData, StartupError};

#[derive(Debug, Serialize, Deserialize)]
#[must_use = "this `Output` must be printed"]
#[serde(tag = "type")]
pub enum Output {
    Startup(Startup),
    Command(CommandOutput),
    Shutdown(Shutdown),
}

impl Output {
    pub fn print(self) {
        let output = serde_json::to_string_pretty(&self).unwrap();
        println!("{}", output);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Startup {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<StartupError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandOutput {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<YypBossError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fatal: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource: Option<SerializedData>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub associated_data: Option<SerializedData>,
}

impl CommandOutput {
    pub fn error(yyp_boss_error: YypBossError) -> Self {
        Self {
            success: false,
            fatal: Some(false),
            error: Some(yyp_boss_error),
            resource: None,
            associated_data: None,
        }
    }

    pub fn ok() -> Self {
        Self {
            success: true,
            error: None,
            fatal: None,
            resource: None,
            associated_data: None,
        }
    }

    pub fn ok_datum(resource: SerializedData, associated_data: SerializedData) -> Self {
        Self {
            success: true,
            error: None,
            fatal: None,
            resource: Some(resource),
            associated_data: Some(associated_data),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Shutdown {
    pub msg: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InputResponse {
    pub msg: String,
    pub fatal: bool,
}

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum YypBossError {
    #[error("could not read Command, error: {}", .0)]
    CouldNotReadCommand(String),

    #[error("could not parse Yy File Provided, error: {}", .0)]
    CouldNotParseYyFile(String),

    #[error("could not parse Associated Data Provided, error: {}", .0)]
    CouldNotParseAssociatedData(String),

    #[error("yyp internally inconsistent -- could not load folders, {}", .0)]
    FolderGraphError(#[from] FolderGraphError),

    #[error("bad add command. {} already exists by the name {}", .0, .1)]
    BadAdd(Resource, String),

    #[error("bad replace command. no resource by the name {} existed", .0)]
    BadReplace(String),

    #[error("internal error -- YypBoss is unstable and condition is undefined. please report an error with logs")]
    InternalError,

    #[error("was given a `Data::File` tag, but was not given a working directory on startup. cannot parse")]
    NoFileMode,

    #[error(
        "cannot be represented with utf8 encoding; must use `Data::File` or `Data::DefaultValue`"
    )]
    CannotUseValue,

    #[error("was given a `Data::File` tag, but path didn't exist, wasn't a file, or couldn't be read. path was {}", .0.to_string_lossy())]
    BadDataFile(std::path::PathBuf),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_serde() {
        Output::Startup(Startup {
            success: false,
            error: Some(StartupError::BadYypPath),
        })
        .print();
    }
}
