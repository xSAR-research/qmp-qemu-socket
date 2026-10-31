//! Retained 167 failures distinguish corner acceptance from complete action acceptance.

use super::tests::{cards, frame, history, settled};
use super::*;


/// The retained standard-green transition passes removal, stock and all position checks.
#[test]
fn temporal_168_green_transition_verifies_whole_expected_position() {
    retained_failure_case("green", 22, 343, Card::new(Rank::Jack, Suit::Hearts), None);
}


/// Both Coral removals and the complete expected state pass the approved default policy.
#[test]
fn temporal_168_coral_transition_verifies_whole_expected_position() {
    retained_failure_case(
        "coral",
        17,
        276,
        Card::new(Rank::Three, Suit::Spades),
        Some(Card::new(Rank::King, Suit::Diamonds)),
    );
}


/// Earlier saved pixels supply occupied corners, not a fresh runtime source receipt.
/// Assert the unchanged exact policy before requiring complete position or refusal.
fn retained_failure_case(
    name: &str,
    failed_slot: usize,
    count: usize,
    left: Card,
    right: Option<Card>,
) {
    let prefix = format!("pyramid-recognition-168/{name}");
    let before_frame = frame(&format!("{prefix}/first-unmatched.png"));
    let after_frame = frame(&format!("{prefix}/last-observed.png"));
    let details: serde_json::Value = serde_json::from_slice(
        &std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures")
                .join(&prefix)
                .join("error-state.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let initial = cards(&format!("{prefix}/deal.json"));
    let state = |phase: &str| {
        let values = details["details"]["pending_action"][phase]["tableau"]
            .as_array()
            .unwrap();
        assert_eq!(values.len(), 28);
        std::array::from_fn(|index| {
            let (row, column) = xsar::games::pyramid::planning::tableau_coordinates(index).unwrap();
            assert_eq!(values[index]["location"], format!("{row}|{column}"));
            if values[index]["card"].is_null() {
                None
            } else {
                initial[index]
            }
        })
    };
    let before = state("before");
    let expected = state("after");
    let original_before = corner(&before_frame, failed_slot).unwrap();
    let original_after = corner(&after_frame, failed_slot).unwrap();
    let measured = xsar::image_matching::packed_rgb::compare_packed_rgb(
        &original_before,
        &original_after,
        26,
        100,
    )
    .unwrap();
    assert_eq!(
        (measured.changed_pixels, measured.total_pixels),
        (count, 1519)
    );
    assert!(substantial(&original_before, &original_after).unwrap());
    let mut history = history(&before);
    let mut vision = ReconVision::bundled_native_v23()
        .unwrap()
        .for_temporal_execution();
    history.begin(&before_frame, &before, expected).unwrap();
    if name == "coral" {
        // This corner-only historical control is not the new quarter-card decision.
        let measured = xsar::image_matching::packed_rgb::compare_packed_rgb(
            &corner(&before_frame,22).unwrap(), &corner(&after_frame,22).unwrap(),26,10).unwrap();
        assert_eq!((measured.changed_pixels,measured.total_pixels,measured.allowed_changes),(147,1519,151));
        assert!(!measured.exceeds_allowance());
    }
    let stock = crate::pyramid_effect::StockBefore::capture(&before_frame).unwrap();
    assert!(stock.verify(&before_frame, [true, false]).is_err());
    stock.verify(&after_frame, [true, false]).unwrap();
    let candidate = settled(&mut history, &vision, &after_frame, &expected);
    vision.set_temporal_removals(candidate.proof);
    vision
        .verify_position(&after_frame, &expected, Some(left), right)
        .unwrap();
    assert!(
        vision
            .verify_position(&after_frame, &expected, None, right)
            .is_err()
    );
    let mut wrong = expected;
    wrong[27] = Some(Card::new(Rank::Ace, Suit::Clubs));
    assert!(
        vision
            .verify_position(&after_frame, &wrong, Some(left), right)
            .is_err()
    );
    history.commit(candidate).unwrap();
}
