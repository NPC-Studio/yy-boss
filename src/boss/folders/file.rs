use super::{ResourceDescriptor, ResourceNames};
use crate::{Resource, YyResource};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use yy_typings::FilesystemPath;

#[derive(Serialize, Deserialize, Default, Debug, Eq, PartialEq, Clone, Hash, Ord, PartialOrd)]
pub struct Files(Vec<FilesystemPath>);

impl Files {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_vec(fpaths: Vec<FilesystemPath>) -> Self {
        Self(fpaths)
    }

    pub fn contains_name(&self, name: &str) -> bool {
        self.0.iter().any(|f| f.name == *name)
    }

    pub fn load_in<T: YyResource>(&mut self, yy: &T, order: usize, rn: &mut ResourceNames) {
        self.0
            .push(FilesystemPath::new(T::SUBPATH_NAME, yy.name()));

        // add to resource names...
        rn.load_in_resource(
            yy.name().to_string(),
            ResourceDescriptor::new(T::RESOURCE, order, yy.parent_view_path().path),
        );
    }

    pub fn add<T: YyResource>(&mut self, yy: &T, order: usize, rn: &mut ResourceNames) {
        self.attach(FilesystemPath::new(T::SUBPATH_NAME, yy.name()));

        // add to resource names...
        rn.insert(
            yy.name().to_string(),
            ResourceDescriptor::new(T::RESOURCE, order, yy.parent_view_path().path),
        );
    }

    pub fn drain_into(
        &mut self,
        rn: &mut ResourceNames,
        buf: &mut HashMap<FilesystemPath, ResourceDescriptor>,
    ) {
        for file in self.0.drain(..) {
            let v = rn.remove(&file.name).unwrap();
            buf.insert(file, v);
        }
    }

    pub fn remove(&mut self, name: &str, rn: &mut ResourceNames) {
        self.detach(name);
        rn.remove(name);
    }

    pub fn edit_name(
        &mut self,
        name: &str,
        new_name: String,
        resource: Resource,
        rn: &mut ResourceNames,
    ) {
        // rename our own thing...
        if let Some(fpath) = self.0.iter_mut().find(|v| v.name == name) {
            *fpath = FilesystemPath::new(resource.subpath_name(), &new_name);
        }

        // remove the old name...
        if let Some(resource_desc) = rn.remove(name) {
            rn.insert(new_name, resource_desc);
        }
    }

    pub fn attach(&mut self, fsyspath: FilesystemPath) {
        self.0.push(fsyspath);
    }

    pub fn detach(&mut self, name: &str) -> Option<FilesystemPath> {
        self.0
            .iter()
            .position(|v| v.name == name)
            .map(|p| self.0.remove(p))
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn inner(&self) -> &Vec<FilesystemPath> {
        &self.0
    }
}
