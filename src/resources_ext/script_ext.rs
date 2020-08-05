use crate::{Resource, YyResource};
use yy_typings::{sprite_yy::script::Script, ViewPath};

impl YyResource for Script {
    type AssociatedData = String;
    const SUBPATH_NAME: &'static str = "scripts";
    const RESOURCE: Resource = Resource::Script;

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
        let file = directory_path.join(&self.name).with_extension("gml");
        let value = std::fs::read_to_string(&file)?;

        Ok(Some(value))
    }
    fn serialize_associated_data(
        &self,
        directory_path: &std::path::Path,
        data: &Self::AssociatedData,
    ) -> anyhow::Result<()> {
        let file = directory_path.join(&self.name).with_extension("gml");
        std::fs::write(file, data)?;

        Ok(())
    }
    fn cleanup_on_replace(&self, _: &mut Vec<std::path::PathBuf>, _: &mut Vec<std::path::PathBuf>) {
        // not much to clean up here which won't get rewritten by a replace op!
    }
}
