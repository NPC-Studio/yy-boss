use super::{
    directory_manager::DirectoryManager, errors::*, folders::*, pipelines::PipelineManager, utils,
    YyResource, YyResourceData, YyResourceHandler, YypSerialization,
};
use crate::{FileSerializationError, ProjectMetadata, Resource};
use anyhow::{Context, Result as AnyResult};
use object_yy::Object;
use shader::Shader;
use std::{fs, path::Path};
use yy_typings::{
    script::Script,
    sounds::Sound,
    sprite_yy::*,
    utils::{ResourceNameValidator, TrailingCommaUtility},
    AnimationCurve, Font, Path as YyPath, Yyp,
};

static TCU: once_cell::sync::Lazy<TrailingCommaUtility> =
    once_cell::sync::Lazy::new(TrailingCommaUtility::new);

static RNV: once_cell::sync::Lazy<ResourceNameValidator> =
    once_cell::sync::Lazy::new(ResourceNameValidator::new);

#[derive(Debug, PartialEq, Default)]
pub struct YypBoss {
    pub directory_manager: DirectoryManager,
    pub pipeline_manager: PipelineManager,
    pub sprites: YyResourceHandler<Sprite>,
    pub scripts: YyResourceHandler<Script>,
    pub objects: YyResourceHandler<Object>,
    pub shaders: YyResourceHandler<Shader>,
    pub notes: YyResourceHandler<Note>,
    pub sounds: YyResourceHandler<Sound>,

    pub animation_curves: YyResourceHandler<AnimationCurve>,
    pub extensions: YyResourceHandler<Extension>,
    pub fonts: YyResourceHandler<Font>,
    pub paths: YyResourceHandler<YyPath>,
    pub rooms: YyResourceHandler<Room>,
    pub sequences: YyResourceHandler<Sequence>,
    pub tilesets: YyResourceHandler<TileSet>,
    pub timelines: YyResourceHandler<Timeline>,

    pub vfs: Vfs,
    yyp: Yyp,
}

impl YypBoss {
    /// Creates a new YyBoss Manager and performs startup file reading.
    pub fn new<P: AsRef<Path>>(path_to_yyp: P) -> Result<YypBoss, StartupError> {
        Self::with_startup_injest(path_to_yyp, &[])
    }

    pub fn with_startup_injest<P: AsRef<Path>>(
        path_to_yyp: P,
        resources_to_scan: &[Resource],
    ) -> Result<YypBoss, StartupError> {
        let yyp: Yyp = utils::deserialize_json_tc(&path_to_yyp, &TCU).map_err(|e| match e {
            FileSerializationError::Serde(e) => StartupError::BadYypDeserialize(e),
            FileSerializationError::Io(error) => StartupError::BadYypPath {
                yyp_filepath: path_to_yyp.as_ref().to_owned(),
                error,
            },
        })?;

        if yyp.meta_data.ide_version != Yyp::DEFAULT_VERSION {
            return Err(StartupError::YypIsWrongVersion(
                Yyp::DEFAULT_VERSION.to_string(),
                yyp.meta_data.ide_version,
            ));
        }

        let directory_manager = DirectoryManager::new(path_to_yyp.as_ref())?;

        let mut yyp_boss = Self {
            vfs: Vfs::new(&yyp.name),
            pipeline_manager: PipelineManager::new(&directory_manager),
            directory_manager,
            yyp,
            ..Self::default()
        };

        // Load in Folders
        yyp_boss.vfs.load_in_folders(&yyp_boss.yyp.folders);

        // load in all of our resources...
        for yyp_resource in yyp_boss.yyp.resources.clone().into_iter() {
            let path_as_str = yyp_resource.id.path.to_string_lossy();

            let subpath = path_as_str
                .split('/')
                .next()
                .ok_or_else(|| StartupError::BadResourceListing(yyp_resource.id.path.clone()))?;

            let resource = Resource::parse_subpath(subpath)
                .ok_or_else(|| StartupError::BadResourceListing(yyp_resource.id.path.clone()))?;
            let assoc = resources_to_scan.contains(&resource);

            match resource {
                Resource::Sprite => load_in_file::<Sprite>(yyp_resource, &mut yyp_boss, assoc),
                Resource::Script => load_in_file::<Script>(yyp_resource, &mut yyp_boss, assoc),
                Resource::Object => load_in_file::<Object>(yyp_resource, &mut yyp_boss, assoc),
                Resource::Note => load_in_file::<Note>(yyp_resource, &mut yyp_boss, assoc),
                Resource::Shader => load_in_file::<Shader>(yyp_resource, &mut yyp_boss, assoc),
                Resource::AnimationCurve => {
                    load_in_file::<AnimationCurve>(yyp_resource, &mut yyp_boss, assoc)
                }
                Resource::Room => load_in_file::<Room>(yyp_resource, &mut yyp_boss, assoc),
                Resource::Extension => {
                    load_in_file::<Extension>(yyp_resource, &mut yyp_boss, assoc)
                }
                Resource::Font => load_in_file::<Font>(yyp_resource, &mut yyp_boss, assoc),
                Resource::Path => load_in_file::<YyPath>(yyp_resource, &mut yyp_boss, assoc),
                Resource::Sequence => load_in_file::<Sequence>(yyp_resource, &mut yyp_boss, assoc),
                Resource::Sound => load_in_file::<Sound>(yyp_resource, &mut yyp_boss, assoc),
                Resource::TileSet => load_in_file::<TileSet>(yyp_resource, &mut yyp_boss, assoc),
                Resource::Timeline => load_in_file::<Timeline>(yyp_resource, &mut yyp_boss, assoc),
            }?;
        }

        return Ok(yyp_boss);

        fn load_in_file<T: YyResource>(
            yyp_resource: YypResource,
            yyp_boss: &mut YypBoss,
            load_in_associated_data: bool,
        ) -> Result<(), StartupError> {
            let yy_file_path = yyp_boss
                .directory_manager
                .root_directory()
                .join(&yyp_resource.id.path);

            let yy_file: T = utils::deserialize_json_tc(&yy_file_path, &TCU).map_err(|e| {
                StartupError::BadYyFile {
                    filepath: yy_file_path,
                    error: e.to_string(),
                }
            })?;

            yyp_boss
                .vfs
                .load_in_file(&yy_file, yyp_resource.order)
                .map_err(|e| StartupError::BadResourceTree {
                    name: yy_file.name().to_owned(),
                    error: e.to_string(),
                })?;

            let name = yy_file.name().to_owned();
            let root_path = yyp_boss.directory_manager.root_directory().to_owned();
            let handler = T::get_handler_mut(yyp_boss);
            handler.load_on_startup(yy_file);

            if load_in_associated_data {
                handler.load_resource_associated_data(&name, &root_path, &TCU)?;
            }

            Ok(())
        }
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

        // serialize all the tracked components
        self.sprites.serialize(&self.directory_manager)?;
        self.objects.serialize(&self.directory_manager)?;
        self.scripts.serialize(&self.directory_manager)?;
        self.notes.serialize(&self.directory_manager)?;
        self.shaders.serialize(&self.directory_manager)?;

        // THESE DO NOT HAVE EXCELLENT TYPINGS YET.
        self.animation_curves.serialize(&self.directory_manager)?;
        self.extensions.serialize(&self.directory_manager)?;
        self.fonts.serialize(&self.directory_manager)?;
        self.paths.serialize(&self.directory_manager)?;
        self.rooms.serialize(&self.directory_manager)?;
        self.sequences.serialize(&self.directory_manager)?;
        self.sounds.serialize(&self.directory_manager)?;
        self.tilesets.serialize(&self.directory_manager)?;
        self.timelines.serialize(&self.directory_manager)?;

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

    pub fn project_metadata(&self) -> ProjectMetadata {
        ProjectMetadata {
            name: self.yyp.name.clone(),
            ide_version: self.yyp.meta_data.ide_version.clone(),
            yyp_version: self.yyp.resource_version,
            root_file: ViewPath {
                name: self.yyp.name.clone(),
                path: self.vfs.root_file_viewpath(),
            },
        }
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
        self.can_use_name(yy_file.name())?;
        if T::RESOURCE.can_manipulate() == false {
            return Err(ResourceManipulationError::ResourceCannotBeManipulated);
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
            .ok_or(ResourceManipulationError::InternalError)
    }

    /// Adds a new resource, which must not already exist within the project.
    pub fn rename_resource<T: YyResource>(
        &mut self,
        name: &str,
        new_name: String,
    ) -> Result<(), ResourceManipulationError> {
        // we cannot rename resources, since we cannot reserialize them...
        if T::RESOURCE.can_manipulate() == false {
            return Err(ResourceManipulationError::ResourceCannotBeManipulated);
        }

        // check to make sure the new name isn't taken...
        if let Some(value) = self.vfs.resource_names.get(&new_name) {
            return Err(ResourceManipulationError::NameCollision(value.resource));
        }

        // check to make sure we're not dealing with some COMEDIANS
        if name == new_name {
            return Ok(());
        }

        // rename the file in the VFS...
        self.vfs
            .rename_resource(name, T::RESOURCE, new_name.clone())?;

        let path = self.directory_manager.root_directory().to_path_buf();
        let handler = T::get_handler_mut(self);
        handler
            .rename(name, new_name, &path, &TCU)
            .map_err(|_| ResourceManipulationError::InternalError)?;

        Ok(())
    }

    pub fn can_use_name(&self, name: &str) -> Result<(), ResourceManipulationError> {
        if let Some(r) = self.vfs.resource_names.get(name) {
            return Err(ResourceManipulationError::NameCollision(r.resource));
        }

        if RNV.is_valid(name) == false {
            return Err(ResourceManipulationError::BadName);
        }

        Ok(())
    }

    /// Move a resource within the Asset Tree
    pub fn move_resource<T: YyResource>(
        &mut self,
        name: &str,
        new_parent: ViewPath,
    ) -> Result<(), ResourceManipulationError> {
        // cannot move them because we cannot reserialize them
        if T::RESOURCE.can_manipulate() == false {
            return Err(ResourceManipulationError::ResourceCannotBeManipulated);
        }

        // vfs
        self.vfs
            .move_resource(name, T::RESOURCE, &new_parent.path)
            .map_err(ResourceManipulationError::FolderGraphError)?;

        let handler = T::get_handler_mut(self);
        handler
            .edit_parent(name, new_parent)
            .map_err(|_| ResourceManipulationError::InternalError)?;

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
    ) -> Result<(), YyResourceHandlerError> {
        // cannot move them because we cannot reserialize them

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
            Resource::Note => self.move_resource::<Note>(name, new_parent),
            Resource::Shader => self.move_resource::<Shader>(name, new_parent),
            _ => Err(ResourceManipulationError::ResourceCannotBeManipulated),
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
                Resource::Note => {
                    self.notes
                        .remove(&fsys.name, self.directory_manager.root_directory(), &TCU);
                }
                Resource::Shader => {
                    self.shaders
                        .remove(&fsys.name, self.directory_manager.root_directory(), &TCU);
                }
                Resource::AnimationCurve => {
                    self.animation_curves.remove(
                        &fsys.name,
                        self.directory_manager.root_directory(),
                        &TCU,
                    );
                }
                Resource::Extension => {
                    self.extensions.remove(
                        &fsys.name,
                        self.directory_manager.root_directory(),
                        &TCU,
                    );
                }
                Resource::Font => {
                    self.fonts
                        .remove(&fsys.name, self.directory_manager.root_directory(), &TCU);
                }
                Resource::Path => {
                    self.paths
                        .remove(&fsys.name, self.directory_manager.root_directory(), &TCU);
                }
                Resource::Room => {
                    self.rooms
                        .remove(&fsys.name, self.directory_manager.root_directory(), &TCU);
                }
                Resource::Sequence => {
                    self.sequences.remove(
                        &fsys.name,
                        self.directory_manager.root_directory(),
                        &TCU,
                    );
                }
                Resource::Sound => {
                    self.sounds
                        .remove(&fsys.name, self.directory_manager.root_directory(), &TCU);
                }
                Resource::TileSet => {
                    self.tilesets
                        .remove(&fsys.name, self.directory_manager.root_directory(), &TCU);
                }
                Resource::Timeline => {
                    self.timelines.remove(
                        &fsys.name,
                        self.directory_manager.root_directory(),
                        &TCU,
                    );
                }
            }
        }

        Ok(())
    }
}
