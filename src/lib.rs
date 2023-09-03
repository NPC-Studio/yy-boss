#![allow(clippy::bool_comparison)]
#![warn(elided_lifetimes_in_paths)]

mod yy_resource;
pub use yy_resource::{FileHolder, SerializedData, SerializedDataError, YyResource};

mod yyp_boss;
pub use yyp_boss::YypBoss;

mod resources;
pub use resources::Resource;

mod yy_resource_handler;
pub use yy_resource_handler::{YyResourceData, YyResourceHandler};

mod directory_manager;
pub mod utils;
pub use utils::{FileSerializationError, SerializationFormat};

mod project_metadata;
pub use project_metadata::ProjectMetadata;

mod errors;
pub use errors::*;

mod dirty_handler;

pub mod folders;

mod resources_ext {
    use super::*;

    mod sprite_ext;
    pub use sprite_ext::*;

    mod paths_ext;
    pub use paths_ext::*;

    mod yyp_serialization;
    pub use yyp_serialization::serialize_yyp;

    mod script_ext;
    pub use script_ext::*;

    mod object_ext;
    pub use object_ext::*;

    pub(crate) mod dummy;

    mod note_ext;
    pub use note_ext::*;

    mod shader_ext;
    pub use shader_ext::*;

    mod room_ext;
    pub use room_ext::*;

    mod sound_ext;
    pub use sound_ext::*;

    mod tile_set_ext;
    pub use tile_set_ext::*;

    mod unidentified_resources;
    pub use unidentified_resources::*;

    pub type SpriteImageBuffer = image::ImageBuffer<image::Rgba<u8>, Vec<u8>>;
}

pub mod cli;

pub use resources_ext::*;
pub use yy_typings;
