//! Exact native rank-eight coverage through production parsing and recognition.
//!
//! The retained one-capture failure attempted no input and proves no complete
//! stock inventory or restoration. Synthetic derivatives establish refusal only;
//! they neither expand supported artwork nor create live acquisition authority.

use super::tests::{labelled_card, native_manifest, native_original, native_patch};
use super::*;
use serde_json::json;


/// Original first-failure PNG supplying the appended native rank-eight crop.
const SOURCE_SHA256: &str = "28e1127ea9d4663ebcca61061779093ebc7b571ec0ebde13a4973906b9913409";


/// Unmodified 27×24 native RGB rank crop at fixed one-based location 2|1.
const EIGHT_RANK_BOUNDS: PixelRect = PixelRect::new(792, 171, 27, 24);


/// Resolve retained fixtures without depending on private runtime evidence paths.
fn fixture_path(generation: &str, name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pyramid-recon")
        .join(generation)
        .join(name)
}


/// Decode an unchanged native original through the production PNG decoder.
fn original(generation: &str, name: &str) -> CapturedFrame {
    crate::capture::decode_png(
        &std::fs::read(fixture_path(generation, name)).expect("retained native PNG"),
    )
    .expect("valid native fixture layout")
}


/// Read a mutable JSON derivative without changing the retained profile on disk.
fn profile_value(generation: &str) -> Value {
    serde_json::from_slice(
        &std::fs::read(fixture_path(generation, "profile.json")).expect("retained profile"),
    )
    .expect("native calibration JSON")
}


/// Load the actual default bundle through its strict production constructor.
fn vision() -> ReconVision {
    ReconVision::bundled_native_v11().expect("bundled native v11 profile")
}


/// Parse in-memory calibration derivatives through the actual bounded parser.
fn parse_profile(value: &Value) -> Result<ReconVision, String> {
    ReconVision::from_bytes(&serde_json::to_vec(value).expect("derived profile JSON"))
}


/// Select a labelled rank without depending on its position in the JSON array.
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


/// Independently reviewed identities in all fixed top-to-bottom tableau slots.
fn expected_tableau() -> [Card; 28] {
    [
        (Rank::Nine, Suit::Clubs),
        (Rank::Eight, Suit::Clubs),
        (Rank::Two, Suit::Spades),
        (Rank::Five, Suit::Spades),
        (Rank::Three, Suit::Clubs),
        (Rank::Jack, Suit::Clubs),
        (Rank::Queen, Suit::Diamonds),
        (Rank::Queen, Suit::Clubs),
        (Rank::Ace, Suit::Clubs),
        (Rank::Ace, Suit::Spades),
        (Rank::Seven, Suit::Clubs),
        (Rank::Four, Suit::Clubs),
        (Rank::Five, Suit::Hearts),
        (Rank::Six, Suit::Hearts),
        (Rank::Five, Suit::Diamonds),
        (Rank::Four, Suit::Spades),
        (Rank::Jack, Suit::Diamonds),
        (Rank::Seven, Suit::Spades),
        (Rank::Ten, Suit::Clubs),
        (Rank::Ten, Suit::Diamonds),
        (Rank::Three, Suit::Diamonds),
        (Rank::Eight, Suit::Diamonds),
        (Rank::Eight, Suit::Hearts),
        (Rank::Two, Suit::Diamonds),
        (Rank::King, Suit::Hearts),
        (Rank::Queen, Suit::Spades),
        (Rank::Nine, Suit::Spades),
        (Rank::Four, Suit::Diamonds),
    ]
    .map(|(rank, suit)| Card::new(rank, suit))
}


/// Paint a checked refusal derivative without changing alpha, dimensions or stride.
fn fill_region(frame: &mut CapturedFrame, bounds: PixelRect, colour: [u8; 3]) {
    frame
        .as_view()
        .expect("native layout")
        .checked_bounds(bounds)
        .expect("bounded derivative");
    for row in bounds.y..bounds.bottom() {
        for column in bounds.x..bounds.right() {
            let offset = row as usize * frame.stride + column as usize * 4;
            frame.pixels[offset..offset + 3].copy_from_slice(&colour);
        }
    }
}


/// Require unknown 2|1 evidence in both banks and refuse the full observation.
fn assert_unknown_eight(reader: &ReconVision, frame: &CapturedFrame) {
    let view = frame.as_view().expect("refusal derivative layout");
    for bank in [&reader.tableau_reader, &reader.stock_reader] {
        assert_eq!(
            bank.recognise(view, TABLEAU_ORIGINS[1])
                .expect("unknown evidence is not ambiguity"),
            None
        );
    }
    assert!(
        reader
            .observe(frame)
            .expect_err("unknown rank-eight card cannot authorise a board")
            .starts_with("unknown rank/suit at tableau slot 1;")
    );
}


/// Only ID and one exact full-mask eight RGB crop per bank differ from retained v10.
#[test]
fn native_v11_exact_eight_crop_and_full_rollback_preserve_all_other_fields() {
    let source = original("native-v11", "baseline.png");
    let mut rollback = profile_value("native-v11");
    let mut prior = profile_value("native-v10");
    let mut measured = native_patch(&source, EIGHT_RANK_BOUNDS, SOURCE_SHA256)["samples"].clone();
    measured["tolerance"] = json!(0);
    assert_eq!(measured["mask"], "1".repeat(648));
    assert_eq!(measured["rgb"].as_str().expect("raw RGB").len(), 1944 * 2);
    assert_eq!(
        rollback["profile_id"],
        "pyramid-native-1920-classic-blue-v11"
    );
    assert_eq!(rollback["tableau_reader"], rollback["stock_reader"]);
    for name in ["tableau_reader", "stock_reader"] {
        assert_eq!(rollback[name]["rank_max_distance"], 24);
        assert_eq!(rollback[name]["suit_max_distance"], 8);
        assert_eq!(rollback[name]["alignment"], 1);
        assert_eq!(rank_variants(&mut prior[name], "8").len(), 3);
        let variants = rank_variants(&mut rollback[name], "8");
        assert_eq!(variants.len(), 4);
        assert_eq!(
            variants
                .pop()
                .expect("one appended exact native eight crop"),
            measured
        );
    }
    rollback["profile_id"] = prior["profile_id"].clone();
    assert_eq!(
        rollback, prior,
        "all old glyphs, limits, masks, geometry, scene, piles and modal remain exact v10"
    );
}


/// Preserve the original zero-input refusal and prove all 29 visible file/bundle identities.
#[test]
fn native_v11_repairs_exact_eight_refusal_and_recognises_all_visible_cards() {
    let frame = original("native-v11", "baseline.png");
    let pixels = frame.pixels.clone();
    let diagnostic_bytes = std::fs::read(fixture_path("native-v11", "INCOMPLETE.json"))
        .expect("retained first-failure report");
    let failure: Value =
        serde_json::from_slice(&diagnostic_bytes).expect("incomplete diagnostics JSON");
    let prior = ReconVision::load(&fixture_path("native-v10", "profile.json"))
        .expect("unchanged native v10 profile");
    assert_eq!(
        prior.observe(&frame).expect_err("original v10 2|1 refusal"),
        failure["error"].as_str().expect("exact retained error")
    );
    for (field, expected) in [
        ("captures_attempted", 1),
        ("core_captures", 1),
        ("core_draws", 0),
        ("draws_attempted", 0),
        ("draws_acknowledged", 0),
        ("stock_count", 0),
        ("tableau_count", 0),
        ("undo_all_attempted", 0),
        ("confirm_attempted", 0),
    ] {
        assert_eq!(failure[field], expected, "{field}");
    }
    for field in [
        "complete",
        "restored",
        "complete_output_published",
        "input_delivery_uncertain",
        "undo_all_acknowledged",
        "confirm_acknowledged",
    ] {
        assert_eq!(failure[field], false, "{field}");
    }
    assert_eq!(failure["publication_status"], "not_attempted");
    assert!(
        xsar::pyramid_json::read_from(diagnostic_bytes.as_slice()).is_err(),
        "initial visible identities do not constitute a complete 24-card stock export"
    );
    let reader = vision();
    let file = ReconVision::load(&fixture_path("native-v11", "profile.json"))
        .expect("strict file-selected v11");
    assert_eq!(reader.profile_id(), file.profile_id());
    let observed = reader
        .observe(&frame)
        .expect("supported visible initial board");
    let ReconScene::Board(board) = &observed else {
        panic!("all independently labelled tableau cards must form a supported board");
    };
    assert_eq!(board.tableau, expected_tableau());
    assert_eq!(
        board.left,
        StockFace::Card(Card::new(Rank::Two, Suit::Clubs))
    );
    assert_eq!(board.right, StockFace::Empty);
    assert_eq!(board.undo_all, None);
    assert_eq!(file.observe(&frame).expect("file/bundle parity"), observed);
    let view = frame.as_view().expect("native card view");
    for bank in [&reader.tableau_reader, &reader.stock_reader] {
        for (index, expected) in expected_tableau().into_iter().enumerate() {
            assert_eq!(
                bank.recognise(view, TABLEAU_ORIGINS[index])
                    .expect("unique tableau identity"),
                Some(expected),
                "fixed tableau slot {index}"
            );
        }
        assert_eq!(
            bank.recognise(view, (LEFT_BOUNDS.x, LEFT_BOUNDS.y))
                .expect("visible LEFT identity"),
            Some(Card::new(Rank::Two, Suit::Clubs))
        );
    }
    assert_eq!(
        frame.pixels, pixels,
        "recognition preserves original pixels"
    );
}


/// All 104 historical labels stay unique, with the same 25 board and four modal scenes.
#[test]
fn native_v11_preserves_historical_glyphs_and_native_scene_corpus() {
    let reader = vision();
    let manifest = native_manifest();
    let mut recognised = 0;
    let mut boards = 0;
    let mut modals = 0;
    for source in manifest["originals"]
        .as_array()
        .expect("historical originals")
    {
        let id = source["id"].as_str().expect("source ID");
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
            .expect("historical labels")
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
            for bank in [&reader.tableau_reader, &reader.stock_reader] {
                assert_eq!(
                    bank.recognise(view, origin)
                        .expect("unique historical label"),
                    Some(labelled_card(observation)),
                    "{}",
                    observation["id"]
                );
            }
            recognised += 1;
        }
    }
    assert_eq!((recognised, boards, modals), (104, 25, 4));
}


/// Retained boards, exact King/clubs corrections and the selected 26-slot position persist.
#[test]
fn native_v11_preserves_retained_boards_king_clubs_and_selected_partial_position() {
    let reader = vision();
    let prior =
        ReconVision::load(&fixture_path("native-v10", "profile.json")).expect("retained v10");
    for generation in [
        "native-v6",
        "native-v7",
        "daily-challenge-solver-unavailable",
        "native-v8",
        "native-v9",
        "native-v10",
    ] {
        let frame = original(generation, "baseline.png");
        let expected = prior.observe(&frame).expect("historical complete board");
        assert!(matches!(expected, ReconScene::Board(_)));
        assert_eq!(
            reader.observe(&frame).expect("v11 retained full board"),
            expected,
            "{generation}"
        );
    }
    let king = original("native-v9", "baseline.png");
    let clubs = original("native-v10", "latest.png");
    assert_eq!(
        reader
            .observe(&clubs)
            .expect("retained v10 clubs correction"),
        prior.observe(&clubs).expect("original v10 meaning")
    );
    for bank in [&reader.tableau_reader, &reader.stock_reader] {
        assert_eq!(
            bank.recognise(
                king.as_view().expect("King source layout"),
                TABLEAU_ORIGINS[4]
            )
            .expect("unique retained King"),
            Some(Card::new(Rank::King, Suit::Spades))
        );
        assert_eq!(
            bank.recognise(
                clubs.as_view().expect("clubs source layout"),
                (RIGHT_BOUNDS.x, RIGHT_BOUNDS.y)
            )
            .expect("unique retained clubs correction"),
            Some(Card::new(Rank::Two, Suit::Clubs))
        );
    }
    let bytes =
        std::fs::read(fixture_path("native-v8", "deal.json")).expect("historical complete deal");
    let deal =
        xsar::pyramid_json::read_from(bytes.as_slice()).expect("bounded complete-deal decoder");
    let mut expected = deal.tableau().map(Some);
    expected[23] = None;
    expected[24] = None;
    assert_eq!(expected.iter().filter(|card| card.is_some()).count(), 26);
    reader
        .verify_position(
            &original("native-v8", "selected-9D.png"),
            &expected,
            Some(Card::new(Rank::Four, Suit::Spades)),
            Some(Card::new(Rank::Eight, Suit::Clubs)),
        )
        .expect("selected 9D and two positive fixed-slot absences");
}


/// Added rank-eight coverage cannot infer identities from erased, damaged or unknown glyphs.
#[test]
fn native_v11_refuses_erased_damaged_and_unknown_eight_glyphs() {
    let reader = vision();
    let source = original("native-v11", "baseline.png");
    for bounds in [
        PixelRect::new(791, 170, 29, 26),
        PixelRect::new(794, 194, 22, 18),
    ] {
        let mut changed = source.clone();
        fill_region(&mut changed, bounds, [255; 3]);
        assert_unknown_eight(&reader, &changed);
    }
    let mut damaged = source.clone();
    fill_region(&mut damaged, PixelRect::new(799, 172, 7, 20), [0; 3]);
    assert_unknown_eight(&reader, &damaged);
    let mut unknown = source;
    for row in 170_u32..196 {
        for column in 791_u32..820 {
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
    assert_unknown_eight(&reader, &unknown);
}


/// Binary occupancy cannot authorise a black eight of clubs with a red rank or suit.
#[test]
fn native_v11_refuses_wrong_colour_eight_rank_or_clubs_suit() {
    let reader = vision();
    let source = original("native-v11", "baseline.png");
    for bounds in [EIGHT_RANK_BOUNDS, PixelRect::new(795, 195, 20, 16)] {
        let mut changed = source.clone();
        for row in bounds.y..bounds.bottom() {
            for column in bounds.x..bounds.right() {
                let offset = row as usize * changed.stride + column as usize * 4;
                let pixel = &mut changed.pixels[offset..offset + 3];
                if ink_class([pixel[0], pixel[1], pixel[2]]) {
                    pixel.copy_from_slice(&[220, 0, 0]);
                }
            }
        }
        assert_unknown_eight(&reader, &changed);
    }
}


/// Exact cross-label duplication and distinct runtime matches both refuse in either reader.
#[test]
fn native_v11_rejects_exact_and_runtime_eight_cross_label_ambiguity() {
    let source = original("native-v11", "baseline.png");
    for name in ["tableau_reader", "stock_reader"] {
        let mut value = profile_value("native-v11");
        let mut duplicate = rank_variants(&mut value[name], "8")
            .last()
            .expect("new native eight crop")
            .clone();
        *rank_variants(&mut value[name], "K") = vec![duplicate.clone()];
        assert!(
            parse_profile(&value)
                .expect_err("exact cross-label occupancy refuses")
                .contains("different glyph labels have identical occupancy")
        );
        let mut rgb = duplicate["rgb"].as_str().expect("RGB samples").to_owned();
        let ink = (0..3).any(|channel| {
            u8::from_str_radix(&rgb[channel * 2..channel * 2 + 2], 16).expect("RGB channel") < 128
        });
        rgb.replace_range(0..6, if ink { "ffffff" } else { "000000" });
        duplicate["rgb"] = json!(rgb);
        *rank_variants(&mut value[name], "K") = vec![duplicate];
        let ambiguous = parse_profile(&value).expect("nonidentical occupancy parses");
        let bank = if name == "tableau_reader" {
            &ambiguous.tableau_reader
        } else {
            &ambiguous.stock_reader
        };
        assert!(
            bank.recognise(source.as_view().expect("native layout"), TABLEAU_ORIGINS[1])
                .expect_err("two distinct matching rank labels refuse")
                .contains("ambiguous glyph matches")
        );
    }
}


/// Caps, eight-variant bounds, zero RGB tolerance and all negative-space samples remain mandatory.
#[test]
fn native_v11_keeps_parser_caps_and_the_complete_eight_mask() {
    for name in ["tableau_reader", "stock_reader"] {
        for (field, invalid) in [
            ("rank_max_distance", 25),
            ("suit_max_distance", 9),
            ("alignment", 2),
        ] {
            let mut value = profile_value("native-v11");
            value[name][field] = json!(invalid);
            assert!(
                parse_profile(&value)
                    .expect_err("native mismatch/alignment cap")
                    .contains("glyph mismatch/alignment limits")
            );
        }
        let mut sparse = profile_value("native-v11");
        rank_variants(&mut sparse[name], "8")
            .last_mut()
            .expect("new eight crop")["mask"] = json!(format!("0{}", "1".repeat(647)));
        assert!(
            parse_profile(&sparse)
                .expect_err("all 648 samples are mandatory")
                .contains("negative space")
        );
        let mut tolerance = profile_value("native-v11");
        rank_variants(&mut tolerance[name], "8")
            .last_mut()
            .expect("new eight crop")["tolerance"] = json!(1);
        assert!(
            parse_profile(&tolerance)
                .expect_err("normalised tolerance remains zero")
                .contains("tolerance must be zero")
        );
        let mut excessive = profile_value("native-v11");
        let variants = rank_variants(&mut excessive[name], "8");
        variants.resize(
            9,
            variants
                .last()
                .expect("bounded existing rank eights")
                .clone(),
        );
        assert!(
            parse_profile(&excessive)
                .expect_err("rank variants remain bounded at eight")
                .contains("variant count exceeds its bound")
        );
    }
}


/// Glyph additions cannot bypass mandatory scenes, positive absence or complete-modal evidence.
#[test]
fn native_v11_retains_scene_positive_empty_and_complete_modal_guards() {
    let reader = vision();
    let source = original("native-v11", "baseline.png");
    for pattern in &reader.board.scene {
        let mut changed = source.clone();
        fill_region(&mut changed, pattern.bounds, [17, 19, 21]);
        assert_eq!(
            reader.observe(&changed).expect("unsupported native scene"),
            ReconScene::Unknown
        );
    }
    let mut missing_empty = source;
    fill_region(
        &mut missing_empty,
        reader.board.right_empty.bounds,
        [17, 19, 21],
    );
    assert!(
        reader.observe(&missing_empty).is_err(),
        "unknown RIGHT is not positive absence"
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
            reader.observe(&changed).expect("damaged modal"),
            ReconScene::Unknown
        );
    }
}
