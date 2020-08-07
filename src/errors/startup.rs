use crate::{boss::FileSerializationError, FolderGraphError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum StartupError {
    #[error(transparent)]
    FileSerializationError(#[from] FileSerializationError),
    #[error("yyp internally inconsistent -- could not load folders, {}", .0)]
    InternalYypError(#[from] FolderGraphError),
    #[error("bad path for yyp was given -- couldn't find parent directory")]
    BadPath,
}
