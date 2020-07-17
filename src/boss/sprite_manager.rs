use super::{
    directory_manager::DirectoryManager, resource_handler::ResourceHandler,
    yy_resource::CreatedResource,
};
use crate::{SpriteImageBuffer, YyResourceHandler};
use anyhow::Result as AnyResult;
use yy_typings::sprite::{FrameId, Sprite};

#[derive(Debug)]
pub struct SpriteManager {
    sprites: YyResourceHandler<Sprite>,
}

impl SpriteManager {
    /// Loads a sprite in on startup.
    pub(crate) fn load_in(&mut self, sprite_yy: Sprite) {
        self.sprites.add_new_startup(sprite_yy, None);
    }

    /// Add a sprite into the YYP Boss. If the sprite doesn't exist, returns an error.
    pub fn replace_sprite(
        &mut self,
        sprite: Sprite,
        associated_data: Vec<(FrameId, SpriteImageBuffer)>,
    ) -> AnyResult<()> {
        self.sprites.overwrite(sprite, associated_data)
    }

    /// Add a sprite into the YYP Boss. It is not immediately serialized,
    /// but will be serialized the next time the entire YYP Boss is.
    ///
    /// Please note -- the name of the Sprite MIGHT change if that name already exists!
    pub fn add_sprite(
        &mut self,
        sprite: Sprite,
        created_resource: CreatedResource,
        associated_data: Vec<(FrameId, SpriteImageBuffer)>,
    ) {
        self.sprites.add_new(sprite, associated_data)?;

        Ok(())

        // match self.add_file_at_end(
        //     sprite.parent_path(),
        //     sprite.name.clone(),
        //     sprite.filesystem_path(),
        // ) {
        //     Ok(order) => {
        //     }
        //     Err(e) => {
        //         log::error!(
        //             "Couldn't add Sprite {}. It reported a parent path of {:#?}, and an FS path of {:#?}.\n\
        //         Error was: {:}",
        //             sprite.name,
        //             sprite.parent_path(),
        //             sprite.filesystem_path(),
        //             e
        //         );

        //         if let Err(e) = self.add_file_at_end(
        //             self.root_path(),
        //             sprite.name.clone(),
        //             sprite.filesystem_path(),
        //         ) {
        //             log::error!(
        //                 "And we couldn't even add to root! {:}. Aborting operation...",
        //                 e
        //             );
        //         }

        //         Err(e.into())
        //     }
        // }
    }

    // /// Removes a given sprite from the game. If the sprite existed, a `YyResourceData<Sprite>`
    // /// will be returned.
    // pub fn remove_sprite(
    //     &mut self,
    //     sprite: FilesystemPath,
    // ) -> Option<(Sprite, Option<<Sprite as YyResource>::AssociatedData>)> {
    //     self.remove_resource(&sprite);
    //     self.sprites.remove(&sprite).map(|i| i.into())
    // }

    // /// This gets the data on a given Sprite with a given name, if it exists.
    // pub fn get_sprite(&self, sprite_name: &str) -> Option<&Sprite> {
    //     if self.resource_names.contains(sprite_name) == false {
    //         return None;
    //     }

    //     // Get the path
    //     let path = self.yyp.resources.iter().find_map(|yypr| {
    //         if yypr.id.name == sprite_name {
    //             Some(&yypr.id)
    //         } else {
    //             None
    //         }
    //     });

    //     path.and_then(|path| {
    //         self.sprites
    //             .resources
    //             .get(path)
    //             .map(|sprite_resource| &sprite_resource.yy_resource)
    //     })
    // }
}

impl ResourceHandler for SpriteManager {
    fn new() -> SpriteManager {
        SpriteManager {
            sprites: YyResourceHandler::new(),
        }
    }

    fn serialize(&mut self, directory_manager: &DirectoryManager) -> AnyResult<()> {
        self.sprites.serialize(directory_manager)
    }
}
