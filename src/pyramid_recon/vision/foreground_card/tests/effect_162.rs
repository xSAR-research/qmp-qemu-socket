//! Full-reader evaluation of structural Coral absence and conservative refusals.

use super::*;


/// Decode an original native frame without changing its geometry or RGB values.
fn original(version: u16) -> CapturedFrame {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "tests/fixtures/pyramid-recognition-{version}/coral-effect.png"
    ));
    decode_png(&std::fs::read(path).unwrap()).unwrap()
}


/// Read this failure's sole card owner and apply only its recorded first MATCH.
fn expected_position() -> [Option<Card>; 28] {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pyramid-recognition-162/deal.json");
    let deal = xsar::pyramid_json::read_from(std::fs::File::open(path).unwrap()).unwrap();
    let mut expected = (*deal.tableau()).map(Some);
    assert_eq!(expected[21], Some(Card::new(Rank::Five, Suit::Spades)));
    assert_eq!(expected[23], Some(Card::new(Rank::Eight, Suit::Spades)));
    expected[21] = None;
    expected[23] = None;
    expected
}


/// Compare the entire pending position, including every identity and both piles.
fn verify(frame: &CapturedFrame) -> Result<(), String> {
    crate::pyramid_recon::vision::ReconVision::bundled_native_v23()
        .unwrap()
        .verify_position(
            frame,
            &expected_position(),
            Some(Card::new(Rank::Eight, Suit::Clubs)),
            None,
        )
}


/// Map a one-based original bottom-row column to the detector's target array.
/// This array includes control targets and is distinct from the 28-card deal.
fn target_slot(column: u8) -> usize {
    crate::pyramid::PyramidTargetKind::Card { row: 7, column }
        .slot_index()
        .unwrap()
}


/// The actual fourth appearance must pass without becoming a calibration reference.
#[test]
fn native_162_reported_fourth_verifies_complete_position() {
    let frame = original(162);
    let pixels = frame.pixels.clone();
    verify(&frame).unwrap();
    let vision = crate::pyramid_recon::vision::ReconVision::bundled_native_v23().unwrap();
    let occupancy = vision.native_occupancy(&frame).unwrap();
    for column in [1, 3] {
        assert_eq!(
            occupancy[target_slot(column)],
            crate::pyramid::FaceEvidence::Absent
        );
    }
    assert_eq!(frame.pixels, pixels, "recognition is read-only");
}


/// All three calibration originals retain complete position verification.
#[test]
fn native_162_all_old_calibration_positions_still_verify() {
    for version in [159, 160, 161] {
        verify(&original(version)).unwrap();
    }
}


/// The evaluation crop is complete and demonstrably outside the old envelope.
#[test]
fn native_162_original_crop_reproduces_the_161_channel_refusal() {
    let frame = original(162);
    let view = frame.as_view().unwrap();
    let actual: Vec<u8> = (588..622)
        .flat_map(|y| (286..430).flat_map(move |x| view.pixel_rgb(x, y).unwrap()))
        .collect();
    assert_eq!(
        actual.as_slice(),
        include_bytes!(
            "../../../../../tests/fixtures/pyramid-recognition-162/underwater-coral-evaluation.rgb"
        )
    );
    let references: Vec<Vec<u8>> = [159, 160, 161]
        .into_iter()
        .map(|version| {
            let frame = original(version);
            let view = frame.as_view().unwrap();
            (588..622)
                .flat_map(|y| (286..430).flat_map(move |x| view.pixel_rgb(x, y).unwrap()))
                .collect()
        })
        .collect();
    let outside = actual
        .iter()
        .enumerate()
        .filter(|(index, value)| {
            let minimum = references.iter().map(|r| r[*index]).min().unwrap();
            let maximum = references.iter().map(|r| r[*index]).max().unwrap();
            **value < minimum || **value > maximum
        })
        .count();
    assert_eq!(outside, 1_924);
}


/// The measured structural edge bounds are inclusive, and one further unit refuses.
#[test]
fn native_162_complete_edge_boundaries_are_inclusive() {
    let source = original(159);
    let rgb = source.as_view().unwrap().pixel_rgb(399, 589).unwrap();
    assert_eq!(rgb, [121, 123, 124]);
    for (channel, limit) in [(0, 5), (1, 7), (2, 9)] {
        let mut allowed = source.clone();
        let mut colour = rgb;
        colour[channel] += limit;
        fill(&mut allowed, PixelRect::new(399, 589, 1, 1), colour);
        verify(&allowed).unwrap();
        let mut refused = source.clone();
        colour[channel] += 1;
        fill(&mut refused, PixelRect::new(399, 589, 1, 1), colour);
        assert!(verify(&refused).is_err());
    }
}


/// Slight smooth brightening cannot introduce new possible or aged card material.
#[test]
fn native_162_smooth_new_material_is_vetoed() {
    let mut frame = original(159);
    let source = frame.clone();
    let view = source.as_view().unwrap();
    for y in 588..622 {
        for x in 286..430 {
            let rgb = view
                .pixel_rgb(x, y)
                .unwrap()
                .map(|channel| channel.checked_add(1).unwrap());
            fill(&mut frame, PixelRect::new(x, y, 1, 1), rgb);
        }
    }
    assert!(verify(&frame).is_err());
}


/// Complete and partial white, warm and aged faces cannot borrow the Coral proof.
#[test]
fn native_162_full_and_partial_card_material_refuses() {
    let vision = crate::pyramid_recon::vision::ReconVision::bundled_native_v23().unwrap();
    for bounds in [
        PixelRect::new(328, 610, 60, 4),
        PixelRect::new(328, 610, 10, 2),
    ] {
        for rgb in [[250; 3], [211, 205, 185], [185; 3], [236, 220, 190]] {
            let mut frame = original(162);
            fill(&mut frame, bounds, rgb);
            assert_ne!(
                vision.native_occupancy(&frame).unwrap()[target_slot(1)],
                crate::pyramid::FaceEvidence::Absent
            );
            assert!(verify(&frame).is_err());
        }
    }
}


/// A retained real face at original 7|1 still blocks the expected removal.
#[test]
fn native_162_real_unremoved_card_refuses() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pyramid-recognition-158/underwater.png");
    let frame = decode_png(&std::fs::read(path).unwrap()).unwrap();
    let vision = crate::pyramid_recon::vision::ReconVision::bundled_native_v23().unwrap();
    assert_ne!(
        vision.native_occupancy(&frame).unwrap()[target_slot(1)],
        crate::pyramid::FaceEvidence::Absent
    );
    assert!(verify(&frame).is_err());
}


/// A shifted whole pattern cannot inherit the measured native footprint.
#[test]
fn native_162_displaced_artwork_refuses() {
    let source = original(162);
    let view = source.as_view().unwrap();
    for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
        let mut frame = source.clone();
        for y in 588..622 {
            for x in 286..430 {
                let rgb = view
                    .pixel_rgb(
                        u32::try_from(i64::from(x) + dx).unwrap(),
                        u32::try_from(i64::from(y) + dy).unwrap(),
                    )
                    .unwrap();
                fill(&mut frame, PixelRect::new(x, y, 1, 1), rgb);
            }
        }
        assert!(verify(&frame).is_err());
    }
}


/// A different expected identity or either pile cannot use the local absence proof.
#[test]
fn native_162_exact_identities_and_piles_remain_required() {
    let frame = original(162);
    let vision = crate::pyramid_recon::vision::ReconVision::bundled_native_v23().unwrap();
    let expected = expected_position();
    for (left, right) in [
        (Some(Card::new(Rank::Nine, Suit::Clubs)), None),
        (
            Some(Card::new(Rank::Eight, Suit::Clubs)),
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
    wrong[0] = Some(Card::new(Rank::Queen, Suit::Spades));
    assert!(
        vision
            .verify_position(
                &frame,
                &wrong,
                Some(Card::new(Rank::Eight, Suit::Clubs)),
                None
            )
            .is_err()
    );
}


/// Corrupting a complete scene role still refuses a locally valid bare footprint.
#[test]
fn native_162_unsupported_scene_refuses() {
    let mut frame = original(162);
    fill(
        &mut frame,
        crate::pyramid::WHITE_SCENE_PROBES[0],
        [17, 19, 21],
    );
    assert!(verify(&frame).is_err());
}


/// Reject each colour sum one unit outside its old measured minimum or maximum.
/// These six single-channel mutations preserve material roles and satisfy the
/// absolute and edge bounds against every reference, isolating the sum guard.
#[test]
fn native_162_whole_patch_brightness_is_bounded() {
    for (version, delta) in [(159, -1_i16), (160, 1_i16)] {
        for channel in 0..3 {
            let mut frame = original(version);
            let mut rgb = frame.as_view().unwrap().pixel_rgb(399, 589).unwrap();
            assert_eq!(rgb, [121, 123, 124]);
            rgb[channel] = u8::try_from(i16::from(rgb[channel]) + delta).unwrap();
            fill(&mut frame, PixelRect::new(399, 589, 1, 1), rgb);
            assert!(
                verify(&frame).is_err(),
                "version {version}, channel {channel}"
            );
        }
    }
}


/// Invalid dimensions or storage layout propagate errors rather than absence.
#[test]
fn native_162_invalid_frame_geometry_refuses() {
    let mut dimensions = original(162);
    dimensions.width = 1919;
    assert!(verify(&dimensions).is_err());
    let mut layout = original(162);
    layout.stride = 1;
    assert!(verify(&layout).is_err());
}


/// Equal opposing red gradients preserve sums, material roles and edge bounds.
/// Their 18-unit peaks exceed the 17-unit absolute bound against every original.
#[test]
fn native_162_smooth_absolute_colour_drift_refuses() {
    let source = original(159);
    let view = source.as_view().unwrap();
    let mut frame = source.clone();
    for y in 0_i32..34 {
        for x in 0_i32..144 {
            let upper = (18 - 3 * ((x - 112).abs() + (y - 8).abs())).max(0);
            let lower = (18 - 3 * ((x - 112).abs() + (y - 25).abs())).max(0);
            let px = u32::try_from(286 + x).unwrap();
            let py = u32::try_from(588 + y).unwrap();
            let mut rgb = view.pixel_rgb(px, py).unwrap();
            rgb[0] = u8::try_from(i32::from(rgb[0]) + upper - lower).unwrap();
            fill(&mut frame, PixelRect::new(px, py, 1, 1), rgb);
        }
    }
    assert!(verify(&frame).is_err());
}
