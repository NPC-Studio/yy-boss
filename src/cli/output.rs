use log::error;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use yy_boss::{
    folders::{FolderGraph, FolderGraphError},
    Resource, SerializedData, SerializedDataError, StartupError,
};

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

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CommandOutput {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exists: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<YypBossError>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub fatal: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource: Option<SerializedData>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub associated_data: Option<SerializedData>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub folder_graph: Option<FolderGraph>,
}

impl CommandOutput {
    pub fn error(yyp_boss_error: YypBossError) -> Self {
        Self {
            success: false,
            fatal: Some(false),
            error: Some(yyp_boss_error),
            ..Self::default()
        }
    }

    pub fn ok() -> Self {
        Self {
            success: true,
            ..Self::default()
        }
    }

    pub fn ok_datum(resource: SerializedData, associated_data: SerializedData) -> Self {
        Self {
            success: true,
            resource: Some(resource),
            associated_data: Some(associated_data),
            ..Self::default()
        }
    }

    pub fn ok_exists(exists: bool) -> Self {
        Self {
            success: true,
            exists: Some(exists),
            ..Self::default()
        }
    }
    
    pub fn ok_folder_graph(f_graph: FolderGraph) -> Self {
        Self {
            success: true,
            folder_graph: Some(f_graph),
            ..Self::default()
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

    #[error("error in the internal virtual file system, {}", .0)]
    FolderGraphError(#[from] FolderGraphError),

    #[error("bad add command. {} already exists by the name {}", .0, .1)]
    BadAdd(Resource, String),

    #[error("bad replace command. no resource by the name {} existed", .0)]
    BadReplace(String),

    #[error("bad remove command. no resource by the name {} existed", .0)]
    BadRemove(String),

    #[error("bad remove command. no resource the name {} existed", .0)]
    BadGet(Resource, String),

    #[error("was given a `Data::File` tag, but was not given a working directory on startup. cannot parse")]
    NoFileMode,

    #[error(
        "cannot be represented with utf8 encoding; must use `Data::File` or `Data::DefaultValue`"
    )]
    CannotUseValue,

    #[error("was given a `Data::File` tag, but path didn't exist, wasn't a file, or couldn't be read. path was {}", .0.to_string_lossy())]
    BadDataFile(std::path::PathBuf),

    #[error("internal error -- command could not be executed. error is fatal: {}", .0)]
    InternalError(bool),
}

impl From<SerializedDataError> for YypBossError {
    fn from(e: SerializedDataError) -> Self {
        match e {
            SerializedDataError::NoFileMode => YypBossError::NoFileMode,
            SerializedDataError::BadDataFile(v) => YypBossError::BadDataFile(v),
            SerializedDataError::CouldNotParseData(v) => {
                YypBossError::CouldNotParseYyFile(v.to_string())
            }
            SerializedDataError::CannotUseValue => YypBossError::CannotUseValue,
            SerializedDataError::CouldNotWriteImage(e) => {
                error!("We couldn't write the image...{}", e);
                YypBossError::InternalError(false)
            }
            SerializedDataError::InnerError(e) => {
                error!("{}", e);
                YypBossError::InternalError(false)
            }
            SerializedDataError::CouldNotReadFile(e) => {
                error!("Couldn't read a file...{}", e);
                YypBossError::InternalError(false)
            }
        }
    }
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
