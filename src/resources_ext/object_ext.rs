use crate::{
    FileHolder, Resource, SerializedData, SerializedDataError, YyResource, YyResourceHandler,
    YypBoss,
};

use std::{collections::HashMap, path::Path};
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

    fn set_parent_view_path(&mut self, vp: yy_typings::ViewPath) {
        self.parent = vp;
    }

    fn parent_view_path(&self) -> ViewPath {
        self.parent.clone()
    }

    fn get_handler(yyp_boss: &YypBoss) -> &YyResourceHandler<Self> {
        &yyp_boss.objects
    }

    fn get_handler_mut(yyp_boss: &mut YypBoss) -> &mut YyResourceHandler<Self> {
        &mut yyp_boss.objects
    }

    fn serialize_associated_data(
        &self,
        directory_path: &std::path::Path,
        data: &HashMap<EventType, String>,
    ) -> anyhow::Result<()> {
        for event_type in self.event_list.iter().map(|v| v.event_type) {
            if let Some(gml) = data.get(&event_type) {
                let (output, last_number) = event_type.filename();
                let path = directory_path.join(&format!("{}{}", output, last_number));
                std::fs::write(&path, gml)?;
            } else {
                log::error!("we couldn't find a {} in our associated data, even though it should have been there. not serialized.", event_type);
            }
        }

        Ok(())
    }

    fn deserialize_associated_data(
        &self,
        directory_path: &Path,
        _: &TrailingCommaUtility,
    ) -> Result<Self::AssociatedData, SerializedDataError> {
        let mut associated_data = HashMap::new();

        for event_type in self.event_list.iter().map(|v| v.event_type) {
            let (output, last_number) = event_type.filename();
            let path = directory_path.join(&format!("{}{}", output, last_number));

            let val = std::fs::read_to_string(&path).map_err(|e| {
                SerializedDataError::CouldNotDeserializeFile(crate::FileSerializationError::Io(
                    e.to_string(),
                ))
            })?;
            associated_data.insert(event_type, val);
        }

        Ok(associated_data)
    }

    fn serialize_associated_data_into_data(
        _: &Path,
        associated_data: &HashMap<EventType, String>,
    ) -> Result<SerializedData, SerializedDataError> {
        let simple_map: HashMap<String, String> = associated_data
            .iter()
            .map(|(k, v)| {
                let (key, numb) = k.filename();

                (format!("{}{}", key, numb), v.clone())
            })
            .collect();

        match serde_json::to_string_pretty(&simple_map) {
            Ok(data) => Ok(SerializedData::Value { data }),
            Err(e) => Err(e.into()),
        }
    }

    fn deserialize_associated_data_from_data(
        &self,
        incoming_data: &SerializedData,
        tcu: &TrailingCommaUtility,
    ) -> Result<Self::AssociatedData, SerializedDataError> {
        fn deserialize_simple_value(
            v: &str,
        ) -> Result<HashMap<EventType, String>, SerializedDataError> {
            let simple_map: HashMap<String, String> =
                serde_json::from_str(v).map_err(SerializedDataError::from)?;

            let output = simple_map
                .into_iter()
                .map(|(k, v)| {
                    let mut iter = k.split('_');
                    let name = iter.next().unwrap();
                    let value: usize = iter.next().unwrap().parse().unwrap_or_default();
                    assert_eq!(iter.next(), None);

                    let event_type = EventType::parse_filename(name, value).unwrap();

                    (event_type, v)
                })
                .collect();

            Ok(output)
        }

        let mut hmap = match incoming_data {
            SerializedData::Value { data: v } => deserialize_simple_value(v),
            SerializedData::Filepath { data: p } => {
                if p.is_dir() {
                    self.deserialize_associated_data(p, tcu)
                } else {
                    let data = std::fs::read_to_string(p).map_err(|e| {
                        SerializedDataError::CouldNotDeserializeFile(
                            crate::FileSerializationError::Io(e.to_string()),
                        )
                    })?;
                    deserialize_simple_value(&data)
                }
            }
            SerializedData::DefaultValue => {
                let mut output = HashMap::new();

                for event_required in self.event_list.iter().map(|v| v.event_type) {
                    output.insert(event_required, String::new());
                }

                Ok(output)
            }
        }?;

        // first check if there are any excess keys...
        // and sheer them off. I don't think we need an error for this.
        hmap = hmap
            .into_iter()
            .filter(|(key, _)| self.event_list.iter().any(|v| v.event_type == *key))
            .collect();

        // second, check if there are any missing keys...
        for event_required in self.event_list.iter().map(|v| v.event_type) {
            if hmap.get(&event_required).is_none() {
                return Err(SerializedDataError::BadData(format!(
                    "missing event {}",
                    event_required
                )));
            }
        }

        Ok(hmap)
    }

    fn cleanup_on_replace(&self, mut files_to_delete: impl FileHolder) {
        for event in self.event_list.iter() {
            let (output, last_number) = event.event_type.filename();
            let path = Path::new(&format!("{}{}", output, last_number)).to_path_buf();
            files_to_delete.push(path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn serialization() {
        let mut hmap: HashMap<EventType, String> = HashMap::new();

        hmap.insert(EventType::Create, "oh hello".to_string());
        let simple_map: HashMap<String, String> = hmap
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect();

        serde_json::to_string(&simple_map).unwrap();
    }
}
