use super::{utils, FilesystemPath};
use anyhow::Result as AnyResult;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct Pipelines(BTreeMap<String, Pipeline>);

impl Pipelines {
    const PIPELINE_MANIFEST: &'static str = "pipeline_manifest.json";

    pub fn new(boss_dir: &Path) -> AnyResult<Pipelines> {
        let pipeline_manifest_path = boss_dir.join(Self::PIPELINE_MANIFEST);

        // If there's no pipeline manifest file, then no worries,
        // just return. Users might not want to make a manifest!
        if pipeline_manifest_path.exists() == false {
            println!(
                "No pipeline manifest found at path {:#?}...",
                pipeline_manifest_path
            );
            Ok(Self(BTreeMap::new()))
        } else {
            let pipeline_manifest: BTreeSet<PathBuf> =
                utils::deserialize(&pipeline_manifest_path, None)?;

            let mut pipelines = BTreeMap::new();

            let found_paths = pipeline_manifest
                .iter()
                .filter(|&path| {
                    let mut joint_path = boss_dir.join(path);
                    joint_path.set_extension("json");

                    if joint_path.exists() {
                        match utils::deserialize(&joint_path, None) {
                            Ok(datum) => {
                                pipelines.insert(path.to_string_lossy().to_string(), datum);
                            }
                            Err(e) => log::error!(
                                "problem reading {:#?}, even though it was in manifest: {:}.",
                                path,
                                e
                            ),
                        }
                        true
                    } else {
                        false
                    }
                })
                .collect::<BTreeSet<_>>();

            // If the found paths are not right, then don't do the thing!
            if found_paths.len() != pipeline_manifest.len() {
                log::error!("pipeline manifest had invalid entries. removing and reserializing valid entires...");
                match utils::serialize(&pipeline_manifest_path, &found_paths) {
                    Ok(_) => {
                        log::error!("done");
                    }
                    Err(e) => {
                        log::error!("failed to save updated manifest: {:}", e);
                    }
                }
            }

            Ok(Self(pipelines))
        }
    }

    pub fn pipeline(&self, name: &str) -> Option<&Pipeline> {
        self.0.get(name)
    }
}

#[derive(Debug, Eq, Serialize, Deserialize, Hash)]
pub struct Pipeline {
    pub name: String,
    pub data: BTreeMap<String, BTreeSet<FilesystemPath>>,
}

impl PartialEq for Pipeline {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
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
