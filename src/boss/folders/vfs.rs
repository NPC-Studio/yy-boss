use super::{utils::DirtyState, Files, FolderGraph, FolderGraphError, ResourceNames};
use crate::{PathStrExt, Resource, ViewPathLocationExt, YyResource};
use std::collections::HashMap;
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
    to_serialize: HashMap<ViewPathLocation, DirtyState>,
    to_remove: HashMap<ViewPathLocation, DirtyState>,
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
            to_serialize: HashMap::new(),
            to_remove: HashMap::new(),
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
    pub fn get_folder<'a>(
        root: &'a FolderGraph,
        view_path: &ViewPathLocation,
        root_view: &ViewPathLocation,
    ) -> Option<&'a FolderGraph> {
        if view_path == root_view {
            Some(root)
        } else {
            let mut folder = root;
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
    pub fn new_folder_end<S: AsRef<str>>(
        &mut self,
        parent_path: &ViewPath,
        name: S,
    ) -> Result<ViewPath, FolderGraphError> {
        let subfolder =
            Self::get_folder_mut(&mut self.root, &parent_path.path, &Self::root_folder().path)
                .ok_or_else(|| {
                    FolderGraphError::PathNotFound(parent_path.path.inner().to_string())
                })?;

        // Don't add a new folder with the same name...
        if subfolder.folders.iter().any(|f| f.name == name.as_ref()) {
            return Err(FolderGraphError::FolderAlreadyPresent);
        }

        let order = subfolder
            .folders
            .last()
            .map(|f| f.order + 1)
            .unwrap_or_default();

        // Create our Path...
        let path = parent_path.path.join(name.as_ref());
        subfolder.folders.push(FolderGraph::new(
            name.as_ref().to_owned(),
            parent_path.path.clone(),
            vec![],
            order,
        ));

        // reserialize it
        match self.to_remove.remove(&path) {
            Some(DirtyState::Lifetime) => {}
            Some(DirtyState::Edit) => {
                self.to_serialize.insert(path.clone(), DirtyState::Edit);
            }
            None => {
                self.to_serialize.insert(path.clone(), DirtyState::Lifetime);
            }
        };

        Ok(ViewPath {
            path,
            name: name.as_ref().to_owned(),
        })
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

            // add to our whatever...
            match self.to_serialize.remove(folder_path) {
                Some(DirtyState::Lifetime) => {}
                Some(DirtyState::Edit) => {
                    self.to_remove
                        .insert(folder_path.to_owned(), DirtyState::Edit);
                }
                None => {
                    self.to_remove
                        .insert(folder_path.to_owned(), DirtyState::Lifetime);
                }
            };

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
        // refry the beans...
        for (reserialize, state) in self.to_serialize.drain() {
            let folder_data =
                Self::get_folder(&self.root, &reserialize, &ROOT_FOLDER_VIEW_PATH.path)
                    .expect("always internally consistent");

            let output = YypFolder {
                folder_path: reserialize.clone(),
                order: folder_data.order,
                name: folder_data.name.clone(),
                tags: folder_data.tags.clone(),
                ..Default::default()
            };

            match state {
                DirtyState::Edit => {
                    let pos = yyp_folders
                        .iter()
                        .position(|v| v.folder_path == reserialize)
                        .expect("must exist for edits");

                    yyp_folders[pos] = output;
                }
                DirtyState::Lifetime => {
                    yyp_folders.push(output);
                }
            }
        }

        // remove the excess beans...
        for (remove_path, _) in self.to_remove.drain() {
            let pos = yyp_folders
                .iter()
                .position(|v| v.folder_path == remove_path)
                .expect("must exist to remove it");
            yyp_folders.remove(pos);
        }

        // resource names...
        self.resource_names.serialize(yyp_resources);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::collections::HashSet;
    #[test]
    fn folder_manipulations() {
        let mut fgm = Vfs::new("project");
        let new_folder = fgm.new_folder_end(Vfs::root_folder(), "Sprites").unwrap();

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
                path_to_parent: Some(ViewPathLocation::new("folders")),
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
            to_serialize: maplit::hashmap! {
                ViewPathLocation::new("folders/Sprites.yy") => DirtyState::Lifetime
            },
            to_remove: HashMap::new(),
            resource_names: ResourceNames::new(),
        };
        assert_eq!(fgm, proof);

        fgm.remove_folder(&new_folder.path).unwrap();
        proof.to_serialize = HashMap::new();
        proof.root.folders.clear();
        assert_eq!(fgm, proof);

        // bit of nesting...
        let new_folder = fgm.new_folder_end(&Vfs::root_folder(), "Sprites").unwrap();
        let subfolder = fgm.new_folder_end(&new_folder, "Npcs").unwrap();
        proof.to_serialize = maplit::hashmap! {
            ViewPathLocation::new("folders/Sprites.yy") => DirtyState::Lifetime,
            ViewPathLocation::new("folders/Sprites/Npcs.yy") => DirtyState::Lifetime,
        };
        proof.root.folders = vec![FolderGraph {
            name: "Sprites".to_string(),
            path_to_parent: Some(ViewPathLocation::new("folders")),
            tags: vec![],
            order: 0,
            folders: vec![FolderGraph {
                name: "Npcs".to_string(),
                path_to_parent: Some(ViewPathLocation::new("folders/Sprites.yy")),
                tags: vec![],
                order: 0,
                folders: vec![],
                files: Files::new(),
            }],
            files: Files::new(),
        }];
        assert_eq!(fgm, proof);

        // removal test...
        assert_eq!(
            fgm.remove_folder(&new_folder.path),
            Err(FolderGraphError::CannotRemoveFolder)
        );
        fgm.remove_folder(&subfolder.path).unwrap();
        fgm.remove_folder(&new_folder.path).unwrap();
        assert_eq!(fgm.to_remove, HashMap::new());
        assert_eq!(fgm.to_serialize, HashMap::new());

        // add and then check removal...
        let new_folder = fgm.new_folder_end(&Vfs::root_folder(), "Sprites").unwrap();
        let subfolder = fgm.new_folder_end(&new_folder, "Npcs").unwrap();

        let mut dummy0 = vec![];
        let mut dummy1 = vec![];
        fgm.serialize(&mut dummy0, &mut dummy1);

        assert_eq!(
            dummy0.into_iter().collect::<HashSet<_>>(),
            maplit::hashset![
                YypFolder {
                    folder_path: ViewPathLocation::new("folders/Sprites.yy"),
                    order: 0,
                    name: "Sprites".to_string(),
                    ..Default::default()
                },
                YypFolder {
                    folder_path: ViewPathLocation::new("folders/Sprites/Npcs.yy"),
                    order: 0,
                    name: "Npcs".to_string(),
                    ..Default::default()
                }
            ]
        );
        assert_eq!(dummy1, vec![]);

        fgm.remove_folder(&subfolder.path).unwrap();
        fgm.remove_folder(&new_folder.path).unwrap();
        assert_eq!(
            fgm.to_remove,
            maplit::hashmap! {
                ViewPathLocation::new("folders/Sprites.yy") => DirtyState::Lifetime,
                ViewPathLocation::new("folders/Sprites/Npcs.yy") => DirtyState::Lifetime,
            }
        );
        assert_eq!(fgm.to_serialize, HashMap::new());
    }
}
