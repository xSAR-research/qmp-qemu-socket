//! Full pending-MATCH verification and fixed-slot bare-rock refusal regressions.

use super::*;


/// Decode a retained native frame without altering any pixel or metadata.
fn original(name: &str) -> CapturedFrame {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pyramid-recognition-163")
        .join(name);
    decode_png(&std::fs::read(path).unwrap()).unwrap()
}


/// Retain the sole original deal and apply the two recorded MATCH operations.
fn expected_position() -> [Option<Card>; 28] {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pyramid-recognition-163/deal.json");
    let deal = xsar::pyramid_json::read_from(std::fs::File::open(path).unwrap()).unwrap();
    let mut expected = (*deal.tableau()).map(Some);
    assert_eq!(expected[21], Some(Card::new(Rank::Five, Suit::Spades)));
    assert_eq!(expected[23], Some(Card::new(Rank::Eight, Suit::Spades)));
    assert_eq!(expected[26], Some(Card::new(Rank::Five, Suit::Hearts)));
    for index in [21, 23, 26] {
        expected[index] = None;
    }
    expected
}


/// Check every original position and both piles through the production reader.
fn verify(frame: &CapturedFrame) -> Result<(), String> {
    crate::pyramid_recon::vision::ReconVision::bundled_native_v23()
        .unwrap()
        .verify_position(
            frame,
            &expected_position(),
            Some(Card::new(Rank::Three, Suit::Clubs)),
            None,
        )
}


/// Map the one-based original bottom row into the detector's target array.
fn target_slot(column: u8) -> usize {
    crate::pyramid::PyramidTargetKind::Card { row: 7, column }
        .slot_index()
        .unwrap()
}


/// Prove the complete recorded post-MATCH state, without mutating the frame.
#[test]
fn native_163_reported_match_verifies_complete_position() {
    let frame = original("rock-effect.png");
    let unchanged = frame.pixels.clone();
    verify(&frame).unwrap();
    let vision = crate::pyramid_recon::vision::ReconVision::bundled_native_v23().unwrap();
    let occupancy = vision.native_occupancy(&frame).unwrap();
    for column in [1, 3, 6] {
        assert_eq!(
            occupancy[target_slot(column)],
            crate::pyramid::FaceEvidence::Absent
        );
    }
    assert_eq!(frame.pixels, unchanged);
}


/// Every calibrated RGB byte comes from the complete unchanged native footprint.
#[test]
fn native_163_complete_crop_is_byte_original() {
    let frame = original("rock-effect.png");
    let view = frame.as_view().unwrap();
    let bytes: Vec<u8> = (588..622)
        .flat_map(|y| (1289..1433).flat_map(move |x| view.pixel_rgb(x, y).unwrap()))
        .collect();
    assert_eq!(bytes.len(), 14_688);
    assert_eq!(
        bytes.as_slice(),
        include_bytes!(
            "../../../../../tests/fixtures/pyramid-recognition-163/underwater-rock-clearance.rgb"
        )
    );
}


/// The real surviving 5H lower footprint cannot use the bare-rock proof.
#[test]
fn native_163_actual_unremoved_card_refuses() {
    let before = original("before-match.png");
    let view = before.as_view().unwrap();
    let mut frame = original("rock-effect.png");
    for y in 588..622 {
        for x in 1289..1433 {
            fill(
                &mut frame,
                PixelRect::new(x, y, 1, 1),
                view.pixel_rgb(x, y).unwrap(),
            );
        }
    }
    assert!(verify(&frame).is_err());
    let vision = crate::pyramid_recon::vision::ReconVision::bundled_native_v23().unwrap();
    assert_ne!(
        vision.native_occupancy(&frame).unwrap()[target_slot(6)],
        crate::pyramid::FaceEvidence::Absent
    );
}


/// Strong-white, aged, warm, grey and black material cannot be masked by the proof.
#[test]
fn native_163_material_overlays_refuse() {
    for colour in [[250; 3], [211, 205, 185], [245, 220, 150], [185; 3], [0; 3]] {
        for region in [
            PixelRect::new(1331, 610, 60, 4),
            PixelRect::new(1331, 610, 10, 2),
        ] {
            let mut frame = original("rock-effect.png");
            fill(&mut frame, region, colour);
            assert!(verify(&frame).is_err(), "{colour:?}, {region:?}");
        }
    }
}


/// Moving the observed artwork by one native pixel cannot use exact calibration.
#[test]
fn native_163_shifted_artwork_refuses() {
    let source = original("rock-effect.png");
    let view = source.as_view().unwrap();
    for (x, y) in [(1288, 588), (1290, 588), (1289, 587), (1289, 589)] {
        let mut frame = source.clone();
        for dy in 0..34 {
            for dx in 0..144 {
                fill(
                    &mut frame,
                    PixelRect::new(1289 + dx, 588 + dy, 1, 1),
                    view.pixel_rgb(x + dx, y + dy).unwrap(),
                );
            }
        }
        assert!(verify(&frame).is_err(), "{x},{y}");
    }
}


/// One changed RGB channel at any sampled edge or centre has zero tolerance.
#[test]
fn native_163_single_channel_changes_refuse() {
    for (x, y) in [
        (1289, 588),
        (1432, 588),
        (1289, 621),
        (1432, 621),
        (1361, 605),
    ] {
        for channel in 0..3 {
            let mut frame = original("rock-effect.png");
            let mut rgb = frame.as_view().unwrap().pixel_rgb(x, y).unwrap();
            rgb[channel] = rgb[channel].checked_add(1).unwrap();
            fill(&mut frame, PixelRect::new(x, y, 1, 1), rgb);
            assert!(verify(&frame).is_err(), "{x},{y} channel {channel}");
        }
    }
}


/// Transplanting the complete calibration does not authorise a different slot.
#[test]
fn native_163_calibration_is_fixed_to_original_seven_six() {
    let source = original("rock-effect.png");
    let view = source.as_view().unwrap();
    let vision = crate::pyramid_recon::vision::ReconVision::bundled_native_v23().unwrap();
    for column in [1, 2, 3, 4, 5, 7] {
        let target = crate::pyramid::PYRAMID_TARGETS[target_slot(column)];
        let x = target.bounds.x - 2;
        let y = target.bounds.y + 156;
        let mut frame = source.clone();
        for dy in 0..34 {
            for dx in 0..144 {
                fill(
                    &mut frame,
                    PixelRect::new(x + dx, y + dy, 1, 1),
                    view.pixel_rgb(1289 + dx, 588 + dy).unwrap(),
                );
            }
        }
        assert_ne!(
            vision.native_occupancy(&frame).unwrap()[target_slot(column)],
            crate::pyramid::FaceEvidence::Absent
        );
    }
}


/// A local absence proof cannot excuse a wrong remaining identity or pile.
#[test]
fn native_163_identities_and_both_piles_remain_required() {
    let frame = original("rock-effect.png");
    let vision = crate::pyramid_recon::vision::ReconVision::bundled_native_v23().unwrap();
    let expected = expected_position();
    for (left, right) in [
        (Some(Card::new(Rank::Nine, Suit::Clubs)), None),
        (
            Some(Card::new(Rank::Three, Suit::Clubs)),
            Some(Card::new(Rank::Ace, Suit::Hearts)),
        ),
    ] {
        assert!(
            vision
                .verify_position(&frame, &expected, left, right)
                .is_err()
        );
    }
    let mut wrong = expected;
    wrong[27] = Some(Card::new(Rank::Queen, Suit::Diamonds));
    assert!(
        vision
            .verify_position(
                &frame,
                &wrong,
                Some(Card::new(Rank::Three, Suit::Clubs)),
                None
            )
            .is_err()
    );
}


/// A complete supported scene remains mandatory around the exact local crop.
#[test]
fn native_163_unsupported_scene_refuses() {
    let mut frame = original("rock-effect.png");
    fill(
        &mut frame,
        crate::pyramid::WHITE_SCENE_PROBES[0],
        [17, 19, 21],
    );
    assert!(verify(&frame).is_err());
}


/// Malformed dimensions, stride and truncated storage cannot become absence.
#[test]
fn native_163_invalid_frame_layout_refuses() {
    let mut frame = original("rock-effect.png");
    frame.width = 1919;
    assert!(verify(&frame).is_err());
    let mut frame = original("rock-effect.png");
    frame.stride = 1;
    assert!(verify(&frame).is_err());
    let mut frame = original("rock-effect.png");
    frame.pixels.truncate(100);
    assert!(verify(&frame).is_err());
}
