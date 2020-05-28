use maplit::hashmap;
use std::collections::HashMap;
use std::hash::Hash;
use std::path::Path;
use yy_typings::{FilesystemPath, ViewPath};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderGraph {
    pub name: String,
    pub path_to_parent: Option<ViewPath>,
    pub members: HashMap<String, FolderMember>,
}

impl Hash for FolderGraph {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FolderMember {
    pub child: Child,
    pub order: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Child {
    SubFolder(FolderGraph),
    File(FilesystemPath),
}

impl FolderGraph {
    pub fn root() -> FolderGraph {
        FolderGraph {
            name: "folders".to_string(),
            path_to_parent: None,
            members: hashmap![],
        }
    }

    pub fn new(name: String, parent: ViewPath) -> FolderGraph {
        FolderGraph {
            name,
            path_to_parent: Some(parent),
            members: hashmap![],
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
}
