use crate::{
    FileHolder, FileSerializationError, Resource, SerializedData, SerializedDataError, YyResource,
    YyResourceHandler, YypBoss,
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

    /// This implementation is seriously flawed, but is necesssary to support the GmCode
    /// downstream crate,
    fn serialize_associated_data(
        &self,
        directory_path: &std::path::Path,
        data: &HashMap<EventType, String>,
    ) -> anyhow::Result<()> {
        let mut allowed_files = std::collections::HashSet::with_capacity(1 + self.event_list.len());
        allowed_files.insert(directory_path.join(format!("{}.yy", self.name)));

        for event_type in self.event_list.iter().map(|v| v.event_type) {
            if let Some(gml) = data.get(&event_type) {
                let path = directory_path.join(format!("{}.gml", event_type.filename_simple()));
                if path.exists() == false {
                    log::info!("writing {} to {}", gml, path.display());
                    std::fs::write(&path, gml)?;
                }

                allowed_files.insert(path);
            } else {
                log::error!("we couldn't find a {} in our associated data, even though it should have been there. not serialized.", event_type);
            }
        }

        let files: std::collections::HashSet<std::path::PathBuf> = directory_path
            .read_dir()
            .map(|d| {
                d.filter_map(|f| f.map(|file_data| file_data.path()).ok())
                    .collect()
            })
            .unwrap_or_default();

        for badfile in files.difference(&allowed_files) {
            log::info!("removing {}", badfile.display());
            std::fs::remove_file(badfile)?;
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
            let path = directory_path.join(format!("{}.gml", event_type.filename_simple()));

            let val = std::fs::read_to_string(&path).map_err(|e| {
                SerializedDataError::CouldNotDeserializeFile(FileSerializationError::Io(
                    e.to_string(),
                ))
            })?;
            associated_data.insert(event_type, val);
        }

        Ok(associated_data)
    }

    fn serialize_associated_data_into_data(
        safe_dir: &Path,
        associated_data: &HashMap<EventType, String>,
    ) -> Result<SerializedData, SerializedDataError> {
        let simple_map: HashMap<String, String> = associated_data
            .iter()
            .map(|(k, v)| (k.filename_simple(), v.clone()))
            .collect();

        let uuid = uuid::Uuid::new_v4().to_string();
        let path = safe_dir.join(uuid);

        let pretty_string =
            serde_json::to_string(&simple_map).expect("serde failed for object deserialization");

        match std::fs::write(&path, pretty_string) {
            Ok(()) => Ok(SerializedData::Filepath { data: path }),
            Err(e) => Err(SerializedDataError::InnerError(e.to_string())),
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
                        SerializedDataError::CouldNotDeserializeFile(FileSerializationError::Io(
                            e.to_string(),
                        ))
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
            let path =
                Path::new(&format!("{}.gml", event.event_type.filename_simple())).to_path_buf();
            files_to_delete.push(path);
        }
    }
}

impl YyResourceHandler<Object> {
    pub fn add_event(&mut self, identifier: &str, event_type: EventType) -> bool {
        let output = unsafe { self.get_mut(identifier).unwrap() };
        let events: &mut HashMap<EventType, String> = output.associated_data.as_mut().unwrap();

        if output
            .yy_resource
            .event_list
            .iter()
            .any(|v| v.event_type == event_type)
            == false
        {
            events.insert(event_type, String::new());
            output.yy_resource.event_list.push(ObjectEvent {
                is_dn_d: false,
                event_type,
                collision_object_id: None,
                resource_version: ResourceVersion::default(),
                name: None,
                tags: Tags::new(),
                resource_type: ConstGmEvent::Const,
            });

            // mark it an serialize...we know this is infallible
            self.force_serialize(identifier).unwrap();

            true
        } else {
            false
        }
    }

    pub fn remove_event(&mut self, identifier: &str, event_type: EventType) -> bool {
        let output = unsafe { self.get_mut(identifier).unwrap() };

        if let Some(v) = output
            .yy_resource
            .event_list
            .iter()
            .position(|v| v.event_type == event_type)
        {
            output.yy_resource.event_list.remove(v);
            output.associated_data.as_mut().unwrap().remove(&event_type);

            // mark it an serialize...we know this is infallible
            self.force_serialize(identifier).unwrap();

            true
        } else {
            false
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
