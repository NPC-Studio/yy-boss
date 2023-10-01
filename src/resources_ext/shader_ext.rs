use crate::{
    utils, FileHolder, Resource, SerializedData, SerializedDataError, YyResource,
    YyResourceHandler, YypBoss,
};
use std::path::Path;
use yy_typings::{Shader, TrailingCommaUtility, ViewPath};

impl YyResource for Shader {
    type AssociatedData = ShaderFile;

    const SUBPATH_NAME: &'static str = "shaders";
    const RESOURCE: Resource = Resource::Shader;

    fn name(&self) -> &str {
        &self.common_data.name
    }

    fn set_name(&mut self, name: String) {
        self.common_data.name = name;
    }

    fn set_parent_view_path(&mut self, vp: ViewPath) {
        self.parent = vp;
    }

    fn parent_view_path(&self) -> ViewPath {
        self.parent.clone()
    }

    fn get_handler(yyp_boss: &YypBoss) -> &YyResourceHandler<Self> {
        &yyp_boss.shaders
    }

    fn get_handler_mut(yyp_boss: &mut YypBoss) -> &mut YyResourceHandler<Self> {
        &mut yyp_boss.shaders
    }

    fn serialize_associated_data(
        &self,
        wd: &Path,
        data: &Self::AssociatedData,
    ) -> anyhow::Result<()> {
        let vtx_path = wd
            .join(&self.common_data.name)
            .with_extension(Self::VERT_FILE_ENDING);
        let frag_path = wd
            .join(&self.common_data.name)
            .with_extension(Self::FRAG_FILE_ENDING);

        std::fs::write(vtx_path, &data.vertex)?;
        std::fs::write(frag_path, &data.pixel)?;

        Ok(())
    }

    fn deserialize_associated_data(
        &self,
        wd: &Path,
        _: &TrailingCommaUtility,
    ) -> Result<Self::AssociatedData, SerializedDataError> {
        let vtx_path = wd
            .join(&self.common_data.name)
            .with_extension(Self::VERT_FILE_ENDING);
        let frag_path = wd
            .join(&self.common_data.name)
            .with_extension(Self::FRAG_FILE_ENDING);

        let assoc_data = Self::AssociatedData {
            vertex: std::fs::read_to_string(vtx_path).map_err(|e| {
                SerializedDataError::CouldNotDeserializeFile(crate::FileSerializationError::Io(
                    e.to_string(),
                ))
            })?,
            pixel: std::fs::read_to_string(frag_path).map_err(|e| {
                SerializedDataError::CouldNotDeserializeFile(crate::FileSerializationError::Io(
                    e.to_string(),
                ))
            })?,
        };

        Ok(assoc_data)
    }

    fn serialize_associated_data_into_data(
        _: &std::path::Path,
        associated_data: &Self::AssociatedData,
    ) -> Result<SerializedData, SerializedDataError> {
        match serde_json::to_string_pretty(associated_data) {
            Ok(data) => Ok(SerializedData::Value { data }),
            Err(e) => Err(e.into()),
        }
    }

    fn deserialize_associated_data_from_data(
        &self,
        incoming_data: &SerializedData,
        tcu: &TrailingCommaUtility,
    ) -> Result<Self::AssociatedData, SerializedDataError> {
        match incoming_data {
            SerializedData::Value { data: v } => {
                serde_json::from_str(v).map_err(|e| SerializedDataError::InnerError(e.to_string()))
            }
            SerializedData::Filepath { data: v } => {
                utils::deserialize_json_tc(v, tcu).map_err(|e| e.into())
            }
            SerializedData::DefaultValue => Ok(Self::AssociatedData::default()),
        }
    }

    fn cleanup_on_replace(&self, _: impl FileHolder) {
        todo!()
    }
}

#[derive(
    Debug,
    Default,
    PartialEq,
    Eq,
    Ord,
    PartialOrd,
    Clone,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct ShaderFile {
    pub vertex: String,
    pub pixel: String,
}

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Clone, Copy, Hash, strum_macros::EnumIter)]
pub enum ShaderKind {
    Vertex,
    Frag,
}

impl ShaderKind {
    pub fn file_ending(&self) -> &'static str {
        match self {
            ShaderKind::Vertex => Shader::VERT_FILE_ENDING,
            ShaderKind::Frag => Shader::FRAG_FILE_ENDING,
        }
    }

    pub fn iter() -> impl IntoIterator<Item = ShaderKind> {
        <Self as strum::IntoEnumIterator>::iter()
    }
}

impl std::ops::Index<ShaderKind> for ShaderFile {
    type Output = String;

    fn index(&self, index: ShaderKind) -> &Self::Output {
        match index {
            ShaderKind::Vertex => &self.vertex,
            ShaderKind::Frag => &self.pixel,
        }
    }
}

impl std::ops::IndexMut<ShaderKind> for ShaderFile {
    fn index_mut(&mut self, index: ShaderKind) -> &mut Self::Output {
        match index {
            ShaderKind::Vertex => &mut self.vertex,
            ShaderKind::Frag => &mut self.pixel,
        }
    }
}
