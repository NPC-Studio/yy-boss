use super::Files;
use crate::ViewPathLocationExt;
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use yy_typings::{Tags, ViewPath, ViewPathLocation};

#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
pub struct FolderGraph {
    pub name: String,
    pub path_to_parent: Option<ViewPathLocation>,
    pub tags: Tags,
    pub order: usize,
    pub folders: Vec<FolderGraph>,
    pub files: Files,
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
            order: 0,
            path_to_parent: None,
            files: Files::new(),
            folders: vec![],
            tags: vec![],
        }
    }

    pub fn new(name: String, parent: ViewPathLocation, tags: Tags, order: usize) -> FolderGraph {
        FolderGraph {
            name,
            path_to_parent: Some(parent),
            tags,
            order,
            files: Files::new(),
            folders: vec![],
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
        if self.files.contains_name(name) {
            return Some(self);
        }

        for subfolder in self.folders.iter_mut() {
            if let Some(found) = subfolder.get_folder_by_fname_mut(name) {
                return Some(found);
            }
        }

        None
    }

    pub(super) fn get_folder_by_fname<'a>(&'a self, name: &str) -> Option<&'a FolderGraph> {
        if self.files.contains_name(name) {
            return Some(self);
        }

        for subfolder in self.folders.iter() {
            if let Some(found) = subfolder.get_folder_by_fname(name) {
                return Some(found);
            }
        }

        None
    }
}
