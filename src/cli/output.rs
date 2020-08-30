use log::error;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use yy_boss::{
    folders::{FolderGraph, FolderGraphError, Item},
    ResourceManipulationError, SerializedData, StartupError,
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

    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_kind: Option<Item>,
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

    pub fn ok_datum(resource: SerializedData, associated_data: Option<SerializedData>) -> Self {
        Self {
            success: true,
            resource: Some(resource),
            associated_data,
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

    pub fn ok_path_kind(item: Item) -> Self {
        Self {
            success: true,
            path_kind: Some(item),
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

    #[error(transparent)]
    ResourceManipulation(#[from] ResourceManipulationError),

    #[error(transparent)]
    FolderGraphError(#[from] FolderGraphError),

    #[error("could not read yyfile, error: {}", .0)]
    YyParseError(String),

    #[error("could not read associated data, error: {}", .0)]
    AssociatedDataParseError(String),

    #[error("could not output data -- operation was SUCCESFUL, but data could not be returned because {}", .0)]
    CouldNotOutputData(String),

    #[error("internal error -- command could not be executed. error is fatal: {}", .0)]
    InternalError(bool),
}
