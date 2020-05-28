use maplit::hashmap;
use std::{collections::HashMap, hash::Hash, path::Path};
use thiserror::Error;
use yy_typings::{FilesystemPath, ViewPath};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderGraph {
    pub name: String,
    pub path_to_parent: Option<ViewPath>,
    pub files: HashMap<String, FileMember>,
    pub folders: HashMap<String, SubfolderMember>,
}

impl Default for FolderGraph {
    fn default() -> Self {
        FolderGraph {
            name: String::new(),
            path_to_parent: None,
            files: hashmap![],
            folders: hashmap![],
        }
    }
}

impl Hash for FolderGraph {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileMember {
    pub child: FilesystemPath,
    pub order: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubfolderMember {
    pub child: FolderGraph,
    pub order: usize,
}

impl FolderGraph {
    pub fn root() -> FolderGraph {
        FolderGraph {
            name: "folders".to_string(),
            ..FolderGraph::default()
        }
    }

    pub fn new(name: String, parent: ViewPath) -> FolderGraph {
        FolderGraph {
            name,
            path_to_parent: Some(parent),
            ..FolderGraph::default()
        }
    }

    pub fn view_path(&self) -> ViewPath {
        ViewPath {
            name: self.name.to_string(),
            path: if let Some(parent_path) = &self.path_to_parent {
                parent_path.path.join(&self.name)
            } else {
                Path::new("folders").to_owned()
            },
        }
    }

    pub fn find_subfolder_mut(
        &mut self,
        view_path: &ViewPath,
    ) -> Result<&mut FolderGraph, FolderGraphError> {
        if view_path.name == self.name {
            Ok(self)
        } else {
            let mut folder = self;

            for path in view_path.path.iter().skip(1) {
                let path_name = path.to_string_lossy();
                let path_name = path_name.trim_end_matches(".yy");

                folder = &mut folder
                    .folders
                    .get_mut(path_name)
                    .ok_or(FolderGraphError::PathNotFound)?
                    .child;
            }

            Ok(folder)
        }
    }

    /// This returns the max_suborder within this Folder. In a sense,
    /// this is the "size" of the folder's children, though due to A-Z sorting
    /// in the Gms2 IDE, order and size are not always directly related.
    ///
    /// Given the folder:
    /// ```norun
    /// Sprites/
    ///     - spr_player
    ///     - OtherMembers/
    ///     - spr_enemy
    /// ```
    /// `max_suborder` will return `2`, which is what `spr_enemy`'s suborder would be.
    pub fn max_suborder(&self) -> Option<usize> {
        let mut output = None;

        if let Some(file_max) = self.files.values().map(|file| file.order).max() {
            if file_max >= output.unwrap_or_default() {
                output = Some(file_max);
            }
        }

        if let Some(file_max) = self.folders.values().map(|file| file.order).max() {
            if file_max >= output.unwrap_or_default() {
                output = Some(file_max);
            }
        }

        output
    }
}

#[derive(Debug, Error)]
pub enum FolderGraphError {
    #[error("path was not found")]
    PathNotFound,
    #[error("folder already existed at that location")]
    FolderAlreadyPresent,
    #[error("file already existed at that location")]
    FileAlreadyPresent,
}
