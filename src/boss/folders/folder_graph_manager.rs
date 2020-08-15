use super::{FolderGraph, FolderGraphError, SubfolderMember};
use crate::{PathStrExt, ViewPathLocationExt};
use serde::{Deserialize, Serialize};
use yy_typings::{ViewPath, ViewPathLocation};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FolderGraphManager {
    pub(crate) root: FolderGraph,
    root_file_location: ViewPathLocation,
}

impl FolderGraphManager {
    pub(crate) fn new(yyp_name: &str) -> Self {
        FolderGraphManager {
            root: FolderGraph::root(),
            root_file_location: ViewPathLocation::root_file(yyp_name),
        }
    }

    pub(crate) fn get_folder_mut(
        &mut self,
        view_path: &ViewPathLocation,
    ) -> Option<&mut FolderGraph> {
        if *view_path == self.root_file_location {
            Some(&mut self.root)
        } else {
            let mut folder = &mut self.root;
            let mut used_root = true;

            for path in view_path.component_paths() {
                used_root = false;
                let path = path.trim_yy();
                folder = &mut folder.folders.get_mut(path)?.child;
            }

            if used_root == false {
                Some(folder)
            } else {
                None
            }
        }
    }

    pub(crate) fn get_folder_by_fname_mut(&mut self, name: &str) -> Option<&mut FolderGraph> {
        self.root.get_folder_by_fname_mut(name)
    }

    /// Gets a folder by the given ViewPathLocation.
    /// If a folder does not exist, or if the path points to a file, None will be returned.
    pub fn get_folder(&self, view_path: &ViewPathLocation) -> Option<&FolderGraph> {
        if *view_path == self.root_file_location {
            Some(&self.root)
        } else {
            let mut folder = &self.root;
            let mut used_root = true;

            for path in view_path.component_paths() {
                used_root = false;
                let path = path.trim_yy();
                folder = &folder.folders.get(path)?.child;
            }

            if used_root == false {
                Some(folder)
            } else {
                None
            }
        }
    }

    /// Gets the root folder.
    pub fn get_root_folder(&self) -> &FolderGraph {
        &self.root
    }
    /// Finds the containing folder for a given file. Returns an error is no file of that name
    /// could be found.
    #[allow(dead_code)]
    pub fn get_folder_by_fname(&self, name: &str) -> Result<&FolderGraph, FolderGraphError> {
        self.root
            .get_folder_by_fname(name)
            .ok_or(FolderGraphError::PathNotFound)
    }

    /// Adds a subfolder to the folder given at `parent_path` with the order set to the end. If a tree looks like:
    ///
    ///```txt
    /// Sprites/
    ///     - spr_player
    ///     - spr_enemy
    /// ```
    ///
    /// and user adds a folder with name `Items` to the `Sprites` folder, then the output tree will be:
    ///
    /// ```txt
    /// Sprites/
    ///     - spr_player
    ///     - spr_enemy
    ///     - Items/
    ///```
    ///
    /// `add_folder_to_end` returns a `Result<ViewPath>`, where `ViewPath` is of the newly created folder.
    /// This allows for easy sequential operations, such as adding a folder and then adding a file to that folder.
    pub fn new_folder_end(
        &mut self,
        parent_path: &ViewPath,
        name: String,
    ) -> Result<ViewPath, FolderGraphError> {
        let subfolder = self
            .get_folder_mut(&parent_path.path)
            .ok_or(FolderGraphError::PathNotFound)?;

        // Don't add a new folder with the same name...
        if subfolder.folders.contains_key(&name) {
            return Err(FolderGraphError::FolderAlreadyPresent);
        }

        // Sometimes Gms2 uses 1 for the default order of folders. This is chaos.
        // No clue what's up with that.
        let order = subfolder.max_suborder().map(|v| v + 1).unwrap_or_default();

        // Create our Path...
        let path = parent_path.path.join(&name);
        subfolder.folders.insert(
            name.clone(),
            SubfolderMember {
                child: FolderGraph::new(name.clone(), parent_path.path.clone()),
                order,
            },
        );

        // self.yyp.folders.push(YypFolder {
        //     folder_path: path.clone(),
        //     order,
        //     name: name.clone(),
        //     ..YypFolder::default()
        // });
        // self.dirty = true;

        Ok(ViewPath { path, name })
    }
}

#[cfg(test)]
mod test {
    // #[test]
    // fn folder_add_root() {
    //     let mut basic_yyp_boss = common::setup_blank_project().unwrap();
    //     let proof = common::load_proof("folder_add_root").unwrap();

    //     common::assert_yypboss_neq(&basic_yyp_boss, &proof);

    //     basic_yyp_boss
    //         .new_folder_end(&YypBoss::root_folder(), "Test At Root".to_string())
    //         .unwrap();

    //     common::assert_yypboss_eq(&basic_yyp_boss, &proof);
    // }

    // #[test]
    // fn folder_add_nonroot() {
    //     let mut basic_yyp_boss = common::setup_blank_project().unwrap();
    //     let proof = common::load_proof("folder_add_nonroot").unwrap();

    //     common::assert_yypboss_neq(&basic_yyp_boss, &proof);

    //     let parent_folder = basic_yyp_boss
    //         .new_folder_end(&YypBoss::root_folder(), "First Folder".to_string())
    //         .unwrap();

    //     basic_yyp_boss
    //         .new_folder_end(&parent_folder, "Subfolder".to_string())
    //         .unwrap();

    //     common::assert_yypboss_eq(&basic_yyp_boss, &proof);
    // }

    // #[test]
    // fn delete_folder_recursively() {}
}
