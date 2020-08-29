use crate::{
    boss::{folders::FolderGraphError, FileSerializationError},
    Resource, SerializedDataError,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Serialize, Deserialize)]
#[serde(rename = "camelCase")]
pub enum StartupError {
    #[error(transparent)]
    FileSerializationError(#[from] FileSerializationError),

    #[error("yyp internally inconsistent -- could not load folders, {}", .0)]
    InternalYypError(#[from] FolderGraphError),

    #[error("bad path for yyp was given -- couldn't find parent directory")]
    BadYypPath,
    #[error("a working directory path was given, but it was invalid")]
    BadWorkingDirectoryPath,
}

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum ResourceManipulationError {
    #[error(transparent)]
    FolderGraphError(#[from] FolderGraphError),

    #[error("cannot add that resource -- a {} of that name already exists", .0)]
    BadAdd(Resource),

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
