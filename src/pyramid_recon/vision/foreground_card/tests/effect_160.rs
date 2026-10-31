//! Retained coral rendering variation and complete fixed-position refusals.

use super::*;


/// Decode the second original rendering retained after two acknowledged clicks.
fn current() -> CapturedFrame {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pyramid-recognition-160/coral-effect.png");
    decode_png(&std::fs::read(path).unwrap()).unwrap()
}


/// Read the shared complete deal and remove only the two original matched slots.
fn expected_position() -> [Option<Card>; 28] {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pyramid-recognition-159/deal.json");
    let deal = xsar::pyramid_json::read_from(std::fs::File::open(path).unwrap()).unwrap();
    let mut expected = (*deal.tableau()).map(Some);
    expected[21] = None;
    expected[23] = None;
    expected
}


/// Every remaining identity and both exact piles must agree on the new rendering.
#[test]
fn native_160_coral_rendering_verifies_complete_position() {
    let frame = current();
    let vision = crate::pyramid_recon::vision::ReconVision::bundled_native_v23().unwrap();
    vision
        .verify_position(
            &frame,
            &expected_position(),
            Some(Card::new(Rank::Eight, Suit::Clubs)),
            None,
        )
        .unwrap();
    for column in [1, 3] {
        let slot = crate::pyramid::PyramidTargetKind::Card { row: 7, column }
            .slot_index()
            .unwrap();
        assert_eq!(
            vision.native_occupancy(&frame).unwrap()[slot],
            crate::pyramid::FaceEvidence::Absent
        );
    }
}


/// The retained crop must be byte-for-byte native RGB, with no rescaling or masking.
#[test]
fn native_160_coral_variant_is_its_complete_original_crop() {
    let frame = current();
    let view = frame.as_view().unwrap();
    let expected = include_bytes!(
        "../../../../../tests/fixtures/pyramid-recognition-160/underwater-coral-clearance-variant.rgb"
    );
    let actual: Vec<u8> = (588..622)
        .flat_map(|y| (286..430).flat_map(move |x| view.pixel_rgb(x, y).unwrap()))
        .collect();
    assert_eq!(actual.as_slice(), expected);
}


/// Card material and out-of-range artwork keep the current slot uncertain.
#[test]
fn native_160_coral_rendering_refuses_material_and_unmeasured_pixels() {
    let vision = crate::pyramid_recon::vision::ReconVision::bundled_native_v23().unwrap();
    let expected = expected_position();
    let left = Some(Card::new(Rank::Eight, Suit::Clubs));
    for rgb in [[250; 3], [211, 205, 185], [185; 3]] {
        let mut frame = current();
        fill(&mut frame, PixelRect::new(328, 610, 60, 4), rgb);
        assert!(
            vision
                .verify_position(&frame, &expected, left, None)
                .is_err()
        );
    }
    let mut damaged = current();
    fill(&mut damaged, PixelRect::new(286, 588, 1, 1), [0; 3]);
    assert!(
        vision
            .verify_position(&damaged, &expected, left, None)
            .is_err()
    );
    let mut outside_range = current();
    let index = 588 * outside_range.stride + 286 * 4;
    outside_range.pixels[index] = 255;
    assert!(
        vision
            .verify_position(&outside_range, &expected, left, None)
            .is_err()
    );
}


/// A valid bare footprint never compensates for a wrong identity or pile.
#[test]
fn native_160_coral_rendering_retains_exact_identity_and_pile_checks() {
    let vision = crate::pyramid_recon::vision::ReconVision::bundled_native_v23().unwrap();
    let frame = current();
    let expected = expected_position();
    let left = Some(Card::new(Rank::Eight, Suit::Clubs));
    assert!(
        vision
            .verify_position(
                &frame,
                &expected,
                Some(Card::new(Rank::Nine, Suit::Clubs)),
                None,
            )
            .is_err()
    );
    assert!(
        vision
            .verify_position(
                &frame,
                &expected,
                left,
                Some(Card::new(Rank::Ace, Suit::Hearts)),
            )
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
