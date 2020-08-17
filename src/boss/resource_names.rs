use crate::Resource;
use std::collections::{HashMap, HashSet};
use yy_typings::{FilesystemPath, Yyp, YypResource};

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
            self.to_remove.insert(name.to_string(), output);
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

    pub(crate) fn serialize(&mut self, yyp: &mut Yyp) {
        for refried_bean in self.to_serialize.drain() {
            let desc = &self.names[&refried_bean];
            if let Some(pos) = yyp.resources.iter().position(|v| v.id.name == refried_bean) {
                yyp.resources[pos] = desc.to_yyp_resource(&refried_bean);
            } else {
                yyp.resources.push(desc.to_yyp_resource(&refried_bean));
            }
        }

        for (name, _) in self.to_remove.drain() {
            if let Some(pos) = yyp.resources.iter().position(|v| v.id.name == name) {
                yyp.resources.remove(pos);
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd, Ord, Hash)]
pub struct ResourceDescriptor {
    pub resource: Resource,
    pub order: usize,
}

impl ResourceDescriptor {
    pub fn new(resource: Resource, order: usize) -> Self {
        Self { resource, order }
    }

    pub fn to_yyp_resource(&self, name: &str) -> YypResource {
        YypResource {
            id: FilesystemPath::new(self.resource.base_name(), &name),
            order: self.order,
        }
    }
}
