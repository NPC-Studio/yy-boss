mod input;
pub use input::*;

mod output;
pub use output::*;

use crate::{
    folders::FolderGraphError, utils, ResourceManipulationError, SerializedData,
    SerializedDataError,
};
use crate::{Resource, YyResource, YypBoss};
use camino::{Utf8Path, Utf8PathBuf};
use yy_typings::{
    object_yy::{EventType, Object},
    script::Script,
    shader::Shader,
    sound::Sound,
    sprite_yy::Sprite,
    utils::TrailingCommaUtility,
    AnimationCurve, Extension, Font, Note, Path, Room, Sequence, TileSet, Timeline,
};

pub fn parse_command(
    command: Command,
    working_directory: &Utf8Path,
    yyp_boss: &mut YypBoss,
) -> Output {
    let command_output = match command {
        Command::Resource(resource_command) => match resource_command.command_type {
            ResourceCommandType::Add(new_resource) => match resource_command.resource {
                Resource::Sprite => add::<Sprite>(yyp_boss, working_directory, new_resource),
                Resource::Script => add::<Script>(yyp_boss, working_directory, new_resource),
                Resource::Object => add::<Object>(yyp_boss, working_directory, new_resource),
                Resource::Note => add::<Note>(yyp_boss, working_directory, new_resource),
                Resource::Shader => add::<Shader>(yyp_boss, working_directory, new_resource),
                Resource::AnimationCurve => {
                    add::<AnimationCurve>(yyp_boss, working_directory, new_resource)
                }
                Resource::Extension => add::<Extension>(yyp_boss, working_directory, new_resource),
                Resource::Font => add::<Font>(yyp_boss, working_directory, new_resource),
                Resource::Room => add::<Room>(yyp_boss, working_directory, new_resource),
                Resource::Path => add::<Path>(yyp_boss, working_directory, new_resource),
                Resource::Sequence => add::<Sequence>(yyp_boss, working_directory, new_resource),
                Resource::Sound => add::<Sound>(yyp_boss, working_directory, new_resource),
                Resource::TileSet => add::<TileSet>(yyp_boss, working_directory, new_resource),
                Resource::Timeline => add::<Timeline>(yyp_boss, working_directory, new_resource),
            },
            ResourceCommandType::Remove { identifier } => match resource_command.resource {
                Resource::Sprite => remove::<Sprite>(yyp_boss, working_directory, identifier),
                Resource::Script => remove::<Script>(yyp_boss, working_directory, identifier),
                Resource::Object => remove::<Object>(yyp_boss, working_directory, identifier),
                Resource::Note => remove::<Note>(yyp_boss, working_directory, identifier),
                Resource::Shader => remove::<Shader>(yyp_boss, working_directory, identifier),
                Resource::AnimationCurve => {
                    remove::<AnimationCurve>(yyp_boss, working_directory, identifier)
                }
                Resource::Extension => remove::<Extension>(yyp_boss, working_directory, identifier),
                Resource::Font => remove::<Font>(yyp_boss, working_directory, identifier),
                Resource::Path => remove::<Path>(yyp_boss, working_directory, identifier),
                Resource::Room => remove::<Room>(yyp_boss, working_directory, identifier),
                Resource::Sequence => remove::<Sequence>(yyp_boss, working_directory, identifier),
                Resource::Sound => remove::<Sound>(yyp_boss, working_directory, identifier),
                Resource::TileSet => remove::<TileSet>(yyp_boss, working_directory, identifier),
                Resource::Timeline => remove::<Timeline>(yyp_boss, working_directory, identifier),
            },
            ResourceCommandType::Rename {
                identifier,
                new_name,
            } => {
                let output = match resource_command.resource {
                    Resource::Sprite => yyp_boss.rename_resource::<Sprite>(&identifier, new_name),
                    Resource::Script => yyp_boss.rename_resource::<Script>(&identifier, new_name),
                    Resource::Object => yyp_boss.rename_resource::<Object>(&identifier, new_name),
                    Resource::Note => yyp_boss.rename_resource::<Note>(&identifier, new_name),
                    Resource::Shader => yyp_boss.rename_resource::<Shader>(&identifier, new_name),
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
                    Resource::TileSet => yyp_boss.rename_resource::<TileSet>(&identifier, new_name),
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
                Resource::Sprite => get_resource::<Sprite>(yyp_boss, identifier),
                Resource::Script => get_resource::<Script>(yyp_boss, identifier),
                Resource::Object => get_resource::<Object>(yyp_boss, identifier),
                Resource::Note => get_resource::<Note>(yyp_boss, identifier),
                Resource::Shader => get_resource::<Shader>(yyp_boss, identifier),
                Resource::Room => get_resource::<Room>(yyp_boss, identifier),
                Resource::TileSet => get_resource::<TileSet>(yyp_boss, identifier),
                Resource::AnimationCurve
                | Resource::Extension
                | Resource::Font
                | Resource::Path
                | Resource::Sequence
                | Resource::Sound
                | Resource::Timeline => Err(YypBossError::ResourceManipulation {
                    data: ResourceManipulationError::ResourceCannotBeManipulated.to_string(),
                }),
            },
            ResourceCommandType::GetAssociatedData { identifier, force } => {
                match resource_command.resource {
                    Resource::Sprite => ensure_associated_data::<Sprite>(
                        yyp_boss,
                        working_directory,
                        identifier,
                        force,
                    ),
                    Resource::Script => ensure_associated_data::<Script>(
                        yyp_boss,
                        working_directory,
                        identifier,
                        force,
                    ),
                    Resource::Object => ensure_associated_data::<Object>(
                        yyp_boss,
                        working_directory,
                        identifier,
                        force,
                    ),
                    Resource::Note => ensure_associated_data::<Note>(
                        yyp_boss,
                        working_directory,
                        identifier,
                        force,
                    ),
                    Resource::Shader => ensure_associated_data::<Shader>(
                        yyp_boss,
                        working_directory,
                        identifier,
                        force,
                    ),

                    Resource::Room => ensure_associated_data::<Room>(
                        yyp_boss,
                        working_directory,
                        identifier,
                        force,
                    ),

                    Resource::TileSet => ensure_associated_data::<TileSet>(
                        yyp_boss,
                        working_directory,
                        identifier,
                        force,
                    ),

                    Resource::AnimationCurve
                    | Resource::Extension
                    | Resource::Font
                    | Resource::Path
                    | Resource::Sequence
                    | Resource::Sound
                    | Resource::Timeline => Err(YypBossError::ResourceManipulation {
                        data: ResourceManipulationError::ResourceCannotBeManipulated.to_string(),
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
            } => match yyp_boss.move_resource_dynamic(&resource_to_move, new_parent, resource) {
                Ok(()) => Ok(CommandOutput::ok()),
                Err(e) => Err(YypBossError::ResourceManipulation {
                    data: e.to_string(),
                }),
            },
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
                Resource::Sprite => create_yy::<Sprite>(create_data),
                Resource::Script => create_yy::<Script>(create_data),
                Resource::Object => create_yy::<Object>(create_data),
                Resource::Note => create_yy::<Note>(create_data),
                Resource::Shader => create_yy::<Shader>(create_data),
                Resource::Room => create_yy::<Room>(create_data),
                Resource::TileSet => create_yy::<TileSet>(create_data),
                Resource::AnimationCurve
                | Resource::Extension
                | Resource::Font
                | Resource::Path
                | Resource::Sequence
                | Resource::Sound
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
                    match yyp_boss.ensure_associated_data_is_loaded::<Object>(&identifier, false) {
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
                    match yyp_boss.ensure_associated_data_is_loaded::<Object>(&identifier, false) {
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
                        .join(format!("{}.gml", script.yy_resource.common_data.name));

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
    };
    Output::Command(command_output.unwrap_or_else(CommandOutput::error))
}

fn add<T: YyResource>(
    yyp_boss: &mut YypBoss,
    working_directory: &Utf8Path,
    new_resource: NewResource,
) -> Result<CommandOutput, YypBossError> {
    let (yy_file, associated_data) =
        read_new_resource::<T>(new_resource, yyp_boss.tcu(), working_directory)?;

    // check for a bad add...
    match yyp_boss.add_resource(yy_file, associated_data) {
        Ok(()) => Ok(CommandOutput::ok()),
        Err(e) => Err(YypBossError::ResourceManipulation {
            data: e.to_string(),
        }),
    }
}

fn remove<T: YyResource>(
    yyp_boss: &mut YypBoss,
    working_directory: &Utf8Path,
    resource_name: String,
) -> Result<CommandOutput, YypBossError> {
    match yyp_boss.remove_resource::<T>(&resource_name) {
        Ok(output) => {
            match serialize_yy_data_for_output(&output.0, working_directory, output.1.as_ref()) {
                Ok((yy, assoc)) => Ok(CommandOutput::ok_datum(yy, assoc)),
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

fn get_resource<T: YyResource>(
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
    yyp_boss: &mut YypBoss,
    working_directory: &Utf8Path,
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

            match T::serialize_associated_data_into_data(working_directory.as_std_path(), data) {
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
    new_resource: NewResource,
    tcu: &TrailingCommaUtility,
    working_directory: &Utf8Path,
) -> Result<(T, T::AssociatedData), YypBossError> {
    let value: T = match new_resource.new_resource {
        SerializedData::Value { data } => {
            serde_json::from_str(&data).map_err(|e| YypBossError::YyParseError {
                data: e.to_string(),
            })
        }
        SerializedData::Filepath { data } => {
            let data = Utf8PathBuf::from_path_buf(data).expect("non-utf8 path");
            let path = working_directory.join(data);
            utils::deserialize_json_tc(path, tcu).map_err(|e| YypBossError::YyParseError {
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
    yy: &T,
    working_directory: &Utf8Path,
    assoc_data: Option<&T::AssociatedData>,
) -> Result<(SerializedData, Option<SerializedData>), SerializedDataError> {
    let yy_data = SerializedData::Value {
        data: serde_json::to_string_pretty(&yy).unwrap(),
    };

    let assoc_output = if let Some(data) = assoc_data {
        Some(T::serialize_associated_data_into_data(
            working_directory.as_std_path(),
            data,
        )?)
    } else {
        None
    };

    Ok((yy_data, assoc_output))
}

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
