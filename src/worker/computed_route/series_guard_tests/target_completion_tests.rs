//! Actual NativeIo with the retained 39-action deal and unchanged native failure pixels.
//!
//! The binding and repeated captures are isolated transport inputs, not live ACKs or a receipt.

use super::*;
use crate::pyramid_progress::{ChallengeKind, ChallengeProgress};
use std::sync::atomic::Ordering;


/// Decode the exact exposed final-board frame without modifying its pixels.
fn frame(name: &str) -> CapturedFrame {
    decode_png(
        &fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/pyramid-recon/native-v20")
                .join(name),
        )
        .unwrap(),
    )
    .unwrap()
}


/// Independently reproduce the native exact-match plan and its complete replay.
fn setup(plan: &mut Prepared) {
    let bytes = fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/pyramid-recon/native-v20/deal.json"),
    )
    .unwrap();
    let deal = xsar::pyramid_json::read_from(bytes.as_slice()).unwrap();
    let result = search::solve_best_effort_with_cost(
        &deal,
        pyramid_search_policy::limits(),
        &AtomicBool::new(false),
        SearchOptions {
            method: SearchMethod::AStar,
            heuristic_weight: std::num::NonZeroU32::new(1).unwrap(),
        },
        search::PyramidCost::CardMatches,
        None,
    );
    let xsar::search::BestEffortOutcome::Solved {
        actions,
        minimum_proven,
        ..
    } = result
    else {
        panic!("retained native final-board deal must solve");
    };
    assert!(minimum_proven);
    assert_eq!(actions.len(), 39);
    plan.trace = replay::replay(&deal, &actions, true).unwrap();
    plan.acquisition.bytes = bytes;
    plan.acquisition.deal = deal;
    plan.acquisition.vision = ReconVision::bundled_native_v20().unwrap();
}


/// Numeric target evidence retires input; one positive actual terminal releases the operator endpoint.
#[test]
fn target_completion_native_adapter_separates_pending_effect_and_operator_endpoint() {
    let terminal = decode_png(include_bytes!(
        "../../../../tests/fixtures/pyramid-objectives/boards-complete-continue.png"
    ))
    .unwrap();
    let evidence = adapter_with_setup(
        vec![frame("last-observed.png"), terminal],
        None,
        setup,
        |io| {
            let initial = ChallengeProgress::new(ChallengeKind::Boards, 1, 2).unwrap();
            let target = ChallengeProgress::new(ChallengeKind::Boards, 2, 2).unwrap();
            io.challenge = Some(initial);
            io.challenge_initial = Some(initial);
            let state = goal(io.plan);
            assert_eq!(
                io.observe_final(state).unwrap(),
                Some(BoardCompletion::ChallengeTargetReached(target))
            );
            assert_eq!(io.challenge_initial, Some(initial));
            assert_eq!(io.challenge, Some(target));
            let Some(BoardCompletion::ChallengeTargetVerified(proof)) =
                io.observe_final(state).unwrap()
            else {
                panic!("one actual terminal observation required");
            };
            assert_eq!(proof.progress(), target);
            assert!(proof.operator_continue());
        },
    );
    assert_eq!(
        (evidence.counts.attempted, evidence.counts.acknowledged),
        (0, 0)
    );
    assert!(evidence.wire.events.is_empty());
    assert_eq!(evidence.wire.unused_frames, 0);
}


/// Ordinary guards, flying frames, fatal caller refusal and pre-capture STOP remain effective.
#[test]
fn target_completion_native_adapter_retains_guard_stop_and_phase_refusal() {
    let calls = Cell::new(0);
    let guard = |_frame: &CapturedFrame, _phase| {
        calls.set(calls.get() + 1);
        if calls.get() == 2 {
            Err("injected target frame guard refusal".to_owned())
        } else {
            Ok(FrameGuardOutcome::Ready)
        }
    };
    let last = frame("last-observed.png");
    let evidence = adapter_with_setup(
        vec![
            last.clone(),
            frame("first-unmatched.png"),
            last.clone(),
            last,
        ],
        Some(&guard),
        setup,
        |io| {
            let initial = ChallengeProgress::new(ChallengeKind::Boards, 1, 2).unwrap();
            io.challenge = Some(initial);
            io.challenge_initial = Some(initial);
            let state = goal(io.plan);
            assert!(!io.observe(state, false).unwrap());
            assert_eq!(io.observe_final(state).unwrap(), None);
            assert!(matches!(
                io.observe_final(state).unwrap(),
                Some(BoardCompletion::ChallengeTargetReached(_))
            ));
            assert!(
                io.observe_final(state)
                    .unwrap_err()
                    .contains("frame guard rejected")
            );
            let captures = io.captures;
            io.cancel.store(true, Ordering::SeqCst);
            assert!(io.observe_final(state).is_err());
            assert_eq!(io.captures, captures);
            assert!(io.cancel.load(Ordering::SeqCst));
        },
    );
    assert_eq!(
        (evidence.counts.attempted, evidence.counts.acknowledged),
        (0, 0)
    );
    assert!(evidence.wire.events.is_empty());
}
