//! Exposed native target-clear regressions; these are offline recognition evidence.

use super::*;
use crate::pyramid_progress::{ChallengeKind, ChallengeProgress};


/// Decode one unchanged retained final-board frame without capturing the guest.
fn frame(name: &str) -> CapturedFrame {
    crate::capture::decode_png(
        &std::fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/pyramid-recon/native-v20")
                .join(name),
        )
        .unwrap(),
    )
    .unwrap()
}


/// Read each actual pile independently through the unchanged native card reader.
fn piles(reader: &ReconVision, frame: &CapturedFrame) -> (Option<Card>, Option<Card>) {
    let view = frame.as_view().unwrap();
    let card = |bounds, empty| match reader.stock_face(view, bounds, empty).unwrap() {
        StockFace::Card(card) => Some(card),
        StockFace::Empty => None,
    };
    (
        card(LEFT_BOUNDS, &reader.board.left_empty),
        card(RIGHT_BOUNDS, &reader.board.right_empty),
    )
}


/// The last retained frame proves the card effect, never a terminal or fresh board.
#[test]
fn target_completion_native_final_clear_requires_its_explicit_capability() {
    let reader = ReconVision::bundled_native_v20().unwrap();
    let old = ReconVision::bundled_native_v19().unwrap();
    let last = frame("last-observed.png");
    let (left, right) = piles(&reader, &last);
    let initial = ChallengeProgress::new(ChallengeKind::Boards, 1, 2).unwrap();
    assert_eq!(
        crate::pyramid_progress::observe_challenge(&last).unwrap(),
        ChallengeProgress::new(ChallengeKind::Boards, 2, 2)
    );
    assert!(
        reader
            .verify_position(&last, &[None; 28], left, right)
            .is_err()
    );
    assert!(
        reader
            .verify_final_clear(&last, &[None; 28], left, right)
            .is_err()
    );
    assert!(
        reader
            .verify_board_count_final_clear(&last, &[None; 28], left, right, initial)
            .is_err()
    );
    assert!(
        old.verify_board_count_target_clear(&last, &[None; 28], left, right, initial)
            .is_err()
    );
    reader
        .verify_board_count_target_clear(&last, &[None; 28], left, right, initial)
        .unwrap();
    assert!(
        reader
            .verify_board_count_target_clear(
                &frame("first-unmatched.png"),
                &[None; 28],
                left,
                right,
                initial,
            )
            .is_err(),
        "the flying frame still fails its native scene"
    );
    assert!(!crate::pyramid_progress::is_challenge_complete(&last).unwrap());
    assert!(!matches!(reader.observe(&last), Ok(ReconScene::Board(_))));
}


/// Every original slot and every positive band remains mandatory at the target.
#[test]
fn target_completion_refuses_remaining_cards_and_one_bad_pixel_in_every_band() {
    let reader = ReconVision::bundled_native_v20().unwrap();
    let original = frame("last-observed.png");
    let (left, right) = piles(&reader, &original);
    let initial = ChallengeProgress::new(ChallengeKind::Boards, 1, 2).unwrap();
    for (index, (x, y)) in TABLEAU_ORIGINS.into_iter().enumerate() {
        let mut expected = [None; 28];
        expected[index] = Some(Card::new(Rank::Ace, Suit::Clubs));
        assert!(
            reader
                .verify_board_count_target_clear(&original, &expected, left, right, initial)
                .is_err()
        );
        let mut bad = original.clone();
        let band_y = if index >= 21 { 164 } else { 4 };
        let offset = (y as usize + band_y) * bad.stride + (x as usize + 40) * 4;
        bad.pixels[offset..offset + 3].fill(255);
        assert!(
            reader
                .verify_board_count_target_clear(&bad, &[None; 28], left, right, initial)
                .is_err(),
            "slot {index} must retain every positive felt pixel"
        );
    }
}


/// Wrong requirements, piles, scene and frame storage cannot borrow the new proof.
#[test]
fn target_completion_refuses_changed_skipped_and_already_complete_bindings() {
    let reader = ReconVision::bundled_native_v20().unwrap();
    let original = frame("last-observed.png");
    let (left, right) = piles(&reader, &original);
    for (kind, current, target) in [
        (ChallengeKind::Boards, 0, 2),
        (ChallengeKind::Boards, 2, 2),
        (ChallengeKind::Boards, 1, 3),
        (ChallengeKind::Score, 1, 2),
        (ChallengeKind::Cards, 1, 2),
    ] {
        let initial = ChallengeProgress::new(kind, current, target).unwrap();
        assert!(
            reader
                .verify_board_count_target_clear(&original, &[None; 28], left, right, initial)
                .is_err()
        );
    }
    let initial = ChallengeProgress::new(ChallengeKind::Boards, 1, 2).unwrap();
    let wrong = |actual| {
        Some(if actual == Some(Card::new(Rank::Ace, Suit::Clubs)) {
            Card::new(Rank::King, Suit::Spades)
        } else {
            Card::new(Rank::Ace, Suit::Clubs)
        })
    };
    for (bad_left, bad_right) in [(wrong(left), right), (left, wrong(right))] {
        assert!(
            reader
                .verify_board_count_target_clear(
                    &original,
                    &[None; 28],
                    bad_left,
                    bad_right,
                    initial,
                )
                .is_err()
        );
    }
    for scene in &reader.board.scene {
        let mut bad = original.clone();
        let offset = scene.bounds.y as usize * bad.stride + scene.bounds.x as usize * 4;
        bad.pixels[offset] ^= 128;
        assert!(
            reader
                .verify_board_count_target_clear(&bad, &[None; 28], left, right, initial)
                .is_err()
        );
    }
    let mut malformed = original;
    malformed.pixels.truncate(17);
    assert!(
        reader
            .verify_board_count_target_clear(&malformed, &[None; 28], left, right, initial)
            .is_err()
    );
}


/// The new schema capability must be explicit and retain its original proof dependency.
#[test]
fn target_completion_profile_rejects_implicit_and_weakened_opt_in() {
    let original: Value = serde_json::from_slice(include_bytes!(
        "../../../tests/fixtures/pyramid-recon/native-v20/profile.json"
    ))
    .unwrap();
    for value in [
        Value::Bool(false),
        Value::Null,
        Value::String("true".into()),
    ] {
        let mut bad = original.clone();
        bad["board"]["board_count_target_clear"] = value;
        assert!(ReconVision::from_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
    let mut bad = original.clone();
    bad["board"]
        .as_object_mut()
        .unwrap()
        .remove("board_count_final_clear");
    assert!(ReconVision::from_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
    let mut bad = original;
    bad["schema_version"] = 1.into();
    assert!(ReconVision::from_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
}


/// Dropping only the capability and restoring the ID recovers every old calibration byte value.
#[test]
fn target_completion_v20_decoded_rollback_is_exactly_v19() {
    let mut current: Value = serde_json::from_slice(include_bytes!(
        "../../../tests/fixtures/pyramid-recon/native-v20/profile.json"
    ))
    .unwrap();
    current["profile_id"] = "pyramid-native-1920-classic-blue-v19".into();
    current["board"]
        .as_object_mut()
        .unwrap()
        .remove("board_count_target_clear");
    let old: Value = serde_json::from_slice(include_bytes!(
        "../../../tests/fixtures/pyramid-recon/native-v19/profile.json"
    ))
    .unwrap();
    assert_eq!(current, old);
}
