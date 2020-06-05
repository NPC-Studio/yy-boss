use anyhow::Result as AnyResult;
use pretty_assertions::assert_eq;
use std::path::Path;
use thiserror::Error;
use yy_boss::YypBoss;

const PATH_TO_TEST_PROJ: &'static str = "tests/examples/test_proj/test_proj.yyp";

pub fn setup_blank_project() -> AnyResult<YypBoss> {
    YypBoss::new(Path::new(PATH_TO_TEST_PROJ))
}

/// Loads a yyp boss by the name of the Proof. It must have a YYP of the same name
/// as the surrounding folder.
pub fn load_proof(proof_name: &str) -> AnyResult<YypBoss> {
    YypBoss::new(Path::new(&format!(
        "tests/examples/proofs/{0}/{0}.yyp",
        proof_name,
    )))
}

pub fn assert_yypboss_eq(ours: &YypBoss, proof: &YypBoss) {
    println!("target: {:#?}", ours.absolute_path());
    println!("proof: {:#?}", proof.absolute_path());

    match yypboss_neq(ours, proof) {
        Ok(()) => {}
        Err(neq) => match neq {
            YypBossNeq::Yyp => {
                let mut our_yyp = ours.yyp().clone();
                let proof_yyp = proof.yyp();
                // We have to equalize the names here to prevent trivial mismatches...
                our_yyp.name = proof_yyp.name.clone();

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
    match yypboss_neq(ours, proof) {
        Ok(()) => {
            panic!("Yyps were equal to each other!");
        }
        Err(_) => {}
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
