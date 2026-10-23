//! Actual-goal lifetime tests use synthetic orchestration and a retained native terminal.

use super::*;
use crate::pyramid_progress::ChallengeKind;


/// Construct explicit synthetic objective facts, never native calibration evidence.
fn requirement(kind: ChallengeKind, current: u32, target: u32) -> ChallengeProgress {
    ChallengeProgress::new(kind, current, target).unwrap()
}


/// Obtain terminal proof through the production reader from the real retained Fours frame.
fn fours_complete() -> ChallengeCompletion {
    let frame = crate::capture::decode_png(include_bytes!(
        "../../../../tests/fixtures/pyramid-objectives/fours-complete.png"
    ))
    .unwrap();
    crate::pyramid_progress::observe_challenge_complete(&frame)
        .unwrap()
        .unwrap()
}


/// Success is withheld when shared reward handling fails or STOP precedes that stage.
#[test]
fn challenge_success_retains_authority_until_shared_rewards_finish() {
    for stop in [false, true] {
        let mut io = Fake::new(Entry::ChallengeComplete(fours_complete()), []);
        if stop {
            io.stop_on_phase = Some((0, SessionPhase::Completion));
        } else {
            io.fail_on = Some(Call::FinishChallenge);
        }
        assert!(challenge::drive(&mut io).is_err());
        assert!(io.acquisitions().is_empty());
        assert!(!io.calls.contains(&Call::Restart));
        assert_eq!(io.calls.contains(&Call::FinishChallenge), !stop);
    }
}


/// Normal repetition cannot acquire a challenge merely because its denominator is three.
#[test]
fn normal_repetition_rejects_actual_challenge_before_acquisition() {
    let mut io = Fake::new(
        Entry::Challenge(requirement(ChallengeKind::Boards, 0, 3)),
        [],
    );
    assert!(drive(WorkflowMode::Continuous, &mut io).is_err());
    assert!(io.acquisitions().is_empty());
}


/// Score/rank values are observed, while board-clear counts advance exactly once.
#[test]
fn challenge_lifetime_accepts_its_own_monotonic_units_across_real_clear_results() {
    for (kind, current, next, target) in [
        (ChallengeKind::Score, 1000, 1300, 3000),
        (ChallengeKind::Cards, 4, 8, 12),
        (ChallengeKind::Boards, 0, 1, 2),
    ] {
        let mut io = Fake::new(
            Entry::Challenge(requirement(kind, current, target)),
            [BoardCompletion::BoardClear],
        );
        io.transitions
            .push_back(Entry::Challenge(requirement(kind, next, target)));
        io.stop_on_phase = Some((2, SessionPhase::Planning));
        assert!(challenge::drive(&mut io).unwrap_err().contains("STOP"));
        assert_eq!(io.acquisitions(), [1, 2]);
        assert!(!io.operations().contains(&Call::Restart));
    }
}


/// A lower next-board counter cannot erase progress seen during the consumed route.
#[test]
fn challenge_transition_preserves_execution_counter_high_water() {
    for kind in [ChallengeKind::Score, ChallengeKind::Cards] {
        let mut io = Fake::new(
            Entry::Challenge(requirement(kind, 4, 12)),
            [BoardCompletion::BoardClear],
        );
        io.observed_challenge = Some(requirement(kind, 8, 12));
        io.transitions
            .push_back(Entry::Challenge(requirement(kind, 6, 12)));
        assert!(challenge::drive(&mut io).unwrap_err().contains("regressed"));
        assert_eq!(io.acquisitions(), [1]);
    }
}


/// Changed kinds/targets, missing headers and skipped board counts never permit board two.
#[test]
fn challenge_transition_refuses_changed_unknown_and_skipped_requirements() {
    for next in [
        Entry::Challenge(requirement(ChallengeKind::Score, 4, 12)),
        Entry::Challenge(requirement(ChallengeKind::Cards, 4, 13)),
        unknown_board(),
        standard_board(1),
        standard_terminal(),
    ] {
        let mut io = Fake::new(
            Entry::Challenge(requirement(ChallengeKind::Cards, 4, 12)),
            [BoardCompletion::BoardClear],
        );
        io.transitions.push_back(next);
        assert!(challenge::drive(&mut io).is_err());
        assert_eq!(io.acquisitions(), [1]);
        assert!(!io.operations().contains(&Call::Restart));
    }
    let mut io = Fake::new(
        Entry::Challenge(requirement(ChallengeKind::Boards, 0, 3)),
        [BoardCompletion::BoardClear],
    );
    io.transitions
        .push_back(Entry::Challenge(requirement(ChallengeKind::Boards, 2, 3)));
    assert!(
        challenge::drive(&mut io)
            .unwrap_err()
            .contains("exactly once")
    );
    assert_eq!(io.acquisitions(), [1]);
}


/// Native success can stop at entry or after one executed prefix without a restart.
#[test]
fn verified_challenge_success_stops_without_new_game_or_another_board() {
    let proof = fours_complete();
    let mut already = Fake::new(Entry::ChallengeComplete(proof), []);
    assert!(
        challenge::drive(&mut already)
            .unwrap()
            .contains("target verified")
    );
    assert!(already.acquisitions().is_empty());
    let mut io = Fake::new(
        Entry::Challenge(requirement(ChallengeKind::Cards, 4, 12)),
        [BoardCompletion::ChallengeTargetVerified(proof)],
    );
    assert!(
        challenge::drive(&mut io)
            .unwrap()
            .contains("target verified")
    );
    assert_eq!(io.acquisitions(), [1]);
    assert!(
        !io.operations()
            .iter()
            .any(|call| matches!(call, Call::Transition | Call::Restart))
    );
}


/// Numeric completion alone and another objective kind's complete terminal are insufficient.
#[test]
fn challenge_counter_alone_and_wrong_kind_terminal_do_not_become_success() {
    let mut io = Fake::new(
        Entry::Challenge(requirement(ChallengeKind::Cards, 12, 12)),
        [],
    );
    assert!(challenge::drive(&mut io).is_err());
    assert!(io.acquisitions().is_empty());
    let mut wrong = Fake::new(
        Entry::Challenge(requirement(ChallengeKind::Score, 4, 12)),
        [BoardCompletion::ChallengeTargetVerified(fours_complete())],
    );
    assert!(challenge::drive(&mut wrong).is_err());
    assert_eq!(wrong.acquisitions(), [1]);
}


/// Partial endpoints and uncertain effects never acquire continuation authority.
#[test]
fn challenge_partial_and_uncertainty_stop_before_any_transition() {
    for completion in [
        BoardCompletion::PartialPositionVerified {
            removed: 20,
            remaining: 8,
        },
        BoardCompletion::DeliveredUnconfirmed,
    ] {
        let mut io = Fake::new(
            Entry::Challenge(requirement(ChallengeKind::Cards, 4, 12)),
            [completion],
        );
        let result = challenge::drive(&mut io);
        assert_eq!(
            result.is_ok(),
            matches!(completion, BoardCompletion::PartialPositionVerified { .. })
        );
        assert!(
            !io.operations()
                .iter()
                .any(|call| matches!(call, Call::Transition | Call::Restart))
        );
    }
}


/// STOP after a positive dependency result wins before success is accepted.
#[test]
fn challenge_stop_after_capture_or_execution_beats_positive_success() {
    let proof = fours_complete();
    for stop in [Call::Entry(WorkflowMode::Continuous), Call::Execute(1)] {
        let mut io = Fake::new(
            Entry::Challenge(requirement(ChallengeKind::Cards, 4, 12)),
            [BoardCompletion::ChallengeTargetVerified(proof)],
        );
        io.stop_after = Some((stop, 1));
        assert!(challenge::drive(&mut io).unwrap_err().contains("STOP"));
        assert!(
            !io.operations()
                .iter()
                .any(|call| matches!(call, Call::Transition | Call::Restart))
        );
    }
}
