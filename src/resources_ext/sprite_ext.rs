use crate::{
    FileHolder, Resource, SerializedData, SerializedDataError, YyResource, YyResourceHandler,
    YypBoss,
};
use anyhow::Context;
use anyhow::Result as AnyResult;
use image::{ImageBuffer, Rgba};
use std::{collections::HashMap, num::NonZeroUsize, path::Path};
use yy_typings::BBoxMode;
use yy_typings::Channels;
use yy_typings::CollisionKind;
use yy_typings::CommonData;
use yy_typings::FilesystemPath;
use yy_typings::FrameId;
use yy_typings::LayerId;
use yy_typings::Origin;
use yy_typings::PlaybackSpeed;
use yy_typings::Sprite;
use yy_typings::SpriteKeyframe;
use yy_typings::SpriteLayer;
use yy_typings::SpriteSequence;
use yy_typings::SpriteSequenceId;
use yy_typings::SpriteZeroChannel;
use yy_typings::TexturePath;
use yy_typings::Track;
use yy_typings::TrailingCommaUtility;
use yy_typings::ViewPath;

pub type SpriteImageBuffer = ImageBuffer<Rgba<u8>, Vec<u8>>;

pub trait SpriteExt: Sized {
    fn with(self, edit: impl Fn(&mut Self)) -> Self;
    fn new(name: &str, texture_group_id: TexturePath, parent: ViewPath) -> Self;
    fn with_layer(
        name: &str,
        texture_group_id: TexturePath,
        layer: SpriteLayer,
        parent: ViewPath,
    ) -> Sprite;
    fn parent(self, parent: ViewPath) -> Sprite;
    fn bbox_mode(self, f: impl Fn(i32, i32) -> BboxModeUtility) -> Self;
    fn collision_kind(self, collision_kind: CollisionKind) -> Self;
    /// Clears all of the frames from the given image. Generally speaking,
    /// a sprite should have at least one frame when imported into GMS2, but this
    /// function will leave it entirely bare.
    ///
    /// Builder version.
    fn clear_all_frames(self) -> Self;
    fn origin(self, origin: OriginUtility, locked: bool) -> Self;
    fn playback_speed(self, pback_speed: PlaybackSpeed, speed: f32) -> Self;
    fn dimensions(self, width: NonZeroUsize, height: NonZeroUsize) -> Self;

    /// Clears all of the frames from the given image. Generally speaking,
    /// a sprite should have at least one frame when imported into GMS2, but this
    /// function will leave it entirely bare.
    fn set_clear_all_frames(&mut self);
    fn set_frame(&mut self, frame_id: FrameId, sprite_sequence_id: SpriteSequenceId);
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
            SpriteLayer {
                visible: true,
                is_locked: false,
                blend_mode: 0,
                opacity: 100.0,
                display_name: "default".to_string(),

                common_data: CommonData::new(LayerId::new()),
            },
            parent,
        )
    }

    fn with_layer(
        name: &str,
        texture_group_id: TexturePath,
        layer: SpriteLayer,
        parent: ViewPath,
    ) -> Sprite {
        Sprite {
            common_data: CommonData::new(name.to_owned()),
            texture_group_id,
            sequence: SpriteSequence {
                playback_speed: 15.0,
                playback_speed_type: PlaybackSpeed::FramesPerSecond,
                length: 1.0,
                tracks: vec![Track::default()],
                visible_range: None,
                backdrop_width: 1920,
                backdrop_height: 1080,
                xorigin: 0,
                yorigin: 0,
                ..SpriteSequence::default()
            },
            layers: vec![layer],
            parent,
            ..Sprite::default()
        }
    }

    fn parent(self, parent: ViewPath) -> Sprite {
        self.with(|me| me.parent = parent.clone())
    }

    fn bbox_mode(mut self, f: impl Fn(i32, i32) -> BboxModeUtility) -> Self {
        let bbox_util = f(self.width.get() as i32, self.height.get() as i32);
        self.bbox_mode = bbox_util.into();

        let bbox = match bbox_util {
            BboxModeUtility::Automatic(bbox) | BboxModeUtility::Manual(bbox) => bbox,
            BboxModeUtility::FullImage => {
                let width = self.width.get() as i32;
                let height = self.height.get() as i32;

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

    fn set_frame(&mut self, frame_name: FrameId, sprite_sequence_id: SpriteSequenceId) {
        let path_to_sprite = format!("sprites/{0}/{0}.yy", self.common_data.name);
        let path_to_sprite = Path::new(&path_to_sprite);
        // Update the Frame
        self.frames.push(CommonData::new(frame_name));

        // Update the Sequence
        let track: &mut Track = &mut self.sequence.tracks[0];
        track.keyframes.keyframes.push(SpriteKeyframe {
            id: sprite_sequence_id,
            key: self.frames.len() as f32 - 1.0,
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
        self.sequence.length = self.frames.len() as f32;
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
            let w = me.width.get() as i32;
            let h = me.height.get() as i32;

            let (origin, (xorigin, yorigin)) = origin.to_origin((w, h));
            me.origin = origin;
            me.sequence.xorigin = xorigin;
            me.sequence.yorigin = yorigin;

            me.sequence.lock_origin = locked;
        })
    }
    fn playback_speed(self, speed_type: PlaybackSpeed, speed: f32) -> Self {
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
        &self.common_data.name
    }

    fn set_name(&mut self, name: String) {
        self.common_data.name = name.clone();
        let new_path = format!(
            "{base}/{name}/{name}.yy",
            base = Self::SUBPATH_NAME,
            name = name
        );
        let new_path = Path::new(&new_path);
        let track: &mut Track = &mut self.sequence.tracks[0];
        for kf in track.keyframes.keyframes.iter_mut() {
            kf.channels.zero.id.path = new_path.to_owned();
        }
    }

    fn set_parent_view_path(&mut self, vp: yy_typings::ViewPath) {
        self.parent = vp;
    }

    fn parent_view_path(&self) -> ViewPath {
        self.parent.clone()
    }

    fn get_handler(yyp_boss: &YypBoss) -> &YyResourceHandler<Self> {
        &yyp_boss.sprites
    }

    fn get_handler_mut(yyp_boss: &mut YypBoss) -> &mut YyResourceHandler<Self> {
        &mut yyp_boss.sprites
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
                .ok_or_else(|| anyhow::anyhow!("All Sprites *must* have a single SpriteLayer!"))?
                .common_data
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

    // WE DON'T HANDLE LAYERS AT ALL IN THIS CODE --
    // WE WILL EVENTUALLY.
    fn deserialize_associated_data(
        &self,
        dir_path: &Path,
        _: &TrailingCommaUtility,
    ) -> Result<HashMap<FrameId, SpriteImageBuffer>, SerializedDataError> {
        let mut output = HashMap::new();

        for frame in self.frames.iter() {
            let path_to_image = dir_path.join(&format!("{}.png", frame.name.inner()));

            match image::open(&path_to_image) {
                Ok(image) => output.insert(frame.name, image.to_rgba8()),
                Err(e) => {
                    return Err(SerializedDataError::BadData(format!(
                        "we couldn't read {:#?} -- {}",
                        path_to_image, e
                    )));
                }
            };
        }

        Ok(output)
    }

    fn serialize_associated_data_into_data(
        working_directory: &Path,
        associated_data: &Self::AssociatedData,
    ) -> Result<SerializedData, SerializedDataError> {
        for (frame_id, img) in associated_data {
            let path = working_directory.join(format!("{}.png", frame_id.inner()));

            img.save(&path)
                .map_err(SerializedDataError::CouldNotWriteImage)?;
        }

        Ok(SerializedData::Filepath {
            data: working_directory.to_owned(),
        })
    }

    fn deserialize_associated_data_from_data(
        &self,
        incoming_data: &SerializedData,
        tcu: &TrailingCommaUtility,
    ) -> Result<Self::AssociatedData, SerializedDataError> {
        match incoming_data {
            SerializedData::Value { .. } => Err(SerializedDataError::CannotUseValue),
            SerializedData::Filepath { data: p } => self.deserialize_associated_data(p, tcu),
            SerializedData::DefaultValue => {
                let output = self
                    .frames
                    .iter()
                    .map(|name| {
                        (
                            name.name,
                            SpriteImageBuffer::new(
                                self.width.get() as u32,
                                self.height.get() as u32,
                            ),
                        )
                    })
                    .collect();

                Ok(output)
            }
        }
    }

    fn cleanup_on_replace(&self, mut files: impl FileHolder) {
        // first, clean up the layer folders...
        let base_path = Path::new(&self.common_data.name);
        let layers_path = base_path.join("layers");

        // clean up the composite image...
        for frame in self.frames.iter() {
            let name = frame.name.inner().to_string();
            let path = Path::new(&name);
            files.push(layers_path.join(path));

            let mut file = path.to_owned();
            file.set_extension("png");
            files.push(base_path.join(file));
        }
    }

    fn serialize_yy_file(&self, path: &Path) -> Result<(), crate::FileSerializationError> {
        use serde::Serialize;

        let mut vec = vec![];
        let mut serializer =
            serde_json::ser::Serializer::with_formatter(&mut vec, SpritePrinter::new());
        self.serialize(&mut serializer).unwrap();

        let string = unsafe {
            // We do not emit invalid UTF-8.
            String::from_utf8_unchecked(vec)
        };

        std::fs::write(path, string).map_err(|e| crate::FileSerializationError::Io(e.to_string()))
    }
}

struct SpritePrinter {
    current_indent: usize,
    has_value: bool,
    indent: &'static [u8],
    mode: SpritePrinterMode,
    check_key: bool,
    object_stack: u32,
}

enum SpritePrinterMode {
    Normal,
    Frames(u32),
    Compact(u32),
}

impl SpritePrinter {
    pub fn new() -> Self {
        SpritePrinter {
            current_indent: 0,
            has_value: false,
            indent: b"  ",
            mode: SpritePrinterMode::Normal,
            check_key: false,
            object_stack: 0,
        }
    }
}

use std::io;
impl serde_json::ser::Formatter for SpritePrinter {
    #[inline]
    fn begin_array<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.current_indent += 1;
        self.has_value = false;

        if let SpritePrinterMode::Frames(v) = &mut self.mode {
            *v += 1;
        }

        writer.write_all(b"[")
    }

    #[inline]
    fn end_array<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.current_indent -= 1;

        if self.has_value {
            writer.write_all(b"\n").unwrap();
            indent(writer, self.current_indent, self.indent).unwrap();
        }

        if let SpritePrinterMode::Frames(v) = &mut self.mode {
            *v -= 1;
            if *v == 0 {
                self.mode = SpritePrinterMode::Normal;
            }
        }

        writer.write_all(b"]")
    }

    #[inline]
    fn begin_array_value<W>(&mut self, writer: &mut W, _first: bool) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b"\n").unwrap();
        indent(writer, self.current_indent, self.indent).unwrap();
        Ok(())
    }

    #[inline]
    fn end_array_value<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b",").unwrap();
        self.has_value = true;
        Ok(())
    }

    #[inline]
    fn begin_object<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.current_indent += 1;
        self.object_stack += 1;
        self.has_value = false;

        // increment our object count WITHIN the frames...
        if let SpritePrinterMode::Compact(v) = &mut self.mode {
            *v += 1;
        }
        writer.write_all(b"{")
    }

    #[inline]
    fn end_object<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.current_indent -= 1;
        self.object_stack -= 1;

        // increment our object count WITHIN the frames...
        match &mut self.mode {
            SpritePrinterMode::Normal => {
                if self.has_value {
                    writer.write_all(b"\n").unwrap();
                    indent(writer, self.current_indent, self.indent).unwrap();
                }
            }
            SpritePrinterMode::Frames(_v) => {}
            SpritePrinterMode::Compact(_v) => {
                self.mode = SpritePrinterMode::Normal;
            }
        }

        writer.write_all(b"}")
    }

    #[inline]
    fn begin_object_key<W>(&mut self, writer: &mut W, _first: bool) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        match self.mode {
            SpritePrinterMode::Normal => {
                writer.write_all(b"\n").unwrap();
                self.check_key = true;
                indent(writer, self.current_indent, self.indent)
            }
            SpritePrinterMode::Frames(_) => Ok(()),
            SpritePrinterMode::Compact(_) => Ok(()),
        }
    }

    fn write_string_fragment<W>(&mut self, writer: &mut W, fragment: &str) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        if self.check_key {
            match fragment {
                "frames" | "tracks" => {
                    self.mode = SpritePrinterMode::Frames(0);
                }
                "spriteId" | "events" | "moments" | "layers" => {
                    self.mode = SpritePrinterMode::Compact(0);
                }
                "parent" => {
                    if self.object_stack > 1 {
                        self.mode = SpritePrinterMode::Compact(0);
                    }
                }
                _ => {}
            }
        }

        writer.write_all(fragment.as_bytes())
    }

    fn end_object_key<W>(&mut self, _writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.check_key = false;

        Ok(())
    }

    #[inline]
    fn begin_object_value<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b":").unwrap();

        match self.mode {
            SpritePrinterMode::Normal => writer.write_all(b" "),
            SpritePrinterMode::Frames(v) | SpritePrinterMode::Compact(v) => {
                if v == 0 {
                    writer.write_all(b" ")
                } else {
                    Ok(())
                }
            }
        }
    }

    #[inline]
    fn end_object_value<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.has_value = true;
        writer.write_all(b",").unwrap();
        Ok(())
    }
}

fn indent<W>(wr: &mut W, n: usize, s: &[u8]) -> io::Result<()>
where
    W: ?Sized + io::Write,
{
    for _ in 0..n {
        wr.write_all(s).unwrap();
    }

    Ok(())
}

// fn print_indentation(string: &mut String, indentation: usize) {
//     for _ in 0..indentation {
//         string.push(' ');
//         string.push(' ');
//     }
// }

// fn print_key_value(
//     output: &mut String,
//     indentation: &mut usize,
//     k: &str,
//     v: &serde_json::Value,
//     obj_end_newline: bool,
// ) {
//     print_key(output, indentation, k);
//     print_value(output, indentation, v, obj_end_newline);
// }

// fn print_key(output: &mut String, indentation: &mut usize, k: &str) {
//     use std::fmt::Write;

//     print_indentation(output, *indentation);

//     write!(output, "\"{}\": ", k).unwrap();
// }

// fn print_value(
//     output: &mut String,
//     indentation: &mut usize,
//     v: &serde_json::Value,
//     obj_end_newline: bool,
// ) {
//     use serde_json::Value;
//     use std::fmt::Write;

//     match v {
//         Value::Null => {
//             write!(output, "null,").unwrap();
//         }
//         Value::Bool(v) => {
//             write!(output, "{},", v).unwrap();
//         }
//         Value::Number(numb) => {
//             write!(output, "{},", numb).unwrap();
//         }
//         Value::String(str) => {
//             write!(output, "\"{}\",", str).unwrap();
//         }
//         Value::Array(arr) => {
//             if arr.is_empty() {
//                 write!(output, "[],").unwrap();
//             } else {
//                 write!(output, "[\n").unwrap();
//                 *indentation += 1;
//                 for v in arr.iter() {
//                     print_indentation(output, *indentation);
//                     print_value(output, indentation, v, false);
//                     write!(output, "\n").unwrap();
//                 }

//                 *indentation -= 1;
//                 print_indentation(output, *indentation);
//                 write!(output, "],").unwrap();
//             }
//         }
//         Value::Object(obj) => {
//             print_object(output, indentation, obj, true, obj_end_newline);
//         }
//     }
// }

// fn print_object(
//     output: &mut String,
//     indentation: &mut usize,
//     obj: &serde_json::Map<String, serde_json::Value>,
//     end_comma: bool,
//     end_newline: bool,
// ) {
//     use std::fmt::Write;

//     if obj.is_empty() {
//         write!(output, "{{}}").unwrap();
//     } else {
//         write!(output, "{{\n").unwrap();
//         *indentation += 1;

//         for (k, v) in obj.iter() {
//             print_key_value(output, indentation, k, v, end_newline);
//             write!(output, "\n").unwrap();
//         }

//         *indentation -= 1;
//         print_indentation(output, *indentation);
//         write!(output, "}}").unwrap();
//     }

//     if end_comma {
//         write!(output, ",").unwrap();
//     }

//     if end_newline {
//         output.push('\n');
//     }
// }

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Bbox {
    pub top_left: (i32, i32),
    pub bottom_right: (i32, i32),
}

#[derive(
    Debug,
    Copy,
    Clone,
    strum_macros::EnumIter,
    strum_macros::Display,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
)]
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
    Custom { x: i32, y: i32 },
}

impl OriginUtility {
    pub fn from_origin(o: Origin, origin_pos: (i32, i32)) -> OriginUtility {
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

    pub fn to_origin(self, canvas_dimensions: (i32, i32)) -> (Origin, (i32, i32)) {
        let w = canvas_dimensions.0;
        let h = canvas_dimensions.1;

        match self {
            OriginUtility::Custom { x, y } => (Origin::Custom, (x, y)),
            OriginUtility::TopLeft => (Origin::TopLeft, (0, 0)),
            OriginUtility::TopCenter => (Origin::TopCenter, ((w / 2), 0)),
            OriginUtility::TopRight => (Origin::TopRight, (w, 0)),
            OriginUtility::MiddleLeft => (Origin::MiddleLeft, (0, h / 2)),
            OriginUtility::MiddleCenter => (Origin::MiddleCenter, (w / 2, h / 2)),
            OriginUtility::MiddleRight => (Origin::MiddleRight, (w, h / 2)),
            OriginUtility::BottomLeft => (Origin::BottomLeft, (0, h)),
            OriginUtility::BottomCenter => (Origin::BottomCenter, (w / 2, h)),
            OriginUtility::BottomRight => (Origin::BottomRight, (w, h)),
        }
    }

    pub fn iter() -> impl Iterator<Item = Self> + Clone {
        <Self as strum::IntoEnumIterator>::iter()
    }
}

#[derive(
    Debug,
    Copy,
    Clone,
    strum_macros::EnumIter,
    strum_macros::Display,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum BboxModeUtility {
    Automatic(Bbox),
    FullImage,
    Manual(Bbox),
}

impl BboxModeUtility {
    pub fn to_bbox(self, canvas_dims: (i32, i32)) -> (BBoxMode, Bbox) {
        let bbox_mode: BBoxMode = self.into();

        let bbox = match self {
            BboxModeUtility::Automatic(bbox) | BboxModeUtility::Manual(bbox) => bbox,
            BboxModeUtility::FullImage => Bbox {
                top_left: (0, 0),
                bottom_right: canvas_dims,
            },
        };

        (bbox_mode, bbox)
    }

    pub fn iter() -> impl Iterator<Item = Self> + Clone {
        <Self as strum::IntoEnumIterator>::iter()
    }
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
    pub fn from_bbox_data(bbox_mode: BBoxMode, bbox: Bbox) -> BboxModeUtility {
        match bbox_mode {
            BBoxMode::Automatic => BboxModeUtility::Automatic(bbox),
            BBoxMode::FullImage => BboxModeUtility::FullImage,
            BBoxMode::Manual => BboxModeUtility::Manual(bbox),
        }
    }
}
