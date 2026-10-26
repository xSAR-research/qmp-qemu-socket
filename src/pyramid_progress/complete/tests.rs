//! Genuine native terminal positives and explicitly corrupted/animated negatives.

use super::*;
use crate::capture::decode_png;

/// The genuine Star Club terminal is an exact completed Boards requirement with operator Continue.
#[test]
fn target_completion_star_club_literal_and_operator_control_are_independent_of_counting() {
    let frame = decode_png(include_bytes!(
        "../../../tests/fixtures/pyramid-objectives/boards-complete-continue.png"
    ))
    .unwrap();
    let proof = observe(&frame).unwrap().unwrap();
    let target = ChallengeProgress::new(ChallengeKind::Boards, 2, 2).unwrap();
    assert_eq!(proof.progress(), target);
    assert!(proof.operator_continue());
    assert!(!proof.reward_counting());
    assert_eq!(
        observe_bound(
            &frame,
            ChallengeProgress::new(ChallengeKind::Score, 2, 2).unwrap()
        )
        .unwrap()
        .unwrap()
        .progress(),
        target,
        "the actual literal header overrides a conflicting binding"
    );
    let counting = observe(&decode_png(FOURS).unwrap()).unwrap().unwrap();
    assert!(counting.reward_counting());
    assert!(!counting.operator_continue());
}


/// A missing or corrupted button never becomes authority to skip counting.
#[test]
fn target_completion_unknown_control_has_neither_continue_nor_skip_authority() {
    let mut frame = decode_png(include_bytes!(
        "../../../tests/fixtures/pyramid-objectives/boards-complete-continue.png"
    ))
    .unwrap();
    let offset = 842 * frame.stride + 804 * 4;
    frame.pixels[offset] ^= 128;
    let proof = observe(&frame).unwrap().unwrap();
    assert!(proof.progress().target_reached());
    assert!(!proof.operator_continue());
    assert!(!proof.reward_counting());
}



/// Real settled Fours terminal captured before the planned route had ended.
const FOURS: &[u8] =
    include_bytes!("../../../tests/fixtures/pyramid-objectives/fours-complete.png");


/// Independent older native Aces terminal; already retained as a normal-header negative.
const ACES: &[u8] =
    include_bytes!("../../../tests/fixtures/pyramid-progress/negative-aces-complete.png");


/// Both actual originals keep Card units and their positively observed reached requirement.
#[test]
fn two_native_complete_terminals_keep_the_exact_requirement() {
    for bytes in [FOURS, ACES] {
        let frame = decode_png(bytes).unwrap();
        let completion = observe(&frame)
            .unwrap()
            .expect("genuine settled Challenge Complete");
        assert_eq!(
            completion.progress(),
            ChallengeProgress::new(ChallengeKind::Cards, 12, 12).unwrap()
        );
        assert!(completion.progress().target_reached());
        assert_eq!(crate::pyramid_progress::observe_challenge(&frame), Ok(None));
        assert_eq!(crate::pyramid_progress::observe_normal(&frame), Ok(None));
    }
}


/// Perfect, flying cards, dealing boards and an animated smaller modal are not completion.
#[test]
fn ordinary_transition_perfect_and_animated_frames_do_not_become_success() {
    for bytes in [
        include_bytes!("../../../tests/fixtures/pyramid-objectives/fours-four-of-twelve.png")
            .as_slice(),
        include_bytes!("../../../tests/fixtures/pyramid-objectives/boards-one-of-two-perfect.png")
            .as_slice(),
        include_bytes!("../../../tests/fixtures/pyramid-objectives/fours-four-flying.png")
            .as_slice(),
        include_bytes!("../../../tests/fixtures/pyramid-objectives/fours-four-next-dealing.png")
            .as_slice(),
        include_bytes!("../../../tests/fixtures/pyramid-objectives/fours-eight-flying.png")
            .as_slice(),
        include_bytes!("../../../tests/fixtures/pyramid-objectives/fours-eight-next-dealing.png")
            .as_slice(),
        include_bytes!("../../../tests/fixtures/pyramid-objectives/fours-complete-animating.png")
            .as_slice(),
        include_bytes!("../../../tests/fixtures/pyramid-objectives/normal-one.png").as_slice(),
        include_bytes!("../../../tests/fixtures/pyramid-objectives/solver-layout.png").as_slice(),
        include_bytes!("../../../tests/fixtures/ok-confirmation-20261016/pyramid/clean.png")
            .as_slice(),
    ] {
        assert_eq!(observe(&decode_png(bytes).unwrap()), Ok(None));
    }
}


/// Every complete required patch participates; valid title alone supplies no goal.
#[test]
fn erasing_each_title_scaffold_or_requirement_refuses() {
    for bounds in [
        PixelRect::new(648, 168, 64, 60),
        PixelRect::new(1000, 168, 64, 60),
        PixelRect::new(1387, 891, 26, 60),
        PixelRect::new(814, 48, 296, 32),
    ] {
        let mut frame = decode_png(FOURS).unwrap();
        for y in bounds.y..bounds.bottom() {
            for x in bounds.x..bounds.right() {
                let i = y as usize * frame.stride + x as usize * 4;
                frame.pixels[i..i + 3].fill(0);
            }
        }
        assert_eq!(observe(&frame), Ok(None));
    }
}


/// Invalid storage and unsupported dimensions cannot carry terminal authority.
#[test]
fn malformed_layout_and_unsupported_geometry_do_not_supply_completion() {
    let mut malformed = decode_png(FOURS).unwrap();
    malformed.pixels.pop();
    assert!(observe(&malformed).is_err());
    let mut unsupported = decode_png(FOURS).unwrap();
    unsupported.height -= 1;
    assert_eq!(observe(&unsupported), Ok(None));
}


/// The actual small counting modal is positive only with a retained challenge binding.
/// Erasing any independent required region prevents completion; no numeric target is invented.
#[test]
fn native_141_small_counting_modal_requires_all_patches_and_retained_binding() {
    let frame = decode_png(include_bytes!(
        "../../../tests/fixtures/pyramid-native-141/jacks-small-complete.png"
    ))
    .unwrap();
    let bound = ChallengeProgress::new(ChallengeKind::Cards, 8, 10).unwrap();
    assert_eq!(observe(&frame), Ok(None));
    assert!(!is_complete(&frame).unwrap());
    let proof = observe_bound(&frame, bound).unwrap().unwrap();
    assert_eq!(proof.progress(), bound);
    assert!(proof.terminal_progress.is_none());
    assert!(proof.reward_counting());
    assert!(!proof.operator_continue());
    for (x, y) in [(850, 400), (800, 395), (1120, 660), (950, 660)] {
        let mut bad = frame.clone();
        let offset = y * bad.stride + x * 4;
        bad.pixels[offset] ^= 128;
        assert!(observe_bound(&bad, bound).unwrap().is_none(), "{x},{y}");
    }
}


/// An existing literal requirement remains authoritative on the new small modal.
/// Only the whole calibrated header is composed; the terminal geometry is unchanged.
#[test]
fn native_142_small_modal_keeps_calibrated_literal_over_conflicting_binding() {
    let mut frame = decode_png(include_bytes!(
        "../../../tests/fixtures/pyramid-native-141/jacks-small-complete.png"
    ))
    .unwrap();
    let rgb =
        include_bytes!("../../../tests/fixtures/pyramid-objectives/complete-boards-two-header.rgb");
    for y in 0..32 {
        for x in 0..296 {
            let source = (y * 296 + x) * 3;
            let destination = (48 + y) * frame.stride + (814 + x) * 4;
            frame.pixels[destination..destination + 3].copy_from_slice(&rgb[source..source + 3]);
        }
    }
    let bound = ChallengeProgress::new(ChallengeKind::Cards, 8, 10).unwrap();
    let actual = ChallengeProgress::new(ChallengeKind::Boards, 2, 2).unwrap();
    assert_eq!(
        observe(&frame),
        Ok(None),
        "cold entry still requires settled geometry"
    );
    let proof = observe_bound(&frame, bound).unwrap().unwrap();
    assert_eq!(proof.progress(), actual);
    assert_eq!(proof.terminal_progress, Some(actual));
    assert!(proof.reward_counting());
    assert!(!proof.progress().follows(bound));
}


/// A genuine Loading completion retires input using its actual Score literal.
#[test]
fn native_150_score_loading_complete_keeps_actual_counter_and_no_control_authority() {
    let frame = decode_png(include_bytes!(
        "../../../tests/fixtures/pyramid-score-loading-150/last-observed.png"
    ))
    .unwrap();
    let bound = ChallengeProgress::new(ChallengeKind::Score, 1260, 1900).unwrap();
    assert_eq!(
        observe(&frame),
        Ok(None),
        "the new Loading appearance is bound-only"
    );
    let proof = observe_bound(&frame, bound)
        .unwrap()
        .expect("actual Score Loading completion");
    let actual = ChallengeProgress::new(ChallengeKind::Score, 2015, 1900).unwrap();
    assert_eq!(proof.progress(), actual);
    assert_eq!(proof.terminal_progress, Some(actual));
    assert!(!proof.operator_continue());
    assert!(!proof.reward_counting());
    let contradiction = observe_bound(
        &frame,
        ChallengeProgress::new(ChallengeKind::Cards, 4, 5).unwrap(),
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        contradiction.progress(),
        actual,
        "positive actual requirement overrides conflicting binding"
    );
    for bounds in [
        PixelRect::new(640, 164, 310, 64),
        PixelRect::new(960, 164, 322, 64),
        PixelRect::new(489, 111, 110, 152),
        PixelRect::new(1387, 891, 26, 60),
        PixelRect::new(859, 875, 123, 33),
    ] {
        let mut damaged = frame.clone();
        for y in bounds.y..bounds.bottom() {
            for x in bounds.x..bounds.right() {
                let i = y as usize * damaged.stride + x as usize * 4;
                damaged.pixels[i..i + 3].fill(0);
            }
        }
        assert_eq!(
            observe_bound(&damaged, bound),
            Ok(None),
            "damaged required Loading patch {bounds:?}"
        );
    }
}
