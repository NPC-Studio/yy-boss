use crate::Resource;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    fmt::Debug,
    path::{Path, PathBuf},
};
use yy_typings::ViewPath;

pub trait YyResource: Serialize + for<'de> Deserialize<'de> + Clone + Default {
    type AssociatedData: Debug;
    const SUBPATH_NAME: &'static str;
    const RESOURCE: Resource;

    /// Get's the resource's name.
    fn name(&self) -> &str;

    /// Sets the name of the resource.
    fn set_name(&mut self, name: String);

    /// Get the path to the parent in the View Virtual File System.
    fn parent_path(&self) -> ViewPath;

    /// Deserialized the associated data with a given Yy File. In a sprite, for example,
    /// this would load the `pngs` into memory.
    fn deserialize_associated_data(
        &self,
        directory_path: &Path,
    ) -> Result<Option<Self::AssociatedData>>;

    /// Serialized the associated data with a given Yy File. In a sprite, for example,
    /// this would serialize the `png` files, or in a script, this would serialize the
    /// associated `gml` files.
    fn serialize_associated_data(
        &self,
        directory_path: &Path,
        data: &Self::AssociatedData,
    ) -> Result<()>;

    /// This cleans up any associated files which won't get cleaned up in the event of a
    /// REPLACEMENT of this resource. For example, when we replace a sprite_yy file, the old
    /// `png` files might not be replaced (because they are based on Uuids which will change).
    ///
    /// This functions is used to clean up those files. All of the paths are relative to the directory
    /// of the yy file.
    ///
    /// This function is ONLY called when a resource is being replaced. When a resource is being removed
    /// outright, then the entire folder is removed, so we don't need to carefully handle this.
    fn cleanup_on_replace(
        &self,
        files_to_delete: &mut Vec<PathBuf>,
        folders_to_delete: &mut Vec<PathBuf>,
    );
}
