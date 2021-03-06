use log::error;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;
use crate::{
    folders::{FlatFolderGraph, Item},
    ProjectMetadata, SerializedData,
};
use yy_typings::ViewPath;

#[derive(Debug, Serialize, Deserialize)]
#[must_use = "this `Output` must be printed"]
#[serde(tag = "type")]
#[allow(clippy::large_enum_variant)]
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
    pub error: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_metadata: Option<ProjectMetadata>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CommandOutput {
    pub success: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<YypBossError>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub exists: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_is_valid: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource: Option<SerializedData>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub associated_data: Option<SerializedData>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub flat_folder_graph: Option<FlatFolderGraph>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_kind: Option<Item>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_folder: Option<ViewPath>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_names: Option<Vec<(String, String)>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_path: Option<PathBuf>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_metadata: Option<ProjectMetadata>,
}

impl CommandOutput {
    pub fn error(yyp_boss_error: YypBossError) -> Self {
        Self {
            success: false,
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

    pub fn ok_metadata(pd: ProjectMetadata) -> Self {
        Self {
            success: true,
            project_metadata: Some(pd),
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

    pub fn ok_resource(resource: SerializedData) -> Self {
        Self {
            success: true,
            resource: Some(resource),
            ..Self::default()
        }
    }

    pub fn ok_associated_data(associated_data: SerializedData) -> Self {
        Self {
            success: true,
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

    pub fn ok_name_is_valid(valid: bool) -> Self {
        Self {
            success: true,
            name_is_valid: Some(valid),
            ..Self::default()
        }
    }

    pub fn ok_folder_graph(ff_graph: FlatFolderGraph) -> Self {
        Self {
            success: true,
            flat_folder_graph: Some(ff_graph),
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

    pub fn ok_created_folder(f: ViewPath) -> Self {
        Self {
            success: true,
            created_folder: Some(f),
            ..Self::default()
        }
    }

    pub fn ok_event_names(ev: Vec<(String, String)>) -> Self {
        Self {
            success: true,
            event_names: Some(ev),
            ..Self::default()
        }
    }

    pub fn ok_path(path: PathBuf) -> Self {
        Self {
            success: true,
            requested_path: Some(path),
            ..Self::default()
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Shutdown {
    pub msg: String,
}

#[derive(Debug, Error, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum YypBossError {
    #[error("could not read Command, error: {}", .data)]
    CouldNotReadCommand { data: String },

    #[error("error manipulating resource, error: {}", .data)]
    ResourceManipulation { data: String },

    #[error("folder graph error, error: {}", .data)]
    FolderGraphError { data: String },

    #[error("could not read yyfile, error: {}", .data)]
    YyParseError { data: String },

    #[error("could not read associated data, error: {}", .data)]
    AssociatedDataParseError { data: String },

    #[error("could not output data -- operation was SUCCESFUL, but data could not be returned because {}", .data)]
    CouldNotOutputData { data: String },

    #[error("could not serialize yypboss...coarse error {}", .data)]
    CouldNotSerializeYypBoss { data: String },

    #[error("internal error -- command could not be executed. error is fatal: {}", .fatal)]
    InternalError { fatal: bool },
}
