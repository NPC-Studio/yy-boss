use super::{
    directory_manager::DirectoryManager, folder_graph::*, pipelines::PipelineManager, utils,
    FolderGraph, PathStrExt, SpriteImageBuffer, ViewPathLocationExt, YyResource, YyResourceHandler,
    YypSerialization,
};
use anyhow::{format_err, Context, Result as AnyResult};
use log::*;
use std::{collections::HashSet, fs, path::Path};
use yy_typings::{sprite::*, utils::TrailingCommaUtility, Yyp};

#[derive(Debug)]
pub struct YypBoss {
    yyp: Yyp,
    pub directory_manager: DirectoryManager,
    sprites: YyResourceHandler<Sprite>,
    folder_graph: FolderGraph,
    resource_names: HashSet<String>,
    tcu: TrailingCommaUtility,
    pub pipeline_manager: PipelineManager,
    dirty: bool,
}

impl YypBoss {
    /// Creates a new YyBoss Manager and performs startup file reading.
    pub fn new(path_to_yyp: &Path) -> AnyResult<YypBoss> {
        let tcu = TrailingCommaUtility::new();
        let yyp = utils::deserialize(path_to_yyp, Some(&tcu)).with_context(|| "on the yyp")?;

        let directory_manager = DirectoryManager::new(path_to_yyp)?;

        let mut yyp_boss = Self {
            yyp,
            dirty: false,
            sprites: YyResourceHandler::new(),
            folder_graph: FolderGraph::root(),
            resource_names: HashSet::new(),
            tcu,
            pipeline_manager: PipelineManager::new(&directory_manager)?,
            directory_manager: DirectoryManager::new(path_to_yyp)?,
        };

        // Load in Folders
        for new_folder in yyp_boss.yyp.folders.iter() {
            let mut folder_graph = &mut yyp_boss.folder_graph;

            for section in new_folder.folder_path.component_paths() {
                let parent_path = folder_graph.view_path();
                let section = section.trim_yy().to_owned();
                let entry = folder_graph.folders.entry(section.clone());

                let new_member = entry.or_insert(SubfolderMember {
                    child: FolderGraph::new(section, parent_path),
                    order: new_folder.order,
                });

                folder_graph = &mut new_member.child;
            }
        }

        // Load in Sprites
        for sprite_resource in yyp_boss
            .yyp
            .resources
            .iter()
            .filter(|value| value.id.path.starts_with("sprites"))
        {
            let sprite_path = yyp_boss
                .directory_manager
                .root_directory()
                .join(&sprite_resource.id.path);

            let sprite_yy: Sprite = utils::deserialize(&sprite_path, Some(&yyp_boss.tcu))?;

            // Add to the folder graph
            yyp_boss
                .folder_graph
                .find_subfolder_mut(&sprite_yy.parent)?
                .files
                .insert(
                    sprite_yy.name.clone(),
                    FileMember {
                        child: sprite_yy.filesystem_path(),
                        order: sprite_resource.order,
                    },
                );

            yyp_boss.resource_names.insert(sprite_yy.name.clone());
            yyp_boss.sprites.add_new_startup(sprite_yy, None);
        }

        // Ensure the directory
        // Self::ensure_yyboss_data(&yyp_boss.directory_manager)?;

        Ok(yyp_boss)
    }

    /// Gets the default texture path, if it exists. The "Default" group simply
    /// has the name `"Default"`.
    pub fn default_texture_path(&self) -> Option<TexturePath> {
        self.yyp
            .texture_groups
            .iter()
            .find(|tex| tex.name == "Default")
            .map(|texture_group| texture_group.into())
    }

    // /// Creates a texture group with the given name and rename preference.
    // /// If it does exist, it returns an error.
    // pub fn create_texture_group(&self) -> Option<TexturePath> {
    //     self.yyp
    //         .texture_groups
    //         .iter()
    //         .find(|tex| tex.name == "Default")
    //         .map(|texture_group| texture_group.into())
    // }

    /// Gets an unordered HashSet of currently used resource names.
    ///
    /// In a project
    /// with a sprite `spr_player` and an object `obj_player`, this HashSet would contain
    /// `"spr_player"` and `"obj_player"`.
    pub fn current_resource_names(&self) -> &HashSet<String> {
        &self.resource_names
    }

    /// Add a sprite into the YYP Boss. If the sprite doesn't exist, throws an error!
    pub fn overwrite_sprite(
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
        mut sprite: Sprite,
        associated_data: Vec<(FrameId, SpriteImageBuffer)>,
    ) {
        match self.add_file_at_end(
            sprite.parent_path(),
            sprite.name.clone(),
            sprite.filesystem_path(),
        ) {
            Ok(order) => {
                self.add_new_resource(&mut sprite, order);
                self.sprites.add_new(sprite, associated_data);
            }
            Err(e) => {
                log::error!(
                    "Couldn't add Sprite {}. It reported a parent path of {:#?}, and an FS path of {:#?}.\n\
                Error was: {:}",
                    sprite.name,
                    sprite.parent_path(),
                    sprite.filesystem_path(),
                    e
                );

                if let Err(e) = self.add_file_at_end(
                    self.root_path(),
                    sprite.name.clone(),
                    sprite.filesystem_path(),
                ) {
                    log::error!(
                        "And we couldn't even add to root! {:}. Aborting operation...",
                        e
                    );
                }
            }
        }
    }

    /// This gets the data on a given Sprite with a given name, if it exists.
    pub fn get_sprite(&self, sprite_name: &str) -> Option<&Sprite> {
        if self.resource_names.contains(sprite_name) == false {
            return None;
        }

        // Get the path
        let path = self.yyp.resources.iter().find_map(|yypr| {
            if yypr.id.name == sprite_name {
                Some(&yypr.id)
            } else {
                None
            }
        });

        path.and_then(|path| {
            self.sprites
                .resources
                .get(path)
                .map(|sprite_resource| &sprite_resource.yy_resource)
        })
    }

    /// Adds a subfolder to the folder given at `parent_path` at the final order. If a tree looks like:
    ///
    ///```norun
    /// Sprites/
    ///     - spr_player
    ///     - spr_enemy
    /// ```
    ///
    /// and user adds a folder with name `Items` to the `Sprites` folder, then the output tree will be:
    ///
    /// ```norun
    /// Sprites/
    ///     - spr_player
    ///     - spr_enemy
    ///     - Items/
    ///```
    ///
    /// `add_folder_to_end` returns a `Result<ViewPath>`, where `ViewPath` is of the newly created folder.
    /// This allows for easy sequential operations, such as adding a folder and then adding a file to that folder.
    pub fn add_folder_to_end(
        &mut self,
        parent_path: &ViewPath,
        name: String,
    ) -> Result<ViewPath, FolderGraphError> {
        let subfolder = self.folder_graph.find_subfolder_mut(parent_path)?;

        // Sometimes Gms2 uses 1 for the default order of folders. This is chaos.
        // No clue what's up with that.
        let order = subfolder.max_suborder().map(|v| v + 1).unwrap_or_default();

        if subfolder.folders.contains_key(&name) {
            return Err(FolderGraphError::FolderAlreadyPresent);
        }

        // Create our Path...
        let path = parent_path.path.join(&name);

        subfolder.folders.insert(
            name.clone(),
            SubfolderMember {
                child: FolderGraph::new(name.clone(), parent_path.clone()),
                order,
            },
        );

        self.yyp.folders.push(YypFolder {
            folder_path: path.clone(),
            order,
            name: name.clone(),
            ..YypFolder::default()
        });
        self.dirty = true;

        Ok(ViewPath { path, name })
    }

    /// Adds a subfolder to the folder given at `parent_path` at given order. If a tree looks like:
    ///
    ///```norun
    /// Sprites/
    ///     - spr_player
    ///     - OtherSprites/
    ///     - spr_enemy
    /// ```
    ///
    /// and user adds a folder with name `Items` to the `Sprites` folder with an order of 1,
    /// then the output tree will be:
    ///
    /// ```norun
    /// Sprites/
    ///     - spr_player
    ///     - Items/
    ///     - OtherSprites/
    ///     - spr_enemy
    ///```
    ///
    /// `add_folder_with_order` returns a `Result<ViewPath>`, where `ViewPath` is of the newly created folder.
    /// This allows for easy sequential operations, such as adding a folder and then adding a file to that folder.
    pub fn add_folder_with_order(
        &mut self,
        parent_path: ViewPath,
        name: String,
        order: usize,
    ) -> Result<ViewPath, FolderGraphError> {
        let subfolder = self.folder_graph.find_subfolder_mut(&parent_path)?;

        if subfolder.folders.contains_key(&name) {
            return Err(FolderGraphError::FolderAlreadyPresent);
        }

        // Add the Subfolder View:
        subfolder.folders.insert(
            name.clone(),
            SubfolderMember {
                child: FolderGraph::new(name.clone(), parent_path.clone()),
                order,
            },
        );

        let path = parent_path.path.join(&name);

        self.yyp.folders.push(YypFolder {
            folder_path: path.clone(),
            order,
            name: name.clone(),
            ..YypFolder::default()
        });
        self.dirty = true;

        // Fix the other Orders:
        for (folder_name, folder) in subfolder.folders.iter_mut() {
            if folder.order <= order {
                folder.order += 1;

                if let Err(e) = folder.update_yyp(&mut self.yyp.folders) {
                    error!(
                    "We couldn't find {0} in the Yyp, even though we had {0} in the FolderGraph.\
                    This may become a hard error in the future. E: {1}",
                    folder_name, e
                    )
                }
            }
        }

        for (file_name, file) in subfolder.files.iter_mut() {
            if file.order <= order {
                file.order += 1;

                if let Err(e) = file.update_yyp(&mut self.yyp.resources) {
                    error!(
                    "We couldn't find {0} in the Yyp, even though we had {0} in the FolderGraph.\
                    This may become a hard error in the future. E: {1}",
                    file_name, e
                    )
                }
            }
        }

        Ok(ViewPath { path: path, name })
    }

    /// Adds a file to the folder given at `parent_path` and with the final order. If a tree looks like:
    ///
    ///```norun
    /// Sprites/
    ///     - spr_player
    ///     - spr_enemy
    /// ```
    ///
    /// and user adds a file with name `spr_item` to the `Sprites` folder, then the output tree will be:
    ///
    /// ```norun
    /// Sprites/
    ///     - spr_player
    ///     - spr_enemy
    ///     - spr_item
    ///```
    pub fn add_file_at_end(
        &mut self,
        parent_path: ViewPath,
        name: String,
        child: FilesystemPath,
    ) -> Result<usize, FolderGraphError> {
        let subfolder = self.folder_graph.find_subfolder_mut(&parent_path)?;
        let order = subfolder.max_suborder().map(|v| v + 1).unwrap_or_default();
        if subfolder.files.contains_key(&name) {
            return Err(FolderGraphError::FileAlreadyPresent);
        }

        subfolder.files.insert(name, FileMember { child, order });
        Ok(order)
    }

    /// Adds a file to the folder given at `parent_path` at the given order. If a tree looks like:
    ///
    ///```norun
    /// Sprites/
    ///     - spr_player
    ///     - spr_enemy
    /// ```
    ///
    /// and user adds a file with name `spr_item` to the `Sprites` folder at order 1, then the output tree will be:
    ///
    /// ```norun
    /// Sprites/
    ///     - spr_player
    ///     - spr_item
    ///     - spr_enemy
    ///```
    ///
    /// Additionally, `spr_enemy`'s order will be updated to be `2`.
    pub fn add_file_with_order(
        &mut self,
        parent_path: ViewPath,
        name: String,
        child: FilesystemPath,
        order: usize,
    ) -> Result<(), FolderGraphError> {
        let subfolder = self.folder_graph.find_subfolder_mut(&parent_path)?;
        if subfolder.files.contains_key(&name) {
            return Err(FolderGraphError::FileAlreadyPresent);
        }

        subfolder.files.insert(name, FileMember { child, order });

        // Fix the Files
        for (file_name, file) in subfolder.files.iter_mut() {
            if file.order >= order {
                file.order += 1;
                if let Err(e) = file.update_yyp(&mut self.yyp.resources) {
                    error!(
                    "We couldn't find {0} in the Yyp, even though we had {0} in the FolderGraph.\
                    This may become a hard error in the future. E: {1}",
                    file_name, e
                    )
                }
            }
        }

        // Fix the Folders
        for (folder_name, folder) in subfolder.folders.iter_mut() {
            if folder.order >= order {
                folder.order += 1;

                if let Err(e) = folder.update_yyp(&mut self.yyp.folders) {
                    error!(
                    "We couldn't find {0} in the Yyp, even though we had {0} in the FolderGraph.\
                    This may become a hard error in the future. E: {1}",
                    folder_name, e
                    )
                }
            }
        }

        Ok(())
    }

    /// Adds a new Resource to be tracked by the YYP. The Resource also will
    /// need to serialize themselves and any additional files which they manage.
    ///
    /// This might include serializing sprites or sprite frames for Sprites, or `.gml`
    /// files for scripts or objects.
    #[allow(dead_code)]
    fn add_new_resource(&mut self, new_resource: &impl YyResource, order: usize) {
        self.resource_names.insert(new_resource.name().to_string());
        let new_yyp_resource = YypResource {
            id: new_resource.filesystem_path(),
            order,
        };

        // Update the Resource
        self.yyp.resources.push(new_yyp_resource);
        self.dirty = true;
    }

    pub fn serialize(&mut self) -> AnyResult<()> {
        if self.dirty {
            // Check if Sprite is Dirty and Serialize that:
            self.sprites
                .serialize(self.directory_manager.root_directory())?;

            // serialize the pipeline manifests
            self.pipeline_manager.serialize(&self.directory_manager)?;

            // Serialize Ourselves:
            let string = self.yyp.yyp_serialization(0);
            fs::write(&self.directory_manager.yyp(), &string)?;

            self.dirty = false;
        }

        Ok(())
    }
}

impl YypBoss {
    pub fn root_path(&self) -> ViewPath {
        ViewPath {
            name: "folders".to_string(),
            path: ViewPathLocation("folders".to_string()),
        }
    }

    /// Shows the underlying Yyp. This is exposed mostly
    /// for integration tests.
    pub fn yyp(&self) -> &Yyp {
        &self.yyp
    }

    /// Gives a reference to the current FolderGraph.
    pub fn root_folder(&self) -> &FolderGraph {
        &self.folder_graph
    }

    /// This could be a very hefty allocation!
    pub fn folder(&self, view_path: &ViewPath) -> Option<FolderGraph> {
        if view_path.name != self.folder_graph.name {
            let mut folder = &self.folder_graph;

            for path in view_path.path.component_paths() {
                folder = &folder
                    .folders
                    .get(path)
                    .ok_or_else(|| format_err!("Couldn't find subfolder {}", path))
                    .ok()?
                    .child;
            }
            Some(folder.clone())
        } else {
            Some(self.folder_graph.clone())
        }
    }
}

impl Into<Yyp> for YypBoss {
    fn into(self) -> Yyp {
        self.yyp
    }
}

impl PartialEq for YypBoss {
    fn eq(&self, other: &Self) -> bool {
        self.yyp == other.yyp
            && self.folder_graph == other.folder_graph
            && self.resource_names == other.resource_names
    }
}
