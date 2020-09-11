use crate::Resource;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum FolderGraphError {
    #[error("path {} was not found", .path)]
    PathNotFound { path: String },

    #[error("folder already existed at that location")]
    FolderAlreadyPresent,

    #[error("file already existed at that location")]
    FileAlreadyPresent,

    #[error("foldergraph is out of sync with internal Yyp -- yypboss is in undefined state")]
    InternalError,

    #[error("couldn't remove folder, not empty")]
    CannotRemoveFolder,

    #[error("cannot remove the root folder, why are you doing that don't do that come on now")]
    CannotEditRootFolder,

    #[error("cannot move folder inside itself")]
    InvalidMoveDestination,

    #[error(transparent)]
    ResourceNameError(#[from] ResourceNameError),
}

#[derive(Debug, Error, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(tag = "resourceNameError")]
pub enum ResourceNameError {
    #[error("cannot use that resource name, as that name is being used already by a {}", .existing_resource)]
    #[serde(rename_all = "camelCase")]
    BadResourceName { existing_resource: Resource },

    #[error("no resource found of that name")]
    NoResourceByThatName,
}
