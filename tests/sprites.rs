use console::Style;
use pretty_assertions::assert_eq;
use std::path::Path;
use yy_boss::{
    yy_typings::sprite::{
        FrameId, Layer, LayerId, Sprite, SpriteKeyframe, SpriteSequenceId, Track,
    },
    SpriteExt, YypBoss,
};

const PATH_TO_TEST_PROJ: &'static str = "tests/examples/test_proj/test_proj.yyp";

#[test]
fn add_sprite_to_yyp() {
    const IMAGE_PATH: &'static str = "tests/examples/test_spr_add.png";
    const PROOF_PATH: &'static str = "tests/examples/proofs/sprite_add_proof/test_proj.yyp";

    let mut yyp_boss = YypBoss::new(Path::new(PATH_TO_TEST_PROJ)).unwrap();
    let new_view = yyp_boss
        .add_folder_to_end(yyp_boss.root_path(), "Sprites".to_string())
        .unwrap();

    let single_frame_id = FrameId::with_string("1df0d96b-d607-46d8-ad4b-144ced21f501");
    let sprite = Sprite::with_layer(
        "spr_test",
        "Default",
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
    yyp_boss.add_sprite(sprite, vec![(single_frame_id, frame_buffer)]);

    let proof_yyp_boss = YypBoss::new(Path::new(PROOF_PATH)).unwrap();

    // Assert the our YYPs are the Same...
    let our_yyp = yyp_boss.yyp();
    let proof_yyp = proof_yyp_boss.yyp();
    if our_yyp != proof_yyp {
        let red = Style::new().red();
        let green = Style::new().green();

        let mut panic_string = String::with_capacity(100);

        if our_yyp.resources != proof_yyp.resources {
            panic_string.push_str("Yyp Resources do not Match!\n");
            let missing = proof_yyp.resources.difference(&our_yyp.resources);
            if missing.clone().count() > 0 {
                panic_string.push_str(&format!(
                    "Missing: \n\
                {:#?}\n",
                    red.apply_to(missing)
                ))
            }

            let extra = our_yyp.resources.difference(&proof_yyp.resources);
            if extra.clone().count() > 0 {
                panic_string.push_str(&format!(
                    "Additional: \n\
                {:#?}\n",
                    green.apply_to(extra)
                ))
            }

            panic_string.push('\n');
        }

        if our_yyp.folders != proof_yyp.folders {
            panic_string.push_str(&format!(
                "Yyp was Missing Proof's Folders... Missing Folders: \n\
            {:#?}\n\n",
                proof_yyp.folders.difference(&our_yyp.folders)
            ));
        }

        if panic_string.is_empty() == false {
            panic!("\n{}", panic_string);
        }

        assert_eq!(our_yyp, proof_yyp);
    }
}
