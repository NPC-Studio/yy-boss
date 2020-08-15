use super::{
    input::{Command, NewResource, ResourceCommandType, VfsCommand},
    output::{CommandOutput, Output, YypBossError},
};
use crate::{Resource, YyResource, YypBoss};
use log::error;
use std::path::PathBuf;
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
            Command::Resource(resource_command) => {
                let command_output = match resource_command.command_type {
                    ResourceCommandType::Add(new_resource) => match resource_command.resource {
                        Resource::Sprite => self.add::<Sprite>(yyp_boss, new_resource),
                        Resource::Script => self.add::<Script>(yyp_boss, new_resource),
                        Resource::Object => self.add::<Object>(yyp_boss, new_resource),
                    },
                    ResourceCommandType::Replace(new_resource) => match resource_command.resource {
                        Resource::Sprite => self.replace::<Sprite>(yyp_boss, new_resource),
                        Resource::Script => self.replace::<Script>(yyp_boss, new_resource),
                        Resource::Object => self.replace::<Object>(yyp_boss, new_resource),
                    },
                    ResourceCommandType::Set(new_resource) => match resource_command.resource {
                        Resource::Sprite => self.set::<Sprite>(yyp_boss, new_resource),
                        Resource::Script => self.set::<Script>(yyp_boss, new_resource),
                        Resource::Object => self.set::<Object>(yyp_boss, new_resource),
                    },
                    ResourceCommandType::Remove { identifier } => match resource_command.resource {
                        Resource::Sprite => self.remove::<Sprite>(yyp_boss, identifier),
                        Resource::Script => self.remove::<Script>(yyp_boss, identifier),
                        Resource::Object => self.remove::<Object>(yyp_boss, identifier),
                    },
                    ResourceCommandType::Get { identifier } => match resource_command.resource {
                        Resource::Sprite => self.get::<Sprite>(yyp_boss, identifier),
                        Resource::Script => self.get::<Script>(yyp_boss, identifier),
                        Resource::Object => self.get::<Object>(yyp_boss, identifier),
                    },
                    ResourceCommandType::Exists { identifier } => match resource_command.resource {
                        Resource::Sprite => self.exists::<Sprite>(yyp_boss, identifier),
                        Resource::Script => self.exists::<Script>(yyp_boss, identifier),
                        Resource::Object => self.exists::<Object>(yyp_boss, identifier),
                    },
                };

                Output::Command(command_output)
            }
            Command::VirtualFileSystem(vfs_command) => match vfs_command {
                VfsCommand::MoveItem { start, end } => unimplemented!(),
                VfsCommand::DeleteFolder { recursive } => unimplemented!(),
                VfsCommand::GetFolder(f) => {
                    if let Some(x) = yyp_boss.folder(&f.path) {
                        Output::Command(CommandOutput::ok_folder_graph(x))
                    } else {
                        Output::Command(CommandOutput::error(YypBossError::FolderGraphError(
                            yy_boss::FolderGraphError::PathNotFound,
                        )))
                    }
                }
                VfsCommand::GetFullVfs => unimplemented!(),
                VfsCommand::GetPathType(path) => unimplemented!(),
            },
        }
    }

    fn add<T: YyResource>(
        &self,
        yyp_boss: &mut YypBoss,
        new_resource: NewResource,
    ) -> CommandOutput {
        let (yy_file, associated_data) = match self.read_new_resource::<T>(new_resource) {
            Ok(o) => o,
            Err(e) => {
                return e;
            }
        };

        // check for a bad add...
        if let Some(found_resource) = yyp_boss.get_resource(yy_file.name()) {
            return CommandOutput::error(YypBossError::BadAdd(
                found_resource.inner(),
                yy_file.name().to_string(),
            ));
        }

        match yyp_boss.new_resource_end(yy_file.parent_path(), yy_file.name(), T::RESOURCE) {
            Ok(crt) => {
                let handler = T::get_handler_mut(yyp_boss);
                let result = handler.set(yy_file, associated_data, crt);
                if let Some(old_result) = result {
                    error!(
                        "yyp resource and yyp resource names out of sync!\n\
                    {} was not in resource names but WAS in our resource manager for {}.",
                        old_result.yy_resource.name(),
                        T::RESOURCE
                    );
                    CommandOutput::error(YypBossError::InternalError(true))
                } else {
                    CommandOutput::ok()
                }
            }
            // we couldn't add the file to the folder...
            Err(e) => CommandOutput::error(YypBossError::FolderGraphError(e)),
        }
    }

    fn replace<T: YyResource>(
        &self,
        yyp_boss: &mut YypBoss,
        new_resource: NewResource,
    ) -> CommandOutput {
        let (yy_file, associated_data) = match self.read_new_resource::<T>(new_resource) {
            Ok(o) => o,
            Err(e) => {
                return e;
            }
        };

        if let Some(crt) = yyp_boss.get_resource(yy_file.name()) {
            let handler = T::get_handler_mut(yyp_boss);
            let result = handler.set(yy_file, associated_data, crt);
            if let Some(old_result) = result {
                match self.deserialize_yy_data::<T>(yyp_boss, &old_result) {
                    Ok((yy_file, serialized_data)) => {
                        CommandOutput::ok_datum(yy_file, serialized_data)
                    }
                    Err(e) => CommandOutput::error(e.into()),
                }
            } else {
                error!(
                    "yyp resource and yyp resource names out of sync!\n\
                    a name was IN resource names but NOT in our resource manager for {}.",
                    T::RESOURCE
                );
                CommandOutput::error(YypBossError::InternalError(true))
            }
        } else {
            // check for a bad replace...
            CommandOutput::error(YypBossError::BadReplace(yy_file.name().to_string()))
        }
    }

    fn set<T: YyResource>(
        &self,
        yyp_boss: &mut YypBoss,
        new_resource: NewResource,
    ) -> CommandOutput {
        let (yy_file, associated_data) = match self.read_new_resource::<T>(new_resource) {
            Ok(o) => o,
            Err(e) => {
                return e;
            }
        };

        // Get it somehow or another...
        let crt = match yyp_boss.get_resource(yy_file.name()) {
            Some(v) => v,
            None => {
                match yyp_boss.new_resource_end(yy_file.parent_path(), yy_file.name(), T::RESOURCE)
                {
                    Ok(v) => v,
                    Err(e) => return CommandOutput::error(YypBossError::FolderGraphError(e)),
                }
            }
        };

        let handler = T::get_handler_mut(yyp_boss);
        handler.set(yy_file, associated_data, crt);

        CommandOutput::ok()
    }

    fn remove<T: YyResource>(
        &self,
        yyp_boss: &mut YypBoss,
        resource_name: String,
    ) -> CommandOutput {
        let rrt = match yyp_boss.remove_resource(&resource_name, T::RESOURCE) {
            Ok(v) => v,
            Err(_) => {
                return CommandOutput::error(YypBossError::BadRemove(resource_name));
            }
        };

        let handler = T::get_handler_mut(yyp_boss);
        let result = handler.remove(&resource_name, rrt);

        if let Some(old_result) = result {
            match self.deserialize_yy_data::<T>(yyp_boss, &old_result) {
                Ok((yy_file, serialized_data)) => CommandOutput::ok_datum(yy_file, serialized_data),
                Err(e) => CommandOutput::error(e.into()),
            }
        } else {
            error!(
                "yyp resource and yyp resource names out of sync!\n\
                a name was IN resource names but NOT in our resource manager for {}.",
                T::RESOURCE
            );
            CommandOutput::error(YypBossError::InternalError(true))
        }
    }

    fn get<T: YyResource>(&self, yyp_boss: &YypBoss, resource_name: String) -> CommandOutput {
        let crt = match yyp_boss.get_resource(&resource_name) {
            Some(v) => v,
            None => {
                return CommandOutput::error(YypBossError::BadRemove(resource_name));
            }
        };

        let handler = T::get_handler(yyp_boss);
        let result = handler.get(&resource_name, crt);

        if let Some(old_result) = result {
            match self.deserialize_yy_data::<T>(yyp_boss, &old_result) {
                Ok((yy_file, serialized_data)) => CommandOutput::ok_datum(yy_file, serialized_data),
                Err(e) => CommandOutput::error(e.into()),
            }
        } else {
            error!(
                "yyp resource and yyp resource names out of sync!\n\
                a name was IN resource names but NOT in our resource manager for {}.",
                T::RESOURCE
            );
            CommandOutput::error(YypBossError::InternalError(true))
        }
    }

    fn exists<T: YyResource>(&self, yyp_boss: &YypBoss, resource_name: String) -> CommandOutput {
        let exists = yyp_boss
            .get_resource(&resource_name)
            .map(|v| {
                let handler = T::get_handler(yyp_boss);
                handler.get(&resource_name, v).is_some()
            })
            .unwrap_or_default();

        CommandOutput::ok_exists(exists)
    }

    fn read_new_resource<T: YyResource>(
        &self,
        new_resource: NewResource,
    ) -> Result<(T, T::AssociatedData), CommandOutput> {
        let value: T = new_resource
            .new_resource
            .read_data_as_file(self.working_directory.as_deref())
            .map_err(|e| CommandOutput::error(e.into()))?;

        let associated_data: T::AssociatedData = value
            .deserialize_associated_data(
                self.working_directory.as_deref(),
                new_resource.associated_data,
            )
            .map_err(|e| {
                CommandOutput::error(YypBossError::CouldNotParseAssociatedData(e.to_string()))
            })?;

        Ok((value, associated_data))
    }

    pub fn deserialize_yy_data<T: YyResource>(
        &self,
        yyp_boss: &YypBoss,
        data: &YyResourceData<T>,
    ) -> Result<(SerializedData, SerializedData), SerializedDataError> {
        let yy_data = SerializedData::Value {
            data: serde_json::to_string_pretty(&data.yy_resource).unwrap(),
        };

        let associated_data = data.yy_resource.serialize_associated_data_into_data(
            &yyp_boss
                .directory_manager
                .resource_file(&data.yy_resource.relative_path()),
            self.working_directory.as_deref(),
            data.associated_data.as_ref(),
        )?;

        Ok((yy_data, associated_data))
    }
}
