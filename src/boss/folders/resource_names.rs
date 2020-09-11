use super::ResourceNameError;
use crate::{
    boss::dirty_handler::{DirtyDrain, DirtyHandler},
    Resource,
};
use std::collections::HashMap;
use yy_typings::{FilesystemPath, ViewPathLocation, YypResource};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ResourceNames {
    names: HashMap<String, ResourceDescriptor>,
    dirty_handler: DirtyHandler<String>,
}

impl ResourceNames {
    pub(crate) fn new() -> Self {
        Self {
            names: HashMap::new(),
            dirty_handler: DirtyHandler::new(),
        }
    }

    pub(crate) fn load_in_resource(&mut self, name: String, resource: ResourceDescriptor) {
        self.names.insert(name, resource);
    }

    pub(crate) fn insert(
        &mut self,
        name: String,
        resource: ResourceDescriptor,
    ) -> Option<ResourceDescriptor> {
        if let Some(ret) = self.names.insert(name.clone(), resource) {
            self.dirty_handler.edit(name);
            Some(ret)
        } else {
            self.dirty_handler.add(name);
            None
        }
    }

    pub(crate) fn remove(&mut self, name: &str) -> Option<ResourceDescriptor> {
        if let Some(output) = self.names.remove(name) {
            self.dirty_handler.remove(name);
            Some(output)
        } else {
            None
        }
    }

    pub fn get(&self, name: &str) -> Option<&ResourceDescriptor> {
        self.names.get(name)
    }

    /// Returns all the currently known names and descriptors in the project.
    pub fn inner(&self) -> &HashMap<String, ResourceDescriptor> {
        &self.names
    }

    pub(crate) fn get_checked(
        &self,
        name: &str,
        r: Resource,
    ) -> Result<&ResourceDescriptor, ResourceNameError> {
        match self.get(name) {
            Some(v) => {
                if v.resource == r {
                    Ok(v)
                } else {
                    Err(ResourceNameError::BadResourceName {
                        existing_resource: r,
                    })
                }
            }
            None => Err(ResourceNameError::NoResourceByThatName),
        }
    }

    pub(crate) fn serialize(&mut self, yyp_resources: &mut Vec<YypResource>) {
        let DirtyDrain {
            resources_to_reserialize,
            resources_to_remove,
            associated_values: _,
        } = self.dirty_handler.drain_all();

        for (refried_bean, _) in resources_to_reserialize {
            let desc = &self.names[&refried_bean];

            if let Some(pos) = yyp_resources.iter().position(|v| v.id.name == refried_bean) {
                yyp_resources[pos] = desc.to_yyp_resource(&refried_bean);
            } else {
                yyp_resources.push(desc.to_yyp_resource(&refried_bean));
            }
        }

        for (name, _) in resources_to_remove {
            if let Some(pos) = yyp_resources.iter().position(|v| v.id.name == name) {
                yyp_resources.remove(pos);
            }
        }
    }
}

#[derive(
    Debug, PartialEq, Eq, Clone, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub struct ResourceDescriptor {
    pub resource: Resource,
    pub order: usize,
    pub parent_location: ViewPathLocation,
}

impl ResourceDescriptor {
    pub fn new(resource: Resource, order: usize, view_path_location: ViewPathLocation) -> Self {
        Self {
            resource,
            order,
            parent_location: view_path_location,
        }
    }

    pub fn to_yyp_resource(&self, name: &str) -> YypResource {
        YypResource {
            id: FilesystemPath::new(self.resource.base_name(), &name),
            order: self.order,
        }
    }
}
