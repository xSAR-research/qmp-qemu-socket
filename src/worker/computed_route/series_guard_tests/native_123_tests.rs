//! Actual adapter cases with original native frames; all bindings/captures are offline inputs.

use super::*;
use crate::pyramid_progress::{ChallengeKind, ChallengeProgress};


/// Decode one exact exposed original without capturing the guest.
fn frame(name: &str) -> CapturedFrame {
    decode_png(
        &fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/pyramid-recon/native-v21")
                .join(name),
        )
        .unwrap(),
    )
    .unwrap()
}


/// Independently solve and replay the captured complete deal in the recorded exact cost.
fn setup_case(plan: &mut Prepared, case: &str, count: usize) {
    let bytes = fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/pyramid-recon/native-v21")
            .join(format!("{case}-deal.json")),
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
        panic!("exposed native deal must independently solve");
    };
    assert!(minimum_proven);
    assert_eq!(actions.len(), count);
    plan.trace = replay::replay(&deal, &actions, true).unwrap();
    plan.acquisition.bytes = bytes;
    plan.acquisition.deal = deal;
    plan.acquisition.vision = ReconVision::bundled_native_v21().unwrap();
}


/// A reached Boards count retires input, while an unmet Score clear permits transition only.
#[test]
fn native_123_adapter_proves_board_target_and_unmet_score_clear_separately() {
    for (case, count, initial, reached) in [
        (
            "boards",
            57,
            ChallengeProgress::new(ChallengeKind::Boards, 2, 3).unwrap(),
            true,
        ),
        (
            "score",
            60,
            ChallengeProgress::new(ChallengeKind::Score, 0, 1400).unwrap(),
            false,
        ),
    ] {
        let evidence = adapter_with_setup(
            vec![frame(&format!("{case}-last-observed.png"))],
            None,
            |plan| setup_case(plan, case, count),
            |io| {
                io.challenge = Some(initial);
                io.challenge_initial = Some(initial);
                let state = goal(io.plan);
                let completion = io.observe_final(state).unwrap().unwrap();
                if reached {
                    assert!(matches!(
                        completion,
                        BoardCompletion::ChallengeTargetReached(_)
                    ));
                } else {
                    assert!(matches!(completion, BoardCompletion::BoardClear));
                    assert_eq!(
                        io.challenge,
                        ChallengeProgress::new(ChallengeKind::Score, 1260, 1400)
                    );
                }
            },
        );
        assert_eq!(
            (evidence.counts.attempted, evidence.counts.acknowledged),
            (0, 0)
        );
        assert!(evidence.wire.events.is_empty());
        assert_eq!(evidence.wire.unused_frames, 0);
    }
}


/// Two genuine Fours terminals retire a still-unfinished route and leave Continue untouched.
#[test]
fn native_123_adapter_confirms_fours_before_remaining_route_actions() {
    let last = frame("fours-last-observed.png");
    let evidence = adapter_with_setup(
        vec![last.clone(), last],
        None,
        |plan| setup_case(plan, "fours", 64),
        |io| {
            let initial = ChallengeProgress::new(ChallengeKind::Cards, 9, 12).unwrap();
            io.challenge = Some(initial);
            io.challenge_initial = Some(initial);
            let state = io
                .plan
                .trace
                .iter()
                .take(47)
                .fold(SearchState::initial(), |state, step| {
                    rules::apply(&io.plan.acquisition.deal, state, step.action).unwrap()
                });
            assert!(!state.is_goal());
            assert!(matches!(
                io.observe_effect(state, false).unwrap(),
                EffectObservation::Position(false)
            ));
            let EffectObservation::Challenge(proof) = io.observe_effect(state, false).unwrap()
            else {
                panic!("two genuine completion frames must retire the unfinished route");
            };
            assert!(proof.operator_continue());
            assert_eq!(
                proof.progress(),
                ChallengeProgress::new(ChallengeKind::Cards, 12, 12).unwrap()
            );
        },
    );
    assert_eq!(
        (evidence.counts.attempted, evidence.counts.acknowledged),
        (0, 0)
    );
    assert!(evidence.wire.events.is_empty());
}

/// The new clear alternative still passes through fatal frame admission and pre-capture STOP.
#[test]
fn native_123_adapter_keeps_guard_and_stop_before_continuation() {
    let guard = |_frame: &CapturedFrame| Err("injected native-123 guard refusal".to_owned());
    let evidence = adapter_with_setup(
        vec![frame("score-last-observed.png")],
        Some(&guard),
        |plan| setup_case(plan, "score", 60),
        |io| {
            let initial = ChallengeProgress::new(ChallengeKind::Score, 0, 1400).unwrap();
            io.challenge = Some(initial);
            io.challenge_initial = Some(initial);
            let state = goal(io.plan);
            assert!(
                io.observe_final(state)
                    .unwrap_err()
                    .contains("frame guard rejected")
            );
            let captures = io.captures;
            io.cancel.store(true, std::sync::atomic::Ordering::SeqCst);
            assert!(io.observe_final(state).is_err());
            assert_eq!(io.captures, captures);
        },
    );
    assert_eq!(
        (evidence.counts.attempted, evidence.counts.acknowledged),
        (0, 0)
    );
    assert!(evidence.wire.events.is_empty());
}
