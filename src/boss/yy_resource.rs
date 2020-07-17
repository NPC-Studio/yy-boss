use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, path::Path};
use yy_typings::{FilesystemPath, ViewPath};

pub trait YyResource: Serialize + for<'de> Deserialize<'de> {
    type AssociatedData: Debug;

    /// Get's the resource's name.
    fn name(&self) -> &str;

    /// Sets the name of the resource.
    fn set_name(&mut self, name: String);

    /// Get the relative filepath from the directory of the YYP
    /// to the resource yy file. For a sprite called `spr_player`,
    /// that path would be `sprites/spr_player/spr_player.yy`.
    fn filesystem_path(&self) -> FilesystemPath;

    fn parent_path(&self) -> ViewPath;

    fn load_associated_data(
        &self,
        project_directory: &Path,
    ) -> Result<Option<Self::AssociatedData>>;

    fn serialize_associated_data(
        &self,
        directory_path: &Path,
        data: &Self::AssociatedData,
    ) -> Result<()>;
}

#[derive(Serialize, Deserialize, Default, Debug, Eq, PartialEq, Clone, Hash, Ord, PartialOrd)]
pub struct CreatedResource(pub(crate) FilesystemPath, pub(crate) usize);
