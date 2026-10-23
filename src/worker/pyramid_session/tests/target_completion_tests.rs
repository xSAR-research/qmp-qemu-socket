//! Synthetic lifetime checks for all currently covered requirement kinds.
//!
//! The erased header is an explicit test composition, never native counter evidence.

use super::*;
use crate::pyramid_progress::ChallengeKind;


/// An unmet target of each covered kind requires a fresh complete three-phase board cycle.
#[test]
fn target_completion_all_five_unmet_requirements_reacquire_after_verified_transition() {
    for kind in [
        ChallengeKind::Boards,
        ChallengeKind::Score,
        ChallengeKind::Cards,
    ] {
        let initial = ChallengeProgress::new(kind, 0, 2).unwrap();
        let next = ChallengeProgress::new(kind, 1, 2).unwrap();
        let target = ChallengeProgress::new(kind, 2, 2).unwrap();
        let mut io = Fake::new(
            Entry::Challenge(initial),
            [
                BoardCompletion::BoardClear,
                BoardCompletion::ChallengeTargetVerified(bound_terminal(target)),
            ],
        );
        io.transitions.push_back(Entry::Challenge(next));
        assert!(
            challenge::drive(&mut io)
                .unwrap()
                .contains("target verified")
        );
        assert_eq!(io.acquisitions(), [1, 2]);
        assert_eq!(
            io.operations(),
            [
                Call::Entry(WorkflowMode::Continuous),
                Call::Acquire(1),
                Call::Plan(1),
                Call::Execute(1),
                Call::Transition,
                Call::Acquire(2),
                Call::Plan(2),
                Call::Execute(2),
                Call::FinishChallenge,
            ]
        );
    }
}


/// Supply actual common terminal structure with its numeric header deliberately erased.
fn bound_terminal(progress: ChallengeProgress) -> ChallengeCompletion {
    let mut frame = crate::capture::decode_png(include_bytes!(
        "../../../../tests/fixtures/pyramid-objectives/fours-complete.png"
    ))
    .unwrap();
    for y in 48..80 {
        for x in 814..1110 {
            let offset = y * frame.stride + x * 4;
            frame.pixels[offset..offset + 3].copy_from_slice(&[3, 5, 7]);
        }
    }
    let proof = crate::pyramid_progress::observe_bound_challenge_complete(&frame, progress)
        .unwrap()
        .unwrap();
    assert_eq!(proof.progress(), progress);
    proof
}


/// Every supported kind retires input at the verified count and waits for a terminal before success.
#[test]
fn target_completion_all_five_kinds_handover_without_reacquisition() {
    for kind in [
        ChallengeKind::Boards,
        ChallengeKind::Score,
        ChallengeKind::Cards,
    ] {
        let initial = ChallengeProgress::new(kind, 1, 2).unwrap();
        let target = ChallengeProgress::new(kind, 2, 2).unwrap();
        let mut io = Fake::new(
            Entry::Challenge(initial),
            [BoardCompletion::ChallengeTargetReached(target)],
        );
        io.observed_challenge = Some(target);
        io.transitions
            .push_back(Entry::ChallengeComplete(bound_terminal(target)));
        assert!(
            challenge::drive(&mut io)
                .unwrap()
                .contains("target verified")
        );
        assert_eq!(io.acquisitions(), [1]);
        assert_eq!(
            io.operations()
                .iter()
                .filter(|call| **call == Call::Transition)
                .count(),
            1
        );
        assert!(io.operations().contains(&Call::FinishChallenge));
        assert!(!io.operations().contains(&Call::Restart));
    }
}


/// Missing or wrong terminals, mismatched counters and STOP cannot turn a pending count into success.
#[test]
fn target_completion_pending_count_refuses_unknown_changed_or_stopped_completion() {
    let initial = ChallengeProgress::new(ChallengeKind::Boards, 1, 2).unwrap();
    let target = ChallengeProgress::new(ChallengeKind::Boards, 2, 2).unwrap();
    for wrong in [
        unknown_board(),
        Entry::Challenge(initial),
        Entry::ChallengeComplete(bound_terminal(
            ChallengeProgress::new(ChallengeKind::Score, 2, 2).unwrap(),
        )),
    ] {
        let mut io = Fake::new(
            Entry::Challenge(initial),
            [BoardCompletion::ChallengeTargetReached(target)],
        );
        io.observed_challenge = Some(target);
        io.transitions.push_back(wrong);
        assert!(challenge::drive(&mut io).is_err());
        assert_eq!(io.acquisitions(), [1]);
        assert!(!io.operations().contains(&Call::FinishChallenge));
    }
    for call in [Call::Transition, Call::FinishChallenge] {
        let mut io = Fake::new(
            Entry::Challenge(initial),
            [BoardCompletion::ChallengeTargetReached(target)],
        );
        io.observed_challenge = Some(target);
        io.transitions
            .push_back(Entry::ChallengeComplete(bound_terminal(target)));
        io.stop_after = Some((call, 1));
        assert!(challenge::drive(&mut io).unwrap_err().contains("STOP"));
        assert_eq!(io.acquisitions(), [1]);
        assert!(!io.operations().contains(&Call::Restart));
    }
}
