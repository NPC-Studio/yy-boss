use super::boss::yy_resource::YyResource;
use image::{ImageBuffer, Rgba};
use std::{num::NonZeroUsize, path::Path};
use yy_typings::sprite::*;

pub type SpriteImageBuffer = ImageBuffer<Rgba<u8>, Vec<u8>>;

pub trait SpriteExt {
    fn with(self, edit: impl Fn(&mut Self)) -> Self;
    fn new(name: &str, texture_group_id: &str) -> Sprite;
    fn with_layer(name: &str, texture_group_id: &str, layer: Layer) -> Sprite;
    fn parent(self, parent: ViewPath) -> Sprite;
    fn bbox_mode(self, f: impl Fn(isize, isize) -> BboxModeUtility) -> Self;
    fn collision_kind(self, collision_kind: CollisionKind) -> Self;
    fn frame(self, frame_id: FrameId) -> Self;
    fn origin(self, origin: OriginUtility, locked: bool) -> Self;
    fn playback_speed(self, pback_speed: PlaybackSpeed, speed: f64) -> Self;
    fn dimensions(self, width: NonZeroUsize, height: NonZeroUsize) -> Self;
}

impl SpriteExt for Sprite {
    fn with(mut self, edit: impl Fn(&mut Self)) -> Self {
        edit(&mut self);
        self
    }

    fn with_layer(name: &str, texture_group_id: &str, layer: Layer) -> Sprite {
        Sprite {
            name: name.to_string(),
            texture_group_id: TextureGroupPath {
                path: Path::new(&format!("texturegroups/{}", texture_group_id)).to_owned(),
                name: texture_group_id.to_string(),
            },
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
            layers: vec![layer],
            ..Sprite::default()
        }
    }

    fn new(name: &str, texture_group_id: &str) -> Sprite {
        Sprite::with_layer(
            name,
            texture_group_id,
            Layer {
                visible: true,
                is_locked: false,
                blend_mode: 0,
                opacity: 100.0,
                display_name: "default".to_string(),
                resource_version: "1.0".to_owned(),
                name: LayerId::new(),
                tags: vec![],
                resource_type: ConstGmImageLayer::Const,
            },
        )
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
    fn collision_kind(self, collision_kind: CollisionKind) -> Self {
        self.with(|me| {
            me.collision_kind = collision_kind;
            if me.collision_kind != CollisionKind::Precise {
                me.separate_masks = false;
            }
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

use anyhow::Context;
impl YyResource for Sprite {
    type AssociatedData = Vec<(FrameId, SpriteImageBuffer)>;

    fn name(&self) -> &str {
        &self.name
    }
    fn set_name(&mut self, name: String) {
        self.name = name.clone();
        let new_path = format!("sprites/{0}/{0}.yy", name);
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
            name: name.clone(),
            path: new_path.to_owned(),
        };
        let track: &mut Track = &mut self.sequence.tracks[0];
        for kf in track.keyframes.keyframes.iter_mut() {
            kf.channels.zero.id.path = new_path.to_owned();
        }
    }
    fn filesystem_path(&self) -> FilesystemPath {
        FilesystemPath {
            name: self.name.clone(),
            path: Path::new(&format!("sprites/{0}/{0}.yy", self.name)).to_owned(),
        }
    }
    fn parent_path(&self) -> ViewPath {
        self.parent.clone()
    }
    fn serialize_associated_data(
        &self,
        directory_path: &Path,
        data: &Self::AssociatedData,
    ) -> anyhow::Result<()> {
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
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Bbox {
    pub top_left: (isize, isize),
    pub bottom_right: (isize, isize),
}

#[derive(Debug, Copy, Clone, strum_macros::EnumIter, strum_macros::Display)]
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

#[derive(Debug, Copy, Clone, strum_macros::EnumIter, strum_macros::Display)]
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
