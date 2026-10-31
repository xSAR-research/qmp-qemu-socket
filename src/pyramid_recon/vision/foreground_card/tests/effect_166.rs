//! Full retained Coral position and complete fixed 7|5 absence regressions.

use super::*;


/// Original request and one-based column for the complete retained RGB proof.
const CASES: [(u8, u8); 1] = [(2, 5)];


/// Decode the byte-original native 165 Request 2 frame with no pixel changes.
fn original(request: u8, name: &str) -> CapturedFrame {
    assert_eq!(request, 2);
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pyramid-recognition-166")
        .join(name);
    decode_png(&std::fs::read(path).unwrap()).unwrap()
}


/// Derive all identities from the original deal and assert all four removed cards.
fn expected_position(request: u8) -> [Option<Card>; 28] {
    assert_eq!(request, 2);
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pyramid-recognition-166/deal.json");
    let deal = xsar::pyramid_json::read_from(std::fs::File::open(path).unwrap()).unwrap();
    let mut expected = (*deal.tableau()).map(Some);
    for (index, card) in [
        (21, Card::new(Rank::Five, Suit::Spades)),
        (23, Card::new(Rank::Eight, Suit::Spades)),
        (25, Card::new(Rank::Ace, Suit::Clubs)),
        (26, Card::new(Rank::Five, Suit::Hearts)),
    ] {
        assert_eq!(expected[index], Some(card));
        expected[index] = None;
    }
    expected
}


/// Return the pending-after DL identity following the AC plus QS removal.
fn left_pile(request: u8) -> Card {
    assert_eq!(request, 2);
    Card::new(Rank::Nine, Suit::Clubs)
}


/// Verify every original identity and the exact occupied DL and DR piles.
fn verify(request: u8, frame: &CapturedFrame) -> Result<(), String> {
    crate::pyramid_recon::vision::ReconVision::bundled_native_v23()
        .unwrap()
        .verify_position(
            frame,
            &expected_position(request),
            Some(left_pile(request)),
            Some(Card::new(Rank::King, Suit::Hearts)),
        )
}


/// Map an original one-based bottom-row column to the semantic target array.
fn target_slot(column: u8) -> usize {
    crate::pyramid::PyramidTargetKind::Card { row: 7, column }
        .slot_index()
        .unwrap()
}


/// Derive the calibrated lower footprint from authoritative typed card bounds.
fn clearance(column: u8) -> PixelRect {
    let target = crate::pyramid::PYRAMID_TARGETS[target_slot(column)];
    PixelRect::new(target.bounds.x - 2, target.bounds.y + 156, 144, 34)
}


/// The sixth logical action must pass every identity and both occupied piles.
#[test]
fn native_166_request2_verifies_complete_position() {
    let frame = original(2, "last-observed.png");
    let unchanged = frame.pixels.clone();
    verify(2, &frame).unwrap();
    let vision = crate::pyramid_recon::vision::ReconVision::bundled_native_v23().unwrap();
    let occupancy = vision.native_occupancy(&frame).unwrap();
    assert_eq!(
        [1, 3, 5, 6].map(|column| occupancy[target_slot(column)]),
        [crate::pyramid::FaceEvidence::Absent; 4]
    );
    assert_eq!(frame.pixels, unchanged);
}


/// All 14,688 reference bytes equal the complete native retained PNG footprint.
#[test]
fn native_166_complete_crop_is_byte_original() {
    let frame = original(2, "last-observed.png");
    let view = frame.as_view().unwrap();
    let bounds = clearance(5);
    assert_eq!(bounds, PixelRect::new(1089, 588, 144, 34));
    let reference =
        include_bytes!("../../../../../tests/fixtures/pyramid-recognition-166/clearance.rgb");
    let pixels = reference.as_chunks::<3>().0;
    assert_eq!(pixels.len(), 4896);
    for y in 0..34 {
        for x in 0..144 {
            assert_eq!(
                view.pixel_rgb(bounds.x + x, bounds.y + y).unwrap(),
                pixels[usize::try_from(y * 144 + x).unwrap()]
            );
        }
    }
}


/// Complete present-card crops cannot become absence through any additional proof.
#[test]
fn native_166_real_card_material_refuses() {
    for (request, column) in CASES {
        let before = original(request, "first-unmatched.png");
        let view = before.as_view().unwrap();
        let bounds = clearance(column);
        let mut frame = original(request, "last-observed.png");
        for y in bounds.y..bounds.bottom() {
            for x in bounds.x..bounds.right() {
                fill(
                    &mut frame,
                    PixelRect::new(x, y, 1, 1),
                    view.pixel_rgb(x, y).unwrap(),
                );
            }
        }
        assert!(verify(request, &frame).is_err());
    }
}


/// Full and partial strong-white, warm, aged, grey and black overlays refuse.
#[test]
fn native_166_material_overlays_refuse() {
    for (request, column) in CASES {
        let bounds = clearance(column);
        for colour in [[250; 3], [211, 205, 185], [245, 220, 150], [185; 3], [0; 3]] {
            for (width, height) in [(60, 4), (10, 2)] {
                let mut frame = original(request, "last-observed.png");
                fill(
                    &mut frame,
                    PixelRect::new(bounds.x + 42, bounds.y + 22, width, height),
                    colour,
                );
                assert!(verify(request, &frame).is_err(), "{request}: {colour:?}");
            }
        }
    }
}


/// One altered RGB byte at corners or centre defeats the additional exact path.
#[test]
fn native_166_changed_rgb_refuses() {
    for (request, column) in CASES {
        let bounds = clearance(column);
        for (dx, dy) in [(0, 0), (143, 0), (0, 33), (143, 33), (72, 17)] {
            for channel in 0..3 {
                let mut frame = original(request, "last-observed.png");
                let (x, y) = (bounds.x + dx, bounds.y + dy);
                let mut rgb = frame.as_view().unwrap().pixel_rgb(x, y).unwrap();
                rgb[channel] = rgb[channel].checked_add(1).unwrap();
                fill(&mut frame, PixelRect::new(x, y, 1, 1), rgb);
                assert!(
                    verify(request, &frame).is_err(),
                    "{request}: {x},{y}, {channel}"
                );
            }
        }
    }
}


/// A one-pixel translation refuses the new exact proof at its fixed location.
/// Other unchanged artwork proofs may independently accept such a bare crop.
#[test]
fn native_166_exact_proof_rejects_shifted_artwork() {
    for (request, column) in CASES {
        let source = original(request, "last-observed.png");
        let view = source.as_view().unwrap();
        let bounds = clearance(column);
        for (x, y) in [
            (bounds.x - 1, bounds.y),
            (bounds.x + 1, bounds.y),
            (bounds.x, bounds.y - 1),
            (bounds.x, bounds.y + 1),
        ] {
            let mut frame = source.clone();
            for dy in 0..34 {
                for dx in 0..144 {
                    fill(
                        &mut frame,
                        PixelRect::new(bounds.x + dx, bounds.y + dy, 1, 1),
                        view.pixel_rgb(x + dx, y + dy).unwrap(),
                    );
                }
            }
            assert!(
                !crate::pyramid_recon::vision::native_colour::measured_coral_seven_five_surface(
                    frame.as_view().unwrap(),
                    bounds,
                )
                .unwrap(),
                "{request}: {x},{y}",
            );
        }
    }
}


/// Transplanted templates cannot authorise absence in any other original slot.
#[test]
fn native_166_calibrations_are_fixed_to_their_original_slots() {
    let vision = crate::pyramid_recon::vision::ReconVision::bundled_native_v23().unwrap();
    for (request, column) in CASES {
        let source = original(request, "last-observed.png");
        let view = source.as_view().unwrap();
        let bounds = clearance(column);
        for destination in 1..=7 {
            if destination == column {
                continue;
            }
            let target = crate::pyramid::PYRAMID_TARGETS[target_slot(destination)];
            let (x, y) = (target.bounds.x - 2, target.bounds.y + 156);
            let mut frame = source.clone();
            for dy in 0..34 {
                for dx in 0..144 {
                    fill(
                        &mut frame,
                        PixelRect::new(x + dx, y + dy, 1, 1),
                        view.pixel_rgb(bounds.x + dx, bounds.y + dy).unwrap(),
                    );
                }
            }
            assert_ne!(
                vision.native_occupancy(&frame).unwrap()[target_slot(destination)],
                crate::pyramid::FaceEvidence::Absent,
                "{request}: destination {destination}"
            );
        }
    }
}


/// Local absence never excuses wrong remaining identities or either pile.
#[test]
fn native_166_remaining_identities_and_piles_are_required() {
    let vision = crate::pyramid_recon::vision::ReconVision::bundled_native_v23().unwrap();
    for (request, _) in CASES {
        let frame = original(request, "last-observed.png");
        let expected = expected_position(request);
        for (left, right) in [
            (
                Some(Card::new(Rank::Ace, Suit::Spades)),
                Some(Card::new(Rank::King, Suit::Hearts)),
            ),
            (Some(left_pile(request)), None),
        ] {
            assert!(
                vision
                    .verify_position(&frame, &expected, left, right)
                    .is_err()
            );
        }
        let mut wrong = expected;
        assert_ne!(wrong[27], Some(Card::new(Rank::Queen, Suit::Diamonds)));
        wrong[27] = Some(Card::new(Rank::Queen, Suit::Diamonds));
        assert!(
            vision
                .verify_position(
                    &frame,
                    &wrong,
                    Some(left_pile(request)),
                    Some(Card::new(Rank::King, Suit::Hearts))
                )
                .is_err()
        );
    }
}


/// The supported-scene signature and native layout remain mandatory.
#[test]
fn native_166_scene_and_frame_layout_are_required() {
    for (request, _) in CASES {
        let mut frame = original(request, "last-observed.png");
        fill(
            &mut frame,
            crate::pyramid::WHITE_SCENE_PROBES[0],
            [17, 19, 21],
        );
        assert!(verify(request, &frame).is_err());
        let mut frame = original(request, "last-observed.png");
        frame.width = 1919;
        assert!(verify(request, &frame).is_err());
        let mut frame = original(request, "last-observed.png");
        frame.stride = 1;
        assert!(verify(request, &frame).is_err());
        let mut frame = original(request, "last-observed.png");
        frame.pixels.truncate(100);
        assert!(verify(request, &frame).is_err());
    }
}
