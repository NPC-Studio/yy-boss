use crate::YyResource;
use yy_typings::{sprite_yy::script::Script, FilesystemPath, ViewPath};

impl YyResource for Script {
    type AssociatedData = String;
    const SUBPATH_NAME: &'static str = "scripts";

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
        project_directory: &std::path::Path,
    ) -> anyhow::Result<Option<Self::AssociatedData>> {
        let script_gml_path =
            project_directory.join(&FilesystemPath::new(Self::SUBPATH_NAME, &self.name).path);

        let value = std::fs::read_to_string(&script_gml_path)?;

        Ok(Some(value))
    }
    fn serialize_associated_data(
        &self,
        directory_path: &std::path::Path,
        data: &Self::AssociatedData,
    ) -> anyhow::Result<()> {
        let mut file = directory_path.join(&self.name);
        file.set_extension(".gml");

        std::fs::write(file, data)?;

        Ok(())
    }
    fn cleanup(
        &self,
        files_to_delete: &mut Vec<std::path::PathBuf>,
        folders_to_delete: &mut Vec<std::path::PathBuf>,
    ) {
    }
}
