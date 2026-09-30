//! Control-flow regressions run against the production Klondike controller.
//!
//! The fake adapter supplies observations and effect verdicts so failures and
//! cancellation can be injected deterministically. Pixel classification/effect
//! correctness is covered by the detector's original-image and synthetic tests.

use std::{collections::VecDeque, sync::atomic::AtomicBool};

use super::*;
use crate::{capture::PixelFormat, klondike::KlondikeTarget, parameters::AnimationSettleDelays};


/// Deterministic adapter recording order while independently injecting I/O failures.
struct FakeIo<'a> {
    /// Fresh captures supplied in order; exhaustion indicates an unexpected recapture.
    frames: VecDeque<FrameObservation>,
    /// Ordered I/O trace, used to verify no delay between Solver and the next capture.
    trace: Vec<&'static str>,
    /// Controlled independent effect verdict used by controller tests.
    proven_effect: bool,
    /// Number of effect evaluations; Solve must bypass effect inference completely.
    effect_checks: usize,
    /// Simulate a decoded frame whose analysis failed, retaining its pixels.
    capture_failure: Option<&'static str>,
    /// Make a gameplay delivery uncertain after recording its single attempt.
    fail_input: bool,
    /// Make Solver delivery uncertain after recording its single attempt.
    fail_solver: bool,
    /// Set STOP at the fresh probe boundary to test the final pre-input guard.
    stop_on_probe: Option<&'a AtomicBool>,
    /// Permit this many probes before setting STOP at the next probe boundary.
    probes_before_stop: usize,
    /// Set STOP during the settle wait to test interruption after one gameplay input.
    stop_on_wait: Option<&'a AtomicBool>,
    /// Set STOP after the diagnostic pixels are acquired but before the controller receives them.
    stop_on_diagnostic_capture: Option<&'a AtomicBool>,
}


impl KlondikeIo for FakeIo<'_> {


    fn capture(&mut self) -> Result<CaptureResult, String> {
        self.trace.push("capture");
        let observation = self.frames.pop_front().ok_or_else(|| "unexpected capture".to_owned())?;


        Ok(match self.capture_failure {
            Some(reason) => CaptureResult::Diagnostic { frame: observation.frame, reason: reason.to_owned() },
            None => CaptureResult::Classified(observation),
        })
    }


    fn capture_diagnostic(&mut self) -> Result<CapturedFrame, String> {
        self.trace.push("capture");
        let frame = self.frames.pop_front().ok_or_else(|| "unexpected capture".to_owned())?.frame;


        if let Some(stop) = self.stop_on_diagnostic_capture {
            stop.store(true, Ordering::Release);
        }
        Ok(frame)
    }


    fn probe(&mut self) -> Result<(), String> {
        self.trace.push("probe");


        if let Some(stop) = self.stop_on_probe
            && self.trace.iter().filter(|entry| **entry == "probe").count() > self.probes_before_stop
        {
            stop.store(true, Ordering::Release);
        }
        Ok(())
    }


    fn input(&mut self, _: StepPlan) -> Result<(), String> {
        self.trace.push("input");


        if self.fail_input {
            Err("simulated uncertain input".to_owned())
        } else {
            Ok(())
        }
    }


    fn solver(&mut self) -> Result<(), String> {
        self.trace.push("solver");


        if self.fail_solver {
            Err("simulated uncertain Solver".to_owned())
        } else {
            Ok(())
        }
    }


    fn wait(&mut self, _: Duration) -> Result<(), String> {
        self.trace.push("wait");


        if let Some(stop) = self.stop_on_wait {
            stop.store(true, Ordering::Release);
            return Err("STOP during wait".to_owned());
        }
        Ok(())
    }


    fn effect(&mut self, _: &CapturedFrame, _: &CapturedFrame, _: GuidedAction) -> Result<bool, String> {
        self.effect_checks += 1;
        Ok(self.proven_effect)
    }
}


/// Use a canonical evidenced draw target; rank classification is irrelevant to controller tests.
fn draw() -> PredictedAction {
    PredictedAction::Action(klondike::canonical_action(KlondikeTarget::Draw).expect("canonical draw"))
}


/// Build a controller observation with explicit scene/target evidence and valid pixel bounds.
fn observation(prediction: PredictedAction, gameplay_scene: bool) -> FrameObservation {
    let width = NOMINAL_FRAME_WIDTH;
    let height = NOMINAL_FRAME_HEIGHT;
    let stride = width as usize * 4;
    FrameObservation {
        frame: CapturedFrame {
            width, height, stride, format: PixelFormat::Rgba8,
            pixels: vec![0; stride * height as usize],
        },
        prediction,
        observed_rows: None,
        gameplay_scene,
        game_progress: None,
    }
}


/// Construct a fake from scene/target pairs without pre-authorising any effects.
fn fake<'a>(states: &[(PredictedAction, bool)]) -> FakeIo<'a> {
    FakeIo {
        frames: states.iter().map(|(prediction, scene)| observation(*prediction, *scene)).collect(),
        trace: Vec::new(),
        proven_effect: false,
        effect_checks: 0,
        capture_failure: None,
        fail_input: false,
        fail_solver: false,
        stop_on_probe: None,
        probes_before_stop: 0,
        stop_on_wait: None,
        stop_on_diagnostic_capture: None,
    }
}


/// Run with the production event sink and return the whether any guest input was attempted.
fn exercise(
    io: &mut impl KlondikeIo,
    approved: PredictedAction,
    limit: usize,
    stop: &AtomicBool,
) -> (Result<RunOutcome, String>, bool, Option<FrameObservation>) {
    let (events, _receiver) = std::sync::mpsc::channel();
    let sink = WorkerEventSink {
        events,
        latest_frame: super::super::LatestFrameSlot::default(),
        capture_context: std::sync::Mutex::new(None),
    };
    let mut latest = None;
    let mut input_attempted = false;
    let result = drive_run(
        io, approved, StepRunSettings::new(AnimationSettleDelays::default(), limit),
        &sink, stop, &mut latest, &mut input_attempted,
    );
    (result, input_attempted, latest)
}


/// Excessive finite limits fail before all capture and input operations.
#[test]
fn limits_fail_before_io() {


    for limit in [KLONDIKE_MAX_MULTI_STEP_ACTIONS + 1, usize::MAX] {
        let mut io = fake(&[]);
        let (result, _, latest) = exercise(&mut io, draw(), limit, &AtomicBool::new(false));
        assert!(result.is_err());
        assert!(io.trace.is_empty());
        assert!(latest.is_none());
    }
}


/// Gold absence on an unknown scene cannot authorise the user's Solver recovery rule.
#[test]
fn unknown_initial_scene_sends_no_input() {
    let mut io = fake(&[(PredictedAction::NoHighlight, false)]);
    let (result, _, latest) = exercise(&mut io, PredictedAction::NoHighlight, 1, &AtomicBool::new(false));
    assert!(result.unwrap_err().contains("unsupported Klondike scene"));
    assert_eq!(io.trace, ["capture"]);
    assert!(latest.is_some());
}


/// Solver refresh must be immediately followed by capture, and initial recovery never plays a move.
#[test]
fn initial_solver_refresh_captures_immediately_and_requires_a_new_run() {
    let mut io = fake(&[(PredictedAction::NoHighlight, true), (draw(), true)]);
    let (result, _, _) = exercise(&mut io, PredictedAction::NoHighlight, 1, &AtomicBool::new(false));
    assert_eq!(result, Ok(RunOutcome::RecoveryOnly));
    assert_eq!(io.trace, ["capture", "probe", "solver", "capture"]);
}


/// Persistent missing HALOs use one Solver input and a finite number of read-only observations.
#[test]
fn missing_halo_recovery_is_bounded_and_never_replays_solver() {
    let states = vec![(PredictedAction::NoHighlight, true); REOBSERVATION_LIMIT + 2];
    let mut io = fake(&states);
    let (result, input_attempted, latest) = exercise(&mut io, PredictedAction::NoHighlight, 1, &AtomicBool::new(false));
    assert!(result.unwrap_err().contains("bounded input-free observations"));
    assert_eq!(io.trace.iter().filter(|entry| **entry == "solver").count(), 1);
    assert_eq!(io.trace.iter().filter(|entry| **entry == "input").count(), 0);
    assert_eq!(io.trace.iter().filter(|entry| **entry == "capture").count(), REOBSERVATION_LIMIT + 2);
    assert!(input_attempted);
    assert!(latest.is_some());
}


/// A changed advisory preview cannot authorise a newly appeared target on the first capture.
#[test]
fn initial_changed_prediction_fails_closed() {
    let mut io = fake(&[(draw(), true)]);
    let (result, _, _) = exercise(&mut io, PredictedAction::NoHighlight, 1, &AtomicBool::new(false));
    assert!(result.unwrap_err().contains("differs from approved preview"));
    assert_eq!(io.trace, ["capture"]);
}


/// A repeated source with no independently proven effect must stop without replaying gameplay.
#[test]
fn repeated_halo_without_effect_does_not_authorise_replay() {
    let states = vec![(draw(), true); REOBSERVATION_LIMIT + 2];
    let mut io = fake(&states);
    let (result, input_attempted, _) = exercise(&mut io, draw(), 2, &AtomicBool::new(false));
    assert!(result.unwrap_err().contains("previous effect verified=false"));
    assert_eq!(io.trace.iter().filter(|entry| **entry == "input").count(), 1);
    assert_eq!(io.trace.iter().filter(|entry| **entry == "solver").count(), 0);
    assert!(input_attempted);
}


/// Step Once sends exactly one gameplay action and normally uses only two captures.
#[test]
fn one_step_sends_one_gameplay_action() {
    let mut io = fake(&[(draw(), true), (draw(), true)]);
    io.proven_effect = true;
    let (result, input_attempted, _) = exercise(&mut io, draw(), 1, &AtomicBool::new(false));
    assert_eq!(result, Ok(RunOutcome::Completed(1)));
    assert_eq!(io.trace, ["capture", "probe", "input", "wait", "capture"]);
    assert!(input_attempted);
}


/// Multi-Step reuses the accepted result frame while freshly probing every next input.
#[test]
fn two_steps_reuse_result_frame_and_probe_each_input() {
    let mut io = fake(&[(draw(), true), (draw(), true), (draw(), true)]);
    io.proven_effect = true;
    let (result, _, _) = exercise(&mut io, draw(), 2, &AtomicBool::new(false));
    assert_eq!(result, Ok(RunOutcome::Completed(2)));
    assert_eq!(io.trace, ["capture", "probe", "input", "wait", "capture", "probe", "input", "wait", "capture"]);
}


/// A successful move followed by no HALO may refresh Solver without adding a second gameplay action.
#[test]
fn result_solver_refresh_is_immediate_and_separate_from_step_once() {
    let mut io = fake(&[(draw(), true), (PredictedAction::NoHighlight, true), (draw(), true)]);
    io.proven_effect = true;
    let (result, _, _) = exercise(&mut io, draw(), 1, &AtomicBool::new(false));
    assert_eq!(result, Ok(RunOutcome::Completed(1)));
    assert_eq!(io.trace, ["capture", "probe", "input", "wait", "capture", "probe", "solver", "capture"]);
}


/// Uncertain input halts without another capture or any attempted replay.
#[test]
fn uncertain_input_stops_and_preserves_last_frame() {
    let mut io = fake(&[(draw(), true)]);
    io.fail_input = true;
    let (result, input_attempted, latest) = exercise(&mut io, draw(), 2, &AtomicBool::new(false));
    assert!(result.unwrap_err().contains("uncertain"));
    assert_eq!(io.trace, ["capture", "probe", "input"]);
    assert!(input_attempted);
    assert!(latest.is_some());
}


/// Uncertain Solver delivery cannot trigger an immediate retry disguised as recovery.
#[test]
fn uncertain_solver_stops_without_retry_or_capture() {
    let mut io = fake(&[(PredictedAction::NoHighlight, true)]);
    io.fail_solver = true;
    let (result, input_attempted, _) = exercise(&mut io, PredictedAction::NoHighlight, 1, &AtomicBool::new(false));
    assert!(result.unwrap_err().contains("uncertain Solver"));
    assert_eq!(io.trace, ["capture", "probe", "solver"]);
    assert!(input_attempted);
}


/// A pending STOP prevents even the initial capture, and STOP after probe prevents input.
#[test]
fn stop_before_capture_and_after_probe_prevents_input() {
    let mut io = fake(&[]);
    let (result, _, _) = exercise(&mut io, draw(), 1, &AtomicBool::new(true));
    assert!(result.is_err());
    assert!(io.trace.is_empty());
    let stop = AtomicBool::new(false);
    let mut after_probe = FakeIo {
        frames: VecDeque::from([observation(draw(), true)]),
        trace: Vec::new(), proven_effect: false, effect_checks: 0, capture_failure: None, fail_input: false, fail_solver: false,
        stop_on_probe: Some(&stop), probes_before_stop: 0, stop_on_wait: None,
        stop_on_diagnostic_capture: None,
    };
    let (result, _, _) = exercise(&mut after_probe, draw(), 1, &stop);
    assert!(result.unwrap_err().contains("STOP"));
    assert_eq!(after_probe.trace, ["capture", "probe"]);
}


/// STOP during settling leaves the delivered action uncertain and never replays it.
#[test]
fn stop_during_settle_preserves_uncertain_input_without_next_capture() {
    let stop = AtomicBool::new(false);
    let mut io = FakeIo {
        frames: VecDeque::from([observation(draw(), true)]),
        trace: Vec::new(), proven_effect: false, effect_checks: 0, capture_failure: None, fail_input: false, fail_solver: false,
        stop_on_probe: None, probes_before_stop: 0, stop_on_wait: Some(&stop),
        stop_on_diagnostic_capture: None,
    };
    let (result, input_attempted, latest) = exercise(&mut io, draw(), 2, &stop);
    assert!(result.unwrap_err().contains("STOP"));
    assert_eq!(io.trace, ["capture", "probe", "input", "wait"]);
    assert!(input_attempted);
    assert!(latest.is_some());
}


/// An unsupported post-action screen gets no Solver, completion or restart input.
#[test]
fn unknown_result_scene_stops_without_terminal_inputs() {
    let mut io = fake(&[(draw(), true), (PredictedAction::NoHighlight, false)]);
    let (result, input_attempted, _) = exercise(&mut io, draw(), 2, &AtomicBool::new(false));
    assert!(result.unwrap_err().contains("unsupported Klondike scene"));
    assert_eq!(io.trace, ["capture", "probe", "input", "wait", "capture"]);
    assert!(input_attempted);
}


/// Solver may restore a HALO, but cannot substitute for the preceding gameplay effect.
#[test]
fn refreshed_halo_without_effect_never_authorises_another_gameplay_input() {
    let mut states = vec![(draw(), true), (PredictedAction::NoHighlight, true)];
    states.extend(vec![(draw(), true); REOBSERVATION_LIMIT + 1]);
    let mut io = fake(&states);
    let (result, input_attempted, latest) = exercise(&mut io, draw(), 2, &AtomicBool::new(false));
    assert!(result.unwrap_err().contains("previous effect verified=false"));
    assert_eq!(io.trace.iter().filter(|entry| **entry == "input").count(), 1);
    assert_eq!(io.trace.iter().filter(|entry| **entry == "solver").count(), 1);
    assert_eq!(io.trace.iter().filter(|entry| **entry == "capture").count(), REOBSERVATION_LIMIT + 3);
    assert!(input_attempted);
    assert!(latest.is_some());
}


/// Continuous play verifies successive actions and still checks STOP before every input.
#[test]
fn continuous_play_runs_until_stop_at_a_fresh_probe() {
    let stop = AtomicBool::new(false);
    let mut io = fake(&[(draw(), true), (draw(), true), (draw(), true)]);
    io.proven_effect = true;
    io.stop_on_probe = Some(&stop);
    io.probes_before_stop = 2;
    let (result, input_attempted, latest) = exercise(&mut io, draw(), 0, &stop);
    assert!(result.unwrap_err().contains("STOP"));
    assert_eq!(io.trace, ["capture", "probe", "input", "wait", "capture", "probe", "input", "wait", "capture", "probe"]);
    assert_eq!(io.effect_checks, 2);
    assert!(input_attempted);
    assert!(latest.is_some());
}


/// Zero removes the action limit but never removes the per-context no-HALO recovery bound.
#[test]
fn continuous_no_halo_recovery_remains_bounded() {
    let mut states = vec![(draw(), true)];
    states.extend(vec![(PredictedAction::NoHighlight, true); REOBSERVATION_LIMIT + 2]);
    let mut io = fake(&states);
    io.proven_effect = true;
    let (result, _, latest) = exercise(&mut io, draw(), 0, &AtomicBool::new(false));
    assert!(result.unwrap_err().contains("bounded result observations exhausted"));
    assert_eq!(io.trace.iter().filter(|entry| **entry == "input").count(), 1);
    assert_eq!(io.trace.iter().filter(|entry| **entry == "solver").count(), 1);
    assert_eq!(io.trace.iter().filter(|entry| **entry == "capture").count(), REOBSERVATION_LIMIT + 3);
    assert!(latest.is_some());
}


/// The next Recycle HALO cannot excuse an unverified preceding RIGHT transfer in any run mode.
#[test]
fn recycle_after_unverified_right_never_authorises_recycle_input() {
    let right = PredictedAction::Action(klondike::canonical_action(KlondikeTarget::Waste { offset: 0 }).unwrap());
    let recycle = PredictedAction::Action(klondike::canonical_action(KlondikeTarget::Recycle).unwrap());


    for limit in [0, 2] {
        let mut states = vec![(right, true)];
        states.extend(vec![(recycle, true); REOBSERVATION_LIMIT + 1]);
        let mut io = fake(&states);
        let (result, _, latest) = exercise(&mut io, right, limit, &AtomicBool::new(false));
        let error = result.unwrap_err();
        assert!(error.contains("previous effect verified=false"));
        assert!(error.contains("Recycle"));
        assert_eq!(io.trace.iter().filter(|entry| **entry == "input").count(), 1);
        assert_eq!(io.trace.iter().filter(|entry| **entry == "solver").count(), 0);
        assert_eq!(io.effect_checks, REOBSERVATION_LIMIT + 1);
        assert_eq!(latest.unwrap().prediction, recycle);
    }
}


/// Solve receives one click, the configured normal settle and one diagnostic capture in every mode.
#[test]
fn solve_is_requested_once_then_stops_without_effect_or_recovery() {
    let solve = PredictedAction::Action(klondike::canonical_action(KlondikeTarget::Solve).unwrap());


    for limit in [0, 1, 10] {


        for result_scene in [(solve, true), (PredictedAction::NoHighlight, true), (PredictedAction::NoHighlight, false)] {
            let mut io = fake(&[(solve, true), result_scene]);
            io.proven_effect = true;
            let (result, input_attempted, latest) = exercise(&mut io, solve, limit, &AtomicBool::new(false));
            assert_eq!(result, Ok(RunOutcome::SolveRequested { previous_verified: 0 }));
            assert_eq!(io.trace, ["capture", "probe", "input", "wait", "capture"]);
            assert_eq!(io.effect_checks, 0);
            assert!(input_attempted);
            let diagnostic = latest.unwrap();
            assert_eq!(diagnostic.prediction, PredictedAction::NoHighlight);
            assert!(!diagnostic.gameplay_scene);
            assert!(diagnostic.game_progress.is_none());
        }
    }
}


/// A fresh Solve appearing instead of the approved card requires review before it can be clicked.
#[test]
fn unexpected_initial_solve_is_not_clicked() {
    let solve = PredictedAction::Action(klondike::canonical_action(KlondikeTarget::Solve).unwrap());
    let mut io = fake(&[(solve, true)]);
    let (result, input_attempted, _) = exercise(&mut io, draw(), 0, &AtomicBool::new(false));
    assert!(result.unwrap_err().contains("differs from approved preview"));
    assert_eq!(io.trace, ["capture"]);
    assert!(!input_attempted);
}


/// A proven move can lead to one Solve request without labelling Solve itself as verified.
#[test]
fn continuous_move_then_solve_reports_only_previous_verified_moves() {
    let solve = PredictedAction::Action(klondike::canonical_action(KlondikeTarget::Solve).unwrap());
    let mut io = fake(&[(draw(), true), (solve, true), (PredictedAction::NoHighlight, false)]);
    io.proven_effect = true;
    let (result, _, _) = exercise(&mut io, draw(), 0, &AtomicBool::new(false));
    assert_eq!(result, Ok(RunOutcome::SolveRequested { previous_verified: 1 }));
    assert_eq!(io.effect_checks, 1);
    assert_eq!(io.trace.iter().filter(|entry| **entry == "input").count(), 2);
    assert_eq!(io.trace.iter().filter(|entry| **entry == "solver").count(), 0);
}


/// Uncertain Solve delivery stops immediately and is never replayed or followed by recovery.
#[test]
fn uncertain_solve_input_stops_without_recapture_or_retry() {
    let solve = PredictedAction::Action(klondike::canonical_action(KlondikeTarget::Solve).unwrap());
    let mut io = fake(&[(solve, true)]);
    io.fail_input = true;
    let (result, _, _) = exercise(&mut io, solve, 0, &AtomicBool::new(false));
    assert!(result.unwrap_err().contains("uncertain"));
    assert_eq!(io.trace, ["capture", "probe", "input"]);
    assert_eq!(io.effect_checks, 0);
}


/// Both continuous diagnostic counters fail before a wrapping increment can authorise input.
#[test]
fn cumulative_counters_reject_overflow() {
    assert_eq!(next_counter(0, "gameplay action").unwrap(), 1);
    assert_eq!(next_counter(usize::MAX - 1, "Solver refresh").unwrap(), usize::MAX);
    assert!(next_counter(usize::MAX, "gameplay action").unwrap_err().contains("before further input"));
    assert!(next_counter(usize::MAX, "Solver refresh").unwrap_err().contains("before further input"));
}


/// Separate resolved contexts may each refresh once during continuous play without a false total limit.
#[test]
fn continuous_recovery_budget_resets_only_after_a_proven_move() {
    let stop = AtomicBool::new(false);
    let states = [
        (draw(), true), (PredictedAction::NoHighlight, true), (draw(), true),
        (PredictedAction::NoHighlight, true), (draw(), true),
    ];
    let mut io = fake(&states);
    io.proven_effect = true;
    io.stop_on_probe = Some(&stop);
    io.probes_before_stop = 4;
    let (result, _, _) = exercise(&mut io, draw(), 0, &stop);
    assert!(result.unwrap_err().contains("STOP"));
    assert_eq!(io.trace.iter().filter(|entry| **entry == "input").count(), 2);
    assert_eq!(io.trace.iter().filter(|entry| **entry == "solver").count(), 2);
    assert_eq!(io.trace.iter().filter(|entry| **entry == "capture").count(), 5);
}


/// Solve preserves newly captured pixels even if STOP arrives during that diagnostic capture.
#[test]
fn solve_late_stop_retains_the_new_diagnostic_frame() {
    let solve = PredictedAction::Action(klondike::canonical_action(KlondikeTarget::Solve).unwrap());
    let stop = AtomicBool::new(false);
    let mut io = fake(&[(solve, true), (PredictedAction::NoHighlight, false)]);
    io.frames.back_mut().unwrap().frame.pixels[0] = 123;
    io.stop_on_diagnostic_capture = Some(&stop);
    let (result, _, latest) = exercise(&mut io, solve, 0, &stop);
    assert!(result.unwrap_err().contains("STOP"));
    assert_eq!(io.trace, ["capture", "probe", "input", "wait", "capture"]);
    let diagnostic = latest.unwrap();
    assert_eq!(diagnostic.frame.pixels[0], 123);
    assert_eq!(diagnostic.prediction, PredictedAction::NoHighlight);
    assert!(!diagnostic.gameplay_scene);
    assert_eq!(io.effect_checks, 0);
}


/// A decoded-frame analysis failure retains that frame and grants no input authority.
#[test]
fn failed_capture_analysis_retains_new_pixels_before_propagating_error() {
    let mut io = fake(&[(draw(), true)]);
    io.frames.front_mut().unwrap().frame.pixels[0] = 77;
    io.capture_failure = Some("simulated unsupported capture dimensions");
    let (result, input_attempted, latest) = exercise(&mut io, draw(), 0, &AtomicBool::new(false));
    assert!(result.unwrap_err().contains("unsupported capture dimensions"));
    assert_eq!(io.trace, ["capture"]);
    assert!(!input_attempted);
    let diagnostic = latest.unwrap();
    assert_eq!(diagnostic.frame.pixels[0], 77);
    assert_eq!(diagnostic.prediction, PredictedAction::NoHighlight);
    assert!(!diagnostic.gameplay_scene);
    assert!(diagnostic.game_progress.is_none());
}


/// The real classification boundary preserves decoded pixels on geometry failure or late STOP.
#[test]
fn classifier_failure_and_late_stop_return_diagnostic_pixels() {
    let scan_state = TableauScanState::for_mode(crate::game::GameMode::Klondike);


    for stopped in [false, true] {
        let mut frame = observation(draw(), true).frame;
        frame.width = 1_919;
        frame.pixels[0] = 61;
        let result = classify_captured_result(frame, &scan_state, &AtomicBool::new(stopped));


        let CaptureResult::Diagnostic { frame, reason } = result else {
            panic!("invalid geometry or late STOP must not classify an actionable frame");
        };
        assert_eq!(frame.width, 1_919);
        assert_eq!(frame.pixels[0], 61);
        assert!(!reason.is_empty());
    }
}
