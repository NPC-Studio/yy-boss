use yy_boss::YypBoss;

mod common;

#[test]
fn sanity_check() {
    let proof = common::load_proof("yyp_boss_sanity").unwrap();

    panic!("Scheduled Panic...");
}
