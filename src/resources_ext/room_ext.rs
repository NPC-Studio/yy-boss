use yy_typings::{Room, ViewPath};

use crate::{SerializedData, YyResource, YyResourceHandler, YypBoss};

impl YyResource for Room {
    type AssociatedData = ();

    const SUBPATH_NAME: &'static str = "rooms";

    const RESOURCE: crate::Resource = crate::Resource::Room;

    fn name(&self) -> &str {
        &self.resource_data.name
    }

    fn set_name(&mut self, name: String) {
        self.resource_data.name = name;
    }

    fn set_parent_view_path(&mut self, vp: yy_typings::ViewPath) {
        self.resource_data.parent = vp;
    }

    fn parent_view_path(&self) -> ViewPath {
        self.resource_data.parent.clone()
    }

    fn get_handler(yyp_boss: &YypBoss) -> &YyResourceHandler<Self> {
        &yyp_boss.rooms
    }

    fn get_handler_mut(yyp_boss: &mut YypBoss) -> &mut YyResourceHandler<Self> {
        &mut yyp_boss.rooms
    }

    fn serialize_associated_data(
        &self,
        _directory_path: &std::path::Path,
        _data: &Self::AssociatedData,
    ) -> anyhow::Result<()> {
        // let file = directory_path
        //     .join(&self.resource_data.name)
        //     .with_extension("gml");
        // std::fs::write(file, data)?;

        Ok(())
    }

    fn deserialize_associated_data(
        &self,
        _directory_path: &std::path::Path,
        _tcu: &yy_typings::utils::TrailingCommaUtility,
    ) -> Result<Self::AssociatedData, crate::SerializedDataError> {
        Ok(())
    }

    fn serialize_associated_data_into_data(
        _working_directory: &std::path::Path,
        _associated_data: &Self::AssociatedData,
    ) -> Result<crate::SerializedData, crate::SerializedDataError> {
        Ok(SerializedData::Value {
            data: String::new(),
        })
    }

    fn deserialize_associated_data_from_data(
        &self,
        _incoming_data: &crate::SerializedData,
        _tcu: &yy_typings::utils::TrailingCommaUtility,
    ) -> Result<Self::AssociatedData, crate::SerializedDataError> {
        Ok(())
    }

    fn cleanup_on_replace(&self, _: impl crate::FileHolder) {}
}
