use crate::{
    FileHolder, Resource, SerializedData, SerializedDataError, YyResource, YyResourceHandler,
    YypBoss,
};
use std::{fs, path::Path};
use yy_typings::{Sound, TrailingCommaUtility, ViewPath};

impl YyResource for Sound {
    type AssociatedData = Vec<u8>;

    const SUBPATH_NAME: &'static str = "sounds";
    const RESOURCE: Resource = Resource::Sound;

    fn name(&self) -> &str {
        &self.common_data.name
    }

    fn set_name(&mut self, name: String) {
        self.common_data.name = name;
    }

    fn set_parent_view_path(&mut self, vp: ViewPath) {
        self.parent = vp;
    }

    fn parent_view_path(&self) -> ViewPath {
        self.parent.clone()
    }

    fn get_handler(yyp_boss: &YypBoss) -> &YyResourceHandler<Self> {
        &yyp_boss.sounds
    }

    fn get_handler_mut(yyp_boss: &mut YypBoss) -> &mut YyResourceHandler<Self> {
        &mut yyp_boss.sounds
    }

    fn serialize_associated_data(
        &self,
        wd: &Path,
        data: &Self::AssociatedData,
    ) -> anyhow::Result<()> {
        // we only write the sound file it's you know...there
        if self.sound_file.is_empty() == false {
            fs::write(wd.join(&self.sound_file), data)?;
        }

        Ok(())
    }

    fn deserialize_associated_data(
        &self,
        wd: &Path,
        _: &TrailingCommaUtility,
    ) -> Result<Self::AssociatedData, SerializedDataError> {
        let data = if self.sound_file.is_empty() {
            // return a blank thingee
            Vec::new()
        } else {
            let path = wd.join(&self.sound_file);
            fs::read(path).map_err(|e| {
                SerializedDataError::CouldNotDeserializeFile(crate::FileSerializationError::Io(
                    e.to_string(),
                ))
            })?
        };

        Ok(data)
    }

    fn serialize_associated_data_into_data(
        _: &std::path::Path,
        _: &Self::AssociatedData,
    ) -> Result<SerializedData, SerializedDataError> {
        Err(SerializedDataError::CannotUseValue)
    }

    fn deserialize_associated_data_from_data(
        &self,
        _: &SerializedData,
        _: &TrailingCommaUtility,
    ) -> Result<Self::AssociatedData, SerializedDataError> {
        Err(SerializedDataError::CannotUseValue)
    }

    fn cleanup_on_replace(&self, mut files_to_cleanup: impl FileHolder) {
        if self.sound_file.is_empty() == false {
            files_to_cleanup.push(Path::new(&self.sound_file).to_owned());
        }
    }
}
