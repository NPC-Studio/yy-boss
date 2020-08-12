use super::{
    input::{Command, NewResource, ResourceCommandType},
    output::{CommandOutput, Output, YypBossError},
};
use crate::{Resource, YyResource, YypBoss};
use log::error;
use std::path::{Path, PathBuf};
use yy_boss::{SerializedData, SerializedDataError, YyResourceData};
use yy_typings::{object_yy::Object, script::Script, sprite_yy::Sprite};

pub struct YyCli {
    pub working_directory: Option<PathBuf>,
}

impl YyCli {
    pub fn new(working_directory: Option<PathBuf>) -> Self {
        YyCli { working_directory }
    }

    pub fn parse_command(&self, command: Command, yyp_boss: &mut YypBoss) -> Output {
        match command {
            Command::Resource(resource_command) => match resource_command.command_type {
                ResourceCommandType::Add(new_resource) => match resource_command.resource {
                    Resource::Sprite => self.add_resource::<Sprite>(yyp_boss, new_resource),
                    Resource::Script => self.add_resource::<Script>(yyp_boss, new_resource),
                    Resource::Object => self.add_resource::<Object>(yyp_boss, new_resource),
                },
                ResourceCommandType::Replace(new_resource) => match resource_command.resource {
                    Resource::Sprite => self.replace_resource::<Sprite>(yyp_boss, new_resource),
                    Resource::Script => self.replace_resource::<Script>(yyp_boss, new_resource),
                    Resource::Object => self.replace_resource::<Object>(yyp_boss, new_resource),
                },
                ResourceCommandType::Set(new_resource) => unimplemented!(),
                ResourceCommandType::Remove { .. } => unimplemented!(),
                ResourceCommandType::Get { .. } => unimplemented!(),
                ResourceCommandType::Exists { .. } => unimplemented!(),
            },
            Command::VirtualFileSystem(_vfs_command) => unimplemented!(),
        }
    }

    pub fn add_resource<T: YyResource>(
        &self,
        yyp_boss: &mut YypBoss,
        new_resource: NewResource,
    ) -> Output {
        let (yy_file, associated_data) = match self.read_new_resource::<T>(new_resource) {
            Ok(o) => o,
            Err(e) => {
                return e;
            }
        };

        // check for a bad add...
        if let Some(found_resource) = yyp_boss.get_resource(yy_file.name()) {
            return Output::Command(CommandOutput::error(YypBossError::BadAdd(
                found_resource.inner(),
                yy_file.name().to_string(),
            )));
        }

        match yyp_boss.new_resource_end(yy_file.parent_path(), yy_file.name(), T::RESOURCE) {
            Ok(crt) => {
                let handler = T::get_handler(yyp_boss);
                let result = handler.set(yy_file, associated_data, crt);
                if let Some(old_result) = result {
                    error!(
                        "yyp resource and yyp resource names out of sync!\n\
                    {} was not in resource names but WAS in our resource manager for {}.",
                        old_result.yy_resource.name(),
                        T::RESOURCE
                    );
                    Output::Command(CommandOutput::error(YypBossError::InternalError))
                } else {
                    Output::Command(CommandOutput::ok())
                }
            }
            // we couldn't add the file to the folder...
            Err(e) => Output::Command(CommandOutput::error(YypBossError::FolderGraphError(e))),
        }
    }

    pub fn replace_resource<T: YyResource>(
        &self,
        yyp_boss: &mut YypBoss,
        new_resource: NewResource,
    ) -> Output {
        let (yy_file, associated_data) = match self.read_new_resource::<T>(new_resource) {
            Ok(o) => o,
            Err(e) => {
                return e;
            }
        };

        if let Some(crt) = yyp_boss.get_resource(yy_file.name()) {
            let handler = T::get_handler(yyp_boss);
            let result = handler.set(yy_file, associated_data, crt);
            if let Some(old_result) = result {
                match self.deserialize_yy_data::<T>(old_result) {
                    Ok((yy_file, serialized_data)) => {
                        Output::Command(CommandOutput::ok_datum(yy_file, serialized_data))
                    }
                    Err(e) => unimplemented!(),
                }
            } else {
                error!(
                    "yyp resource and yyp resource names out of sync!\n\
                    a name was IN resource names but NOT in our resource manager for {}.",
                    T::RESOURCE
                );
                Output::Command(CommandOutput::error(YypBossError::InternalError))
            }
        } else {
            // check for a bad replace...
            Output::Command(CommandOutput::error(YypBossError::BadReplace(
                yy_file.name().to_string(),
            )))
        }
    }

    pub fn read_new_resource<T: YyResource>(
        &self,
        new_resource: NewResource,
    ) -> Result<(T, T::AssociatedData), Output> {
        let value: T = new_resource
            .new_resource
            .read_data_as_file(self.working_directory.as_deref())
            .map_err(|e| {
                let yyp_error = match e {
                    SerializedDataError::NoFileMode => YypBossError::NoFileMode,
                    SerializedDataError::BadDataFile(v) => YypBossError::BadDataFile(v),
                    SerializedDataError::CouldNotParseData(v) => {
                        YypBossError::CouldNotParseYyFile(v.to_string())
                    }
                    SerializedDataError::CannotUseValue => YypBossError::CannotUseValue,
                };

                Output::Command(CommandOutput::error(yyp_error))
            })?;

        let associated_data: T::AssociatedData = value
            .deserialize_associated_data(
                self.working_directory.as_deref(),
                new_resource.associated_data,
            )
            .map_err(|e| {
                Output::Command(CommandOutput::error(
                    YypBossError::CouldNotParseAssociatedData(e.to_string()),
                ))
            })?;

        Ok((value, associated_data))
    }

    pub fn deserialize_yy_data<T: YyResource>(
        &self,
        data: YyResourceData<T>,
    ) -> anyhow::Result<(SerializedData, SerializedData)> {
        let yy_data = SerializedData::Value {
            data: serde_json::to_string_pretty(&data.yy_resource).unwrap(),
        };

        let associated_data = data.yy_resource.serialize_associated_data_into_data(
            Path::new(""),
            self.working_directory.as_deref(),
            data.associated_data.as_ref(),
        )?;

        Ok((yy_data, associated_data))
    }
}
