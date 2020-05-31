use super::{folder_graph::*, FolderGraph, SpriteImageBuffer, YyResource, YyResourceHandler};
use crate::YypSerialization;
use anyhow::{format_err, Context, Result as AnyResult};
use log::info;
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};
use yy_typings::{sprite::*, utils::TrailingCommaUtility, Yyp};

#[derive(Debug)]
pub struct YypBoss {
    yyp: Yyp,
    absolute_path: PathBuf,
    sprites: YyResourceHandler<Sprite>,
    folder_graph: FolderGraph,
    resource_names: HashSet<String>,
    tcu: TrailingCommaUtility,
    dirty: bool,
}

impl YypBoss {
    /// Creates a new YyBoss Manager and performs startup file reading.
    pub fn new(path_to_yyp: &Path) -> AnyResult<YypBoss> {
        let tcu = TrailingCommaUtility::new();
        let yyp_file = fs::read_to_string(&path_to_yyp)
            .with_context(|| format!("Path given: {:?}", path_to_yyp))?;
        let yyp: Yyp = serde_json::from_str(&tcu.clear_trailing_comma(&yyp_file))
            .with_context(|| "on the Yyp itself")?;

        let mut yyp_boss = Self {
            yyp,
            absolute_path: path_to_yyp.to_owned(),
            dirty: false,
            sprites: YyResourceHandler::new(),
            folder_graph: FolderGraph::root(),
            resource_names: HashSet::new(),
            tcu,
        };

        // Load in Folders
        for new_folder in yyp_boss.yyp.folders.iter() {
            let mut folder_graph = &mut yyp_boss.folder_graph;

            for section in new_folder.folder_path.iter().skip(1) {
                let section_name = section.to_string_lossy();
                let section_name = if section_name.ends_with(".yy") {
                    new_folder.name.clone()
                } else {
                    section_name.to_string()
                };

                let parent_path = folder_graph.view_path();
                let entry = folder_graph.folders.entry(section_name.clone());
                let new_member = entry.or_insert(SubfolderMember {
                    child: FolderGraph::new(section_name, parent_path),
                    order: new_folder.order,
                });

                folder_graph = &mut new_member.child;
            }
        }
        info!("Loading in Sprites...");

        // Load in Sprites
        yyp_boss.sprites.shared_data = Sprite::load_shared_data(&yyp_boss.absolute_path)
            .with_context(|| "loading the sprite shared data")?;
        for sprite_resource in yyp_boss
            .yyp
            .resources
            .iter()
            .filter(|value| value.id.path.starts_with("sprites"))
        {
            let sprite_path = yyp_boss
                .absolute_path
                .parent()
                .unwrap()
                .join(&sprite_resource.id.path);

            let sprite_yy: Sprite =
                yyp_boss.deserialize_yyfile(&sprite_path).with_context(|| {
                    format!("Error deserializing sprite with Path {:#?}", sprite_path)
                })?;

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

        Ok(yyp_boss)
    }

    pub fn absolute_path(&self) -> &Path {
        &self.absolute_path
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

    /// For the shared data in a sprite. a little messy!
    pub fn mark_sprite_with_path(&mut self, sprite_id: FilesystemPath, ase_name: String) {
        let shared_data = self.sprites.shared_data.get_or_insert(Default::default());
        shared_data.insert(ase_name, sprite_id);
        self.sprites.dirty = true;
        self.dirty = true;
    }

    /// This gets the data on a given Sprite with a given name. If no Sprite by that name exists,
    /// then a None is returned. It does not return a handle on the Associated Data of the Sprite.
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

    pub fn get_sprite_data(&self) -> Option<<Sprite as YyResource>::SharedData> {
        self.sprites.shared_data.clone()
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
        parent_path: ViewPath,
        name: String,
    ) -> Result<ViewPath, FolderGraphError> {
        let subfolder = self.folder_graph.find_subfolder_mut(&parent_path)?;
        let order = subfolder.max_suborder().map(|v| v + 1).unwrap_or_default();

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

        let path_name = parent_path.path.to_string_lossy();
        let path_name = path_name.trim_end_matches(".yy");

        let path = Path::new(&format!("{}/{}.yy", path_name, name)).to_owned();

        self.yyp.folders.push(YypFolder {
            folder_path: path.clone(),
            order,
            name: name.clone(),
            ..YypFolder::default()
        });
        self.dirty = true;

        Ok(ViewPath { path, name })
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
        for file in subfolder.files.values_mut() {
            if file.order >= order {
                file.order += 1;
            }
        }

        // Fix the Folders
        for folder in subfolder.folders.values_mut() {
            if folder.order >= order {
                folder.order += 1;
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
                .serialize(&self.absolute_path.parent().unwrap())?;
            // Serialize Ourselves:
            let string = self.yyp.yyp_serialization(0);
            fs::write(&self.absolute_path, &string)?;

            self.dirty = false;
        }

        Ok(())
    }
}

impl YypBoss {
    pub fn root_path(&self) -> ViewPath {
        ViewPath {
            name: "folders".to_string(),
            path: Path::new("folders").to_owned(),
        }
    }

    /// Shows the underlying Yyp. This is exposed mostly
    /// for integration tests.
    pub fn yyp(&self) -> &Yyp {
        &self.yyp
    }

    /// This could be a very hefty allocation!
    pub fn root_folder(&self) -> FolderGraph {
        self.folder_graph.clone()
    }

    /// This could be a very hefty allocation!
    pub fn folder(&self, view_path: &ViewPath) -> Option<FolderGraph> {
        if view_path.name != self.folder_graph.name {
            let mut folder = &self.folder_graph;

            for path in view_path.path.iter().skip(1) {
                let path_name = path.to_string_lossy();
                let path_name = if let Some(pos) = path_name.find(".yy") {
                    std::borrow::Cow::Borrowed(&path_name[..pos])
                } else {
                    path_name
                };

                folder = &folder
                    .folders
                    .get(path_name.as_ref())
                    .ok_or_else(|| format_err!("Couldn't find subfolder {}", path_name))
                    .ok()?
                    .child;
            }
            Some(folder.clone())
        } else {
            Some(self.folder_graph.clone())
        }
    }
}

/// Utilities
impl YypBoss {
    fn deserialize_yyfile<T>(&self, path: &Path) -> AnyResult<T>
    where
        for<'de> T: serde::Deserialize<'de>,
    {
        let file_string = fs::read_to_string(path)?;
        let data = serde_json::from_str(&self.tcu.clear_trailing_comma(&file_string))?;
        Ok(data)
    }

    pub fn ensure_yyboss_data(path: &Path) -> AnyResult<()> {
        let subdir = path.join("/.yyboss");
        if subdir.exists() == false {
            std::fs::create_dir(subdir)?;
        }

        Ok(())
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
    }
}
