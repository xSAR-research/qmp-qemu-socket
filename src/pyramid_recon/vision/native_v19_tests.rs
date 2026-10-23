//! Exposed native Boards-clear regressions; no guest or fresh-executable live proof.

use super::*;
use crate::pyramid_progress::{ChallengeKind, ChallengeProgress};


/// Decode one unchanged native 121 failure frame from its permanent fixture.
fn frame(name: &str) -> CapturedFrame {
    let bytes = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/pyramid-recon/native-v19")
            .join(name),
    )
    .unwrap();
    crate::capture::decode_png(&bytes).unwrap()
}


/// Original matched requirement for the first board of the two-board trial.
fn initial() -> ChallengeProgress {
    ChallengeProgress::new(ChallengeKind::Boards, 0, 2).unwrap()
}


/// Exact replayed final pile identities, not inferred from the displayed counter.
fn piles() -> (Option<Card>, Option<Card>) {
    (
        Some(Card::new(Rank::King, Suit::Spades)),
        Some(Card::new(Rank::Nine, Suit::Spades)),
    )
}


/// Paint only an explicitly synthetic refusal derivative in checked native storage.
fn paint(frame: &mut CapturedFrame, bounds: PixelRect, colour: [u8; 3]) {
    frame.as_view().unwrap().checked_bounds(bounds).unwrap();
    for y in bounds.y..bounds.bottom() {
        for x in bounds.x..bounds.right() {
            let offset = y as usize * frame.stride + x as usize * 4;
            frame.pixels[offset..offset + 3].copy_from_slice(&colour);
        }
    }
}


/// Both animation stages prove only the bound intermediate clear, never board readiness.
#[test]
fn native_intermediate_board_count_clear_requires_new_capability_and_exact_piles() {
    let reader = ReconVision::bundled_native_v19().unwrap();
    let previous = ReconVision::bundled_native_v18().unwrap();
    let (left, right) = piles();
    for name in ["first-unmatched.png", "last-observed.png"] {
        let frame = frame(name);
        let actual = crate::pyramid_progress::observe_challenge(&frame)
            .unwrap()
            .unwrap();
        assert_eq!(
            actual,
            ChallengeProgress::new(ChallengeKind::Boards, 1, 2).unwrap()
        );
        assert!(
            reader
                .verify_position(&frame, &[None; 28], left, right)
                .is_err()
        );
        assert!(
            reader
                .verify_final_clear(&frame, &[None; 28], left, right)
                .is_err()
        );
        assert!(
            reader
                .verify_normal_final_clear(&frame, &[None; 28], left, right)
                .is_err()
        );
        assert!(
            previous
                .verify_board_count_final_clear(&frame, &[None; 28], left, right, initial())
                .is_err()
        );
        reader
            .verify_board_count_final_clear(&frame, &[None; 28], left, right, initial())
            .unwrap();
        assert!(!matches!(reader.observe(&frame), Ok(ReconScene::Board(_))));
        for (bad_left, bad_right) in [(None, right), (left, None), (right, left)] {
            assert!(
                reader
                    .verify_board_count_final_clear(
                        &frame,
                        &[None; 28],
                        bad_left,
                        bad_right,
                        initial()
                    )
                    .is_err()
            );
        }
    }
}


/// Every remaining expected card and every single unexplained band pixel refuses.
#[test]
fn board_count_alone_never_replaces_all_28_positive_absence_bands_or_replay() {
    let reader = ReconVision::bundled_native_v19().unwrap();
    let original = frame("last-observed.png");
    let (left, right) = piles();
    for (index, (x, y)) in TABLEAU_ORIGINS.iter().copied().enumerate() {
        let mut expected = [None; 28];
        expected[index] = Some(Card::new(Rank::Ace, Suit::Clubs));
        assert!(
            reader
                .verify_board_count_final_clear(&original, &expected, left, right, initial())
                .is_err()
        );
        let mut damaged = original.clone();
        paint(&mut damaged, PixelRect::new(x + 40, y + 4, 1, 1), [255; 3]);
        assert!(
            reader
                .verify_board_count_final_clear(&damaged, &[None; 28], left, right, initial())
                .is_err(),
            "slot {index}"
        );
    }
}


/// Stale, regressed, different-target and other-kind bindings cannot establish a clear.
#[test]
fn board_count_clear_requires_exactly_one_increment_of_the_same_incomplete_requirement() {
    let reader = ReconVision::bundled_native_v19().unwrap();
    let frame = frame("last-observed.png");
    let (left, right) = piles();
    for bound in [
        ChallengeProgress::new(ChallengeKind::Boards, 1, 2).unwrap(),
        ChallengeProgress::new(ChallengeKind::Boards, 2, 2).unwrap(),
        ChallengeProgress::new(ChallengeKind::Boards, 0, 3).unwrap(),
        ChallengeProgress::new(ChallengeKind::Score, 0, 2).unwrap(),
        ChallengeProgress::new(ChallengeKind::Cards, 0, 2).unwrap(),
    ] {
        assert!(
            reader
                .verify_board_count_final_clear(&frame, &[None; 28], left, right, bound)
                .is_err(),
            "{bound:?}"
        );
    }
}


/// A synthetic complete 3/3 header retains the empty pixels but cannot prove a terminal.
#[test]
fn completed_target_header_cannot_use_intermediate_board_clear() {
    let reader = ReconVision::bundled_native_v19().unwrap();
    let mut frame = frame("last-observed.png");
    let terminal = crate::capture::decode_png(include_bytes!(
        "../../../tests/fixtures/pyramid-progress/negative-three-of-three.png"
    ))
    .unwrap();
    let bounds = PixelRect::new(814, 40, 296, 48);
    for y in bounds.y..bounds.bottom() {
        for x in bounds.x..bounds.right() {
            let destination = y as usize * frame.stride + x as usize * 4;
            let source = y as usize * terminal.stride + x as usize * 4;
            frame.pixels[destination..destination + 4]
                .copy_from_slice(&terminal.pixels[source..source + 4]);
        }
    }
    assert_eq!(
        crate::pyramid_progress::observe_challenge(&frame).unwrap(),
        ChallengeProgress::new(ChallengeKind::Boards, 3, 3)
    );
    let (left, right) = piles();
    assert!(
        reader
            .verify_board_count_final_clear(
                &frame,
                &[None; 28],
                left,
                right,
                ChallengeProgress::new(ChallengeKind::Boards, 2, 3).unwrap()
            )
            .is_err()
    );
}


/// Identity, independent scene, exact piles, modal veto and frame layout remain required.
#[test]
fn board_count_clear_keeps_scene_modal_pile_and_frame_layout_refusals() {
    let reader = ReconVision::bundled_native_v19().unwrap();
    let original = frame("last-observed.png");
    let (left, right) = piles();
    for bounds in [
        PixelRect::new(839, 50, 96, 28),
        PixelRect::new(1038, 48, 72, 32),
        PixelRect::new(1796, 116, 109, 18),
        PixelRect::new(1796, 388, 109, 20),
        PixelRect::new(931, 695, 60, 50),
        PixelRect::new(767, 712, 60, 60),
        PixelRect::new(1022, 678, 60, 60),
    ] {
        let mut damaged = original.clone();
        paint(&mut damaged, bounds, [30, 80, 20]);
        assert!(
            reader
                .verify_board_count_final_clear(&damaged, &[None; 28], left, right, initial())
                .is_err(),
            "{bounds:?}"
        );
    }
    let mut modal = original.clone();
    for pattern in &reader.dialog.signature {
        for (index, pixel) in pattern.pixels.iter().enumerate() {
            if let Some(colour) = pixel {
                let x = pattern.bounds.x as usize + index % pattern.bounds.width as usize;
                let y = pattern.bounds.y as usize + index / pattern.bounds.width as usize;
                let offset = y * modal.stride + x * 4;
                modal.pixels[offset..offset + 3].copy_from_slice(colour);
            }
        }
    }
    assert!(
        reader
            .dialog
            .matches(modal.as_view().unwrap(), true)
            .unwrap()
    );
    assert!(
        reader
            .verify_board_count_final_clear(&modal, &[None; 28], left, right, initial())
            .is_err()
    );
    let mut malformed = original.clone();
    malformed.pixels.truncate(17);
    assert!(
        reader
            .verify_board_count_final_clear(&malformed, &[None; 28], left, right, initial())
            .is_err()
    );
    let mut resized = original;
    resized.width = 1919;
    assert!(
        reader
            .verify_board_count_final_clear(&resized, &[None; 28], left, right, initial())
            .is_err()
    );
}


/// The opt-in requires schema 2, a literal true and all original final calibration.
#[test]
fn board_count_capability_parser_refuses_implicit_or_weakened_admission() {
    let original: Value = serde_json::from_slice(include_bytes!(
        "../../../tests/fixtures/pyramid-recon/native-v19/profile.json"
    ))
    .unwrap();
    for replacement in [
        Value::Bool(false),
        Value::Null,
        Value::String("true".into()),
    ] {
        let mut value = original.clone();
        value["board"]["board_count_final_clear"] = replacement;
        assert!(ReconVision::from_bytes(&serde_json::to_vec(&value).unwrap()).is_err());
    }
    let mut value = original.clone();
    value["board"]
        .as_object_mut()
        .unwrap()
        .remove("final_clear");
    assert!(ReconVision::from_bytes(&serde_json::to_vec(&value).unwrap()).is_err());
    let mut value = original;
    value["schema_version"] = 1.into();
    assert!(ReconVision::from_bytes(&serde_json::to_vec(&value).unwrap()).is_err());
}


/// Removing only the explicit capability and version identifier exactly recovers v18.
#[test]
fn v19_rollback_preserves_every_v18_glyph_scene_and_final_clear_input() {
    let mut value: Value = serde_json::from_slice(include_bytes!(
        "../../../tests/fixtures/pyramid-recon/native-v19/profile.json"
    ))
    .unwrap();
    value["profile_id"] = "pyramid-native-1920-classic-blue-v18".into();
    value["board"]
        .as_object_mut()
        .unwrap()
        .remove("board_count_final_clear");
    let previous: Value = serde_json::from_slice(include_bytes!(
        "../../../tests/fixtures/pyramid-recon/native-v18/profile.json"
    ))
    .unwrap();
    assert_eq!(value, previous);
}
