//! Offline temporal-corner contract, retained RGB transitions and guard regressions.

use std::path::PathBuf;

use super::*;
use crate::capture::decode_png;


/// Load unchanged retained bytes, without capturing or modifying a live guest.
pub(super) fn frame(relative: &str) -> CapturedFrame {
    decode_png(
        &std::fs::read(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures")
                .join(relative),
        )
        .unwrap(),
    )
    .unwrap()
}


/// Read the original complete deal through xSAR's bounded strict public codec.
pub(super) fn cards(relative: &str) -> [Option<Card>; 28] {
    (*xsar::pyramid_json::read_from(
        std::fs::File::open(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures")
                .join(relative),
        )
        .unwrap(),
    )
    .unwrap()
    .tableau())
    .map(Some)
}


/// Seed a test's explicitly supplied previously verified history, never product input.
pub(super) fn history(before: &[Option<Card>; 28]) -> RemovalHistory {
    RemovalHistory {
        confirmed: RemovalProof {
            slots: before.iter().enumerate().fold(0, |mask, (slot, card)| {
                mask | if card.is_none() { 1 << slot } else { 0 }
            }),
        },
        pending: None,
    }
}


/// Run one expected change through two distinct supported observation serials.
pub(super) fn settled(
    history: &mut RemovalHistory,
    vision: &ReconVision,
    after: &CapturedFrame,
    expected: &[Option<Card>; 28],
) -> RemovalCandidate {
    assert!(!history.stage(after, expected, 1, vision).unwrap().settled);
    let second = history.stage(after, expected, 2, vision).unwrap();
    assert!(second.settled);
    second
}


/// Retained pairs explicitly record both passing and under-threshold joint RGB changes.
#[test]
fn temporal_168_retained_corners_record_exact_joint_policy() {
    for (folder, slot, accepted) in [
        ("pyramid-recognition-166", 25, true),
        ("pyramid-recognition-165/request2", 26, true),
        ("pyramid-recognition-165/request13", 22, true),
    ] {
        let before = frame(&format!("{folder}/first-unmatched.png"));
        let after = frame(&format!("{folder}/last-observed.png"));
        assert_eq!(
            substantial(
                &corner(&before, slot).unwrap(),
                &corner(&after, slot).unwrap(),
            ).unwrap(), accepted,
            "{folder}"
        );
    }
    let before = frame("pyramid-recognition-167/initial.png");
    let after = frame("pyramid-recognition-167/last-observed.png");
    for slot in [15, 18] {
        assert!(substantial(
            &corner(&before, slot).unwrap(),
            &corner(&after, slot).unwrap(),
        ).unwrap());
    }
}


/// The actual row-6 pending pair passes all later identities/piles after temporal settling.
#[test]
fn temporal_167_row_six_complete_position_and_original_corner_geometry() {
    let initial = frame("pyramid-recognition-167/initial.png");
    let after = frame("pyramid-recognition-167/last-observed.png");
    let unchanged = after.pixels.clone();
    let mut before = cards("pyramid-recognition-167/deal.json");
    // Supplied prior verified occupancy from the packet, not reconstructed live timing.
    for slot in [17, 21, 22, 23, 24, 25, 26] {
        before[slot] = None;
    }
    assert_eq!(before[15], Some(Card::new(Rank::Two, Suit::Spades)));
    assert_eq!(before[18], Some(Card::new(Rank::Jack, Suit::Spades)));
    let mut expected = before;
    expected[15] = None;
    expected[18] = None;
    let mut history = history(&before);
    let mut vision = ReconVision::bundled_native_v23()
        .unwrap()
        .for_temporal_execution();
    vision.set_temporal_removals(history.proof());
    assert!(
        vision
            .verify_position(
                &after,
                &expected,
                Some(Card::new(Rank::Three, Suit::Spades)),
                Some(Card::new(Rank::King, Suit::Diamonds))
            )
            .is_err()
    );
    history.begin(&initial, &before, expected).unwrap();
    assert!(
        history
            .pending
            .as_ref()
            .unwrap()
            .targets
            .iter()
            .all(|target| target.before.len() == 5874)
    );
    let candidate = settled(&mut history, &vision, &after, &expected);
    vision.set_temporal_removals(candidate.proof);
    vision
        .verify_position(
            &after,
            &expected,
            Some(Card::new(Rank::Three, Suit::Spades)),
            Some(Card::new(Rank::King, Suit::Diamonds)),
        )
        .unwrap();
    assert!(
        history
            .pending
            .as_ref()
            .unwrap()
            .targets
            .iter()
            .all(|target| target.previous.as_ref().unwrap().len() == 1519)
    );
    history.commit(candidate).unwrap();
    assert!(history.proof().contains(15) && history.proof().contains(18));
    let pixels = corner(&initial, 15).unwrap();
    assert_eq!(pixels.len(), 31 * 49);
    assert_eq!(
        pixels[0].to_le_bytes()[..3],
        initial.as_view().unwrap().pixel_rgb(392, 382).unwrap()
    );
    assert_eq!(after.pixels, unchanged);
    assert!(
        vision
            .verify_position(
                &after,
                &expected,
                Some(Card::new(Rank::Four, Suit::Spades)),
                Some(Card::new(Rank::King, Suit::Diamonds))
            )
            .is_err()
    );
    let mut wrong = expected;
    wrong[27] = Some(Card::new(Rank::Ace, Suit::Diamonds));
    assert!(
        vision
            .verify_position(
                &after,
                &wrong,
                Some(Card::new(Rank::Three, Suit::Spades)),
                Some(Card::new(Rank::King, Suit::Diamonds))
            )
            .is_err()
    );
}


/// Settling requires distinct captures and is broken by an intervening guard mismatch.
#[test]
fn temporal_167_settling_rejects_reused_moving_or_interrupted_observations() {
    let initial = frame("pyramid-recognition-167/initial.png");
    let after = frame("pyramid-recognition-167/last-observed.png");
    let before = cards("pyramid-recognition-167/deal.json");
    let mut expected = before;
    expected[15] = None;
    let mut history = history(&before);
    let vision = ReconVision::bundled_native_v23()
        .unwrap()
        .for_temporal_execution();
    history.begin(&initial, &before, expected).unwrap();
    assert!(history.stage(&initial, &expected, 1, &vision).is_err());
    history.reject();
    let first = history.stage(&after, &expected, 2, &vision).unwrap();
    assert!(!first.settled);
    assert!(history.commit(first).is_err());
    assert!(history.stage(&after, &expected, 2, &vision).is_err());
    history.reject();
    assert!(
        !history
            .stage(&after, &expected, 3, &vision)
            .unwrap()
            .settled
    );
    history.reject();
    assert!(
        !history
            .stage(&after, &expected, 4, &vision)
            .unwrap()
            .settled
    );
    let candidate = history.stage(&after, &expected, 5, &vision).unwrap();
    assert!(
        history
            .pending
            .as_ref()
            .unwrap()
            .targets
            .iter()
            .all(|target| target.previous.as_ref().unwrap().len() == 1519)
    );
    history.commit(candidate).unwrap();
    assert!(history.proof().contains(15));
}


/// Small anti-aliasing, alpha-only and small partial changes cannot prove removal.
#[test]
fn temporal_167_small_partial_and_alpha_changes_are_insufficient() {
    let before = vec![u32::from_le_bytes([120, 130, 140, 0]); 1519];
    let aa = vec![u32::from_le_bytes([122, 128, 142, 255]); 1519];
    assert!(!substantial(&before, &aa).unwrap());
    assert!(quiet(
        compare_packed_bytes(&before, &aa, BYTE_TOLERANCE).unwrap()
    ));
    let mut partial = before.clone();
    partial[..75].fill(0);
    assert!(!substantial(&before, &partial).unwrap());
    let mut light = before.clone();
    light.fill(u32::from_le_bytes([220, 230, 240, 0]));
    let mut dark = before.clone();
    dark.fill(u32::from_le_bytes([20, 30, 40, 0]));
    assert!(substantial(&before, &light).unwrap());
    assert!(substantial(&before, &dark).unwrap());
    assert!(!quiet(
        compare_packed_bytes(&light, &dark, BYTE_TOLERANCE).unwrap()
    ));
}


/// Malformed storage, resurrection, wrong transitions and unproven history fail closed.
#[test]
fn temporal_167_rejects_bad_geometry_and_unbound_transitions() {
    let mut initial = frame("pyramid-recognition-167/initial.png");
    let before = cards("pyramid-recognition-167/deal.json");
    let mut after = before;
    after[15] = None;
    let mut history = history(&before);
    let vision = ReconVision::bundled_native_v23()
        .unwrap()
        .for_temporal_execution();
    assert!(history.stage(&initial, &after, 1, &vision).is_err());
    initial.pixels.pop();
    assert!(history.begin(&initial, &before, after).is_err());
    let initial = frame("pyramid-recognition-167/initial.png");
    let mut different = before;
    different[15] = Some(Card::new(Rank::Ace, Suit::Clubs));
    assert!(history.begin(&initial, &before, different).is_err());
    history.begin(&initial, &before, after).unwrap();
    assert!(history.begin(&initial, &before, after).is_err());
    assert!(history.stage(&initial, &before, 2, &vision).is_err());
}


/// Older retained pairs either verify fully or retain their measured policy refusal.
#[test]
fn temporal_168_earlier_full_positions_preserve_joint_policy_refusals() {
    for (folder, previous, removed, left) in [
        (
            "pyramid-recognition-165/request2",
            vec![21, 23],
            vec![26],
            Card::new(Rank::Three, Suit::Clubs),
        ),
        (
            "pyramid-recognition-165/request13",
            vec![21],
            vec![22, 26],
            Card::new(Rank::Jack, Suit::Diamonds),
        ),
    ] {
        let source = frame(&format!("{folder}/first-unmatched.png"));
        let after = frame(&format!("{folder}/last-observed.png"));
        let mut before = cards(&format!("{folder}/deal.json"));
        for slot in previous {
            before[slot] = None;
        }
        let mut expected = before;
        for slot in removed {
            expected[slot] = None;
        }
        let mut history = history(&before);
        let mut vision = ReconVision::bundled_native_v23()
            .unwrap()
            .for_temporal_execution();
        history.begin(&source, &before, expected).unwrap();
        if folder.ends_with("request13") {
            // Preserve the historical 31-by-49 control independently of the new ROI.
            let measured = xsar::image_matching::packed_rgb::compare_packed_rgb(
                &corner(&source,22).unwrap(), &corner(&after,22).unwrap(),26,10).unwrap();
            assert_eq!((measured.changed_pixels,measured.total_pixels,measured.allowed_changes),(127,1519,151));
            assert!(!measured.exceeds_allowance());
        }
        let candidate = settled(&mut history, &vision, &after, &expected);
        vision.set_temporal_removals(candidate.proof);
        vision
            .verify_position(&after, &expected, Some(left), None)
            .unwrap();
        history.commit(candidate).unwrap();
    }
}


/// Strict joint RGB coverage and the retained quiet policy have separate boundaries.
#[test]
fn temporal_168_joint_threshold_boundaries_and_channel_agreement() {
    let before = vec![0; 1519];
    let mut after = before.clone();
    after[..75].fill(u32::from_le_bytes([14, 14, 14, 0]));
    assert!(!substantial(&before, &after).unwrap());
    after[75] = u32::from_le_bytes([14, 14, 14, 0]);
    assert!(substantial(&before, &after).unwrap());
    after.fill(u32::from_le_bytes([255, 255, 5, 255]));
    assert!(!substantial(&before, &after).unwrap());
    after.fill(u32::from_le_bytes([14, 14, 14, 0]));
    assert!(substantial(&before, &after).unwrap());
    after.fill(u32::from_le_bytes([255, 0, 0, 255]));
    assert!(!substantial(&before, &after).unwrap());
    for (i, pixel) in after.iter_mut().enumerate() {
        *pixel = u32::from_le_bytes(match i % 3 {
            0 => [255, 255, 0, 255],
            1 => [0, 255, 255, 255],
            _ => [255, 0, 255, 255],
        });
    }
    assert!(!substantial(&before, &after).unwrap());
    after.fill(0);
    after[..15].fill(u32::from_le_bytes([20; 4]));
    assert!(quiet(
        compare_packed_bytes(&before, &after, BYTE_TOLERANCE).unwrap()
    ));
    after[15] = u32::from_le_bytes([20; 4]);
    assert!(!quiet(
        compare_packed_bytes(&before, &after, BYTE_TOLERANCE).unwrap()
    ));
    after.fill(u32::from_le_bytes([2; 4]));
    assert!(quiet(
        compare_packed_bytes(&before, &after, BYTE_TOLERANCE).unwrap()
    ));
    after.fill(u32::from_le_bytes([3; 4]));
    assert!(!quiet(
        compare_packed_bytes(&before, &after, BYTE_TOLERANCE).unwrap()
    ));
}


/// A temporal mode cannot silently fall back to historical static absence proofs.
#[test]
fn temporal_167_static_background_cannot_supply_missing_route_history() {
    let after = frame("pyramid-recognition-166/last-observed.png");
    let mut expected = cards("pyramid-recognition-166/deal.json");
    for slot in [21, 23, 25, 26] {
        expected[slot] = None;
    }
    let vision = ReconVision::bundled_native_v23()
        .unwrap()
        .for_temporal_execution();
    assert!(
        vision
            .verify_position(
                &after,
                &expected,
                Some(Card::new(Rank::Nine, Suit::Clubs)),
                Some(Card::new(Rank::King, Suit::Hearts))
            )
            .unwrap_err()
            .contains("temporal removal evidence")
    );
    assert!(
        vision
            .verify_temporal_clear(after.as_view().unwrap())
            .is_err()
    );
}
