//! Whole-action dispatch and prefix accounting; synthetic timing is not live evidence.

/// Numeric targets retire only after the complete exact effect, including both final-route paths.
#[test]
fn target_completion_verified_pending_retires_prefix_and_final_clear_or_partial() {
    for plan in [accepted_plan(), partial_plan()] {
        let matched = plan
            .trace
            .iter()
            .position(|step| matches!(step.action, RecordedAction::Match(..)))
            .unwrap()
            + 1;
        for end in [matched, plan.trace.len()] {
            let progress = crate::pyramid_progress::ChallengeProgress::new(
                crate::pyramid_progress::ChallengeKind::Score,
                2,
                2,
            )
            .unwrap();
            let gestures: usize = plan.trace[..end]
                .iter()
                .map(|step| oracle_operations(step.action).len())
                .sum();
            let mut io = FakeIo::new(&plan);
            io.pending_after = Some((end, progress));
            let report = drive(&plan, &mut io).unwrap();
            assert_eq!(
                report.completion,
                BoardCompletion::ChallengeTargetReached(progress)
            );
            assert_eq!(io.delivered.len(), gestures);
            assert_eq!(io.committed.len(), end);
            assert!(
                io.challenge_observations
                    .iter()
                    .all(|(_, partial)| *partial == 0)
            );
            assert!(report.message.contains("completion still requires"));
        }
    }
}


/// A reached-counter injection cannot replace an unobserved card effect or beat STOP.
#[test]
fn target_completion_pending_retains_exact_effect_and_post_capture_stop() {
    let plan = accepted_plan();
    let end = plan
        .trace
        .iter()
        .position(|step| matches!(step.action, RecordedAction::Match(..)))
        .unwrap()
        + 1;
    let gestures: usize = plan.trace[..end]
        .iter()
        .map(|step| oracle_operations(step.action).len())
        .sum();
    for missing_effect in [false, true] {
        let mut io = FakeIo::new(&plan);
        io.pending_after = Some((
            end,
            crate::pyramid_progress::ChallengeProgress::new(
                crate::pyramid_progress::ChallengeKind::Score,
                2,
                2,
            )
            .unwrap(),
        ));
        io.freeze_effects = missing_effect;
        io.stop_during_challenge = !missing_effect;
        assert!(drive(&plan, &mut io).is_err());
        assert!(io.delivered.len() <= gestures);
        assert!(io.committed.len() < end);
    }
}


use super::*;


/// Decode an actual finite native terminal rather than constructing a success flag.
pub(super) fn proof() -> ChallengeCompletion {
    let frame = crate::capture::decode_png(include_bytes!(
        "../../../../tests/fixtures/pyramid-objectives/fours-complete.png"
    ))
    .unwrap();
    crate::pyramid_progress::observe_challenge_complete(&frame)
        .unwrap()
        .unwrap()
}


/// Terminal interception retires a legal route suffix after both MATCH acknowledgements.
#[test]
fn challenge_mid_route_retires_suffix_without_claiming_hidden_tableau_effect() {
    let plan = accepted_plan();
    let end = plan
        .trace
        .iter()
        .position(|step| matches!(step.action, RecordedAction::Match(..)))
        .unwrap()
        + 1;
    let gestures: usize = plan.trace[..end]
        .iter()
        .map(|step| oracle_operations(step.action).len())
        .sum();
    let mut io = FakeIo::new(&plan);
    io.challenge_after = Some(end);
    let report = drive(&plan, &mut io).unwrap();
    assert_eq!(
        report.completion,
        BoardCompletion::ChallengeTargetVerified(proof())
    );
    assert_eq!(io.counts.acknowledged, gestures);
    assert_eq!(io.delivered.len(), gestures);
    assert_eq!(io.committed.len(), end - 1);
    assert!(
        io.challenge_observations
            .iter()
            .all(|(_, partial)| *partial == 0)
    );
    assert!(report.message.contains(&format!(
        "{end}/{} route actions delivered",
        plan.trace.len()
    )));
}


/// STOP between MATCH clicks or during terminal capture prevents success and every next input.
#[test]
fn challenge_does_not_bypass_selection_or_post_capture_stop() {
    let plan = accepted_plan();
    let end = plan
        .trace
        .iter()
        .position(|step| matches!(step.action, RecordedAction::Match(..)))
        .unwrap()
        + 1;
    let gestures: usize = plan.trace[..end]
        .iter()
        .map(|step| oracle_operations(step.action).len())
        .sum();
    for during_capture in [false, true] {
        let mut io = FakeIo::new(&plan);
        io.challenge_after = Some(end);
        if during_capture {
            io.stop_during_challenge = true;
        } else {
            io.stop_after_ack = Some(gestures - 1);
        }
        let error = drive(&plan, &mut io).unwrap_err();
        assert!(
            error.contains(if during_capture {
                "STOP"
            } else {
                "execution stopped: Aborted"
            }),
            "during_capture={during_capture}: {error}"
        );
        assert_eq!(
            io.delivered.len(),
            if during_capture {
                gestures
            } else {
                gestures - 1
            }
        );
        assert!(
            io.challenge_observations
                .iter()
                .all(|(_, partial)| *partial == 0)
        );
        assert_eq!(io.committed.len(), end - 1);
    }
}


/// Both final-clear and final-partial dispatch retain the separate challenge outcome.
#[test]
fn challenge_final_fastpaths_do_not_disguise_terminal_as_clear_or_partial() {
    for plan in [accepted_plan(), partial_plan()] {
        let mut io = FakeIo::new(&plan);
        io.challenge_after = Some(plan.trace.len());
        let report = drive(&plan, &mut io).unwrap();
        assert_eq!(
            report.completion,
            BoardCompletion::ChallengeTargetVerified(proof())
        );
        assert_eq!(io.action, plan.trace.len());
        assert_eq!(io.committed.len(), plan.trace.len() - 1);
    }
}
