use crate::{folders::FolderGraphError, FileSerializationError, SerializedDataError};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StartupError {
    #[error("couldn't deserialize yyp -- {0}")]
    BadYypDeserialize(String),

    #[error("yyp is wrong year version -- needed {0}, got {1}")]
    YypYearNotMatch(String, String),

    #[error("yyp is wrong version -- needed {0}, got {1}")]
    YypDoesNotMatch(semver::VersionReq, semver::Version),

    #[error("couldn't make or find the boss directory -- {0}")]
    BossDirectory(String),

    #[error("couldn't deserialize file at {filepath:?} -- {error}")]
    BadYyFile { filepath: PathBuf, error: String },

    #[error("couldn't read resource {} in yyp -- bad subpath given", .0.display())]
    BadResourceListing(PathBuf),

    #[error("couldn't load in {0} because {1}")]
    BadAssociatedData(String, YyResourceHandlerError),

    #[error("couldn't load in resource {name} in Asset Browser. Could be corrupted -- {error}")]
    BadResourceTree { name: String, error: String },

    #[error("bad path for yyp was given -- {yyp_filepath:?}, {error}")]
    BadYypPath {
        yyp_filepath: PathBuf,
        error: String,
    },

    #[error("a working directory path was given, but it was invalid")]
    BadWorkingDirectoryPath,

    #[error("bad arguments -- {0}")]
    BadCliArguments(String),
}

#[derive(Debug, Error)]
pub enum ResourceManipulationError {
    #[error(transparent)]
    FolderGraphError(#[from] FolderGraphError),

    #[error("name already used")]
    NameCollision,

    #[error("bad name given")]
    BadName,

    #[error("cannot find that resource")]
    BadGet,

    #[error("internal error -- yyp-boss is in undefined state")]
    InternalError,

    #[error("resource cannot be manipulated yet -- yyp-boss does not have full support yet. please file an issue")]
    ResourceCannotBeManipulated,
}

#[derive(Debug, Error)]
pub enum YyResourceHandlerError {
    #[error(transparent)]
    FileSerializationError(#[from] FileSerializationError),

    #[error(transparent)]
    SerializedDataError(#[from] SerializedDataError),

    #[error("the given resource was not found or managed on the type")]
    ResourceNotFound,

    #[error("we cannot force serialization because the associated data could not be found")]
    CannotForceSerialization,
}
