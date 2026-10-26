//! Actual normal final-clear regressions; repeated or altered frames are offline evidence.

use super::*;


/// Decode one fixed retained original without touching its immutable bytes.
fn frame(name: &str) -> CapturedFrame {
    let bytes = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/pyramid-recon/native-v18")
            .join(name),
    )
    .unwrap();
    crate::capture::decode_png(&bytes).unwrap()
}


/// Paint a bounded synthetic refusal derivative in memory, retaining native geometry.
fn paint(frame: &mut CapturedFrame, bounds: PixelRect, colour: [u8; 3]) {
    assert!(frame.as_view().unwrap().checked_bounds(bounds).is_ok());
    for y in bounds.y..bounds.y + bounds.height {
        for x in bounds.x..bounds.x + bounds.width {
            let offset = y as usize * frame.stride + x as usize * 4;
            frame.pixels[offset..offset + 3].copy_from_slice(&colour);
        }
    }
}


/// The three actual normal ordinals need exact positive piles and the new opt-in.
#[test]
fn normal_perfect_originals_require_exact_piles_and_do_not_become_starting_boards() {
    let new = ReconVision::bundled_native_v18().unwrap();
    let previous = ReconVision::bundled_native_v17().unwrap();
    for (ordinal, name, left, right) in [
        (
            0,
            "normal-one-perfect.png",
            Some(Card::new(Rank::Three, Suit::Hearts)),
            Some(Card::new(Rank::Nine, Suit::Clubs)),
        ),
        (1, "normal-two-perfect.png", None, None),
        (2, "normal-three-perfect.png", None, None),
    ] {
        let frame = frame(name);
        assert_eq!(
            crate::pyramid_progress::observe_normal(&frame)
                .unwrap()
                .unwrap()
                .completed,
            ordinal
        );
        assert!(
            new.verify_normal_final_clear(&frame, &[None; 28], left, right)
                .is_ok(),
            "{name}"
        );
        assert!(
            previous
                .verify_normal_final_clear(&frame, &[None; 28], left, right)
                .is_err()
        );
        assert!(
            new.verify_position(&frame, &[None; 28], left, right)
                .is_err()
        );
        assert!(
            new.verify_final_clear(&frame, &[None; 28], left, right)
                .is_err()
        );
        assert!(!matches!(new.observe(&frame), Ok(ReconScene::Board(_))));
        assert!(
            new.verify_normal_final_clear(
                &frame,
                &[None; 28],
                Some(Card::new(Rank::Ace, Suit::Spades)),
                right
            )
            .is_err()
        );
        assert!(
            new.verify_normal_final_clear(
                &frame,
                &[None; 28],
                left,
                Some(Card::new(Rank::Ace, Suit::Spades))
            )
            .is_err()
        );
    }
}


/// Any replayed remaining card or a single unexplained band pixel prevents clear proof.
#[test]
fn all_twenty_eight_positive_absence_bands_and_replayed_empty_slots_are_required() {
    let reader = ReconVision::bundled_native_v18().unwrap();
    let original = frame("normal-three-perfect.png");
    for (index, (x, y)) in TABLEAU_ORIGINS.iter().copied().enumerate() {
        let mut expected = [None; 28];
        expected[index] = Some(Card::new(Rank::King, Suit::Clubs));
        assert!(
            reader
                .verify_normal_final_clear(&original, &expected, None, None)
                .is_err()
        );
        let mut changed = original.clone();
        let y = y + if index < 21 { 4 } else { 164 };
        paint(&mut changed, PixelRect::new(x + 40, y, 1, 1), [255; 3]);
        assert!(
            reader
                .verify_normal_final_clear(&changed, &[None; 28], None, None)
                .unwrap_err()
                .contains(&format!("slot {index}"))
        );
    }
}


/// Header/footer, independent scene anchors and empty-pile decoration remain mandatory.
#[test]
fn erasing_identity_scene_or_either_pile_refuses_normal_clear() {
    let reader = ReconVision::bundled_native_v18().unwrap();
    let original = frame("normal-three-perfect.png");
    for bounds in [
        PixelRect::new(1658, 46, 160, 32),
        PixelRect::new(212, 998, 58, 31),
        PixelRect::new(1796, 116, 109, 18),
        PixelRect::new(1796, 388, 109, 20),
        PixelRect::new(767, 712, 60, 60),
        PixelRect::new(1022, 678, 60, 60),
    ] {
        let mut changed = original.clone();
        paint(&mut changed, bounds, [30, 80, 20]);
        assert!(
            reader
                .verify_normal_final_clear(&changed, &[None; 28], None, None)
                .is_err(),
            "{bounds:?}"
        );
    }
    let mut malformed = original.clone();
    malformed.pixels.truncate(17);
    assert!(
        reader
            .verify_normal_final_clear(&malformed, &[None; 28], None, None)
            .is_err()
    );
    let mut resized = original;
    resized.width = 1919;
    assert!(
        reader
            .verify_normal_final_clear(&resized, &[None; 28], None, None)
            .is_err()
    );
}


/// A genuine remaining tableau or a challenge Perfect is not this normal empty-board family.
#[test]
fn starting_boards_and_challenge_finals_do_not_supply_normal_clear() {
    let reader = ReconVision::bundled_native_v18().unwrap();
    for bytes in [
        include_bytes!("../../../tests/fixtures/pyramid-objectives/normal-grandmaster-refused.png")
            .as_slice(),
        include_bytes!(
            "../../../tests/fixtures/pyramid-recon/native-v17/final-clear-empty-stock.png"
        )
        .as_slice(),
        include_bytes!("../../../tests/fixtures/pyramid-objectives/fours-complete.png").as_slice(),
        include_bytes!("../../../tests/fixtures/pyramid-objectives/boards-one-of-two-perfect.png")
            .as_slice(),
    ] {
        let frame = crate::capture::decode_png(bytes).unwrap();
        assert!(
            reader
                .verify_normal_final_clear(&frame, &[None; 28], None, None)
                .is_err()
        );
    }
}


/// The actual first post-ACK failures still contain unexplained white pixels in a band.
#[test]
fn retained_flying_cards_are_not_normal_clear() {
    let reader = ReconVision::bundled_native_v18().unwrap();
    for (name, left, right) in [
        (
            "normal-one-flying.png",
            Some(Card::new(Rank::Three, Suit::Hearts)),
            Some(Card::new(Rank::Nine, Suit::Clubs)),
        ),
        ("normal-two-flying.png", None, None),
        ("normal-three-flying.png", None, None),
    ] {
        assert!(
            reader
                .verify_normal_final_clear(&frame(name), &[None; 28], left, right)
                .is_err(),
            "{name}"
        );
    }
}


/// Opt-in is schema-2 only and cannot weaken the native circle/cross patterns.
#[test]
fn parser_rejects_weakened_normal_empty_patterns_or_old_schema() {
    let original: Value = serde_json::from_slice(include_bytes!(
        "../../../tests/fixtures/pyramid-recon/native-v18/profile.json"
    ))
    .unwrap();
    for key in ["left_circle", "left_exhausted"] {
        for change in 0..3 {
            let mut value = original.clone();
            let pattern = &mut value["board"]["normal_final_clear"][key];
            match change {
                0 => pattern["samples"]["tolerance"] = 5.into(),
                1 => pattern["samples"]["mask"] = format!("0{}", "1".repeat(3599)).into(),
                _ => pattern["bounds"][0] = 768.into(),
            }
            assert!(ReconVision::from_bytes(&serde_json::to_vec(&value).unwrap()).is_err());
        }
    }
    let mut value = original;
    value["schema_version"] = 1.into();
    assert!(ReconVision::from_bytes(&serde_json::to_vec(&value).unwrap()).is_err());
}


/// Removing the explicit opt-in and identity recovers every v17 calibration value.
#[test]
fn v18_rollback_retains_all_v17_card_scene_and_parser_inputs() {
    let mut value: Value = serde_json::from_slice(include_bytes!(
        "../../../tests/fixtures/pyramid-recon/native-v18/profile.json"
    ))
    .unwrap();
    value["profile_id"] = "pyramid-native-1920-classic-blue-v17".into();
    value["board"]
        .as_object_mut()
        .unwrap()
        .remove("normal_final_clear");
    let previous: Value = serde_json::from_slice(include_bytes!(
        "../../../tests/fixtures/pyramid-recon/native-v17/profile.json"
    ))
    .unwrap();
    assert_eq!(value, previous);
}


/// Decode the unchanged actual third-board timeout image; its route was not recorded.
fn normal_final_152() -> CapturedFrame {
    crate::capture::decode_png(include_bytes!(
        "../../../tests/fixtures/pyramid-normal-final-152/last-observed.png"
    ))
    .unwrap()
}


/// Actual glow may block the old lower layout while a whole upper layout proves absence.
#[test]
fn native_152_normal_third_clear_recognises_actual_empty_tableau_and_piles() {
    let frame = normal_final_152();
    let reader = ReconVision::bundled_native_v23().unwrap();
    let left = Some(Card::new(Rank::Six, Suit::Clubs));
    let right = Some(Card::new(Rank::Two, Suit::Spades));
    assert_eq!(
        crate::pyramid_progress::observe_normal(&frame).unwrap(),
        Some(crate::pyramid_progress::BoardCount {
            completed: 2,
            target: 3
        })
    );
    let view = frame.as_view().unwrap();
    assert!(
        !reader
            .native_absence_band(view, PixelRect::new(1131, 596, 60, 4))
            .unwrap()
    );
    assert!(
        !reader
            .native_clear_band_material_veto(view, PixelRect::new(1131, 596, 60, 4))
            .unwrap()
    );
    for (x, y) in TABLEAU_ORIGINS {
        assert!(
            reader
                .native_absence_band(view, PixelRect::new(x + 40, y + 4, 60, 4))
                .unwrap()
        );
    }
    assert!(
        reader
            .verify_position(&frame, &[None; 28], left, right)
            .is_err()
    );
    assert!(
        reader
            .verify_final_clear(&frame, &[None; 28], left, right)
            .is_err()
    );
    reader
        .verify_normal_final_clear(&frame, &[None; 28], left, right)
        .unwrap();
    assert!(!matches!(reader.observe(&frame), Ok(ReconScene::Board(_))));
    assert!(
        ReconVision::bundled_native_v18()
            .unwrap()
            .verify_normal_final_clear(&frame, &[None; 28], left, right)
            .is_err()
    );
    let custom = ReconVision::from_bytes(include_bytes!(
        "../../../tests/fixtures/pyramid-recon/native-v18/profile.json"
    ))
    .unwrap();
    assert!(!custom.colour_independent);
    assert!(
        custom
            .verify_normal_final_clear(&frame, &[None; 28], left, right)
            .is_err()
    );
}


/// A remaining expected card, any obstructed upper band or mixed layouts cannot prove clear.
#[test]
fn native_152_normal_clear_requires_one_coherent_complete_layout() {
    let original = normal_final_152();
    let reader = ReconVision::bundled_native_v23().unwrap();
    let left = Some(Card::new(Rank::Six, Suit::Clubs));
    let right = Some(Card::new(Rank::Two, Suit::Spades));
    for (index, (x, y)) in TABLEAU_ORIGINS.into_iter().enumerate() {
        let mut expected = [None; 28];
        expected[index] = Some(Card::new(Rank::King, Suit::Clubs));
        assert!(
            reader
                .verify_normal_final_clear(&original, &expected, left, right)
                .is_err()
        );
        let mut changed = original.clone();
        paint(&mut changed, PixelRect::new(x + 40, y + 4, 60, 4), [255; 3]);
        assert!(
            reader
                .verify_normal_final_clear(&changed, &[None; 28], left, right)
                .is_err(),
            "upper band {index}"
        );
    }
    let mut mixed = original;
    let (x, y) = TABLEAU_ORIGINS[21];
    paint(&mut mixed, PixelRect::new(x + 40, y + 4, 60, 4), [255; 3]);
    let view = mixed.as_view().unwrap();
    assert!(
        reader
            .native_absence_band(view, PixelRect::new(x + 40, y + 164, 60, 4))
            .unwrap()
    );
    assert!(
        reader
            .native_absence_band(view, PixelRect::new(1131, 436, 60, 4))
            .unwrap()
    );
    assert!(
        reader
            .verify_normal_final_clear(&mixed, &[None; 28], left, right)
            .is_err()
    );
}


/// Retained flying cards still veto the new upper alternative through lower card material.
#[test]
fn native_152_normal_clear_retains_all_three_flying_card_refusals() {
    let reader = ReconVision::bundled_native_v23().unwrap();
    for (name, left, right) in [
        (
            "normal-one-flying.png",
            Some(Card::new(Rank::Three, Suit::Hearts)),
            Some(Card::new(Rank::Nine, Suit::Clubs)),
        ),
        ("normal-two-flying.png", None, None),
        ("normal-three-flying.png", None, None),
    ] {
        let frame = frame(name);
        let view = frame.as_view().unwrap();
        assert!(TABLEAU_ORIGINS.into_iter().all(|(x, y)| {
            reader
                .native_absence_band(view, PixelRect::new(x + 40, y + 4, 60, 4))
                .unwrap()
        }));
        assert!(
            TABLEAU_ORIGINS
                .into_iter()
                .enumerate()
                .any(|(index, (x, y))| reader
                    .native_clear_band_material_veto(
                        view,
                        PixelRect::new(x + 40, y + if index < 21 { 4 } else { 164 }, 60, 4)
                    )
                    .unwrap())
        );
        assert!(
            reader
                .verify_normal_final_clear(&frame, &[None; 28], left, right)
                .is_err(),
            "{name}"
        );
    }
}


/// New final geometry preserves positive identity/scene/piles and checked frame admission.
#[test]
fn native_152_normal_clear_keeps_scene_header_piles_and_frame_refusals() {
    let reader = ReconVision::bundled_native_v23().unwrap();
    let original = normal_final_152();
    let left = Some(Card::new(Rank::Six, Suit::Clubs));
    let right = Some(Card::new(Rank::Two, Suit::Spades));
    for (left, right) in [
        (None, right),
        (left, None),
        (Some(Card::new(Rank::King, Suit::Clubs)), right),
        (left, Some(Card::new(Rank::King, Suit::Spades))),
    ] {
        assert!(
            reader
                .verify_normal_final_clear(&original, &[None; 28], left, right)
                .is_err()
        );
    }
    for bounds in [
        PixelRect::new(1658, 46, 160, 32),
        PixelRect::new(212, 998, 58, 31),
        PixelRect::new(1796, 116, 109, 18),
        PixelRect::new(1796, 388, 109, 20),
        PixelRect::new(931, 695, 60, 50),
    ] {
        let mut changed = original.clone();
        paint(&mut changed, bounds, [255; 3]);
        assert!(
            reader
                .verify_normal_final_clear(&changed, &[None; 28], left, right)
                .is_err(),
            "{bounds:?}"
        );
    }
    let mut malformed = original.clone();
    malformed.pixels.truncate(17);
    assert!(
        reader
            .verify_normal_final_clear(&malformed, &[None; 28], left, right)
            .is_err()
    );
    let mut resized = original;
    resized.width = 1919;
    assert!(
        reader
            .verify_normal_final_clear(&resized, &[None; 28], left, right)
            .is_err()
    );
}
