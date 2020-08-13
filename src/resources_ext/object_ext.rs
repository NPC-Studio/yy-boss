use crate::{
    Resource, SerializedData, SerializedDataError, YyResource, YyResourceHandler, YypBoss,
};
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

    fn get_handler(yyp_boss: &YypBoss) -> &YyResourceHandler<Self> {
        &yyp_boss.objects
    }

    fn get_handler_mut(yyp_boss: &mut YypBoss) -> &mut YyResourceHandler<Self> {
        &mut yyp_boss.objects
    }

    fn deserialize_associated_data(
        &self,
        directory_path: Option<&Path>,
        data: SerializedData,
    ) -> Result<Self::AssociatedData, SerializedDataError> {
        let deserialized = match data {
            SerializedData::Value { data } => serde_json::from_str(&data)?,
            SerializedData::Filepath { data } => {
                if let Some(directory_path) = directory_path {
                    let directory_path = directory_path.join(data);
                    let mut value = Self::AssociatedData::new();

                    for event in &self.event_list {
                        let (output, last_number) = event.event_type.filename();
                        let path = directory_path.join(&format!("{}{}", output, last_number));
                        let code = std::fs::read_to_string(&path)
                            .map_err(SerializedDataError::CouldNotReadFile)?;

                        value.insert(event.event_type, code);
                    }

                    value
                } else {
                    return Err(SerializedDataError::NoFileMode);
                }
            }
            SerializedData::DefaultValue => Self::AssociatedData::new(),
        };

        Ok(deserialized)
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

    fn serialize_associated_data_into_data(
        &self,
        our_directory: &Path,
        _: Option<&Path>,
        associated_data: Option<&Self::AssociatedData>,
    ) -> Result<SerializedData, SerializedDataError> {
        let data = if let Some(data) = associated_data {
            serde_json::to_string_pretty(data)?
        } else {
            let data = self.deserialize_associated_data(
                Some(our_directory),
                SerializedData::Filepath {
                    data: PathBuf::default(),
                },
            )?;

            serde_json::to_string_pretty(&data)?
        };

        Ok(SerializedData::Value { data })
    }

    fn cleanup_on_replace(&self, files_to_delete: &mut Vec<PathBuf>, _: &mut Vec<PathBuf>) {
        for event in self.event_list.iter() {
            let (output, last_number) = event.event_type.filename();
            let path = Path::new(&format!("{}{}", output, last_number)).to_path_buf();
            files_to_delete.push(path);
        }
    }
}
