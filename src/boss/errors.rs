use crate::{
    boss::{folders::FolderGraphError, FileSerializationError},
    Resource, SerializedDataError,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum StartupError {
    #[error("couldn't deserialize yyp -- {}", .0)]
    BadYypDeserialize(String),

    #[error("couldn't make or find the boss directory -- {}", .0)]
    BossDirectory(String),

    #[error("couldn't deserialize file at {:?} -- {}", .filepath, .error)]
    BadYyFile { filepath: PathBuf, error: String },

    #[error("couldn't load in resource {} in Asset Browser. Could be corrupted -- {}", .name, .error)]
    BadResourceTree { name: String, error: String },

    #[error("bad path for yyp was given -- {:?}, {}", .yyp_filepath, .error)]
    BadYypPath {
        yyp_filepath: PathBuf,
        error: String,
    },

    #[error("a working directory path was given, but it was invalid")]
    BadWorkingDirectoryPath,

    #[error("bad arguments -- {}", .0)]
    BadCliArguments(String),
}

#[derive(Debug, Error, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResourceManipulationError {
    #[error(transparent)]
    FolderGraphError(#[from] FolderGraphError),

    #[error("cannot add that resource -- a {} of that name already exists", .existing_resource)]
    #[serde(rename_all = "camelCase")]
    BadAdd { existing_resource: Resource },

    #[error("cannot find that resource")]
    BadGet,

    #[error("internal error -- yyp-boss is in undefined state")]
    InternalError,
}

#[derive(Debug, Error)]
pub enum YyResourceHandlerErrors {
    #[error(transparent)]
    FileSerializationError(#[from] FileSerializationError),

    #[error(transparent)]
    SerializedDataError(#[from] SerializedDataError),

    #[error("the given resource was not found or managed on the type")]
    ResourceNotFound,
}
