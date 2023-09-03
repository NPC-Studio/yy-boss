#![allow(clippy::bool_comparison)]
#![warn(elided_lifetimes_in_paths)]

mod directory_manager;
mod dirty_handler;

mod yy_resource;
pub use yy_resource::{FileHolder, SerializedData, SerializedDataError, YyResource};

mod yyp_boss;
pub use yyp_boss::YypBoss;

mod resources;
pub use resources::Resource;

mod yy_resource_handler;
pub use yy_resource_handler::{YyResourceData, YyResourceHandler};

pub mod utils;
pub use utils::{FileSerializationError, SerializationFormat};

mod project_metadata;
pub use project_metadata::ProjectMetadata;

mod errors;
pub use errors::*;

mod folders;
pub use folders::*;

mod resources_ext;
pub use resources_ext::*;

pub mod cli;

/// We re-export yy_typings to help dependencies by users.
pub use yy_typings;
