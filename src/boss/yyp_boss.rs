use super::{
    directory_manager::DirectoryManager, errors::*, folders::*, pipelines::PipelineManager, utils,
    PathStrExt, Resource, ResourceDescriptor, ResourceNames, ViewPathLocationExt, YyResource,
    YyResourceData, YyResourceHandler, YypSerialization,
};
use anyhow::{Context, Result as AnyResult};
use object_yy::Object;
use std::{fs, path::Path};
use yy_typings::{script::Script, sprite_yy::*, utils::TrailingCommaUtility, Yyp};

#[derive(Debug)]
pub struct YypBoss {
    pub directory_manager: DirectoryManager,
    pub pipeline_manager: PipelineManager,
    pub sprites: YyResourceHandler<Sprite>,
    pub scripts: YyResourceHandler<Script>,
    pub objects: YyResourceHandler<Object>,
    pub folder_graph_manager: FolderGraphManager,
    pub resource_names: ResourceNames,
    pub tcu: TrailingCommaUtility,
    yyp: Yyp,
}

impl YypBoss {
    /// Creates a new YyBoss Manager and performs startup file reading.
    pub fn new(path_to_yyp: &Path) -> Result<YypBoss, StartupError> {
        let tcu = TrailingCommaUtility::new();
        let yyp: Yyp = utils::deserialize_json_tc(path_to_yyp, &tcu)?;

        let directory_manager = DirectoryManager::new(path_to_yyp)?;

        let mut yyp_boss = Self {
            folder_graph_manager: FolderGraphManager::new(&yyp.name),
            resource_names: ResourceNames::new(),
            tcu,
            sprites: YyResourceHandler::new(),
            scripts: YyResourceHandler::new(),
            objects: YyResourceHandler::new(),
            pipeline_manager: PipelineManager::new(&directory_manager)?,
            directory_manager,
            yyp,
        };

        // Load in Folders
        for new_folder in yyp_boss.yyp.folders.iter() {
            let mut folder_graph = &mut yyp_boss.folder_graph_manager.root;

            for section in new_folder.folder_path.component_paths() {
                let parent_path = folder_graph.view_path_location();
                let section = section.trim_yy().to_owned();

                // find or insert the new folder...
                if folder_graph.folders.iter().any(|f| f.child.name == section) == false {
                    folder_graph.folders.push(SubfolderMember {
                        child: FolderGraph::new(section.clone(), parent_path),
                        order: new_folder.order,
                    });

                    folder_graph
                        .folders
                        .sort_unstable_by(SubfolderMember::sort_by_name);
                }

                folder_graph = &mut folder_graph
                    .folders
                    .iter_mut()
                    .find(|f| f.child.name == section)
                    .unwrap()
                    .child;
            }
        }

        fn load_in_resource<T: YyResource>(
            resource: &mut YyResourceHandler<T>,
            folder_graph: &mut FolderGraphManager,
            resource_names: &mut ResourceNames,
            yyp_resources: &[YypResource],
            directory_manager: &DirectoryManager,
            tcu: &TrailingCommaUtility,
        ) -> Result<(), StartupError> {
            for yyp_resource in yyp_resources
                .iter()
                .filter(|value| value.id.path.starts_with(T::SUBPATH_NAME))
            {
                let yy_file_path = directory_manager
                    .root_directory()
                    .join(&yyp_resource.id.path);

                let yy_file: T = utils::deserialize_json_tc(&yy_file_path, &tcu)?;

                // Add to the folder graph
                let folder = folder_graph
                    .get_folder_mut(&yy_file.parent_view_path().path)
                    .ok_or(FolderGraphError::PathNotFound)?;

                // add and sort
                folder.files.push(FileMember {
                    child: FilesystemPath::new(T::SUBPATH_NAME, &yy_file.name()),
                    order: yyp_resource.order,
                });
                folder.files.sort_unstable_by(FileMember::sort_by_name);

                // add to resource names...
                resource_names.insert(
                    yy_file.name().to_string(),
                    ResourceDescriptor::new(T::RESOURCE, yyp_resource.order),
                );
                resource.load_on_startup(yy_file);
            }

            Ok(())
        }

        // Load in our Resources
        // @update_resource
        load_in_resource(
            &mut yyp_boss.sprites,
            &mut yyp_boss.folder_graph_manager,
            &mut yyp_boss.resource_names,
            &yyp_boss.yyp.resources,
            &yyp_boss.directory_manager,
            &yyp_boss.tcu,
        )?;
        load_in_resource(
            &mut yyp_boss.scripts,
            &mut yyp_boss.folder_graph_manager,
            &mut yyp_boss.resource_names,
            &yyp_boss.yyp.resources,
            &yyp_boss.directory_manager,
            &yyp_boss.tcu,
        )?;
        load_in_resource(
            &mut yyp_boss.objects,
            &mut yyp_boss.folder_graph_manager,
            &mut yyp_boss.resource_names,
            &yyp_boss.yyp.resources,
            &yyp_boss.directory_manager,
            &yyp_boss.tcu,
        )?;

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

    /// Adds a new resource, which must not already exist within the project.
    pub fn add_resource<T: YyResource>(
        &mut self,
        yy_file: T,
        associated_data: T::AssociatedData,
    ) -> Result<(), ResourceManipulationError> {
        if let Some(r) = self.resource_names.get(yy_file.name()) {
            return Err(ResourceManipulationError::BadResourceName(r.resource));
        }

        let child = FilesystemPath::new(T::SUBPATH_NAME, yy_file.name());
        let order = self
            .folder_graph_manager
            .new_resource_end(&yy_file.parent_view_path().path, child)?;

        self.resource_names.insert(
            yy_file.name().to_owned(),
            ResourceDescriptor::new(T::RESOURCE, order),
        );
        let handler = T::get_handler_mut(self);

        if handler.set(yy_file, associated_data).is_some() {
            Err(ResourceManipulationError::InternalError)
        } else {
            Ok(())
        }
    }

    /// Adds a new resource, which must not already exist within the project.
    pub fn remove_resource<T: YyResource>(
        &mut self,
        name: &str,
    ) -> Result<(T, Option<T::AssociatedData>), ResourceManipulationError> {
        // confirm the resource exists...
        if let Some(v) = self.resource_names.get(name) {
            if v.resource != T::RESOURCE {
                return Err(ResourceManipulationError::BadResourceName(v.resource));
            }
        } else {
            return Err(ResourceManipulationError::NoResourceByThatName);
        }

        // remove the file from the VFS...
        // self.folder_graph_manager.remove_resource(name)?;

        // remove from our name tracking
        self.resource_names.remove(name);

        let handler = T::get_handler_mut(self);
        let tcu = TrailingCommaUtility::new();
        handler
            .remove(name, &tcu)
            .ok_or_else(|| ResourceManipulationError::InternalError)
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

    /// Gets a resource via the type. Users should probably not use this method unless they're doing
    /// some generic code. Instead, simply use each resources manager as appropriate -- for example,
    /// to get an object's data, use `yyp_boss.objects.get`.
    ///
    /// *Nb*: `YyResourceData` might not have any AssociatedData on it. See its warning on how Associated
    /// Data is held lazily.
    pub fn get_resource<T: YyResource>(&self, name: &str) -> Option<&YyResourceData<T>> {
        let handler = T::get_handler(self);
        handler.get(name)
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

    /// Serializes the YypBoss data to disk at the path of the Yyp.
    pub fn serialize(&mut self) -> AnyResult<()> {
        let mut dirty = false;

        // serialize the resource names...
        dirty |= self.resource_names.serialize(&mut self.yyp)?;

        // serialize the folder graph...
        dirty |= self.folder_graph_manager.serialize(&mut self.yyp)?;

        // serialize all the whatever
        // @update_resource
        self.sprites.serialize(&self.directory_manager)?;
        self.objects.serialize(&self.directory_manager)?;
        self.scripts.serialize(&self.directory_manager)?;

        // serialize the pipeline manifests
        self.pipeline_manager
            .serialize(&self.directory_manager)
            .context("serializing pipelines")?;

        // Serialize Ourselves:
        if dirty {
            let string = self.yyp.yyp_serialization(0);
            fs::write(&self.directory_manager.yyp(), &string)?;
        }

        Ok(())
    }
}

impl YypBoss {
    /// The root path for a folder at the root of a project.
    ///
    /// Gms2 projects have historically had immutable folders at the top of a project's
    /// virtual file system, such as "Sprites", "Objects", etc, for each resource type.
    /// In Gms2.3, that restriction has been lifted, along with the internal changes to the
    /// Yyp, so it is now possible for any folder to be at the root of the project.
    ///
    /// This function returns the "root" folder of the project -- it does not actually exist in
    /// the project in any way, but you can use it to build top level folders.
    ///
    /// ```no_run
    /// # use yy_boss::YypBoss;
    /// # let mut basic_yyp_boss = YypBoss::new(std::path::Path::new("")).unwrap();
    /// basic_yyp_boss
    ///    .new_folder_end(&YypBoss::root_folder(), "New Folder at Root".to_string())
    ///    .unwrap();
    /// ```
    /// The above code generates a new folder from the root folder called "New Folder at Root".
    /// As an example, if, like many projects, the top level folders are named after resources,
    /// such as "Sprite" or "Object", then "New Folder at Root" will be at the same level as those folders.
    pub fn root_folder() -> ViewPath {
        ViewPath {
            name: "folders".to_string(),
            path: ViewPathLocation::root_folder(),
        }
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
    pub fn root_resource(&self) -> ViewPath {
        ViewPath {
            name: self.yyp.name.to_string(),
            path: ViewPathLocation::root_file(&self.yyp.name),
        }
    }

    /// Shows the underlying Yyp. This is exposed mostly
    /// for integration tests.
    pub fn yyp(&self) -> &Yyp {
        &self.yyp
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
            && self.folder_graph_manager == other.folder_graph_manager
            && self.resource_names == other.resource_names
    }
}
