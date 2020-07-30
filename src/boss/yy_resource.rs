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
        project_directory: &Path,
    ) -> Result<Option<Self::AssociatedData>>;

    /// Serialized the associated data with a given Yy File. In a sprite, for example,
    /// this would serialize the `png`s.
    fn serialize_associated_data(
        &self,
        directory_path: &Path,
        data: &Self::AssociatedData,
    ) -> Result<()>;

    fn cleanup(&self, files_to_delete: &mut Vec<PathBuf>, folders_to_delete: &mut Vec<PathBuf>);
}
