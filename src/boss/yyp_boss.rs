use super::{
    directory_manager::DirectoryManager,
    folder_graph::*,
    pipelines::PipelineManager,
    resources::{CreatedResource, RemovedResource},
    utils, FolderGraph, PathStrExt, ViewPathLocationExt, YyResource, YyResourceHandler,
    YypSerialization,
};
use crate::Resource;
use anyhow::{format_err, Context, Result as AnyResult};
use log::*;
use std::{collections::HashMap, fs, path::Path};
use yy_typings::{script::Script, sprite_yy::*, utils::TrailingCommaUtility, Yyp};

#[derive(Debug)]
pub struct YypBoss {
    pub directory_manager: DirectoryManager,
    pub pipeline_manager: PipelineManager,
    pub sprites: YyResourceHandler<Sprite>,
    pub scripts: YyResourceHandler<Script>,
    yyp: Yyp,
    folder_graph: FolderGraph,
    resource_names: HashMap<String, Resource>,
    tcu: TrailingCommaUtility,
    dirty: bool,
}

impl YypBoss {
    /// Creates a new YyBoss Manager and performs startup file reading.
    pub fn new(path_to_yyp: &Path) -> AnyResult<YypBoss> {
        let tcu = TrailingCommaUtility::new();
        let yyp = utils::deserialize(path_to_yyp, Some(&tcu)).with_context(|| "on the yyp")?;

        let directory_manager = DirectoryManager::new(path_to_yyp)?;

        let mut yyp_boss = Self {
            yyp,
            dirty: false,
            folder_graph: FolderGraph::root(),
            resource_names: HashMap::new(),
            tcu,
            sprites: YyResourceHandler::new(),
            scripts: YyResourceHandler::new(),
            pipeline_manager: PipelineManager::new(&directory_manager)?,
            directory_manager: DirectoryManager::new(path_to_yyp)?,
        };

        // Load in Folders
        for new_folder in yyp_boss.yyp.folders.iter() {
            let mut folder_graph = &mut yyp_boss.folder_graph;

            for section in new_folder.folder_path.component_paths() {
                let parent_path = folder_graph.view_path();
                let section = section.trim_yy().to_owned();
                let entry = folder_graph.folders.entry(section.clone());

                let new_member = entry.or_insert(SubfolderMember {
                    child: FolderGraph::new(section, parent_path),
                    order: new_folder.order,
                });

                folder_graph = &mut new_member.child;
            }
        }

        fn load_in_resource<T: YyResource>(
            resource: &mut YyResourceHandler<T>,
            folder_graph: &mut FolderGraph,
            resource_names: &mut HashMap<String, Resource>,
            yyp_resources: &[YypResource],
            directory_manager: &DirectoryManager,
            tcu: &TrailingCommaUtility,
        ) -> AnyResult<()> {
            for yyp_resource in yyp_resources
                .iter()
                .filter(|value| value.id.path.starts_with(T::SUBPATH_NAME))
            {
                let yy_file_path = directory_manager
                    .root_directory()
                    .join(&yyp_resource.id.path);

                let yy_file: T = utils::deserialize(&yy_file_path, Some(&tcu))
                    .with_context(|| format!("on resource {:?}", yy_file_path))?;

                // Add to the folder graph
                folder_graph
                    .find_subfolder_mut(&yy_file.parent_path())?
                    .files
                    .insert(
                        yy_file.name().to_owned(),
                        FileMember {
                            child: FilesystemPath::new(T::SUBPATH_NAME, &yy_file.name()),
                            order: yyp_resource.order,
                        },
                    );

                resource_names.insert(yy_file.name().to_string(), T::RESOURCE);
                resource.load_on_startup(yy_file);
            }

            Ok(())
        }

        // Load in our Resources
        load_in_resource(
            &mut yyp_boss.sprites,
            &mut yyp_boss.folder_graph,
            &mut yyp_boss.resource_names,
            &yyp_boss.yyp.resources,
            &yyp_boss.directory_manager,
            &yyp_boss.tcu,
        )?;

        load_in_resource(
            &mut yyp_boss.scripts,
            &mut yyp_boss.folder_graph,
            &mut yyp_boss.resource_names,
            &yyp_boss.yyp.resources,
            &yyp_boss.directory_manager,
            &yyp_boss.tcu,
        )?;

        // Ensure the directory
        // Self::ensure_yyboss_data(&yyp_boss.directory_manager)?;

        Ok(yyp_boss)
    }

    /// Gets the default texture path, if it exists. The "Default" group simply
    /// has the name `"Default"`.
    pub fn default_texture_path(&self) -> Option<TexturePath> {
        self.yyp
            .texture_groups
            .iter()
            .find(|tex| tex.name == "Default")
            .map(|texture_group| texture_group.into())
    }

    // /// Creates a texture group with the given name and rename preference.
    // /// If it does exist, it returns an error.
    // pub fn create_texture_group(&self) -> Option<TexturePath> {
    //     self.yyp
    //         .texture_groups
    //         .iter()
    //         .find(|tex| tex.name == "Default")
    //         .map(|texture_group| texture_group.into())
    // }

    /// Returns a list of resource names already being used by the system.
    ///
    /// In a project
    /// with a sprite `spr_player` and an object `obj_player`, this HashSet would contain
    /// `"spr_player"` and `"obj_player"`.
    pub fn current_resource_names(&self) -> Vec<(String, Resource)> {
        self.resource_names.clone().into_iter().collect()
    }

    /// Adds a subfolder to the folder given at `parent_path` at the final order. If a tree looks like:
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
        let subfolder = self.folder_graph.find_subfolder_mut(parent_path)?;

        // Sometimes Gms2 uses 1 for the default order of folders. This is chaos.
        // No clue what's up with that.
        let order = subfolder.max_suborder().map(|v| v + 1).unwrap_or_default();

        if subfolder.folders.contains_key(&name) {
            return Err(FolderGraphError::FolderAlreadyPresent);
        }

        // Create our Path...
        let path = parent_path.path.join(&name);

        subfolder.folders.insert(
            name.clone(),
            SubfolderMember {
                child: FolderGraph::new(name.clone(), parent_path.clone()),
                order,
            },
        );

        self.yyp.folders.push(YypFolder {
            folder_path: path.clone(),
            order,
            name: name.clone(),
            ..YypFolder::default()
        });
        self.dirty = true;

        Ok(ViewPath { path, name })
    }

    /// Adds a subfolder to the folder given at `parent_path` at given order. If a tree looks like:
    ///
    ///```txt
    /// Sprites/
    ///     - spr_player
    ///     - OtherSprites/
    ///     - spr_enemy
    /// ```
    ///
    /// and user adds a folder with name `Items` to the `Sprites` folder with an order of 1,
    /// then the output tree will be:
    ///
    /// ```txt
    /// Sprites/
    ///     - spr_player
    ///     - Items/
    ///     - OtherSprites/
    ///     - spr_enemy
    ///```
    ///
    /// `add_folder_with_order` returns a `Result<ViewPath>`, where `ViewPath` is of the newly created folder.
    /// This allows for easy sequential operations, such as adding a folder and then adding a file to that folder.
    ///
    /// **Nb:** when users have Gms2 in "Alphabetical" sort order, the `order` value here is largely ignored by the IDE.
    /// This can make for odd and unexpected results.
    pub fn new_folder_order(
        &mut self,
        parent_path: ViewPath,
        name: String,
        order: usize,
    ) -> Result<ViewPath, FolderGraphError> {
        let subfolder = self.folder_graph.find_subfolder_mut(&parent_path)?;

        if subfolder.folders.contains_key(&name) {
            return Err(FolderGraphError::FolderAlreadyPresent);
        }

        // Add the Subfolder View:
        subfolder.folders.insert(
            name.clone(),
            SubfolderMember {
                child: FolderGraph::new(name.clone(), parent_path.clone()),
                order,
            },
        );

        let path = parent_path.path.join(&name);

        self.yyp.folders.push(YypFolder {
            folder_path: path.clone(),
            order,
            name: name.clone(),
            ..YypFolder::default()
        });
        self.dirty = true;

        // Fix the other Orders:
        for (folder_name, folder) in subfolder.folders.iter_mut() {
            if folder.order <= order {
                folder.order += 1;

                if let Err(e) = folder.update_yyp(&mut self.yyp.folders) {
                    error!(
                    "We couldn't find {0} in the Yyp, even though we had {0} in the FolderGraph.\
                    This may become a hard error in the future. E: {1}",
                    folder_name, e
                    )
                }
            }
        }

        for (file_name, file) in subfolder.files.iter_mut() {
            if file.order <= order {
                file.order += 1;

                if let Err(e) = file.update_yyp(&mut self.yyp.resources) {
                    error!(
                    "We couldn't find {0} in the Yyp, even though we had {0} in the FolderGraph.\
                    This may become a hard error in the future. E: {1}",
                    file_name, e
                    )
                }
            }
        }

        Ok(ViewPath { path, name })
    }

    /// Adds a file to the folder given at `parent_path` and with the final order. If a tree looks like:
    ///
    ///```no run
    /// Sprites/
    ///     - spr_player
    ///     - spr_enemy
    /// ```
    ///
    /// and user adds a file with name `spr_item` to the `Sprites` folder, then the output tree will be:
    ///
    /// ```txt
    /// Sprites/
    ///     - spr_player
    ///     - spr_enemy
    ///     - spr_item
    ///```
    ///
    /// This function returns a `FilledResourceToken`, which is a required parameter for then assigning the Sprite
    /// to the Token.
    pub fn new_resource_end(
        &mut self,
        parent_path: ViewPath,
        resource_name: &str,
        resource_kind: Resource,
    ) -> Result<CreatedResource, FolderGraphError> {
        let subfolder = self.folder_graph.find_subfolder_mut(&parent_path)?;
        let order = subfolder.max_suborder().map(|v| v + 1).unwrap_or_default();
        if subfolder.files.contains_key(resource_name) {
            return Err(FolderGraphError::FileAlreadyPresent);
        }

        let child = FilesystemPath::new(resource_kind.base_name(), resource_name);
        subfolder.files.insert(
            resource_name.to_owned(),
            FileMember {
                child: child.clone(),
                order,
            },
        );

        // add the resource
        self.add_new_yyp_resource(child, order, resource_kind);

        Ok(CreatedResource(resource_kind))
    }

    // /// Adds a file to the folder given at `parent_path` at the given order. If a tree looks like:
    // ///
    // ///```txt
    // /// Sprites/
    // ///     - spr_player
    // ///     - spr_enemy
    // /// ```
    // ///
    // /// and user adds a file with name `spr_item` to the `Sprites` folder at order 1, then the output tree will be:
    // ///
    // /// ```txt
    // /// Sprites/
    // ///     - spr_player
    // ///     - spr_item
    // ///     - spr_enemy
    // ///```
    // ///
    // /// Additionally, `spr_enemy`'s order will be updated to be `2`.
    // pub fn new_resource_order(
    //     &mut self,
    //     parent_path: ViewPath,
    //     child: FilesystemPath,
    //     order: usize,
    // ) -> Result<(), FolderGraphError> {
    //     let subfolder = self.folder_graph.find_subfolder_mut(&parent_path)?;
    //     if subfolder.files.contains_key(&child.name) {
    //         return Err(FolderGraphError::FileAlreadyPresent);
    //     }

    //     subfolder
    //         .files
    //         .insert(child.name.clone(), FileMember { child, order });

    //     // Fix the Files
    //     for (file_name, file) in subfolder.files.iter_mut() {
    //         if file.order >= order {
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

    //     // Fix the Folders
    //     for (folder_name, folder) in subfolder.folders.iter_mut() {
    //         if folder.order >= order {
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

    //     Ok(())
    // }

    pub fn remove_resource(
        &mut self,
        resource_name: &str,
        resource_kind: Resource,
    ) -> Result<RemovedResource, FolderGraphError> {
        let fp = FilesystemPath::new(resource_kind.base_name(), resource_name);
        self.remove_yyp_resource(&fp);

        if let Some(subfolder) = self.folder_graph.find_subfolder_by_file(resource_name) {
            subfolder.files.remove(resource_name);
        } else {
            return Err(FolderGraphError::FolderGraphOutofSyncWithYyp);
        }

        Ok(RemovedResource(resource_kind))
    }

    /// Checks if a resource with a given name exists. If it does, it will return information
    /// on that resource in the form of the `CreatedResource` token, which can tell the user
    /// the type of resource.
    pub fn get_resource(&self, resource_name: &str) -> Option<CreatedResource> {
        self.resource_names
            .get(resource_name)
            .map(|resource_kind| CreatedResource(*resource_kind))
    }

    /// Adds a new Resource to be tracked by the Yyp.
    fn add_new_yyp_resource(&mut self, id: FilesystemPath, order: usize, resource: Resource) {
        self.resource_names.insert(id.name.clone(), resource);
        let new_yyp_resource = YypResource { id, order };

        // Update the Resource
        self.yyp.resources.push(new_yyp_resource);
        self.dirty = true;
    }

    /// Removes the resource. Does not error if the resource was not found!
    fn remove_yyp_resource(&mut self, resource: &FilesystemPath) {
        self.resource_names.remove(&resource.name);
        if let Some(pos) = self.yyp.resources.iter().position(|p| &p.id == resource) {
            self.yyp.resources.remove(pos);
        }

        self.dirty = true;
    }

    /// Serializes the YypBoss data to disk at the path of the Yyp.
    pub fn serialize(&mut self) -> AnyResult<()> {
        self.sprites.serialize(&self.directory_manager)?;

        // serialize the pipeline manifests
        self.pipeline_manager
            .serialize(&self.directory_manager)
            .context("serializing pipelines")?;

        // Serialize Ourselves:
        if self.dirty {
            let string = self.yyp.yyp_serialization(0);
            fs::write(&self.directory_manager.yyp(), &string)?;

            self.dirty = false;
        }

        Ok(())
    }
}

impl YypBoss {
    pub fn root_path(&self) -> ViewPath {
        ViewPath {
            name: "folders".to_string(),
            path: ViewPathLocation("folders".to_string()),
        }
    }

    /// Shows the underlying Yyp. This is exposed mostly
    /// for integration tests.
    pub fn yyp(&self) -> &Yyp {
        &self.yyp
    }

    /// Gives a reference to the current FolderGraph.
    pub fn root_folder(&self) -> &FolderGraph {
        &self.folder_graph
    }

    /// This could be a very hefty allocation!
    pub fn folder(&self, view_path: &ViewPath) -> Option<FolderGraph> {
        if view_path.name != self.folder_graph.name {
            let mut folder = &self.folder_graph;

            for path in view_path.path.component_paths() {
                folder = &folder
                    .folders
                    .get(path)
                    .ok_or_else(|| format_err!("Couldn't find subfolder {}", path))
                    .ok()?
                    .child;
            }
            Some(folder.clone())
        } else {
            Some(self.folder_graph.clone())
        }
    }
}

impl Into<Yyp> for YypBoss {
    fn into(self) -> Yyp {
        self.yyp
    }
}

impl PartialEq for YypBoss {
    fn eq(&self, other: &Self) -> bool {
        self.yyp == other.yyp
            && self.folder_graph == other.folder_graph
            && self.resource_names == other.resource_names
    }
}
