use super::YyResource;
use yy_typings::sprite::Sprite;

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Copy, Clone, Hash)]
pub enum Resource {
    Sprite,
}

impl Resource {
    pub fn base_name(&self) -> &'static str {
        match self {
            Resource::Sprite => Sprite::SUBPATH_NAME,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct CreatedResource(pub(crate) Resource);

#[derive(Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct RemovedResource(pub(crate) Resource);