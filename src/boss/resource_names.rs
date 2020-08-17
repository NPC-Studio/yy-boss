use crate::Resource;
use std::collections::{HashMap, HashSet};
use yy_typings::Yyp;

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct ResourceNames {
    names: HashMap<String, ResourceDescriptor>,
    to_serialize: HashSet<String>,
    to_remove: HashSet<ResourceDescriptor>,
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
        self.to_remove.remove(&resource);

        self.names.insert(name, resource)
    }

    pub(crate) fn remove(&mut self, name: &str) -> Option<ResourceDescriptor> {
        // just in case
        self.to_serialize.remove(name);

        if let Some(output) = self.names.remove(name) {
            self.to_remove.insert(output);
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

    pub(crate) fn serialize(&self, yyp: &mut Yyp) -> anyhow::Result<bool> {
        unimplemented!()
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
}
