use super::{errors::StartupError, utils};
use crate::FileSerializationError;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Eq)]
pub struct DirectoryManager {
    /// The path to the folder which houses the Yyp and the various other folders
    /// that Gms2 projects need, such as `sprites` or `objects`.
    root_directory: PathBuf,

    /// The path to the yyp itself, which is within the `root_directory`. We cache it here
    /// for simplicity.
    ///
    /// The yyp path is `root_directory.join(yyp_name)`.
    yyp: PathBuf,

    /// The path to the boss directory, which is within the `root_directory`. We cache it here
    /// for simplicity.
    ///
    /// The boss directory is `root_directory.join(".boss")`.
    boss_directory: PathBuf,
}

impl DirectoryManager {
    const YYBOSS_DIR: &'static str = ".boss";

    /// Creates a new Directory manager and initializes the boss directory, if it doesn't exist.
    pub(crate) fn new(yyp: &Path) -> Result<Self, StartupError> {
        let root_directory = yyp
            .parent()
            .ok_or_else(|| StartupError::BadYypPath {
                yyp_filepath: yyp.to_owned(),
                error: "no parent directory to find".to_string(),
            })?
            .to_owned();

        let boss_directory = root_directory.join(Path::new(Self::YYBOSS_DIR));

        if boss_directory.exists() == false {
            std::fs::create_dir(&boss_directory)
                .map_err(|e| StartupError::BossDirectory(e.to_string()))?;
        }

        let output = DirectoryManager {
            boss_directory,
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

    /// Creates a path into the boss directory.
    pub fn boss_file(&self, relative_path: &Path) -> PathBuf {
        self.boss_directory.join(relative_path)
    }

    /// Creates a path within the root directory, probably to a resource.
    pub fn resource_file(&self, relative_path: &Path) -> PathBuf {
        self.root_directory.join(&relative_path)
    }

    /// Saves a file within the boss directory, as a compound operation.
    pub fn serialize_boss_file(
        &self,
        relative_path: &Path,
        data: String,
    ) -> Result<(), utils::FileSerializationError> {
        std::fs::write(self.boss_file(relative_path), data)
            .map_err(|e| FileSerializationError::Io(e.to_string()))
    }
}
