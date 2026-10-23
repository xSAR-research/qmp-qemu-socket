//! Exposed native 123 failures; no fixture is a new guest trial or receipt.

use super::*;
use crate::pyramid_progress::{ChallengeKind, ChallengeProgress};


/// Decode an unchanged original from the retained native packet.
fn frame(name: &str) -> CapturedFrame {
    crate::capture::decode_png(
        &std::fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/pyramid-recon/native-v21")
                .join(name),
        )
        .unwrap(),
    )
    .unwrap()
}


/// Read exact pile identities independently of the positive empty tableau bands.
fn piles(reader: &ReconVision, frame: &CapturedFrame) -> (Option<Card>, Option<Card>) {
    let view = frame.as_view().unwrap();
    let read = |bounds, empty| match reader.stock_face(view, bounds, empty).unwrap() {
        StockFace::Card(card) => Some(card),
        StockFace::Empty => None,
    };
    (
        read(LEFT_BOUNDS, &reader.board.left_empty),
        read(RIGHT_BOUNDS, &reader.board.right_empty),
    )
}


/// Both actual clear frames require new opt-in; their first flying frames still refuse.
#[test]
fn native_123_clear_frames_require_bound_native_v21_proof() {
    let reader = ReconVision::bundled_native_v21().unwrap();
    let old = ReconVision::bundled_native_v20().unwrap();
    for (case, initial, current) in [
        (
            "boards",
            ChallengeProgress::new(ChallengeKind::Boards, 2, 3).unwrap(),
            ChallengeProgress::new(ChallengeKind::Boards, 3, 3).unwrap(),
        ),
        (
            "score",
            ChallengeProgress::new(ChallengeKind::Score, 0, 1400).unwrap(),
            ChallengeProgress::new(ChallengeKind::Score, 1260, 1400).unwrap(),
        ),
    ] {
        let last = frame(&format!("{case}-last-observed.png"));
        let (left, right) = piles(&reader, &last);
        assert_eq!(
            crate::pyramid_progress::observe_challenge(&last).unwrap(),
            Some(current)
        );
        assert!(
            reader
                .verify_position(&last, &[None; 28], left, right)
                .is_err()
        );
        assert!(
            old.verify_challenge_final_clear(&last, &[None; 28], left, right, initial)
                .is_err()
        );
        reader
            .verify_challenge_final_clear(&last, &[None; 28], left, right, initial)
            .unwrap();
        assert!(
            reader
                .verify_challenge_final_clear(
                    &frame(&format!("{case}-first-unmatched.png")),
                    &[None; 28],
                    left,
                    right,
                    initial
                )
                .is_err()
        );
        assert!(!crate::pyramid_progress::is_challenge_complete(&last).unwrap());
        assert!(!matches!(reader.observe(&last), Ok(ReconScene::Board(_))));
    }
}


/// The new fixed lower-row band retains all 240 pixels and rejects every remaining card.
#[test]
fn native_123_all_slots_and_every_band_pixel_remain_required() {
    let reader = ReconVision::bundled_native_v21().unwrap();
    let original = frame("boards-last-observed.png");
    let initial = ChallengeProgress::new(ChallengeKind::Boards, 2, 3).unwrap();
    let (left, right) = piles(&reader, &original);
    reader
        .verify_challenge_final_clear(&original, &[None; 28], left, right, initial)
        .unwrap();
    for (index, (x, y)) in TABLEAU_ORIGINS.into_iter().enumerate() {
        let mut expected = [None; 28];
        expected[index] = Some(Card::new(Rank::Ace, Suit::Clubs));
        assert!(
            reader
                .verify_challenge_final_clear(&original, &expected, left, right, initial)
                .is_err()
        );
        let mut bad = original.clone();
        let y_offset = if index >= 21 { 168 } else { 4 };
        let offset = (y as usize + y_offset) * bad.stride + (x as usize + 40) * 4;
        bad.pixels[offset..offset + 3].fill(255);
        assert!(
            reader
                .verify_challenge_final_clear(&bad, &[None; 28], left, right, initial)
                .is_err()
        );
    }
}


/// No changed/regressed/already-complete binding, wrong piles or scene can borrow clear proof.
#[test]
fn native_123_clear_rejects_wrong_requirements_piles_and_scene() {
    let reader = ReconVision::bundled_native_v21().unwrap();
    let last = frame("score-last-observed.png");
    let (left, right) = piles(&reader, &last);
    for (kind, current, target) in [
        (ChallengeKind::Score, 0, 1500),
        (ChallengeKind::Score, 1300, 1400),
        (ChallengeKind::Score, 1400, 1400),
        (ChallengeKind::Boards, 0, 1400),
        (ChallengeKind::Cards, 0, 1400),
    ] {
        assert!(
            reader
                .verify_challenge_final_clear(
                    &last,
                    &[None; 28],
                    left,
                    right,
                    ChallengeProgress::new(kind, current, target).unwrap()
                )
                .is_err()
        );
    }
    let initial = ChallengeProgress::new(ChallengeKind::Score, 0, 1400).unwrap();
    assert!(
        reader
            .verify_challenge_final_clear(&last, &[None; 28], None, right, initial)
            .is_err()
    );
    assert!(
        reader
            .verify_challenge_final_clear(&last, &[None; 28], left, None, initial)
            .is_err()
    );
    for pattern in &reader.board.scene {
        let mut bad = last.clone();
        let offset = pattern.bounds.y as usize * bad.stride + pattern.bounds.x as usize * 4;
        bad.pixels[offset] ^= 128;
        assert!(
            reader
                .verify_challenge_final_clear(&bad, &[None; 28], left, right, initial)
                .is_err()
        );
    }
}


/// Actual Twos 0/10 and Fours 12/12/Continue use exact new appearances, not new thresholds.
#[test]
fn native_123_twos_entry_and_fours_operator_endpoint_are_recognised() {
    let twos = frame("twos-last-observed.png");
    assert_eq!(
        crate::pyramid_progress::observe_challenge(&twos).unwrap(),
        ChallengeProgress::new(ChallengeKind::Cards, 0, 10)
    );
    let last = frame("fours-last-observed.png");
    let proof = crate::pyramid_progress::observe_challenge_complete(&last)
        .unwrap()
        .unwrap();
    assert_eq!(
        proof.progress(),
        ChallengeProgress::new(ChallengeKind::Cards, 12, 12).unwrap()
    );
    assert!(proof.operator_continue());
    assert!(!proof.reward_counting());
    let mut bad = last;
    let offset = 891 * bad.stride + 1387 * 4;
    bad.pixels[offset] ^= 128;
    assert!(!crate::pyramid_progress::is_challenge_complete(&bad).unwrap());
}


/// Removing only the new flag/id yields v20 exactly; weakened schema/opt-in still refuses.
#[test]
fn native_123_profile_rolls_back_exactly_and_requires_strict_opt_in() {
    let original: Value = serde_json::from_slice(include_bytes!(
        "../../../tests/fixtures/pyramid-recon/native-v21/profile.json"
    ))
    .unwrap();
    for value in [
        Value::Bool(false),
        Value::Null,
        Value::String("true".to_owned()),
    ] {
        let mut bad = original.clone();
        bad["board"]["challenge_final_clear"] = value;
        assert!(ReconVision::from_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
    let mut missing = original.clone();
    missing["board"]
        .as_object_mut()
        .unwrap()
        .remove("board_count_target_clear");
    assert!(ReconVision::from_bytes(&serde_json::to_vec(&missing).unwrap()).is_err());
    let mut rollback = original;
    rollback["board"]
        .as_object_mut()
        .unwrap()
        .remove("challenge_final_clear");
    rollback["profile_id"] = Value::String("pyramid-native-1920-classic-blue-v20".to_owned());
    let prior: Value = serde_json::from_slice(include_bytes!(
        "../../../tests/fixtures/pyramid-recon/native-v20/profile.json"
    ))
    .unwrap();
    assert_eq!(rollback, prior);
}
