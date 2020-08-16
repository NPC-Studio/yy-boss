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

    pub(super) fn get_folder_by_fname_mut<'a>(
        &'a mut self,
        name: &str,
    ) -> Option<&'a mut FolderGraph> {
        if self.files.iter().any(|f| f.child.name == *name) {
            return Some(self);
        }

        for subfolder in self.folders.iter_mut() {
            if let Some(found) = subfolder.child.get_folder_by_fname_mut(name) {
                return Some(found);
            }
        }

        None
    }

    pub(super) fn get_folder_by_fname<'a>(&'a self, name: &str) -> Option<&'a FolderGraph> {
        if self.files.iter().any(|f| f.child.name == *name) {
            return Some(self);
        }

        for subfolder in self.folders.iter() {
            if let Some(found) = subfolder.child.get_folder_by_fname(name) {
                return Some(found);
            }
        }

        None
    }
}
