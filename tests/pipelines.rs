mod common;

#[test]
fn sanity_check() {
    let proof = common::load_proof("pipelines_proof").unwrap();

    println!("{:#?}", proof.pipeline_manager);
}
