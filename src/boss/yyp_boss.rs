use super::{
    directory_manager::DirectoryManager, errors::*, folders::*, pipelines::PipelineManager, utils,
    YyResource, YyResourceData, YyResourceHandler, YypSerialization,
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
    pub vfs: Vfs,
    pub tcu: TrailingCommaUtility,
    yyp: Yyp,
}

impl YypBoss {
    /// Creates a new YyBoss Manager and performs startup file reading.
    pub fn new<P: AsRef<Path>>(path_to_yyp: P) -> Result<YypBoss, StartupError> {
        let tcu = TrailingCommaUtility::new();
        let yyp: Yyp = utils::deserialize_json_tc(&path_to_yyp, &tcu)?;

        let directory_manager = DirectoryManager::new(path_to_yyp.as_ref())?;

        let mut yyp_boss = Self {
            vfs: Vfs::new(&yyp.name),
            tcu,
            sprites: YyResourceHandler::new(),
            scripts: YyResourceHandler::new(),
            objects: YyResourceHandler::new(),
            pipeline_manager: PipelineManager::new(&directory_manager)?,
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

                folder_graph.load_in_file(&yy_file, yyp_resource.order)?;
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
            &yyp_boss.tcu,
        )?;
        load_in_resource(
            &mut yyp_boss.scripts,
            &mut yyp_boss.vfs,
            &yyp_boss.yyp.resources,
            &yyp_boss.directory_manager,
            &yyp_boss.tcu,
        )?;
        load_in_resource(
            &mut yyp_boss.objects,
            &mut yyp_boss.vfs,
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
        if let Some(r) = self.vfs.resource_names.get(yy_file.name()) {
            return Err(ResourceManipulationError::BadResourceName(r.resource));
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
        // confirm the resource exists...
        if let Some(v) = self.vfs.resource_names.get(name) {
            if v.resource != T::RESOURCE {
                return Err(ResourceManipulationError::BadResourceName(v.resource));
            }
        } else {
            return Err(ResourceManipulationError::NoResourceByThatName);
        }

        // remove the file from the VFS...
        self.vfs.remove_resource(name);

        let handler = T::get_handler_mut(self);
        let tcu = TrailingCommaUtility::new();
        handler
            .remove(name, &tcu)
            .ok_or_else(|| ResourceManipulationError::InternalError)
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
        self.yyp == other.yyp && self.vfs == other.vfs
    }
}
