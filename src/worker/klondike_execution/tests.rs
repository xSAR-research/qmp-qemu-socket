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
    /// Make a gameplay delivery uncertain after recording its single attempt.
    fail_input: bool,
    /// Make Solver delivery uncertain after recording its single attempt.
    fail_solver: bool,
    /// Set STOP at the fresh probe boundary to test the final pre-input guard.
    stop_on_probe: Option<&'a AtomicBool>,
    /// Set STOP during the settle wait to test interruption after one gameplay input.
    stop_on_wait: Option<&'a AtomicBool>,
}


impl KlondikeIo for FakeIo<'_> {


    fn capture(&mut self) -> Result<FrameObservation, String> {
        self.trace.push("capture");
        self.frames.pop_front().ok_or_else(|| "unexpected capture".to_owned())
    }


    fn probe(&mut self) -> Result<(), String> {
        self.trace.push("probe");


        if let Some(stop) = self.stop_on_probe {
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
fn fake(states: &[(PredictedAction, bool)]) -> FakeIo<'static> {
    FakeIo {
        frames: states.iter().map(|(prediction, scene)| observation(*prediction, *scene)).collect(),
        trace: Vec::new(),
        proven_effect: false,
        fail_input: false,
        fail_solver: false,
        stop_on_probe: None,
        stop_on_wait: None,
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


/// Zero/unbounded and excessive limits fail before all capture and input operations.
#[test]
fn limits_fail_before_io() {


    for limit in [0, KLONDIKE_MAX_MULTI_STEP_ACTIONS + 1] {
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
        trace: Vec::new(), proven_effect: false, fail_input: false, fail_solver: false,
        stop_on_probe: Some(&stop), stop_on_wait: None,
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
        trace: Vec::new(), proven_effect: false, fail_input: false, fail_solver: false,
        stop_on_probe: None, stop_on_wait: Some(&stop),
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
