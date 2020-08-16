use crate::{
    utils, AssocDataLocation, Resource, SerializedData, SerializedDataError, YyResource,
    YyResourceHandler, YypBoss,
};
use std::path::Path;
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
    fn parent_path(&self) -> ViewPath {
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
            AssocDataLocation::Value(v) => serde_json::from_str(v).map_err(|e| e.into()),
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

    // fn serialize_associated_data_into_data(
    //     &self,
    //     our_directory: &Path,
    //     _: Option<&Path>,
    //     associated_data: Option<&Self::AssociatedData>,
    // ) -> Result<SerializedData, SerializedDataError> {
    //     if let Some(data) = associated_data {
    //         Ok(SerializedData::Value {
    //             data: data.to_owned(),
    //         })
    //     } else {
    //         let data = self.deserialize_associated_data(Some(our_directory))?;

    //         Ok(SerializedData::Value { data })
    //     }
    // }

    fn cleanup_on_replace(&self, _: &mut Vec<std::path::PathBuf>, _: &mut Vec<std::path::PathBuf>) {
        // not much to clean up here which won't get rewritten by a replace op!
    }
}
