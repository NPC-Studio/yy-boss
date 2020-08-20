use super::YyResource;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub struct DummyResource(String, usize);

impl DummyResource {
    #[allow(dead_code)]
    pub fn new<S: AsRef<str>>(name: S, id: usize) -> Self {
        Self(name.as_ref().to_owned(), id)
    }
}

impl YyResource for DummyResource {
    type AssociatedData = ();
    const SUBPATH_NAME: &'static str = "dummy";
    const RESOURCE: crate::Resource = crate::Resource::Script;

    fn name(&self) -> &str {
        &self.0
    }
    fn set_name(&mut self, name: String) {
        self.0 = name;
    }

    fn parent_view_path(&self) -> yy_typings::ViewPath {
        unimplemented!()
    }

    fn get_handler(_: &crate::YypBoss) -> &crate::YyResourceHandler<Self> {
        unimplemented!()
    }

    fn get_handler_mut(_: &mut crate::YypBoss) -> &mut crate::YyResourceHandler<Self> {
        unimplemented!()
    }

    fn deserialize_associated_data(
        &self,
        _: crate::AssocDataLocation<'_>,
        _: &yy_typings::utils::TrailingCommaUtility,
    ) -> Result<Self::AssociatedData, crate::SerializedDataError> {
        Ok(())
    }
    fn serialize_associated_data(&self, _: &Path, _: &Self::AssociatedData) -> anyhow::Result<()> {
        Ok(())
    }
    fn serialize_associated_data_into_data(
        _: &Path,
        _: &Self::AssociatedData,
    ) -> Result<crate::SerializedData, crate::SerializedDataError> {
        Ok(crate::SerializedData::Value {
            data: String::new(),
        })
    }
    fn cleanup_on_replace(
        &self,
        files: &mut Vec<std::path::PathBuf>,
        folders: &mut Vec<std::path::PathBuf>,
    ) {
        files.push(Path::new(&format!("{}/{}.txt", self.0, self.1)).to_owned());
        folders.push(Path::new(&format!("{}/{}", self.0, self.1)).to_owned());
    }
}
