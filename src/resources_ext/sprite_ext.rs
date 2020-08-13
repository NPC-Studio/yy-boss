use crate::{
    Resource, SerializedData, SerializedDataError, YyResource, YyResourceHandler, YypBoss,
};
use anyhow::Context;
use anyhow::Result as AnyResult;
use image::{ImageBuffer, Rgba};
use std::{
    collections::HashMap,
    num::NonZeroUsize,
    path::{Path, PathBuf},
};
use yy_typings::{sprite_yy::*, TexturePath};

pub type SpriteImageBuffer = ImageBuffer<Rgba<u8>, Vec<u8>>;

pub trait SpriteExt {
    fn with(self, edit: impl Fn(&mut Self)) -> Self;
    fn new(name: &str, texture_group_id: TexturePath, parent: ViewPath) -> Sprite;
    fn with_layer(
        name: &str,
        texture_group_id: TexturePath,
        layer: Layer,
        parent: ViewPath,
    ) -> Sprite;
    fn parent(self, parent: ViewPath) -> Sprite;
    fn bbox_mode(self, f: impl Fn(isize, isize) -> BboxModeUtility) -> Self;
    fn collision_kind(self, collision_kind: CollisionKind) -> Self;
    fn frame(self, frame_id: FrameId) -> Self;
    /// Clears all of the frames from the given image. Generally speaking,
    /// a sprite should have at least one frame when imported into GMS2, but this
    /// function will leave it entirely bare.
    ///
    /// Builder version.
    fn clear_all_frames(self) -> Self;
    fn origin(self, origin: OriginUtility, locked: bool) -> Self;
    fn playback_speed(self, pback_speed: PlaybackSpeed, speed: f64) -> Self;
    fn dimensions(self, width: NonZeroUsize, height: NonZeroUsize) -> Self;

    /// Clears all of the frames from the given image. Generally speaking,
    /// a sprite should have at least one frame when imported into GMS2, but this
    /// function will leave it entirely bare.
    fn set_clear_all_frames(&mut self);
}

impl SpriteExt for Sprite {
    fn with(mut self, edit: impl Fn(&mut Self)) -> Self {
        edit(&mut self);
        self
    }

    fn new(name: &str, texture_group_id: TexturePath, parent: ViewPath) -> Sprite {
        Sprite::with_layer(
            name,
            texture_group_id,
            Layer {
                visible: true,
                is_locked: false,
                blend_mode: 0,
                opacity: 100.0,
                display_name: "default".to_string(),
                resource_version: ResourceVersion::default(),
                name: LayerId::new(),
                tags: vec![],
                resource_type: ConstGmImageLayer::Const,
            },
            parent,
        )
    }

    fn with_layer(
        name: &str,
        texture_group_id: TexturePath,
        layer: Layer,
        parent: ViewPath,
    ) -> Sprite {
        Sprite {
            name: name.to_string(),
            texture_group_id,
            sequence: SpriteSequence {
                sprite_id: FilesystemPath {
                    name: name.to_string(),
                    path: Path::new(&format!("sprites/{spr}/{spr}.yy", spr = name)).to_owned(),
                },
                playback_speed: 15.0,
                playback_speed_type: PlaybackSpeed::FramesPerSecond,
                length: 1.0,
                tracks: vec![Track::default()],
                visible_range: None,
                backdrop_width: 1920,
                backdrop_height: 1080,
                xorigin: 0,
                yorigin: 0,
                parent: FilesystemPath {
                    name: name.to_string(),
                    path: Path::new(&format!("sprites/{spr}/{spr}.yy", spr = name)).to_owned(),
                },
                ..SpriteSequence::default()
            },
            parent,
            layers: vec![layer],
            ..Sprite::default()
        }
    }

    fn parent(self, parent: ViewPath) -> Sprite {
        self.with(|me| me.parent = parent.clone())
    }

    fn bbox_mode(mut self, f: impl Fn(isize, isize) -> BboxModeUtility) -> Self {
        let bbox_util = f(self.width.get() as isize, self.height.get() as isize);
        self.bbox_mode = bbox_util.into();

        let bbox = match bbox_util {
            BboxModeUtility::Automatic(bbox) | BboxModeUtility::Manual(bbox) => bbox,
            BboxModeUtility::FullImage => {
                let width = self.width.get() as isize;
                let height = self.height.get() as isize;

                Bbox {
                    top_left: (0, 0),
                    bottom_right: (width, height),
                }
            }
        };

        self.bbox_left = bbox.top_left.0;
        self.bbox_top = bbox.top_left.1;
        self.bbox_right = bbox.bottom_right.0;
        self.bbox_bottom = bbox.bottom_right.1;
        self
    }

    fn frame(self, frame_name: FrameId) -> Self {
        self.with(|me| {
            let path_to_sprite = format!("sprites/{0}/{0}.yy", me.name);
            let path_to_sprite = Path::new(&path_to_sprite);
            // Update the Frame
            me.frames.push(Frame {
                composite_image: Image {
                    frame_id: FilesystemPath {
                        name: frame_name.inner().to_string(),
                        path: path_to_sprite.to_owned(),
                    },
                    layer_id: None,
                    name: Some("composite".to_string()),
                    ..Image::default()
                },
                images: me
                    .layers
                    .iter()
                    .map(|layer| Image {
                        frame_id: FilesystemPath {
                            name: frame_name.inner().to_string(),
                            path: path_to_sprite.to_owned(),
                        },
                        layer_id: Some(FilesystemPath {
                            name: layer.name.inner().to_string(),
                            path: path_to_sprite.to_owned(),
                        }),
                        name: None,
                        ..Image::default()
                    })
                    .collect(),
                parent: FilesystemPath {
                    name: me.name.clone(),
                    path: path_to_sprite.to_owned(),
                },
                name: frame_name,
                ..Frame::default()
            });

            // Update the Sequence
            let track: &mut Track = &mut me.sequence.tracks[0];
            track.keyframes.keyframes.push(SpriteKeyframe {
                id: SpriteSequenceId::new(),
                key: me.frames.len() as f64 - 1.0,
                channels: Channels {
                    zero: SpriteZeroChannel {
                        id: FilesystemPath {
                            name: frame_name.inner().to_string(),
                            path: path_to_sprite.to_owned(),
                        },
                        ..Default::default()
                    },
                },
                ..SpriteKeyframe::default()
            });
            me.sequence.length = me.frames.len() as f64;
        })
    }

    /// Test
    fn clear_all_frames(self) -> Self {
        self.with(Self::set_clear_all_frames)
    }

    /// Another test
    fn set_clear_all_frames(&mut self) {
        self.frames.clear();

        self.sequence.length = 0.0;
        let track: &mut Track = &mut self.sequence.tracks[0];
        track.keyframes.keyframes.clear();
    }

    fn collision_kind(self, collision_kind: CollisionKind) -> Self {
        self.with(|me| {
            me.collision_kind = collision_kind;
        })
    }
    fn origin(self, origin: OriginUtility, locked: bool) -> Self {
        self.with(|me| {
            let w = me.width.get();
            let h = me.height.get();

            match origin {
                OriginUtility::Custom { x, y } => {
                    me.origin = Origin::Custom;
                    me.sequence.xorigin = x;
                    me.sequence.yorigin = y;
                }
                OriginUtility::TopLeft => {
                    me.origin = Origin::TopLeft;
                    me.sequence.xorigin = 0;
                    me.sequence.yorigin = 0;
                }
                OriginUtility::TopCenter => {
                    me.origin = Origin::TopCenter;
                    me.sequence.xorigin = (w / 2) as isize;
                    me.sequence.yorigin = 0;
                }
                OriginUtility::TopRight => {
                    me.origin = Origin::TopRight;
                    me.sequence.xorigin = (w - 1) as isize;
                    me.sequence.yorigin = 0;
                }
                OriginUtility::MiddleLeft => {
                    me.origin = Origin::MiddleLeft;
                    me.sequence.xorigin = 0;
                    me.sequence.yorigin = (h / 2) as isize;
                }
                OriginUtility::MiddleCenter => {
                    me.origin = Origin::MiddleCenter;
                    me.sequence.xorigin = (w / 2) as isize;
                    me.sequence.yorigin = (h / 2) as isize;
                }
                OriginUtility::MiddleRight => {
                    me.origin = Origin::MiddleRight;
                    me.sequence.xorigin = (w - 1) as isize;
                    me.sequence.yorigin = (h / 2) as isize;
                }
                OriginUtility::BottomLeft => {
                    me.origin = Origin::BottomLeft;
                    me.sequence.xorigin = 0;
                    me.sequence.yorigin = (h - 1) as isize;
                }
                OriginUtility::BottomCenter => {
                    me.origin = Origin::BottomCenter;
                    me.sequence.xorigin = (w / 2) as isize;
                    me.sequence.yorigin = (h - 1) as isize;
                }
                OriginUtility::BottomRight => {
                    me.origin = Origin::BottomRight;
                    me.sequence.xorigin = (w - 1) as isize;
                    me.sequence.yorigin = (h - 1) as isize;
                }
            }
            me.sequence.lock_origin = locked;
        })
    }
    fn playback_speed(self, speed_type: PlaybackSpeed, speed: f64) -> Self {
        self.with(|me| {
            me.sequence.playback_speed_type = speed_type;
            me.sequence.playback_speed = speed;
        })
    }
    fn dimensions(self, width: NonZeroUsize, height: NonZeroUsize) -> Self {
        self.with(|me| {
            me.width = width;
            me.height = height;
        })
    }
}

impl YyResource for Sprite {
    type AssociatedData = HashMap<FrameId, SpriteImageBuffer>;
    const SUBPATH_NAME: &'static str = "sprites";
    const RESOURCE: Resource = Resource::Sprite;

    fn name(&self) -> &str {
        &self.name
    }

    fn set_name(&mut self, name: String) {
        self.name = name.clone();
        let new_path = format!(
            "{base}/{name}/{name}.yy",
            base = Self::SUBPATH_NAME,
            name = name
        );
        let new_path = Path::new(&new_path);
        for frame in &mut self.frames {
            frame.parent = FilesystemPath {
                name: name.clone(),
                path: new_path.to_owned(),
            };

            frame.composite_image.frame_id.path = new_path.to_owned();
            for image in frame.images.iter_mut() {
                image.frame_id.path = new_path.to_owned();
                image.layer_id.as_mut().unwrap().path = new_path.to_owned();
            }
        }
        self.sequence.sprite_id = FilesystemPath {
            name,
            path: new_path.to_owned(),
        };
        let track: &mut Track = &mut self.sequence.tracks[0];
        for kf in track.keyframes.keyframes.iter_mut() {
            kf.channels.zero.id.path = new_path.to_owned();
        }
    }

    fn parent_path(&self) -> ViewPath {
        self.parent.clone()
    }

    fn get_handler(yyp_boss: &YypBoss) -> &YyResourceHandler<Self> {
        &yyp_boss.sprites
    }

    fn get_handler_mut(yyp_boss: &mut YypBoss) -> &mut YyResourceHandler<Self> {
        &mut yyp_boss.sprites
    }

    fn deserialize_associated_data(
        &self,
        directory_path: Option<&Path>,
        data: SerializedData,
    ) -> Result<Self::AssociatedData, SerializedDataError> {
        match data {
            SerializedData::Value { .. } => Err(SerializedDataError::CannotUseValue),
            SerializedData::Filepath { data } => {
                if let Some(directory) = directory_path {
                    let directory_path = directory.join(data);
                    let output = self
                        .frames
                        .iter()
                        .filter_map(|frame: &Frame| {
                            let path_to_image =
                                directory_path.join(&format!("{}.png", frame.name.inner()));

                            match image::open(&path_to_image) {
                                Ok(image) => Some((frame.name, image.to_rgba())),
                                Err(e) => {
                                    log::error!("We couldn't read {:?} -- {}", path_to_image, e);
                                    None
                                }
                            }
                        })
                        .collect();

                    Ok(output)
                } else {
                    Err(SerializedDataError::NoFileMode)
                }
            }
            SerializedData::DefaultValue => {
                let output = self
                    .frames
                    .iter()
                    .map(|frame: &Frame| {
                        (
                            frame.name,
                            SpriteImageBuffer::from_raw(
                                self.width.get() as u32,
                                self.height.get() as u32,
                                vec![0; self.width.get() * self.height.get() * 4],
                            )
                            .expect("Jack messed up the math in the Frame Buffer defaults"),
                        )
                    })
                    .collect();

                Ok(output)
            }
        }
    }

    fn serialize_associated_data(
        &self,
        directory_path: &Path,
        data: &Self::AssociatedData,
    ) -> AnyResult<()> {
        let layers_path = directory_path.join("layers");
        if layers_path.exists() == false {
            std::fs::create_dir(&layers_path)?;
        }

        for (frame_id, image) in data {
            let inner_id_string = frame_id.inner().to_string();
            let image: &ImageBuffer<_, _> = image;

            // Make the Core Image:
            let path = directory_path.join(&inner_id_string).with_extension("png");
            image.save(&path).with_context(|| {
                format!("We couldn't serialize the Core Image at path {:?}", path)
            })?;

            // Make the folder and layer image:
            let folder_path = layers_path.join(&inner_id_string);
            if folder_path.exists() == false {
                std::fs::create_dir(&folder_path)?;
            }

            let image_layer_id = self
                .layers
                .first()
                .ok_or_else(|| anyhow::anyhow!("All Sprites *must* have a single Layer!"))?
                .name
                .inner()
                .to_string();

            let final_layer_path = folder_path.join(&image_layer_id).with_extension("png");
            image
                .save(&final_layer_path)
                .with_context(|| format!("We couldn't save an Image to {:?}", final_layer_path))?;
        }

        Ok(())
    }

    fn serialize_associated_data_into_data(
        &self,
        our_directory: &Path,
        working_directory: Option<&Path>,
        associated_data: Option<&Self::AssociatedData>,
    ) -> Result<SerializedData, SerializedDataError> {
        let working_directory = working_directory
            .ok_or_else(|| SerializedDataError::NoFileMode)?
            .join(self.name());

        fn perform_serialization(
            data: &HashMap<FrameId, SpriteImageBuffer>,
            working_directory: PathBuf,
        ) -> Result<SerializedData, SerializedDataError> {
            for (frame_id, img) in data {
                let path = working_directory.join(format!("{}.png", frame_id.inner()));

                img.save(&path)
                    .map_err(SerializedDataError::CouldNotWriteImage)?;
            }

            Ok(SerializedData::Filepath {
                data: working_directory,
            })
        }

        if let Some(data) = associated_data {
            perform_serialization(data, working_directory)
        } else {
            let value: HashMap<FrameId, SpriteImageBuffer> = self
                .deserialize_associated_data(
                    Some(our_directory),
                    SerializedData::Filepath {
                        data: PathBuf::new(),
                    },
                )
                .map_err(|e| SerializedDataError::InnerError(e.to_string()))?;

            perform_serialization(&value, working_directory)
        }
    }

    fn cleanup_on_replace(
        &self,
        files_to_delete: &mut Vec<PathBuf>,
        folders_to_delete: &mut Vec<PathBuf>,
    ) {
        // first, clean up the layer folders...
        let base_path = Path::new(&self.name);
        let layers_path = base_path.join("layers");

        // clean up the composite image...
        for frame in self.frames.iter() {
            let name = frame.name.inner().to_string();
            let path = Path::new(&name);
            folders_to_delete.push(layers_path.join(path));

            let mut file = path.to_owned();
            file.set_extension("png");
            files_to_delete.push(base_path.join(file));
        }
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct Bbox {
    pub top_left: (isize, isize),
    pub bottom_right: (isize, isize),
}

#[derive(Debug, Copy, Clone, strum_macros::EnumIter, strum_macros::Display, PartialEq, Eq)]
pub enum OriginUtility {
    TopLeft,
    TopCenter,
    TopRight,
    MiddleLeft,
    MiddleCenter,
    MiddleRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
    Custom { x: isize, y: isize },
}

impl OriginUtility {
    pub fn from_origin(o: Origin, origin_pos: (isize, isize)) -> OriginUtility {
        match o {
            Origin::TopLeft => OriginUtility::TopLeft,
            Origin::TopCenter => OriginUtility::TopCenter,
            Origin::TopRight => OriginUtility::TopRight,
            Origin::MiddleLeft => OriginUtility::MiddleLeft,
            Origin::MiddleCenter => OriginUtility::MiddleCenter,
            Origin::MiddleRight => OriginUtility::MiddleRight,
            Origin::BottomLeft => OriginUtility::BottomLeft,
            Origin::BottomCenter => OriginUtility::BottomCenter,
            Origin::BottomRight => OriginUtility::BottomRight,
            Origin::Custom => OriginUtility::Custom {
                x: origin_pos.0,
                y: origin_pos.1,
            },
        }
    }
}

#[derive(Debug, Copy, Clone, strum_macros::EnumIter, strum_macros::Display, PartialEq, Eq)]
pub enum BboxModeUtility {
    Automatic(Bbox),
    FullImage,
    Manual(Bbox),
}

impl From<BboxModeUtility> for BBoxMode {
    fn from(o: BboxModeUtility) -> Self {
        match o {
            BboxModeUtility::Automatic(_) => BBoxMode::Automatic,
            BboxModeUtility::FullImage => BBoxMode::FullImage,
            BboxModeUtility::Manual(_) => BBoxMode::Manual,
        }
    }
}

impl BboxModeUtility {
    pub fn from_bbox_data(
        bbox_mode: BBoxMode,
        left: isize,
        top: isize,
        right: isize,
        bottom: isize,
    ) -> BboxModeUtility {
        match bbox_mode {
            BBoxMode::Automatic => BboxModeUtility::Automatic(Bbox {
                top_left: (top, left),
                bottom_right: (bottom, right),
            }),
            BBoxMode::FullImage => BboxModeUtility::FullImage,
            BBoxMode::Manual => BboxModeUtility::Manual(Bbox {
                top_left: (top, left),
                bottom_right: (bottom, right),
            }),
        }
    }
}
