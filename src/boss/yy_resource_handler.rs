use super::{directory_manager::DirectoryManager, utils, FilesystemPath, YyResource};
use anyhow::{bail, Result as AnyResult};
use std::{collections::HashMap, fs, path::Path};

#[derive(Debug, Default)]
pub struct YyResourceHandler<T: YyResource> {
    pub resources: HashMap<FilesystemPath, YyResourceData<T>>,
    pub dirty_resources: Vec<FilesystemPath>,
    pub resources_to_remove: Vec<FilesystemPath>,
    pub folders_to_be_deleted: Vec<FilesystemPath>,
}

impl<T: YyResource> YyResourceHandler<T> {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
            dirty_resources: Vec::new(),
            resources_to_remove: Vec::new(),
            folders_to_be_deleted: Vec::new(),
        }
    }

    /// Initialize Shared Data and Associated Data. For a sprite,
    /// for example, this will include loading all `pngs`.
    pub fn load_data(&mut self, project_directory: &Path) -> AnyResult<()> {
        // initialize the associated data for each sprite...
        for resource in self.resources.values_mut() {
            if resource.associated_data.is_none() {
                resource.associated_data = resource
                    .yy_resource
                    .load_associated_data(project_directory)?;
            }
        }

        Ok(())
    }

    /// Adds a new sprite! If that sprite already exists, it will error instead. To replace
    /// a sprite, please use `YyResourceHandler::overwrite` instead.
    pub fn add_new(&mut self, value: T, associated_data: T::AssociatedData) -> AnyResult<()> {
        if self.resources.contains_key(&value.filesystem_path()) {
            bail!("That sprite already existed!");
        }
        self.dirty_resources.push(value.filesystem_path());
        self.add_new_startup(value, Some(associated_data));

        Ok(())
    }

    /// Replaces a sprite which already existed. If that sprite doesn't exist, it will return
    /// an error instead. Use `YyResourceHandler::add_new` instead.
    pub fn overwrite(&mut self, value: T, associated_data: T::AssociatedData) -> AnyResult<()> {
        if self.remove(&value.filesystem_path()).is_none() {
            bail!("We didn't have an original sprite!");
        }

        self.add_new(value, associated_data)?;
        Ok(())
    }

    /// Attempts to remove the resource. Returns the data if it was present.
    pub fn remove(&mut self, value: &FilesystemPath) -> Option<YyResourceData<T>> {
        if let Some(res) = self.resources.remove(&value) {
            self.resources_to_remove.push(value.clone());
            Some(res)
        } else {
            None
        }
    }

    /// This is the same as `add_new` but it doesn't dirty the resource. It is used
    /// for startup operations, where we're loading in assets from disk.
    pub fn add_new_startup(
        &mut self,
        value: T,
        associated_data: Option<T::AssociatedData>,
    ) -> Option<YyResourceData<T>> {
        self.resources.insert(
            value.filesystem_path(),
            YyResourceData {
                yy_resource: value,
                associated_data,
            },
        )
    }

    pub fn serialize(&mut self, directory_manager: &DirectoryManager) -> AnyResult<()> {
        if self.resources_to_remove.is_empty() == false {
            while let Some(resource_to_remove) = self.resources_to_remove.pop() {
                let yy_path = directory_manager.resource_file(&resource_to_remove.path);
                fs::remove_dir_all(yy_path.parent().unwrap())?;
            }
        }

        if self.dirty_resources.is_empty() == false {
            while let Some(dirty_resource) = self.dirty_resources.pop() {
                let resource = self
                    .resources
                    .get(&dirty_resource)
                    .expect("This should always be valid.");

                let yy_path =
                    directory_manager.resource_file(&resource.yy_resource.filesystem_path().path);

                if let Some(parent_dir) = yy_path.parent() {
                    fs::create_dir_all(parent_dir)?;
                    if let Some(associated_data) = &resource.associated_data {
                        T::serialize_associated_data(
                            &resource.yy_resource,
                            parent_dir,
                            associated_data,
                        )?;
                    }
                }

                utils::serialize(&yy_path, &resource.yy_resource)?;
            }
        }

        Ok(())
    }
}

#[derive(Default)]
pub struct YyResourceData<T: YyResource> {
    pub yy_resource: T,
    pub associated_data: Option<T::AssociatedData>,
}

impl<T: YyResource> Into<(T, Option<T::AssociatedData>)> for YyResourceData<T> {
    fn into(self) -> (T, Option<T::AssociatedData>) {
        (self.yy_resource, self.associated_data)
    }
}

impl<T: YyResource + std::fmt::Debug> std::fmt::Debug for YyResourceData<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} !!**ASSOCIATED DATA IS NOT PRINTED IN DEBUG OUTPUT**!!",
            self.yy_resource
        )
    }
}
