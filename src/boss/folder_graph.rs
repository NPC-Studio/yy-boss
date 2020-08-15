use super::{PathStrExt, ViewPathLocationExt};

use log::error;
use maplit::btreemap;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, hash::Hash};
use thiserror::Error;
use yy_typings::{FilesystemPath, ViewPath, ViewPathLocation, YypFolder, YypResource};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FolderGraphManager {
    pub root: FolderGraph,
    root_file_location: ViewPathLocation,
}

impl FolderGraphManager {
    pub(crate) fn new(yyp_name: &str) -> Self {
        FolderGraphManager {
            root: FolderGraph::root(),
            root_file_location: ViewPathLocation::root_file(yyp_name),
        }
    }

    pub(crate) fn find_subfolder_mut(
        &mut self,
        view_path: &ViewPathLocation,
    ) -> Result<&mut FolderGraph, FolderGraphError> {
        if *view_path == self.root_file_location {
            Ok(&mut self.root)
        } else {
            let mut folder = &mut self.root;
            let mut used_root = true;

            for path in view_path.component_paths() {
                used_root = false;
                let path = path.trim_yy();
                folder = &mut folder
                    .folders
                    .get_mut(path)
                    .ok_or(FolderGraphError::PathNotFound)?
                    .child;
            }

            if used_root == false {
                Ok(folder)
            } else {
                Err(FolderGraphError::PathNotFound)
            }
        }
    }

    pub(crate) fn get_folder_by_fname_mut(&mut self, name: &str) -> Option<&mut FolderGraph> {
        fn iterable<'a>(name: &str, fg: &'a mut FolderGraph) -> Option<&'a mut FolderGraph> {
            if fg.files.contains_key(name) {
                return Some(fg);
            }

            for subfolder in fg.folders.values_mut() {
                if let Some(found) = iterable(name, &mut subfolder.child) {
                    return Some(found);
                }
            }

            None
        }

        iterable(name, &mut self.root)
    }

    /// Clones a folder
    pub fn clone_folder(&self, view_path: &ViewPathLocation) -> Option<FolderGraph> {


        Some(folder.clone())
    }
}
#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
pub struct FolderGraph {
    pub name: String,
    pub path_to_parent: Option<ViewPathLocation>,
    pub files: BTreeMap<String, FileMember>,
    pub folders: BTreeMap<String, SubfolderMember>,
}

impl Default for FolderGraph {
    fn default() -> Self {
        FolderGraph {
            name: String::new(),
            path_to_parent: None,
            files: btreemap![],
            folders: btreemap![],
        }
    }
}

impl PartialEq for FolderGraph {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Hash for FolderGraph {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl FolderGraph {
    fn root() -> FolderGraph {
        FolderGraph {
            name: "folders".to_string(),
            ..FolderGraph::default()
        }
    }

    pub fn new(name: String, parent: ViewPathLocation) -> FolderGraph {
        FolderGraph {
            name,
            path_to_parent: Some(parent),
            ..FolderGraph::default()
        }
    }

    pub fn view_path_location(&self) -> ViewPathLocation {
        match &self.path_to_parent {
            Some(parent_path) => parent_path.join(&self.name),
            None => ViewPathLocation::root_folder(),
        }
    }

    pub fn view_path(&self) -> ViewPath {
        let path = match &self.path_to_parent {
            Some(parent_path) => parent_path.join(&self.name),
            None => ViewPathLocation::root_folder(),
        };

        ViewPath {
            name: self.name.clone(),
            path,
        }
    }

    /// This returns the max_suborder within this Folder. In a sense,
    /// this is the "size" of the folder's children, though due to A-Z sorting
    /// in the Gms2 IDE, order and size are not always directly related.
    ///
    /// Given the folder:
    /// ```no run
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

#[derive(Debug, Error, serde::Serialize, serde::Deserialize)]
pub enum FolderGraphError {
    #[error("path was not found")]
    PathNotFound,

    #[error("folder already existed at that location")]
    FolderAlreadyPresent,

    #[error("file already existed at that location")]
    FileAlreadyPresent,

    #[error("foldergraph is out of sync with internal Yyp -- yypboss is in undefined state")]
    InternalError,

    #[error("couldn't remove folder, given file")]
    BadRemove,
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

#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
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
            .ok_or(FolderGraphError::InternalError)?;

        yyp_resource.order = self.order;
        yyp_resource.id.path = self.child.path.clone();

        Ok(())
    }
}

#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
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
            .ok_or(FolderGraphError::InternalError)?;

        yyp_folder.order = self.order;
        yyp_folder.folder_path = self.child.view_path_location();

        Ok(())
    }
}
