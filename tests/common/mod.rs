use pretty_assertions::assert_eq;
use std::{
    fs,
    path::{Path, PathBuf},
};
use thiserror::Error;
use uuid::Uuid;
use yy_boss::{StartupError, YypBoss};

#[allow(dead_code)]
const PATH_TO_TEST_PROJ: &str = "tests/examples/test_proj";

#[allow(dead_code)]
const TEST_PROJECT_NAME: &str = "test_proj.yyp";

#[allow(dead_code)]
const WORKING_DIR: &str = "tests/working_dir";

#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd)]
#[must_use = "clean up the resulting directories"]
pub struct CleanUpToken(Uuid);

impl CleanUpToken {
    #[allow(dead_code)]
    pub fn dispose(self) {
        std::fs::remove_dir_all(Path::new(WORKING_DIR).join(self.0.to_string())).unwrap();
    }
}

#[allow(dead_code)]
pub fn setup_blank_project() -> (YypBoss, CleanUpToken) {
    let subdir = Uuid::new_v4();
    let path = Path::new(WORKING_DIR).join(subdir.to_string());
    let token = CleanUpToken(subdir);

    println!("Working Dir is {}", subdir);

    copy_dir_r(PATH_TO_TEST_PROJ, &path).unwrap();

    (YypBoss::new(path.join(TEST_PROJECT_NAME)).unwrap(), token)
}

/// Loads a yyp boss by the name of the Proof. It must have a YYP of the same name
/// as the surrounding folder.
#[allow(dead_code)]
pub fn load_proof(proof_name: &str) -> Result<YypBoss, StartupError> {
    YypBoss::new(Path::new(&format!(
        "tests/examples/proofs/{0}/{0}.yyp",
        proof_name,
    )))
}

#[allow(dead_code)]
pub fn assert_yypboss_eq(ours: &YypBoss, proof: &YypBoss) {
    println!("target: {:#?}", ours.directory_manager.yyp());
    println!("proof: {:#?}", proof.directory_manager.yyp());

    match yypboss_neq(ours, proof) {
        Ok(()) => {}
        Err(neq) => match neq {
            YypBossNeq::Yyp => {
                let mut our_yyp = ours.yyp().clone();
                let proof_yyp = proof.yyp();
                // We have to equalize the names here to prevent trivial mismatches...
                our_yyp.name = proof_yyp.name.clone();

                // and we have to normalize orders
                for yyp_resource in proof_yyp.folders.iter() {
                    if let Some(ours) = our_yyp
                        .folders
                        .iter_mut()
                        .find(|f| f.folder_path == yyp_resource.folder_path)
                    {
                        ours.order = yyp_resource.order;
                    } else {
                        println!("Couldn't find {} in our Yyp...", yyp_resource.name);
                    }
                }

                assert_eq!(
                    our_yyp, *proof_yyp,
                    "target yyp and proof yyp were not equal"
                );
            }
            YypBossNeq::FolderGraph => {
                println!("target: {:#?}", ours.vfs.get_root_folder());
                println!("proof: {:#?}", proof.vfs.get_root_folder());
                panic!("target folder graph and proof folder graph were not equal");
            }
            YypBossNeq::ResourceNames => {
                assert_eq!(
                    ours.vfs.resource_names.inner(),
                    proof.vfs.resource_names.inner(),
                    "target resource names and proof resource names were not equal"
                );
            }
        },
    }
}

#[allow(dead_code)]
pub fn assert_yypboss_neq(ours: &YypBoss, proof: &YypBoss) {
    if yypboss_neq(ours, proof).is_ok() {
        panic!("Yyps were equal to each other!");
    }
}

fn yypboss_neq(ours: &YypBoss, proof: &YypBoss) -> Result<(), YypBossNeq> {
    // Assert the our YYPs are the Same...
    let mut our_yyp = ours.yyp().clone();
    let proof_yyp = proof.yyp();
    our_yyp.name = proof_yyp.name.clone();

    // Assert our Yyps are the Same
    if our_yyp != *proof_yyp {
        return Err(YypBossNeq::Yyp);
    }

    // Assert our Folder Graphs are the same...
    if ours.vfs.get_root_folder() != proof.vfs.get_root_folder() {
        return Err(YypBossNeq::FolderGraph);
    }

    // Assert our Current Resource names are the same...
    if ours.vfs.resource_names.inner() != proof.vfs.resource_names.inner() {
        return Err(YypBossNeq::ResourceNames);
    }

    Ok(())
}

#[derive(Debug, Error)]
enum YypBossNeq {
    #[error("the yyps are not the same")]
    Yyp,
    #[error("the folder graphs are not the same")]
    FolderGraph,
    #[error("the resource names are not the same")]
    ResourceNames,
}

fn copy_dir_r<U: AsRef<Path>, V: AsRef<Path>>(from: U, to: V) -> Result<(), std::io::Error> {
    let mut stack = vec![PathBuf::from(from.as_ref())];

    let output_root = PathBuf::from(to.as_ref());
    let input_root = PathBuf::from(from.as_ref()).components().count();

    while let Some(working_path) = stack.pop() {
        // Generate a relative path
        let src: PathBuf = working_path.components().skip(input_root).collect();

        // Create a destination if missing
        let dest = if src.components().count() == 0 {
            output_root.clone()
        } else {
            output_root.join(&src)
        };
        if fs::metadata(&dest).is_err() {
            fs::create_dir_all(&dest)?;
        }

        for entry in fs::read_dir(working_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                match path.file_name() {
                    Some(filename) => {
                        let dest_path = dest.join(filename);
                        fs::copy(&path, &dest_path)?;
                    }
                    None => {
                        println!("failed: {:?}", path);
                    }
                }
            }
        }
    }

    Ok(())
}
