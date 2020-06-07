use super::{utils, FilesystemPath, YyResource};
use anyhow::{bail, Result as AnyResult};
use std::{collections::HashMap, fs, path::Path};

#[derive(Debug, Default)]
pub struct YyResourceHandler<T: YyResource> {
    pub dirty: bool,
    pub resources: HashMap<FilesystemPath, YyResourceData<T>>,
    pub dirty_resources: Vec<FilesystemPath>,
}

impl<T: YyResource> YyResourceHandler<T> {
    pub fn new() -> Self {
        Self {
            dirty: false,
            resources: HashMap::new(),
            dirty_resources: Vec::new(),
        }
    }

    /// Initialize Shared Data and Associated Data. For a sprite,
    /// for example, this will include loading all `pngs` and loading
    /// the Shared Import Data.
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

    /// This loads or reloads data for each Associated Data and the Shared Data. In other words,
    /// this will OVERWRITE the current data for any resource which has a `Some(_)` Associated Data.
    /// If you just want to just load Associated Data for any Resources with `None`, use `load_data`.
    pub fn force_load_data(&mut self, project_directory: &Path) -> AnyResult<()> {
        // initialize the associated data for each sprite...
        for resource in self.resources.values_mut() {
            resource.associated_data = resource
                .yy_resource
                .load_associated_data(project_directory)?;
        }

        Ok(())
    }

    pub fn add_new(
        &mut self,
        value: T,
        associated_data: T::AssociatedData,
    ) -> Option<YyResourceData<T>> {
        self.dirty_resources.push(value.filesystem_path());
        self.dirty = true;
        self.add_new_startup(value, Some(associated_data))
    }

    /// Returns an error if the given resource did not exist.
    pub fn overwrite(&mut self, value: T, associated_data: T::AssociatedData) -> AnyResult<()> {
        if self.resources.contains_key(&value.filesystem_path()) == false {
            bail!("We didn't have an original resource!");
        }
        self.add_new(value, associated_data);

        Ok(())
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

    pub fn serialize(&mut self, project_path: &Path) -> AnyResult<()> {
        if self.dirty {
            while let Some(dirty_resource) = self.dirty_resources.pop() {
                let resource = self
                    .resources
                    .get(&dirty_resource)
                    .expect("This should always be valid.");

                let yy_path = project_path.join(&resource.yy_resource.filesystem_path().path);

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

impl<T: YyResource + std::fmt::Debug> std::fmt::Debug for YyResourceData<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} !!**ASSOCIATED DATA IS NOT PRINTED IN DEBUG OUTPUT**!!",
            self.yy_resource
        )
    }
}
