use super::YyResource;
use std::fmt;
use yy_typings::{object_yy::Object, script::Script, sprite_yy::Sprite};

#[derive(
    Debug, PartialEq, Eq, Ord, PartialOrd, Copy, Clone, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum Resource {
    Sprite,
    Script,
    Object,
}

impl Resource {
    pub fn base_name(&self) -> &'static str {
        match self {
            Resource::Sprite => Sprite::SUBPATH_NAME,
            Resource::Script => Script::SUBPATH_NAME,
            Resource::Object => Object::SUBPATH_NAME,
        }
    }
}

impl fmt::Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Resource::Sprite => write!(f, "sprite"),
            Resource::Script => write!(f, "script"),
            Resource::Object => write!(f, "object"),
        }
    }
}
