use super::{ResourceDescriptor, ResourceNames};
use crate::YyResource;
use serde::{Deserialize, Serialize};
use yy_typings::FilesystemPath;

#[derive(Serialize, Deserialize, Default, Debug, Eq, PartialEq, Clone, Hash, Ord, PartialOrd)]
pub struct Files(Vec<FilesystemPath>);

impl Files {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn contains_name(&self, name: &str) -> bool {
        self.0.iter().any(|f| f.name == *name)
    }

    pub fn load_in<T: YyResource>(&mut self, yy: &T, order: usize, rn: &mut ResourceNames) {
        self.0
            .push(FilesystemPath::new(T::SUBPATH_NAME, &yy.name()));
        self.0.sort_unstable();

        // add to resource names...
        rn.load_in_resource(
            yy.name().to_string(),
            ResourceDescriptor::new(T::RESOURCE, order, yy.parent_view_path().path),
        );
    }

    pub fn add<T: YyResource>(&mut self, yy: &T, order: usize, rn: &mut ResourceNames) {
        self.0
            .push(FilesystemPath::new(T::SUBPATH_NAME, &yy.name()));
        self.0.sort_unstable();

        // add to resource names...
        rn.insert(
            yy.name().to_string(),
            ResourceDescriptor::new(T::RESOURCE, order, yy.parent_view_path().path),
        );
    }

    pub fn remove(&mut self, name: &str, rn: &mut ResourceNames) {
        if let Some(pos) = self.0.iter().position(|v| v.name == name) {
            self.0.remove(pos);
        }

        rn.remove(name);
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
