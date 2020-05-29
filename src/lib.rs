mod boss {
    use super::*;

    mod yy_resource;
    pub use yy_resource::YyResource;

    mod yyp_boss;
    pub use yyp_boss::YypBoss;

    mod yy_resource_handler;
    pub use yy_resource_handler::YyResourceHandler;

    mod utils;

    mod folder_graph;
    pub use folder_graph::FolderGraph;
}
mod resources_ext {
    use super::*;

    mod sprite_ext;
    pub use sprite_ext::*;

    mod yyp_ext;
    pub use yyp_ext::*;

    pub type SpriteImageBuffer = image::ImageBuffer<image::Rgba<u8>, Vec<u8>>;
}

pub use boss::*;
pub use resources_ext::*;
pub use yy_typings;
