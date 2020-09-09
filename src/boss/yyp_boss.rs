use super::{
    directory_manager::DirectoryManager, errors::*, folders::*, pipelines::PipelineManager, utils,
    YyResource, YyResourceData, YyResourceHandler, YypSerialization,
};
use crate::Resource;
use anyhow::{Context, Result as AnyResult};
use object_yy::Object;
use std::{fs, path::Path};
use yy_typings::{script::Script, sprite_yy::*, utils::TrailingCommaUtility, Yyp};

static TCU: once_cell::sync::Lazy<TrailingCommaUtility> =
    once_cell::sync::Lazy::new(TrailingCommaUtility::new);

#[derive(Debug, PartialEq)]
pub struct YypBoss {
    pub directory_manager: DirectoryManager,
    pub pipeline_manager: PipelineManager,
    pub sprites: YyResourceHandler<Sprite>,
    pub scripts: YyResourceHandler<Script>,
    pub objects: YyResourceHandler<Object>,
    pub vfs: Vfs,
    yyp: Yyp,
}

impl YypBoss {
    /// Creates a new YyBoss Manager and performs startup file reading.
    pub fn new<P: AsRef<Path>>(path_to_yyp: P) -> Result<YypBoss, StartupError> {
        let yyp: Yyp = utils::deserialize_json_tc(&path_to_yyp, &TCU).map_err(|e| match e {
            crate::FileSerializationError::Serde(e) => StartupError::BadYypDeserialize(e),
            crate::FileSerializationError::Io(error) => StartupError::BadYypPath {
                yyp_filepath: path_to_yyp.as_ref().to_owned(),
                error,
            },
        })?;

        let directory_manager = DirectoryManager::new(path_to_yyp.as_ref())?;

        let mut yyp_boss = Self {
            vfs: Vfs::new(&yyp.name),
            sprites: YyResourceHandler::new(),
            scripts: YyResourceHandler::new(),
            objects: YyResourceHandler::new(),
            pipeline_manager: PipelineManager::new(&directory_manager),
            directory_manager,
            yyp,
        };

        // Load in Folders
        yyp_boss.vfs.load_in_folders(&yyp_boss.yyp.folders);

        fn load_in_resource<T: YyResource>(
            resource: &mut YyResourceHandler<T>,
            folder_graph: &mut Vfs,
            yyp_resources: &[YypResource],
            directory_manager: &DirectoryManager,
        ) -> Result<(), StartupError> {
            for yyp_resource in yyp_resources
                .iter()
                .filter(|value| value.id.path.starts_with(T::SUBPATH_NAME))
            {
                let yy_file_path = directory_manager
                    .root_directory()
                    .join(&yyp_resource.id.path);

                let yy_file: T = utils::deserialize_json_tc(&yy_file_path, &TCU).map_err(|e| {
                    StartupError::BadYyFile {
                        filepath: yy_file_path,
                        error: e.to_string(),
                    }
                })?;

                folder_graph
                    .load_in_file(&yy_file, yyp_resource.order)
                    .map_err(|e| StartupError::BadResourceTree {
                        name: yy_file.name().to_owned(),
                        error: e.to_string(),
                    })?;
                resource.load_on_startup(yy_file);
            }

            Ok(())
        }

        // Load in our Resources
        // @update_resource
        load_in_resource(
            &mut yyp_boss.sprites,
            &mut yyp_boss.vfs,
            &yyp_boss.yyp.resources,
            &yyp_boss.directory_manager,
        )?;
        load_in_resource(
            &mut yyp_boss.scripts,
            &mut yyp_boss.vfs,
            &yyp_boss.yyp.resources,
            &yyp_boss.directory_manager,
        )?;
        load_in_resource(
            &mut yyp_boss.objects,
            &mut yyp_boss.vfs,
            &yyp_boss.yyp.resources,
            &yyp_boss.directory_manager,
        )?;

        Ok(yyp_boss)
    }

    /// Gets the default texture path, if it exists. The "Default" group simply
    /// has the name `"Default"`.
    ///
    /// This method will almost certainly be refactored soon to a dedicated TextureManager.
    pub fn default_texture_path(&self) -> Option<TexturePath> {
        self.yyp
            .texture_groups
            .iter()
            .find(|tex| tex.name == "Default")
            .map(|texture_group| texture_group.into())
    }

    /// Serializes the YypBoss data to disk at the path of the Yyp.
    pub fn serialize(&mut self) -> AnyResult<()> {
        // serialize the vfs
        self.vfs
            .serialize(&mut self.yyp.folders, &mut self.yyp.resources);

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
        let string = self.yyp.yyp_serialization(0);
        fs::write(&self.directory_manager.yyp(), &string)?;

        Ok(())
    }

    pub fn version_string(&self) -> &str {
        &self.yyp.meta_data.ide_version
    }

    pub fn tcu(&self) -> &TrailingCommaUtility {
        &TCU
    }

    pub fn yyp(&self) -> &Yyp {
        &self.yyp
    }
}

// for generics
impl YypBoss {
    /// Adds a new resource, which must not already exist within the project.
    pub fn add_resource<T: YyResource>(
        &mut self,
        yy_file: T,
        associated_data: T::AssociatedData,
    ) -> Result<(), ResourceManipulationError> {
        if let Some(r) = self.vfs.resource_names.get(yy_file.name()) {
            return Err(ResourceManipulationError::BadAdd {
                existing_resource: r.resource,
            });
        }

        self.vfs.new_resource_end(&yy_file)?;
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
        // remove the file from the VFS...
        self.vfs.remove_resource(name, T::RESOURCE)?;

        let path = self.directory_manager.root_directory().to_path_buf();
        let handler = T::get_handler_mut(self);
        handler
            .remove(name, &path, &TCU)
            .ok_or_else(|| ResourceManipulationError::InternalError)
    }

    /// Move a resource within the Asset Tree
    pub fn move_resource<T: YyResource>(
        &mut self,
        name: &str,
        new_parent: ViewPath,
    ) -> Result<(), ResourceManipulationError> {
        // vfs
        self.vfs
            .move_resource(name, T::RESOURCE, &new_parent.path)
            .map_err(ResourceManipulationError::FolderGraphError)?;

        let handler = T::get_handler_mut(self);
        handler.edit_parent(name, new_parent);

        Ok(())
    }

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

    /// Ensures some associated data is loaded by generic type. If you aren't working generically, just access
    /// the individual handlers for this.
    ///
    /// If `force` is passed in, then this will *always* reload the associated data. Be careful out there -- hot
    /// reloading isn't a feature we really support yet.
    ///
    /// This operation will return a reference to the associated data if we succeeded.
    pub fn ensure_associated_data_is_loaded<T: YyResource>(
        &mut self,
        name: &str,
        force: bool,
    ) -> Result<(), YyResourceHandlerErrors> {
        let path = self.directory_manager.root_directory().to_path_buf();
        let handler = T::get_handler_mut(self);

        let reload = handler
            .get(name)
            .map(|data| data.associated_data.is_none() || force)
            .unwrap_or(true);

        if reload {
            handler.load_resource_associated_data(name, &path, &TCU)?;
        }

        Ok(())
    }
}

// resource handling!
impl YypBoss {
    /// Move a resource within the Asset Tree, using the passed in resource type
    pub fn move_resource_dynamic(
        &mut self,
        name: &str,
        new_parent: ViewPath,
        resource: Resource,
    ) -> Result<(), ResourceManipulationError> {
        match resource {
            Resource::Sprite => self.move_resource::<Sprite>(name, new_parent),
            Resource::Script => self.move_resource::<Script>(name, new_parent),
            Resource::Object => self.move_resource::<Object>(name, new_parent),
        }
    }

    /// Removes a folder RECURSIVELY. **All resources within will be removed**. Be careful out there.
    pub fn remove_folder(
        &mut self,
        folder: &ViewPathLocation,
    ) -> Result<(), ResourceManipulationError> {
        // easy!
        if self.vfs.remove_empty_folder(folder).is_ok() {
            return Ok(());
        }

        // okay okay, more complex operation
        let deleted_resources = self.vfs.remove_non_empty_folder(folder)?;

        for (fsys, descriptor) in deleted_resources {
            match descriptor.resource {
                Resource::Sprite => {
                    self.sprites
                        .remove(&fsys.name, self.directory_manager.root_directory(), &TCU);
                }
                Resource::Script => {
                    self.scripts
                        .remove(&fsys.name, self.directory_manager.root_directory(), &TCU);
                }
                Resource::Object => {
                    self.objects
                        .remove(&fsys.name, self.directory_manager.root_directory(), &TCU);
                }
            }
        }

        Ok(())
    }
}
