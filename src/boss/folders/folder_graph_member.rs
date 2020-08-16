use super::{FolderGraph, FolderGraphError};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use yy_typings::{FilesystemPath, YypFolder, YypResource};

pub trait FolderGraphMember: PartialOrd + Ord {
    type YypReference;

    /// Applies the State of the Folder Graph to the current YypResource which each
    /// folder graph member corresponds to. Essentially, this keeps the foldergraph and the yyp
    /// in sync.
    fn update_yyp(
        &self,
        yyp_resource: &mut Vec<Self::YypReference>,
    ) -> Result<(), FolderGraphError>;

    fn sort_by_name(lhs: &Self, rhs: &Self) -> Ordering;
}

#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
pub struct FileMember {
    pub child: FilesystemPath,
    pub order: usize,
}

impl PartialOrd for FileMember {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.order.partial_cmp(&other.order)
    }
}

impl Ord for FileMember {
    fn cmp(&self, other: &Self) -> Ordering {
        self.order.cmp(&other.order)
    }
}

impl PartialEq for FileMember {
    fn eq(&self, other: &Self) -> bool {
        self.child == other.child
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
    fn sort_by_name(lhs: &Self, rhs: &Self) -> Ordering {
        lhs.child.name.cmp(&rhs.child.name)
    }
}

#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
pub struct SubfolderMember {
    pub child: FolderGraph,
    pub order: usize,
}

impl PartialOrd for SubfolderMember {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.order.partial_cmp(&other.order)
    }
}

impl Ord for SubfolderMember {
    fn cmp(&self, other: &Self) -> Ordering {
        self.order.cmp(&other.order)
    }
}

impl PartialEq for SubfolderMember {
    fn eq(&self, other: &Self) -> bool {
        self.child == other.child
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
    fn sort_by_name(lhs: &Self, rhs: &Self) -> Ordering {
        lhs.child.name.cmp(&rhs.child.name)
    }
}
