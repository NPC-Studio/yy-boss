use crate::{Resource, YyResource};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use yy_typings::{sprite_yy::object_yy::*, ViewPath};

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

    fn deserialize_associated_data(
        &self,
        directory_path: &std::path::Path,
    ) -> anyhow::Result<Option<Self::AssociatedData>> {
        let mut value = Self::AssociatedData::new();

        for event in &self.event_list {
            let (output, last_number) = event.event_type.filename();
            let path = directory_path.join(&format!("{}{}", output, last_number));
            let code = std::fs::read_to_string(&path)?;

            value.insert(event.event_type, code);
        }

        Ok(Some(value))
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

    fn cleanup_on_replace(&self, files_to_delete: &mut Vec<PathBuf>, _: &mut Vec<PathBuf>) {
        for event in self.event_list.iter() {
            let (output, last_number) = event.event_type.filename();
            let path = Path::new(&format!("{}{}", output, last_number)).to_path_buf();
            files_to_delete.push(path);
        }
    }
}
