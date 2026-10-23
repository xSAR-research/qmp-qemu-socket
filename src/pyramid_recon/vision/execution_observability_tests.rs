//! Original 132 execution regressions; direct card truth and labelled negative derivatives.

use super::*;
use crate::pyramid_progress::{ChallengeKind, ChallengeProgress};


/// Decode a fixed immutable original, never manufacturing a live acquisition receipt.
fn frame(name: &str) -> CapturedFrame {
    crate::capture::decode_png(
        &std::fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/pyramid-execution-observability")
                .join(name),
        )
        .unwrap(),
    )
    .unwrap()
}


/// Admit the measured zero-deal stock layout without making the apex or ordinary Recon empty.
#[test]
fn execution_observability_apex_requires_exact_cards_empty_piles_and_complete_native_signatures() {
    let reader = ReconVision::bundled_native_v22().unwrap();
    let original = frame("apex.png");
    let mut expected = [None; 28];
    expected[0] = Some(Card::new(Rank::King, Suit::Clubs));
    reader
        .verify_position(&original, &expected, None, None)
        .unwrap();
    assert!(!matches!(
        reader.observe(&original),
        Ok(ReconScene::Board(_))
    ));
    assert!(
        reader
            .verify_final_clear(&original, &[None; 28], None, None)
            .is_err()
    );
    let custom = ReconVision::from_bytes(include_bytes!(
        "../../../tests/fixtures/pyramid-recon/native-v22/profile.json"
    ))
    .unwrap();
    assert!(
        custom
            .verify_position(&original, &expected, None, None)
            .is_err()
    );
    expected[0] = Some(Card::new(Rank::King, Suit::Diamonds));
    assert!(
        reader
            .verify_position(&original, &expected, None, None)
            .is_err()
    );
    expected[0] = Some(Card::new(Rank::King, Suit::Clubs));
    for (x, y, width, height) in [
        (931, 695, 60, 50),
        (890, 874, 140, 26),
        (767, 712, 60, 60),
        (1022, 678, 60, 60),
        (1796, 116, 109, 18),
    ] {
        let mut bad = original.clone();
        for row in y..y + height {
            for column in x..x + width {
                let offset = row * bad.stride + column * 4;
                bad.pixels[offset..offset + 3].fill(255);
            }
        }
        assert!(
            reader.verify_position(&bad, &expected, None, None).is_err(),
            "{x},{y}"
        );
    }
    assert!(
        reader
            .verify_position(
                &original,
                &expected,
                Some(Card::new(Rank::Ace, Suit::Clubs)),
                None
            )
            .is_err()
    );
}


/// A moving Perfect overlay can prove only the bound fully replayed card effect, not its target.
#[test]
fn execution_observability_score_top_bands_require_same_incomplete_target_and_exact_position() {
    let reader = ReconVision::bundled_native_v22().unwrap();
    let original = frame("score-final.png");
    let initial = crate::pyramid_progress::observe_challenge(&frame("score-start.png"))
        .unwrap()
        .unwrap();
    let current = crate::pyramid_progress::observe_challenge(&original)
        .unwrap()
        .unwrap();
    assert_eq!(
        (initial.kind, initial.target.get()),
        (ChallengeKind::Score, 2900)
    );
    assert_eq!(current.current, 2510);
    assert!(!current.target_reached());
    let left = Some(Card::new(Rank::King, Suit::Diamonds));
    reader
        .verify_challenge_final_clear(&original, &[None; 28], left, None, initial)
        .unwrap();
    assert!(!crate::pyramid_progress::is_challenge_complete(&original).unwrap());
    let custom = ReconVision::from_bytes(include_bytes!(
        "../../../tests/fixtures/pyramid-recon/native-v22/profile.json"
    ))
    .unwrap();
    assert!(
        custom
            .verify_challenge_final_clear(&original, &[None; 28], left, None, initial)
            .is_err()
    );
    for wrong in [
        ChallengeProgress::new(ChallengeKind::Score, 0, 2800).unwrap(),
        ChallengeProgress::new(ChallengeKind::Score, 2900, 2900).unwrap(),
        ChallengeProgress::new(ChallengeKind::Boards, 0, 3).unwrap(),
    ] {
        assert!(
            reader
                .verify_challenge_final_clear(&original, &[None; 28], left, None, wrong)
                .is_err()
        );
    }
    for (index, (x, y)) in TABLEAU_ORIGINS.into_iter().enumerate() {
        let mut remaining = [None; 28];
        remaining[index] = Some(Card::new(Rank::Ace, Suit::Clubs));
        assert!(
            reader
                .verify_challenge_final_clear(&original, &remaining, left, None, initial)
                .is_err()
        );
        let mut damaged = original.clone();
        let offset = (y as usize + 4) * damaged.stride + (x as usize + 40) * 4;
        damaged.pixels[offset..offset + 3].fill(255);
        assert!(
            reader
                .verify_challenge_final_clear(&damaged, &[None; 28], left, None, initial)
                .is_err(),
            "slot {index}"
        );
    }
    assert!(
        reader
            .verify_challenge_final_clear(&original, &[None; 28], None, None, initial)
            .is_err()
    );
}
