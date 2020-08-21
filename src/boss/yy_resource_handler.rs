use super::{directory_manager::DirectoryManager, utils, FilesystemPath, YyResource};
use crate::{AssocDataLocation, YyResourceHandlerErrors};
use anyhow::Result as AnyResult;
use log::{error, info};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};
use yy_typings::utils::TrailingCommaUtility;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub enum DirtyState {
    Edit,
    New,
}

#[derive(Debug, Default)]
pub struct YyResourceHandler<T: YyResource> {
    resources: HashMap<String, YyResourceData<T>>,
    pub(crate) resources_to_reserialize: HashMap<String, DirtyState>,
    pub(crate) resources_to_remove: HashMap<String, (DirtyState, T)>,
    pub(crate) associated_files_to_cleanup: Vec<PathBuf>,
    pub(crate) associated_folders_to_cleanup: Vec<PathBuf>,
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
    pub(crate) fn set(
        &mut self,
        value: T,
        associated_data: T::AssociatedData,
    ) -> Option<YyResourceData<T>> {
        let name = value.name().to_owned();
        let ret = self.insert_resource(value.clone(), Some(associated_data));

        if let Some(old) = &ret {
            old.yy_resource.cleanup_on_replace(
                &mut self.associated_files_to_cleanup,
                &mut self.associated_folders_to_cleanup,
            );

            let dirty_state = if let Some(state) = self.resources_to_reserialize.remove(&name) {
                state
            } else {
                DirtyState::Edit
            };

            self.resources_to_reserialize.insert(name, dirty_state);
        } else {
            match self.resources_to_remove.remove(&name) {
                Some((DirtyState::New, huh)) => {
                    if huh != value {
                        self.resources_to_reserialize.insert(name, DirtyState::New);
                    }
                    // do nothing
                }
                Some((DirtyState::Edit, _)) => {
                    self.resources_to_reserialize.insert(name, DirtyState::Edit);
                }
                None => {
                    self.resources_to_reserialize.insert(name, DirtyState::New);
                }
            }
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

    /// Removes the resource out of the handler. If that resource was being used,
    /// then this will return that resource.
    pub(crate) fn remove(
        &mut self,
        value: &str,
        tcu: &TrailingCommaUtility,
    ) -> Option<(T, Option<T::AssociatedData>)> {
        let ret = self.resources.remove(value);
        if let Some(ret) = ret {
            match self.resources_to_reserialize.remove(value) {
                Some(DirtyState::New) => {
                    // do nothing... let it die
                }
                Some(DirtyState::Edit) => {
                    self.resources_to_remove.insert(
                        value.to_owned(),
                        (DirtyState::Edit, ret.yy_resource.clone()),
                    );
                }
                None => {
                    self.resources_to_remove
                        .insert(value.to_owned(), (DirtyState::New, ret.yy_resource.clone()));
                }
            }

            let (yy, mut assoc) = ret.into();

            // Try to load this guy up...
            if assoc.is_none() {
                let output = self
                    .load_resource_associated_data(yy.name(), &yy.relative_yy_directory(), tcu)
                    .map_err(|e| {
                        error!("Couldn't deserialize {}'s assoc data...{}", value, e);
                        e
                    })
                    .ok();

                assoc = output.cloned();
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
        path: &Path,
        tcu: &TrailingCommaUtility,
    ) -> Result<&T::AssociatedData, YyResourceHandlerErrors> {
        if let Some(resource) = self.resources.get_mut(resource_name) {
            let associated_data = resource
                .yy_resource
                .deserialize_associated_data(AssocDataLocation::Path(path), tcu)?;

            resource.associated_data = Some(associated_data);

            Ok(&resource.associated_data.as_ref().unwrap())
        } else {
            Err(YyResourceHandlerErrors::ResourceNotFound)
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
        for (resource_to_remove, _) in self.resources_to_remove.drain() {
            let path = FilesystemPath::new_path(T::SUBPATH_NAME, &resource_to_remove);
            info!("removing resource {} at {:?}", resource_to_remove, path);
            let yy_path = directory_manager.resource_file(&path);
            fs::remove_dir_all(yy_path.parent().unwrap())?;
        }

        // Remove folders
        for folder in self.associated_folders_to_cleanup.drain(..) {
            let path = directory_manager
                .resource_file(Path::new(T::SUBPATH_NAME))
                .join(folder);
            info!("remove folder {:?}", path);
            fs::remove_dir_all(path)?;
        }

        // Remove files
        for file in self.associated_files_to_cleanup.drain(..) {
            let path = directory_manager
                .resource_file(Path::new(T::SUBPATH_NAME))
                .join(file);
            info!("removing path {:?}", path);
            fs::remove_file(path)?;
        }

        // Finally, reserialize resources
        for (resource_to_reserialize, _) in self.resources_to_reserialize.drain() {
            info!("reserializing {}", resource_to_reserialize);

            let resource = self
                .resources
                .get(&resource_to_reserialize)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dummy::DummyResource;
    use maplit::hashmap;

    #[test]
    fn add() {
        let mut dummy_handler: YyResourceHandler<DummyResource> = YyResourceHandler::new();

        assert!(dummy_handler.set(DummyResource::new("a", 0), ()).is_none());
        assert!(dummy_handler.set(DummyResource::new("b", 0), ()).is_none());

        assert_eq!(
            dummy_handler.resources_to_reserialize,
            hashmap! {
                "a".to_string() => DirtyState::New,
                "b".to_string() => DirtyState::New
            }
        );
        assert_eq!(dummy_handler.resources_to_remove, HashMap::default());

        assert_eq!(
            dummy_handler.set(DummyResource::new("a", 1), ()),
            Some(YyResourceData {
                yy_resource: DummyResource::new("a", 0),
                associated_data: Some(())
            })
        );
    }

    #[test]
    fn replace() {
        let mut dummy_handler: YyResourceHandler<DummyResource> = YyResourceHandler::new();

        dummy_handler.set(DummyResource::new("a", 0), ());
        dummy_handler.set(DummyResource::new("b", 0), ());
        dummy_handler.resources_to_reserialize.clear();

        assert_eq!(
            dummy_handler.set(DummyResource::new("a", 1), ()),
            Some(YyResourceData {
                yy_resource: DummyResource::new("a", 0),
                associated_data: Some(())
            })
        );
        assert_eq!(
            dummy_handler.associated_files_to_cleanup,
            vec![Path::new("a/0.txt")]
        );
        assert_eq!(
            dummy_handler.associated_folders_to_cleanup,
            vec![Path::new("a/0")]
        );
        assert_eq!(
            dummy_handler.resources_to_reserialize,
            hashmap! {
                "a".to_string() => DirtyState::Edit,
            }
        );
    }

    #[test]
    fn remove() {
        let mut dummy_handler: YyResourceHandler<DummyResource> = YyResourceHandler::new();
        let tcu = TrailingCommaUtility::new();

        dummy_handler.set(DummyResource::new("a", 0), ());
        dummy_handler.resources_to_reserialize.clear();

        assert!(dummy_handler.resources_to_reserialize.is_empty());
        assert!(dummy_handler.resources_to_remove.is_empty());

        assert_eq!(
            dummy_handler.remove("a", &tcu),
            Some((DummyResource::new("a", 0), Some(())))
        );
        assert_eq!(dummy_handler.remove("a", &tcu), None);

        assert_eq!(dummy_handler.resources_to_reserialize, hashmap! {});
        assert_eq!(
            dummy_handler.resources_to_remove,
            hashmap! {
                "a".to_string() => (DirtyState::New, DummyResource::new("a", 0)),
            }
        );
    }

    #[test]
    fn add_remove_simple_symmetry() {
        let mut dummy_handler: YyResourceHandler<DummyResource> = YyResourceHandler::new();
        let tcu = TrailingCommaUtility::new();

        dummy_handler.set(DummyResource::new("a", 0), ());
        assert!(dummy_handler.remove("a", &tcu).is_some());

        assert!(dummy_handler.resources_to_reserialize.is_empty());
        assert!(dummy_handler.resources_to_remove.is_empty());
    }

    #[test]
    fn remove_add_simple_symmetry() {
        let mut dummy_handler: YyResourceHandler<DummyResource> = YyResourceHandler::new();
        let tcu = TrailingCommaUtility::new();

        dummy_handler.set(DummyResource::new("a", 0), ());
        dummy_handler.resources_to_reserialize.clear();

        // we removed it!
        assert_eq!(
            dummy_handler.remove("a", &tcu),
            Some((DummyResource::new("a", 0), Some(())))
        );

        // reset the thing...
        dummy_handler.set(DummyResource::new("a", 0), ());

        assert_eq!(dummy_handler.resources_to_reserialize, hashmap! {});
        assert!(dummy_handler.resources_to_remove.is_empty());
    }

    #[test]
    fn remove_add_complex_symmetry() {
        let mut dummy_handler: YyResourceHandler<DummyResource> = YyResourceHandler::new();
        let tcu = TrailingCommaUtility::new();

        dummy_handler.set(DummyResource::new("a", 0), ());
        dummy_handler.resources_to_reserialize.clear();

        // we removed it!
        assert_eq!(
            dummy_handler.remove("a", &tcu),
            Some((DummyResource::new("a", 0), Some(())))
        );

        // reset the thing...
        dummy_handler.set(DummyResource::new("a", 1), ());

        assert_eq!(
            dummy_handler.resources_to_reserialize,
            hashmap! {
                "a".to_string() => DirtyState::New,
            }
        );
        assert!(dummy_handler.resources_to_remove.is_empty());
    }

    #[test]
    fn remove_add_remove_symmetry() {
        let mut dummy_handler: YyResourceHandler<DummyResource> = YyResourceHandler::new();
        let tcu = TrailingCommaUtility::new();

        dummy_handler.set(DummyResource::new("a", 0), ());
        dummy_handler.resources_to_reserialize.clear();

        // we removed it!
        dummy_handler.remove("a", &tcu);
        dummy_handler.set(DummyResource::new("a", 0), ());
        dummy_handler.remove("a", &tcu);

        assert!(dummy_handler.resources_to_reserialize.is_empty(),);
        assert_eq!(
            dummy_handler.resources_to_remove,
            hashmap! {
                "a".to_string() => (DirtyState::New, DummyResource::new("a", 0)),
            }
        );

        assert!(dummy_handler.associated_files_to_cleanup.is_empty());
        assert!(dummy_handler.associated_folders_to_cleanup.is_empty());
    }

    #[test]
    fn add_remove_add_symmetry() {
        let mut dummy_handler: YyResourceHandler<DummyResource> = YyResourceHandler::new();
        let tcu = TrailingCommaUtility::new();

        // we removed it!
        dummy_handler.set(DummyResource::new("a", 0), ());
        dummy_handler.remove("a", &tcu);
        dummy_handler.set(DummyResource::new("a", 0), ());

        assert_eq!(
            dummy_handler.resources_to_reserialize,
            hashmap! {
                "a".to_string() => DirtyState::New
            }
        );
        assert_eq!(dummy_handler.resources_to_remove, hashmap! {});

        assert!(dummy_handler.associated_files_to_cleanup.is_empty());
        assert!(dummy_handler.associated_folders_to_cleanup.is_empty());
    }

    #[test]
    fn replace_remove() {
        let mut dummy_handler: YyResourceHandler<DummyResource> = YyResourceHandler::new();
        let tcu = TrailingCommaUtility::new();

        // we removed it!
        dummy_handler.set(DummyResource::new("a", 0), ());
        dummy_handler.set(DummyResource::new("a", 0), ());
        dummy_handler.remove("a", &tcu);

        assert_eq!(dummy_handler.resources_to_reserialize, hashmap! {});
        assert_eq!(dummy_handler.resources_to_remove, hashmap! {});

        assert!(dummy_handler.associated_files_to_cleanup.is_empty());
        assert!(dummy_handler.associated_folders_to_cleanup.is_empty());
    }
}
