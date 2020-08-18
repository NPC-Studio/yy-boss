use super::{Files, FolderGraph, FolderGraphError, ResourceNames};
use crate::{PathStrExt, Resource, ViewPathLocationExt, YyResource};
use std::collections::HashSet;
use yy_typings::{ViewPath, ViewPathLocation, YypFolder, YypResource};

static ROOT_FOLDER_VIEW_PATH: once_cell::sync::Lazy<ViewPath> =
    once_cell::sync::Lazy::new(|| ViewPath {
        name: "folders".to_owned(),
        path: ViewPathLocation::root_folder(),
    });

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vfs {
    pub resource_names: ResourceNames,
    root: FolderGraph,
    root_resource: ViewPath,
    folders_to_reserialize: HashSet<ViewPathLocation>,
    folders_to_remove: HashSet<ViewPathLocation>,
}

impl Vfs {
    pub(crate) fn new(yyp_name: &str) -> Self {
        Vfs {
            root: FolderGraph::root(),
            root_resource: ViewPath {
                name: yyp_name.to_string(),
                path: ViewPathLocation::root_file(yyp_name),
            },
            resource_names: ResourceNames::new(),
            folders_to_reserialize: HashSet::new(),
            folders_to_remove: HashSet::new(),
        }
    }

    pub(crate) fn load_in_folders(&mut self, folders: &[YypFolder]) {
        for new_folder in folders.iter() {
            let mut folder_graph = &mut self.root;

            // ensure subfolders are loaded in...
            for section in new_folder.folder_path.component_paths() {
                let path_to_parent = folder_graph.view_path_location();
                let section = section.trim_yy().to_owned();

                // find or insert the new folder...
                if folder_graph.folders.iter().any(|f| f.name == section) == false {
                    folder_graph.folders.push(FolderGraph {
                        name: section.clone(),
                        path_to_parent: Some(path_to_parent),
                        // all of these are defaults..below we add in specs for each
                        order: 0,
                        tags: vec![],
                        folders: vec![],
                        files: Files::new(),
                    });
                }

                folder_graph = folder_graph
                    .folders
                    .iter_mut()
                    .find(|f| f.name == section)
                    .unwrap();
            }

            // get the folder and add in its order and what not...
            let f = Vfs::get_folder_mut(
                &mut self.root,
                &new_folder.folder_path,
                &Self::root_folder().path,
            )
            .unwrap();
            f.order = new_folder.order;
            f.tags = new_folder.tags.clone();
        }
    }

    pub(crate) fn load_in_file<T: YyResource>(
        &mut self,
        yy: &T,
        order: usize,
    ) -> Result<(), FolderGraphError> {
        // Add to the folder graph
        let folder = Vfs::get_folder_mut(
            &mut self.root,
            &yy.parent_view_path().path,
            &self.root_resource.path,
        )
        .ok_or_else(|| {
            FolderGraphError::PathNotFound(yy.parent_view_path().path.inner().to_string())
        })?;

        // add and sort
        folder.files.load_in(yy, order, &mut self.resource_names);

        Ok(())
    }

    pub(crate) fn get_folder_mut<'a>(
        root: &'a mut FolderGraph,
        view_path: &ViewPathLocation,
        root_view: &ViewPathLocation,
    ) -> Option<&'a mut FolderGraph> {
        if view_path == root_view {
            Some(root)
        } else {
            let mut folder = root;
            let mut used_root = true;

            for path in view_path.component_paths() {
                used_root = false;
                let path = path.trim_yy();
                folder = folder.folders.iter_mut().find(|f| f.name == path)?;
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
        if *view_path == self.root_resource.path {
            Some(&self.root)
        } else {
            let mut folder = &self.root;
            let mut used_root = true;

            for path in view_path.component_paths() {
                used_root = false;
                let path = path.trim_yy();
                folder = folder.folders.iter().find(|f| f.name == path)?;
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
            .ok_or_else(|| FolderGraphError::PathNotFound(name.to_string()))
    }

    /// The root path for a folder at the root of a project.
    ///
    /// Gms2 projects have historically had immutable folders at the top of a project's
    /// virtual file system, such as "Sprites", "Objects", etc, for each resource type.
    /// In Gms2.3, that restriction has been lifted, along with the internal changes to the
    /// Yyp, so it is now possible for any folder to be at the root of the project.
    pub fn root_folder() -> &'static ViewPath {
        &ROOT_FOLDER_VIEW_PATH
    }

    /// The root path for a file at the root of the project.
    ///
    /// Gms2 projects have historically had immutable folders at the top of a project's
    /// virtual file system, such as "Sprites", "Objects", etc, for each resource type.
    /// In Gms2.3, that restriction has been lifted, along with the internal changes to the
    /// Yyp, so it is now possible for a Resource to be at the root of a project.
    ///
    /// In that case, this function gives the path that resource will have. Note that this path
    /// is odd, and is not build into any other paths.
    pub fn root_resource(&self) -> &ViewPath {
        &self.root_resource
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
        let subfolder =
            Self::get_folder_mut(&mut self.root, &parent_path.path, &Self::root_folder().path)
                .ok_or_else(|| {
                    FolderGraphError::PathNotFound(parent_path.path.inner().to_string())
                })?;

        // Don't add a new folder with the same name...
        if subfolder.folders.iter().any(|f| f.name == name) {
            return Err(FolderGraphError::FolderAlreadyPresent);
        }

        let order = subfolder
            .folders
            .last()
            .map(|f| f.order + 1)
            .unwrap_or_default();

        // Create our Path...
        let path = parent_path.path.join(&name);
        subfolder.folders.push(FolderGraph::new(
            name.clone(),
            parent_path.path.clone(),
            vec![],
            order,
        ));

        // reserialize it
        self.folders_to_reserialize.insert(path.clone());
        // just in case...
        self.folders_to_remove.remove(&path);

        Ok(ViewPath { path, name })
    }

    /// Removes an empty folder from the virtual file system. If *anything* is within this folder, it will not be deleted,
    /// including other empty folders.
    ///
    /// ```
    ///
    ///
    /// ```
    pub fn remove_folder(
        &mut self,
        folder_path: &ViewPathLocation,
    ) -> Result<(), FolderGraphError> {
        let subfolder =
            Self::get_folder_mut(&mut self.root, &folder_path, &Self::root_folder().path)
                .ok_or_else(|| FolderGraphError::PathNotFound(folder_path.inner().to_string()))?;

        if subfolder.files.is_empty() == false || subfolder.folders.is_empty() == false {
            return Err(FolderGraphError::CannotRemoveFolder);
        }

        let name = subfolder.name.clone();
        if let Some(parent_path) = subfolder.path_to_parent.clone() {
            let parent =
                Self::get_folder_mut(&mut self.root, &parent_path, &Self::root_folder().path)
                    .ok_or_else(|| {
                        FolderGraphError::PathNotFound(folder_path.inner().to_string())
                    })?;

            let pos = parent.folders.iter().position(|v| v.name == name).unwrap();
            parent.folders.remove(pos);

            if self.folders_to_reserialize.contains(folder_path) {
                
            } else {
                self.folders_to_remove.insert(folder_path.clone());
            }

            Ok(())
        } else {
            Err(FolderGraphError::CannotRemoveRootFolder)
        }
    }

    /// Checks if a resource with a given name exists. If it does, it will return information
    /// on that resource in the form of the `CreatedResource` token, which can tell the user
    /// the type of resource.
    pub fn get_resource_type(&self, resource_name: &str) -> Option<Resource> {
        self.resource_names.get(resource_name).map(|v| v.resource)
    }

    /// Checks if a resource with a given name exists.
    pub fn resource_exists(&self, resource_name: &str) -> bool {
        self.get_resource_type(resource_name).is_some()
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

    pub(crate) fn new_resource_end<T: YyResource>(
        &mut self,
        yy: &T,
    ) -> Result<(), FolderGraphError> {
        let subfolder = Self::get_folder_mut(
            &mut self.root,
            &yy.parent_view_path().path,
            &self.root_resource.path,
        )
        .ok_or_else(|| {
            FolderGraphError::PathNotFound(yy.parent_view_path().path.inner().to_string())
        })?;

        let order = subfolder
            .folders
            .last()
            .map(|f| f.order + 1)
            .unwrap_or_default();

        subfolder.files.add(yy, order, &mut self.resource_names);
        Ok(())
    }

    pub(crate) fn remove_resource(&mut self, name: &str) {
        if let Some(desc) = self.resource_names.get(name) {
            if let Some(folder) = Self::get_folder_mut(
                &mut self.root,
                &desc.parent_location,
                &self.root_resource.path,
            ) {
                folder.files.remove(name, &mut self.resource_names);
            }
        }
    }

    pub(crate) fn serialize(
        &mut self,
        yyp_folders: &mut Vec<YypFolder>,
        yyp_resources: &mut Vec<YypResource>,
    ) {
        // folder graph...
        for reserialize in self.folders_to_reserialize.iter() {
            let folder_data = self
                .get_folder(&reserialize)
                .expect("always internally consistent");

            let output = YypFolder {
                folder_path: reserialize.clone(),
                order: folder_data.order,
                name: folder_data.name.clone(),
                tags: folder_data.tags.clone(),
                ..Default::default()
            };

            if let Some(pos) = yyp_folders
                .iter()
                .position(|v| v.folder_path == *reserialize)
            {
                yyp_folders[pos] = output;
            } else {
                yyp_folders.push(output);
            }
        }
        self.folders_to_reserialize.clear();

        for remove_path in self.folders_to_remove.drain() {
            if let Some(pos) = yyp_folders
                .iter()
                .position(|v| v.folder_path == remove_path)
            {
                yyp_folders.remove(pos);
            }
        }

        // resource names...
        self.resource_names.serialize(yyp_resources);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    #[test]
    fn folder_root() {
        let mut fgm = Vfs::new("project");
        let new_folder = fgm
            .new_folder_end(Vfs::root_folder(), "Sprites".to_owned())
            .unwrap();

        let root = FolderGraph {
            name: "folders".to_string(),
            order: 0,
            path_to_parent: None,
            files: Files::new(),
            folders: vec![FolderGraph {
                name: "Sprites".to_string(),
                order: 0,
                folders: vec![],
                files: Files::new(),
                path_to_parent: Some(ViewPathLocation("folders".to_string())),
                tags: vec![],
            }],
            tags: vec![],
        };

        let mut proof = Vfs {
            root,
            root_resource: ViewPath {
                name: "project".to_string(),
                path: ViewPathLocation::root_file("project"),
            },
            folders_to_reserialize: maplit::hashset! {
                ViewPathLocation("folders/Sprites.yy".to_string())
            },
            folders_to_remove: HashSet::new(),
            resource_names: ResourceNames::new(),
        };
        assert_eq!(fgm, proof);

        fgm.remove_folder(&new_folder.path).unwrap();
        proof.folders_to_reserialize = HashSet::new();
        proof.root.folders.clear();

        assert_eq!(fgm, proof);
    }

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
