//! Third retained coral rendering and refusals through the complete native reader.

use super::*;


/// Decode the original final observation from the stopped 160 request, without input.
fn retained() -> CapturedFrame {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pyramid-recognition-161/coral-effect.png");
    decode_png(&std::fs::read(path).unwrap()).unwrap()
}


/// Read this request's original complete deal and apply only its pending first MATCH.
fn expected_position() -> [Option<Card>; 28] {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pyramid-recognition-161/deal.json");
    let deal = xsar::pyramid_json::read_from(std::fs::File::open(path).unwrap()).unwrap();
    let mut expected = (*deal.tableau()).map(Some);
    assert_eq!(expected[21], Some(Card::new(Rank::Five, Suit::Spades)));
    assert_eq!(expected[23], Some(Card::new(Rank::Eight, Suit::Spades)));
    expected[21] = None;
    expected[23] = None;
    expected
}


/// Every remaining identity and both piles must agree after the two removed faces.
#[test]
fn native_161_third_coral_rendering_verifies_complete_position() {
    let frame = retained();
    let vision = crate::pyramid_recon::vision::ReconVision::bundled_native_v23().unwrap();
    vision
        .verify_position(
            &frame,
            &expected_position(),
            Some(Card::new(Rank::Eight, Suit::Clubs)),
            None,
        )
        .unwrap();
    let occupancy = vision.native_occupancy(&frame).unwrap();
    for column in [1, 3] {
        let slot = crate::pyramid::PyramidTargetKind::Card { row: 7, column }
            .slot_index()
            .unwrap();
        assert_eq!(occupancy[slot], crate::pyramid::FaceEvidence::Absent);
    }
}


/// All 4,896 native pixels are retained in RGB order without scaling or masking.
#[test]
fn native_161_coral_crop_is_complete_original_rgb() {
    let frame = retained();
    let view = frame.as_view().unwrap();
    let expected = include_bytes!(
        "../../../../../tests/fixtures/pyramid-recognition-161/underwater-coral-clearance-third.rgb"
    );
    let actual: Vec<u8> = (588..622)
        .flat_map(|y| (286..430).flat_map(move |x| view.pixel_rgb(x, y).unwrap()))
        .collect();
    assert_eq!(actual.as_slice(), expected);
}


/// Structural pixel damage refuses under the approved 162 footprint contract.
/// The historical exact-channel boundary requirement is explicitly superseded.
#[test]
fn native_161_coral_rendering_refuses_structural_pixel_damage() {
    let vision = crate::pyramid_recon::vision::ReconVision::bundled_native_v23().unwrap();
    let expected = expected_position();
    let view_frame = retained();
    let view = view_frame.as_view().unwrap();
    for (x, y, channel, value) in [
        (287, 588, 0, 0),
        (287, 588, 0, 255),
        (298, 588, 0, 0),
        (298, 588, 0, 255),
    ] {
        let mut changed = retained();
        let mut rgb = view.pixel_rgb(x, y).unwrap();
        rgb[channel] = value;
        fill(&mut changed, PixelRect::new(x, y, 1, 1), rgb);
        assert!(
            vision
                .verify_position(
                    &changed,
                    &expected,
                    Some(Card::new(Rank::Eight, Suit::Clubs)),
                    None,
                )
                .is_err(),
            "structural damage at {x},{y} must not borrow the calibrated footprint"
        );
    }
}


/// White, aged and possible card material cannot borrow the bare-coral witness.
#[test]
fn native_161_coral_rendering_refuses_card_material() {
    let vision = crate::pyramid_recon::vision::ReconVision::bundled_native_v23().unwrap();
    let expected = expected_position();
    for rgb in [[250; 3], [211, 205, 185], [185; 3]] {
        let mut frame = retained();
        fill(&mut frame, PixelRect::new(328, 610, 60, 4), rgb);
        assert_ne!(
            vision.native_occupancy(&frame).unwrap()[21],
            crate::pyramid::FaceEvidence::Absent
        );
        assert!(
            vision
                .verify_position(
                    &frame,
                    &expected,
                    Some(Card::new(Rank::Eight, Suit::Clubs)),
                    None,
                )
                .is_err()
        );
    }
}


/// This local absence proof cannot hide a wrong remaining identity or either pile.
#[test]
fn native_161_coral_rendering_retains_exact_identity_and_pile_checks() {
    let vision = crate::pyramid_recon::vision::ReconVision::bundled_native_v23().unwrap();
    let frame = retained();
    let expected = expected_position();
    let left = Some(Card::new(Rank::Eight, Suit::Clubs));
    for (left, right) in [
        (Some(Card::new(Rank::Nine, Suit::Clubs)), None),
        (left, Some(Card::new(Rank::Ace, Suit::Hearts))),
    ] {
        assert!(
            vision
                .verify_position(&frame, &expected, left, right)
                .is_err()
        );
    }
    let mut wrong_identity = expected;
    wrong_identity[0] = Some(Card::new(Rank::Queen, Suit::Spades));
    assert!(
        vision
            .verify_position(&frame, &wrong_identity, left, None)
            .is_err()
    );
    let mut unreadable = frame;
    let origin = TABLEAU_ORIGINS[0];
    fill(
        &mut unreadable,
        PixelRect::new(origin.0 + 3, origin.1 + 4, 31, 49),
        [236; 3],
    );
    assert!(
        vision
            .verify_position(&unreadable, &expected, left, None)
            .is_err()
    );
}


/// The complete third pattern at 7|2 does not inherit the fixed 7|1 exception.
#[test]
fn native_161_coral_rendering_is_fixed_to_its_measured_location() {
    let vision = crate::pyramid_recon::vision::ReconVision::bundled_native_v23().unwrap();
    let original = retained();
    let mut shifted = original.clone();
    let stride = shifted.stride;
    let destination_x = usize::try_from(TABLEAU_ORIGINS[22].0 - 2).unwrap();
    for row in 0..34 {
        let source = (588 + row) * stride + 286 * 4;
        let destination = (588 + row) * stride + destination_x * 4;
        shifted.pixels[destination..destination + 144 * 4]
            .copy_from_slice(&original.pixels[source..source + 144 * 4]);
    }
    assert_ne!(
        vision.native_occupancy(&shifted).unwrap()[22],
        crate::pyramid::FaceEvidence::Absent
    );
}
