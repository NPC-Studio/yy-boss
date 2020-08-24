use serde::{Deserialize, Serialize};
use yy_boss::{Resource, SerializedData};
use yy_typings::{ViewPath, ViewPathLocation};

/// The type of command to give, pertaining to each of the general areas the YyBoss can give.
///
/// All commands return an [`Output`] with a `Command` tag except [`Shutdown`], which will return
/// an output with a [`Shutdown`] tag on it instead, after which the server will shutdown.
///
/// [`Output`]: ../output/enum.Output.html
/// [`Shutdown`]: ./struct.Shutdown.html
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Command {
    /// A command type pertaining to Resources. To see the subcommand for resources, see
    /// [`ResourceCommand`].
    ///
    /// [`ResourceCommand`]: ./struct.ResourceCommand.html
    Resource(ResourceCommand),

    /// A command type pertaining to the Virtual Filesystem. To see the subcommand for the VFS, see
    /// [`VfsCommand`].
    ///
    /// [`VfsCommand`]: ./struct.VfsCommand.html
    VirtualFileSystem(VfsCommand),
    // Serialization,
    // Shutdown,
}

/// A resource command, which will allow users to read and write resources
/// into the YypBoss.
///
/// The subtype of command, such as [`Add`] or [`Remove`] is indicated by the [`command_type`]
/// subfield.
///
/// The Resource type to manipulate is given by the [`resource`] field.
///
/// Each [`resource`] paired with each [`command_type`], indicates which of the optional fields is required. If
/// an optional field is not given, but should have been, this command will abort.
///
/// [`Add`]: ./enum.ResourceCommandType.html#variant.Add
/// [`Remove`]: ./enum.ResourceCommandType.html#variant.Remove
/// [`command_type`]: #structfield.command_type
/// [`resource`]: #structfield.command_type
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename = "subCommand")]
pub struct ResourceCommand {
    /// The command type for this ResourceCommand.
    #[serde(flatten)]
    pub command_type: ResourceCommandType,

    /// The type of resource to interact with.
    pub resource: Resource,
}

/// The command type to run.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(tag = "subCommand")]
pub enum ResourceCommandType {
    /// Adds a resource to the project.
    ///
    /// ## Errors
    /// If there is a resource by the name already, this command will abort and return
    /// an error.
    ///
    /// ## Returns
    /// If it succeeds, it will return without any extra data, like a `void`.
    Add(NewResource),

    // /// Replaces a resource in the project.
    // ///
    // /// ## Errors
    // /// If there is no resource by that name already, this command will abort and return
    // /// an error.
    // ///
    // /// ## Returns
    // /// If it succeeds, it will return the resource and its associated data
    // /// after having replaced it.
    // Replace(NewResource),

    // /// Sets a resource in a project, regardless of the current resources in the project.
    // /// Functionally, this will replace any resource of the same name, or add a new resource.
    // /// Users can think of this command as a "forceAdd".
    // ///
    // /// ## Errors
    // /// This command is infallible.
    // ///
    // /// ## Returns
    // /// This command returns without any extra data. If a User wants the resource data
    // /// which was present, they will have to run [`Exists`] and then [`Replace`] as two commands.
    // ///
    // /// [`Exists`]: #variant.Exists
    // /// [`Replace`]: #variant.Replace
    // Set(NewResource),
    /// Removes and returns the resource.
    ///
    /// ## Errors
    /// If there isn't a resource by that name of the type given, it will return an error.
    ///
    /// ## Returns
    /// If this command succeeds, it will return the resource and its associated data
    /// after having removed it.
    Remove {
        /// The name of the resource to remove.
        identifier: String,
    },

    /// Returns a copy of a resource and its associated data.
    ///
    /// ## Errors
    /// If there isn't a resource by the given name of the given type, an error will be returned.
    ///
    /// ## Returns
    /// If this command succeeds, it will return a copy of the resource and its associated data.
    /// This command will not mutate any data in the project.
    Get {
        /// The name of the resource to get.
        identifier: String,
    },

    /// Returns a boolean indicating if a resource of the given name and given type exists.
    /// This command is a shortcut for performance reasons over [`Get`], since no string writing and
    /// serialization/deserialization will be required.
    ///
    /// ## Errors
    /// This command is infallible.
    ///
    /// ## Returns
    /// This command will return `true` if a resource of the given name and given type exists,
    /// and `false` otherwise.
    ///
    /// [`Get`]: #variant.Get
    Exists {
        /// The name of the resource to check if it exists.
        identifier: String,
    },
}

/// This struct describes the new data needed to [`Add`], [`Replace`], or [`Set`] a resource
/// in the [`ResourceCommandType`].
///
/// The types of the Data required for [`new_resource`] and [`associated_data`] are written in the table
/// below for convenience, but for the absolute answer, look in the [`resources_ext`] module.
///
/// ## Types of Each Field
///|   Resource Type  |   new_resource     | associated_data   |
///|------------------|-----------------------|----------------------|
///| [`Sprite`]       |  [`Sprite Yy File`]   | [`HashMap`]<[`Frame Uuid`], [`SpriteImageBuffer`]>  |
///| [`Object`]       |  [`Object Yy File`]   | [`HashMap`]<[`EventType`], [`String`]>              |
///| [`Script`]       |  [`Script Yy File`]   | String                                       |
///| [`Shader`]       |  [`Shader Yy File`]   | [`ShaderScriptType`] |
///
/// **NB:** Above, "Single-Void" means that a given Map must have only one key (if multiple are present,
/// the command will abort with an error), and the contents of the key do not matter. Using `data` might
/// be a good idea for users, but any name is fine.
///
/// [`Add`]: ./enum.ResourceCommandType.html#variant.Add
/// [`Replace`]: ./enum.ResourceCommandType.html#variant.Replace
/// [`Set`]: ./enum.ResourceCommandType.html#variant.Set
/// [`ResourceCommandType`]: ./enum.ResourceCommandType.html
/// [`new_resource`]: #structfield.new_resource
/// [`associated_data`]: #structfield.associated_data
/// [`Sprite`]: ./enum.Resource.html#variant.Sprite
/// [`Object`]: ./enum.Resource.html#variant.Object
/// [`Script`]: ./enum.Resource.html#variant.Script
/// [`Shader`]: ./enum.Resource.html#variant.Shader
/// [`Sprite Yy File`]: ../../yy_typings/sprite_yy/struct.Sprite.html
/// [`Object Yy File`]: ../../yy_typings/object_yy/struct.Object.html
/// [`Script Yy File`]: ../../yy_typings/struct.Script.html
/// [`Shader Yy File`]: ./error.html
/// [`Frame Uuid`]: ../../yy_typings/sprite_yy/struct.Frame.html#structfield.name
/// [`EventType`]: ../../yy_typings/sprite_yy/object_yy/enum.EventType.html
/// [`ShaderScriptType`]: ./error.html
/// [`HashMap`]: ../../../std/collects/struct.HashMap.html
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct NewResource {
    /// This field must contain the Data of a Yy File to add for the given resource.
    ///
    /// See the chart in [`NewResource`] above for more details.
    ///
    /// [`NewResource`]: ./struct.NewResource.html
    pub new_resource: SerializedData,

    /// This fields must contain the Associated Data of a given Yy File.
    ///
    /// See the chart in [`NewResource`] above for more details of which
    /// types of associated data are expected.
    ///
    /// [`NewResource`]: ./struct.NewResource.html
    pub associated_data: SerializedData,
}

/// The Virtual File System command type to run.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(tag = "subCommand")]
pub enum VfsCommand {
    /// An instruction to move a Resource from one location to another.
    ///
    /// ## Errors
    /// If the [`resource_to_move`] field is not set to a valid resource, or
    /// if [`new_parent`] is not set to a valid folder, this command aborts and
    /// returns an error.
    ///
    /// [`resource_to_move`]: #structfield.resource_to_move
    /// [`new_parent`]: #structfield.new_parent
    MoveResource {
        /// The name of the Resource to move.
        resource_to_move: String,
        /// The kind of the Resource to move.
        resource: Resource,
        /// The new parent of the Resource, which must be a folder.
        new_parent: ViewPath,
    },

    /// An instruction to move a Folder from one location to another, along with the
    /// resources within it.
    ///
    /// ## Errors
    /// If the [`folder_to_move`] or [`new_parent`] fields are not set to a valid folder,
    /// this command aborts and returns an error. If `new_parent` is a folder within `folder_to_move`,
    /// this command will abort and return a error.
    ///
    /// [`folder_to_move`]: #structfield.folder_to_move
    /// [`new_parent`]: #structfield.new_parent
    MoveFolder {
        /// The location of the Resource to move.
        folder_to_move: ViewPathLocation,
        /// The new parent of the Resource, which must be a folder.
        new_parent: ViewPathLocation,
    },

    /// Deletes a folder.
    ///
    /// If the folder is not empty, then the `recursive` flag must be passed -- otherwise, this command
    /// will abort.
    RemoveFolder {
        /// The location of the Folder to remove.
        folder_to_remove: ViewPathLocation,

        /// If the folder is not empty, and this flag is not set to true, then the command will abort with
        /// an error.
        ///
        /// Since the YypBoss is modeled as a black box, there is no notion of "force" -- otherwise, this command
        /// is similar to `rm -rf` on Unix systems.
        recursive: bool,
    },

    /// Returns a [`FolderGraph`] for this folder.
    ///
    /// ## Errors
    /// If the [`ViewPathLocation`] provided does not describe a valid Folder, this command aborts and returns an error.
    GetFolder(ViewPathLocation),

    /// Returns a [`FolderGraph`] for the entire Virtual File System.
    /// Please note, this can result in a fairly massive Json being sent back.
    ///
    /// ## Errors
    /// This command is infallible.
    GetFullVfs,

    /// Returns an enum if the given ViewPath links to a Folder or a File.
    ///
    /// ## Errors
    /// If the [`ViewPath`] provided does not describe a valid Item, this command aborts and returns an error.
    GetPathType(ViewPath),
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn test() {
        fn harness(command: Command) {
            let command_str = serde_json::to_string_pretty(&command).unwrap();
            println!("{}", command_str);

            let new_command: Command = serde_json::from_str(&command_str).unwrap();
            assert_eq!(new_command, command);
        }
        harness(Command::Resource(ResourceCommand {
            command_type: ResourceCommandType::Add(NewResource {
                new_resource: SerializedData::Value {
                    data: "Hello".to_string(),
                },
                associated_data: SerializedData::Value {
                    data: json!({
                        "test": "10"
                    })
                    .to_string(),
                },
            }),
            resource: Resource::Script,
        }));

        harness(Command::Resource(ResourceCommand {
            command_type: ResourceCommandType::Get {
                identifier: "Something".to_string(),
            },
            resource: Resource::Sprite,
        }));

        harness(Command::VirtualFileSystem(VfsCommand::MoveResource {
            resource_to_move: "jim".to_string(),
            resource: Resource::Script,
            new_parent: ViewPath::default(),
        }));

        harness(Command::VirtualFileSystem(VfsCommand::RemoveFolder {
            recursive: true,
            folder_to_remove: ViewPathLocation::new("okay"),
        }));

        harness(Command::VirtualFileSystem(VfsCommand::GetFolder(
            ViewPathLocation::default(),
        )));

        harness(Command::VirtualFileSystem(VfsCommand::GetFullVfs));
        harness(Command::VirtualFileSystem(VfsCommand::GetPathType(
            ViewPath::default(),
        )));
    }
}
