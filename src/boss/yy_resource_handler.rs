use super::{
    directory_manager::DirectoryManager,
    resources::{CreatedResource, RemovedResource},
    utils, FilesystemPath, YyResource,
};
use anyhow::Result as AnyResult;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Default)]
pub struct YyResourceHandler<T: YyResource> {
    resources: HashMap<FilesystemPath, YyResourceData<T>>,
    pub(crate) resources_to_reserialize: Vec<FilesystemPath>,
    pub(crate) associated_files_to_cleanup: Vec<PathBuf>,
    pub(crate) associated_folders_to_cleanup: Vec<PathBuf>,
    pub(crate) resources_to_remove: Vec<FilesystemPath>,
}

impl<T: YyResource> YyResourceHandler<T> {
    pub(crate) fn new() -> Self {
        Self::default()
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
        self.resources_to_reserialize
            .push(FilesystemPath::new(T::SUBPATH_NAME, value.name()));
        let ret = self.insert_resource(value, Some(associated_data));

        if let Some(old) = &ret {
            old.yy_resource.cleanup(
                &mut self.associated_files_to_cleanup,
                &mut self.associated_folders_to_cleanup,
            );
        }

        ret
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

    /// Writes all of the resources to disk, and cleans up excess files.
    pub(crate) fn serialize(&mut self, directory_manager: &DirectoryManager) -> AnyResult<()> {
        // Removes the resources!
        for resource_to_remove in self.resources_to_remove.drain(..) {
            let yy_path = directory_manager.resource_file(&resource_to_remove.path);
            fs::remove_dir_all(yy_path.parent().unwrap())?;
        }

        // Remove folders
        for folder in self.associated_folders_to_cleanup.drain(..) {
            fs::remove_dir_all(
                directory_manager
                    .resource_file(Path::new(T::SUBPATH_NAME))
                    .join(folder),
            )?;
        }

        // Remove files
        for file in self.associated_files_to_cleanup.drain(..) {
            fs::remove_file(
                directory_manager
                    .resource_file(Path::new(T::SUBPATH_NAME))
                    .join(file),
            )?;
        }

        for dirty_resource in self.resources_to_reserialize.drain(..) {
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
                    resource
                        .yy_resource
                        .serialize_associated_data(parent_dir, associated_data)?;
                }
            }

            utils::serialize(&yy_path, &resource.yy_resource)?;
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
