use super::errors::StartupError;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Eq, Default)]
pub struct DirectoryManager {
    /// The path to the folder which houses the Yyp and the various other folders
    /// that Gms2 projects need, such as `sprites` or `objects`.
    root_directory: PathBuf,

    /// The path to the yyp itself, which is within the `root_directory`. We cache it here
    /// for simplicity.
    ///
    /// The yyp path is `root_directory.join(yyp_name)`.
    yyp: PathBuf,
}

impl DirectoryManager {
    /// Creates a new Directory manager and initializes the boss directory, if it doesn't exist.
    pub(crate) fn new(yyp: &Path) -> Result<Self, StartupError> {
        let root_directory = yyp
            .parent()
            .ok_or_else(|| StartupError::BadYypPath {
                yyp_filepath: yyp.to_owned(),
                error: "no parent directory to find".to_string(),
            })?
            .to_owned();

        let output = DirectoryManager {
            root_directory,
            yyp: yyp.to_owned(),
        };

        Ok(output)
    }

    /// Returns the root directory of the project.
    pub fn root_directory(&self) -> &Path {
        &self.root_directory
    }

    /// Returns the path to the yyp of the project.
    pub fn yyp(&self) -> &Path {
        &self.yyp
    }

    /// Creates a path within the root directory, probably to a resource.
    pub fn resource_file(&self, relative_path: &Path) -> PathBuf {
        self.root_directory.join(relative_path)
    }
}
