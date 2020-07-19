use std::{iter::Skip, str::Split};
use thiserror::Error;
use yy_typings::{utils::PathValidator, TexturePathLocation, ViewPathLocation};

pub trait ViewPathLocationExt {
    /// Iterates over the folder subpaths -- not including the root `folders`.
    /// This can, therefore, be empty.
    fn component_paths(&self) -> Skip<Split<'_, char>>;
    fn join(&self, other: &str) -> ViewPathLocation;
}

impl ViewPathLocationExt for ViewPathLocation {
    fn component_paths(&self) -> Skip<Split<'_, char>> {
        self.0.split('/').skip(1)
    }

    fn join(&self, other: &str) -> ViewPathLocation {
        let path_name = self.0.trim_end_matches(".yy");

        ViewPathLocation(format!("{}/{}.yy", path_name, other))
    }
}

pub trait TexturePathLocationExt {
    fn new(texture_group_name: &str, pv: &PathValidator) -> Result<TexturePathLocation, PathError>;
}

impl TexturePathLocationExt for TexturePathLocation {
    fn new(texture_group_name: &str, pv: &PathValidator) -> Result<TexturePathLocation, PathError> {
        if pv.is_valid(texture_group_name) == false {
            Err(PathError::PathContainsInvalidCharacters)
        } else {
            Ok(TexturePathLocation(texture_group_name.to_string()))
        }
    }
}

#[derive(Debug, Error)]
pub enum PathError {
    #[error("path should only contain 0-9, a-zA-Z, or _, and should not begin with 0-9.")]
    PathContainsInvalidCharacters,
    // #[error("path must end in a .yy file")]
    // PathDoesNotEndInYyFile,
}

pub trait PathStrExt {
    fn trim_yy(&self) -> &str;
}

impl PathStrExt for &str {
    fn trim_yy(&self) -> &str {
        self.trim_end_matches(".yy")
    }
}
