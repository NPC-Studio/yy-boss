use super::{directory_manager::DirectoryManager, utils, FilesystemPath};
use log::{error, trace};
use serde::{Deserialize, Serialize};
use std::{
    collections::{btree_map::Entry, BTreeMap, BTreeSet},
    hash::Hash,
    path::{Path, PathBuf},
};
use utils::SerializationFormat;

type PipelineResult = Result<(), PipelineError>;
const CURRENT_PIPELINE_MANIFEST_SEMVER: semver::Version = semver::Version {
    major: 0,
    minor: 1,
    patch: 0,
    build: Vec::new(),
    pre: Vec::new(),
};

#[derive(Debug, Default, Clone, Eq)]
pub struct PipelineManager {
    pipelines: BTreeMap<String, Pipeline>,
    pipelines_to_remove: Vec<PathBuf>,
    dirty: bool,
}

impl PipelineManager {
    const PIPELINE_MANIFEST: &'static str = "pipeline_manifest.json";

    pub(crate) fn new(directory_manager: &DirectoryManager) -> PipelineManager {
        let pipeline_manifest_path =
            directory_manager.boss_file(Path::new(Self::PIPELINE_MANIFEST));

        let mut dirty = false;

        // If there's no pipeline manifest file, then no worries,
        // just return. Users might not want to make a manifest!
        if pipeline_manifest_path.exists() == false {
            trace!(
                "No pipeline manifest found at path {:#?}...",
                pipeline_manifest_path
            );
            Self::default()
        } else {
            let pipeline_manifest: PipelineManifest = {
                match utils::deserialize_json::<PipelineManifest>(&pipeline_manifest_path) {
                    Ok(v) => v,
                    Err(_) => {
                        if let Ok(pipeline_manifest) =
                            utils::deserialize_json::<BTreeSet<PathBuf>>(&pipeline_manifest_path)
                        {
                            let pipelines: BTreeSet<PipelineDescriptor> =
                                pipeline_manifest.into_iter().map(|v| v.into()).collect();
                            dirty = true;

                            PipelineManifest {
                                pipelines,
                                version: CURRENT_PIPELINE_MANIFEST_SEMVER,
                            }
                        } else {
                            error!(
                                "We couldn't parse the pipeline manifest! It looked like {:?}",
                                std::fs::read_to_string(&pipeline_manifest_path)
                            );
                            return Self::default();
                        }
                    }
                }
            };

            let mut pipelines = BTreeMap::new();

            let found_paths = pipeline_manifest
                .pipelines
                .clone()
                .into_iter()
                .filter(|pipeline_desc| {
                    let mut joint_path = directory_manager.boss_file(&pipeline_desc.path);
                    joint_path.set_extension(pipeline_desc.serialization_format.file_ending());

                    if joint_path.exists() {
                        match pipeline_desc
                            .serialization_format
                            .deserialize_and_read::<Pipeline>(&joint_path)
                        {
                            Ok(mut datum) => {
                                datum.serialization_format = pipeline_desc.serialization_format;
                                pipelines.insert(
                                    pipeline_desc.path.to_string_lossy().to_string(),
                                    datum,
                                );
                                true
                            }
                            Err(e) => {
                                error!(
                                    "problem reading {:#?}, even though it was in manifest: {:}.",
                                    pipeline_desc.path, e
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
            if found_paths.len() != pipeline_manifest.pipelines.len() {
                dirty = true;
                let difference = found_paths
                    .difference(&pipeline_manifest.pipelines)
                    .map(|entry| entry.path.to_string_lossy().to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                error!("pipeline manifest had invalid entries: [{}]. they will be removed on serialization...", difference);
            }

            Self {
                pipelines,
                dirty,
                pipelines_to_remove: vec![],
            }
        }
    }

    pub(crate) fn serialize(
        &mut self,
        directory_manager: &DirectoryManager,
    ) -> Result<(), utils::FileSerializationError> {
        if self.dirty {
            // Serialize Manifest...
            let pipelines = self
                .pipelines
                .iter()
                .map(|(name, val)| PipelineDescriptor {
                    path: Path::new(name).to_owned(),
                    serialization_format: val.serialization_format,
                })
                .collect::<BTreeSet<PipelineDescriptor>>();

            let pipeline_manifest = PipelineManifest {
                pipelines,
                version: CURRENT_PIPELINE_MANIFEST_SEMVER,
            };

            directory_manager.serialize_boss_file(
                Path::new(Self::PIPELINE_MANIFEST),
                serde_json::to_string_pretty(&pipeline_manifest).unwrap(),
            )?;
            // reset dirty
            self.dirty = false;
        }

        for pipeline in self.pipelines_to_remove.drain(..) {
            let path = directory_manager.boss_file(&pipeline);

            if let Err(e) = std::fs::remove_file(&path) {
                error!("Couldn't remove path {:?}...{:#?}", path, e);
            }
        }

        // Serialize each Pipeline..
        for pipeline in self.pipelines.values_mut() {
            if pipeline.dirty {
                let pipeline_datum = pipeline.serialization_format.serialize(pipeline)?;

                directory_manager.serialize_boss_file(
                    &Path::new(&pipeline.name)
                        .with_extension(pipeline.serialization_format.file_ending()),
                    pipeline_datum,
                )?;
                pipeline.dirty = false;
            }
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
    pub fn add_pipeline(
        &mut self,
        name: impl Into<String>,
        sf: SerializationFormat,
    ) -> PipelineResult {
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
                    serialization_format: sf,
                },
            );
            self.dirty = true;
            Ok(())
        }
    }

    /// Changes the pipeline serialization format on a given pipeline.
    ///
    /// If the pipeline doesn't exist, an error is returned.
    pub fn set_pipeline_serialization(
        &mut self,
        name: impl Into<String>,
        sf: SerializationFormat,
    ) -> PipelineResult {
        let name = name.into();

        if let Some(pipeline) = self.pipelines.get_mut(&name) {
            if pipeline.serialization_format != sf {
                self.pipelines_to_remove.push(
                    Path::new(&pipeline.name)
                        .to_owned()
                        .with_extension(pipeline.serialization_format.file_ending()),
                );

                pipeline.serialization_format = sf;
                pipeline.dirty = true;
            }
            Ok(())
        } else {
            Err(PipelineError::PipelineDoesNotExist)
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
        let pipeline = self
            .pipelines
            .get_mut(&pipeline_name.into())
            .ok_or(PipelineError::PipelineDoesNotExist)?;

        let source_name = source_name.into();
        match pipeline.source_destinations.entry(source_name) {
            Entry::Vacant(e) => {
                e.insert(Default::default());
                pipeline.dirty = true;
                self.dirty = true;
                Ok(())
            }
            Entry::Occupied(_) => Err(PipelineError::PipelineSourceAlreadyExists),
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

    /// Removes a given **pipeline** from the manager. If any sources are on the pipeline,
    /// they will be lost permanently!
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

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, PartialOrd, Ord)]
struct PipelineManifest {
    pipelines: BTreeSet<PipelineDescriptor>,
    version: semver::Version,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, PartialOrd, Ord)]
struct PipelineDescriptor {
    path: PathBuf,
    serialization_format: SerializationFormat,
}

impl From<PathBuf> for PipelineDescriptor {
    fn from(path: PathBuf) -> Self {
        Self {
            path,
            serialization_format: SerializationFormat::default(),
        }
    }
}

pub type PipelineDesinations = BTreeMap<String, FilesystemPath>;
#[derive(Debug, Eq, Serialize, Deserialize, Clone, Default)]
pub struct Pipeline {
    pub name: String,
    pub source_destinations: BTreeMap<String, PipelineDesinations>,
    #[serde(default)]
    pub serialization_format: SerializationFormat,
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
        Some(self.cmp(other))
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

#[derive(Debug, Copy, Clone, thiserror::Error, PartialEq, Eq)]
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
        pipelines
            .add_pipeline("sprites", SerializationFormat::Json)
            .unwrap();
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

        pipelines
            .add_pipeline("sprites", SerializationFormat::Json)
            .unwrap();
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
            pipelines.add_pipeline("sprites", SerializationFormat::Json),
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
            |p| p.add_pipeline("sprites", SerializationFormat::Json),
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
        p.add_pipeline("s", SerializationFormat::Json).unwrap();
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
