//! Production-reader regressions for measured native-v13 diamond additions.
//!
//! Direct labels describe visible original pixels only. Retained PLAY diagnostics,
//! zero-input RECON refusals and offline recognition remain distinct evidence;
//! none supplies a fresh acquisition, unseen stock, restoration or input authority.

use std::path::PathBuf;

use serde_json::json;

use super::tests::{labelled_card, native_original, native_patch};
use super::*;


/// Direct row-major identities read visually from the retained king RECON original.
const KING_TABLEAU: [&str; 28] = [
    "10H", "4D", "9C", "JS", "6S", "JD", "6H", "7S", "9D", "2H", "6C", "KD", "6D", "10C", "3C",
    "JC", "2S", "9S", "4S", "8D", "JH", "5S", "5C", "QS", "7H", "4C", "8S", "10S",
];


/// Direct row-major identities read visually from the retained seven RECON original.
const SEVEN_TABLEAU: [&str; 28] = [
    "QD", "4D", "KS", "7H", "7D", "6D", "3D", "AS", "10D", "4H", "5D", "8C", "9D", "9C", "QS",
    "4S", "JH", "6H", "KC", "AD", "3C", "5S", "10C", "5H", "JC", "2C", "2D", "10S",
];


/// Resolve retained evidence relative to the checkout without writing its originals.
fn fixture_path(generation: &str, name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pyramid-recon")
        .join(generation)
        .join(name)
}


/// Read a retained JSON document; malformed or missing evidence fails the test.
fn fixture_value(generation: &str, name: &str) -> Value {
    serde_json::from_slice(&std::fs::read(fixture_path(generation, name)).expect("retained JSON"))
        .expect("well-formed retained JSON")
}


/// Decode original PNG bytes through production capture code and require native dimensions.
fn fixture_frame(generation: &str, name: &str) -> CapturedFrame {
    let bytes = std::fs::read(fixture_path(generation, name)).expect("original native PNG bytes");
    let frame = crate::capture::decode_png(&bytes).expect("decode native original");
    assert_eq!((frame.width, frame.height), (1920, 1080));
    frame
}


/// Construct the deployed default profile through the same strict production parser.
fn vision() -> ReconVision {
    ReconVision::bundled_native_v13().expect("strict bundled native-v13 profile")
}


/// Decode an independently reviewed conventional label without consulting recognition.
fn direct_card(label: &str) -> Card {
    let (rank, suit) = label.split_at(label.len() - 1);
    let rank = Rank::ALL
        .into_iter()
        .find(|known| known.as_str() == rank)
        .expect("direct rank label");
    let suit = match suit {
        "C" => Suit::Clubs,
        "D" => Suit::Diamonds,
        "H" => Suit::Hearts,
        "S" => Suit::Spades,
        _ => panic!("invalid direct suit label"),
    };
    Card::new(rank, suit)
}


/// Keep every original slot, including seventeen visually absent PLAY cards.
fn selected_position() -> [Option<Card>; 28] {
    let mut cards = [None; 28];
    for (index, label) in ["3S", "JS", "AC", "8S", "KD", "KS", "5C", "6D", "6H", "5S"]
        .into_iter()
        .enumerate()
    {
        cards[index] = Some(direct_card(label));
    }
    cards[13] = Some(direct_card("4S"));
    cards
}


/// Select only the diamond variants from an explicit complete profile bank.
fn diamond_variants<'a>(profile: &'a Value, bank: &str) -> &'a [Value] {
    profile[bank]["suits"]
        .as_array()
        .expect("complete suit bank")
        .iter()
        .find(|entry| entry["label"] == "diamonds")
        .expect("diamond label")["variants"]
        .as_array()
        .expect("diamond variants")
}


/// Change only checked RGB pixels; preserve layout, alpha and all other native evidence.
fn fill_region(frame: &mut CapturedFrame, bounds: PixelRect, colour: [u8; 3]) {
    frame
        .as_view()
        .expect("native derivative layout")
        .checked_bounds(bounds)
        .expect("bounded derivative pixels");
    for row in bounds.y..bounds.bottom() {
        for column in bounds.x..bounds.right() {
            let offset = row as usize * frame.stride + column as usize * 4;
            frame.pixels[offset..offset + 3].copy_from_slice(&colour);
        }
    }
}


/// Removing only the measured diamond append and profile ID restores the complete v12 JSON.
#[test]
fn native_v13_full_profile_rolls_back_to_exact_v12_with_frozen_policy() {
    let prior = fixture_value("native-v12", "profile.json");
    let mut rollback = fixture_value("native-v13", "profile.json");
    assert_eq!(rollback["schema_version"], 2);
    assert_eq!(
        rollback["profile_id"],
        "pyramid-native-1920-classic-blue-v13"
    );
    for bank in ["tableau_reader", "stock_reader"] {
        assert_eq!(rollback[bank]["alignment"], 1);
        assert_eq!(rollback[bank]["suit_roi"], json!([5, 30, 20, 16]));
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
            .expect("measured diamond variants");
        assert_eq!(old.len(), 10);
        assert_eq!(variants.len(), old.len() + 2);
        assert_eq!(&variants[..old.len()], old);
        variants.truncate(old.len());
    }
    rollback["profile_id"] = prior["profile_id"].clone();
    assert_eq!(rollback, prior, "every other field is retained exactly");
    let bundled = vision();
    let file = ReconVision::load(&fixture_path("native-v13", "profile.json"))
        .expect("operator-selectable native-v13 profile");
    assert_eq!(file.profile_id(), bundled.profile_id());
}


/// Both appended witnesses retain exact full-ROI RGB, including the measured one-pixel king offset.
#[test]
fn native_v13_appended_diamonds_equal_original_rgb_in_both_banks() {
    let profile = fixture_value("native-v13", "profile.json");
    for (offset, filename, bounds, source_hash) in [
        (
            0,
            "recon-king.png",
            PixelRect::new(695, 354, 20, 16),
            "4129c5cfe89c83f396a2c59601927707752447c6ea99541cbd185748a0312856",
        ),
        (
            1,
            "selected.png",
            PixelRect::new(895, 248, 20, 16),
            "af3cbbc3b38a3cf8291bbea9ca45396f20f4c9be7e7dbf034dbdb3196baaf970",
        ),
    ] {
        let frame = fixture_frame("native-v13", filename);
        let crop = native_patch(&frame, bounds, source_hash);
        for bank in ["tableau_reader", "stock_reader"] {
            let variant = &diamond_variants(&profile, bank)[10 + offset];
            assert_eq!(variant["rgb"], crop["samples"]["rgb"], "{bank}: {filename}");
            assert_eq!(variant["source_sha256"], source_hash);
            assert_eq!(variant["mask"], "1".repeat(320));
            assert_eq!(variant["tolerance"], 0);
            assert_ne!(
                variant["source_sha256"],
                "12dd83cf991f7cce41e790acd90d287cb44babcd318b480a24377b3819f6018b",
                "the already exposed seven capture adds no template and is development evidence",
            );
        }
    }
    assert_eq!(
        diamond_variants(&profile, "tableau_reader"),
        diamond_variants(&profile, "stock_reader"),
    );
}


/// All 423 retained v12 direct labels remain correct independently in both production banks.
#[test]
fn native_v13_preserves_all_423_retained_v12_visible_identities_in_both_banks() {
    let corpus = fixture_value("native-v12", "corpus.json");
    let reader = vision();
    let mut recognised = 0;
    for source in corpus["sources"]
        .as_array()
        .expect("retained source inventory")
    {
        let frame = fixture_frame(
            "native-v12",
            source["file"].as_str().expect("original filename"),
        );
        let view = frame.as_view().expect("original native layout");
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
            for (name, bank) in [
                ("tableau", &reader.tableau_reader),
                ("stock", &reader.stock_reader),
            ] {
                assert_eq!(
                    bank.recognise(view, origin)
                        .expect("bounded production decision"),
                    Some(labelled_card(observation)),
                    "{} {} in {name}: {}",
                    source["split"],
                    observation["id"],
                    bank.failure_details(view, origin),
                );
            }
            recognised += 1;
        }
    }
    assert_eq!(
        recognised, 423,
        "each label receives two independent bank decisions"
    );
}


/// The original selected PLAY stop changes from strict refusal to the exact partial position.
#[test]
fn native_v13_recognises_selected_play_position_without_inferring_removed_cards() {
    let frame = fixture_frame("native-v13", "selected.png");
    let cards = selected_position();
    let left = Some(direct_card("9D"));
    let right = Some(direct_card("7C"));
    let retained = fixture_value("native-v13", "error-state.json");
    let legacy = ReconVision::bundled_native_v12().expect("unchanged v12 reader");
    let error = legacy
        .verify_position(&frame, &cards, left, right)
        .expect_err("retained strict selected-king refusal");
    assert_eq!(error, retained["details"]["first_unmatched_observation"]);
    assert!(error.contains("120979ppm"));
    assert!(error.contains("suit: paper_ink_l1_v2 refused: absolute shape error"));
    assert_eq!(retained["details"]["verified_actions"], 27);
    assert_eq!(retained["details"]["planned_actions"], 42);
    assert_eq!(retained["details"]["gestures_acknowledged"], 42);
    assert_eq!(retained["input_authority"], false);
    assert_eq!(retained["resume_authority"], false);
    let reader = vision();
    reader
        .verify_position(&frame, &cards, left, right)
        .expect("all eleven remaining identities, both piles and seventeen positive absences");
    let view = frame.as_view().expect("selected native layout");
    for (slot, card) in cards
        .into_iter()
        .enumerate()
        .filter(|(_, card)| card.is_some())
    {
        for bank in [&reader.tableau_reader, &reader.stock_reader] {
            assert_eq!(
                bank.recognise(view, TABLEAU_ORIGINS[slot])
                    .expect("remaining card"),
                card
            );
        }
    }
    for (bounds, card) in [(LEFT_BOUNDS, left), (RIGHT_BOUNDS, right)] {
        for bank in [&reader.tableau_reader, &reader.stock_reader] {
            assert_eq!(
                bank.recognise(view, (bounds.x, bounds.y))
                    .expect("direct pile identity"),
                card,
            );
        }
    }
    assert!(
        reader.observe(&frame).is_err(),
        "partial PLAY evidence is not a full tableau"
    );
}


/// Exposed RECON development captures match direct full-board labels and positive empty RIGHT.
#[test]
fn native_v13_matches_both_recon_boards_and_preserves_exact_v12_refusals() {
    let legacy = ReconVision::bundled_native_v12().expect("unchanged v12 profile");
    let reader = vision();
    for (stem, labels, left) in [
        ("recon-king", KING_TABLEAU, "7D"),
        ("recon-seven", SEVEN_TABLEAU, "QH"),
    ] {
        let frame = fixture_frame("native-v13", &format!("{stem}.png"));
        let incomplete = fixture_value("native-v13", &format!("{stem}-INCOMPLETE.json"));
        assert_eq!(
            legacy
                .observe(&frame)
                .expect_err("retained original strict refusal"),
            incomplete["error"],
        );
        let expected = labels.map(direct_card);
        let ReconScene::Board(observed) = reader.observe(&frame).expect("supported full board")
        else {
            panic!("directly labelled RECON original must be a complete board");
        };
        assert_eq!(observed.tableau, expected, "{stem}");
        assert_eq!(observed.left, StockFace::Card(direct_card(left)), "{stem}");
        assert_eq!(observed.right, StockFace::Empty, "{stem}");
        reader
            .verify_position(&frame, &expected.map(Some), Some(direct_card(left)), None)
            .expect("whole independent position agrees");
        let view = frame.as_view().expect("native board layout");
        for (slot, expected_card) in expected.into_iter().enumerate() {
            for bank in [&reader.tableau_reader, &reader.stock_reader] {
                assert_eq!(
                    bank.recognise(view, TABLEAU_ORIGINS[slot])
                        .expect("bounded glyph decision"),
                    Some(expected_card),
                    "{stem} slot {slot}: {}",
                    bank.failure_details(view, TABLEAU_ORIGINS[slot]),
                );
            }
        }
        for bank in [&reader.tableau_reader, &reader.stock_reader] {
            assert_eq!(
                bank.recognise(view, (LEFT_BOUNDS.x, LEFT_BOUNDS.y))
                    .expect("direct LEFT identity"),
                Some(direct_card(left)),
                "{stem}: LEFT",
            );
        }
    }
}


/// Retained baseline diagnostics stay zero-input failures and cannot become complete deal input.
#[test]
fn native_v13_recon_diagnostics_retain_zero_input_and_no_publication() {
    for filename in ["recon-king-INCOMPLETE.json", "recon-seven-INCOMPLETE.json"] {
        let bytes =
            std::fs::read(fixture_path("native-v13", filename)).expect("original diagnostic");
        let report: Value = serde_json::from_slice(&bytes).expect("original diagnostic JSON");
        for field in [
            "draws_attempted",
            "draws_acknowledged",
            "undo_all_attempted",
            "confirm_attempted",
            "core_draws",
            "stock_count",
            "tableau_count",
        ] {
            assert_eq!(report[field], 0, "{filename}: {field}");
        }
        assert_eq!(report["captures_attempted"], 1);
        assert_eq!(report["core_captures"], 1);
        for field in [
            "complete",
            "restored",
            "complete_output_published",
            "input_delivery_uncertain",
            "undo_all_acknowledged",
            "confirm_acknowledged",
        ] {
            assert_eq!(report[field], false, "{filename}: {field}");
        }
        assert_eq!(report["diagnostic_only"], true);
        assert_eq!(report["status"], "INCOMPLETE");
        assert_eq!(report["publication_status"], "not_attempted");
        assert!(xsar::pyramid_json::read_from(bytes.as_slice()).is_err());
    }
}


/// New measured samples do not permit erased, contradictory, paperless or unrelated suit shapes.
#[test]
fn native_v13_refuses_colour_erasure_paper_checkerboard_and_shape_derivatives() {
    let original = fixture_frame("native-v13", "selected.png");
    let reader = vision();
    let origin = TABLEAU_ORIGINS[4];
    let bounds = PixelRect::new(origin.0 + 4, origin.1 + 29, 22, 18);
    for derivative in ["erased", "paperless", "colour", "checkerboard", "shape"] {
        let mut changed = original.clone();
        match derivative {
            "erased" => fill_region(&mut changed, bounds, [255; 3]),
            "paperless" => fill_region(&mut changed, bounds, [0; 3]),
            "colour" => {
                for row in bounds.y..bounds.bottom() {
                    for column in bounds.x..bounds.right() {
                        let offset = row as usize * changed.stride + column as usize * 4;
                        let rgb = &mut changed.pixels[offset..offset + 3];
                        let minimum = *rgb.iter().min().expect("RGB channels");
                        rgb.copy_from_slice(&[minimum; 3]);
                    }
                }
            }
            "checkerboard" => {
                fill_region(&mut changed, bounds, [255; 3]);
                for row in bounds.y..bounds.bottom() {
                    for column in bounds.x..bounds.right() {
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
                fill_region(&mut changed, bounds, [255; 3]);
                fill_region(
                    &mut changed,
                    PixelRect::new(origin.0 + 5, origin.1 + 36, 20, 4),
                    [220, 0, 0],
                );
            }
            _ => unreachable!("enumerated diagnostic derivative"),
        }
        for bank in [&reader.tableau_reader, &reader.stock_reader] {
            let view = changed.as_view().expect("bounded native derivative");
            assert_eq!(
                bank.recognise(view, origin)
                    .expect("honest unsupported glyph decision"),
                None,
                "{derivative}: {}",
                bank.failure_details(view, origin),
            );
        }
        assert!(
            reader
                .verify_position(
                    &changed,
                    &selected_position(),
                    Some(direct_card("9D")),
                    Some(direct_card("7C")),
                )
                .is_err(),
            "{derivative} cannot authorise the known position",
        );
    }
}


/// Scene, modal, pile and positive-absence evidence remain mandatory independently of glyphs.
#[test]
fn native_v13_keeps_scene_modal_pile_and_positive_absence_guards() {
    let original = fixture_frame("native-v13", "recon-king.png");
    let reader = vision();
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
    let error = reader
        .observe(&unknown_pile)
        .expect_err("unknown pile is not empty");
    assert!(
        error.starts_with("RIGHT: unknown rank/suit or empty pile evidence;"),
        "{error}"
    );
    let modal = native_original("undo-all-confirmation");
    assert!(matches!(
        reader.observe(&modal).expect("complete retained modal"),
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
    for slot in [4, 13] {
        let mut cards = selected_position();
        cards[slot] = None;
        let error = reader
            .verify_position(
                &selected,
                &cards,
                Some(direct_card("9D")),
                Some(direct_card("7C")),
            )
            .expect_err("known present card lacks positive absence");
        assert_eq!(
            error,
            format!("execution slot {slot} lacks positive absence")
        );
    }
}


/// Profile admission still refuses relaxed policy, sparse masks, tolerance and wider alignment.
#[test]
fn native_v13_parser_keeps_frozen_bounds_full_masks_and_alignment() {
    for bank in ["tableau_reader", "stock_reader"] {
        for (field, changed) in [
            ("rank_max_error_ppm", json!(220_001)),
            ("suit_max_error_ppm", json!(120_001)),
            ("min_label_gap_ppm", json!(79_999)),
            ("score_scale", json!(1_000_001)),
            ("policy", json!("nearest")),
        ] {
            let mut profile = fixture_value("native-v13", "profile.json");
            profile[bank]["matching"][field] = changed;
            assert!(
                ReconVision::from_bytes(&serde_json::to_vec(&profile).expect("invalid derivative"))
                    .is_err(),
                "{bank}: changed {field}",
            );
        }
        for mutation in ["mask", "tolerance", "alignment"] {
            let mut profile = fixture_value("native-v13", "profile.json");
            if mutation == "alignment" {
                profile[bank]["alignment"] = json!(2);
            } else {
                let variant = &mut profile[bank]["suits"]
                    .as_array_mut()
                    .expect("complete suit bank")
                    .iter_mut()
                    .find(|entry| entry["label"] == "diamonds")
                    .expect("diamond label")["variants"][10];
                variant[mutation] = if mutation == "mask" {
                    json!(format!("0{}", "1".repeat(319)))
                } else {
                    json!(1)
                };
            }
            assert!(
                ReconVision::from_bytes(&serde_json::to_vec(&profile).expect("invalid derivative"))
                    .is_err(),
                "{bank}: changed {mutation}",
            );
        }
    }
}
