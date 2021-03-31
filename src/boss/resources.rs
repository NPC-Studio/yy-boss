use super::YyResource;
use std::fmt;
use yy_typings::{
    object_yy::Object, script::Script, shader::Shader, sounds::Sound, sprite_yy::Sprite,
    AnimationCurve, Extension, Font, Note, Path, Room, Sequence, TileSet, Timeline,
};

#[derive(
    Debug, PartialEq, Eq, Ord, PartialOrd, Copy, Clone, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum Resource {
    Sprite,
    Script,
    Object,
    Note,
    Shader,
    Sound,

    // unidentified resources
    AnimationCurve,
    Extension,
    Font,
    Path,
    Room,
    Sequence,
    TileSet,
    Timeline,
}

impl Resource {
    pub fn subpath_name(&self) -> &'static str {
        match self {
            Resource::Sprite => Sprite::SUBPATH_NAME,
            Resource::Script => Script::SUBPATH_NAME,
            Resource::Object => Object::SUBPATH_NAME,
            Resource::Note => Note::SUBPATH_NAME,
            Resource::Shader => Shader::SUBPATH_NAME,
            Resource::AnimationCurve => AnimationCurve::SUBPATH_NAME,
            Resource::Extension => Extension::SUBPATH_NAME,
            Resource::Font => Font::SUBPATH_NAME,
            Resource::Path => Path::SUBPATH_NAME,
            Resource::Room => Room::SUBPATH_NAME,
            Resource::Sequence => Sequence::SUBPATH_NAME,
            Resource::Sound => Sound::SUBPATH_NAME,
            Resource::TileSet => TileSet::SUBPATH_NAME,
            Resource::Timeline => Timeline::SUBPATH_NAME,
        }
    }

    pub fn parse_subpath(subpath: &str) -> Option<Resource> {
        match subpath {
            Sprite::SUBPATH_NAME => Some(Resource::Sprite),
            Script::SUBPATH_NAME => Some(Resource::Script),
            Object::SUBPATH_NAME => Some(Resource::Object),
            Note::SUBPATH_NAME => Some(Resource::Note),
            Shader::SUBPATH_NAME => Some(Resource::Shader),
            AnimationCurve::SUBPATH_NAME => Some(Resource::AnimationCurve),
            Extension::SUBPATH_NAME => Some(Resource::Extension),
            Font::SUBPATH_NAME => Some(Resource::Font),
            Path::SUBPATH_NAME => Some(Resource::Path),
            Room::SUBPATH_NAME => Some(Resource::Room),
            Sequence::SUBPATH_NAME => Some(Resource::Sequence),
            Sound::SUBPATH_NAME => Some(Resource::Sound),
            TileSet::SUBPATH_NAME => Some(Resource::TileSet),
            Timeline::SUBPATH_NAME => Some(Resource::Timeline),
            _ => None,
        }
    }

    pub fn can_manipulate(&self) -> bool {
        match self {
            Resource::Sprite
            | Resource::Script
            | Resource::Object
            | Resource::Note
            | Resource::Sound
            | Resource::Shader => true,
            Resource::AnimationCurve
            | Resource::Extension
            | Resource::Font
            | Resource::Path
            | Resource::Room
            | Resource::Sequence
            | Resource::TileSet
            | Resource::Timeline => false,
        }
    }
}

impl fmt::Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Resource::Sprite => write!(f, "sprite"),
            Resource::Script => write!(f, "script"),
            Resource::Object => write!(f, "object"),
            Resource::Note => write!(f, "note"),
            Resource::Shader => write!(f, "shader"),
            Resource::AnimationCurve => write!(f, "animation curve"),
            Resource::Extension => write!(f, "extension"),
            Resource::Font => write!(f, "font"),
            Resource::Path => write!(f, "path"),
            Resource::Room => write!(f, "room"),
            Resource::Sequence => write!(f, "sequence"),
            Resource::Sound => write!(f, "sound"),
            Resource::TileSet => write!(f, "tile set"),
            Resource::Timeline => write!(f, "timeline"),
        }
    }
}
