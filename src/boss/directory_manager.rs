use super::{errors::StartupError, utils};
use crate::FileSerializationError;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Eq)]
pub struct DirectoryManager {
    /// The Directory which houses the Yyp and the various other folders
    /// that Gms2 projects need, such as `sprites` or `objects`.
    root_directory: PathBuf,
    yyp: PathBuf,
    boss_directory: PathBuf,
}

impl DirectoryManager {
    const YYBOSS_DIR: &'static str = ".boss";

    pub fn new(yyp: &Path) -> Result<Self, StartupError> {
        let root_directory = yyp
            .parent()
            .ok_or_else(|| StartupError::BadYypPath)?
            .to_owned();

        let boss_directory = root_directory.join(Path::new(Self::YYBOSS_DIR));

        if boss_directory.exists() == false {
            std::fs::create_dir(&boss_directory).map_err(|e| {
                StartupError::FileSerializationError(FileSerializationError::Io(e.to_string()))
            })?;
        }

        let output = DirectoryManager {
            boss_directory,
            root_directory,
            yyp: yyp.to_owned(),
        };

        Ok(output)
    }

    pub fn root_directory(&self) -> &Path {
        &self.root_directory
    }

    pub fn yyp(&self) -> &Path {
        &self.yyp
    }

    pub fn boss_file(&self, relative_path: &Path) -> PathBuf {
        self.boss_directory.join(relative_path)
    }

    pub fn resource_file(&self, relative_path: &Path) -> PathBuf {
        self.root_directory.join(&relative_path)
    }

    pub fn serialize_boss_file(
        &self,
        relative_path: &Path,
        data: String,
    ) -> Result<(), utils::FileSerializationError> {
        std::fs::write(self.boss_file(relative_path), data)
            .map_err(|e| FileSerializationError::Io(e.to_string()))
    }
}
