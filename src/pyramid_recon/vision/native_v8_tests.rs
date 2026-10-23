//! Native v8 scene regressions over unchanged historical and selected-card evidence.
//!
//! Static chart anchors replace two dynamic-text signatures. Card readers, absence
//! evidence and input authority are unchanged. These tests establish offline
//! recognition only; they neither resume the interrupted route nor supply consent.

use super::tests::{labelled_card, native_manifest, native_original, native_patch};
use super::*;
use serde_json::json;


/// Unchanged native v7 original supplying both static chart-anchor RGB crops.
const ANCHOR_SOURCE_SHA256: &str =
    "f00076f38a2906dc17d1dc5b7c54806b6285f363f936924e5c785785e9173edf";


/// Resolve retained test evidence without depending on a private runtime directory.
fn fixture_path(generation: &str, name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pyramid-recon")
        .join(generation)
        .join(name)
}


/// Decode original fixture bytes through the production native PNG decoder.
fn original(generation: &str, name: &str) -> CapturedFrame {
    crate::capture::decode_png(
        &std::fs::read(fixture_path(generation, name)).expect("retained native PNG"),
    )
    .expect("native fixture layout")
}


/// Parse a mutable JSON copy without rewriting the calibration on disk.
fn profile_value(generation: &str) -> Value {
    serde_json::from_slice(
        &std::fs::read(fixture_path(generation, "profile.json")).expect("retained profile"),
    )
    .expect("native calibration JSON")
}


/// Load retained native-v8 test bytes through the production strict parser.
fn vision() -> ReconVision {
    ReconVision::from_bytes(include_bytes!(
        "../../../tests/fixtures/pyramid-recon/native-v8/profile.json"
    ))
    .expect("retained bundled native v8 test profile")
}


/// Read the complete acquired deal through xSAR's bounded authoritative decoder.
fn acquired_tableau() -> [Card; 28] {
    let bytes = std::fs::read(fixture_path("native-v8", "deal.json"))
        .expect("retained complete acquired deal");
    let deal = xsar::pyramid_json::read_from(bytes.as_slice()).expect("complete acquired deal");
    *deal.tableau()
}


/// Preserve fixed original slots after the observed removal of 7|3 and 7|4.
fn partial_tableau() -> [Option<Card>; 28] {
    let mut expected = acquired_tableau().map(Some);
    expected[23] = None;
    expected[24] = None;
    expected
}


/// Compare the selected screenshot with its independently recorded partial position.
fn verify_selected(reader: &ReconVision, frame: &CapturedFrame) -> Result<(), String> {
    reader.verify_position(
        frame,
        &partial_tableau(),
        Some(Card::new(Rank::Four, Suit::Spades)),
        Some(Card::new(Rank::Eight, Suit::Clubs)),
    )
}


/// Paint a checked in-memory derivative while preserving alpha and native geometry.
fn fill_region(frame: &mut CapturedFrame, bounds: PixelRect, colour: [u8; 3]) {
    let _ = frame
        .as_view()
        .expect("valid native layout")
        .checked_bounds(bounds)
        .expect("bounded test derivative");
    for row in bounds.y..bounds.bottom() {
        for column in bounds.x..bounds.right() {
            let offset = row as usize * frame.stride + column as usize * 4;
            frame.pixels[offset..offset + 3].copy_from_slice(&colour);
        }
    }
}


/// Format the precise mandatory-scene failure contract for a native patch.
fn scene_error(index: usize, bounds: PixelRect) -> String {
    format!(
        "execution board scene[{index}] mismatch at [{},{},{},{}]",
        bounds.x, bounds.y, bounds.width, bounds.height
    )
}


/// Only the profile ID and two measured static scene patches differ from native v7.
#[test]
fn native_v8_exact_anchor_crops_and_rollback_preserve_every_other_profile_field() {
    let mut rollback = profile_value("native-v8");
    let prior = profile_value("native-v7");
    let source = original("native-v7", "baseline.png");
    assert_eq!(
        rollback["profile_id"],
        "pyramid-native-1920-classic-blue-v8"
    );
    assert_eq!(
        rollback["board"]["scene"].as_array().expect("scene").len(),
        3
    );
    for (index, bounds, selected) in [
        (0, PixelRect::new(1796, 116, 109, 18), 1962),
        (1, PixelRect::new(1796, 388, 109, 20), 2180),
    ] {
        let mut measured = native_patch(&source, bounds, ANCHOR_SOURCE_SHA256);
        measured["samples"]["tolerance"] = json!(4);
        assert_eq!(rollback["board"]["scene"][index], measured);
        assert_eq!(
            rollback["board"]["scene"][index]["samples"]["mask"]
                .as_str()
                .expect("full static mask")
                .bytes()
                .filter(|byte| *byte == b'1')
                .count(),
            selected
        );
        rollback["board"]["scene"][index] = prior["board"]["scene"][index].clone();
    }
    for bank in ["tableau_reader", "stock_reader"] {
        assert_eq!(rollback[bank]["rank_max_distance"], 24);
        assert_eq!(rollback[bank]["suit_max_distance"], 8);
        assert_eq!(rollback[bank]["alignment"], 1);
        assert_eq!(rollback[bank], prior[bank]);
    }
    rollback["profile_id"] = prior["profile_id"].clone();
    assert_eq!(
        rollback, prior,
        "glyph banks, arrow, geometry, Solver metadata, piles and modal remain exact v7"
    );
}


/// The actual bundle verifies all 26 remaining cards and positive absence without input.
#[test]
fn native_v8_bundled_reader_verifies_selected_nine_diamonds_partial_position() {
    let reader = vision();
    let frame = original("native-v8", "selected-9D.png");
    let pixels = frame.pixels.clone();
    let expected = partial_tableau();
    assert_eq!(expected.iter().filter(|card| card.is_some()).count(), 26);
    assert_eq!(expected[17], Some(Card::new(Rank::Nine, Suit::Diamonds)));
    verify_selected(&reader, &frame).expect("exact selected partial position");
    let file = ReconVision::load(&fixture_path("native-v8", "profile.json"))
        .expect("strict file-source v8 profile");
    assert_eq!(reader.profile_id(), file.profile_id());
    verify_selected(&file, &frame).expect("file and embedded sources agree");
    assert_eq!(frame.pixels, pixels, "observation preserves source pixels");
}


/// Selection leaves both fixed glyph crops identical to the restored unselected baseline.
#[test]
fn native_v8_selected_card_has_no_rank_or_suit_crop_displacement() {
    let selected = original("native-v8", "selected-9D.png");
    let baseline = original("native-v8", "baseline.png");
    let selected_view = selected.as_view().expect("selected native layout");
    let baseline_view = baseline.as_view().expect("baseline native layout");
    for bounds in [
        PixelRect::new(792, 384, 27, 24),
        PixelRect::new(795, 408, 20, 16),
    ] {
        for row in bounds.y..bounds.bottom() {
            for column in bounds.x..bounds.right() {
                assert_eq!(
                    selected_view.pixel_rgb(column, row),
                    baseline_view.pixel_rgb(column, row),
                    "fixed glyph pixel {column},{row}"
                );
            }
        }
    }
    let reader = vision();
    assert_eq!(
        reader
            .tableau_reader
            .recognise(selected_view, TABLEAU_ORIGINS[17])
            .expect("unique selected-card glyphs"),
        Some(Card::new(Rank::Nine, Suit::Diamonds))
    );
}


/// The unchanged v7 profile retains the exact dynamic-chart failure on this screenshot.
#[test]
fn native_v8_retains_v7_selected_frame_failure_as_historical_evidence() {
    let frame = original("native-v8", "selected-9D.png");
    let prior = ReconVision::load(&fixture_path("native-v7", "profile.json"))
        .expect("unchanged v7 rollback profile");
    assert_eq!(
        prior.observe(&frame).expect("valid unsupported v7 scene"),
        ReconScene::Unknown
    );
    assert_eq!(
        verify_selected(&prior, &frame).expect_err("retained first failing scene"),
        scene_error(0, PixelRect::new(1810, 132, 78, 40))
    );
}


/// Every historical board scene and all 104 direct glyph observations remain supported.
#[test]
fn native_v8_preserves_historical_scene_and_labelled_glyph_corpus() {
    let reader = vision();
    let manifest = native_manifest();
    let training = manifest["labelled_card_observations"]
        .as_array()
        .expect("historical direct labels");
    let endpoints = manifest["endpoint_card_observations"]
        .as_array()
        .expect("historical endpoint labels");
    let mut recognised = 0;
    let mut boards = 0;
    let mut modals = 0;
    for source in manifest["originals"].as_array().expect("native originals") {
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
            assert_eq!(
                verify_selected(&reader, &frame).expect_err("modal cannot authorise input"),
                "execution refused: matched complete Undo All dialog",
                "{id}"
            );
            modals += 1;
        } else {
            assert!(!matches_all(&reader.dialog.signature, view).expect("not a modal"));
            boards += 1;
        }
        for observation in training
            .iter()
            .chain(endpoints)
            .filter(|entry| entry["source_id"] == source["id"])
        {
            let origin = (
                integer(&observation["origin"][0]).expect("labelled x"),
                integer(&observation["origin"][1]).expect("labelled y"),
            );
            let bank = if origin == (LEFT_BOUNDS.x, LEFT_BOUNDS.y)
                || origin == (RIGHT_BOUNDS.x, RIGHT_BOUNDS.y)
            {
                &reader.stock_reader
            } else {
                &reader.tableau_reader
            };
            assert_eq!(
                bank.recognise(view, origin)
                    .expect("unique historical glyphs"),
                Some(labelled_card(observation)),
                "{}",
                observation["id"]
            );
            recognised += 1;
        }
    }
    assert_eq!(recognised, 104);
    assert_eq!(boards, 25);
    assert_eq!(modals, 4);
}


/// V6, v7, daily-original and current restored boards retain exact full observations.
#[test]
fn native_v8_preserves_full_observations_of_retained_original_boards() {
    let reader = vision();
    let prior =
        ReconVision::load(&fixture_path("native-v7", "profile.json")).expect("retained native v7");
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
            reader.observe(&frame).expect("v8 full board"),
            expected,
            "{generation}"
        );
    }
}


/// Dynamic chart text can change colour without replacing static scene or card evidence.
#[test]
fn native_v8_dynamic_chart_recolour_preserves_exact_partial_position() {
    let reader = vision();
    let frame = original("native-v8", "selected-9D.png");
    for colour in [[255, 255, 255], [11, 13, 17], [0, 210, 255]] {
        let mut changed = frame.clone();
        fill_region(&mut changed, PixelRect::new(1810, 134, 78, 254), colour);
        verify_selected(&reader, &changed)
            .expect("dynamic text supplies no static scene authority");
    }
}


/// Both static anchors and the unchanged arrow are mandatory with precise diagnostics.
#[test]
fn native_v8_erased_anchors_or_arrow_refuse_at_the_exact_scene_patch() {
    let reader = vision();
    let frame = original("native-v8", "selected-9D.png");
    for (index, pattern) in reader.board.scene.iter().enumerate() {
        let mut changed = frame.clone();
        fill_region(&mut changed, pattern.bounds, [17, 19, 21]);
        assert_eq!(
            reader.observe(&changed).expect("valid unsupported scene"),
            ReconScene::Unknown
        );
        assert_eq!(
            verify_selected(&reader, &changed).expect_err("mandatory static patch"),
            scene_error(index, pattern.bounds)
        );
    }
}


/// Static signatures retain inclusive per-channel caps with no mismatching-pixel budget.
#[test]
fn native_v8_static_scene_tolerance_boundary_remains_exact() {
    let reader = vision();
    let frame = original("native-v8", "selected-9D.png");
    for (index, pattern) in reader.board.scene.iter().enumerate() {
        let (sample, expected) = pattern
            .pixels
            .iter()
            .enumerate()
            .find_map(|(sample, value)| value.map(|rgb| (sample, rgb)))
            .expect("selected static scene pixel");
        let column = pattern.bounds.x + sample as u32 % pattern.bounds.width;
        let row = pattern.bounds.y + sample as u32 / pattern.bounds.width;
        for (extra, accepted) in [(0, true), (1, false)] {
            let mut changed = frame.clone();
            let mut colour = expected;
            let delta = pattern.tolerance + extra;
            colour[0] = if expected[0] <= 255 - delta {
                expected[0] + delta
            } else {
                expected[0] - delta
            };
            fill_region(&mut changed, PixelRect::new(column, row, 1, 1), colour);
            let result = verify_selected(&reader, &changed);
            if accepted {
                result.expect("inclusive static RGB cap");
            } else {
                assert_eq!(
                    result.expect_err("one beyond the static RGB cap"),
                    scene_error(index, pattern.bounds)
                );
            }
        }
    }
}


/// One-pixel movement of either static anchor cannot borrow dynamic-chart tolerance.
#[test]
fn native_v8_shifted_static_anchors_are_not_an_alignment_fallback() {
    let reader = vision();
    let frame = original("native-v8", "selected-9D.png");
    let view = frame.as_view().expect("selected native layout");
    for (index, pattern) in reader.board.scene.iter().enumerate().take(2) {
        let mut changed = frame.clone();
        for row in pattern.bounds.y..pattern.bounds.bottom() {
            for column in pattern.bounds.x..pattern.bounds.right() {
                let colour = view
                    .pixel_rgb(column + 1, row)
                    .expect("native adjacent pixel");
                fill_region(&mut changed, PixelRect::new(column, row, 1, 1), colour);
            }
        }
        assert_eq!(
            verify_selected(&reader, &changed).expect_err("static anchor displacement"),
            scene_error(index, pattern.bounds)
        );
    }
}


/// A supported scene does not authorise a wrong surviving identity or pile expectation.
#[test]
fn native_v8_partial_position_requires_exact_selected_card_and_both_piles() {
    let reader = vision();
    let frame = original("native-v8", "selected-9D.png");
    let mut wrong = partial_tableau();
    wrong[17] = Some(Card::new(Rank::Nine, Suit::Clubs));
    assert_eq!(
        reader
            .verify_position(
                &frame,
                &wrong,
                Some(Card::new(Rank::Four, Suit::Spades)),
                Some(Card::new(Rank::Eight, Suit::Clubs)),
            )
            .expect_err("wrong selected suit"),
        "execution tableau identity mismatch at slot 17"
    );
    for (left, right, error) in [
        (
            None,
            Some(Card::new(Rank::Eight, Suit::Clubs)),
            "execution LEFT mismatch",
        ),
        (
            Some(Card::new(Rank::Four, Suit::Spades)),
            None,
            "execution RIGHT mismatch",
        ),
    ] {
        assert_eq!(
            reader
                .verify_position(&frame, &partial_tableau(), left, right)
                .expect_err("visible pile cannot be declared empty"),
            error
        );
    }
}


/// Removed slots require positive felt, while selected and blocked cards remain occupied.
#[test]
fn native_v8_partial_position_never_infers_absence_from_unknown_or_selected_pixels() {
    let reader = vision();
    let frame = original("native-v8", "selected-9D.png");
    for (index, expected_error) in [
        (17, "execution slot 17 lacks positive absence"),
        (16, "execution slot 16 lacks positive absence"),
    ] {
        let mut expected = partial_tableau();
        expected[index] = None;
        assert_eq!(
            reader
                .verify_position(
                    &frame,
                    &expected,
                    Some(Card::new(Rank::Four, Suit::Spades)),
                    Some(Card::new(Rank::Eight, Suit::Clubs)),
                )
                .expect_err("selected or covered card is not absent"),
            expected_error
        );
    }
    for colour in [[11, 13, 17], [255, 255, 255]] {
        let mut changed = frame.clone();
        fill_region(&mut changed, PixelRect::new(730, 610, 60, 4), colour);
        assert_eq!(
            verify_selected(&reader, &changed).expect_err("removed slot needs felt"),
            "execution slot 23 lacks positive absence"
        );
    }
    let mut changed = frame;
    fill_region(&mut changed, PixelRect::new(790, 378, 40, 52), [11, 13, 17]);
    assert_eq!(
        verify_selected(&reader, &changed).expect_err("unknown selected glyph"),
        "execution tableau identity mismatch at slot 17"
    );
}


/// Real Undo All dialogs remain recognised as modals and never as executable positions.
#[test]
fn native_v8_complete_and_damaged_modals_grant_no_route_position() {
    let reader = vision();
    let modal = native_original("undo-all-confirmation");
    assert!(matches!(
        reader.observe(&modal).expect("complete native modal"),
        ReconScene::UndoConfirmation { .. }
    ));
    assert_eq!(
        verify_selected(&reader, &modal).expect_err("positive complete modal"),
        "execution refused: matched complete Undo All dialog"
    );
    for pattern in &reader.dialog.signature {
        let mut changed = modal.clone();
        fill_region(&mut changed, pattern.bounds, [127, 127, 127]);
        assert_eq!(
            reader
                .observe(&changed)
                .expect("valid damaged-modal layout"),
            ReconScene::Unknown
        );
        assert!(verify_selected(&reader, &changed).is_err());
    }
}
