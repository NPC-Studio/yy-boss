use crate::{Resource, SerializedData, YyResource, YyResourceHandler, YypBoss};
use yy_typings::{TileSet, ViewPath};

impl YyResource for TileSet {
    type AssociatedData = ();

    const SUBPATH_NAME: &'static str = "tilesets";
    const RESOURCE: Resource = Resource::TileSet;

    fn name(&self) -> &str {
        &self.common_data.name
    }

    fn set_name(&mut self, name: String) {
        self.common_data.name = name;
    }

    fn set_parent_view_path(&mut self, vp: ViewPath) {
        self.parent = vp;
    }

    fn parent_view_path(&self) -> ViewPath {
        self.parent.clone()
    }

    fn get_handler(yyp_boss: &YypBoss) -> &YyResourceHandler<Self> {
        &yyp_boss.tilesets
    }

    fn get_handler_mut(yyp_boss: &mut YypBoss) -> &mut YyResourceHandler<Self> {
        &mut yyp_boss.tilesets
    }

    fn serialize_associated_data(
        &self,
        _directory_path: &std::path::Path,
        _data: &Self::AssociatedData,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    fn deserialize_associated_data(
        &self,
        _directory_path: &std::path::Path,
        _tcu: &yy_typings::TrailingCommaUtility,
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
        _tcu: &yy_typings::TrailingCommaUtility,
    ) -> Result<Self::AssociatedData, crate::SerializedDataError> {
        Ok(())
    }

    fn cleanup_on_replace(&self, _: impl crate::FileHolder) {}
}
