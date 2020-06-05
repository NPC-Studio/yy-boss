use super::{PathStrExt, ViewPathLocationExt};
use log::error;
use maplit::hashmap;
use std::{collections::HashMap, hash::Hash};
use thiserror::Error;
use yy_typings::{FilesystemPath, ViewPath, ViewPathLocation, YypFolder, YypResource};

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
            path: match &self.path_to_parent {
                Some(parent_path) => parent_path.path.join(&self.name),
                None => ViewPathLocation::root(),
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

            for path in view_path.path.component_paths() {
                let path = path.trim_yy();
                folder = &mut folder
                    .folders
                    .get_mut(path)
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
    #[error("foldergraph is out of sync with internal Yyp")]
    FolderGraphOutofSyncWithYyp,
}

pub trait FolderGraphMember {
    type YypReference;

    /// Applies the State of the Folder Graph to the current YypResource which each
    /// folder graph member corresponds to. Essentially, this keeps the foldergraph and the yyp
    /// in sync.
    fn update_yyp(
        &self,
        yyp_resource: &mut Vec<Self::YypReference>,
    ) -> Result<(), FolderGraphError>;
}

#[derive(Debug, Clone, Eq)]
pub struct FileMember {
    pub child: FilesystemPath,
    pub order: usize,
}

impl PartialEq for FileMember {
    fn eq(&self, other: &Self) -> bool {
        self.child == other.child && self.order <= other.order
    }
}

impl FolderGraphMember for FileMember {
    type YypReference = YypResource;

    fn update_yyp(&self, files: &mut Vec<YypResource>) -> Result<(), FolderGraphError> {
        let yyp_resource = files
            .iter_mut()
            .find(|f| f.id.name == self.child.name)
            .ok_or(FolderGraphError::FolderGraphOutofSyncWithYyp)?;

        yyp_resource.order = self.order;
        yyp_resource.id.path = self.child.path.clone();

        Ok(())
    }
}

#[derive(Debug, Clone, Eq)]
pub struct SubfolderMember {
    pub child: FolderGraph,
    pub order: usize,
}

impl PartialEq for SubfolderMember {
    fn eq(&self, other: &Self) -> bool {
        self.child == other.child && self.order <= other.order
    }
}

impl FolderGraphMember for SubfolderMember {
    type YypReference = YypFolder;
    fn update_yyp(&self, folders: &mut Vec<YypFolder>) -> Result<(), FolderGraphError> {
        let yyp_folder = folders
            .iter_mut()
            .find(|f| f.name == self.child.name)
            .ok_or(FolderGraphError::FolderGraphOutofSyncWithYyp)?;

        yyp_folder.order = self.order;
        yyp_folder.folder_path = self.child.view_path().path;

        Ok(())
    }
}
