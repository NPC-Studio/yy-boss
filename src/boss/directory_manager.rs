use super::utils;
use anyhow::{anyhow, Result as AnyResult};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct DirectoryManager {
    root_directory: PathBuf,
    yyp: PathBuf,
    boss_directory: PathBuf,
}

impl DirectoryManager {
    const YYBOSS_DIR: &'static str = ".boss";

    pub fn new(yyp: &Path) -> AnyResult<DirectoryManager> {
        let root_directory = yyp
            .parent()
            .ok_or(anyhow!("couldn't get parent"))?
            .to_owned();

        let boss_directory = root_directory.join(Path::new(Self::YYBOSS_DIR));

        if boss_directory.exists() == false {
            std::fs::create_dir(&boss_directory)?;
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

    pub fn serialize_boss_file(
        &self,
        relative_path: &Path,
        value: &impl serde::Serialize,
    ) -> AnyResult<()> {
        utils::serialize(&self.boss_file(relative_path), value)
    }
}
