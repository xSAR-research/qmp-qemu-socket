//! Retained final-clear regressions and refusal cases; no guest execution evidence.

use super::*;


/// Decode one unchanged retained original; names are fixed private test inputs.
fn final_frame(name: &str) -> CapturedFrame {
    let bytes = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/pyramid-recon/native-v17")
            .join(name),
    )
    .unwrap();
    crate::capture::decode_png(&bytes).unwrap()
}


/// Paint a bounded synthetic refusal derivative without changing frame layout.
fn paint(frame: &mut CapturedFrame, bounds: PixelRect, colour: [u8; 3]) {
    assert!(frame.as_view().unwrap().checked_bounds(bounds).is_ok());
    for y in bounds.y..bounds.y + bounds.height {
        for x in bounds.x..bounds.x + bounds.width {
            let index = y as usize * frame.stride + x as usize * 4;
            frame.pixels[index..index + 3].copy_from_slice(&colour);
        }
    }
}


/// Both exposed clear scenes need the complete explicit final proof, not legacy inference.
#[test]
fn measured_perfect_scenes_prove_tableau_clear_with_exact_piles_only() {
    let reader = ReconVision::bundled_native_v17().unwrap();
    let previous = ReconVision::bundled_native_v16().unwrap();
    for (name, right) in [
        (
            "final-clear-right-card.png",
            Some(Card::new(Rank::Five, Suit::Hearts)),
        ),
        ("final-clear-empty-stock.png", None),
    ] {
        let frame = final_frame(name);
        assert!(
            reader
                .verify_final_clear(&frame, &[None; 28], None, right)
                .is_ok()
        );
        assert!(
            reader
                .verify_position(&frame, &[None; 28], None, right)
                .is_err()
        );
        assert!(
            previous
                .verify_final_clear(&frame, &[None; 28], None, right)
                .is_err()
        );
        assert!(!matches!(reader.observe(&frame), Ok(ReconScene::Board(_))));
        assert!(
            reader
                .verify_final_clear(
                    &frame,
                    &[None; 28],
                    Some(Card::new(Rank::Ace, Suit::Spades)),
                    right
                )
                .is_err()
        );
        assert!(
            reader
                .verify_final_clear(
                    &frame,
                    &[None; 28],
                    None,
                    Some(Card::new(Rank::Ace, Suit::Spades))
                )
                .is_err()
        );
    }
}


/// Any replayed remaining card or unknown visible band prevents the final-only alternative.
#[test]
fn every_remaining_or_unknown_slot_refuses_final_clear() {
    let reader = ReconVision::bundled_native_v17().unwrap();
    let frame = final_frame("final-clear-empty-stock.png");
    for (index, (x, y)) in TABLEAU_ORIGINS.iter().copied().enumerate() {
        let mut expected = [None; 28];
        expected[index] = Some(Card::new(Rank::King, Suit::Clubs));
        assert!(
            reader
                .verify_final_clear(&frame, &expected, None, None)
                .is_err()
        );
        let mut damaged = frame.clone();
        paint(&mut damaged, PixelRect::new(x + 40, y + 4, 60, 4), [128; 3]);
        assert!(
            reader
                .verify_final_clear(&damaged, &[None; 28], None, None)
                .is_err(),
            "slot {index}"
        );
    }
}


/// A genuinely visible apex copied from retained stock-end evidence is never empty.
#[test]
fn retained_apex_face_prevents_perfect_overlay_from_claiming_clear() {
    let reader = ReconVision::bundled_native_v17().unwrap();
    let original = crate::capture::decode_png(include_bytes!(
        "../../../tests/fixtures/pyramid-recon/native-v15/stock-empty.png"
    ))
    .unwrap();
    let mut frame = final_frame("final-clear-empty-stock.png");
    for y in 112usize..299 {
        let start = y * frame.stride + 890 * 4;
        let source = y * original.stride + 890 * 4;
        frame.pixels[start..start + 139 * 4]
            .copy_from_slice(&original.pixels[source..source + 139 * 4]);
    }
    assert!(
        reader
            .verify_final_clear(&frame, &[None; 28], None, None)
            .unwrap_err()
            .contains("slot 0")
    );
}


/// A title, a missing card read, or an isolated scene component cannot supply clear proof.
#[test]
fn coupled_marker_empty_circle_and_scene_anchors_are_all_required() {
    let reader = ReconVision::bundled_native_v17().unwrap();
    let original = final_frame("final-clear-empty-stock.png");
    for bounds in [
        PixelRect::new(716, 468, 56, 64),
        PixelRect::new(929, 453, 48, 80),
        PixelRect::new(1165, 468, 42, 90),
        PixelRect::new(767, 712, 60, 60),
        PixelRect::new(1796, 116, 109, 18),
        PixelRect::new(1796, 388, 109, 20),
        PixelRect::new(1022, 678, 60, 60),
    ] {
        let mut damaged = original.clone();
        paint(&mut damaged, bounds, [30, 80, 20]);
        assert!(
            reader
                .verify_final_clear(&damaged, &[None; 28], None, None)
                .is_err(),
            "{bounds:?}"
        );
    }
}


/// Earlier animated captures and invalid dimensions/storage are not accepted as final clear.
#[test]
fn animated_and_invalid_frames_remain_unconfirmed() {
    let reader = ReconVision::bundled_native_v17().unwrap();
    for name in [
        "final-animation-right-card.png",
        "final-animation-empty-stock.png",
    ] {
        assert!(
            reader
                .verify_final_clear(&final_frame(name), &[None; 28], None, None)
                .is_err()
        );
    }
    let mut frame = final_frame("final-clear-empty-stock.png");
    frame.pixels.truncate(17);
    assert!(
        reader
            .verify_final_clear(&frame, &[None; 28], None, None)
            .is_err()
    );
    let mut frame = final_frame("final-clear-empty-stock.png");
    frame.width = 1919;
    assert!(
        reader
            .verify_final_clear(&frame, &[None; 28], None, None)
            .is_err()
    );
}


/// Optional calibration cannot relax masks, tolerance, cardinality, positions or schema1.
#[test]
fn final_clear_parser_rejects_weakened_or_misplaced_evidence() {
    let original: Value = serde_json::from_slice(include_bytes!(
        "../../../tests/fixtures/pyramid-recon/native-v17/profile.json"
    ))
    .unwrap();
    for change in 0..6 {
        let mut value = original.clone();
        match change {
            0 => value["board"]["final_clear"]["marker"][0]["samples"]["tolerance"] = 5.into(),
            1 => {
                value["board"]["final_clear"]["marker"][0]["samples"]["mask"] =
                    "0".repeat(3584).into()
            }
            2 => value["board"]["final_clear"]["marker"][0]["bounds"][0] = 717.into(),
            3 => value["board"]["final_clear"]["marker"]
                .as_array_mut()
                .unwrap()
                .pop()
                .map(|_| ())
                .unwrap(),
            4 => {
                value["board"]["final_clear"]["left_empty"]["samples"]["mask"] =
                    format!("0{}", "1".repeat(3599)).into()
            }
            _ => value["schema_version"] = 1.into(),
        }
        assert!(
            ReconVision::from_bytes(&serde_json::to_vec(&value).unwrap()).is_err(),
            "case {change}"
        );
    }
}


/// Removing only the explicit addition and profile ID recovers the entire decoded old contract.
#[test]
fn v17_rollback_retains_all_v16_card_scene_and_parser_inputs_exactly() {
    let mut value: Value = serde_json::from_slice(include_bytes!(
        "../../../tests/fixtures/pyramid-recon/native-v17/profile.json"
    ))
    .unwrap();
    value["profile_id"] = "pyramid-native-1920-classic-blue-v16".into();
    value["board"]
        .as_object_mut()
        .unwrap()
        .remove("final_clear");
    let previous: Value = serde_json::from_slice(include_bytes!(
        "../../../tests/fixtures/pyramid-recon/native-v16/profile.json"
    ))
    .unwrap();
    assert_eq!(value, previous);
}
