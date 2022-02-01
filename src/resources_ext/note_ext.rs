use crate::{
    utils, FileHolder, Resource, SerializedData, SerializedDataError, YyResource,
    YyResourceHandler, YypBoss,
};
use std::path::Path;
use yy_typings::{utils::TrailingCommaUtility, Note, ViewPath};

impl YyResource for Note {
    type AssociatedData = String;

    const SUBPATH_NAME: &'static str = "notes";
    const RESOURCE: Resource = Resource::Note;

    fn name(&self) -> &str {
        &self.resource_data.name
    }

    fn set_name(&mut self, name: String) {
        self.resource_data.name = name;
    }

    fn set_parent_view_path(&mut self, vp: ViewPath) {
        self.resource_data.parent = vp;
    }

    fn parent_view_path(&self) -> ViewPath {
        self.resource_data.parent.clone()
    }

    fn get_handler(yyp_boss: &YypBoss) -> &YyResourceHandler<Self> {
        &yyp_boss.notes
    }

    fn get_handler_mut(yyp_boss: &mut YypBoss) -> &mut YyResourceHandler<Self> {
        &mut yyp_boss.notes
    }

    fn serialize_associated_data(
        &self,
        directory_path: &std::path::Path,
        data: &Self::AssociatedData,
    ) -> anyhow::Result<()> {
        let file = directory_path
            .join(&self.resource_data.name)
            .with_extension("txt");
        std::fs::write(file, data)?;

        Ok(())
    }

    fn deserialize_associated_data(
        &self,
        directory_path: &Path,
        _: &TrailingCommaUtility,
    ) -> Result<Self::AssociatedData, SerializedDataError> {
        let path = directory_path.join(format!("{}.txt", self.resource_data.name));

        std::fs::read_to_string(path).map_err(|e| {
            SerializedDataError::CouldNotDeserializeFile(crate::FileSerializationError::Io(
                e.to_string(),
            ))
        })
    }

    fn serialize_associated_data_into_data(
        _: &std::path::Path,
        associated_data: &Self::AssociatedData,
    ) -> Result<SerializedData, SerializedDataError> {
        Ok(SerializedData::Value {
            data: associated_data.clone(),
        })
    }

    fn deserialize_associated_data_from_data(
        &self,
        incoming_data: &SerializedData,
        tcu: &TrailingCommaUtility,
    ) -> Result<Self::AssociatedData, SerializedDataError> {
        match incoming_data {
            SerializedData::Value { data: v } => Ok(v.to_string()),
            SerializedData::Filepath { data: v } => {
                utils::deserialize_json_tc(v, tcu).map_err(|e| e.into())
            }
            SerializedData::DefaultValue => Ok(Self::AssociatedData::default()),
        }
    }

    fn cleanup_on_replace(&self, _: impl FileHolder) {}
}
