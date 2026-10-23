//! Production regressions for the measured native-v14 diamond addition.
//!
//! Labels come from retained visible originals, independently of recognition.
//! Full-region matching, colour exclusion, scene and positive-absence gates
//! remain mandatory; none of these offline observations authorises guest input.

use std::path::PathBuf;

use serde_json::json;

use super::tests::{labelled_card, native_original, native_patch};
use super::*;


/// Direct row-major labels read visually from the original Queens-cleared 8/15 board.
const NEW_TABLEAU: [&str; 28] = [
    "2C", "6H", "5S", "4H", "JH", "JD", "6S", "QD", "4S", "3H", "5D", "10D", "KD", "AS", "10H",
    "3D", "QH", "9S", "9H", "7C", "10S", "QC", "QS", "9C", "4C", "2H", "8D", "5H",
];


/// Direct labels of the previous exposed King-artwork RECON original.
const PRIOR_KING_TABLEAU: [&str; 28] = [
    "10H", "4D", "9C", "JS", "6S", "JD", "6H", "7S", "9D", "2H", "6C", "KD", "6D", "10C", "3C",
    "JC", "2S", "9S", "4S", "8D", "JH", "5S", "5C", "QS", "7H", "4C", "8S", "10S",
];


/// Direct labels of the previous exposed seven-diamonds RECON original.
const PRIOR_SEVEN_TABLEAU: [&str; 28] = [
    "QD", "4D", "KS", "7H", "7D", "6D", "3D", "AS", "10D", "4H", "5D", "8C", "9D", "9C", "QS",
    "4S", "JH", "6H", "KC", "AD", "3C", "5S", "10C", "5H", "JC", "2C", "2D", "10S",
];


/// Resolve an immutable fixture relative to this package, independently of the shell directory.
fn fixture_path(generation: &str, name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pyramid-recon")
        .join(generation)
        .join(name)
}


/// Read original fixture JSON; absent or malformed evidence fails the test.
fn fixture_value(generation: &str, name: &str) -> Value {
    serde_json::from_slice(&std::fs::read(fixture_path(generation, name)).expect("retained JSON"))
        .expect("well-formed retained JSON")
}


/// Decode unchanged original PNG bytes using the actual capture decoder.
fn fixture_frame(generation: &str, name: &str) -> CapturedFrame {
    let bytes = std::fs::read(fixture_path(generation, name)).expect("original PNG bytes");
    let frame = crate::capture::decode_png(&bytes).expect("decode original native PNG");
    assert_eq!((frame.width, frame.height), (1920, 1080));
    frame
}


/// Load the production embedded profile through its strict production parser.
fn vision() -> ReconVision {
    ReconVision::bundled_native_v14().expect("strict bundled native-v14 profile")
}


/// Decode directly reviewed rank/suit labels without calling a glyph classifier.
fn direct_card(label: &str) -> Card {
    let (rank, suit) = label.split_at(label.len() - 1);
    let rank = Rank::ALL
        .into_iter()
        .find(|known| known.as_str() == rank)
        .expect("reviewed rank label");
    let suit = match suit {
        "C" => Suit::Clubs,
        "D" => Suit::Diamonds,
        "H" => Suit::Hearts,
        "S" => Suit::Spades,
        _ => panic!("invalid reviewed suit label"),
    };
    Card::new(rank, suit)
}


/// Return only the measured diamond variants from a complete named bank.
fn diamond_variants<'a>(profile: &'a Value, bank: &str) -> &'a [Value] {
    profile[bank]["suits"]
        .as_array()
        .expect("complete suit bank")
        .iter()
        .find(|entry| entry["label"] == "diamonds")
        .expect("diamond label")["variants"]
        .as_array()
        .expect("measured diamond variants")
}


/// Preserve original fixed indices for eleven remaining cards and seventeen visible absences.
fn selected_position() -> [Option<Card>; 28] {
    let mut cards = [None; 28];
    for (slot, label) in ["3S", "JS", "AC", "8S", "KD", "KS", "5C", "6D", "6H", "5S"]
        .into_iter()
        .enumerate()
    {
        cards[slot] = Some(direct_card(label));
    }
    cards[13] = Some(direct_card("4S"));
    cards
}


/// Check all directly labelled cards in both banks and the complete board/pile observation.
fn assert_full_board(
    reader: &ReconVision,
    frame: &CapturedFrame,
    labels: [&str; 28],
    left: &str,
) -> usize {
    let expected = labels.map(direct_card);
    let left = direct_card(left);
    let ReconScene::Board(observed) = reader.observe(frame).expect("supported full native board")
    else {
        panic!("directly labelled original must be a complete tableau");
    };
    assert_eq!(observed.tableau, expected);
    assert_eq!(observed.left, StockFace::Card(left));
    assert_eq!(observed.right, StockFace::Empty);
    reader
        .verify_position(frame, &expected.map(Some), Some(left), None)
        .expect("full position, original LEFT and positively empty RIGHT");
    let view = frame.as_view().expect("native frame layout");
    for (slot, card) in expected.into_iter().enumerate() {
        for bank in [&reader.tableau_reader, &reader.stock_reader] {
            assert_eq!(
                bank.recognise(view, TABLEAU_ORIGINS[slot])
                    .expect("bounded independent glyph decision"),
                Some(card),
                "slot {slot}: {}",
                bank.failure_details(view, TABLEAU_ORIGINS[slot]),
            );
        }
    }
    for bank in [&reader.tableau_reader, &reader.stock_reader] {
        assert_eq!(
            bank.recognise(view, (LEFT_BOUNDS.x, LEFT_BOUNDS.y))
                .expect("bounded LEFT observation"),
            Some(left),
        );
    }
    29
}


/// Change only checked RGB bytes; retain dimensions, stride, alpha and unrelated native pixels.
fn fill_region(frame: &mut CapturedFrame, bounds: PixelRect, colour: [u8; 3]) {
    frame
        .as_view()
        .expect("native derivative layout")
        .checked_bounds(bounds)
        .expect("bounded derivative region");
    for row in bounds.y..bounds.bottom() {
        for column in bounds.x..bounds.right() {
            let offset = row as usize * frame.stride + column as usize * 4;
            frame.pixels[offset..offset + 3].copy_from_slice(&colour);
        }
    }
}


/// One appended diamond per bank and the profile ID are the entire difference from v13.
#[test]
fn native_v14_whole_profile_rolls_back_to_exact_v13_with_frozen_policy() {
    let prior = fixture_value("native-v13", "profile.json");
    let mut rollback = fixture_value("native-v14", "profile.json");
    assert_eq!(rollback["schema_version"], 2);
    assert_eq!(
        rollback["profile_id"],
        "pyramid-native-1920-classic-blue-v14"
    );
    for bank in ["tableau_reader", "stock_reader"] {
        assert_eq!(rollback[bank]["alignment"], 1);
        assert_eq!(rollback[bank]["suit_roi"], json!([5, 30, 20, 16]));
        assert_eq!(rollback[bank]["rank_roi"], json!([2, 6, 27, 24]));
        assert_eq!(
            rollback[bank]["matching"],
            json!({
                "policy": "paper_ink_l1_v2", "score_scale": 1_000_000,
                "rank_max_error_ppm": 220_000, "suit_max_error_ppm": 120_000,
                "min_label_gap_ppm": 80_000,
            })
        );
        let old = diamond_variants(&prior, bank);
        let variants = rollback[bank]["suits"]
            .as_array_mut()
            .expect("suit labels")
            .iter_mut()
            .find(|entry| entry["label"] == "diamonds")
            .expect("diamond label")["variants"]
            .as_array_mut()
            .expect("diamond variants");
        assert_eq!(old.len(), 12);
        assert_eq!(variants.len(), 13);
        assert_eq!(&variants[..old.len()], old);
        variants.truncate(old.len());
    }
    rollback["profile_id"] = prior["profile_id"].clone();
    assert_eq!(rollback, prior, "all unrelated profile fields remain exact");
    let file = ReconVision::load(&fixture_path("native-v14", "profile.json"))
        .expect("literal operator-selectable v14 profile");
    assert_eq!(file.profile_id(), vision().profile_id());
}


/// Both full-mask appends are the exact measured one-pixel-right/up original RGB crop.
#[test]
fn native_v14_appended_diamond_preserves_shifted_rgb_mask_and_source_in_both_banks() {
    let profile = fixture_value("native-v14", "profile.json");
    let frame = fixture_frame("native-v14", "baseline.png");
    let source_hash = "7263c1ccc78dd7eed213217bbbd739c9128adb7375fb0465a2263300b856d857";
    let bounds = PixelRect::new(896, 354, 20, 16);
    let origin = TABLEAU_ORIGINS[12];
    assert_eq!(
        (
            i64::from(bounds.x) - i64::from(origin.0 + 5),
            i64::from(bounds.y) - i64::from(origin.1 + 30)
        ),
        (1, -1)
    );
    let crop = native_patch(&frame, bounds, source_hash);
    for bank in ["tableau_reader", "stock_reader"] {
        let variant = &diamond_variants(&profile, bank)[12];
        assert_eq!(variant["rgb"], crop["samples"]["rgb"]);
        assert_eq!(variant["source_sha256"], source_hash);
        assert_eq!(variant["mask"], "1".repeat(320));
        assert_eq!(variant["tolerance"], 0);
    }
    assert_eq!(
        diamond_variants(&profile, "tableau_reader"),
        diamond_variants(&profile, "stock_reader")
    );
}


/// The new original supplies 29 directly labelled identities, including K-diamonds at fixed 5|3.
#[test]
fn native_v14_recognises_all_29_new_visible_cards_and_complete_original_position() {
    let frame = fixture_frame("native-v14", "baseline.png");
    assert_eq!(NEW_TABLEAU[12], "KD");
    assert_eq!(assert_full_board(&vision(), &frame, NEW_TABLEAU, "6D"), 29);
}


/// Retain the v13 refusal literally and prove the original failed RECON attempted no input.
#[test]
fn native_v14_preserves_original_v13_refusal_and_zero_input_diagnostic_boundary() {
    let frame = fixture_frame("native-v14", "baseline.png");
    let bytes = std::fs::read(fixture_path("native-v14", "INCOMPLETE.json"))
        .expect("unchanged original diagnostics");
    let report: Value = serde_json::from_slice(&bytes).expect("original diagnostic JSON");
    let prior = ReconVision::bundled_native_v13().expect("immutable prior reader");
    assert_eq!(
        prior.observe(&frame).expect_err("retained v13 refusal"),
        report["error"]
    );
    assert!(
        report["error"]
            .as_str()
            .expect("original refusal")
            .contains("tableau slot 12")
    );
    for field in [
        "draws_attempted",
        "draws_acknowledged",
        "undo_all_attempted",
        "confirm_attempted",
        "core_draws",
        "stock_count",
        "tableau_count",
    ] {
        assert_eq!(report[field], 0, "{field}");
    }
    for field in ["captures_attempted", "core_captures"] {
        assert_eq!(report[field], 1, "{field}");
    }
    for field in [
        "complete",
        "restored",
        "complete_output_published",
        "input_delivery_uncertain",
        "undo_all_acknowledged",
        "confirm_acknowledged",
    ] {
        assert_eq!(report[field], false, "{field}");
    }
    assert_eq!(report["diagnostic_only"], true);
    assert_eq!(report["publication_status"], "not_attempted");
    assert_eq!(report["status"], "INCOMPLETE");
    assert!(xsar::pyramid_json::read_from(bytes.as_slice()).is_err());
}


/// Preserve all 423 older labels plus 71 exposed v13 labels and seventeen positive absences.
#[test]
fn native_v14_preserves_retained_corpus_and_prior_diamond_positions_in_both_banks() {
    let reader = vision();
    let corpus = fixture_value("native-v12", "corpus.json");
    let mut older_labels = 0;
    for source in corpus["sources"].as_array().expect("retained sources") {
        let frame = fixture_frame(
            "native-v12",
            source["file"].as_str().expect("source filename"),
        );
        let view = frame.as_view().expect("original source layout");
        for observation in corpus["observations"]
            .as_array()
            .expect("direct labels")
            .iter()
            .filter(|observation| observation["source_id"] == source["id"])
        {
            assert_eq!(observation["source_sha256"], source["sha256"]);
            assert_eq!(observation["recognition_authority"], false);
            let origin = (
                integer(&observation["origin"][0]).expect("native x"),
                integer(&observation["origin"][1]).expect("native y"),
            );
            for bank in [&reader.tableau_reader, &reader.stock_reader] {
                assert_eq!(
                    bank.recognise(view, origin)
                        .expect("bounded glyph decision"),
                    Some(labelled_card(observation)),
                    "{}: {}",
                    observation["id"],
                    bank.failure_details(view, origin)
                );
            }
            older_labels += 1;
        }
    }
    assert_eq!(older_labels, 423);
    let mut prior_labels = 0;
    for (filename, labels, left) in [
        ("recon-king.png", PRIOR_KING_TABLEAU, "7D"),
        ("recon-seven.png", PRIOR_SEVEN_TABLEAU, "QH"),
    ] {
        prior_labels += assert_full_board(
            &reader,
            &fixture_frame("native-v13", filename),
            labels,
            left,
        );
    }
    let selected = fixture_frame("native-v13", "selected.png");
    let cards = selected_position();
    assert_eq!(cards.iter().filter(|card| card.is_none()).count(), 17);
    reader
        .verify_position(
            &selected,
            &cards,
            Some(direct_card("9D")),
            Some(direct_card("7C")),
        )
        .expect("eleven remaining cards, both piles and seventeen positive absences");
    let view = selected.as_view().expect("original partial position");
    for (slot, card) in cards
        .into_iter()
        .enumerate()
        .filter(|(_, card)| card.is_some())
    {
        for bank in [&reader.tableau_reader, &reader.stock_reader] {
            assert_eq!(
                bank.recognise(view, TABLEAU_ORIGINS[slot])
                    .expect("remaining identity"),
                card
            );
        }
        prior_labels += 1;
    }
    for (bounds, label) in [(LEFT_BOUNDS, "9D"), (RIGHT_BOUNDS, "7C")] {
        for bank in [&reader.tableau_reader, &reader.stock_reader] {
            assert_eq!(
                bank.recognise(view, (bounds.x, bounds.y))
                    .expect("retained pile identity"),
                Some(direct_card(label))
            );
        }
        prior_labels += 1;
    }
    assert_eq!(prior_labels, 71);
    assert!(
        reader.observe(&selected).is_err(),
        "partial position is not full RECON input"
    );
}


/// Full-region colour/shape, scene, modal and positive-empty evidence cannot be bypassed.
#[test]
fn native_v14_keeps_colour_shape_scene_modal_and_absence_refusals() {
    let reader = vision();
    let original = fixture_frame("native-v14", "baseline.png");
    let origin = TABLEAU_ORIGINS[12];
    let suit_bounds = PixelRect::new(origin.0 + 4, origin.1 + 29, 22, 18);
    for derivative in [
        "erased",
        "paperless",
        "colour",
        "mixed",
        "checkerboard",
        "shape",
    ] {
        let mut changed = original.clone();
        match derivative {
            "erased" => fill_region(&mut changed, suit_bounds, [255; 3]),
            "paperless" => fill_region(&mut changed, suit_bounds, [0; 3]),
            "colour" => {
                for row in suit_bounds.y..suit_bounds.bottom() {
                    for column in suit_bounds.x..suit_bounds.right() {
                        let offset = row as usize * changed.stride + column as usize * 4;
                        let minimum = *changed.pixels[offset..offset + 3]
                            .iter()
                            .min()
                            .expect("RGB");
                        changed.pixels[offset..offset + 3].fill(minimum);
                    }
                }
            }
            "mixed" => {
                let view = original.as_view().expect("source layout");
                let point = (origin.1 + 31..origin.1 + 45)
                    .find_map(|row| {
                        (origin.0 + 6..origin.0 + 24).find_map(|column| {
                            let [red, green, blue] =
                                view.pixel_rgb(column, row).expect("bounded core");
                            (red >= 160
                                && red.saturating_sub(green) >= 80
                                && red.saturating_sub(blue) >= 80)
                                .then_some((column, row))
                        })
                    })
                    .expect("genuine red stroke in the intersection of every permitted shift");
                fill_region(&mut changed, PixelRect::new(point.0, point.1, 1, 1), [0; 3]);
            }
            "checkerboard" => {
                fill_region(&mut changed, suit_bounds, [255; 3]);
                for row in suit_bounds.y..suit_bounds.bottom() {
                    for column in suit_bounds.x..suit_bounds.right() {
                        if (row + column) % 2 == 0 {
                            fill_region(
                                &mut changed,
                                PixelRect::new(column, row, 1, 1),
                                [220, 0, 0],
                            );
                        }
                    }
                }
            }
            "shape" => {
                fill_region(&mut changed, suit_bounds, [255; 3]);
                fill_region(
                    &mut changed,
                    PixelRect::new(origin.0 + 5, origin.1 + 36, 20, 4),
                    [220, 0, 0],
                );
            }
            _ => unreachable!("enumerated negative derivative"),
        }
        let view = changed.as_view().expect("bounded native derivative");
        for bank in [&reader.tableau_reader, &reader.stock_reader] {
            assert_eq!(
                bank.recognise(view, origin).expect("bounded refusal"),
                None,
                "{derivative}: {}",
                bank.failure_details(view, origin)
            );
        }
        assert!(
            reader.observe(&changed).is_err(),
            "{derivative} grants no complete-board evidence"
        );
    }
    for pattern in &reader.board.scene {
        let mut changed = original.clone();
        fill_region(&mut changed, pattern.bounds, [17, 19, 21]);
        assert_eq!(
            reader.observe(&changed).expect("unsupported scene"),
            ReconScene::Unknown
        );
    }
    let mut unknown_pile = original;
    fill_region(
        &mut unknown_pile,
        reader.board.right_empty.bounds,
        [17, 19, 21],
    );
    assert!(
        reader
            .observe(&unknown_pile)
            .expect_err("unknown is not empty RIGHT")
            .starts_with("RIGHT: unknown rank/suit or empty pile evidence;")
    );
    let modal = native_original("undo-all-confirmation");
    assert!(matches!(
        reader.observe(&modal).expect("complete modal"),
        ReconScene::UndoConfirmation { .. }
    ));
    for pattern in &reader.dialog.signature {
        let mut changed = modal.clone();
        fill_region(&mut changed, pattern.bounds, [127; 3]);
        assert_eq!(
            reader.observe(&changed).expect("incomplete modal"),
            ReconScene::Unknown
        );
    }
    let selected = fixture_frame("native-v13", "selected.png");
    let mut missing_card = selected_position();
    missing_card[4] = None;
    assert_eq!(
        reader
            .verify_position(
                &selected,
                &missing_card,
                Some(direct_card("9D")),
                Some(direct_card("7C"))
            )
            .expect_err("covered present king cannot become an inferred absence"),
        "execution slot 4 lacks positive absence"
    );
}


/// Strict admission keeps quality/gap constants, full masks, zero tolerance and variant bounds.
#[test]
fn native_v14_parser_keeps_all_frozen_bounds_masks_alignment_and_variant_caps() {
    for bank in ["tableau_reader", "stock_reader"] {
        for (field, changed) in [
            ("rank_max_error_ppm", json!(220_001)),
            ("suit_max_error_ppm", json!(120_001)),
            ("min_label_gap_ppm", json!(79_999)),
            ("score_scale", json!(1_000_001)),
            ("policy", json!("nearest")),
        ] {
            let mut profile = fixture_value("native-v14", "profile.json");
            profile[bank]["matching"][field] = changed;
            assert!(
                ReconVision::from_bytes(&serde_json::to_vec(&profile).expect("invalid derivative"))
                    .is_err(),
                "{bank}: changed {field}"
            );
        }
        for mutation in ["mask", "tolerance", "alignment", "suit-cap", "rank-cap"] {
            let mut profile = fixture_value("native-v14", "profile.json");
            match mutation {
                "alignment" => profile[bank]["alignment"] = json!(2),
                "suit-cap" | "rank-cap" => {
                    let (kind, count) = if mutation == "suit-cap" {
                        ("suits", 17)
                    } else {
                        ("ranks", 9)
                    };
                    let variants = profile[bank][kind][0]["variants"]
                        .as_array_mut()
                        .expect("measured variants");
                    let first = variants[0].clone();
                    variants.resize(count, first);
                }
                _ => {
                    let variant = &mut profile[bank]["suits"]
                        .as_array_mut()
                        .expect("suits")
                        .iter_mut()
                        .find(|entry| entry["label"] == "diamonds")
                        .expect("diamond label")["variants"][12];
                    variant[mutation] = if mutation == "mask" {
                        json!(format!("0{}", "1".repeat(319)))
                    } else {
                        json!(1)
                    };
                }
            }
            assert!(
                ReconVision::from_bytes(&serde_json::to_vec(&profile).expect("invalid derivative"))
                    .is_err(),
                "{bank}: changed {mutation}"
            );
        }
    }
}
