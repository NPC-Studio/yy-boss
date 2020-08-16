use crate::{
    boss::{folders::FolderGraphError, FileSerializationError},
    Resource, SerializedDataError,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Serialize, Deserialize)]
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

#[derive(Debug, Error)]
pub enum ResourceManipulationError {
    #[error(transparent)]
    FolderGraphError(#[from] FolderGraphError),

    #[error("cannot use that resource name, as that name is being used already by a {}", .0)]
    BadResourceName(Resource),

    #[error("no resource found of that name")]
    NoResourceByThatName,

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
