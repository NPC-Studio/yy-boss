use crate::Resource;
use std::collections::{HashMap, HashSet};
use yy_typings::{FilesystemPath, ViewPathLocation, YypResource};

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct ResourceNames {
    names: HashMap<String, ResourceDescriptor>,
    to_serialize: HashSet<String>,
    to_remove: HashMap<String, ResourceDescriptor>,
}

impl ResourceNames {
    pub(crate) fn new() -> Self {
        ResourceNames::default()
    }

    pub(crate) fn load_in_resource(&mut self, name: String, resource: ResourceDescriptor) {
        self.names.insert(name, resource);
    }

    pub(crate) fn insert(
        &mut self,
        name: String,
        resource: ResourceDescriptor,
    ) -> Option<ResourceDescriptor> {
        self.to_serialize.insert(name.clone());
        // just in case...
        self.to_remove.remove(&name);

        self.names.insert(name, resource)
    }

    pub(crate) fn remove(&mut self, name: &str) -> Option<ResourceDescriptor> {
        // just in case
        self.to_serialize.remove(name);

        if let Some(output) = self.names.remove(name) {
            self.to_remove.insert(name.to_string(), output.clone());
            Some(output)
        } else {
            None
        }
    }

    pub fn get<'a>(&'a self, name: &str) -> Option<&'a ResourceDescriptor> {
        self.names.get(name)
    }

    /// Returns all the currently known names and descriptors in the project.
    pub fn get_all(&self) -> &HashMap<String, ResourceDescriptor> {
        &self.names
    }

    pub(crate) fn serialize(&mut self, yyp_resources: &mut Vec<YypResource>) {
        for refried_bean in self.to_serialize.drain() {
            let desc = &self.names[&refried_bean];
            if let Some(pos) = yyp_resources.iter().position(|v| v.id.name == refried_bean) {
                yyp_resources[pos] = desc.to_yyp_resource(&refried_bean);
            } else {
                yyp_resources.push(desc.to_yyp_resource(&refried_bean));
            }
        }

        for (name, _) in self.to_remove.drain() {
            if let Some(pos) = yyp_resources.iter().position(|v| v.id.name == name) {
                yyp_resources.remove(pos);
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord, Hash)]
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
