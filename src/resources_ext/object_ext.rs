// use crate::{Resource, YyResource};
// use std::{collections::HashMap, path::PathBuf};
// use yy_typings::{sprite_yy::object_yy::*, FilesystemPath, ViewPath};

// impl YyResource for Object {
//     type AssociatedData = HashMap<EventType, String>;
//     const SUBPATH_NAME: &'static str = "objects";
//     const RESOURCE: Resource = Resource::Object;

//     fn name(&self) -> &str {
//         &self.name
//     }
//     fn set_name(&mut self, name: String) {
//         self.name = name;
//     }
//     fn parent_path(&self) -> ViewPath {
//         self.parent.clone()
//     }

//     fn deserialize_associated_data(
//         &self,
//         project_directory: &std::path::Path,
//     ) -> anyhow::Result<Option<Self::AssociatedData>> {
//         // for event in &self.event_list {
//         //     let (output, last_number) = event.event_type.filename();
//         //     compile_error!("Jack, we're taking a god damn stand, and `deserialize` associated data is going to point to the directory, dammit");
//         // }

//         // Ok(())

//         // let value = std::fs::read_to_string(&script_gml_path)?;
//         Ok(Some(value))
//     }

//     fn serialize_associated_data(
//         &self,
//         directory_path: &std::path::Path,
//         data: &Self::AssociatedData,
//     ) -> anyhow::Result<()> {
//         // let mut file = directory_path.join(&self.name);
//         // file.set_extension(".gml");

//         // std::fs::write(file, data)?;

//         Ok(())
//     }

//     fn cleanup_on_replace(
//         &self,
//         files_to_delete: &mut Vec<PathBuf>,
//         folders_to_delete: &mut Vec<PathBuf>,
//     ) {
//     }
// }
