//! Frozen graded-reader evaluation through production parsing and observation.
//!
//! Capture-lineage splits remain explicit: retained development/regression,
//! previously exposed validation and the prospective fifth-failure capture are
//! separate tests. Labels describe visible evidence only; none supplies a full
//! unseen stock, restoration receipt, guest effect or input authority.

use std::path::PathBuf;

use serde_json::json;

use super::tests::native_original;
use super::*;


/// Resolve immutable retained evidence relative to the package checkout.
fn fixture_path(generation: &str, name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pyramid-recon")
        .join(generation)
        .join(name)
}


/// Read a fixture JSON document without modifying its retained bytes.
fn fixture_value(generation: &str, name: &str) -> Value {
    serde_json::from_slice(&std::fs::read(fixture_path(generation, name)).expect("retained JSON"))
        .expect("well-formed fixture JSON")
}


/// Construct the actual bundled schema-2 reader through its production parser.
fn vision() -> ReconVision {
    ReconVision::bundled_native_v12().expect("strict bundled native-v12 calibration")
}


/// Parse in-memory refusal derivatives through the same bounded production API.
fn parse_profile(value: &Value) -> Result<ReconVision, String> {
    ReconVision::from_bytes(&serde_json::to_vec(value).expect("derived JSON"))
}


/// Decode one original with the production PNG decoder and retained dimensions.
fn source_frame(source: &Value) -> CapturedFrame {
    let file = source["file"].as_str().expect("relative source filename");
    let bytes = std::fs::read(fixture_path("native-v12", file)).expect("original PNG bytes");
    assert_eq!(
        bytes.len() as u64,
        source["bytes"].as_u64().expect("original length")
    );
    assert_eq!(source["original_bytes"], true);
    assert_eq!(source["transformation"], "none");
    let frame = crate::capture::decode_png(&bytes).expect("retained native PNG");
    assert_eq!((frame.width, frame.height), (1920, 1080));
    frame
}


/// Resolve a directly labelled corpus source by its explicit immutable ID.
fn source<'a>(corpus: &'a Value, id: &Value) -> &'a Value {
    corpus["sources"]
        .as_array()
        .expect("source array")
        .iter()
        .find(|entry| entry["id"] == *id)
        .expect("referenced source exists")
}


/// Decode a fixed labelled rank without consulting recognition or deck contents.
fn rank_label(value: &str) -> Rank {
    Rank::ALL
        .into_iter()
        .find(|rank| rank.as_str() == value)
        .expect("direct rank label")
}


/// Decode a full suit name independently of the reader's matching decision.
fn suit_label(value: &str) -> Suit {
    Suit::ALL
        .into_iter()
        .find(|suit| suit.as_str() == value)
        .expect("direct suit label")
}


/// Convert a direct corpus label into its expected rank-and-suit identity.
fn observation_card(observation: &Value) -> Card {
    Card::new(
        rank_label(observation["rank"].as_str().expect("direct rank")),
        suit_label(observation["suit"].as_str().expect("direct suit")),
    )
}


/// Decode conventional retained position text such as `10H`; null means known absence.
fn position_card(value: &Value) -> Option<Card> {
    let text = value.as_str()?;
    let (rank, suit) = text.split_at(text.len() - 1);
    let suit = match suit {
        "C" => Suit::Clubs,
        "D" => Suit::Diamonds,
        "H" => Suit::Hearts,
        "S" => Suit::Spades,
        _ => panic!("invalid directly labelled suit"),
    };
    Some(Card::new(rank_label(rank), suit))
}


/// Evaluate a whole immutable split in both production banks without adding samples.
fn evaluate_split(split: &str, expected: usize) {
    let corpus = fixture_value("native-v12", "corpus.json");
    let reader = vision();
    let mut recognised = 0;
    for entry in corpus["sources"].as_array().expect("source array") {
        if entry["split"] != split {
            continue;
        }
        let frame = source_frame(entry);
        let view = frame.as_view().expect("original native layout");
        for observation in corpus["observations"]
            .as_array()
            .expect("visible observations")
            .iter()
            .filter(|observation| observation["source_id"] == entry["id"])
        {
            assert_eq!(observation["source_sha256"], entry["sha256"]);
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
                        .expect("bounded production glyph evaluation"),
                    Some(observation_card(observation)),
                    "{split}: {} in {name}; {}",
                    observation["id"],
                    bank.failure_details(view, origin),
                );
            }
            recognised += 1;
        }
    }
    assert_eq!(
        recognised, expected,
        "visible labels counted once, with two bank decisions each"
    );
}


/// Retained development labels cover known templates and earlier native captures.
#[test]
fn native_v12_development_corpus_has_250_correct_visible_identities_in_both_banks() {
    evaluate_split("development", 250);
}


/// The ten failure is disclosed development regression, never prospective evidence.
#[test]
fn native_v12_ten_regression_has_29_correct_visible_identities_in_both_banks() {
    evaluate_split("development_regression", 29);
}


/// Entire previously exposed capture lineages remain outside frozen-policy tuning.
#[test]
fn native_v12_validation_corpus_has_115_correct_visible_identities_in_both_banks() {
    evaluate_split("validation_holdout", 115);
}


/// The fifth capture is exposed regression after the preserved initial-policy refusal.
#[test]
fn native_v12_exposed_fifth_regression_has_29_correct_visible_identities_in_both_banks() {
    evaluate_split("exposed_regression_fifth", 29);
}


/// The profile changes semantics explicitly while retaining every measured RGB sample.
#[test]
fn native_v12_policy_is_frozen_and_full_profile_rolls_back_to_exact_v11() {
    let policy = fixture_value("native-v12", "frozen-policy.json");
    let corpus = fixture_value("native-v12", "corpus.json");
    assert_eq!(policy["policy"], "paper_ink_l1_v2");
    let initial = fixture_value("native-v12", "initial-policy-results.json");
    assert_eq!(initial["focused_tests"]["failed"], 4);
    assert_eq!(
        initial["fifth_initial_prospective_result"]["suit"],
        "unknown"
    );
    assert_eq!(
        corpus["initial_policy_sha256"],
        "d6c8a647946ae2898cb27068a88c5314d5f2550f80048fa7976b47312871b3e1"
    );
    assert_eq!(policy["rank_max_score"], 220_000);
    assert_eq!(policy["suit_max_score"], 120_000);
    assert_eq!(policy["minimum_label_gap"], 80_000);
    assert_eq!(policy["score_scale"], 1_000_000);
    assert_eq!(
        corpus["frozen_policy_sha256"],
        "5343651cea940c25add3ee79653b2c9c9da257b30d9874143f6b400cf0d81236"
    );
    assert_eq!(corpus["sources"].as_array().expect("sources").len(), 40);
    assert_eq!(
        corpus["observations"].as_array().expect("labels").len(),
        423
    );
    let prior = fixture_value("native-v11", "profile.json");
    let mut rollback = fixture_value("native-v12", "profile.json");
    assert_eq!(rollback["schema_version"], 2);
    assert_eq!(
        rollback["profile_id"],
        "pyramid-native-1920-classic-blue-v12"
    );
    for name in ["tableau_reader", "stock_reader"] {
        assert_eq!(
            rollback[name]["matching"],
            json!({
                "policy": "paper_ink_l1_v2", "score_scale": 1_000_000,
                "rank_max_error_ppm": 220_000, "suit_max_error_ppm": 120_000,
                "min_label_gap_ppm": 80_000,
            })
        );
        let fields = rollback[name].as_object_mut().expect("reader object");
        fields.remove("matching");
        fields.insert(
            "rank_max_distance".to_owned(),
            prior[name]["rank_max_distance"].clone(),
        );
        fields.insert(
            "suit_max_distance".to_owned(),
            prior[name]["suit_max_distance"].clone(),
        );
    }
    rollback["schema_version"] = prior["schema_version"].clone();
    rollback["profile_id"] = prior["profile_id"].clone();
    assert_eq!(
        rollback, prior,
        "no RGB variants, scene, masks or geometry were changed"
    );
    let bundled = vision();
    let file = ReconVision::load(&fixture_path("native-v12", "profile.json"))
        .expect("file schema-2 reader");
    assert_eq!(file.profile_id(), bundled.profile_id());
}


/// Actual full/partial observations preserve fixed identities, positive absence and modal guards.
#[test]
fn native_v12_all_14_positions_and_40_scene_expectations_use_production_observers() {
    let corpus = fixture_value("native-v12", "corpus.json");
    let reader = vision();
    let mut boards = 0;
    let mut modals = 0;
    for entry in corpus["sources"].as_array().expect("sources") {
        let frame = source_frame(entry);
        let view = frame.as_view().expect("original layout");
        if entry["scene_expectation"] == "modal" {
            assert!(matches!(
                reader.observe(&frame).expect("complete retained modal"),
                ReconScene::UndoConfirmation { .. }
            ));
            assert!(!reader.board.matches(view).expect("dimmed board refuses"));
            modals += 1;
        } else {
            assert!(
                reader.board.matches(view).expect("full native scene"),
                "{}",
                entry["id"]
            );
            assert!(!matches_all(&reader.dialog.signature, view).expect("no modal borrowing"));
            boards += 1;
        }
    }
    assert_eq!((boards, modals), (36, 4));
    let mut positions = 0;
    let mut partial = 0;
    for position in corpus["positions"].as_array().expect("explicit positions") {
        let frame = source_frame(source(&corpus, &position["source_id"]));
        let cards: [Option<Card>; 28] = position["tableau"]
            .as_array()
            .expect("fixed 28 positions")
            .iter()
            .map(position_card)
            .collect::<Vec<_>>()
            .try_into()
            .expect("all 28 fixed slots");
        let left = position_card(&position["left"]);
        let right = position_card(&position["right"]);
        reader
            .verify_position(&frame, &cards, left, right)
            .expect("independently labelled live position");
        let removed: Vec<usize> = cards
            .iter()
            .enumerate()
            .filter_map(|(index, card)| card.is_none().then_some(index))
            .collect();
        assert_eq!(json!(removed), position["positive_empty_tableau_slots"]);
        assert_eq!(
            left.is_none(),
            position["positive_empty_left"]
                .as_bool()
                .expect("left absence flag")
        );
        assert_eq!(
            right.is_none(),
            position["positive_empty_right"]
                .as_bool()
                .expect("right absence flag")
        );
        if position["full_tableau"] == true {
            let ReconScene::Board(board) = reader.observe(&frame).expect("full native board")
            else {
                panic!("full-board corpus position")
            };
            assert_eq!(board.tableau.map(Some), cards);
            assert_eq!(board.left, left.map_or(StockFace::Empty, StockFace::Card));
            assert_eq!(board.right, right.map_or(StockFace::Empty, StockFace::Card));
        } else {
            assert_eq!(
                removed,
                vec![23, 24],
                "selected position keeps original fixed indices"
            );
            partial += 1;
        }
        positions += 1;
    }
    assert_eq!((positions, partial), (14, 1));
}


/// Schema-1 custom profiles retain the exact old ten refusal and zero-input evidence.
#[test]
fn native_v12_keeps_legacy_schema1_ten_failure_and_original_diagnostics() {
    let corpus = fixture_value("native-v12", "corpus.json");
    let frame = source_frame(source(&corpus, &json!("ten-failure-baseline")));
    let bytes = std::fs::read(fixture_path("native-v12", "ten-failure-INCOMPLETE.json"))
        .expect("original diagnostics");
    let failure: Value = serde_json::from_slice(&bytes).expect("original incomplete JSON");
    let legacy = ReconVision::load(&fixture_path("native-v11", "profile.json"))
        .expect("unchanged schema-1 reader");
    assert_eq!(
        legacy.observe(&frame).expect_err("original ten refuses"),
        failure["error"].as_str().expect("original error")
    );
    for name in [
        "draws_attempted",
        "draws_acknowledged",
        "undo_all_attempted",
        "confirm_attempted",
        "core_draws",
    ] {
        assert_eq!(failure[name], 0, "{name}");
    }
    assert_eq!(failure["captures_attempted"], 1);
    for name in [
        "restored",
        "complete",
        "complete_output_published",
        "input_delivery_uncertain",
    ] {
        assert_eq!(failure[name], false, "{name}");
    }
    assert_eq!(failure["publication_status"], "not_attempted");
    assert!(xsar::pyramid_json::read_from(bytes.as_slice()).is_err());
    assert_eq!(
        legacy.tableau_reader.policy(),
        "ink<128; XOR rank<=24, suit<=8; alignment=1px; unique labels/paper/colour required"
    );
}


/// Strict schema-2 admission rejects mixed contracts, typos and any altered frozen bound.
#[test]
fn native_v12_parser_rejects_unknown_missing_mixed_or_relaxed_policies() {
    for bank in ["tableau_reader", "stock_reader"] {
        for (key, invalid) in [
            ("policy", json!("nearest")),
            ("score_scale", json!(1_000_001)),
            ("rank_max_error_ppm", json!(220_001)),
            ("rank_max_error_ppm", json!(219_999)),
            ("suit_max_error_ppm", json!(120_001)),
            ("min_label_gap_ppm", json!(79_999)),
            ("rank_max_error_ppm", json!(220000.0)),
            ("suit_max_error_ppm", json!("120000")),
        ] {
            let mut value = fixture_value("native-v12", "profile.json");
            value[bank]["matching"][key] = invalid;
            assert!(parse_profile(&value).is_err(), "{bank} changed {key}");
        }
        for key in [
            "policy",
            "score_scale",
            "rank_max_error_ppm",
            "suit_max_error_ppm",
            "min_label_gap_ppm",
        ] {
            let mut value = fixture_value("native-v12", "profile.json");
            value[bank]["matching"]
                .as_object_mut()
                .expect("policy object")
                .remove(key);
            assert!(parse_profile(&value).is_err(), "{bank} missing {key}");
        }
        for (key, invalid) in [
            ("rank_max_distance", json!(24)),
            ("suit_max_distance", json!(8)),
            ("typo", json!(1)),
        ] {
            let mut value = fixture_value("native-v12", "profile.json");
            value[bank][key] = invalid;
            assert!(
                parse_profile(&value).is_err(),
                "{bank} unknown or legacy {key}"
            );
        }
        let mut unknown = fixture_value("native-v12", "profile.json");
        unknown[bank]["matching"]["typo"] = json!(0);
        assert!(parse_profile(&unknown).is_err());
        let mut missing = fixture_value("native-v12", "profile.json");
        missing[bank]
            .as_object_mut()
            .expect("reader object")
            .remove("matching");
        assert!(parse_profile(&missing).is_err());
    }
    let mut mixed = fixture_value("native-v12", "profile.json");
    mixed["schema_version"] = json!(1);
    assert!(parse_profile(&mixed).is_err());
    let mut mixed = fixture_value("native-v11", "profile.json");
    mixed["schema_version"] = json!(2);
    assert!(parse_profile(&mixed).is_err());
}


/// Every glyph keeps its full negative space, zero tolerance and finite variant/alignment caps.
#[test]
fn native_v12_parser_keeps_masks_variant_counts_and_geometry_bounded() {
    for bank in ["tableau_reader", "stock_reader"] {
        for group in ["ranks", "suits"] {
            for field in ["mask", "tolerance"] {
                let mut value = fixture_value("native-v12", "profile.json");
                let variant = &mut value[bank][group][0]["variants"][0];
                if field == "mask" {
                    let mask = variant["mask"].as_str().expect("full mask");
                    variant["mask"] = json!(format!("0{}", &mask[1..]));
                } else {
                    variant["tolerance"] = json!(1);
                }
                assert!(parse_profile(&value).is_err(), "{bank} {group} {field}");
            }
            let mut missing = fixture_value("native-v12", "profile.json");
            missing[bank][group]
                .as_array_mut()
                .expect("complete labels")
                .pop();
            assert!(parse_profile(&missing).is_err());
            let mut excessive = fixture_value("native-v12", "profile.json");
            let variants = excessive[bank][group][0]["variants"]
                .as_array_mut()
                .expect("variants");
            variants.resize(if group == "ranks" { 9 } else { 17 }, variants[0].clone());
            assert!(parse_profile(&excessive).is_err());
        }
        let mut alignment = fixture_value("native-v12", "profile.json");
        alignment[bank]["alignment"] = json!(2);
        assert!(parse_profile(&alignment).is_err());
    }
}


/// Alter only bounded RGB pixels while preserving dimensions, stride and alpha.
fn fill_region(frame: &mut CapturedFrame, bounds: PixelRect, colour: [u8; 3]) {
    frame
        .as_view()
        .expect("native layout")
        .checked_bounds(bounds)
        .expect("checked derivative bounds");
    for row in bounds.y..bounds.bottom() {
        for column in bounds.x..bounds.right() {
            let index = row as usize * frame.stride + column as usize * 4;
            frame.pixels[index..index + 3].copy_from_slice(&colour);
        }
    }
}


/// Unknown ranks explain their actual score/competitor and retain independent suit evidence.
#[test]
fn native_v12_unknown_diagnostics_identify_rank_suit_closest_and_refusal() {
    let corpus = fixture_value("native-v12", "corpus.json");
    let mut frame = source_frame(source(&corpus, &json!("ten-failure-baseline")));
    let reader = vision();
    let origin = TABLEAU_ORIGINS[5];
    fill_region(
        &mut frame,
        PixelRect::new(origin.0 + 1, origin.1 + 5, 29, 26),
        [255; 3],
    );
    for bank in [&reader.tableau_reader, &reader.stock_reader] {
        let view = frame.as_view().expect("blank-rank derivative");
        assert_eq!(
            bank.recognise(view, origin)
                .expect("unknown is not a fabricated label"),
            None
        );
        let explanation = bank.failure_details(view, origin);
        for detail in [
            "paper_ink_l1_v2",
            "rank:",
            "suit:",
            "closest=[",
            "other-label=[",
            "refused:",
            "error=",
            "paper-refusals=",
            "colour-refusals=",
        ] {
            assert!(
                explanation.contains(detail),
                "missing {detail} in {explanation}"
            );
        }
        assert!(
            explanation.contains("suit: paper_ink_l1_v2 accepted"),
            "{explanation}"
        );
    }
    let error = reader
        .observe(&frame)
        .expect_err("blank rank prevents complete board");
    assert!(
        error.starts_with("unknown rank/suit at tableau slot 5;"),
        "{error}"
    );
    assert!(error.contains("rank:") && error.contains("suit:") && error.contains("refused:"));
}


/// Colour contradictions cannot be repaired by a similar shape or the other glyph.
#[test]
fn native_v12_refuses_wrong_colour_without_nearest_label_fallback() {
    let corpus = fixture_value("native-v12", "corpus.json");
    let reader = vision();
    let original = source_frame(source(&corpus, &json!("ten-failure-baseline")));
    let origin = TABLEAU_ORIGINS[5];
    let mut changed = original;
    for row in origin.1 + 5..origin.1 + 31 {
        for column in origin.0 + 1..origin.0 + 30 {
            let offset = row as usize * changed.stride + column as usize * 4;
            let pixel = &mut changed.pixels[offset..offset + 3];
            let minimum = *pixel.iter().min().expect("three RGB channels");
            pixel.copy_from_slice(&[minimum; 3]);
        }
    }
    for bank in [&reader.tableau_reader, &reader.stock_reader] {
        let view = changed.as_view().expect("wrong-colour derivative");
        assert_eq!(
            bank.recognise(view, origin)
                .expect("bounded contradictory evidence"),
            None
        );
        assert!(
            bank.failure_details(view, origin)
                .contains("rank/suit colour contradiction")
        );
    }
}


/// Required scenes, positive-empty piles and complete modal components remain independent gates.
#[test]
fn native_v12_preserves_scene_pile_and_modal_refusal_guards() {
    let corpus = fixture_value("native-v12", "corpus.json");
    let reader = vision();
    let original = source_frame(source(&corpus, &json!("ten-failure-baseline")));
    for pattern in &reader.board.scene {
        let mut changed = original.clone();
        fill_region(&mut changed, pattern.bounds, [17, 19, 21]);
        assert_eq!(
            reader
                .observe(&changed)
                .expect("unsupported scene is unknown"),
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
        .expect_err("unknown RIGHT is not an empty pile");
    assert!(
        error.starts_with("RIGHT: unknown rank/suit or empty pile evidence;"),
        "{error}"
    );
    assert!(error.contains("rank:") && error.contains("suit:"));
    let modal = native_original("undo-all-confirmation");
    assert!(matches!(
        reader.observe(&modal).expect("complete retained modal"),
        ReconScene::UndoConfirmation { .. }
    ));
    for pattern in &reader.dialog.signature {
        let mut changed = modal.clone();
        fill_region(&mut changed, pattern.bounds, [127; 3]);
        assert_eq!(
            reader
                .observe(&changed)
                .expect("incomplete modal is unknown"),
            ReconScene::Unknown
        );
    }
}
