use super::{
    input::{Command, NewResource, ResourceCommandType, VfsCommand},
    output::{CommandOutput, Output, YypBossError},
};
use crate::{Resource, YyResource, YypBoss};
use std::path::PathBuf;
use yy_boss::{
    folders::FolderGraphError, utils, ResourceManipulationError, SerializedData,
    SerializedDataError,
};
use yy_typings::{
    object_yy::Object, script::Script, sprite_yy::Sprite, utils::TrailingCommaUtility,
};

pub struct YyCli {
    pub working_directory: PathBuf,
}

impl YyCli {
    pub fn new(working_directory: PathBuf) -> Self {
        YyCli { working_directory }
    }

    pub fn parse_command(&self, command: Command, yyp_boss: &mut YypBoss) -> Output {
        let command_output = match command {
            Command::Resource(resource_command) => {
                match resource_command.command_type {
                    ResourceCommandType::Add(new_resource) => match resource_command.resource {
                        Resource::Sprite => self.add::<Sprite>(yyp_boss, new_resource),
                        Resource::Script => self.add::<Script>(yyp_boss, new_resource),
                        Resource::Object => self.add::<Object>(yyp_boss, new_resource),
                    },
                    // ResourceCommandType::Replace(new_resource) => match resource_command.resource {
                    //     Resource::Sprite => self.replace::<Sprite>(yyp_boss, new_resource),
                    //     Resource::Script => self.replace::<Script>(yyp_boss, new_resource),
                    //     Resource::Object => self.replace::<Object>(yyp_boss, new_resource),
                    // },
                    // ResourceCommandType::Set(new_resource) => match resource_command.resource {
                    //     Resource::Sprite => self.set::<Sprite>(yyp_boss, new_resource),
                    //     Resource::Script => self.set::<Script>(yyp_boss, new_resource),
                    //     Resource::Object => self.set::<Object>(yyp_boss, new_resource),
                    // },
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
                }
            }
            Command::VirtualFileSystem(vfs_command) => match vfs_command {
                VfsCommand::MoveFolder {
                    folder_to_move,
                    new_parent,
                } => match yyp_boss.vfs.move_folder(folder_to_move, &new_parent) {
                    Ok(()) => Ok(CommandOutput::ok()),
                    Err(e) => Err(YypBossError::ResourceManipulation(
                        ResourceManipulationError::FolderGraphError(e),
                    )),
                },
                VfsCommand::MoveResource {
                    resource_to_move,
                    resource,
                    new_parent,
                } => {
                    match yyp_boss.move_resource_dynamic(&resource_to_move, new_parent, resource) {
                        Ok(()) => Ok(CommandOutput::ok()),
                        Err(e) => Err(YypBossError::ResourceManipulation(e)),
                    }
                }
                VfsCommand::RemoveFolder {
                    folder_to_remove,
                    recursive,
                } => {
                    if recursive {
                        match yyp_boss.remove_folder(&folder_to_remove) {
                            Ok(()) => Ok(CommandOutput::ok()),
                            Err(e) => Err(YypBossError::ResourceManipulation(e)),
                        }
                    } else {
                        match yyp_boss.vfs.remove_empty_folder(&folder_to_remove) {
                            Ok(()) => Ok(CommandOutput::ok()),
                            Err(e) => Err(YypBossError::FolderGraphError(e)),
                        }
                    }
                }
                VfsCommand::GetFolder(folder_name) => match yyp_boss.vfs.get_folder(&folder_name) {
                    Some(v) => Ok(CommandOutput::ok_folder_graph(v.clone())),
                    None => Err(YypBossError::FolderGraphError(
                        FolderGraphError::PathNotFound(folder_name.to_string()),
                    )),
                },
                VfsCommand::GetFullVfs => {
                    let vfs = yyp_boss.vfs.get_root_folder().clone();

                    Ok(CommandOutput::ok_folder_graph(vfs))
                }
                VfsCommand::GetPathType(path_type) => match yyp_boss.vfs.path_kind(&path_type) {
                    Some(v) => Ok(CommandOutput::ok_path_kind(v)),
                    None => Err(YypBossError::FolderGraphError(
                        FolderGraphError::PathNotFound(path_type.path.to_string()),
                    )),
                },
            },
        };
        Output::Command(command_output.unwrap_or_else(CommandOutput::error))
    }

    fn add<T: YyResource>(
        &self,
        yyp_boss: &mut YypBoss,
        new_resource: NewResource,
    ) -> Result<CommandOutput, YypBossError> {
        let (yy_file, associated_data) =
            self.read_new_resource::<T>(new_resource, &yyp_boss.tcu())?;

        // check for a bad add...
        match yyp_boss.add_resource(yy_file, associated_data) {
            Ok(()) => Ok(CommandOutput::ok()),
            Err(e) => Err(YypBossError::ResourceManipulation(e)),
        }
    }

    // fn replace<T: YyResource>(
    //     &self,
    //     yyp_boss: &mut YypBoss,
    //     new_resource: NewResource,
    // ) -> CommandOutput {
    //     let (yy_file, associated_data) = match self.read_new_resource::<T>(new_resource) {
    //         Ok(o) => o,
    //         Err(e) => {
    //             return e;
    //         }
    //     };

    //     if let Some(crt) = yyp_boss.get_resource(yy_file.name()) {
    //         let handler = T::get_handler_mut(yyp_boss);
    //         let result = handler.set(yy_file, associated_data, crt);
    //         if let Some(old_result) = result {
    //             match self.deserialize_yy_data::<T>(yyp_boss, &old_result) {
    //                 Ok((yy_file, serialized_data)) => {
    //                     CommandOutput::ok_datum(yy_file, serialized_data)
    //                 }
    //                 Err(e) => CommandOutput::error(e.into()),
    //             }
    //         } else {
    //             error!(
    //                 "yyp resource and yyp resource names out of sync!\n\
    //                 a name was IN resource names but NOT in our resource manager for {}.",
    //                 T::RESOURCE
    //             );
    //             CommandOutput::error(YypBossError::InternalError(true))
    //         }
    //     } else {
    //         // check for a bad replace...
    //         CommandOutput::error(YypBossError::BadReplace(yy_file.name().to_string()))
    //     }
    // }

    // fn set<T: YyResource>(
    //     &self,
    //     yyp_boss: &mut YypBoss,
    //     new_resource: NewResource,
    // ) -> CommandOutput {
    //     let (yy_file, associated_data) = match self.read_new_resource::<T>(new_resource) {
    //         Ok(o) => o,
    //         Err(e) => {
    //             return e;
    //         }
    //     };

    //     // Get it somehow or another...
    //     let crt = match yyp_boss.get_resource(yy_file.name()) {
    //         Some(v) => v,
    //         None => {
    //             match yyp_boss.new_resource_end(
    //                 yy_file.parent_view_path(),
    //                 yy_file.name(),
    //                 T::RESOURCE,
    //             ) {
    //                 Ok(v) => v,
    //                 Err(e) => return CommandOutput::error(YypBossError::FolderGraphError(e)),
    //             }
    //         }
    //     };

    //     let handler = T::get_handler_mut(yyp_boss);
    //     handler.set(yy_file, associated_data, crt);

    //     CommandOutput::ok()
    // }

    fn remove<T: YyResource>(
        &self,
        yyp_boss: &mut YypBoss,
        resource_name: String,
    ) -> Result<CommandOutput, YypBossError> {
        match yyp_boss.remove_resource::<T>(&resource_name) {
            Ok(output) => match self.serialize_yy_data_for_output(&output.0, output.1.as_ref()) {
                Ok((yy, assoc)) => Ok(CommandOutput::ok_datum(yy, assoc)),
                Err(e) => Err(YypBossError::CouldNotOutputData(e.to_string())),
            },
            Err(e) => Err(YypBossError::ResourceManipulation(e)),
        }
    }

    fn get<T: YyResource>(
        &self,
        yyp_boss: &YypBoss,
        resource_name: String,
    ) -> Result<CommandOutput, YypBossError> {
        match yyp_boss.get_resource::<T>(&resource_name) {
            Some(output) => {
                match self.serialize_yy_data_for_output(
                    &output.yy_resource,
                    output.associated_data.as_ref(),
                ) {
                    Ok((yy, assoc)) => Ok(CommandOutput::ok_datum(yy, assoc)),
                    Err(e) => Err(YypBossError::CouldNotOutputData(e.to_string())),
                }
            }
            None => Err(YypBossError::ResourceManipulation(
                ResourceManipulationError::BadGet,
            )),
        }

        // let crt = match yyp_boss.get_resource(&resource_name) {
        //     Some(v) => v,
        //     None => {
        //         return CommandOutput::error(YypBossError::BadRemove(resource_name));
        //     }
        // };

        // let handler = T::get_handler(yyp_boss);
        // let result = handler.get(&resource_name, crt);

        // if let Some(old_result) = result {
        //     match self.deserialize_yy_data::<T>(yyp_boss, &old_result) {
        //         Ok((yy_file, serialized_data)) => CommandOutput::ok_datum(yy_file, serialized_data),
        //         Err(e) => CommandOutput::error(e.into()),
        //     }
        // } else {
        //     error!(
        //         "yyp resource and yyp resource names out of sync!\n\
        //         a name was IN resource names but NOT in our resource manager for {}.",
        //         T::RESOURCE
        //     );
        //     CommandOutput::error(YypBossError::InternalError(true))
        // }
    }

    fn exists<T: YyResource>(
        &self,
        yyp_boss: &YypBoss,
        resource_name: String,
    ) -> Result<CommandOutput, YypBossError> {
        Ok(CommandOutput::ok_exists(
            yyp_boss.vfs.resource_exists(&resource_name),
        ))
    }

    fn read_new_resource<T: YyResource>(
        &self,
        new_resource: NewResource,
        tcu: &TrailingCommaUtility,
    ) -> Result<(T, T::AssociatedData), YypBossError> {
        let value: T = match new_resource.new_resource {
            SerializedData::Value { data } => {
                serde_json::from_str(&data).map_err(|e| YypBossError::YyParseError(e.to_string()))
            }
            SerializedData::Filepath { data } => {
                let path = self.working_directory.join(data);
                utils::deserialize_json_tc(&path, tcu)
                    .map_err(|e| YypBossError::YyParseError(e.to_string()))
            }
            SerializedData::DefaultValue => Ok(T::default()),
        }?;
        let incoming_data = new_resource.associated_data.as_assoc_data_location();

        let associated_data: T::AssociatedData = value
            .deserialize_associated_data(incoming_data, tcu)
            .map_err(|e| YypBossError::AssociatedDataParseError(e.to_string()))?;

        Ok((value, associated_data))
    }

    fn serialize_yy_data_for_output<T: YyResource>(
        &self,
        yy: &T,
        assoc_data: Option<&T::AssociatedData>,
    ) -> Result<(SerializedData, Option<SerializedData>), SerializedDataError> {
        let yy_data = SerializedData::Value {
            data: serde_json::to_string_pretty(&yy).unwrap(),
        };

        let assoc_output = if let Some(data) = assoc_data {
            Some(T::serialize_associated_data_into_data(
                &self.working_directory,
                data,
            )?)
        } else {
            None
        };

        Ok((yy_data, assoc_output))
    }
}
