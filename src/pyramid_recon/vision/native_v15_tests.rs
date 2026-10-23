//! Regressions over actual native-v15 stock exhaustion and post-confirmation originals.
//!
//! Retained native frames establish only their visible position or modal state.
//! Explicit in-memory corruptions and mixed patches are synthetic refusal tests;
//! none establishes guest effects, a win, REDEAL authority or unseen ordinal glyphs.

use serde_json::json;

use super::*;


/// Resolve one immutable native-v15 fixture within the owning package.
fn fixture_path(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pyramid-recon/native-v15")
        .join(name)
}


/// Decode retained original PNG bytes with the production capture decoder.
fn frame(name: &str) -> CapturedFrame {
    let bytes = std::fs::read(fixture_path(name)).expect("retained native-v15 original");
    let frame = crate::capture::decode_png(&bytes).expect("original PNG layout");
    assert_eq!((frame.width, frame.height), (1920, 1080));
    frame
}


/// Parse the actual embedded calibration through its strict production entry point.
fn vision() -> ReconVision {
    ReconVision::bundled_native_v15().expect("strict measured native-v15 profile")
}


/// Read JSON evidence without changing its original bytes.
fn profile() -> Value {
    serde_json::from_slice(&std::fs::read(fixture_path("profile.json")).expect("profile bytes"))
        .expect("profile JSON")
}


/// Exercise the production bounded parser with explicitly synthetic profile mutations.
fn parse(value: &Value) -> Result<ReconVision, String> {
    ReconVision::from_bytes(&serde_json::to_vec(value).expect("serialise test profile"))
}


/// Read the original complete deal through the shared strict codec and retain only its apex.
fn apex_position() -> [Option<Card>; 28] {
    let bytes =
        std::fs::read(fixture_path("stock-empty-deal.json")).expect("retained complete deal");
    let deal =
        xsar::pyramid_json::read_from(bytes.as_slice()).expect("strict original complete deal");
    assert_eq!(deal.tableau()[0], Card::new(Rank::King, Suit::Clubs));
    let mut expected = [None; 28];
    expected[0] = Some(deal.tableau()[0]);
    expected
}


/// Paint selected reference pixels into an explicitly synthetic mixed-evidence derivative.
fn paint_pattern(frame: &mut CapturedFrame, pattern: &Pattern) {
    assert!(frame.as_view().is_ok());
    for (index, expected) in pattern.pixels.iter().enumerate() {
        if let Some(rgb) = expected {
            let column = pattern.bounds.x + index as u32 % pattern.bounds.width;
            let row = pattern.bounds.y + index as u32 / pattern.bounds.width;
            let offset = row as usize * frame.stride + column as usize * 4;
            frame.pixels[offset..offset + 3].copy_from_slice(rgb);
        }
    }
}


/// Corrupt one positively selected RGB sample without changing dimensions or neighbouring roles.
fn corrupt_pattern(frame: &mut CapturedFrame, pattern: &Pattern) {
    let (index, rgb) = pattern
        .pixels
        .iter()
        .enumerate()
        .find_map(|(index, rgb)| rgb.map(|rgb| (index, rgb)))
        .expect("bounded discriminating selection");
    let column = pattern.bounds.x + index as u32 % pattern.bounds.width;
    let row = pattern.bounds.y + index as u32 / pattern.bounds.width;
    let offset = row as usize * frame.stride + column as usize * 4;
    frame.pixels[offset] = rgb[0].wrapping_add(128);
}


/// Erase a bounded role only for explicitly synthetic negative evidence.
fn erase(frame: &mut CapturedFrame, bounds: PixelRect) {
    for row in bounds.y..bounds.bottom() {
        for column in bounds.x..bounds.right() {
            let offset = row as usize * frame.stride + column as usize * 4;
            frame.pixels[offset..offset + 3].copy_from_slice(&[11, 13, 17]);
        }
    }
}


/// The actual last-stock-MATCH frame is a legal observed partial position, never a clear.
#[test]
fn measured_empty_stock_accepts_exact_apex_and_refuses_goal_or_other_expectations() {
    let reader = vision();
    let original = frame("stock-empty.png");
    let pixels = original.pixels.clone();
    let apex = apex_position();
    ReconVision::bundled_native_v14()
        .expect("unchanged old calibration")
        .verify_position(&original, &apex, None, None)
        .expect_err("retained original reproduces the old refusal");
    reader
        .verify_position(&original, &apex, None, None)
        .expect("coupled native empty-stock evidence and exact remaining King");
    assert!(matches!(
        reader.observe(&original).expect("ordinary scene refusal"),
        ReconScene::Unknown
    ));
    let wrong_card = Card::new(Rank::Queen, Suit::Clubs);
    for (left, right) in [
        (Some(wrong_card), None),
        (None, Some(wrong_card)),
        (Some(wrong_card), Some(wrong_card)),
    ] {
        assert!(
            reader
                .verify_position(&original, &apex, left, right)
                .is_err()
        );
    }
    let mut wrong_apex = apex;
    wrong_apex[0] = Some(wrong_card);
    let error = reader
        .verify_position(&original, &wrong_apex, None, None)
        .expect_err("a different known apex cannot pass");
    assert!(error.contains("identity mismatch at slot 0"), "{error}");
    let error = reader
        .verify_position(&original, &[None; 28], None, None)
        .expect_err("visible apex prevents a false board clear");
    assert!(error.contains("slot 0 lacks positive absence"), "{error}");
    assert_eq!(
        original.pixels, pixels,
        "observation preserves retained evidence"
    );
}


/// Every coupled component, fixed anchor and absent tableau slot remains mandatory.
#[test]
fn measured_empty_stock_refuses_corrupt_and_independently_mixed_components() {
    let reader = vision();
    let original = frame("stock-empty.png");
    let apex = apex_position();
    let variant = reader
        .board
        .execution_empty_stock
        .as_ref()
        .expect("explicit coupled variant");
    for pattern in [
        &variant.scene,
        &variant.left_empty,
        &reader.board.right_empty,
        &reader.board.scene[0],
        &reader.board.scene[1],
    ] {
        let mut changed = original.clone();
        corrupt_pattern(&mut changed, pattern);
        assert!(reader.verify_position(&changed, &apex, None, None).is_err());
    }
    let mut new_left_without_positive_redeal = original.clone();
    erase(&mut new_left_without_positive_redeal, variant.scene.bounds);
    paint_pattern(
        &mut new_left_without_positive_redeal,
        &reader.board.scene[2],
    );
    assert!(
        reader
            .verify_position(&new_left_without_positive_redeal, &apex, None, None)
            .is_err()
    );
    let mut redeal_with_old_left = original.clone();
    paint_pattern(&mut redeal_with_old_left, &reader.board.left_empty);
    assert!(
        reader
            .verify_position(&redeal_with_old_left, &apex, None, None)
            .is_err()
    );
    let mut unknown_apex = original.clone();
    let (x, y) = TABLEAU_ORIGINS[0];
    erase(&mut unknown_apex, PixelRect::new(x, y, 40, 52));
    assert!(
        reader
            .verify_position(&unknown_apex, &apex, None, None)
            .is_err()
    );
    let mut unknown_absence = original;
    let (x, y) = TABLEAU_ORIGINS[27];
    erase(&mut unknown_absence, PixelRect::new(x + 40, y + 178, 60, 4));
    let evidence = crate::pyramid::tableau_evidence(&unknown_absence).unwrap();
    let leaf = crate::pyramid::PyramidTargetKind::Card { row: 7, column: 7 }
        .slot_index()
        .unwrap();
    let ancestor = crate::pyramid::PyramidTargetKind::Card { row: 2, column: 2 }
        .slot_index()
        .unwrap();
    assert_eq!(evidence[leaf], crate::pyramid::FaceEvidence::Unknown);
    assert_eq!(evidence[ancestor], crate::pyramid::FaceEvidence::Blocked);
    let error = reader
        .verify_position(&unknown_absence, &apex, None, None)
        .expect_err("unknown lower probe is not positive absence");
    // Unknown children block their ancestors; verification visits row 2 first.
    assert_eq!(error, "execution slot 2 lacks positive absence");
}


/// Three retained post-OK originals are known stale dialogs only through the explicit method.
#[test]
fn measured_post_confirmation_originals_preserve_strict_normal_observation() {
    let reader = vision();
    let target = reader.dialog.ok;
    for name in [
        "post-confirm-3.png",
        "post-confirm-4.png",
        "post-confirm-5.png",
    ] {
        let original = frame(name);
        let pixels = original.pixels.clone();
        assert_eq!(
            reader.observe(&original).expect("strict recognition"),
            ReconScene::Unknown
        );
        assert_eq!(
            reader
                .observe_after_confirmation(&original)
                .expect("known stale full modal"),
            ReconScene::UndoConfirmation { ok: target }
        );
        let error = reader
            .verify_position(&original, &apex_position(), None, None)
            .expect_err("known modal remains an execution veto");
        assert!(
            error.contains("matched complete Undo All dialog"),
            "{error}"
        );
        assert_eq!(original.pixels, pixels, "classifier is read only");
    }
    for name in ["post-confirm-start.png", "post-confirm-restored.png"] {
        let original = frame(name);
        let normal = reader
            .observe(&original)
            .expect("retained original full board");
        assert!(matches!(normal, ReconScene::Board(_)));
        assert_eq!(
            reader
                .observe_after_confirmation(&original)
                .expect("same ordinary board"),
            normal
        );
    }
    // Synthetic restoration of only the original OK role preserves the old full-modal path.
    let mut ordinary_modal = frame("post-confirm-5.png");
    paint_pattern(&mut ordinary_modal, &reader.dialog.signature[4]);
    assert_eq!(
        reader
            .observe(&ordinary_modal)
            .expect("original complete modal signature"),
        ReconScene::UndoConfirmation { ok: target }
    );
    assert_eq!(
        reader
            .observe_after_confirmation(&ordinary_modal)
            .expect("same ordinary modal"),
        ReconScene::UndoConfirmation { ok: target }
    );
}


/// No missing panel/content/control/backdrop can be repaired by the alternate OK patch.
#[test]
fn measured_post_confirmation_requires_all_seven_roles_and_rejects_ambiguity() {
    let reader = vision();
    let original = frame("post-confirm-5.png");
    for (index, pattern) in reader.dialog.signature.iter().enumerate() {
        let pattern = if index == 4 {
            reader
                .dialog
                .post_confirm_ok
                .as_ref()
                .expect("post-OK pattern")
        } else {
            pattern
        };
        let mut changed = original.clone();
        corrupt_pattern(&mut changed, pattern);
        assert_eq!(
            reader
                .observe_after_confirmation(&changed)
                .expect("bounded corrupt modal"),
            ReconScene::Unknown
        );
    }
    let mut ambiguous = original;
    for pattern in &reader.board.scene {
        paint_pattern(&mut ambiguous, pattern);
    }
    let error = reader
        .observe_after_confirmation(&ambiguous)
        .expect_err("positive board and modal cannot coexist");
    assert_eq!(error, "ambiguous recon board/dialog signatures");
    let mut bad_layout = frame("post-confirm-3.png");
    bad_layout.pixels.clear();
    assert!(reader.observe_after_confirmation(&bad_layout).is_err());
}


/// Removing the two explicit extensions reconstructs the complete unchanged v14 profile.
#[test]
fn measured_v15_retains_original_glyph_banks_and_legacy_profile_semantics() {
    let mut stripped = profile();
    stripped["board"]
        .as_object_mut()
        .expect("board")
        .remove("execution_empty_stock");
    stripped["dialog"]
        .as_object_mut()
        .expect("dialog")
        .remove("post_confirm_ok");
    let bytes = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/pyramid-recon/native-v14/profile.json"),
    )
    .expect("unchanged v14");
    let legacy: Value = serde_json::from_slice(&bytes).expect("strict v14 JSON");
    stripped["profile_id"] = legacy["profile_id"].clone();
    assert_eq!(
        stripped, legacy,
        "all original policy, banks, masks, bounds and caps stay exact"
    );
    let reader = parse(&stripped).expect("absent schema-2 extensions preserve old behaviour");
    assert!(reader.board.execution_empty_stock.is_none());
    assert!(reader.dialog.post_confirm_ok.is_none());
    let pressed = frame("post-confirm-5.png");
    assert_eq!(
        reader
            .observe_after_confirmation(&pressed)
            .expect("legacy unsupported state"),
        ReconScene::Unknown
    );
    assert!(
        reader
            .verify_position(&frame("stock-empty.png"), &apex_position(), None, None)
            .is_err()
    );
    for bank in ["tableau_reader", "stock_reader"] {
        let mut relaxed = profile();
        relaxed[bank]["matching"]["rank_max_error_ppm"] = json!(220_001);
        assert!(parse(&relaxed).is_err(), "graded policy remains frozen");
    }
}


/// Optional fields are opt-in schema-2 data; null, partial objects and unknown keys refuse.
#[test]
fn measured_optional_fields_refuse_null_malformed_and_schema_one_extensions() {
    let original = profile();
    let mut unrelated_role = original.clone();
    unrelated_role["board"]["scene"][2]["bounds"] = json!([50, 35, 60, 50]);
    let error = parse(&unrelated_role)
        .expect_err("an unrelated scene role cannot be replaced by stock evidence");
    assert!(
        error.contains("original execution stock scene role"),
        "{error}"
    );
    for (section, key) in [
        ("board", "execution_empty_stock"),
        ("dialog", "post_confirm_ok"),
    ] {
        for invalid in [Value::Null, json!({}), json!([]), json!(false)] {
            let mut changed = original.clone();
            changed[section][key] = invalid;
            assert!(
                parse(&changed).is_err(),
                "invalid optional object must not disappear"
            );
        }
        let mut extra = original.clone();
        extra[section][key]["unexpected"] = json!(true);
        assert!(parse(&extra).is_err());
        let legacy_bytes = std::fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/pyramid-recon/native-v11/profile.json"),
        )
        .expect("retained schema-1 profile");
        let mut schema_one: Value = serde_json::from_slice(&legacy_bytes).expect("legacy JSON");
        assert_eq!(schema_one["schema_version"], json!(1));
        parse(&schema_one).expect("unchanged schema-1 semantics");
        schema_one[section][key] = original[section][key].clone();
        assert!(
            parse(&schema_one).is_err(),
            "schema-1 must reject new fields"
        );
    }
}


/// New roles retain positive contrast, the stock corridor and the original role constraints.
#[test]
fn measured_optional_roles_reject_geometry_masks_tolerances_and_blank_felt() {
    let original = profile();
    let mut cases = Vec::new();
    let stock = frame("stock-empty.png");
    let source = original["board"]["execution_empty_stock"]["scene"]["samples"]["source_sha256"]
        .as_str()
        .expect("actual stock original digest");
    let mut blank_old_arrow = original.clone();
    blank_old_arrow["board"]["execution_empty_stock"]["scene"] =
        super::tests::native_patch(&stock, PixelRect::new(931, 695, 60, 50), source);
    blank_old_arrow["board"]["execution_empty_stock"]["scene"]["samples"]["tolerance"] = json!(4);
    let error = parse(&blank_old_arrow)
        .expect_err("actual blank old-arrow ROI cannot be a positive signature");
    assert!(
        error.contains("lacks discriminating RGB contrast"),
        "{error}"
    );
    for bounds in [json!([920, 678, 81, 187]), json!([931, 695, 60, 50])] {
        let mut changed = original.clone();
        changed["board"]["execution_empty_stock"]["scene"]["bounds"] = bounds;
        cases.push(changed);
    }
    for (section, key, child) in [
        ("board", "execution_empty_stock", Some("scene")),
        ("board", "execution_empty_stock", Some("left_empty")),
        ("dialog", "post_confirm_ok", None),
    ] {
        let mut blank = original.clone();
        let pattern = match child {
            Some(child) => &mut blank[section][key][child],
            None => &mut blank[section][key],
        };
        let mask = pattern["samples"]["mask"]
            .as_str()
            .expect("mask")
            .to_owned();
        pattern["samples"]["rgb"] = json!("1e5f41".repeat(mask.len()));
        cases.push(blank);
        let mut changed = original.clone();
        let pattern = match child {
            Some(child) => &mut changed[section][key][child],
            None => &mut changed[section][key],
        };
        pattern["samples"]["tolerance"] = json!(25);
        cases.push(changed);
    }
    let mut changed_mask = original.clone();
    let mask = changed_mask["dialog"]["post_confirm_ok"]["samples"]["mask"]
        .as_str()
        .expect("OK mask")
        .to_owned();
    let mut bytes = mask.into_bytes();
    let index = bytes
        .iter()
        .position(|byte| *byte == b'1')
        .expect("selected sample");
    bytes[index] = b'0';
    changed_mask["dialog"]["post_confirm_ok"]["samples"]["mask"] =
        json!(String::from_utf8(bytes).expect("binary mask"));
    cases.push(changed_mask);
    let mut changed_tolerance = original.clone();
    changed_tolerance["dialog"]["post_confirm_ok"]["samples"]["tolerance"] = json!(9);
    cases.push(changed_tolerance);
    let mut changed_left_tolerance = original.clone();
    changed_left_tolerance["board"]["execution_empty_stock"]["left_empty"]["samples"]["tolerance"] =
        json!(5);
    cases.push(changed_left_tolerance);
    let mut changed_scene_mask = original.clone();
    let mask = changed_scene_mask["board"]["execution_empty_stock"]["scene"]["samples"]["mask"]
        .as_str()
        .expect("positive full artwork mask")
        .to_owned();
    changed_scene_mask["board"]["execution_empty_stock"]["scene"]["samples"]["mask"] =
        json!(format!("0{}", &mask[1..]));
    cases.push(changed_scene_mask);
    let mut changed_ok_bounds = original.clone();
    changed_ok_bounds["dialog"]["post_confirm_ok"]["bounds"] = json!([764, 611, 65, 38]);
    cases.push(changed_ok_bounds);
    let mut overlapping_anchor = original.clone();
    overlapping_anchor["board"]["scene"][0] =
        original["board"]["execution_empty_stock"]["scene"].clone();
    let error = parse(&overlapping_anchor)
        .expect_err("positive icon cannot stand in for another scene anchor");
    assert!(
        error.contains("overlaps an unchanged scene anchor"),
        "{error}"
    );
    for changed in cases {
        assert!(parse(&changed).is_err());
    }
}
