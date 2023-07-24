use super::{Files, ResourceDescriptor, ResourceNames};
use crate::ViewPathLocationExt;
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use yy_typings::{FilesystemPath, ViewPath, ViewPathLocation};

#[derive(Debug, Clone, Eq, Serialize, Deserialize, Default)]
pub struct FolderGraph {
    pub name: String,
    pub path_to_parent: Option<ViewPathLocation>,
    pub folders: Vec<FolderGraph>,
    pub files: Files,
}

#[derive(Debug, Clone, Copy, Eq, Ord, PartialOrd, PartialEq, Serialize, Deserialize, Hash)]
pub enum Item {
    Folder,
    Resource,
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
            path_to_parent: None,
            files: Files::new(),
            folders: vec![],
        }
    }

    pub fn new(name: String, parent: ViewPathLocation) -> FolderGraph {
        FolderGraph {
            name,
            path_to_parent: Some(parent),
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
        let path = self.view_path_location();

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

    pub fn to_flat(&self, resource_names: &ResourceNames) -> FlatFolderGraph {
        let view_path = self.view_path();

        FlatFolderGraph {
            path_to_parent: self.path_to_parent.clone(),
            folders: self.folders.iter().map(|v| v.view_path()).collect(),
            files: self
                .files
                .inner()
                .iter()
                .filter_map(|v| {
                    resource_names
                        .get(&v.name)
                        .map(|rd| FlatResourceDescriptor {
                            filesystem_path: v.clone(),
                            resource_descriptor: rd.clone(),
                        })
                })
                .collect(),
            view_path,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlatFolderGraph {
    pub view_path: ViewPath,
    pub path_to_parent: Option<ViewPathLocation>,
    pub folders: Vec<ViewPath>,
    pub files: Vec<FlatResourceDescriptor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlatResourceDescriptor {
    pub filesystem_path: FilesystemPath,
    pub resource_descriptor: ResourceDescriptor,
}
