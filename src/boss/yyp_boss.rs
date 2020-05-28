use super::{folder_graph::*, FolderGraph, SpriteImageBuffer, YyResource};
use anyhow::{bail, format_err, Context, Result};
use std::collections::HashMap;
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
    pub fn new(path_to_yyp: &Path) -> Result<YypBoss> {
        let tcu = TrailingCommaUtility::new();
        let yyp_file = fs::read_to_string(&path_to_yyp)
            .with_context(|| format!("Path given: {:?}", path_to_yyp))?;
        let yyp: Yyp = serde_json::from_str(&tcu.clear_trailing_comma(&yyp_file))?;

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
                let mut entry = folder_graph.members.entry(section_name.clone());
                let new_member = entry.or_insert(FolderMember {
                    child: Child::SubFolder(FolderGraph::new(section_name, parent_path)),
                    order: new_folder.order,
                });

                folder_graph = match &mut new_member.child {
                    Child::SubFolder(folder) => folder,
                    _ => unimplemented!("We're only adding folders here"),
                };
            }
        }

        // Load in Sprites
        for sprite_resource in yyp_boss
            .yyp
            .resources
            .iter()
            .filter(|value| value.id.path.starts_with("sprites"))
        {
            let sprite_resource: &YypResource = sprite_resource;
            let sprite_path = yyp_boss
                .absolute_path
                .parent()
                .unwrap()
                .join(&sprite_resource.id.path);

            let sprite_yy: Sprite = yyp_boss
                .deserialize_yyfile(&sprite_path)
                .with_context(|| format!("Error importing sprite with Path {:#?}", sprite_path))?;

            let frame_buffers: Vec<_> = sprite_yy
                .frames
                .iter()
                .filter_map(|frame: &Frame| {
                    let path_to_image = sprite_path
                        .parent()
                        .unwrap()
                        .join(Path::new(&frame.name.inner().to_string()).with_extension("png"));

                    match image::open(&path_to_image) {
                        Ok(image) => Some((frame.name, image.to_rgba())),
                        Err(e) => {
                            log::error!("We couldn't read {:?} -- {}", path_to_image, e);
                            None
                        }
                    }
                })
                .collect();

            YypBoss::add_file_iteratable(
                &mut yyp_boss.folder_graph,
                sprite_yy.parent.clone(),
                sprite_yy.filesystem_path(),
            )?;
            yyp_boss.resource_names.insert(sprite_yy.name.clone());
            yyp_boss.sprites.add_new_startup(sprite_yy, frame_buffers);
        }

        Ok(yyp_boss)
    }

    pub fn absolute_path(&self) -> &Path {
        &self.absolute_path
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
        // Add to the YYP
        self.add_new_resource(&mut sprite);

        // Add to the Views...
        if let Err(e) = self.add_file(sprite.parent_path(), sprite.filesystem_path()) {
            log::error!(
                        "Couldn't add Sprite {}. It reported a parent path of {:#?}, and an FS path of {:#?}.\n\
                    Error was: {:}",
                        sprite.name,
                        sprite.parent_path(),
                        sprite.filesystem_path(),
                        e
                    );

            if let Err(e) = self.add_file(self.root_path(), sprite.filesystem_path()) {
                log::error!(
                    "And we couldn't even add to root! {:}. Aborting operation...",
                    e
                );
            }
        }

        // Add to our own Sprite Tracking
        self.sprites.add_new(sprite, associated_data);
    }

    pub fn add_folder(&mut self, folder_name: String, parent: ViewPath) -> Result<ViewPath> {
        todo!("Many fixes are needed here!
        
        First:
        1. We can have a file and a folder, in the same subfolder, with the same name. (we 
            cannot have a file and a file with the same name ,or a folder and a folder with the same name).
        2. We need a quick way to calculate order, and deal with asking users for Order on insertion.
        3. That's probably it actually. We'll need to make two hashmaps here and work with that I think.");

        let mut folder = &mut self.folder_graph;

        if folder.name != folder_name {
            for path in parent.path.iter().skip(1) {
                let string = path.to_string_lossy();
                let path_name: &str = string.split(".yy").next().unwrap();

                match &mut folder
                    .members
                    .get_mut(path_name)
                    .ok_or_else(|| format_err!("Couldn't find the subfolder {}", path_name))?
                    .child
                {
                    Child::SubFolder(sf) => {
                        folder = sf;
                    }
                    Child::File(_) => {
                        bail!("Path ended with a File, not a Folder.");
                    }
                }
            }
        }

        if folder.members.contains_key(&folder_name) {
            bail!("Path already had a folder by that name. Duplicate folders are not allowed.");
        }

        folder.members.insert(
            folder_name,
            FolderMember {
                child: Child::SubFolder(FolderGraph::new(folder_name.clone(), folder.view_path())),
                order: 0,
            },
        );

        let string = parent.path.to_string_lossy();
        let path_name: &str = string.split(".yy").next().unwrap();

        let path = Path::new(&format!("{}/{}.yy", path_name, folder_name)).to_owned();

        self.yyp.folders.insert(YypFolder {
            folder_path: path.clone(),
            order: 0,
            name: folder_name.clone(),
            ..YypFolder::default()
        });
        self.dirty = true;

        Ok(ViewPath {
            path,
            name: folder_name,
        })
    }

    /// Adds a file to the Virtual File System.
    pub fn add_file(
        &mut self,
        parent_path: ViewPath,
        filesystem_path: FilesystemPath,
    ) -> Result<()> {
        YypBoss::add_file_iteratable(&mut self.folder_graph, parent_path, filesystem_path)
    }

    /// Stupid ass borrow checker
    fn add_file_iteratable(
        mut folder: &mut FolderGraph,
        parent_path: ViewPath,
        filesystem_path: FilesystemPath,
    ) -> Result<()> {
        if parent_path.name != folder.name {
            for path in parent_path.path.iter().skip(1) {
                let path_name = path.to_string_lossy();
                let path_name = if let Some(pos) = path_name.find(".yy") {
                    std::borrow::Cow::Borrowed(&path_name[..pos])
                } else {
                    path_name
                };

                folder = folder
                    .subfolders
                    .iter_mut()
                    .find(|sf| sf.name == path_name)
                    .ok_or_else(|| format_err!("Couldn't find subfolder {}", path_name))?;
            }
            folder.files.push(filesystem_path);
        } else {
            folder.files.push(filesystem_path);
        }

        Ok(())
    }

    /// Adds a new Resource to be tracked by the YYP. The Resource also will
    /// need to serialize themselves and any additional files which they manage.
    ///
    /// This might include serializing sprites or sprite frames for Sprites, or `.gml`
    /// files for scripts or objects.
    #[allow(dead_code)]
    fn add_new_resource(&mut self, new_resource: &mut impl YyResource) {
        let mut iteration_count = 0;
        let mut name = new_resource.name().to_owned();
        while self.resource_names.contains(&name) {
            name = format!("{}_{}", name, iteration_count);
            iteration_count += 1;
        }
        if name != new_resource.name() {
            new_resource.set_name(name.clone());
        }

        self.resource_names.insert(name);
        let new_yyp_resource = YypResource {
            id: new_resource.filesystem_path(),
            order: 0,
        };

        // Update the Resource
        self.yyp.resources.insert(new_yyp_resource);
        self.dirty = true;
    }

    pub fn serialize(&mut self) -> Result<()> {
        if self.dirty {
            // self.yyp
            //     .resources
            //     .sort_by(|lr, rr| lr.key.inner().cmp(&rr.key.inner()));

            // Check if Sprite is Dirty and Serialize that:
            self.sprites
                .serialize(&self.absolute_path.parent().unwrap())?;
            // Serialize Ourselves:
            serialize(&self.absolute_path, &self.yyp)?;

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

                folder = folder
                    .subfolders
                    .iter()
                    .find(|sf| sf.name == path_name)
                    .ok_or_else(|| format_err!("Couldn't find subfolder {}", path_name))
                    .ok()?;
            }
            Some(folder.clone())
        } else {
            Some(self.folder_graph.clone())
        }
    }
}

/// Utilities
impl YypBoss {
    fn deserialize_yyfile<T>(&self, path: &Path) -> Result<T>
    where
        for<'de> T: serde::Deserialize<'de>,
    {
        let file_string = fs::read_to_string(path)?;
        let data = serde_json::from_str(&self.tcu.clear_trailing_comma(&file_string))?;
        Ok(data)
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

#[derive(Default)]
pub struct YyResourceData<T: YyResource> {
    pub yy_resource: T,
    pub associated_data: T::AssociatedData,
}

impl<T: YyResource + std::fmt::Debug> std::fmt::Debug for YyResourceData<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} !!**ASSOCIATED DATA IS NOT PRINTED IN DEBUG OUTPUT**!!",
            self.yy_resource
        )
    }
}

#[derive(Debug, Default)]
pub struct YyResourceHandler<T: YyResource> {
    dirty: bool,
    resources: HashMap<FilesystemPath, YyResourceData<T>>,
    dirty_resources: Vec<FilesystemPath>,
}

impl<T: YyResource> YyResourceHandler<T> {
    pub fn new() -> Self {
        Self {
            dirty: false,
            resources: HashMap::new(),
            dirty_resources: Vec::new(),
        }
    }

    pub fn add_new(&mut self, value: T, associated_data: T::AssociatedData) {
        self.dirty_resources.push(value.filesystem_path());
        self.dirty = true;
        self.add_new_startup(value, associated_data);
    }

    /// This is the same as `add_new` but it doesn't dirty the resource. It is used
    /// for startup operations, where we're loading in assets from disk.
    fn add_new_startup(&mut self, value: T, associated_data: T::AssociatedData) {
        self.resources.insert(
            value.filesystem_path(),
            YyResourceData {
                yy_resource: value,
                associated_data,
            },
        );
    }

    pub fn serialize(&mut self, project_path: &Path) -> Result<()> {
        if self.dirty {
            while let Some(dirty_resource) = self.dirty_resources.pop() {
                let resource = self
                    .resources
                    .get(&dirty_resource)
                    .expect("This should always be valid.");

                let yy_path = project_path.join(&resource.yy_resource.filesystem_path().path);

                if let Some(parent_dir) = yy_path.parent() {
                    fs::create_dir_all(parent_dir)?;
                    T::serialize_associated_data(
                        &resource.yy_resource,
                        parent_dir,
                        &resource.associated_data,
                    )?;
                }
                serialize(&yy_path, &resource.yy_resource)?;
            }
        }

        Ok(())
    }
}

fn serialize(absolute_path: &Path, data: &impl serde::Serialize) -> Result<()> {
    let data = serde_json::to_string_pretty(data)?;
    fs::write(absolute_path, data)?;
    Ok(())
}

// fn deserialize_json(path: &Path) -> Result<serde_json::Value> {
//     let file_string = fs::read_to_string(path)?;
//     let data = serde_json::from_str(&file_string)?;
//     Ok(data)
// }
