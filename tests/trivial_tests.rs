use include_dir::{include_dir, Dir, DirEntry};
use pretty_assertions::assert_eq;
use std::path::Path;
use yy_boss::{
    yy_typings::utils::TrailingCommaUtility, yy_typings::Yyp, YypBoss, YypSerialization,
};

/// The purpose of this test is to make sure that the YypBoss can
/// take in YYPs without breaking those YYPs.
#[test]
fn no_mangle_yyp() {
    // We cannot save this string into a constant, due to issues with the
    // `include_dir` macro.
    let root_path = Path::new("tests/examples/project_database");
    let all_yyps: Dir = include_dir!("tests/examples/project_database");
    let tcu = TrailingCommaUtility::new();

    for yyps in all_yyps.find("**/*.yyp").unwrap() {
        if let DirEntry::File(file) = yyps {
            println!("{}", file.path);
            let path = root_path.join(file.path);
            let parsed_yyp = YypBoss::new(&path).unwrap().yyp().clone();

            let original = std::str::from_utf8(file.contents).unwrap();
            let original_pure_parsed_yyp: Yyp =
                serde_json::from_str(&tcu.clear_trailing_comma(original)).unwrap();

            assert_eq!(original_pure_parsed_yyp, parsed_yyp);
            assert_eq!(original, parsed_yyp.yyp_serialization(0));
        }
    }
}
