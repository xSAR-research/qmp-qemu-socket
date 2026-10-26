//! Real native 121 intermediate-clear pixels on the production observation adapter.
//!
//! The saved deal is solved/replayed independently. Injected initial bindings,
//! repeated frames, frame-guard refusal and STOP are offline transport inputs;
//! this does not establish a guest ACK, fresh receipt or second-board Recon.

use super::*;
use crate::pyramid_progress::{ChallengeKind, ChallengeProgress};


/// Decode one unchanged native 121 frame from the retained permanent packet.
fn frame(name: &str) -> CapturedFrame {
    decode_png(
        &fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/pyramid-recon/native-v19")
                .join(name),
        )
        .unwrap(),
    )
    .unwrap()
}


/// Independently obtain the actual 51-action exact-match-cost route and replay it.
fn setup(plan: &mut Prepared) {
    let bytes = fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/pyramid-recon/native-v19/deal.json"),
    )
    .unwrap();
    let deal = xsar::pyramid_json::read_from(bytes.as_slice()).unwrap();
    let outcome = search::solve_best_effort_with_cost(
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
    } = outcome
    else {
        panic!("retained actual challenge deal must solve");
    };
    assert!(minimum_proven);
    assert_eq!(actions.len(), 51);
    plan.trace = replay::replay(&deal, &actions, true).unwrap();
    plan.acquisition.bytes = bytes;
    plan.acquisition.deal = deal;
    plan.acquisition.vision = ReconVision::bundled_native_v19().unwrap();
}


/// Final-only exact clear retains immutable binding, fatal guard and pre-capture STOP.
#[test]
fn actual_board_count_final_clear_adapter_retains_phase_piles_initial_binding_guard_and_stop() {
    let first = frame("first-unmatched.png");
    let last = frame("last-observed.png");
    let calls = Cell::new(0);
    let guard = |frame: &CapturedFrame, _phase| {
        let current = crate::pyramid_progress::observe_challenge(frame)?;
        assert_eq!(current, ChallengeProgress::new(ChallengeKind::Boards, 1, 2));
        calls.set(calls.get() + 1);
        if calls.get() == 3 {
            Err("injected caller frame guard refusal".to_owned())
        } else {
            Ok(FrameGuardOutcome::Ready)
        }
    };
    let evidence = adapter_with_setup(
        vec![last.clone(), first, last.clone(), last],
        Some(&guard),
        setup,
        |io| {
            let state = goal(io.plan);
            let (tableau, left, right) = expected(&io.plan.acquisition.deal, state).unwrap();
            assert_eq!(tableau, [None; 28]);
            assert_eq!(
                left,
                Some(xsar::games::cards::Card::new(
                    xsar::games::cards::Rank::King,
                    xsar::games::cards::Suit::Spades
                ))
            );
            assert_eq!(
                right,
                Some(xsar::games::cards::Card::new(
                    xsar::games::cards::Rank::Nine,
                    xsar::games::cards::Suit::Spades
                ))
            );
            let initial = ChallengeProgress::new(ChallengeKind::Boards, 0, 2).unwrap();
            io.challenge = Some(initial);
            io.challenge_initial = Some(initial);
            assert!(
                !io.observe(state, false).unwrap(),
                "ordinary gesture phase remains strict"
            );
            assert_eq!(
                io.observe_final(state).unwrap(),
                Some(BoardCompletion::BoardClear)
            );
            assert_eq!(
                io.challenge,
                ChallengeProgress::new(ChallengeKind::Boards, 1, 2)
            );
            assert_eq!(io.challenge_initial, Some(initial));
            assert_eq!(
                io.observe_final(state).unwrap(),
                Some(BoardCompletion::BoardClear)
            );
            assert_eq!(
                io.challenge_initial,
                Some(initial),
                "initial binding cannot follow progress"
            );
            assert!(
                io.observe_final(state)
                    .unwrap_err()
                    .contains("frame guard rejected")
            );
            let before = io.captures;
            io.cancel.store(true, std::sync::atomic::Ordering::SeqCst);
            let stopped = io.observe_final(state);
            io.cancel.store(false, std::sync::atomic::Ordering::SeqCst);
            assert!(
                stopped.is_err(),
                "STOP wins before any extra capture or proof"
            );
            assert_eq!(io.captures, before);
        },
    );
    assert_eq!(calls.get(), 3);
    assert_eq!(evidence.captures, 4);
    assert_eq!(
        (evidence.counts.attempted, evidence.counts.acknowledged),
        (0, 0)
    );
    assert!(evidence.wire.events.is_empty());
    assert_eq!(evidence.wire.unused_frames, 0);
}


/// Compose only the literal piles from the independently replayed native121 deal.
///
/// Perfect/header pixels remain the actual151 third-clear incident. The piles and
/// replay belong to a different retained51-action route, not the missing67-action
/// guest trace. This synthetic transport derivative is labelled deliberately.
fn normal_152_replayed_piles() -> CapturedFrame {
    let mut final_frame = decode_png(include_bytes!(
        "../../../../tests/fixtures/pyramid-normal-final-152/last-observed.png"
    ))
    .unwrap();
    let piles = frame("last-observed.png");
    for bounds in [
        crate::pyramid::LEFT_TARGET.bounds,
        crate::pyramid::RIGHT_TARGET.bounds,
    ] {
        final_frame
            .as_view()
            .unwrap()
            .checked_bounds(bounds)
            .unwrap();
        piles.as_view().unwrap().checked_bounds(bounds).unwrap();
        for y in bounds.y..bounds.y + bounds.height {
            let destination = y as usize * final_frame.stride + bounds.x as usize * 4;
            let source = y as usize * piles.stride + bounds.x as usize * 4;
            let bytes = bounds.width as usize * 4;
            final_frame.pixels[destination..destination + bytes]
                .copy_from_slice(&piles.pixels[source..source + bytes]);
        }
    }
    final_frame
}


/// Retain an independently solved/replayed deal while selecting the actual current reader.
fn normal_152_setup(plan: &mut Prepared) {
    setup(plan);
    plan.acquisition.challenge_progress = None;
    plan.acquisition.vision = ReconVision::bundled_native_v23().unwrap();
}


/// New geometry is final-only; actual normal guards and STOP still own completion authority.
#[test]
fn native_152_normal_final_adapter_keeps_phase_guard_stop_and_third_receipt() {
    let frame = normal_152_replayed_piles();
    let progress = BoardCount {
        completed: 2,
        target: 3,
    };
    let calls = Cell::new(0);
    let guard = |frame: &CapturedFrame, phase| {
        calls.set(calls.get() + 1);
        crate::worker::pyramid_session::guard_normal_progress(frame, phase, progress)
    };
    let evidence = adapter_with_setup(
        vec![frame.clone(), frame],
        Some(&guard),
        normal_152_setup,
        |io| {
            io.policy = ExecutionPolicy::NormalSeries(progress);
            assert!(
                !io.handoff_after_final_ack(),
                "third ACK cannot request a read-only next-board handoff"
            );
            let state = goal(io.plan);
            assert!(
                !io.observe(state, false).unwrap(),
                "ordinary observation never receives new final geometry"
            );
            let completion = io.observe_final(state).unwrap().unwrap();
            assert_eq!(completion, BoardCompletion::BoardClear);
            crate::worker::post_game::PyramidRestartEntry::verified_third_completion(
                progress, completion,
            )
            .unwrap();
            assert!(
                crate::worker::post_game::PyramidRestartEntry::verified_third_completion(
                    progress,
                    BoardCompletion::DeliveredUnconfirmed
                )
                .is_err()
            );
            let captures = io.captures;
            io.cancel.store(true, std::sync::atomic::Ordering::SeqCst);
            assert!(io.observe_final(state).is_err());
            assert_eq!(io.captures, captures);
        },
    );
    assert_eq!(calls.get(), 1);
    assert_eq!(evidence.captures, 2);
    assert_eq!(
        (evidence.counts.attempted, evidence.counts.acknowledged),
        (0, 0)
    );
    assert!(evidence.wire.events.is_empty());
    assert_eq!(evidence.wire.unused_frames, 0);
}


/// A positive final-position proof never overrides an actual different normal ordinal.
#[test]
fn native_152_normal_final_adapter_rejects_positive_header_contradiction() {
    let progress = BoardCount {
        completed: 1,
        target: 3,
    };
    let guard = |frame: &CapturedFrame, phase| {
        crate::worker::pyramid_session::guard_normal_progress(frame, phase, progress)
    };
    let evidence = adapter_with_setup(
        vec![normal_152_replayed_piles()],
        Some(&guard),
        normal_152_setup,
        |io| {
            assert!(
                io.observe_final(goal(io.plan))
                    .unwrap_err()
                    .contains("frame guard rejected")
            );
        },
    );
    assert_eq!(evidence.captures, 1);
    assert_eq!(
        (evidence.counts.attempted, evidence.counts.acknowledged),
        (0, 0)
    );
    assert!(evidence.wire.events.is_empty());
    assert_eq!(evidence.wire.unused_frames, 0);
}
