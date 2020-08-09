use pretty_assertions::assert_eq;
use std::path::Path;
use thiserror::Error;
use yy_boss::{errors::StartupError, YypBoss};

#[allow(dead_code)]
const PATH_TO_TEST_PROJ: &str = "tests/examples/test_proj/test_proj.yyp";

#[allow(dead_code)]
pub fn setup_blank_project() -> Result<YypBoss, StartupError> {
    YypBoss::new(Path::new(PATH_TO_TEST_PROJ))
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
                println!("target: {:#?}", ours.root_folder());
                println!("proof: {:#?}", proof.root_folder());
                panic!("target folder graph and proof folder graph were not equal");
            }
            YypBossNeq::ResourceNames => {
                assert_eq!(
                    ours.current_resource_names(),
                    proof.current_resource_names(),
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
    if ours.root_folder() != proof.root_folder() {
        return Err(YypBossNeq::FolderGraph);
    }
    assert_eq!(ours.root_folder(), proof.root_folder());

    // Assert our Current Resource names are the same...
    if ours.current_resource_names() != proof.current_resource_names() {
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
