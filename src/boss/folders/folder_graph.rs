use super::{FileMember, SubfolderMember};
use crate::ViewPathLocationExt;
use maplit::btreemap;
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use yy_typings::{ViewPath, ViewPathLocation};

#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
pub struct FolderGraph {
    pub name: String,
    pub path_to_parent: Option<ViewPathLocation>,
    pub files: Vec<FileMember>,
    pub folders: Vec<SubfolderMember>,
}

impl Default for FolderGraph {
    fn default() -> Self {
        FolderGraph {
            name: String::new(),
            path_to_parent: None,
            files: vec![],
            folders: vec![],
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
    pub(super) fn root() -> FolderGraph {
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

        if let Some(file_max) = self.files.iter().max() {
            if file_max >= output.unwrap_or_default() {
                output = Some(file_max);
            }
        }

        if let Some(file_max) = self.folders.iter().max() {
            if file_max >= output.unwrap_or_default() {
                output = Some(file_max);
            }
        }

        output
    }

    pub(super) fn get_folder_by_fname_mut<'a>(
        &'a mut self,
        name: &str,
    ) -> Option<&'a mut FolderGraph> {
        if self.files.contains_key(name) {
            return Some(self);
        }

        for subfolder in self.folders.values_mut() {
            if let Some(found) = subfolder.child.get_folder_by_fname_mut(name) {
                return Some(found);
            }
        }

        None
    }

    pub(super) fn get_folder_by_fname<'a>(&'a self, name: &str) -> Option<&'a FolderGraph> {
        if self.files.contains_key(name) {
            return Some(self);
        }

        for subfolder in self.folders.values() {
            if let Some(found) = subfolder.child.get_folder_by_fname(name) {
                return Some(found);
            }
        }

        None
    }
}
