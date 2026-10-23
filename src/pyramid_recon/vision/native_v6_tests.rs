//! Original-image v6 calibration regressions; no complete scan or restoration proof.

use super::tests::{labelled_card, native_manifest, native_original, native_patch};
use super::*;
use serde_json::json;


/// Exact unchanged original supplying the two additional native RGB templates.
const SOURCE_SHA256: &str = "02b1f0f98c8a4c5b5d57b4075fceb2829d1175d8b80ef2455d94fc81fff1c988";


/// Resolve the separately retained v6 fixture without selecting an operator profile.
fn fixture_path(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pyramid-recon/native-v6")
        .join(name)
}


/// Decode the exact original PNG; mutations in individual tests stay in memory.
fn original() -> CapturedFrame {
    crate::capture::decode_png(
        &std::fs::read(fixture_path("baseline.png")).expect("v6 original PNG"),
    )
    .expect("native v6 PNG layout")
}


/// Load the real operator-selectable JSON through the production bounded loader.
fn vision() -> ReconVision {
    ReconVision::load(&fixture_path("profile.json")).expect("native v6 profile")
}


/// Read a mutable test copy; the retained calibration is never rewritten.
fn profile_value() -> Value {
    serde_json::from_slice(&std::fs::read(fixture_path("profile.json")).expect("v6 profile bytes"))
        .expect("v6 JSON")
}


/// Exercise the complete production byte parser for every derived test profile.
fn parse_profile(value: &Value) -> Result<ReconVision, String> {
    ReconVision::from_bytes(&serde_json::to_vec(value).expect("derived profile JSON"))
}


/// Select one labelled bank entry without depending on its array ordinal.
fn variants<'a>(reader: &'a mut Value, bank: &str, label: &str) -> &'a mut Vec<Value> {
    reader[bank]
        .as_array_mut()
        .expect("glyph bank")
        .iter_mut()
        .find(|entry| entry["label"] == label)
        .expect("labelled glyph entry")["variants"]
        .as_array_mut()
        .expect("glyph variants")
}


/// Paint a synthetic refusal derivative while preserving the original fixture bytes.
fn fill_region(frame: &mut CapturedFrame, bounds: PixelRect, colour: [u8; 3]) {
    for row in bounds.y..bounds.bottom() {
        for column in bounds.x..bounds.right() {
            let offset = row as usize * frame.stride + column as usize * 4;
            frame.pixels[offset..offset + 3].copy_from_slice(&colour);
        }
    }
}


/// The only v6 additions are two raw RGB crops; all reader limits stay unchanged.
#[test]
fn native_v6_adds_only_two_exact_native_variants_to_the_accepted_scene_masks() {
    let frame = original();
    let mut rollback = profile_value();
    assert_eq!(
        rollback["profile_id"],
        "pyramid-native-1920-classic-blue-v6"
    );
    assert_eq!(rollback["tableau_reader"], rollback["stock_reader"]);
    for (index, selected) in [(0, 2355), (1, 1728)] {
        assert_eq!(
            rollback["board"]["scene"][index]["samples"]["mask"]
                .as_str()
                .expect("accepted v5 mask")
                .bytes()
                .filter(|byte| *byte == b'1')
                .count(),
            selected
        );
    }

    for reader in ["tableau_reader", "stock_reader"] {
        assert_eq!(rollback[reader]["rank_max_distance"], 24);
        assert_eq!(rollback[reader]["suit_max_distance"], 8);
        assert_eq!(rollback[reader]["alignment"], 1);
        for (bank, label, bounds, count) in [
            ("ranks", "Q", PixelRect::new(1093, 331, 27, 24), 3),
            ("suits", "spades", PixelRect::new(895, 462, 20, 16), 9),
        ] {
            let measured = native_patch(&frame, bounds, SOURCE_SHA256);
            let mut expected = measured["samples"].clone();
            expected["tolerance"] = json!(0);
            let entries = variants(&mut rollback[reader], bank, label);
            assert_eq!(entries.len(), count);
            assert_eq!(entries.pop().expect("new measured variant"), expected);
        }
    }

    let old: Value = serde_json::from_slice(include_bytes!(
        "../../../tests/fixtures/pyramid-recon/native-profile.json"
    ))
    .expect("unchanged original v4 profile");
    rollback["profile_id"] = old["profile_id"].clone();
    for index in 0..2 {
        rollback["board"]["scene"][index]["samples"]["mask"] =
            old["board"]["scene"][index]["samples"]["mask"].clone();
    }
    assert_eq!(
        rollback, old,
        "no unrelated reader, scene, pile or dialog calibration changes"
    );
}


/// Direct visual labels cover this starting board and two piles, not all 52 cards.
#[test]
fn native_v6_observes_all_28_tableau_cards_left_ten_spades_and_positive_empty_right() {
    let frame = original();
    let vision = vision();
    let ReconScene::Board(board) = vision.observe(&frame).expect("current native board") else {
        panic!("the current starting frame must be a supported board");
    };
    let expected = [
        (Rank::Four, Suit::Hearts),
        (Rank::Jack, Suit::Diamonds),
        (Rank::Ace, Suit::Spades),
        (Rank::Queen, Suit::Clubs),
        (Rank::Seven, Suit::Clubs),
        (Rank::Nine, Suit::Diamonds),
        (Rank::Queen, Suit::Diamonds),
        (Rank::Three, Suit::Diamonds),
        (Rank::Eight, Suit::Spades),
        (Rank::Jack, Suit::Hearts),
        (Rank::Three, Suit::Hearts),
        (Rank::Nine, Suit::Hearts),
        (Rank::Ace, Suit::Diamonds),
        (Rank::Queen, Suit::Hearts),
        (Rank::Five, Suit::Clubs),
        (Rank::Two, Suit::Hearts),
        (Rank::King, Suit::Spades),
        (Rank::Ten, Suit::Diamonds),
        (Rank::King, Suit::Diamonds),
        (Rank::Jack, Suit::Spades),
        (Rank::Eight, Suit::Clubs),
        (Rank::Four, Suit::Spades),
        (Rank::Four, Suit::Clubs),
        (Rank::Four, Suit::Diamonds),
        (Rank::Queen, Suit::Spades),
        (Rank::Six, Suit::Spades),
        (Rank::Five, Suit::Spades),
        (Rank::Eight, Suit::Hearts),
    ]
    .map(|(rank, suit)| Card::new(rank, suit));
    assert_eq!(board.tableau, expected);
    assert_eq!(
        board.left,
        StockFace::Card(Card::new(Rank::Ten, Suit::Spades))
    );
    assert_eq!(board.right, StockFace::Empty);
    assert_eq!(board.undo_all, None);
    let view = frame.as_view().expect("native frame");
    assert!(
        vision
            .board
            .right_empty
            .matches(view, (0, 0))
            .expect("positive empty RIGHT")
    );
    assert_eq!(
        vision
            .stock_reader
            .recognise(view, (RIGHT_BOUNDS.x, RIGHT_BOUNDS.y))
            .expect("RIGHT independently has no unique card"),
        None
    );
}


/// One additional spade exemplar repairs two held-out slots at the existing caps.
#[test]
fn native_v6_repairs_the_three_actual_refusals_without_broadening_the_reader() {
    let frame = original();
    let view = frame.as_view().expect("native frame");
    let current = vision();
    let mut prior = profile_value();
    for reader in ["tableau_reader", "stock_reader"] {
        variants(&mut prior[reader], "ranks", "Q")
            .pop()
            .expect("additional Q");
        variants(&mut prior[reader], "suits", "spades")
            .pop()
            .expect("additional spade");
    }
    prior["profile_id"] = json!("pyramid-native-1920-classic-blue-v5");
    let prior =
        parse_profile(&prior).expect("accepted v5 policy reconstructed without the two additions");
    assert!(
        prior
            .observe(&frame)
            .expect_err("original refusal")
            .starts_with("unknown rank/suit at tableau slot 13;")
    );
    for (index, rank, suit) in [
        (13, Rank::Queen, Suit::Hearts),
        (16, Rank::King, Suit::Spades),
        (19, Rank::Jack, Suit::Spades),
    ] {
        let origin = TABLEAU_ORIGINS[index];
        assert_eq!(
            prior
                .tableau_reader
                .recognise(view, origin)
                .expect("prior unknown"),
            None
        );
        assert_eq!(
            current
                .tableau_reader
                .recognise(view, origin)
                .expect("current unique label"),
            Some(Card::new(rank, suit)),
            "logical slot {index}"
        );
    }
}


/// New evidence must preserve all 104 historical observations and 16 held-out LEFT cards.
#[test]
fn native_v6_preserves_the_complete_historical_glyph_corpus_in_both_readers() {
    let vision = vision();
    let manifest = native_manifest();
    let observations: Vec<_> = manifest["labelled_card_observations"]
        .as_array()
        .expect("historical labels")
        .iter()
        .chain(
            manifest["endpoint_card_observations"]
                .as_array()
                .expect("endpoint labels"),
        )
        .collect();
    let mut count = 0;
    let mut held_out_left = 0;
    for source in manifest["originals"].as_array().expect("originals") {
        let relevant: Vec<_> = observations
            .iter()
            .copied()
            .filter(|observation| observation["source_id"] == source["id"])
            .collect();
        if relevant.is_empty() {
            continue;
        }
        let source_id = source["id"].as_str().expect("source ID");
        let original = native_original(source_id);
        let view = original.as_view().expect("native view");
        for observation in relevant {
            let origin = (
                integer(&observation["origin"][0]).expect("x"),
                integer(&observation["origin"][1]).expect("y"),
            );
            for reader in [&vision.tableau_reader, &vision.stock_reader] {
                assert_eq!(
                    reader
                        .recognise(view, origin)
                        .expect("unique historical label"),
                    Some(labelled_card(observation)),
                    "{}",
                    observation["id"]
                );
            }
            count += 1;
            held_out_left += usize::from(
                source_id.starts_with("issue1-")
                    && observation["template_training"] == json!(false),
            );
        }
    }
    assert_eq!(count, 104);
    assert_eq!(held_out_left, 16);
}


/// Nine synthetic derivatives retain erasure and contradictory-colour refusal.
#[test]
fn native_v6_does_not_fill_erased_or_mixed_colour_rank_and_suit_evidence() {
    let vision = vision();
    let original = original();
    for index in [13, 16, 19] {
        let origin = TABLEAU_ORIGINS[index];
        for relative in [PixelRect::new(1, 5, 30, 26), PixelRect::new(4, 29, 22, 20)] {
            let mut erased = original.clone();
            fill_region(
                &mut erased,
                PixelRect::new(
                    origin.0 + relative.x,
                    origin.1 + relative.y,
                    relative.width,
                    relative.height,
                ),
                [255; 3],
            );
            for reader in [&vision.tableau_reader, &vision.stock_reader] {
                assert_eq!(
                    reader
                        .recognise(erased.as_view().expect("erased derivative"), origin)
                        .expect("missing glyph"),
                    None
                );
            }
            assert!(
                vision
                    .observe(&erased)
                    .expect_err("erased evidence refuses")
                    .starts_with(&format!("unknown rank/suit at tableau slot {index};"))
            );
        }
        let mut mixed = original.clone();
        for row in origin.1 + 5..origin.1 + 31 {
            for column in origin.0 + 1..origin.0 + 31 {
                let offset = row as usize * mixed.stride + column as usize * 4;
                let pixel = &mut mixed.pixels[offset..offset + 3];
                if pixel.iter().any(|channel| *channel < 128) {
                    pixel.copy_from_slice(if index == 13 {
                        &[0, 0, 0]
                    } else {
                        &[220, 0, 0]
                    });
                }
            }
        }
        for reader in [&vision.tableau_reader, &vision.stock_reader] {
            assert_eq!(
                reader
                    .recognise(mixed.as_view().expect("mixed derivative"), origin)
                    .expect("contradictory colours"),
                None
            );
        }
        assert!(
            vision
                .observe(&mixed)
                .expect_err("mixed evidence refuses")
                .starts_with(&format!("unknown rank/suit at tableau slot {index};"))
        );
    }
}


/// Adding exemplars never bypasses the fixed XOR, alignment or variant-count caps.
#[test]
fn native_v6_retains_the_existing_parser_limits() {
    for reader in ["tableau_reader", "stock_reader"] {
        for (field, invalid) in [
            ("rank_max_distance", 25),
            ("suit_max_distance", 9),
            ("alignment", 2),
        ] {
            let mut value = profile_value();
            value[reader][field] = json!(invalid);
            assert!(
                parse_profile(&value)
                    .expect_err("unchanged distance/alignment cap")
                    .contains("glyph mismatch/alignment limits")
            );
        }
        for (bank, label, excessive) in [("ranks", "Q", 9), ("suits", "spades", 17)] {
            let mut value = profile_value();
            let entries = variants(&mut value[reader], bank, label);
            let exemplar = entries[0].clone();
            entries.resize(excessive, exemplar);
            assert!(
                parse_profile(&value)
                    .expect_err("unchanged per-label variant cap")
                    .contains("glyph variant count exceeds its bound")
            );
        }
    }
}


/// Exact duplicate occupancy refuses parsing; near duplicates still refuse at runtime.
#[test]
fn native_v6_rejects_cross_label_ambiguity_in_added_glyph_evidence() {
    let mut value = profile_value();
    let mut duplicate = variants(&mut value["tableau_reader"], "ranks", "Q")
        .last()
        .expect("new Q")
        .clone();
    *variants(&mut value["tableau_reader"], "ranks", "K") = vec![duplicate.clone()];
    assert!(
        parse_profile(&value)
            .expect_err("duplicate occupancy refuses")
            .contains("different glyph labels have identical occupancy")
    );

    // A synthetic one-pixel occupancy change remains within both matching caps.
    let mut rgb = duplicate["rgb"].as_str().expect("RGB samples").to_owned();
    let first_ink = (0..3).any(|channel| {
        u8::from_str_radix(&rgb[channel * 2..channel * 2 + 2], 16).expect("RGB channel") < 128
    });
    rgb.replace_range(0..6, if first_ink { "ffffff" } else { "000000" });
    duplicate["rgb"] = json!(rgb);
    *variants(&mut value["tableau_reader"], "ranks", "K") = vec![duplicate];
    let ambiguous =
        parse_profile(&value).expect("nonidentical templates require runtime uniqueness");
    let frame = original();
    assert!(
        ambiguous
            .tableau_reader
            .recognise(frame.as_view().expect("native view"), TABLEAU_ORIGINS[13])
            .expect_err("two current rank labels refuse")
            .contains("ambiguous glyph matches")
    );
    assert!(ambiguous.observe(&frame).is_err());
}


/// Supported cards cannot replace missing scene or positive-empty evidence.
#[test]
fn native_v6_retains_scene_and_positive_empty_pile_requirements() {
    let vision = vision();
    let original = original();
    for pattern in &vision.board.scene {
        let mut changed = original.clone();
        fill_region(&mut changed, pattern.bounds, [17, 19, 21]);
        assert!(
            !vision
                .board
                .matches(changed.as_view().expect("changed scene"))
                .expect("scene observation")
        );
        assert_eq!(
            vision.observe(&changed).expect("unsupported scene"),
            ReconScene::Unknown
        );
    }
    let mut missing_empty = original;
    fill_region(
        &mut missing_empty,
        vision.board.right_empty.bounds,
        [17, 19, 21],
    );
    assert!(
        vision
            .observe(&missing_empty)
            .expect_err("missing positive-empty RIGHT refuses")
            .starts_with("RIGHT:")
    );
}


/// Solver toolbar pixels do not change supported card identities or scene evidence.
#[test]
fn native_v6_accepts_changed_solver_toolbar_without_recalibration() {
    let vision = vision();
    let original = original();
    let expected = vision.observe(&original).expect("supported v6 board");
    assert!(matches!(expected, ReconScene::Board(_)));
    for colour in [[0, 0, 0], [255, 255, 255], [128, 128, 128], [255, 210, 0]] {
        let mut changed = original.clone();
        fill_region(&mut changed, SOLVER_BOUNDS, colour);
        assert_eq!(
            vision
                .observe(&changed)
                .expect("toolbar has no scene authority"),
            expected
        );
    }
}
