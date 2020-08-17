use super::{FileMember, FolderGraph, FolderGraphError, SubfolderMember};
use crate::{PathStrExt, ViewPathLocationExt};
use serde::{Deserialize, Serialize};
use yy_typings::{FilesystemPath, ViewPath, ViewPathLocation};

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
                folder = &mut folder
                    .folders
                    .iter_mut()
                    .find(|f| f.child.name == path)?
                    .child;
            }

            if used_root == false {
                Some(folder)
            } else {
                None
            }
        }
    }

    #[allow(dead_code)]
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
                folder = &folder.folders.iter().find(|f| f.child.name == path)?.child;
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
        if subfolder.folders.iter().any(|f| f.child.name == name) {
            return Err(FolderGraphError::FolderAlreadyPresent);
        }

        let order = subfolder
            .files
            .last()
            .map(|f| f.order + 1)
            .unwrap_or_default();

        // Create our Path...
        let path = parent_path.path.join(&name);
        subfolder.folders.push(SubfolderMember {
            child: FolderGraph::new(name.clone(), parent_path.path.clone()),
            order,
        });

        unimplemented!();

        // self.yyp.folders.push(YypFolder {
        //     folder_path: path.clone(),
        //     order,
        //     name: name.clone(),
        //     ..YypFolder::default()
        // });
        // self.dirty = true;

        Ok(ViewPath { path, name })
    }

    // / Adds a subfolder to the folder given at `parent_path` at given order. If a tree looks like:
    // /
    // /```txt
    // / Sprites/
    // /     - spr_player
    // /     - OtherSprites/
    // /     - spr_enemy
    // / ```
    // /
    // / and user adds a folder with name `Items` to the `Sprites` folder with an order of 1,
    // / then the output tree will be:
    // /
    // / ```txt
    // / Sprites/
    // /     - spr_player
    // /     - Items/
    // /     - OtherSprites/
    // /     - spr_enemy
    // /```
    // /
    // / `add_folder_with_order` returns a `Result<ViewPath>`, where `ViewPath` is of the newly created folder.
    // / This allows for easy sequential operations, such as adding a folder and then adding a file to that folder.
    // /
    // / **Nb:** when users have Gms2 in "Alphabetical" sort order, the `order` value here is largely ignored by the IDE.
    // / This can make for odd and unexpected results.
    // pub fn new_folder_order(
    //     &mut self,
    //     parent_path: ViewPath,
    //     name: String,
    //     order: usize,
    // ) -> Result<ViewPath, FolderGraphError> {
    //     let subfolder = self
    //         .folder_graph_manager
    //         .get_folder_mut(&parent_path.path)
    //         .ok_or(FolderGraphError::PathNotFound)?;

    //     if subfolder.folders.contains_key(&name) {
    //         return Err(FolderGraphError::FolderAlreadyPresent);
    //     }

    //     // Add the Subfolder View:
    //     subfolder.folders.insert(
    //         name.clone(),
    //         SubfolderMember {
    //             child: FolderGraph::new(name.clone(), parent_path.path.clone()),
    //             order,
    //         },
    //     );

    //     let path = parent_path.path.join(&name);

    //     self.yyp.folders.push(YypFolder {
    //         folder_path: path.clone(),
    //         order,
    //         name: name.clone(),
    //         ..YypFolder::default()
    //     });
    //     self.dirty = true;

    //     // Fix the other Orders:
    //     for (folder_name, folder) in subfolder.folders.iter_mut() {
    //         if folder.order <= order {
    //             folder.order += 1;

    //             if let Err(e) = folder.update_yyp(&mut self.yyp.folders) {
    //                 error!(
    //                 "We couldn't find {0} in the Yyp, even though we had {0} in the FolderGraph.\
    //                 This may become a hard error in the future. E: {1}",
    //                 folder_name, e
    //                 )
    //             }
    //         }
    //     }

    //     for (file_name, file) in subfolder.files.iter_mut() {
    //         if file.order <= order {
    //             file.order += 1;

    //             if let Err(e) = file.update_yyp(&mut self.yyp.resources) {
    //                 error!(
    //                 "We couldn't find {0} in the Yyp, even though we had {0} in the FolderGraph.\
    //                 This may become a hard error in the future. E: {1}",
    //                 file_name, e
    //                 )
    //             }
    //         }
    //     }

    //     Ok(ViewPath { path, name })
    // }

    pub(crate) fn new_resource_end(
        &mut self,
        view_path: &ViewPathLocation,
        child: FilesystemPath,
    ) -> Result<usize, FolderGraphError> {
        let subfolder = self
            .get_folder_mut(view_path)
            .ok_or(FolderGraphError::PathNotFound)?;

        let order = subfolder
            .files
            .last()
            .map(|f| f.order + 1)
            .unwrap_or_default();

        // add the resource
        subfolder.files.push(FileMember { child, order });

        Ok(order)
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
