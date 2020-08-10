use crate::{Resource, SerializedData, YyResource, YyResourceHandler, YypBoss};
use std::path::Path;
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

    fn get_handler(yyp_boss: &mut YypBoss) -> &mut YyResourceHandler<Self> {
        &mut yyp_boss.scripts
    }

    fn deserialize_associated_data(
        &self,
        directory_path: Option<&Path>,
        data: SerializedData,
    ) -> anyhow::Result<Self::AssociatedData> {
        let data = data.read_data_as_file(directory_path)?;
        Ok(data)
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
