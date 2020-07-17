use pretty_assertions::assert_eq;
use yy_boss::{
    yy_typings::sprite::{
        FrameId, Layer, LayerId, Sprite, SpriteKeyframe, SpriteSequenceId, Track,
    },
    SpriteExt,
};
mod common;

#[test]
fn add_sprite_to_yyp() {
    const IMAGE_PATH: &'static str = "tests/examples/test_spr_add.png";

    let mut yyp_boss = common::setup_blank_project().unwrap();
    assert!(
        yyp_boss.sprite_manager.get_sprite("spr_test").is_none(),
        "The sprite we're trying to add is already in the project!"
    );

    let new_view = yyp_boss
        .add_folder_to_end(&yyp_boss.root_path(), "Sprites".to_string())
        .unwrap();

    let single_frame_id = FrameId::with_string("1df0d96b-d607-46d8-ad4b-144ced21f501");
    let default_texture_group = yyp_boss.default_texture_path().unwrap();

    let sprite = Sprite::with_layer(
        "spr_test",
        default_texture_group,
        Layer {
            name: LayerId::with_string("17463651-1c81-4dea-a381-8f4a7635b32e"),
            ..Layer::default()
        },
    )
    .parent(new_view)
    .frame(single_frame_id)
    .with(|spr| {
        let track: &mut Track = &mut spr.sequence.tracks[0];
        let kf: &mut SpriteKeyframe = &mut track.keyframes.keyframes[0];
        kf.id = SpriteSequenceId::with_string("ab8911a2-4626-42b7-b1a2-3b8d23b6fd3b");
    })
    .bbox_mode(|_, _| yy_boss::BboxModeUtility::FullImage);

    let frame_buffer = image::open(IMAGE_PATH).unwrap().to_rgba();
    yyp_boss.add_sprite(sprite.clone(), vec![(single_frame_id, frame_buffer)]);

    assert!(
        yyp_boss.get_sprite("spr_test").is_some(),
        "We didn't add, or couldn't find, the sprite we just tried to add!"
    );
    assert_eq!(
        yyp_boss.get_sprite("spr_test").unwrap().clone(),
        sprite,
        "We mangled this sprite in the YypBoss!"
    );

    let proof_yyp_boss = common::load_proof("sprite_add_proof").unwrap();

    // // Assert the our YYPs are the Same...
    common::assert_yypboss_eq(&yyp_boss, &proof_yyp_boss);
}
