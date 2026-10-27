//! Original testing-table observations; no guest input or new capture is requested.

use super::*;
use crate::pyramid_progress::{ChallengeKind, ChallengeProgress};


/// Decode one untouched native observation retained from the operator's testing table.
fn retained(bytes: &[u8]) -> CapturedFrame {
    crate::capture::decode_png(bytes).expect("retained 156 observation")
}


/// The complete Score word and decimal fraction must identify the actual untouched requirement.
#[test]
fn native_156_score_entry_recognises_actual_zero_of_2500() {
    let frame = retained(include_bytes!(
        "../../../../tests/fixtures/pyramid-testing-156/score-entry.png"
    ));
    let expected = ChallengeProgress::new(ChallengeKind::Score, 0, 2500).unwrap();
    assert_eq!(
        crate::pyramid_progress::observe_challenge(&frame),
        Ok(Some(expected))
    );
}


/// Real paired entry binds the actual Score target and revokes before a second capture.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_156_score_entry_pair_preserves_stop_and_context() {
    let bytes = include_bytes!("../../../../tests/fixtures/pyramid-testing-156/score-entry.png");
    let episode = run(
        Operation::ColourEntry,
        vec![bytes.to_vec(), bytes.to_vec()],
        Revocation::None,
    );
    assert_eq!(
        episode.result,
        Ok(Entry::Challenge(
            ChallengeProgress::new(ChallengeKind::Score, 0, 2500).unwrap()
        ))
    );
    assert_eq!((episode.wire.captures, episode.wire.remaining), (2, 0));
    assert!(episode.retained_frame && episode.owns_context && !episode.cancelled);
    for revocation in [Revocation::Stop, Revocation::ContextRoundTrip] {
        let episode = run(
            Operation::ColourEntry,
            vec![bytes.to_vec(), bytes.to_vec()],
            revocation,
        );
        assert!(
            episode
                .result
                .unwrap_err()
                .contains("STOP or changed context revoked")
        );
        assert_eq!((episode.wire.captures, episode.wire.remaining), (1, 1));
        assert!(episode.retained_frame && episode.cancelled && !episode.owns_context);
    }
}


/// A fresh whole-action observation of the full counting event retires a bound Score route.
#[test]
fn native_156_score_complete_counting_retires_bound_route() {
    let frame = retained(include_bytes!(
        "../../../../tests/fixtures/pyramid-testing-156/score-complete.png"
    ));
    let bound = ChallengeProgress::new(ChallengeKind::Score, 1660, 1700).unwrap();
    let proof = crate::pyramid_progress::observe_bound_challenge_complete(&frame, bound).unwrap();
    assert!(
        proof.is_some(),
        "complete native Score counting modal must be recognised"
    );
    let proof = proof.unwrap();
    assert!(proof.reward_counting());
    assert_eq!(
        proof.progress(),
        ChallengeProgress::new(ChallengeKind::Score, 1755, 1700).unwrap()
    );
}


/// Independent original Q-clubs and K-spades finals require the exact remaining DR.
#[test]
fn native_156_normal_final_clear_preserves_exact_remaining_right_pile() {
    use xsar::games::cards::{Card, Rank, Suit};
    let vision = ReconVision::bundled_native_v23().unwrap();
    for (bytes, right) in [
        (
            include_bytes!("../../../../tests/fixtures/pyramid-testing-156/normal-final.png")
                .as_slice(),
            Card::new(Rank::Queen, Suit::Clubs),
        ),
        (
            include_bytes!(
                "../../../../tests/fixtures/pyramid-testing-156/normal-final-king-spades.png"
            )
            .as_slice(),
            Card::new(Rank::King, Suit::Spades),
        ),
    ] {
        let frame = retained(bytes);
        let result = vision.verify_normal_final_clear(&frame, &[None; 28], None, Some(right));
        assert!(result.is_ok(), "{right:?}: {result:?}");
        assert!(
            vision
                .verify_normal_final_clear(
                    &frame,
                    &[None; 28],
                    None,
                    Some(Card::new(Rank::Ace, Suit::Hearts)),
                )
                .is_err()
        );
    }
}


/// The normal Congratulations layout is independently recognised without a challenge requirement.
#[test]
fn native_156_small_normal_reward_is_normal_evidence() {
    let frame = retained(include_bytes!(
        "../../../../tests/fixtures/pyramid-testing-156/normal-reward-small.png"
    ));
    assert_eq!(post_game::normal_reward::observe(&frame), Ok(true));
}


/// Corrupt a checked native RGB region in memory; no derivative is native calibration.
fn corrupt(frame: &mut CapturedFrame, bounds: PixelRect) {
    frame.as_view().unwrap().checked_bounds(bounds).unwrap();
    for y in bounds.y..bounds.bottom() {
        for x in bounds.x..bounds.right() {
            let offset =
                usize::try_from(y).unwrap() * frame.stride + usize::try_from(x).unwrap() * 4;
            frame.pixels[offset..offset + 3].copy_from_slice(&[255, 0, 255]);
        }
    }
}


/// Whole Score letters and their negative space remain required independently of decimal digits.
#[test]
fn native_156_score_entry_refuses_incomplete_word_and_changed_backdrop() {
    for bounds in [
        PixelRect::new(840, 48, 16, 32),
        PixelRect::new(814, 48, 1, 1),
    ] {
        let mut frame = retained(include_bytes!(
            "../../../../tests/fixtures/pyramid-testing-156/score-entry.png"
        ));
        corrupt(&mut frame, bounds);
        assert_eq!(crate::pyramid_progress::observe_challenge(&frame), Ok(None));
    }
}


/// Complete title, medal, scaffold and skip instruction are jointly mandatory for bound counting.
#[test]
fn native_156_score_counting_requires_complete_event_and_preserves_wrong_target() {
    let bytes = include_bytes!("../../../../tests/fixtures/pyramid-testing-156/score-complete.png");
    let bound = ChallengeProgress::new(ChallengeKind::Score, 1660, 1700).unwrap();
    for bounds in [
        PixelRect::new(640, 164, 310, 64),
        PixelRect::new(960, 164, 322, 64),
        PixelRect::new(489, 111, 110, 152),
        PixelRect::new(1387, 891, 26, 60),
        PixelRect::new(795, 858, 330, 40),
    ] {
        let mut frame = retained(bytes);
        corrupt(&mut frame, bounds);
        assert!(
            crate::pyramid_progress::observe_bound_challenge_complete(&frame, bound)
                .unwrap()
                .is_none()
        );
    }
    let frame = retained(bytes);
    assert!(
        crate::pyramid_progress::observe_challenge_complete(&frame)
            .unwrap()
            .is_none()
    );
    let wrong = ChallengeProgress::new(ChallengeKind::Boards, 1, 3).unwrap();
    let actual = crate::pyramid_progress::observe_bound_challenge_complete(&frame, wrong)
        .unwrap()
        .unwrap()
        .progress();
    assert_eq!(
        actual,
        ChallengeProgress::new(ChallengeKind::Score, 1755, 1700).unwrap()
    );
    assert!(!actual.follows(wrong));
}


/// An exhausted-LEFT replacement never bypasses exact RIGHT, empty replay or independent scene evidence.
#[test]
fn native_156_normal_final_clear_keeps_pile_scene_and_remaining_card_vetoes() {
    use xsar::games::cards::{Card, Rank, Suit};
    let bytes = include_bytes!("../../../../tests/fixtures/pyramid-testing-156/normal-final.png");
    let frame = retained(bytes);
    let vision = ReconVision::bundled_native_v23().unwrap();
    let right = Some(Card::new(Rank::Queen, Suit::Clubs));
    assert!(
        vision
            .verify_normal_final_clear(
                &frame,
                &[None; 28],
                None,
                Some(Card::new(Rank::Queen, Suit::Spades))
            )
            .is_err()
    );
    let mut expected = [None; 28];
    expected[0] = Some(Card::new(Rank::Six, Suit::Hearts));
    assert!(
        vision
            .verify_normal_final_clear(&frame, &expected, None, right)
            .is_err()
    );
    for bounds in [
        PixelRect::new(767, 712, 60, 60),
        PixelRect::new(1796, 116, 109, 290),
    ] {
        let mut frame = retained(bytes);
        corrupt(&mut frame, bounds);
        assert!(
            vision
                .verify_normal_final_clear(&frame, &[None; 28], None, right)
                .is_err(),
            "corrupted {bounds:?} remained accepted"
        );
    }
    assert!(
        vision
            .verify_position(&frame, &[None; 28], None, right)
            .is_err()
    );
}


/// Normal counting requires its own title, skip, ordinal and Pyramid legend, with no challenge substitution.
#[test]
fn native_156_small_normal_reward_requires_complete_normal_scene() {
    let bytes =
        include_bytes!("../../../../tests/fixtures/pyramid-testing-156/normal-reward-small.png");
    for bounds in [
        PixelRect::new(782, 306, 360, 50),
        PixelRect::new(845, 746, 230, 30),
        PixelRect::new(1658, 46, 160, 32),
        PixelRect::new(1796, 116, 109, 290),
    ] {
        let mut frame = retained(bytes);
        corrupt(&mut frame, bounds);
        assert_eq!(post_game::normal_reward::observe(&frame), Ok(false));
    }
    let challenge = retained(include_bytes!(
        "../../../../tests/fixtures/pyramid-testing-156/score-complete.png"
    ));
    assert_eq!(post_game::normal_reward::observe(&challenge), Ok(false));
}


/// A supplied third-clear permit routes the small reward through real restart stages.
///
/// The permit and post-input frames are isolated test inputs, not live completion
/// evidence. The production adapter retains fresh probes and revocation checks.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_156_small_normal_reward_restart_preserves_authority() {
    let frame = retained(include_bytes!(
        "../../../../tests/fixtures/pyramid-testing-156/normal-reward-small.png"
    ));
    let fixture = Fixture::new();
    assert_eq!(
        fixture.adapter().observe(&frame),
        Ok(NativeObservation::Unknown)
    );
    let episode = restart_episode(
        RestartEntry::WitnessedSmallNormalReward,
        NORMAL_ONE,
        RestartBoundary::None,
    );
    let progress = BoardCount {
        completed: 0,
        target: 3,
    };
    assert_eq!(restarted_progress(episode.result.unwrap()), Ok(progress));
    assert_eq!(episode.series, Some(progress));
    assert_eq!(episode.latest.unwrap().pixels, native(NORMAL_ONE).pixels);
    assert!(
        episode.starting.is_some()
            && episode.permit_consumed
            && episode.owns_context
            && !episode.cancelled
    );
    assert_eq!(episode.wire.connections, 3);
    assert_eq!(episode.wire.captures, [2, 10, 2]);
    assert_eq!(
        (
            episode.wire.probes,
            episode.wire.ups,
            episode.wire.remaining
        ),
        (5, 5, 0)
    );
    let skip = restart_wire_point(crate::parameters::SCORE_SKIP_CONTROL.click_point);
    let ok = restart_wire_point(
        control_point(&native(TERMINAL), PYRAMID_CONTROLS.spec(Control::Ok)).unwrap(),
    );
    let new_game = restart_wire_point(
        control_point(
            &synthetic_restart_menu(Control::NewGame),
            PYRAMID_CONTROLS.spec(Control::NewGame),
        )
        .unwrap(),
    );
    let play = restart_wire_point(
        control_point(
            &synthetic_restart_menu(Control::Play),
            PYRAMID_CONTROLS.spec(Control::Play),
        )
        .unwrap(),
    );
    assert_eq!(episode.wire.downs, vec![skip, ok, skip, new_game, play]);
    for revocation in [Revocation::Stop, Revocation::ContextRoundTrip] {
        let episode = restart_episode(
            RestartEntry::WitnessedSmallNormalReward,
            NORMAL_ONE,
            RestartBoundary::LaterProbe(3, revocation),
        );
        assert!(episode.result.is_err());
        assert_eq!(episode.wire.connections, 2);
        assert_eq!(episode.wire.captures, [2, 6, 0]);
        assert_eq!(
            (
                episode.wire.probes,
                episode.wire.downs.len(),
                episode.wire.ups
            ),
            (3, 2, 2)
        );
        assert!(episode.starting.is_none());
    }
}
