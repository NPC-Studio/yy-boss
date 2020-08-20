#![allow(clippy::bool_comparison)]
#![warn(elided_lifetimes_in_paths)]
mod boss {
    use super::*;

    mod yy_resource;
    pub use yy_resource::{AssocDataLocation, SerializedData, SerializedDataError, YyResource};

    mod yyp_boss;
    pub use yyp_boss::YypBoss;

    mod resources;
    pub use resources::Resource;

    mod yy_resource_handler;
    pub use yy_resource_handler::{YyResourceData, YyResourceHandler};

    mod directory_manager;
    pub mod utils;
    pub use utils::{FileSerializationError, SerializationFormat};

    mod errors;
    pub use errors::*;

    mod pipelines;
    pub use pipelines::{PipelineDesinations, PipelineError, PipelineManager};

    pub mod folders {
        use super::utils;

        mod folder_graph;
        pub use folder_graph::*;

        mod folder_graph_error;
        pub use folder_graph_error::*;

        mod vfs;
        pub use vfs::*;

        mod resource_names;
        pub use resource_names::*;

        mod file;
        pub(crate) use file::*;
    }
}

mod resources_ext {
    use super::*;

    mod sprite_ext;
    pub use sprite_ext::*;

    mod paths_ext;
    pub use paths_ext::*;

    mod yyp_serialization;
    pub use yyp_serialization::*;

    mod script_ext;
    pub use script_ext::*;

    mod object_ext;
    pub use object_ext::*;

    pub(crate) mod dummy;

    pub type SpriteImageBuffer = image::ImageBuffer<image::Rgba<u8>, Vec<u8>>;
}

pub use boss::*;
pub use resources_ext::*;
pub use yy_typings;
