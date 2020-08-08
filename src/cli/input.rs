use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};
use yy_boss::Resource;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum InputCommand {
    Resource(ResourceCommand),
    Serialization,
    VirtualFileSystem,
    Shutdown,
}

/// A resource command, which will allow users to read and write resources
/// into the YypBoss.
///
/// The subtype of command, such as `Add` and `Remove` is indicated by the `command_type`
/// subfield.
///
/// The Resource type to manipulate is given by the `resource` field.
///
/// Additionally, there are three optional fields, `new_resource`, `associated_data`, and `resource_name.`
///
/// Each `Resource`, paired with each `command_type`, indicates which of the optional fields is required. If
/// an optional field is not given, but should have been, an Error will be returned and no operation will be
/// performed.
///
/// ## Types of Optional Fields
///|   Resource Type  |   NewResourceType  | AssociatedData Key   |  AssociatedData Value   |
///|------------------|--------------------|----------------------| ----------------------- |
///| Sprite           |  Sprite Yy File   | Frame Uuid in Yy File |  Png                    |
///| Object           |  Object Yy File   | EventType             |  Gml                    |
///| Script           |  Script Yy File   | Void (can be anything |  Gml                    |
#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceCommand {
    pub command_type: ResourceCommandType,
    pub resource: Resource,
    pub new_resource: Option<Data>,
    pub associated_data: Option<HashMap<String, Data>>,
    pub resource_name: Option<String>,
}

/// The command type which will be given.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "command_type")]
pub enum ResourceCommandType {
    /// Adds a resource to the project.
    ///
    /// ## Required Data
    /// The `ResourceCommand` must have `new_resource` AND `associated_data`
    /// for the given resource type. See `ResourceCommand` for the type of data needed
    /// for each given resource.
    ///
    /// ## Errors
    /// If there is a resource
    /// by the name already, this command will fail and error out without
    /// changing any data.
    ///
    /// ## Returns
    /// If it succeeds, it will return without any extra data, like a `void`.
    Add,

    /// Replaces a resource in the project.
    ///
    /// ## Required Data
    /// The `ResourceCommand` must have `new_resource` AND `associated_data`
    /// for the given resource type. See `ResourceCommand` for the type of data needed
    /// for each given resource.
    ///
    /// ## Errors
    /// If there is **no** resource by that name and of the resource type given,
    /// this command will fail and error out, without changing any data.
    ///
    /// ## Returns
    /// If it succeeds, it will return the resource and its associated data
    /// after having replaced it.
    Replace,

    /// Sets a resource in a project, regardless of the current resources in the project.
    /// Functionally, this will replace any resource of the same name, or add a new resource.
    /// Users can think of this command as a "forceAdd".
    ///
    /// ## Required Data
    /// The `ResourceCommand` must have `new_resource` AND `associated_data`
    /// for the given resource type. See `ResourceCommand` for the type of data needed
    /// for each given resource.
    ///
    /// ## Errors
    /// This command is infallible.
    ///
    /// ## Returns
    /// This command returns without any extra data. If a User wants the resource data
    /// which was present, they will have to run `Exists` and then `Replace` as two commands.
    Set,

    /// Removes and returns the resource.
    ///
    /// ## Required Data
    /// The `ResourceCommand` must have `resource_name`, which is the human readable name
    /// for the Resource, such as "spr_player".
    ///
    /// ## Errors
    /// If there isn't a resource by that name of the type given, it will return an error.
    ///
    /// ## Returns
    /// If this command succeeds, it will return the resource and its associated data
    /// after having removed it.
    Remove,

    /// Returns a copy of a resource and its associated data.
    ///
    /// ## Required Data
    /// The `ResourceCommand` must have `resource_name`, which is the human readable name
    /// for the Resource, such as "spr_player".
    ///
    /// ## Errors
    /// If there isn't a resource by the given name of the given type, an error will be returned.
    ///
    /// ## Returns
    /// If this command succeeds, it will return a copy of the resource and its associated data.
    /// This command will not mutate any data in the project.
    Get,

    /// Returns a boolean indicating if a resource of the given name and given type exists.
    /// This command is a shortcut for performance reasons over `Get`, since no string writing and
    /// serialization/deserialization will be required.
    ///
    /// ## Required Data
    /// The `ResourceCommand` must have `resource_name`, which is the human readable name
    /// for the Resource, such as "spr_player".
    ///
    /// ## Errors
    /// This command is infallible.
    ///
    /// ## Returns
    /// This command will return `true` if a resource of the given name and given type exists,
    /// and `false` otherwise.
    Exists,
}

/// The data which is passed in as part of a Command. Each tag represents a different way to
/// pass data into the given Resource.
///
/// **NB:** the type of data which is passed in is determined by the containing Command. In a `ResourceCommand`,
/// for example, it is determined by the `Resource` which is passed in; for the `VirtualFileSystemCommand`, it is
/// determined by the `FileOrFolder` tag. See each documentation for more details.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "data_type")]
pub enum Data {
    /// The data itself, represented in some valid utf8 format. Scripts, yyfiles, and most resources
    /// will likely be passed in with this tag.
    ///
    /// It is an error to try to pass in any binary data which cannot be represented in utf8 format.
    /// To pass in images and other similar files, prefer using `Filepath`.
    Value(String),

    /// A path to the data iself, read from the ManagedDirectory. Symbolic links will not be followed.
    ///
    /// Anything, including gml and yy files, can be passed in with this tag, though its use is primarily
    /// for binary files, such as images and sound files.
    Filepath(PathBuf),

    /// A default for the type of data provided, which the YypBoss will generate for users.
    ///
    /// For example, to create a new Script, a user would want the Script's AssociatedData, which is the gml
    /// file, to be blank. This will generate such an empty string.
    /// In a more complex example, if a user is making a new Object, and is fine with default settings,
    /// included an autogenerated name, this tag will do that. Since all data can be edited afterwards,
    /// this can provide a convenient way to generate new objects.
    DefaultValue,
}
