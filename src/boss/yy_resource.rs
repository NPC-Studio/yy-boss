use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, path::Path};
use yy_typings::ViewPath;

pub trait YyResource: Serialize + for<'de> Deserialize<'de> + Clone {
    type AssociatedData: Debug;
    const SUBPATH_NAME: &'static str;

    /// Get's the resource's name.
    fn name(&self) -> &str;

    /// Sets the name of the resource.
    fn set_name(&mut self, name: String);

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
