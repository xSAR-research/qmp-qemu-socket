//! Native v7 calibration regressions over retained original images.
//!
//! Added glyph evidence is checked independently from held-out card recognition.
//! Synthetic derivatives test refusal only; none is scan or restoration evidence.

use super::tests::{labelled_card, native_manifest, native_original, native_patch};
use super::*;
use serde_json::json;


/// Original PNG identity recorded by the independent fixture provenance check.
const SOURCE_SHA256: &str = "f00076f38a2906dc17d1dc5b7c54806b6285f363f936924e5c785785e9173edf";


/// Resolve one retained fixture without changing the operator-selected profile.
fn fixture_path(generation: &str, name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pyramid-recon")
        .join(generation)
        .join(name)
}


/// Decode unchanged native PNG bytes; all refusal derivatives stay in memory.
fn original(generation: &str) -> CapturedFrame {
    crate::capture::decode_png(
        &std::fs::read(fixture_path(generation, "baseline.png")).expect("retained original PNG"),
    )
    .expect("native PNG layout")
}


/// Load the actual v7 calibration through the production bounded file loader.
fn vision() -> ReconVision {
    ReconVision::load(&fixture_path("native-v7", "profile.json")).expect("native v7 profile")
}


/// Read a mutable JSON copy while preserving each original profile on disk.
fn profile_value(generation: &str) -> Value {
    serde_json::from_slice(
        &std::fs::read(fixture_path(generation, "profile.json")).expect("retained profile bytes"),
    )
    .expect("native profile JSON")
}


/// Exercise production JSON and calibration validation for synthetic profiles.
fn parse_profile(value: &Value) -> Result<ReconVision, String> {
    ReconVision::from_bytes(&serde_json::to_vec(value).expect("derived profile JSON"))
}


/// Select a labelled bank independently of label ordering in the JSON array.
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


/// Paint one synthetic refusal region without changing alpha or fixture bytes.
fn fill_region(frame: &mut CapturedFrame, bounds: PixelRect, colour: [u8; 3]) {
    for row in bounds.y..bounds.bottom() {
        for column in bounds.x..bounds.right() {
            let offset = row as usize * frame.stride + column as usize * 4;
            frame.pixels[offset..offset + 3].copy_from_slice(&colour);
        }
    }
}


/// Independently reviewed labels for the new image in fixed logical row order.
fn new_tableau() -> [Card; 28] {
    [
        (Rank::Nine, Suit::Clubs),
        (Rank::King, Suit::Clubs),
        (Rank::Jack, Suit::Spades),
        (Rank::Seven, Suit::Clubs),
        (Rank::Eight, Suit::Clubs),
        (Rank::Four, Suit::Diamonds),
        (Rank::Two, Suit::Clubs),
        (Rank::Two, Suit::Spades),
        (Rank::Queen, Suit::Diamonds),
        (Rank::Six, Suit::Spades),
        (Rank::Ace, Suit::Hearts),
        (Rank::King, Suit::Hearts),
        (Rank::Ace, Suit::Diamonds),
        (Rank::Nine, Suit::Spades),
        (Rank::Six, Suit::Diamonds),
        (Rank::Two, Suit::Hearts),
        (Rank::Eight, Suit::Diamonds),
        (Rank::Seven, Suit::Hearts),
        (Rank::Queen, Suit::Spades),
        (Rank::Jack, Suit::Diamonds),
        (Rank::Seven, Suit::Spades),
        (Rank::Queen, Suit::Hearts),
        (Rank::Seven, Suit::Diamonds),
        (Rank::Five, Suit::Spades),
        (Rank::Six, Suit::Clubs),
        (Rank::Three, Suit::Spades),
        (Rank::Ace, Suit::Spades),
        (Rank::Queen, Suit::Clubs),
    ]
    .map(|(rank, suit)| Card::new(rank, suit))
}


/// Retained visual labels for the v6 original, independent of either reader.
fn prior_tableau() -> [Card; 28] {
    [
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
    .map(|(rank, suit)| Card::new(rank, suit))
}


/// Only two raw crops are appended per reader; removing them restores exact v6.
#[test]
fn native_v7_additions_are_exact_native_crops_and_the_whole_profile_rolls_back() {
    let frame = original("native-v7");
    let mut rollback = profile_value("native-v7");
    let mut prior = profile_value("native-v6");
    assert_eq!(
        rollback["profile_id"],
        "pyramid-native-1920-classic-blue-v7"
    );
    assert_eq!(rollback["tableau_reader"], rollback["stock_reader"]);
    for reader in ["tableau_reader", "stock_reader"] {
        assert_eq!(rollback[reader]["rank_max_distance"], 24);
        assert_eq!(rollback[reader]["suit_max_distance"], 8);
        assert_eq!(rollback[reader]["alignment"], 1);
        for (bank, label, bounds, maximum) in [
            ("ranks", "2", PixelRect::new(792, 278, 27, 24), 8),
            ("suits", "spades", PixelRect::new(1096, 462, 20, 16), 16),
        ] {
            let measured = native_patch(&frame, bounds, SOURCE_SHA256);
            let mut expected = measured["samples"].clone();
            expected["tolerance"] = json!(0);
            let before = variants(&mut prior[reader], bank, label).len();
            let entries = variants(&mut rollback[reader], bank, label);
            assert_eq!(entries.len(), before + 1);
            assert!(entries.len() <= maximum);
            assert_eq!(entries.pop().expect("appended raw sample"), expected);
        }
    }
    rollback["profile_id"] = prior["profile_id"].clone();
    assert_eq!(
        rollback, prior,
        "all earlier samples, limits, scene, Solver, piles and dialog must be unchanged"
    );
}


/// Direct labels cover all tableau slots and initial piles, without complete stock claims.
#[test]
fn native_v7_observes_the_new_tableau_left_four_spades_and_positive_empty_right() {
    let frame = original("native-v7");
    let vision = vision();
    let ReconScene::Board(board) = vision.observe(&frame).expect("new supported native board")
    else {
        panic!("the new starting image must be a supported board");
    };
    assert_eq!(board.tableau, new_tableau());
    assert_eq!(
        board.left,
        StockFace::Card(Card::new(Rank::Four, Suit::Spades))
    );
    assert_eq!(board.right, StockFace::Empty);
    assert_eq!(board.undo_all, None);
    let view = frame.as_view().expect("native view");
    for (index, card) in new_tableau().into_iter().enumerate() {
        for reader in [&vision.tableau_reader, &vision.stock_reader] {
            assert_eq!(
                reader
                    .recognise(view, TABLEAU_ORIGINS[index])
                    .expect("unique new card"),
                Some(card),
                "logical slot {index}"
            );
        }
    }
    for reader in [&vision.tableau_reader, &vision.stock_reader] {
        assert_eq!(
            reader
                .recognise(view, (LEFT_BOUNDS.x, LEFT_BOUNDS.y))
                .expect("initial LEFT"),
            Some(Card::new(Rank::Four, Suit::Spades))
        );
        assert_eq!(
            reader
                .recognise(view, (RIGHT_BOUNDS.x, RIGHT_BOUNDS.y))
                .expect("no RIGHT card"),
            None
        );
    }
    assert!(
        vision
            .board
            .right_empty
            .matches(view, (0, 0))
            .expect("positive empty RIGHT")
    );
}


/// Added evidence comes from recognised sources; separate held-out slots reproduce repair.
#[test]
fn native_v7_repairs_two_held_out_slots_from_two_already_recognised_source_cards() {
    let frame = original("native-v7");
    let view = frame.as_view().expect("native view");
    let prior = parse_profile(&profile_value("native-v6")).expect("unchanged v6 calibration");
    let current = vision();
    assert!(
        prior
            .observe(&frame)
            .expect_err("retained first refusal")
            .starts_with("unknown rank/suit at tableau slot 6;")
    );
    for index in [7, 25] {
        for reader in [&prior.tableau_reader, &prior.stock_reader] {
            assert_eq!(
                reader
                    .recognise(view, TABLEAU_ORIGINS[index])
                    .expect("recognised source"),
                Some(new_tableau()[index]),
                "source slot {index} must already recognise before training"
            );
        }
    }
    for index in [6, 23] {
        for reader in [&prior.tableau_reader, &prior.stock_reader] {
            assert_eq!(
                reader
                    .recognise(view, TABLEAU_ORIGINS[index])
                    .expect("prior refusal"),
                None,
                "held-out slot {index}"
            );
        }
        for reader in [&current.tableau_reader, &current.stock_reader] {
            assert_eq!(
                reader
                    .recognise(view, TABLEAU_ORIGINS[index])
                    .expect("repaired held-out card"),
                Some(new_tableau()[index]),
                "held-out slot {index}"
            );
        }
    }
}


/// v7 must preserve all 104 independently labelled historical observations in both banks.
#[test]
fn native_v7_preserves_the_complete_historical_glyph_corpus_in_both_readers() {
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
                        .expect("unique historical identity"),
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


/// The separately retained v6 image contributes another 29 actual card observations.
#[test]
fn native_v7_preserves_all_v6_original_cards_in_both_readers() {
    let vision = vision();
    let frame = original("native-v6");
    let view = frame.as_view().expect("native v6 view");
    let mut count = 0;
    for (index, card) in prior_tableau().into_iter().enumerate() {
        for reader in [&vision.tableau_reader, &vision.stock_reader] {
            assert_eq!(
                reader
                    .recognise(view, TABLEAU_ORIGINS[index])
                    .expect("retained v6 identity"),
                Some(card),
                "v6 logical slot {index}"
            );
        }
        count += 1;
    }
    for reader in [&vision.tableau_reader, &vision.stock_reader] {
        assert_eq!(
            reader
                .recognise(view, (LEFT_BOUNDS.x, LEFT_BOUNDS.y))
                .expect("retained v6 LEFT"),
            Some(Card::new(Rank::Ten, Suit::Spades))
        );
    }
    count += 1;
    assert_eq!(count, 29);
    let ReconScene::Board(board) = vision.observe(&frame).expect("retained v6 board") else {
        panic!("the v6 original must remain a supported board");
    };
    assert_eq!(board.tableau, prior_tableau());
    assert_eq!(
        board.left,
        StockFace::Card(Card::new(Rank::Ten, Suit::Spades))
    );
    assert_eq!(board.right, StockFace::Empty);
}


/// Missing, contradictory-colour and unknown glyph pixels never become inferred cards.
#[test]
fn native_v7_refuses_erased_mixed_colour_and_unknown_held_out_evidence() {
    let vision = vision();
    let original = original("native-v7");
    for index in [6, 23] {
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
        let mut unknown = original.clone();
        for row in origin.1 + 5..origin.1 + 31 {
            for column in origin.0 + 1..origin.0 + 31 {
                let offset = row as usize * mixed.stride + column as usize * 4;
                let pixel = &mut mixed.pixels[offset..offset + 3];
                if pixel.iter().any(|channel| *channel < 128) {
                    pixel.copy_from_slice(&[220, 0, 0]);
                }
                unknown.pixels[offset..offset + 3].copy_from_slice(
                    if (row + column).is_multiple_of(2) {
                        &[0; 3]
                    } else {
                        &[255; 3]
                    },
                );
            }
        }
        for derivative in [&mixed, &unknown] {
            for reader in [&vision.tableau_reader, &vision.stock_reader] {
                assert_eq!(
                    reader
                        .recognise(derivative.as_view().expect("refusal derivative"), origin)
                        .expect("contradictory or unknown glyph"),
                    None
                );
            }
            assert!(
                vision
                    .observe(derivative)
                    .expect_err("invalid card evidence refuses")
                    .starts_with(&format!("unknown rank/suit at tableau slot {index};"))
            );
        }
    }
}


/// New samples do not change mismatch, alignment, per-label or complete-bank limits.
#[test]
fn native_v7_retains_the_production_parser_limits() {
    for reader in ["tableau_reader", "stock_reader"] {
        for (field, invalid) in [
            ("rank_max_distance", 25),
            ("suit_max_distance", 9),
            ("alignment", 2),
        ] {
            let mut value = profile_value("native-v7");
            value[reader][field] = json!(invalid);
            assert!(
                parse_profile(&value)
                    .expect_err("unchanged mismatch/alignment cap")
                    .contains("glyph mismatch/alignment limits")
            );
        }
        for (bank, label, excessive) in [("ranks", "2", 9), ("suits", "spades", 17)] {
            let mut value = profile_value("native-v7");
            let entries = variants(&mut value[reader], bank, label);
            let exemplar = entries[0].clone();
            entries.resize(excessive, exemplar);
            assert!(
                parse_profile(&value)
                    .expect_err("unchanged per-label variant cap")
                    .contains("glyph variant count exceeds its bound")
            );
        }
        for bank in ["ranks", "suits"] {
            let mut incomplete = profile_value("native-v7");
            incomplete[reader][bank]
                .as_array_mut()
                .expect("glyph bank")
                .pop()
                .expect("remove one required label");
            assert!(
                parse_profile(&incomplete)
                    .expect_err("complete labels remain required")
                    .contains("glyph bank must cover every rank or suit")
            );
        }
    }
}


/// Exact and near cross-label duplicates still refuse for either newly extended bank.
#[test]
fn native_v7_rejects_exact_and_runtime_ambiguity_for_both_added_glyphs() {
    let frame = original("native-v7");
    let view = frame.as_view().expect("native view");
    for reader_name in ["tableau_reader", "stock_reader"] {
        for (bank, label, other_label, source) in
            [("ranks", "2", "K", 7), ("suits", "spades", "hearts", 25)]
        {
            let mut value = profile_value("native-v7");
            let mut duplicate = variants(&mut value[reader_name], bank, label)
                .last()
                .expect("appended native variant")
                .clone();
            *variants(&mut value[reader_name], bank, other_label) = vec![duplicate.clone()];
            assert!(
                parse_profile(&value)
                    .expect_err("exact cross-label occupancy refuses")
                    .contains("different glyph labels have identical occupancy")
            );

            // One changed occupancy pixel is nonidentical but within both XOR caps.
            let mut rgb = duplicate["rgb"].as_str().expect("RGB samples").to_owned();
            let first_ink = (0..3).any(|channel| {
                u8::from_str_radix(&rgb[channel * 2..channel * 2 + 2], 16).expect("RGB channel")
                    < 128
            });
            rgb.replace_range(0..6, if first_ink { "ffffff" } else { "000000" });
            duplicate["rgb"] = json!(rgb);
            *variants(&mut value[reader_name], bank, other_label) = vec![duplicate];
            let ambiguous = parse_profile(&value).expect("nonidentical occupancy parses");
            let reader = if reader_name == "tableau_reader" {
                &ambiguous.tableau_reader
            } else {
                &ambiguous.stock_reader
            };
            assert!(
                reader
                    .recognise(view, TABLEAU_ORIGINS[source])
                    .expect_err("two matching labels refuse")
                    .contains("ambiguous glyph matches")
            );
        }
    }
}


/// Glyph support cannot substitute for scene or positive-empty signatures.
#[test]
fn native_v7_retains_scene_and_positive_empty_pile_requirements() {
    let vision = vision();
    let original = original("native-v7");
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
fn native_v7_accepts_changed_solver_toolbar_without_recalibration() {
    let vision = vision();
    let original = original("native-v7");
    let expected = vision.observe(&original).expect("supported v7 board");
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


/// The retained failure remains a pre-input diagnostic, never restoration or route proof.
#[test]
fn native_v7_preserves_the_original_failure_as_incomplete_no_input_evidence() {
    let report: Value = serde_json::from_slice(
        &std::fs::read(fixture_path("native-v7", "INCOMPLETE.json"))
            .expect("original failure JSON"),
    )
    .expect("failure JSON");
    assert_eq!(report["captures_attempted"], 1);
    assert_eq!(report["core_captures"], 1);
    for field in [
        "core_draws",
        "draws_attempted",
        "draws_acknowledged",
        "undo_all_attempted",
        "confirm_attempted",
        "tableau_count",
        "stock_count",
    ] {
        assert_eq!(report[field], 0, "retained {field}");
    }
    for field in [
        "complete",
        "restored",
        "complete_output_published",
        "input_delivery_uncertain",
        "undo_all_acknowledged",
        "confirm_acknowledged",
    ] {
        assert_eq!(report[field], false, "retained {field}");
    }
    assert_eq!(report["diagnostic_only"], true);
    assert_eq!(report["status"], "INCOMPLETE");
    assert_eq!(report["publication_status"], "not_attempted");
    assert!(
        report["error"]
            .as_str()
            .expect("original refusal")
            .starts_with("unknown rank/suit at tableau slot 6; accepted identities=0;")
    );
}
