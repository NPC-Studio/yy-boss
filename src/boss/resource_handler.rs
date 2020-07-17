use anyhow::Result as AnyResult;
use super::directory_manager::DirectoryManager;

pub trait ResourceHandler {
    fn new() -> Self;
    fn serialize(&mut self, directory_manager: &DirectoryManager) -> AnyResult<()>;
}
