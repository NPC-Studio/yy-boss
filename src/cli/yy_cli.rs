use super::{
    input::{Command, CreateCommand, NewResource, ResourceCommandType, UtilityCommand, VfsCommand},
    output::{CommandOutput, Output, YypBossError},
};
use crate::{
    folders::FolderGraphError, utils, ResourceManipulationError, SerializedData,
    SerializedDataError,
};
use crate::{Resource, YyResource, YypBoss};
use std::path::PathBuf;
use yy_typings::{
    object_yy::{EventType, Object},
    script::Script,
    shader::Shader,
    sprite_yy::Sprite,
    utils::TrailingCommaUtility,
    AnimationCurve, Extension, Font, Note, Path, Room, Sequence, Sound, TileSet, Timeline,
};

pub struct YyCli {
    pub working_directory: PathBuf,
}

impl YyCli {
    pub fn new(working_directory: PathBuf) -> Self {
        YyCli { working_directory }
    }

    pub fn parse_command(
        &self,
        command: Command,
        yyp_boss: &mut YypBoss,
        shutdown_flag: &mut bool,
    ) -> Output {
        let command_output = match command {
            Command::Resource(resource_command) => match resource_command.command_type {
                ResourceCommandType::Add(new_resource) => match resource_command.resource {
                    Resource::Sprite => self.add::<Sprite>(yyp_boss, new_resource),
                    Resource::Script => self.add::<Script>(yyp_boss, new_resource),
                    Resource::Object => self.add::<Object>(yyp_boss, new_resource),
                    Resource::Note => self.add::<Note>(yyp_boss, new_resource),
                    Resource::Shader => self.add::<Shader>(yyp_boss, new_resource),
                    Resource::AnimationCurve => self.add::<AnimationCurve>(yyp_boss, new_resource),
                    Resource::Extension => self.add::<Extension>(yyp_boss, new_resource),
                    Resource::Font => self.add::<Font>(yyp_boss, new_resource),
                    Resource::Room => self.add::<Room>(yyp_boss, new_resource),
                    Resource::Path => self.add::<Path>(yyp_boss, new_resource),
                    Resource::Sequence => self.add::<Sequence>(yyp_boss, new_resource),
                    Resource::Sound => self.add::<Sound>(yyp_boss, new_resource),
                    Resource::TileSet => self.add::<TileSet>(yyp_boss, new_resource),
                    Resource::Timeline => self.add::<Timeline>(yyp_boss, new_resource),
                },
                ResourceCommandType::Remove { identifier } => match resource_command.resource {
                    Resource::Sprite => self.remove::<Sprite>(yyp_boss, identifier),
                    Resource::Script => self.remove::<Script>(yyp_boss, identifier),
                    Resource::Object => self.remove::<Object>(yyp_boss, identifier),
                    Resource::Note => self.remove::<Note>(yyp_boss, identifier),
                    Resource::Shader => self.remove::<Shader>(yyp_boss, identifier),
                    Resource::AnimationCurve => self.remove::<AnimationCurve>(yyp_boss, identifier),
                    Resource::Extension => self.remove::<Extension>(yyp_boss, identifier),
                    Resource::Font => self.remove::<Font>(yyp_boss, identifier),
                    Resource::Path => self.remove::<Path>(yyp_boss, identifier),
                    Resource::Room => self.remove::<Room>(yyp_boss, identifier),
                    Resource::Sequence => self.remove::<Sequence>(yyp_boss, identifier),
                    Resource::Sound => self.remove::<Sound>(yyp_boss, identifier),
                    Resource::TileSet => self.remove::<TileSet>(yyp_boss, identifier),
                    Resource::Timeline => self.remove::<Timeline>(yyp_boss, identifier),
                },
                ResourceCommandType::Rename {
                    identifier,
                    new_name,
                } => {
                    let output = match resource_command.resource {
                        Resource::Sprite => {
                            yyp_boss.rename_resource::<Sprite>(&identifier, new_name)
                        }
                        Resource::Script => {
                            yyp_boss.rename_resource::<Script>(&identifier, new_name)
                        }
                        Resource::Object => {
                            yyp_boss.rename_resource::<Object>(&identifier, new_name)
                        }
                        Resource::Note => yyp_boss.rename_resource::<Note>(&identifier, new_name),
                        Resource::Shader => {
                            yyp_boss.rename_resource::<Shader>(&identifier, new_name)
                        }
                        Resource::AnimationCurve => {
                            yyp_boss.rename_resource::<AnimationCurve>(&identifier, new_name)
                        }
                        Resource::Extension => {
                            yyp_boss.rename_resource::<Extension>(&identifier, new_name)
                        }
                        Resource::Font => yyp_boss.rename_resource::<Font>(&identifier, new_name),
                        Resource::Path => yyp_boss.rename_resource::<Path>(&identifier, new_name),
                        Resource::Room => yyp_boss.rename_resource::<Room>(&identifier, new_name),
                        Resource::Sequence => {
                            yyp_boss.rename_resource::<Sequence>(&identifier, new_name)
                        }
                        Resource::Sound => yyp_boss.rename_resource::<Sound>(&identifier, new_name),
                        Resource::TileSet => {
                            yyp_boss.rename_resource::<TileSet>(&identifier, new_name)
                        }
                        Resource::Timeline => {
                            yyp_boss.rename_resource::<Timeline>(&identifier, new_name)
                        }
                    };

                    match output {
                        Ok(()) => Ok(CommandOutput::ok()),
                        Err(e) => Err(YypBossError::ResourceManipulation {
                            data: e.to_string(),
                        }),
                    }
                }
                ResourceCommandType::Get { identifier } => match resource_command.resource {
                    Resource::Sprite => self.get_resource::<Sprite>(yyp_boss, identifier),
                    Resource::Script => self.get_resource::<Script>(yyp_boss, identifier),
                    Resource::Object => self.get_resource::<Object>(yyp_boss, identifier),
                    Resource::Note => self.get_resource::<Note>(yyp_boss, identifier),
                    Resource::Shader => self.get_resource::<Shader>(yyp_boss, identifier),
                    Resource::AnimationCurve
                    | Resource::Extension
                    | Resource::Font
                    | Resource::Path
                    | Resource::Room
                    | Resource::Sequence
                    | Resource::Sound
                    | Resource::TileSet
                    | Resource::Timeline => Err(YypBossError::ResourceManipulation {
                        data: ResourceManipulationError::ResourceCannotBeManipulated.to_string(),
                    }),
                },
                ResourceCommandType::GetAssociatedData { identifier, force } => {
                    match resource_command.resource {
                        Resource::Sprite => {
                            self.ensure_associated_data::<Sprite>(yyp_boss, identifier, force)
                        }
                        Resource::Script => {
                            self.ensure_associated_data::<Script>(yyp_boss, identifier, force)
                        }
                        Resource::Object => {
                            self.ensure_associated_data::<Object>(yyp_boss, identifier, force)
                        }
                        Resource::Note => {
                            self.ensure_associated_data::<Note>(yyp_boss, identifier, force)
                        }
                        Resource::Shader => {
                            self.ensure_associated_data::<Shader>(yyp_boss, identifier, force)
                        }

                        Resource::AnimationCurve
                        | Resource::Extension
                        | Resource::Font
                        | Resource::Path
                        | Resource::Room
                        | Resource::Sequence
                        | Resource::Sound
                        | Resource::TileSet
                        | Resource::Timeline => Err(YypBossError::ResourceManipulation {
                            data: ResourceManipulationError::ResourceCannotBeManipulated
                                .to_string(),
                        }),
                    }
                }
                ResourceCommandType::Exists { identifier } => Ok(CommandOutput::ok_exists(
                    yyp_boss
                        .vfs
                        .resource_exists(&identifier, resource_command.resource),
                )),
            },
            Command::VirtualFileSystem(vfs_command) => match vfs_command {
                VfsCommand::MoveFolder { folder, new_parent } => {
                    match yyp_boss.vfs.move_folder(folder, &new_parent) {
                        Ok(()) => Ok(CommandOutput::ok()),
                        Err(e) => Err(YypBossError::ResourceManipulation {
                            data: ResourceManipulationError::FolderGraphError(e).to_string(),
                        }),
                    }
                }

                VfsCommand::CreateFolder {
                    folder_name,
                    parent_folder,
                } => match yyp_boss.vfs.new_folder_end(&parent_folder, &folder_name) {
                    Ok(v) => Ok(CommandOutput::ok_created_folder(v)),
                    Err(e) => Err(YypBossError::ResourceManipulation {
                        data: ResourceManipulationError::FolderGraphError(e).to_string(),
                    }),
                },

                VfsCommand::MoveResource {
                    resource_to_move,
                    resource,
                    new_parent,
                } => {
                    match yyp_boss.move_resource_dynamic(&resource_to_move, new_parent, resource) {
                        Ok(()) => Ok(CommandOutput::ok()),
                        Err(e) => Err(YypBossError::ResourceManipulation {
                            data: e.to_string(),
                        }),
                    }
                }
                VfsCommand::RemoveFolder { folder, recursive } => {
                    if recursive {
                        match yyp_boss.remove_folder(&folder) {
                            Ok(()) => Ok(CommandOutput::ok()),
                            Err(e) => Err(YypBossError::ResourceManipulation {
                                data: e.to_string(),
                            }),
                        }
                    } else {
                        match yyp_boss.vfs.remove_empty_folder(&folder) {
                            Ok(()) => Ok(CommandOutput::ok()),
                            Err(e) => Err(YypBossError::FolderGraphError {
                                data: e.to_string(),
                            }),
                        }
                    }
                }
                VfsCommand::RenameFolder { folder, new_name } => {
                    match yyp_boss.vfs.rename_folder(&folder, new_name) {
                        Ok(()) => Ok(CommandOutput::ok()),
                        Err(e) => Err(YypBossError::FolderGraphError {
                            data: e.to_string(),
                        }),
                    }
                }
                VfsCommand::GetFolder { folder } => match yyp_boss.vfs.get_folder(&folder) {
                    Some(v) => Ok(CommandOutput::ok_folder_graph(
                        v.to_flat(&yyp_boss.vfs.resource_names),
                    )),
                    None => Err(YypBossError::FolderGraphError {
                        data: FolderGraphError::PathNotFound {
                            path: folder.to_string(),
                        }
                        .to_string(),
                    }),
                },
                VfsCommand::GetFullVfs => {
                    let vfs = yyp_boss.vfs.get_root_folder();
                    let flat = vfs.to_flat(&yyp_boss.vfs.resource_names);

                    Ok(CommandOutput::ok_folder_graph(flat))
                }
                VfsCommand::GetPathType { path } => match yyp_boss.vfs.path_kind(&path) {
                    Some(v) => Ok(CommandOutput::ok_path_kind(v)),
                    None => Err(YypBossError::FolderGraphError {
                        data: FolderGraphError::PathNotFound {
                            path: path.path.to_string(),
                        }
                        .to_string(),
                    }),
                },
            },
            Command::Utilities(util) => match util {
                UtilityCommand::ProjectInfo => {
                    Ok(CommandOutput::ok_metadata(yyp_boss.project_metadata()))
                }

                UtilityCommand::Create(create_data) => match create_data.resource {
                    Resource::Sprite => Self::create_yy::<Sprite>(create_data),
                    Resource::Script => Self::create_yy::<Script>(create_data),
                    Resource::Object => Self::create_yy::<Object>(create_data),
                    Resource::Note => Self::create_yy::<Note>(create_data),
                    Resource::Shader => Self::create_yy::<Shader>(create_data),
                    Resource::AnimationCurve
                    | Resource::Extension
                    | Resource::Font
                    | Resource::Path
                    | Resource::Room
                    | Resource::Sequence
                    | Resource::Sound
                    | Resource::TileSet
                    | Resource::Timeline => Err(YypBossError::ResourceManipulation {
                        data: ResourceManipulationError::ResourceCannotBeManipulated.to_string(),
                    }),
                },
                UtilityCommand::PrettyEventNames { event_names: v } => {
                    let mut output = v
                        .into_iter()
                        .map(|v| EventType::parse_filename_simple(&v).map_err(|_| v.to_string()))
                        .collect::<Vec<_>>();
                    output.sort();

                    let output = output
                        .into_iter()
                        .map(|v| match v {
                            Ok(output) => (output.filename_simple(), output.to_string()),
                            Err(e) => (e.clone(), e),
                        })
                        .collect();

                    Ok(CommandOutput::ok_event_names(output))
                }

                UtilityCommand::CreateEvent {
                    identifier,
                    event_file_name,
                } => match EventType::parse_filename_simple(&event_file_name) {
                    Ok(event_type) => {
                        match yyp_boss
                            .ensure_associated_data_is_loaded::<Object>(&identifier, false)
                        {
                            Ok(()) => {
                                if yyp_boss.objects.add_event(&identifier, event_type) {
                                    Ok(CommandOutput::ok())
                                } else {
                                    Err(YypBossError::ResourceManipulation {
                                        data: format!(
                                            "{} already had an event {}.",
                                            identifier, event_file_name
                                        ),
                                    })
                                }
                            }
                            Err(e) => Err(YypBossError::ResourceManipulation {
                                data: e.to_string(),
                            }),
                        }
                    }
                    Err(e) => Err(YypBossError::CouldNotReadCommand {
                        data: format!(
                            "{} was not a valid event filename -- {}",
                            event_file_name, e
                        ),
                    }),
                },

                UtilityCommand::DeleteEvent {
                    identifier,
                    event_file_name,
                } => match EventType::parse_filename_simple(&event_file_name) {
                    Ok(event_type) => {
                        match yyp_boss
                            .ensure_associated_data_is_loaded::<Object>(&identifier, false)
                        {
                            Ok(()) => {
                                if yyp_boss.objects.remove_event(&identifier, event_type) {
                                    Ok(CommandOutput::ok())
                                } else {
                                    Err(YypBossError::ResourceManipulation {
                                        data: format!(
                                            "{} did not have an event {}.",
                                            identifier, event_file_name
                                        ),
                                    })
                                }
                            }
                            Err(e) => Err(YypBossError::ResourceManipulation {
                                data: e.to_string(),
                            }),
                        }
                    }
                    Err(e) => Err(YypBossError::CouldNotReadCommand {
                        data: format!(
                            "{} was not a valid event filename -- {}",
                            event_file_name, e
                        ),
                    }),
                },

                UtilityCommand::ScriptGmlPath { script_name } => {
                    if let Some(script) = yyp_boss.scripts.get(&script_name) {
                        let path = yyp_boss
                            .directory_manager
                            .resource_file(&script.yy_resource.relative_yy_directory())
                            .join(format!("{}.gml", script.yy_resource.name));

                        Ok(CommandOutput::ok_path(path))
                    } else {
                        Err(YypBossError::CouldNotOutputData {
                            data: format!("could not find {}", script_name),
                        })
                    }
                }
                UtilityCommand::EventGmlPath {
                    object_name,
                    event_file_name,
                } => {
                    if let Some(object) = yyp_boss.objects.get(&object_name) {
                        if EventType::parse_filename_simple(&event_file_name).is_ok() {
                            let path = yyp_boss
                                .directory_manager
                                .resource_file(&object.yy_resource.relative_yy_directory())
                                .join(format!("{}.gml", event_file_name));
                            Ok(CommandOutput::ok_path(path))
                        } else {
                            Err(YypBossError::CouldNotOutputData {
                                data: format!("event_filename {} was invalid", event_file_name),
                            })
                        }
                    } else {
                        Err(YypBossError::CouldNotOutputData {
                            data: format!("could not find {}", object_name),
                        })
                    }
                }

                UtilityCommand::CanUseResourceName { identifier } => Ok(
                    CommandOutput::ok_name_is_valid(yyp_boss.can_use_name(&identifier).is_ok()),
                ),
                UtilityCommand::CanUseFolderName {
                    parent_folder,
                    identifier,
                } => Ok(CommandOutput::ok_name_is_valid(
                    yyp_boss
                        .vfs
                        .can_name_folder(&parent_folder, &identifier)
                        .is_ok(),
                )),
            },
            Command::Serialize => match yyp_boss.serialize() {
                Ok(()) => Ok(CommandOutput::ok()),
                Err(e) => Err(YypBossError::CouldNotSerializeYypBoss {
                    data: e.to_string(),
                }),
            },
            Command::Shutdown => {
                *shutdown_flag = true;
                Ok(CommandOutput::ok())
            }
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
            Err(e) => Err(YypBossError::ResourceManipulation {
                data: e.to_string(),
            }),
        }
    }

    fn remove<T: YyResource>(
        &self,
        yyp_boss: &mut YypBoss,
        resource_name: String,
    ) -> Result<CommandOutput, YypBossError> {
        match yyp_boss.remove_resource::<T>(&resource_name) {
            Ok(output) => match self.serialize_yy_data_for_output(&output.0, output.1.as_ref()) {
                Ok((yy, assoc)) => Ok(CommandOutput::ok_datum(yy, assoc)),
                Err(e) => Err(YypBossError::CouldNotOutputData {
                    data: e.to_string(),
                }),
            },
            Err(e) => Err(YypBossError::ResourceManipulation {
                data: e.to_string(),
            }),
        }
    }

    fn get_resource<T: YyResource>(
        &self,
        yyp_boss: &YypBoss,
        resource_name: String,
    ) -> Result<CommandOutput, YypBossError> {
        match yyp_boss.get_resource::<T>(&resource_name) {
            Some(output) => {
                let yy_data = SerializedData::Value {
                    data: serde_json::to_string_pretty(&output.yy_resource).unwrap(),
                };

                Ok(CommandOutput::ok_resource(yy_data))
            }

            None => Err(YypBossError::ResourceManipulation {
                data: ResourceManipulationError::BadGet.to_string(),
            }),
        }
    }

    fn ensure_associated_data<T: YyResource>(
        &self,
        yyp_boss: &mut YypBoss,
        resource_name: String,
        force: bool,
    ) -> Result<CommandOutput, YypBossError> {
        match yyp_boss.ensure_associated_data_is_loaded::<T>(&resource_name, force) {
            Ok(()) => {
                let output = yyp_boss.get_resource::<T>(&resource_name).unwrap();
                let data = output
                    .associated_data
                    .as_ref()
                    .expect("must have been loaded by above");

                match T::serialize_associated_data_into_data(&self.working_directory, data) {
                    Ok(assoc_data) => Ok(CommandOutput::ok_associated_data(assoc_data)),
                    Err(e) => Err(YypBossError::CouldNotOutputData {
                        data: e.to_string(),
                    }),
                }
            }

            Err(e) => Err(YypBossError::ResourceManipulation {
                data: e.to_string(),
            }),
        }
    }

    fn read_new_resource<T: YyResource>(
        &self,
        new_resource: NewResource,
        tcu: &TrailingCommaUtility,
    ) -> Result<(T, T::AssociatedData), YypBossError> {
        let value: T = match new_resource.new_resource {
            SerializedData::Value { data } => {
                serde_json::from_str(&data).map_err(|e| YypBossError::YyParseError {
                    data: e.to_string(),
                })
            }
            SerializedData::Filepath { data } => {
                let path = self.working_directory.join(data);
                utils::deserialize_json_tc(&path, tcu).map_err(|e| YypBossError::YyParseError {
                    data: e.to_string(),
                })
            }
            SerializedData::DefaultValue => Ok(T::default()),
        }?;

        let associated_data: T::AssociatedData = value
            .deserialize_associated_data_from_data(&new_resource.associated_data, tcu)
            .map_err(|e| YypBossError::AssociatedDataParseError {
                data: e.to_string(),
            })?;

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

    #[allow(clippy::unnecessary_wraps)]
    fn create_yy<T: YyResource>(cr: CreateCommand) -> Result<CommandOutput, YypBossError> {
        let mut yy = T::default();

        let CreateCommand {
            name,
            parent,
            resource: _,
        } = cr;

        if let Some(name) = name {
            yy.set_name(name);
        }

        if let Some(parent) = parent {
            yy.set_parent_view_path(parent);
        }

        Ok(CommandOutput::ok_resource(SerializedData::Value {
            data: serde_json::to_string_pretty(&yy).unwrap(),
        }))
    }
}
