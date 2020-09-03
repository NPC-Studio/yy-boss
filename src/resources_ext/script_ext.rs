use crate::{
    utils, AssocDataLocation, FileHolder, Resource, SerializedData, SerializedDataError,
    YyResource, YyResourceHandler, YypBoss,
};
use yy_typings::{sprite_yy::script::Script, utils::TrailingCommaUtility, ViewPath};

impl YyResource for Script {
    type AssociatedData = String;
    const SUBPATH_NAME: &'static str = "scripts";
    const RESOURCE: Resource = Resource::Script;

    fn name(&self) -> &str {
        &self.name
    }
    fn set_name(&mut self, name: String) {
        self.name = name;
    }

    fn set_parent_view_path(&mut self, vp: yy_typings::ViewPath) {
        self.parent = vp;
    }

    fn parent_view_path(&self) -> ViewPath {
        self.parent.clone()
    }

    fn get_handler(yyp_boss: &YypBoss) -> &YyResourceHandler<Self> {
        &yyp_boss.scripts
    }

    fn get_handler_mut(yyp_boss: &mut YypBoss) -> &mut YyResourceHandler<Self> {
        &mut yyp_boss.scripts
    }

    fn deserialize_associated_data(
        &self,
        incoming_data: AssocDataLocation<'_>,
        tcu: &TrailingCommaUtility,
    ) -> Result<Self::AssociatedData, SerializedDataError> {
        match incoming_data {
            AssocDataLocation::Value(v) => Ok(v.to_string()),
            AssocDataLocation::Path(v) => utils::deserialize_json_tc(v, tcu).map_err(|e| e.into()),
            AssocDataLocation::Default => Ok(Self::AssociatedData::default()),
        }
    }

    fn serialize_associated_data(
        &self,
        directory_path: &std::path::Path,
        data: &Self::AssociatedData,
    ) -> anyhow::Result<()> {
        let file = directory_path.join(&self.name).with_extension("gml");
        std::fs::write(file, data)?;

        Ok(())
    }

    fn serialize_associated_data_into_data(
        _: &std::path::Path,
        associated_data: &Self::AssociatedData,
    ) -> Result<SerializedData, SerializedDataError> {
        match serde_json::to_string_pretty(associated_data) {
            Ok(data) => Ok(SerializedData::Value { data }),
            Err(e) => Err(e.into()),
        }
    }

    fn cleanup_on_replace(&self, _: impl FileHolder) {}
}
