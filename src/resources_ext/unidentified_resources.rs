use crate::{
    FileHolder, Resource, SerializedData, SerializedDataError, YyResource, YyResourceHandler,
    YypBoss,
};
use std::path::Path;
use yy_typings::{
    utils::TrailingCommaUtility, AnimationCurve, Extension, Font, Path as YyPath, Room, Sequence,
    Timeline, ViewPath,
};

macro_rules! unidentified_resource {
    ($struct_name:ident, $subpath:expr, $resource_kind:expr, $accessor:ident) => {
        impl YyResource for $struct_name {
            type AssociatedData = ();

            const SUBPATH_NAME: &'static str = $subpath;
            const RESOURCE: Resource = $resource_kind;

            fn name(&self) -> &str {
                &self.name
            }

            fn set_name(&mut self, name: String) {
                self.name = name;
            }

            fn set_parent_view_path(&mut self, vp: ViewPath) {
                self.parent = vp;
            }

            fn parent_view_path(&self) -> ViewPath {
                self.parent.clone()
            }

            fn get_handler(yyp_boss: &YypBoss) -> &YyResourceHandler<Self> {
                &yyp_boss.$accessor
            }

            fn get_handler_mut(yyp_boss: &mut YypBoss) -> &mut YyResourceHandler<Self> {
                &mut yyp_boss.$accessor
            }

            fn serialize_associated_data(
                &self,
                _: &Path,
                _: &Self::AssociatedData,
            ) -> anyhow::Result<()> {
                anyhow::bail!("we cannot serialization operations for this yyfile type")
            }

            fn deserialize_associated_data(
                &self,
                _: &Path,
                _: &TrailingCommaUtility,
            ) -> Result<Self::AssociatedData, SerializedDataError> {
                Ok(())
            }

            fn serialize_associated_data_into_data(
                _: &Path,
                _: &Self::AssociatedData,
            ) -> Result<SerializedData, SerializedDataError> {
                Ok(SerializedData::Value {
                    data: String::new(),
                })
            }

            fn deserialize_associated_data_from_data(
                &self,
                _: &SerializedData,
                _: &TrailingCommaUtility,
            ) -> Result<Self::AssociatedData, SerializedDataError> {
                Ok(())
            }

            fn cleanup_on_replace(&self, _: impl FileHolder) {}
        }
    };
}
unidentified_resource!(
    AnimationCurve,
    "animcurves",
    Resource::AnimationCurve,
    animation_curves
);
unidentified_resource!(Extension, "extensions", Resource::Extension, extensions);
unidentified_resource!(Font, "fonts", Resource::Font, fonts);
unidentified_resource!(YyPath, "paths", Resource::Path, paths);
unidentified_resource!(Sequence, "sequences", Resource::Sequence, sequences);
unidentified_resource!(Room, "rooms", Resource::Room, rooms);
// unidentified_resource!(Sound, "sounds", Resource::Sound, sounds);
unidentified_resource!(Timeline, "timelines", Resource::Timeline, timelines);
