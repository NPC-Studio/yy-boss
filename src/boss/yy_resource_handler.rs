use super::{
    directory_manager::DirectoryManager,
    resources::{CreatedResource, RemovedResource},
    utils, FilesystemPath, YyResource,
};
use anyhow::Result as AnyResult;
use std::{collections::HashMap, fs};

#[derive(Debug, Default)]
pub struct YyResourceHandler<T: YyResource> {
    pub resources: HashMap<FilesystemPath, YyResourceData<T>>,
    pub dirty_resources: Vec<FilesystemPath>,
    pub resources_to_remove: Vec<FilesystemPath>,
}

impl<T: YyResource> YyResourceHandler<T> {
    pub(crate) fn new() -> Self {
        Self {
            resources: HashMap::new(),
            dirty_resources: Vec::new(),
            resources_to_remove: Vec::new(),
            // folders_to_be_deleted: Vec::new(),
        }
    }

    /// Adds a new sprite into the game. It requires a `CreatedResource`,
    /// which is created from the `YypBoss`, which guarantees that the resource
    /// has been created in the Yyp.
    ///
    /// This operation is used to `add` or to `replace` the resource. If it is used
    /// to replace a resource, the resource will be returned.
    pub fn set(
        &mut self,
        value: T,
        associated_data: T::AssociatedData,
        _frt: CreatedResource,
    ) -> Option<YyResourceData<T>> {
        self.dirty_resources
            .push(FilesystemPath::new(T::SUBPATH_NAME, value.name()));
        self.insert_resource(value, Some(associated_data))
    }

    /// Returns the data on the sprite yy, if it exists.
    ///
    /// In general, this will return a `Some`, but if users add
    /// a resource, without using the `FilledResource`token, then this will return a `None`.
    ///
    /// You can check if this is possible beforehand by checking the `YypBoss`'s prunable state.
    pub fn get(&self, name: &str, _crt: CreatedResource) -> Option<T> {
        self.resources
            .get(&FilesystemPath::new(T::SUBPATH_NAME, name))
            .map(|v| v.yy_resource.clone())
    }

    /// Removes the resource out of the handler. If that resource was being used,
    /// then this will return that resource.
    pub fn remove(
        &mut self,
        value: &FilesystemPath,
        _rrt: RemovedResource,
    ) -> Option<YyResourceData<T>> {
        if let Some(res) = self.resources.remove(&value) {
            self.resources_to_remove.push(value.clone());
            Some(res)
        } else {
            None
        }
    }

    /// Loads the resource in on startup. We don't track associated data by default,
    /// and we don't mark the resource as dirty.
    pub(crate) fn load_on_startup(&mut self, value: T) {
        self.insert_resource(value, None);
    }

    /// Writes all of the resources to disk.
    pub(crate) fn serialize(&mut self, directory_manager: &DirectoryManager) -> AnyResult<()> {
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

                let yy_path = directory_manager.resource_file(
                    &FilesystemPath::new(T::SUBPATH_NAME, resource.yy_resource.name()).path,
                );

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

    /// Wrapper around inserting the resource into `self.resources`.
    pub(crate) fn insert_resource(
        &mut self,
        value: T,
        associated_data: Option<T::AssociatedData>,
    ) -> Option<YyResourceData<T>> {
        self.resources.insert(
            FilesystemPath::new(T::SUBPATH_NAME, value.name()),
            YyResourceData {
                yy_resource: value,
                associated_data,
            },
        )
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
