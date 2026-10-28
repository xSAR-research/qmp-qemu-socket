//! Retained coral effect evidence and material refusals without guest input.

use super::*;


/// Decode the byte-original frame retained after the two acknowledged clicks.
fn retained() -> CapturedFrame {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pyramid-recognition-159/coral-effect.png");
    decode_png(&std::fs::read(path).unwrap()).unwrap()
}


/// Read the original complete identity owner, then remove the two fixed slots.
fn expected_position() -> [Option<Card>; 28] {
    let file = std::fs::File::open(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/pyramid-recognition-159/deal.json"),
    )
    .unwrap();
    let deal = xsar::pyramid_json::read_from(file).unwrap();
    let mut expected = (*deal.tableau()).map(Some);
    expected[21] = None;
    expected[23] = None;
    expected
}


/// Both removed original slots need positive absence and every other face must agree.
#[test]
fn native_159_retained_coral_pair_has_complete_position_evidence() {
    let vision = crate::pyramid_recon::vision::ReconVision::bundled_native_v23().unwrap();
    let frame = retained();
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


/// White and aged card material at the measured probe must still withhold absence.
#[test]
fn native_159_coral_reference_rejects_present_and_aged_material() {
    let vision = crate::pyramid_recon::vision::ReconVision::bundled_native_v23().unwrap();
    let slot = crate::pyramid::PyramidTargetKind::Card { row: 7, column: 1 }
        .slot_index()
        .unwrap();
    for rgb in [[250; 3], [211, 205, 185], [185; 3]] {
        let mut frame = retained();
        fill(&mut frame, PixelRect::new(328, 610, 60, 4), rgb);
        assert_ne!(
            vision.native_occupancy(&frame).unwrap()[slot],
            crate::pyramid::FaceEvidence::Absent,
            "material {rgb:?} cannot borrow a bare-artwork witness"
        );
        assert!(
            vision
                .verify_position(
                    &frame,
                    &expected_position(),
                    Some(Card::new(Rank::Eight, Suit::Clubs)),
                    None,
                )
                .is_err()
        );
    }
}


/// One changed artwork pixel outside the probe invalidates the full-clearance proof.
#[test]
fn native_159_coral_reference_checks_the_entire_clearance() {
    let vision = crate::pyramid_recon::vision::ReconVision::bundled_native_v23().unwrap();
    let mut frame = retained();
    fill(&mut frame, PixelRect::new(286, 588, 1, 1), [0; 3]);
    assert!(
        vision
            .verify_position(
                &frame,
                &expected_position(),
                Some(Card::new(Rank::Eight, Suit::Clubs)),
                None,
            )
            .is_err()
    );
}


/// The identical RGB pattern at another original slot grants no measured exception.
#[test]
fn native_159_coral_reference_is_fixed_to_its_measured_location() {
    let vision = crate::pyramid_recon::vision::ReconVision::bundled_native_v23().unwrap();
    let mut frame = retained();
    let original = frame.clone();
    let stride = frame.stride;
    let destination_x = usize::try_from(TABLEAU_ORIGINS[22].0 - 2).unwrap();
    for row in 0..34 {
        let source = (588 + row) * stride + 286 * 4;
        let destination = (588 + row) * stride + destination_x * 4;
        frame.pixels[destination..destination + 144 * 4]
            .copy_from_slice(&original.pixels[source..source + 144 * 4]);
    }
    let other_slot = crate::pyramid::PyramidTargetKind::Card { row: 7, column: 2 }
        .slot_index()
        .unwrap();
    assert_ne!(
        vision.native_occupancy(&frame).unwrap()[other_slot],
        crate::pyramid::FaceEvidence::Absent
    );
}


/// Original card material, unreadable remaining identities and wrong piles still refuse.
#[test]
fn native_159_coral_absence_preserves_complete_card_and_pile_checks() {
    let vision = crate::pyramid_recon::vision::ReconVision::bundled_native_v23().unwrap();
    let left = Some(Card::new(Rank::Eight, Suit::Clubs));
    let expected = expected_position();
    let frame = retained();
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
    let mut damaged = frame;
    let origin = TABLEAU_ORIGINS[0];
    fill(
        &mut damaged,
        PixelRect::new(origin.0 + 3, origin.1 + 4, 31, 49),
        [236; 3],
    );
    assert!(
        vision
            .verify_position(&damaged, &expected, left, None)
            .is_err()
    );
    let untouched = decode_png(
        &std::fs::read(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/pyramid-recognition-158/underwater.png"),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(
        vision
            .verify_position(&untouched, &expected, left, None)
            .is_err()
    );
}
