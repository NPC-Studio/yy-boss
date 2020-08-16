use crate::{
    utils, AssocDataLocation, FileSerializationError, Resource, SerializedData,
    SerializedDataError, YyResource, YyResourceHandler, YypBoss,
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use yy_typings::{sprite_yy::object_yy::*, utils::TrailingCommaUtility, ViewPath};

impl YyResource for Object {
    type AssociatedData = HashMap<EventType, String>;
    const SUBPATH_NAME: &'static str = "objects";
    const RESOURCE: Resource = Resource::Object;

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
        &yyp_boss.objects
    }

    fn get_handler_mut(yyp_boss: &mut YypBoss) -> &mut YyResourceHandler<Self> {
        &mut yyp_boss.objects
    }

    fn deserialize_associated_data(
        &self,
        incoming_data: AssocDataLocation<'_>,
        tcu: &TrailingCommaUtility,
    ) -> Result<Self::AssociatedData, SerializedDataError> {
        match incoming_data {
            AssocDataLocation::Value(v) => serde_json::from_str(v).map_err(|e| e.into()),
            AssocDataLocation::Path(p) => utils::deserialize_json_tc(p, tcu).map_err(|e| e.into()),
            AssocDataLocation::Default => Ok(HashMap::new()),
        }
    }

    fn serialize_associated_data(
        &self,
        directory_path: &std::path::Path,
        data: &Self::AssociatedData,
    ) -> anyhow::Result<()> {
        for (event_type, code) in data {
            let (output, last_number) = event_type.filename();
            let path = directory_path.join(&format!("{}{}", output, last_number));

            std::fs::write(&path, code)?;
        }

        Ok(())
    }

    // fn serialize_associated_data_into_data(
    //     &self,
    //     our_directory: &Path,
    //     _: Option<&Path>,
    //     associated_data: Option<&Self::AssociatedData>,
    // ) -> Result<SerializedData, SerializedDataError> {
    //     let data = if let Some(data) = associated_data {
    //         serde_json::to_string_pretty(data)?
    //     } else {
    //         let data =
    //             self.deserialize_associated_data(AssocDataLocation::Path(our_directory), tcu)?;

    //         serde_json::to_string_pretty(&data)?
    //     };

    //     Ok(SerializedData::Value { data })
    // }

    fn cleanup_on_replace(&self, files_to_delete: &mut Vec<PathBuf>, _: &mut Vec<PathBuf>) {
        for event in self.event_list.iter() {
            let (output, last_number) = event.event_type.filename();
            let path = Path::new(&format!("{}{}", output, last_number)).to_path_buf();
            files_to_delete.push(path);
        }
    }
}
