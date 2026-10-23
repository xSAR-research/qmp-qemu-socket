//! Exact native clubs-variant coverage through production parsing and recognition.
//!
//! Retained originals establish visible identities only. The interrupted two-draw
//! scan is incomplete and unrestored; these tests create no deal or live receipt.
//! Synthetic derivatives establish refusal, never additional supported artwork.

use super::tests::{labelled_card, native_manifest, native_original, native_patch};
use super::*;
use serde_json::json;


/// Original failing second-draw capture supplying the appended clubs suit.
const SOURCE_SHA256: &str = "a6827e24bea61323a315fb3cd0c0e9458c60b0ccd4849ba9efdead6b0b0c8407";


/// Unmodified 20×16 native RGB suit crop on the retained RIGHT two of clubs.
const CLUB_BOUNDS: PixelRect = PixelRect::new(1027, 708, 20, 16);


/// Resolve checked-in evidence without using private runtime acquisition folders.
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


/// Read a mutable JSON derivative while leaving the retained calibration untouched.
fn profile_value(generation: &str) -> Value {
    serde_json::from_slice(
        &std::fs::read(fixture_path(generation, "profile.json")).expect("retained profile"),
    )
    .expect("native calibration JSON")
}


/// Load the preserved historical v10 profile through its strict production parser.
fn vision() -> ReconVision {
    ReconVision::from_bytes(include_bytes!(
        "../../../tests/fixtures/pyramid-recon/native-v10/profile.json"
    ))
    .expect("historical native v10 profile")
}


/// Parse only in-memory calibration derivatives through the actual bounded parser.
fn parse_profile(value: &Value) -> Result<ReconVision, String> {
    ReconVision::from_bytes(&serde_json::to_vec(value).expect("derived profile JSON"))
}


/// Select one suit label without relying on its position in the profile array.
fn suit_variants<'a>(reader: &'a mut Value, label: &str) -> &'a mut Vec<Value> {
    reader["suits"]
        .as_array_mut()
        .expect("suit bank")
        .iter_mut()
        .find(|entry| entry["label"] == label)
        .expect("labelled suit")["variants"]
        .as_array_mut()
        .expect("suit variants")
}


/// Independently reviewed identities at all fixed top-to-bottom tableau slots.
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


/// Paint a checked synthetic refusal derivative without changing geometry or alpha.
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


/// Require unknown RIGHT card evidence in both banks and refuse the full observation.
fn assert_unknown_club(reader: &ReconVision, frame: &CapturedFrame) {
    let view = frame.as_view().expect("refusal derivative layout");
    for bank in [&reader.tableau_reader, &reader.stock_reader] {
        assert_eq!(
            bank.recognise(view, (RIGHT_BOUNDS.x, RIGHT_BOUNDS.y))
                .expect("unknown evidence is not ambiguity"),
            None
        );
    }
    assert!(
        reader
            .observe(frame)
            .expect_err("unknown RIGHT cannot authorise a board")
            .starts_with("RIGHT: unknown rank/suit or empty pile evidence;")
    );
}


/// Only ID and one exact full-mask clubs RGB crop in each bank differ from retained v9.
#[test]
fn native_v10_exact_clubs_crop_and_full_rollback_preserve_all_other_fields() {
    let source = original("native-v10", "latest.png");
    let mut rollback = profile_value("native-v10");
    let mut prior = profile_value("native-v9");
    let mut measured = native_patch(&source, CLUB_BOUNDS, SOURCE_SHA256)["samples"].clone();
    measured["tolerance"] = json!(0);
    assert_eq!(measured["mask"], "1".repeat(320));
    assert_eq!(measured["rgb"].as_str().expect("raw RGB").len(), 960 * 2);
    assert_eq!(
        rollback["profile_id"],
        "pyramid-native-1920-classic-blue-v10"
    );
    assert_eq!(rollback["tableau_reader"], rollback["stock_reader"]);
    for name in ["tableau_reader", "stock_reader"] {
        assert_eq!(rollback[name]["rank_max_distance"], 24);
        assert_eq!(rollback[name]["suit_max_distance"], 8);
        assert_eq!(rollback[name]["alignment"], 1);
        assert_eq!(suit_variants(&mut prior[name], "clubs").len(), 9);
        let variants = suit_variants(&mut rollback[name], "clubs");
        assert_eq!(variants.len(), 10);
        assert_eq!(
            variants.pop().expect("one appended exact clubs crop"),
            measured
        );
    }
    rollback["profile_id"] = prior["profile_id"].clone();
    assert_eq!(
        rollback, prior,
        "all old glyphs, limits, masks, geometry, scene, piles and modal remain exact v9"
    );
}


/// Retain the original refusal and prove only current visible identities plus file/bundle parity.
#[test]
fn native_v10_repairs_exact_right_refusal_and_recognises_visible_current_and_initial_cards() {
    let frame = original("native-v10", "latest.png");
    let baseline = original("native-v10", "baseline.png");
    let pixels = frame.pixels.clone();
    let diagnostic_bytes = std::fs::read(fixture_path("native-v10", "INCOMPLETE.json"))
        .expect("retained first-failure report");
    let failure: Value =
        serde_json::from_slice(&diagnostic_bytes).expect("incomplete diagnostics JSON");
    let prior = ReconVision::load(&fixture_path("native-v9", "profile.json"))
        .expect("unchanged native v9 profile");
    assert_eq!(
        prior
            .observe(&frame)
            .expect_err("original v9 RIGHT refusal"),
        failure["error"].as_str().expect("exact retained error")
    );
    for (field, expected) in [
        ("captures_attempted", 4),
        ("draws_attempted", 2),
        ("draws_acknowledged", 2),
        ("stock_count", 2),
        ("tableau_count", 28),
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
    ] {
        assert_eq!(failure[field], false, "{field}");
    }
    assert_eq!(failure["publication_status"], "not_attempted");
    assert!(
        xsar::pyramid_json::read_from(diagnostic_bytes.as_slice()).is_err(),
        "two known stock entries are not a complete 24-card export"
    );
    let reader = vision();
    let file = ReconVision::load(&fixture_path("native-v10", "profile.json"))
        .expect("strict file-selected v10");
    assert_eq!(reader.profile_id(), file.profile_id());
    for (source, left, right) in [
        (
            &frame,
            Card::new(Rank::Ace, Suit::Spades),
            StockFace::Card(Card::new(Rank::Two, Suit::Clubs)),
        ),
        (
            &baseline,
            Card::new(Rank::Four, Suit::Clubs),
            StockFace::Empty,
        ),
    ] {
        let observed = reader.observe(source).expect("supported visible board");
        let ReconScene::Board(board) = &observed else {
            panic!("visible board required");
        };
        assert_eq!(board.tableau, expected_tableau());
        assert_eq!(board.left, StockFace::Card(left));
        assert_eq!(board.right, right);
        assert_eq!(file.observe(source).expect("file/bundle parity"), observed);
        let view = source.as_view().expect("native card view");
        for bank in [&reader.tableau_reader, &reader.stock_reader] {
            for (index, expected) in expected_tableau().into_iter().enumerate() {
                assert_eq!(
                    bank.recognise(view, TABLEAU_ORIGINS[index])
                        .expect("unique tableau identity"),
                    Some(expected)
                );
            }
            assert_eq!(
                bank.recognise(view, (LEFT_BOUNDS.x, LEFT_BOUNDS.y))
                    .expect("visible LEFT identity"),
                Some(left)
            );
            if let StockFace::Card(expected) = right {
                assert_eq!(
                    bank.recognise(view, (RIGHT_BOUNDS.x, RIGHT_BOUNDS.y))
                        .expect("visible RIGHT identity"),
                    Some(expected)
                );
            }
        }
    }
    assert_eq!(
        frame.pixels, pixels,
        "recognition preserves original pixels"
    );
}


/// All 104 historical labels remain unique, with the same 25 board and four modal signatures.
#[test]
fn native_v10_preserves_historical_glyphs_and_native_scene_corpus() {
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


/// Existing full-board meanings, the retained King fix and the selected 26-slot position persist.
#[test]
fn native_v10_preserves_retained_boards_kings_and_selected_partial_position() {
    let reader = vision();
    let prior = ReconVision::load(&fixture_path("native-v9", "profile.json")).expect("retained v9");
    for generation in [
        "native-v6",
        "native-v7",
        "daily-challenge-solver-unavailable",
        "native-v8",
        "native-v9",
    ] {
        let frame = original(generation, "baseline.png");
        let expected = prior.observe(&frame).expect("historical complete board");
        assert!(matches!(expected, ReconScene::Board(_)));
        assert_eq!(
            reader.observe(&frame).expect("v10 retained full board"),
            expected,
            "{generation}"
        );
    }
    let king = original("native-v9", "baseline.png");
    for bank in [&reader.tableau_reader, &reader.stock_reader] {
        assert_eq!(
            bank.recognise(
                king.as_view().expect("King source layout"),
                TABLEAU_ORIGINS[4]
            )
            .expect("unique retained King"),
            Some(Card::new(Rank::King, Suit::Spades))
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


/// Added clubs coverage cannot invent identities from erased, damaged or unknown suits.
#[test]
fn native_v10_refuses_erased_damaged_and_unknown_clubs() {
    let reader = vision();
    let source = original("native-v10", "latest.png");
    for bounds in [
        PixelRect::new(1026, 707, 22, 18),
        PixelRect::new(1031, 708, 12, 16),
    ] {
        let mut changed = source.clone();
        fill_region(&mut changed, bounds, [255; 3]);
        assert_unknown_club(&reader, &changed);
    }
    let mut unknown = source;
    for row in 707_u32..725 {
        for column in 1026_u32..1048 {
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
    assert_unknown_club(&reader, &unknown);
}


/// Binary occupancy alone cannot authorise a black card with a red rank or suit.
#[test]
fn native_v10_refuses_wrong_colour_two_rank_or_clubs_suit() {
    let reader = vision();
    let source = original("native-v10", "latest.png");
    for bounds in [PixelRect::new(1024, 684, 27, 24), CLUB_BOUNDS] {
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
        assert_unknown_club(&reader, &changed);
    }
}


/// Exact cross-label duplicates and distinct runtime matches refuse in both readers.
#[test]
fn native_v10_rejects_exact_and_runtime_clubs_cross_label_ambiguity() {
    let source = original("native-v10", "latest.png");
    for name in ["tableau_reader", "stock_reader"] {
        let mut value = profile_value("native-v10");
        let mut duplicate = suit_variants(&mut value[name], "clubs")
            .last()
            .expect("new clubs crop")
            .clone();
        *suit_variants(&mut value[name], "spades") = vec![duplicate.clone()];
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
        *suit_variants(&mut value[name], "spades") = vec![duplicate];
        let ambiguous = parse_profile(&value).expect("nonidentical occupancy parses");
        let bank = if name == "tableau_reader" {
            &ambiguous.tableau_reader
        } else {
            &ambiguous.stock_reader
        };
        assert!(
            bank.recognise(
                source.as_view().expect("native layout"),
                (RIGHT_BOUNDS.x, RIGHT_BOUNDS.y)
            )
            .expect_err("two distinct matching suit labels refuse")
            .contains("ambiguous glyph matches")
        );
    }
}


/// Limits, variant bounds, zero RGB tolerance and every negative-space sample remain mandatory.
#[test]
fn native_v10_keeps_parser_caps_and_the_complete_clubs_mask() {
    for name in ["tableau_reader", "stock_reader"] {
        for (field, invalid) in [
            ("rank_max_distance", 25),
            ("suit_max_distance", 9),
            ("alignment", 2),
        ] {
            let mut value = profile_value("native-v10");
            value[name][field] = json!(invalid);
            assert!(
                parse_profile(&value)
                    .expect_err("native mismatch/alignment cap")
                    .contains("glyph mismatch/alignment limits")
            );
        }
        let mut sparse = profile_value("native-v10");
        suit_variants(&mut sparse[name], "clubs")
            .last_mut()
            .expect("new clubs crop")["mask"] = json!(format!("0{}", "1".repeat(319)));
        assert!(
            parse_profile(&sparse)
                .expect_err("all 320 samples are mandatory")
                .contains("negative space")
        );
        let mut tolerance = profile_value("native-v10");
        suit_variants(&mut tolerance[name], "clubs")
            .last_mut()
            .expect("new clubs crop")["tolerance"] = json!(1);
        assert!(
            parse_profile(&tolerance)
                .expect_err("normalised tolerance remains zero")
                .contains("tolerance must be zero")
        );
        let mut excessive = profile_value("native-v10");
        let variants = suit_variants(&mut excessive[name], "clubs");
        variants.resize(17, variants.last().expect("bounded existing clubs").clone());
        assert!(
            parse_profile(&excessive)
                .expect_err("suit variants remain bounded")
                .contains("variant count exceeds its bound")
        );
    }
}


/// Glyph additions cannot bypass mandatory scene, positive absence or complete-modal evidence.
#[test]
fn native_v10_retains_scene_positive_empty_and_complete_modal_guards() {
    let reader = vision();
    let source = original("native-v10", "baseline.png");
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
