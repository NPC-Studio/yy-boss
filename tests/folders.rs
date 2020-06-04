mod common;

#[test]
fn folder_add_root() {
    let mut basic_yyp_boss = common::setup_blank_project().unwrap();
    let proof = common::load_proof("folder_add_root").unwrap();

    common::assert_yypboss_neq(&basic_yyp_boss, &proof);

    basic_yyp_boss
        .add_folder_to_end(&basic_yyp_boss.root_path(), "Test At Root".to_string())
        .unwrap();

    common::assert_yypboss_eq(&basic_yyp_boss, &proof);
}

#[test]
fn folder_add_nonroot() {
    let mut basic_yyp_boss = common::setup_blank_project().unwrap();
    let proof = common::load_proof("folder_add_nonroot").unwrap();

    common::assert_yypboss_neq(&basic_yyp_boss, &proof);

    let parent_folder = basic_yyp_boss
        .add_folder_to_end(&basic_yyp_boss.root_path(), "First Folder".to_string())
        .unwrap();

    basic_yyp_boss
        .add_folder_to_end(&parent_folder, "Subfolder".to_string())
        .unwrap();

    common::assert_yypboss_eq(&basic_yyp_boss, &proof);
}

#[test]
fn delete_folder_recursively() {}

// STARTING UP AT 9:55 -- DOING RUST + GAMEMAKER WORK
