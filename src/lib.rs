#![allow(clippy::bool_comparison)]

mod boss {
    use super::*;

    mod yy_resource;
    pub use yy_resource::{SerializedData, SerializedDataError, YyResource};

    mod yyp_boss;
    pub use yyp_boss::YypBoss;

    mod resources;
    pub use resources::Resource;

    mod yy_resource_handler;
    pub use yy_resource_handler::{YyResourceData, YyResourceHandler};

    mod directory_manager;
    mod utils;
    pub use utils::{FileSerializationError, SerializationFormat};

    mod errors;
    pub use errors::*;

    mod pipelines;
    pub use pipelines::{PipelineDesinations, PipelineError, PipelineManager};

    pub mod folders {
        mod folder_graph;
        pub use folder_graph::*;

        mod folder_graph_error;
        pub use folder_graph_error::*;

        mod folder_graph_manager;
        pub use folder_graph_manager::*;

        mod folder_graph_member;
        pub use folder_graph_member::*;
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

    pub type SpriteImageBuffer = image::ImageBuffer<image::Rgba<u8>, Vec<u8>>;
}

pub use boss::*;
pub use resources_ext::*;
pub use yy_typings;
