//! Actual131 challenge control variants, independent position truth and refusal checks.

use super::*;


/// Read one unchanged native final frame, with no synthetic recognition inputs.
fn challenge_frame(name: &str) -> CapturedFrame {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pyramid-challenge-redeal")
        .join(format!("{name}.png"));
    crate::capture::decode_png(&std::fs::read(path).unwrap()).unwrap()
}


/// Return the independently visually recorded fixed tableau slots and RIGHT card.
fn challenge_position(name: &str) -> ([Option<Card>; 28], Card) {
    let (slots, right): (&[(usize, &str)], &str) = match name {
        "jacks" => (
            &[
                (0, "5S"),
                (1, "5H"),
                (2, "4D"),
                (4, "AC"),
                (5, "7S"),
                (8, "KH"),
                (9, "5D"),
                (13, "8H"),
                (14, "QS"),
                (19, "6C"),
            ],
            "9S",
        ),
        "eights" => (
            &[
                (0, "6C"),
                (2, "9S"),
                (5, "3H"),
                (9, "8C"),
                (14, "9H"),
                (20, "7D"),
            ],
            "4S",
        ),
        _ => panic!("named actual fixture"),
    };
    let mut tableau = [None; 28];
    for &(slot, label) in slots {
        tableau[slot] = Some(card(label));
    }
    (tableau, card(right))
}


/// Both real failures require added whole variants and prove the exact same-frame position.
#[test]
fn actual_challenge_redeal_variants_prove_both_reported_positions() {
    for name in ["jacks", "eights"] {
        let frame = challenge_frame(name);
        let (tableau, right) = challenge_position(name);
        let mut reader = ReconVision::bundled_native_v22().unwrap();
        assert!(
            reader
                .verify_position(&frame, &tableau, None, Some(right))
                .is_err()
        );
        assert_eq!(reader.remaining_native_redeals(&frame).unwrap(), Some(1));
        assert_eq!(
            reader
                .verify_native_redeal_position(&frame, &tableau, Some(right))
                .unwrap(),
            PixelPoint::new(960, 825)
        );
        let native = reader.native_redeal.as_mut().unwrap();
        native.enabled[1] = native.enabled[0].clone();
        native.enabled[2] = native.enabled[0].clone();
        native.one[2] = native.one[0].clone();
        native.one[3] = native.one[0].clone();
        assert_eq!(reader.remaining_native_redeals(&frame).unwrap(), None);
        let error = reader
            .verify_native_redeal_position(&frame, &tableau, Some(right))
            .unwrap_err();
        assert!(error.contains("enabled icon=false"), "{error}");
        assert!(error.contains("one-deal caption=false"), "{error}");
    }
}


/// Every added variant retains all position/static/X/control and custom-profile refusals.
#[test]
fn actual_challenge_redeal_variants_refuse_missing_or_wrong_evidence() {
    for name in ["jacks", "eights"] {
        let frame = challenge_frame(name);
        let (tableau, right) = challenge_position(name);
        let reader = ReconVision::bundled_native_v22().unwrap();
        for (x, y) in [(936, 792), (890, 874), (767, 712), (1796, 116), (1796, 388)] {
            let mut changed = challenge_frame(name);
            let offset = y * changed.stride + x * 4;
            for channel in &mut changed.pixels[offset..offset + 3] {
                *channel ^= 255;
            }
            assert!(
                reader
                    .verify_native_redeal_position(&changed, &tableau, Some(right))
                    .is_err()
            );
        }
        let mut wrong = tableau;
        wrong[0] = None;
        assert!(
            reader
                .verify_native_redeal_position(&frame, &wrong, Some(right))
                .is_err()
        );
        assert!(
            reader
                .verify_native_redeal_position(&frame, &tableau, Some(card("AS")))
                .is_err()
        );
        assert!(
            reader
                .verify_native_stock_continuation(&frame, &tableau, Some(right), Some(right))
                .is_err()
        );
        let zero = super::frame("after.png");
        let mut changed = challenge_frame(name);
        for y in 874..900 {
            let begin = y * changed.stride + 890 * 4;
            let end = begin + 140 * 4;
            changed.pixels[begin..end].copy_from_slice(&zero.pixels[begin..end]);
        }
        assert_eq!(reader.remaining_native_redeals(&changed).unwrap(), Some(0));
        let error = reader
            .verify_native_redeal_position(&changed, &tableau, Some(right))
            .unwrap_err();
        assert!(error.contains("zero-deal caption=true"), "{error}");
        let custom = ReconVision::from_bytes(include_bytes!(
            "../../../../../tests/fixtures/pyramid-recon/native-v22/profile.json"
        ))
        .unwrap();
        assert!(
            custom
                .verify_native_redeal_position(&frame, &tableau, Some(right))
                .is_err()
        );
    }
}


/// Per-pixel mixtures of measured alternatives cannot replace a complete matched pattern.
#[test]
fn challenge_redeal_matching_requires_one_complete_variant() {
    let frame = challenge_frame("eights");
    let view = frame.as_view().unwrap();
    let base = ReconVision::bundled_native_v22()
        .unwrap()
        .native_redeal
        .unwrap()
        .enabled[2]
        .clone();
    let mut candidates = [base.clone(), base.clone(), base];
    for (index, pattern) in candidates.iter_mut().enumerate() {
        let pixel = pattern.pixels[index].as_mut().unwrap();
        for channel in pixel {
            *channel ^= 255;
        }
    }
    assert!(!matches_any(&candidates, view).unwrap());
}
