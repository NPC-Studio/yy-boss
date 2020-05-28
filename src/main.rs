pub mod boss {
    use super::*;

    mod yy_resource;
    mod yyp_boss;

    use yy_resource::YyResource;
    pub use yyp_boss::YypBoss;
    mod folder_graph;
    pub use folder_graph::FolderGraph;
    mod resources_ext {
        use super::*;

        mod sprite_ext;
        pub use sprite_ext::*;

        pub type SpriteImageBuffer = image::ImageBuffer<image::Rgba<u8>, Vec<u8>>;
    }
    pub use resources_ext::*;
}

fn main() {
    
}
