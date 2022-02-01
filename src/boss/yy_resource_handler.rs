use super::{
    directory_manager::DirectoryManager,
    dirty_handler::{DirtyDrain, DirtyHandler},
    utils, FilesystemPath, YyResource,
};
use crate::YyResourceHandlerError;
use anyhow::Result as AnyResult;
use log::{error, info};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};
use yy_typings::{utils::TrailingCommaUtility, ViewPath};

#[derive(Debug, PartialEq)]
pub struct YyResourceHandler<T: YyResource> {
    pub resources: HashMap<String, YyResourceData<T>>,
    dirty_handler: DirtyHandler<String, Vec<PathBuf>>,
}

impl<T: YyResource> Default for YyResourceHandler<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: YyResource> YyResourceHandler<T> {
    pub(crate) fn new() -> Self {
        Self {
            resources: HashMap::new(),
            dirty_handler: DirtyHandler::new_assoc(),
        }
    }

    /// Adds a new sprite into the game. It requires a `CreatedResource`,
    /// which is created from the `YypBoss`, which guarantees that the resource
    /// has been created in the Yyp.
    ///
    /// This operation is used to `add` or to `replace` the resource. If it is used
    /// to replace a resource, the resource will be returned.
    pub(crate) fn set(
        &mut self,
        value: T,
        associated_data: T::AssociatedData,
    ) -> Option<YyResourceData<T>> {
        let name = value.name().to_owned();
        let ret = self.insert_resource(value, Some(associated_data));

        if let Some(old) = &ret {
            self.dirty_handler
                .replace_associated(name, |f| old.yy_resource.cleanup_on_replace(f));
        } else {
            self.dirty_handler.add(name);
        }

        ret
    }

    /// Returns an immutable reference to a resource's data, if it exists.
    ///
    /// Since associated data is lazily loaded, and be unloaded at any time,
    /// there may not be any associated data returned. You can request that data to be
    /// loaded using [`load_resource_associated_data`].
    ///
    /// [`load_resource_associated_data`]: #method.load_resource_associated_data
    pub fn get(&self, name: &str) -> Option<&YyResourceData<T>> {
        self.resources.get(name)
    }

    /// Returns an **mutable** reference to a resource's data, if it exists.
    ///
    /// Since associated data is lazily loaded, and be unloaded at any time,
    /// there may not be any associated data returned. You can request that data to be
    /// loaded using [`load_resource_associated_data`].
    ///
    /// # Safety
    ///
    /// This function is VERY UNSAFE, but not in a memory sense. The entire purpose
    /// of this library is, in effect, to shield the user from making changes to the Yyp
    /// database which require side effects. How could a user know which effects to use and what
    /// to not use? They absolutely cannot. Unless you are very certain that the change you
    /// are making will not impact *anything* outside that file, do not use this function.
    ///
    /// Additionally, be aware that mutating data does not guarentee serialization: please use [`force_serialize`].
    ///
    /// [`load_resource_associated_data`]: #method.load_resource_associated_data
    /// [`force_serialize`]: #method.force_serialize
    pub unsafe fn get_mut(&mut self, name: &str) -> Option<&mut YyResourceData<T>> {
        self.resources.get_mut(name)
    }

    /// Attempts to mark a resource for serialization, and returns if it was succesfully marked.
    pub fn force_serialize(&mut self, name: &str) -> Result<(), YyResourceHandlerError> {
        if let Some(inner) = self.resources.get(name) {
            if inner.associated_data.is_some() {
                if self
                    .dirty_handler
                    .resources_to_reserialize()
                    .get(name)
                    .is_none()
                {
                    self.dirty_handler.edit(name.to_string());
                } else {
                    log::warn!(
                        "attempt to force serialize {}, which was already marked for serialization",
                        name
                    );
                }

                Ok(())
            } else {
                Err(YyResourceHandlerError::CannotForceSerialization)
            }
        } else {
            Err(YyResourceHandlerError::ResourceNotFound)
        }
    }

    pub(crate) fn edit_parent(
        &mut self,
        name: &str,
        parent: ViewPath,
    ) -> Result<(), YyResourceHandlerError> {
        if let Some(inner) = self.resources.get_mut(name) {
            inner.yy_resource.set_parent_view_path(parent);
            self.dirty_handler.edit(name.to_string());

            Ok(())
        } else {
            Err(YyResourceHandlerError::ResourceNotFound)
        }
    }

    pub(crate) fn rename(
        &mut self,
        current_name: &str,
        new_name: String,
        dir_path: &Path,
        tcu: &TrailingCommaUtility,
    ) -> Result<(), YyResourceHandlerError> {
        if let Some(mut inner) = self.resources.remove(current_name) {
            // Try to load this guy up...
            if inner.associated_data.is_none() {
                let output = inner
                    .yy_resource
                    .deserialize_associated_data(
                        &dir_path.join(&inner.yy_resource.relative_yy_directory()),
                        tcu,
                    )
                    .map_err(|e| {
                        error!(
                            "Couldn't deserialize {}'s associated data...{}",
                            current_name, e
                        );
                        e
                    })
                    .ok()
                    .unwrap_or_default();

                inner.associated_data = Some(output);
            }

            inner.yy_resource.set_name(new_name.clone());
            self.dirty_handler.remove(current_name);
            self.dirty_handler.add(new_name.clone());

            self.resources.insert(new_name, inner);

            Ok(())
        } else {
            Err(YyResourceHandlerError::ResourceNotFound)
        }
    }

    /// Removes the resource out of the handler. If that resource was being used,
    /// then this will return that resource.
    pub(crate) fn remove(
        &mut self,
        value: &str,
        dir_path: &Path,
        tcu: &TrailingCommaUtility,
    ) -> Option<(T, Option<T::AssociatedData>)> {
        let ret = self.resources.remove(value);
        if let Some(ret) = ret {
            self.dirty_handler.remove(value);

            let (yy, mut assoc) = ret.into();

            // Try to load this guy up...
            if assoc.is_none() {
                let output = yy
                    .deserialize_associated_data(&dir_path.join(&yy.relative_yy_directory()), tcu)
                    .map_err(|e| {
                        error!("Couldn't deserialize {}'s assoc data...{}", value, e);
                        e
                    })
                    .ok();

                assoc = output;
            }

            Some((yy, assoc))
        } else {
            None
        }
    }

    /// Loads in the associated data of a given resource name, if that resource exists and is managed.
    ///
    /// If that resource already has some associated data, it will be discarded, and the new data will be loaded.
    /// If the resource does not exist or is not of the type that this manager handles, an error will be
    /// returned.
    pub fn load_resource_associated_data(
        &mut self,
        resource_name: &str,
        root: &Path,
        tcu: &TrailingCommaUtility,
    ) -> Result<&T::AssociatedData, YyResourceHandlerError> {
        if let Some(resource) = self.resources.get_mut(resource_name) {
            let associated_data = resource.yy_resource.deserialize_associated_data(
                &root.join(resource.yy_resource.relative_yy_directory()),
                tcu,
            )?;

            resource.associated_data = Some(associated_data);

            Ok(resource.associated_data.as_ref().unwrap())
        } else {
            Err(YyResourceHandlerError::ResourceNotFound)
        }
    }

    /// Unloads a resource. This will free up some memory.
    ///
    /// If the resource does not exist on this handler, an error will be returned.
    pub fn unload_resource_associated_data(
        &mut self,
        resource_name: &str,
    ) -> Result<(), YyResourceHandlerError> {
        if let Some(resource) = self.resources.get_mut(resource_name) {
            resource.associated_data = None;
            Ok(())
        } else {
            Err(YyResourceHandlerError::ResourceNotFound)
        }
    }

    /// Loads the resource in on startup. We don't track associated data by default,
    /// and we don't mark the resource as dirty.
    pub(crate) fn load_on_startup(&mut self, value: T) {
        self.insert_resource(value, None);
    }

    /// Writes all of the resources to disk, and cleans up excess files.
    pub(crate) fn serialize(&mut self, directory_manager: &DirectoryManager) -> AnyResult<()> {
        let DirtyDrain {
            resources_to_remove,
            resources_to_reserialize,
            associated_values,
        } = self.dirty_handler.drain_all();

        // Remove files or folders...
        if let Some(ass_values) = associated_values {
            for (name, mut filepaths) in ass_values {
                let base_path = directory_manager
                    .resource_file(Path::new(T::SUBPATH_NAME))
                    .join(name);

                for fpath in filepaths.drain(..) {
                    let path = base_path.join(fpath);
                    if path.is_dir() {
                        match fs::remove_dir_all(&path) {
                            Ok(()) => {
                                info!("removed folder {:?}", path);
                            }
                            Err(e) => {
                                error!("couldn't remove folder {:#?}, {:#?}", path, e);
                            }
                        }
                    } else {
                        match fs::remove_file(&path) {
                            Ok(()) => {
                                info!("removed file {:?}", path);
                            }
                            Err(e) => {
                                error!("couldn't remove file {:#?}, {:#?}", path, e);
                            }
                        }
                    }
                }
            }
        }

        // Removes the resources!
        for (resource_to_remove, _) in resources_to_remove {
            let path = FilesystemPath::new_path(T::SUBPATH_NAME, &resource_to_remove);
            info!("removing resource {} at {:?}", resource_to_remove, path);
            let yy_path = directory_manager.resource_file(&path);
            fs::remove_dir_all(yy_path.parent().unwrap())?;
        }

        // Finally, reserialize resources
        for (resource_to_reserialize, _) in resources_to_reserialize {
            let resource = self
                .resources
                .get(&resource_to_reserialize)
                .expect("This should always be valid.");

            let yy_path =
                directory_manager.resource_file(&resource.yy_resource.relative_yy_filepath());

            let parent_dir = yy_path.parent().expect("impossible");
            fs::create_dir_all(parent_dir)?;

            if let Some(associated_data) = &resource.associated_data {
                resource
                    .yy_resource
                    .serialize_associated_data(parent_dir, associated_data)?;
            } else {
                error!(
                    "{} {} was marked for serialization but we didn't have its associated data",
                    T::RESOURCE,
                    resource.yy_resource.name()
                )
            }

            utils::serialize_json(&yy_path, &resource.yy_resource)?;
        }

        Ok(())
    }

    /// Wrapper around inserting the resource into `self.resources`.
    fn insert_resource(
        &mut self,
        value: T,
        associated_data: Option<T::AssociatedData>,
    ) -> Option<YyResourceData<T>> {
        self.resources.insert(
            value.name().to_owned(),
            YyResourceData {
                yy_resource: value,
                associated_data,
            },
        )
    }
}

#[derive(Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct YyResourceData<T: YyResource> {
    pub yy_resource: T,
    pub associated_data: Option<T::AssociatedData>,
}

impl<T: YyResource> From<YyResourceData<T>> for (T, Option<T::AssociatedData>) {
    fn from(o: YyResourceData<T>) -> Self {
        (o.yy_resource, o.associated_data)
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

impl<'a, T: YyResource> IntoIterator for &'a YyResourceHandler<T> {
    type Item = &'a YyResourceData<T>;
    type IntoIter = std::collections::hash_map::Values<'a, String, YyResourceData<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.resources.values()
    }
}
