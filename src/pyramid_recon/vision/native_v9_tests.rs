//! Native v9 King-rank regressions over retained original capture evidence.
//!
//! One measured King crop extends both glyph banks without changing thresholds,
//! geometry, scene signatures or input authority. Synthetic derivatives establish
//! refusal only; a recognised initial frame is not a complete scan or restoration.

use super::tests::{labelled_card, native_manifest, native_original, native_patch};
use super::*;
use serde_json::json;


/// Original first-failure PNG supplying the additional native King-rank crop.
const SOURCE_SHA256: &str = "0b2f53ff00b32233bdbad78659f2c18304972338ae6d5af7a7a1640ece6b1310";


/// Native King rank at fixed one-based tableau location 3|2.
const KING_RANK_BOUNDS: PixelRect = PixelRect::new(892, 224, 27, 24);


/// Resolve checked-in evidence independently of the operator's private directories.
fn fixture_path(generation: &str, name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pyramid-recon")
        .join(generation)
        .join(name)
}


/// Decode unchanged native PNG bytes through the production capture decoder.
fn original(generation: &str, name: &str) -> CapturedFrame {
    crate::capture::decode_png(
        &std::fs::read(fixture_path(generation, name)).expect("retained native PNG"),
    )
    .expect("valid native fixture layout")
}


/// Read a mutable JSON copy without changing a retained calibration on disk.
fn profile_value(generation: &str) -> Value {
    serde_json::from_slice(
        &std::fs::read(fixture_path(generation, "profile.json")).expect("retained profile"),
    )
    .expect("native calibration JSON")
}


/// Load the retained v9 calibration independently of the newer production default.
fn vision() -> ReconVision {
    ReconVision::from_bytes(include_bytes!(
        "../../../tests/fixtures/pyramid-recon/native-v9/profile.json"
    ))
    .expect("retained strict native v9 profile")
}


/// Exercise the strict production parser with in-memory calibration derivatives.
fn parse_profile(value: &Value) -> Result<ReconVision, String> {
    ReconVision::from_bytes(&serde_json::to_vec(value).expect("derived profile JSON"))
}


/// Select one labelled rank without depending on the bank's JSON ordering.
fn rank_variants<'a>(reader: &'a mut Value, label: &str) -> &'a mut Vec<Value> {
    reader["ranks"]
        .as_array_mut()
        .expect("rank bank")
        .iter_mut()
        .find(|entry| entry["label"] == label)
        .expect("labelled rank")["variants"]
        .as_array_mut()
        .expect("rank variants")
}


/// Independently reviewed visible identities in fixed top-to-bottom tableau order.
fn expected_tableau() -> [Card; 28] {
    [
        (Rank::Six, Suit::Spades),
        (Rank::Five, Suit::Hearts),
        (Rank::Six, Suit::Hearts),
        (Rank::Two, Suit::Hearts),
        (Rank::King, Suit::Spades),
        (Rank::King, Suit::Clubs),
        (Rank::Two, Suit::Spades),
        (Rank::Seven, Suit::Hearts),
        (Rank::Six, Suit::Clubs),
        (Rank::Six, Suit::Diamonds),
        (Rank::Nine, Suit::Spades),
        (Rank::Five, Suit::Spades),
        (Rank::Ten, Suit::Diamonds),
        (Rank::Four, Suit::Hearts),
        (Rank::Ten, Suit::Spades),
        (Rank::Nine, Suit::Clubs),
        (Rank::Three, Suit::Diamonds),
        (Rank::Ace, Suit::Hearts),
        (Rank::Ten, Suit::Clubs),
        (Rank::Two, Suit::Diamonds),
        (Rank::Queen, Suit::Clubs),
        (Rank::King, Suit::Hearts),
        (Rank::Jack, Suit::Spades),
        (Rank::Ten, Suit::Hearts),
        (Rank::Nine, Suit::Hearts),
        (Rank::Five, Suit::Diamonds),
        (Rank::Four, Suit::Diamonds),
        (Rank::Seven, Suit::Clubs),
    ]
    .map(|(rank, suit)| Card::new(rank, suit))
}


/// Paint a bounded in-memory refusal derivative while preserving alpha and geometry.
fn fill_region(frame: &mut CapturedFrame, bounds: PixelRect, colour: [u8; 3]) {
    let _ = frame
        .as_view()
        .expect("native frame layout")
        .checked_bounds(bounds)
        .expect("bounded derivative");
    for row in bounds.y..bounds.bottom() {
        for column in bounds.x..bounds.right() {
            let offset = row as usize * frame.stride + column as usize * 4;
            frame.pixels[offset..offset + 3].copy_from_slice(&colour);
        }
    }
}


/// Require unknown card evidence in both banks and refuse the complete board observation.
fn assert_unknown_king(reader: &ReconVision, frame: &CapturedFrame) {
    let view = frame.as_view().expect("refusal derivative layout");
    for bank in [&reader.tableau_reader, &reader.stock_reader] {
        assert_eq!(
            bank.recognise(view, TABLEAU_ORIGINS[4])
                .expect("unknown evidence is not ambiguity"),
            None
        );
    }
    assert!(
        reader
            .observe(frame)
            .expect_err("unknown King cannot authorise a board")
            .starts_with("unknown rank/suit at tableau slot 4;")
    );
}


/// The only calibration additions are exact raw King RGB samples in both readers.
#[test]
fn native_v9_exact_king_crop_and_rollback_preserve_every_other_profile_field() {
    let source = original("native-v9", "baseline.png");
    let mut rollback = profile_value("native-v9");
    let mut prior = profile_value("native-v8");
    let mut measured = native_patch(&source, KING_RANK_BOUNDS, SOURCE_SHA256)["samples"].clone();
    measured["tolerance"] = json!(0);
    assert_eq!(
        rollback["profile_id"],
        "pyramid-native-1920-classic-blue-v9"
    );
    assert_eq!(rollback["tableau_reader"], rollback["stock_reader"]);
    for name in ["tableau_reader", "stock_reader"] {
        assert_eq!(rollback[name]["rank_max_distance"], 24);
        assert_eq!(rollback[name]["suit_max_distance"], 8);
        assert_eq!(rollback[name]["alignment"], 1);
        let before = rank_variants(&mut prior[name], "K").len();
        assert_eq!(before, 1, "the retained v8 King bank has one variant");
        let added = rank_variants(&mut rollback[name], "K");
        assert_eq!(added.len(), before + 1);
        assert_eq!(
            added.pop().expect("one appended native King crop"),
            measured
        );
    }
    rollback["profile_id"] = prior["profile_id"].clone();
    assert_eq!(
        rollback, prior,
        "every old glyph, threshold, scene, pile, geometry and modal remains exact v8"
    );
}


/// The original refusal is reproducible, while both current banks recognise all visible cards.
#[test]
fn native_v9_repairs_the_retained_king_failure_and_observes_all_visible_cards() {
    let frame = original("native-v9", "baseline.png");
    let original_pixels = frame.pixels.clone();
    let prior = ReconVision::load(&fixture_path("native-v8", "profile.json"))
        .expect("unchanged native v8 profile");
    let failure: Value = serde_json::from_slice(
        &std::fs::read(fixture_path("native-v9", "INCOMPLETE.json"))
            .expect("retained first-failure report"),
    )
    .expect("first-failure JSON");
    assert_eq!(
        prior.observe(&frame).expect_err("retained v8 refusal"),
        failure["error"].as_str().expect("original failure text")
    );
    assert_eq!(failure["captures_attempted"], 1);
    assert_eq!(failure["complete"], false);
    assert_eq!(failure["restored"], false);
    assert_eq!(failure["draws_attempted"], 0);
    assert_eq!(failure["undo_all_attempted"], 0);
    assert_eq!(failure["confirm_attempted"], 0);
    assert_eq!(failure["publication_status"], "not_attempted");

    let reader = vision();
    let observed = reader.observe(&frame).expect("v9 initial native board");
    let ReconScene::Board(board) = &observed else {
        panic!("all 28 independently labelled cards must form a supported board");
    };
    assert_eq!(board.tableau, expected_tableau());
    assert_eq!(
        board.left,
        StockFace::Card(Card::new(Rank::Four, Suit::Clubs))
    );
    assert_eq!(board.right, StockFace::Empty);
    assert_eq!(board.undo_all, None);
    let file = ReconVision::load(&fixture_path("native-v9", "profile.json"))
        .expect("strict file-selected v9 profile");
    assert_eq!(reader.profile_id(), file.profile_id());
    assert_eq!(file.observe(&frame).expect("file/bundle parity"), observed);

    let view = frame.as_view().expect("native card view");
    for (index, card) in expected_tableau().into_iter().enumerate() {
        for bank in [&reader.tableau_reader, &reader.stock_reader] {
            assert_eq!(
                bank.recognise(view, TABLEAU_ORIGINS[index])
                    .expect("unique visible identity"),
                Some(card),
                "fixed tableau slot {index}"
            );
        }
    }
    for bank in [&reader.tableau_reader, &reader.stock_reader] {
        assert_eq!(
            bank.recognise(view, (LEFT_BOUNDS.x, LEFT_BOUNDS.y))
                .expect("visible initial DL"),
            Some(Card::new(Rank::Four, Suit::Clubs))
        );
    }
    assert_eq!(
        frame.pixels, original_pixels,
        "recognition preserves captured pixels"
    );
}


/// Every historical labelled glyph remains unique in both banks, including Kings.
#[test]
fn native_v9_preserves_historical_glyphs_board_scenes_and_modals() {
    let reader = vision();
    let manifest = native_manifest();
    let mut recognised = 0;
    let mut kings = 0;
    let mut boards = 0;
    let mut modals = 0;
    for source in manifest["originals"]
        .as_array()
        .expect("retained originals")
    {
        let id = source["id"].as_str().expect("native source ID");
        let frame = native_original(id);
        let view = frame.as_view().expect("historical native layout");
        let modal = id.contains("modal") || id == "undo-all-confirmation";
        assert_eq!(
            reader.board.matches(view).expect("historical scene"),
            !modal,
            "{id}"
        );
        if modal {
            assert!(matches!(
                reader.observe(&frame).expect("complete historical modal"),
                ReconScene::UndoConfirmation { .. }
            ));
            modals += 1;
        } else {
            assert!(!matches_all(&reader.dialog.signature, view).expect("not a modal"));
            boards += 1;
        }
        for observation in manifest["labelled_card_observations"]
            .as_array()
            .expect("historical direct labels")
            .iter()
            .chain(
                manifest["endpoint_card_observations"]
                    .as_array()
                    .expect("endpoint labels"),
            )
            .filter(|entry| entry["source_id"] == source["id"])
        {
            let origin = (
                integer(&observation["origin"][0]).expect("labelled x"),
                integer(&observation["origin"][1]).expect("labelled y"),
            );
            let expected = labelled_card(observation);
            for bank in [&reader.tableau_reader, &reader.stock_reader] {
                assert_eq!(
                    bank.recognise(view, origin)
                        .expect("unique historical identity"),
                    Some(expected),
                    "{}",
                    observation["id"]
                );
            }
            recognised += 1;
            kings += usize::from(expected.rank() == Rank::King);
        }
    }
    assert_eq!(recognised, 104);
    assert!(kings > 0, "historical King variants remain exercised");
    assert_eq!(boards, 25);
    assert_eq!(modals, 4);
}


/// All retained complete boards and the selected partial position retain exact meaning.
#[test]
fn native_v9_preserves_retained_boards_and_selected_partial_position() {
    let reader = vision();
    let prior = ReconVision::load(&fixture_path("native-v8", "profile.json"))
        .expect("historical native v8 reader");
    for generation in [
        "native-v6",
        "native-v7",
        "daily-challenge-solver-unavailable",
        "native-v8",
    ] {
        let frame = original(generation, "baseline.png");
        let expected = prior.observe(&frame).expect("historical full board");
        assert!(matches!(expected, ReconScene::Board(_)), "{generation}");
        assert_eq!(
            reader.observe(&frame).expect("v9 full board"),
            expected,
            "{generation}"
        );
    }
    let bytes = std::fs::read(fixture_path("native-v8", "deal.json"))
        .expect("retained complete acquired deal");
    let deal = xsar::pyramid_json::read_from(bytes.as_slice()).expect("checked acquired deal");
    let mut expected = deal.tableau().map(Some);
    expected[23] = None;
    expected[24] = None;
    reader
        .verify_position(
            &original("native-v8", "selected-9D.png"),
            &expected,
            Some(Card::new(Rank::Four, Suit::Spades)),
            Some(Card::new(Rank::Eight, Suit::Clubs)),
        )
        .expect("selected 9D, 26 occupied fixed slots and two positive absences");
}


/// Added King evidence cannot convert erased, damaged or unknown glyphs into identities.
#[test]
fn native_v9_refuses_erased_damaged_and_unknown_king_glyphs() {
    let reader = vision();
    let frame = original("native-v9", "baseline.png");
    for bounds in [
        PixelRect::new(891, 223, 30, 26),
        PixelRect::new(894, 247, 22, 20),
    ] {
        let mut erased = frame.clone();
        fill_region(&mut erased, bounds, [255; 3]);
        assert_unknown_king(&reader, &erased);
    }
    let mut damaged = frame.clone();
    fill_region(&mut damaged, PixelRect::new(897, 225, 7, 20), [255; 3]);
    assert_unknown_king(&reader, &damaged);
    let mut unknown = frame;
    for row in 223_u32..249 {
        for column in 891_u32..921 {
            let offset = row as usize * unknown.stride + column as usize * 4;
            unknown.pixels[offset..offset + 3].copy_from_slice(
                if (row + column).is_multiple_of(2) {
                    &[0; 3]
                } else {
                    &[255; 3]
                },
            );
        }
    }
    assert_unknown_king(&reader, &unknown);
}


/// Contradictory rank/suit colours remain unknown even when binary occupancy matches.
#[test]
fn native_v9_refuses_wrong_colour_king_rank_or_suit() {
    let reader = vision();
    let frame = original("native-v9", "baseline.png");
    for bounds in [KING_RANK_BOUNDS, PixelRect::new(895, 248, 20, 16)] {
        let mut mixed = frame.clone();
        for row in bounds.y..bounds.bottom() {
            for column in bounds.x..bounds.right() {
                let offset = row as usize * mixed.stride + column as usize * 4;
                let pixel = &mut mixed.pixels[offset..offset + 3];
                if ink_class([pixel[0], pixel[1], pixel[2]]) {
                    pixel.copy_from_slice(&[220, 0, 0]);
                }
            }
        }
        assert_unknown_king(&reader, &mixed);
    }
}


/// Exact cross-label duplication and near matches both refuse rather than choose a label.
#[test]
fn native_v9_rejects_exact_and_runtime_king_rank_ambiguity_in_both_readers() {
    let frame = original("native-v9", "baseline.png");
    for name in ["tableau_reader", "stock_reader"] {
        let mut value = profile_value("native-v9");
        let mut duplicate = rank_variants(&mut value[name], "K")
            .last()
            .expect("new King crop")
            .clone();
        *rank_variants(&mut value[name], "8") = vec![duplicate.clone()];
        assert!(
            parse_profile(&value)
                .expect_err("identical cross-label occupancy refuses")
                .contains("different glyph labels have identical occupancy")
        );
        let mut rgb = duplicate["rgb"].as_str().expect("RGB samples").to_owned();
        let first_ink = (0..3).any(|channel| {
            u8::from_str_radix(&rgb[channel * 2..channel * 2 + 2], 16).expect("RGB channel") < 128
        });
        rgb.replace_range(0..6, if first_ink { "ffffff" } else { "000000" });
        duplicate["rgb"] = json!(rgb);
        *rank_variants(&mut value[name], "8") = vec![duplicate];
        let ambiguous = parse_profile(&value).expect("nonidentical occupancy parses");
        let bank = if name == "tableau_reader" {
            &ambiguous.tableau_reader
        } else {
            &ambiguous.stock_reader
        };
        assert!(
            bank.recognise(frame.as_view().expect("native layout"), TABLEAU_ORIGINS[4])
                .expect_err("distinct matching ranks refuse")
                .contains("ambiguous glyph matches")
        );
    }
}


/// The corrective sample cannot increase mismatch/alignment limits or hide negative space.
#[test]
fn native_v9_retains_parser_caps_and_complete_king_masks() {
    for name in ["tableau_reader", "stock_reader"] {
        for (field, invalid) in [
            ("rank_max_distance", 25),
            ("suit_max_distance", 9),
            ("alignment", 2),
        ] {
            let mut value = profile_value("native-v9");
            value[name][field] = json!(invalid);
            assert!(
                parse_profile(&value)
                    .expect_err("native mismatch/alignment cap")
                    .contains("glyph mismatch/alignment limits")
            );
        }
        let mut sparse = profile_value("native-v9");
        let last = rank_variants(&mut sparse[name], "K")
            .last_mut()
            .expect("new King");
        last["mask"] = json!("0".repeat(27 * 24));
        assert!(
            parse_profile(&sparse).is_err(),
            "all King negative space remains mandatory"
        );
    }
}


/// New glyph coverage grants no scene, empty-pile or incomplete-modal authority.
#[test]
fn native_v9_retains_scene_empty_pile_and_complete_modal_requirements() {
    let reader = vision();
    let frame = original("native-v9", "baseline.png");
    for pattern in &reader.board.scene {
        let mut changed = frame.clone();
        fill_region(&mut changed, pattern.bounds, [17, 19, 21]);
        assert_eq!(
            reader.observe(&changed).expect("unsupported native scene"),
            ReconScene::Unknown
        );
    }
    let mut missing_empty = frame;
    fill_region(
        &mut missing_empty,
        reader.board.right_empty.bounds,
        [17, 19, 21],
    );
    assert!(
        reader.observe(&missing_empty).is_err(),
        "unknown DR is not positive absence"
    );
    let modal = native_original("undo-all-confirmation");
    assert!(matches!(
        reader.observe(&modal).expect("complete native modal"),
        ReconScene::UndoConfirmation { .. }
    ));
    for pattern in &reader.dialog.signature {
        let mut changed = modal.clone();
        fill_region(&mut changed, pattern.bounds, [127; 3]);
        assert_eq!(
            reader.observe(&changed).expect("damaged native modal"),
            ReconScene::Unknown
        );
    }
}
