//! Complete retained Coral positions and fixed-location exact-proof regressions.

use super::*;


/// Original request and one-based column for each complete retained RGB proof.
const CASES: [(u8, u8); 3] = [(2, 6), (13, 1), (13, 2)];


/// Decode one byte-original retained native frame for the specified request.
fn original(request: u8, name: &str) -> CapturedFrame {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!(
            "tests/fixtures/pyramid-recognition-165/request{request}"
        ))
        .join(name);
    decode_png(&std::fs::read(path).unwrap()).unwrap()
}


/// Derive every expected identity from the sole original deal and recorded removals.
fn expected_position(request: u8) -> [Option<Card>; 28] {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "tests/fixtures/pyramid-recognition-165/request{request}/deal.json"
    ));
    let deal = xsar::pyramid_json::read_from(std::fs::File::open(path).unwrap()).unwrap();
    let mut expected = (*deal.tableau()).map(Some);
    let removals = match request {
        2 => [
            (21, Card::new(Rank::Five, Suit::Spades)),
            (23, Card::new(Rank::Eight, Suit::Spades)),
            (26, Card::new(Rank::Five, Suit::Hearts)),
        ],
        13 => [
            (21, Card::new(Rank::King, Suit::Diamonds)),
            (22, Card::new(Rank::Three, Suit::Hearts)),
            (26, Card::new(Rank::Ten, Suit::Diamonds)),
        ],
        _ => panic!("unsupported retained request"),
    };
    for (index, card) in removals {
        assert_eq!(expected[index], Some(card));
        expected[index] = None;
    }
    expected
}


/// Return the pending-after DL identity; both packets require an empty DR.
fn left_pile(request: u8) -> Card {
    match request {
        2 => Card::new(Rank::Three, Suit::Clubs),
        13 => Card::new(Rank::Jack, Suit::Diamonds),
        _ => panic!("unsupported retained request"),
    }
}


/// Validate all original identities and both piles with the production reader.
fn verify(request: u8, frame: &CapturedFrame) -> Result<(), String> {
    crate::pyramid_recon::vision::ReconVision::bundled_native_v23()
        .unwrap()
        .verify_position(
            frame,
            &expected_position(request),
            Some(left_pile(request)),
            None,
        )
}


/// Map a one-based original bottom-row column into the semantic target array.
fn target_slot(column: u8) -> usize {
    crate::pyramid::PyramidTargetKind::Card { row: 7, column }
        .slot_index()
        .unwrap()
}


/// Locate each calibrated footprint using the authoritative typed card geometry.
fn clearance(column: u8) -> PixelRect {
    let target = crate::pyramid::PYRAMID_TARGETS[target_slot(column)];
    PixelRect::new(target.bounds.x - 2, target.bounds.y + 156, 144, 34)
}


/// The retained second MATCH must verify later identities and piles after 7|6.
#[test]
fn native_165_request2_verifies_complete_position() {
    let frame = original(2, "last-observed.png");
    let unchanged = frame.pixels.clone();
    verify(2, &frame).unwrap();
    assert_eq!(frame.pixels, unchanged);
}


/// A previously verified King removal remains absent during the subsequent pair.
#[test]
fn native_165_request13_verifies_complete_position() {
    let frame = original(13, "last-observed.png");
    let unchanged = frame.pixels.clone();
    let vision = crate::pyramid_recon::vision::ReconVision::bundled_native_v23().unwrap();
    let occupancy = vision.native_occupancy(&frame).unwrap();
    let removed = [1, 2, 6].map(|column| occupancy[target_slot(column)]);
    assert_eq!(removed, [crate::pyramid::FaceEvidence::Absent; 3]);
    verify(13, &frame).unwrap();
    assert_eq!(frame.pixels, unchanged);
}


/// All three exact references contain every original RGB byte from their full footprints.
#[test]
fn native_165_complete_crops_are_byte_original() {
    for (request, column, reference) in [
        (
            2,
            6,
            include_bytes!(
                "../../../../../tests/fixtures/pyramid-recognition-165/request2/clearance.rgb"
            ),
        ),
        (
            13,
            1,
            include_bytes!(
                "../../../../../tests/fixtures/pyramid-recognition-165/request13/clearance.rgb"
            ),
        ),
        (
            13,
            2,
            include_bytes!(
                "../../../../../tests/fixtures/pyramid-recognition-165/request13/clearance-seven-two.rgb"
            ),
        ),
    ] {
        let frame = original(request, "last-observed.png");
        let view = frame.as_view().unwrap();
        let bounds = clearance(column);
        let bytes: Vec<u8> = (bounds.y..bounds.bottom())
            .flat_map(|y| {
                (bounds.x..bounds.right()).flat_map(move |x| view.pixel_rgb(x, y).unwrap())
            })
            .collect();
        assert_eq!(bytes.len(), 14_688);
        assert_eq!(bytes.as_slice(), reference);
    }
}


/// Complete present-card crops cannot become absence through any additional proof.
#[test]
fn native_165_real_card_material_refuses() {
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
fn native_165_material_overlays_refuse() {
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


/// One altered RGB byte at corners or centre defeats every additional exact path.
#[test]
fn native_165_changed_rgb_refuses() {
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


/// A one-pixel translation has no complete exact match at any fixed location.
#[test]
fn native_165_shifted_artwork_refuses() {
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
            assert!(verify(request, &frame).is_err(), "{request}: {x},{y}");
        }
    }
}


/// Transplanted templates cannot authorise absence in any other original slot.
#[test]
fn native_165_calibrations_are_fixed_to_their_original_slots() {
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
fn native_165_remaining_identities_and_piles_are_required() {
    let vision = crate::pyramid_recon::vision::ReconVision::bundled_native_v23().unwrap();
    for request in [2, 13] {
        let frame = original(request, "last-observed.png");
        let expected = expected_position(request);
        for (left, right) in [
            (Some(Card::new(Rank::Ace, Suit::Spades)), None),
            (
                Some(left_pile(request)),
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
        assert_ne!(wrong[27], Some(Card::new(Rank::Queen, Suit::Diamonds)));
        wrong[27] = Some(Card::new(Rank::Queen, Suit::Diamonds));
        assert!(
            vision
                .verify_position(&frame, &wrong, Some(left_pile(request)), None)
                .is_err()
        );
    }
}


/// The supported-scene signature and native layout remain mandatory.
#[test]
fn native_165_scene_and_frame_layout_are_required() {
    for request in [2, 13] {
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
