use super::{directory_manager::DirectoryManager, utils, FilesystemPath};
use anyhow::Result as AnyResult;
use log::{error, info, trace};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    hash::Hash,
    path::{Path, PathBuf},
};

type PipelineResult = Result<(), PipelineError>;

#[derive(Debug, Default, Clone, Eq)]
pub struct PipelineManager {
    pipelines: BTreeMap<String, Pipeline>,
    dirty: bool,
}

impl PipelineManager {
    const PIPELINE_MANIFEST: &'static str = "pipeline_manifest.json";

    pub(crate) fn new(
        directory_manager: &DirectoryManager,
    ) -> Result<PipelineManager, utils::FileSerializationError> {
        let pipeline_manifest_path =
            directory_manager.boss_file(Path::new(Self::PIPELINE_MANIFEST));

        // If there's no pipeline manifest file, then no worries,
        // just return. Users might not want to make a manifest!
        if pipeline_manifest_path.exists() == false {
            trace!(
                "No pipeline manifest found at path {:#?}...",
                pipeline_manifest_path
            );
            Ok(Self::default())
        } else {
            let pipeline_manifest: BTreeSet<PathBuf> =
                utils::deserialize(&pipeline_manifest_path, None)?;

            let mut pipelines = BTreeMap::new();

            let found_paths = pipeline_manifest
                .clone()
                .into_iter()
                .filter(|path| {
                    let mut joint_path = directory_manager.boss_file(path);
                    joint_path.set_extension("json");

                    if joint_path.exists() {
                        match utils::deserialize(&joint_path, None) {
                            Ok(datum) => {
                                pipelines.insert(path.to_string_lossy().to_string(), datum);
                                true
                            }
                            Err(e) => {
                                error!(
                                    "problem reading {:#?}, even though it was in manifest: {:}.",
                                    path, e
                                );
                                false
                            }
                        }
                    } else {
                        false
                    }
                })
                .collect::<BTreeSet<_>>();

            // If the found paths are not right, then don't do the thing!
            if found_paths.len() != pipeline_manifest.len() {
                let difference = found_paths
                    .difference(&pipeline_manifest)
                    .map(|entry| entry.to_string_lossy().to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                error!("pipeline manifest had invalid entries: [{}]. they will be removed on serialization...", difference);
            }

            let output = Self {
                pipelines,
                dirty: found_paths.len() != pipeline_manifest.len(),
            };

            info!("pipelines loaded in...{:?}", output);

            Ok(output)
        }
    }

    pub(crate) fn serialize(&mut self, directory_manager: &DirectoryManager) -> AnyResult<()> {
        if self.dirty {
            // Serialize Manifest...
            let pipeline_manifest = self
                .pipelines
                .keys()
                .map(|name| Path::new(name).to_owned())
                .collect::<BTreeSet<_>>();

            directory_manager
                .serialize_boss_file(Path::new(Self::PIPELINE_MANIFEST), &pipeline_manifest)?;

            // Serialize each Pipeline..
            for pipeline in self.pipelines.values_mut() {
                if pipeline.dirty {
                    directory_manager.serialize_boss_file(
                        &Path::new(&pipeline.name).with_extension("json"),
                        &pipeline,
                    )?;
                    pipeline.dirty = false;
                }
            }

            // reset dirty
            self.dirty = false;
        }

        Ok(())
    }

    /// Returns a map of all the pipelines currently known.
    pub fn pipelines(&self) -> &BTreeMap<String, Pipeline> {
        &self.pipelines
    }

    /// Returns a specific pipeline by name, if it exists.
    pub fn pipeline(&self, pipeline_name: impl Into<String>) -> Option<&Pipeline> {
        self.pipelines.get(&pipeline_name.into())
    }

    /// Gets all of the destinations for a given source within a given pipeline,
    /// if both the pipeline and a source within that pipeline exist.
    pub fn pipeline_destinations(
        &self,
        pipeline_name: impl Into<String>,
        source_name: impl Into<String>,
    ) -> Option<&PipelineDesinations> {
        self.pipelines
            .get(&pipeline_name.into())
            .and_then(|pl| pl.source_destinations.get(&source_name.into()))
    }

    /// Creates a pipeline. If a pipeline of that name already exists, an error is returned.
    pub fn add_pipeline(&mut self, name: impl Into<String>) -> PipelineResult {
        let name = name.into();

        if self.pipelines.contains_key(&name) {
            Err(PipelineError::PipelineAlreadyExists)
        } else {
            self.pipelines.insert(
                name.clone(),
                Pipeline {
                    name,
                    source_destinations: Default::default(),
                    dirty: true,
                },
            );
            self.dirty = true;
            Ok(())
        }
    }

    /// Adds a source to a given pipeline.
    ///
    /// If a pipeline doesn't exist, an error is returned.
    pub fn add_source_to_pipeline(
        &mut self,
        pipeline_name: impl Into<String>,
        source_name: impl Into<String>,
    ) -> PipelineResult {
        match self.pipelines.get_mut(&pipeline_name.into()) {
            Some(pipeline) => {
                let source_name = source_name.into();
                if pipeline.source_destinations.contains_key(&source_name) {
                    Err(PipelineError::PipelineSourceAlreadyExists)
                } else {
                    pipeline
                        .source_destinations
                        .insert(source_name, Default::default());
                    pipeline.dirty = true;
                    self.dirty = true;
                    Ok(())
                }
            }
            None => Err(PipelineError::PipelineDoesNotExist),
        }
    }

    /// Adds a destination to a given source on a given pipeline.
    ///
    /// If the pipeline doesn't exist or the source doesn't exist on the pipeline,
    /// an error is returned.
    pub fn add_destination_to_source<S: Into<String>>(
        &mut self,
        pipeline_name: S,
        source_name: S,
        destination_key: S,
        destination_value: FilesystemPath,
    ) -> PipelineResult {
        let destination_key = destination_key.into();
        match self.pipelines.get_mut(&pipeline_name.into()) {
            Some(pipeline) => match pipeline.source_destinations.get_mut(&source_name.into()) {
                Some(destinations) => {
                    if destinations.contains_key(&destination_key) {
                        return Err(PipelineError::PipelineDestinationAlreadyExistsOnSource);
                    }

                    destinations.insert(destination_key, destination_value);
                    pipeline.dirty = true;
                    self.dirty = true;
                    Ok(())
                }
                None => Err(PipelineError::PipelineSourceDoesNotExist),
            },
            None => Err(PipelineError::PipelineDoesNotExist),
        }
    }

    /// Adds a destination to a given source on a given pipeline.
    ///
    /// **If any elements do not exist,
    /// they will be created. If a destination exists on a source/pipeline which already exists, it will
    /// be replaced and lost.**
    pub fn add_destination_to_source_rf<S: Into<String>>(
        &mut self,
        pipeline_name: S,
        source_name: S,
        destination_key: S,
        destination_value: FilesystemPath,
    ) {
        let destination_key = destination_key.into();
        let pipeline = self.pipelines.entry(pipeline_name.into()).or_default();
        let destinations = pipeline
            .source_destinations
            .entry(source_name.into())
            .or_default();
        destinations.insert(destination_key, destination_value);
        pipeline.dirty = true;
        self.dirty = true;
    }

    /// Removes a given **pipeline** from the manager. If any data is on the pipeline,
    /// it will be lost permanently!
    ///
    /// If the *pipeline* does not exist, an error is
    /// returned.
    pub fn remove_pipeline(&mut self, pipeline_name: impl Into<String>) -> PipelineResult {
        match self.pipelines.remove(&pipeline_name.into()) {
            Some(_) => Ok(()),
            None => Err(PipelineError::PipelineDoesNotExist),
        }
    }

    /// Removes a given **source** from a **pipeline**. If any destinations are in the
    /// source, it will be lost permanently!
    ///
    /// If the *pipeline* does not exist, or if the *source does not exist on the pipeline*,
    /// an error is returned.
    pub fn remove_source_from_pipeline(
        &mut self,
        pipeline_name: impl Into<String>,
        source_name: impl Into<String>,
    ) -> PipelineResult {
        let pipeline = self
            .pipelines
            .get_mut(&pipeline_name.into())
            .ok_or(PipelineError::PipelineDoesNotExist)?;

        pipeline
            .source_destinations
            .remove(&source_name.into())
            .ok_or(PipelineError::PipelineSourceDoesNotExist)?;

        Ok(())
    }

    /// Removes a given **destination** from a **source**.
    ///
    /// If the *pipeline* does not exist, or if the *source does not exist on the pipeline*,
    /// or if the *destination does not exist on the source*, an error is returned.
    pub fn remove_destination_from_source(
        &mut self,
        pipeline_name: impl Into<String>,
        source_name: impl Into<String>,
        destination_name: &str,
    ) -> PipelineResult {
        let pipeline = self
            .pipelines
            .get_mut(&pipeline_name.into())
            .ok_or(PipelineError::PipelineDoesNotExist)?;

        let destinations = pipeline
            .source_destinations
            .get_mut(&source_name.into())
            .ok_or(PipelineError::PipelineSourceDoesNotExist)?;

        if destinations.remove(destination_name).is_some() {
            Ok(())
        } else {
            Err(PipelineError::PipelineDestinationDoesNotExist)
        }
    }
}

pub type PipelineDesinations = BTreeMap<String, FilesystemPath>;

#[derive(Debug, Eq, Serialize, Deserialize, Clone, Default)]
pub struct Pipeline {
    pub name: String,
    pub source_destinations: BTreeMap<String, PipelineDesinations>,
    #[serde(skip)]
    dirty: bool,
}

impl Hash for Pipeline {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.source_destinations.hash(state);
    }
}

impl PartialEq for Pipeline {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.source_destinations == other.source_destinations
    }
}

impl PartialOrd for Pipeline {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for Pipeline {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialEq for PipelineManager {
    fn eq(&self, other: &Self) -> bool {
        self.pipelines == other.pipelines
    }
}

use thiserror::Error;
#[derive(Debug, Copy, Clone, Error, PartialEq, Eq)]
pub enum PipelineError {
    #[error("no pipeline by that name exists")]
    PipelineDoesNotExist,
    #[error("no pipeline source exists on that pipeline")]
    PipelineSourceDoesNotExist,
    #[error("no pipeline destinations exists on that pipeline source")]
    PipelineDestinationDoesNotExist,
    #[error("a pipeline by that name already exists")]
    PipelineAlreadyExists,
    #[error("a pipeline source by that name already exists")]
    PipelineSourceAlreadyExists,
    #[error("pipeline destination already exists on a source by that name")]
    PipelineDestinationAlreadyExistsOnSource,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    fn trivial() {
        let mut pipelines = PipelineManager::default();
        pipelines.add_pipeline("sprites").unwrap();
        pipelines
            .add_source_to_pipeline("sprites", "spr_source_sprite")
            .unwrap();
        pipelines
            .add_destination_to_source(
                "sprites",
                "spr_source_sprite",
                "spr_source_sprite",
                FilesystemPath {
                    name: "spr_destination".to_string(),
                    path: Path::new("sprites/spr_destination/spr_destination.yy").to_owned(),
                },
            )
            .unwrap();
        let our_pipeline = pipelines.pipeline("sprites").unwrap();

        let raw_pipeline = r#"{
            "name": "sprites",
            "source_destinations": {
                "spr_source_sprite": {
                    "spr_source_sprite": {
                        "name": "spr_destination",
                        "path": "sprites/spr_destination/spr_destination.yy"
                    }
                }
            }
        }"#;

        let proof_pipeline: Pipeline = serde_json::from_str(raw_pipeline).unwrap();

        assert_eq!(proof_pipeline, *our_pipeline);
    }

    #[test]
    fn errors() {
        let mut pipelines = PipelineManager::default();

        let destination = FilesystemPath {
            name: "spr_destination".to_string(),
            path: Path::new("sprites/spr_destination/spr_destination.yy").to_owned(),
        };

        assert_eq!(
            pipelines.add_source_to_pipeline("sprites", "spr_source"),
            Err(PipelineError::PipelineDoesNotExist)
        );
        assert_eq!(
            pipelines.add_destination_to_source(
                "sprites",
                "spr_source",
                "spr_source",
                destination.clone()
            ),
            Err(PipelineError::PipelineDoesNotExist)
        );

        pipelines.add_pipeline("sprites").unwrap();
        assert_eq!(
            pipelines.add_destination_to_source(
                "sprites",
                "spr_source",
                "spr_source",
                destination.clone()
            ),
            Err(PipelineError::PipelineSourceDoesNotExist)
        );

        pipelines
            .add_source_to_pipeline("sprites", "spr_source")
            .unwrap();

        assert_eq!(
            pipelines.add_pipeline("sprites"),
            Err(PipelineError::PipelineAlreadyExists)
        );

        assert_eq!(
            pipelines.add_source_to_pipeline("sprites", "spr_source"),
            Err(PipelineError::PipelineSourceAlreadyExists)
        );

        pipelines
            .add_destination_to_source("sprites", "spr_source", "spr_source", destination.clone())
            .unwrap();
        assert_eq!(
            pipelines.add_destination_to_source("sprites", "spr_source", "spr_source", destination),
            Err(PipelineError::PipelineDestinationAlreadyExistsOnSource)
        );

        pipelines
            .remove_destination_from_source("sprites", "spr_source", "spr_source")
            .unwrap();

        assert_eq!(
            pipelines.remove_destination_from_source("sprites", "spr_source", "spr_source"),
            Err(PipelineError::PipelineDestinationDoesNotExist)
        );
    }

    #[test]
    fn symmetry() {
        fn harness(
            mut pipeline: PipelineManager,
            add_function: impl Fn(&mut PipelineManager) -> PipelineResult,
            remove_function: impl Fn(&mut PipelineManager) -> PipelineResult,
        ) -> PipelineManager {
            let mut original_clone = pipeline.clone();
            println!("Original...{:#?}", original_clone);

            add_function(&mut pipeline).unwrap();
            println!("Adding...{:#?}", pipeline);

            assert_ne!(original_clone, pipeline);

            remove_function(&mut pipeline).unwrap();
            println!("Removing...{:#?}", pipeline);

            assert_eq!(original_clone, pipeline);

            add_function(&mut original_clone).unwrap();
            original_clone
        }

        println!("Adding pipeline...");
        let p = harness(
            PipelineManager::default(),
            |p| p.add_pipeline("sprites"),
            |p| p.remove_pipeline("sprites"),
        );

        println!("Adding source...");
        let p = harness(
            p,
            |p| p.add_source_to_pipeline("sprites", "spr_source"),
            |p| p.remove_source_from_pipeline("sprites", "spr_source"),
        );

        let destination0c = FilesystemPath {
            name: "spr_destination".to_string(),
            path: Path::new("sprites/spr_destination0/spr_destination0.yy").to_owned(),
        };

        let destination1c = FilesystemPath {
            name: "spr_destination".to_string(),
            path: Path::new("sprites/spr_destination1/spr_destination1.yy").to_owned(),
        };

        println!("Adding destination and source...");
        harness(
            p,
            move |p| {
                p.add_destination_to_source(
                    "sprites",
                    "spr_source",
                    "spr_destination0",
                    destination0c.clone(),
                )
                .unwrap();
                p.add_destination_to_source(
                    "sprites",
                    "spr_source",
                    "spr_destination1",
                    destination1c.clone(),
                )
            },
            |p| {
                p.remove_destination_from_source("sprites", "spr_source", "spr_destination0")
                    .unwrap();
                p.remove_destination_from_source("sprites", "spr_source", "spr_destination1")
            },
        );
    }

    #[test]
    fn dirty() {
        let mut p = PipelineManager::default();
        p.add_pipeline("s").unwrap();
        assert!(p.dirty);
        assert!(p.pipelines["s"].dirty);

        p.dirty = false;
        p.pipelines.get_mut("s").unwrap().dirty = false;

        p.add_source_to_pipeline("s", "so").unwrap();
        assert!(p.dirty);
        assert!(p.pipelines["s"].dirty);

        p.dirty = false;
        p.pipelines.get_mut("s").unwrap().dirty = false;

        p.add_destination_to_source("s", "so", "d", Default::default())
            .unwrap();
        assert!(p.dirty);
        assert!(p.pipelines["s"].dirty);

        p.dirty = false;
        p.pipelines.get_mut("s").unwrap().dirty = false;

        p.add_destination_to_source_rf("new", "so", "d", Default::default());
        assert!(p.dirty);
        assert!(p.pipelines["s"].dirty == false);
        assert!(p.pipelines["new"].dirty);
    }
}
