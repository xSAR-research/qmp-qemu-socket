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
    /// Controlled mode-owned source replacement proof, independent of full effect.
    proven_source_replacement: bool,
    /// Optional ordered verdicts distinguish verified and HALO counts within one run.
    effect_verdicts: VecDeque<EffectVerdict>,
    /// Acknowledged or attempted gameplay targets, distinguishing a new action from replay.
    inputs: Vec<PredictedAction>,
    /// Number of effect evaluations; Solve must bypass effect inference completely.
    effect_checks: usize,
    /// Independent completion candidates supplied for fresh completion observations.
    completion_candidates: VecDeque<bool>,
    /// Number of independent completion evaluations.
    completion_checks: usize,
    /// Measured waits requested by the production controller.
    waits: Vec<Duration>,
    /// Independently classified terminal stages, consumed in observation order.
    terminal_stages: VecDeque<Option<klondike::terminal::TerminalStage>>,
    /// Ordered terminal controls, distinct from ordinary gameplay actions.
    terminal_inputs: Vec<klondike::terminal::TerminalStage>,
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
    /// Permit this many waits before STOP, distinguishing settle from recapture.
    waits_before_stop: usize,
    /// Set STOP after an ordinary fresh capture has acquired pixels.
    stop_on_capture: Option<&'a AtomicBool>,
    /// Set STOP after the diagnostic pixels are acquired but before the controller receives them.
    stop_on_diagnostic_capture: Option<&'a AtomicBool>,
}


impl KlondikeIo for FakeIo<'_> {


    fn capture(&mut self) -> Result<CaptureResult, String> {
        self.trace.push("capture");
        let observation = self.frames.pop_front().ok_or_else(|| "unexpected capture".to_owned())?;


        if let Some(stop) = self.stop_on_capture {
            stop.store(true, Ordering::Release);
            return Ok(CaptureResult::Diagnostic { frame: observation.frame, reason: "STOP during capture".to_owned() });
        }


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


    fn completion(&mut self, _: &CapturedFrame) -> Result<bool, String> {
        self.completion_checks += 1;
        Ok(self.completion_candidates.pop_front().unwrap_or(false))
    }


    fn terminal_stage(&mut self, _: &CapturedFrame) -> Result<Option<klondike::terminal::TerminalStage>, String> {
        Ok(self.terminal_stages.pop_front().unwrap_or(None))
    }


    fn terminal_input(&mut self, stage: klondike::terminal::TerminalStage) -> Result<(), String> {
        self.trace.push("terminal");
        self.terminal_inputs.push(stage);


        if self.fail_input {
            Err("simulated uncertain terminal input".to_owned())
        } else {
            Ok(())
        }
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


    fn input(&mut self, plan: StepPlan) -> Result<(), String> {
        self.trace.push("input");
        self.inputs.push(plan.before());


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


    fn wait(&mut self, duration: Duration) -> Result<(), String> {
        self.trace.push("wait");
        self.waits.push(duration);


        if let Some(stop) = self.stop_on_wait
            && self.waits.len() > self.waits_before_stop
        {
            stop.store(true, Ordering::Release);
            return Err("STOP during wait".to_owned());
        }
        Ok(())
    }


    fn effect(&mut self, _: &CapturedFrame, _: &CapturedFrame, _: GuidedAction) -> Result<EffectVerdict, String> {
        self.effect_checks += 1;
        Ok(self.effect_verdicts.pop_front().unwrap_or(EffectVerdict {
            verified: self.proven_effect, source_replaced: self.proven_source_replacement,
        }))
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
        proven_source_replacement: false,
        effect_verdicts: VecDeque::new(),
        inputs: Vec::new(),
        effect_checks: 0,
        completion_candidates: VecDeque::new(),
        completion_checks: 0,
        waits: Vec::new(),
        terminal_stages: VecDeque::new(),
        terminal_inputs: Vec::new(),
        capture_failure: None,
        fail_input: false,
        fail_solver: false,
        stop_on_probe: None,
        probes_before_stop: 0,
        stop_on_wait: None,
        waits_before_stop: 0,
        stop_on_capture: None,
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
    exercise_with_settings(io, approved, StepRunSettings::new(AnimationSettleDelays::default(), limit), stop)
}


/// Exercise an explicit immutable timing snapshot through the production controller.
fn exercise_with_settings(
    io: &mut impl KlondikeIo,
    approved: PredictedAction,
    settings: StepRunSettings,
    stop: &AtomicBool,
) -> (Result<RunOutcome, String>, bool, Option<FrameObservation>) {
    let sink = event_sink();
    let mut latest = None;
    let mut input_attempted = false;
    let result = drive_run(io, approved, settings, &sink, stop, &mut latest, &mut input_attempted);
    (result, input_attempted, latest)
}


/// Create a bounded production frame mailbox without making UI consumption part of a flow test.
fn event_sink() -> WorkerEventSink {
    let (events, _receiver) = std::sync::mpsc::channel();
    WorkerEventSink {
        events,
        latest_frame: super::super::LatestFrameSlot::default(),
        capture_context: std::sync::Mutex::new(None),
    }
}


/// Start an already confirmed win at its fresh terminal frame to inject later stage failures.
fn exercise_terminal(
    io: &mut impl KlondikeIo,
    stop: &AtomicBool,
) -> (Result<(), String>, bool, Option<FrameObservation>) {
    let mut latest = Some(observation(PredictedAction::NoHighlight, false));
    let mut input_attempted = false;
    let result = resume_after_win(
        io, &mut latest, StepRunSettings::new(AnimationSettleDelays::default(), 0),
        &event_sink(), stop, &mut input_attempted,
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
fn initial_changed_prediction_requires_explicit_fresh_approval() {
    let mut io = fake(&[(draw(), true)]);
    let (result, input_attempted, latest) = exercise(&mut io, PredictedAction::NoHighlight, 1, &AtomicBool::new(false));
    assert_eq!(result, Ok(RunOutcome::PreviewChanged));
    assert_eq!(io.trace, ["capture"]);
    assert!(!input_attempted);
    assert_eq!(latest.unwrap().prediction, draw());
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
    assert_eq!(result, Ok(RunOutcome::Completed { verified: 1, halo: 0 }));
    assert_eq!(io.trace, ["capture", "probe", "input", "wait", "capture"]);
    assert!(input_attempted);
}


/// Multi-Step reuses the accepted result frame while freshly probing every next input.
#[test]
fn two_steps_reuse_result_frame_and_probe_each_input() {
    let mut io = fake(&[(draw(), true), (draw(), true), (draw(), true)]);
    io.proven_effect = true;
    let (result, _, _) = exercise(&mut io, draw(), 2, &AtomicBool::new(false));
    assert_eq!(result, Ok(RunOutcome::Completed { verified: 2, halo: 0 }));
    assert_eq!(io.trace, ["capture", "probe", "input", "wait", "capture", "probe", "input", "wait", "capture"]);
}


/// A successful move followed by no HALO may refresh Solver without adding a second gameplay action.
#[test]
fn result_solver_refresh_is_immediate_and_separate_from_step_once() {
    let mut io = fake(&[(draw(), true), (PredictedAction::NoHighlight, true), (draw(), true)]);
    io.proven_effect = true;
    let (result, _, _) = exercise(&mut io, draw(), 1, &AtomicBool::new(false));
    assert_eq!(result, Ok(RunOutcome::Completed { verified: 1, halo: 0 }));
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
        trace: Vec::new(), proven_effect: false, proven_source_replacement: false, effect_verdicts: VecDeque::new(), inputs: Vec::new(), effect_checks: 0,
        completion_candidates: VecDeque::new(), completion_checks: 0, waits: Vec::new(), terminal_stages: VecDeque::new(), terminal_inputs: Vec::new(), capture_failure: None, fail_input: false, fail_solver: false,
        stop_on_probe: Some(&stop), probes_before_stop: 0, stop_on_wait: None, waits_before_stop: 0,
        stop_on_capture: None,
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
        trace: Vec::new(), proven_effect: false, proven_source_replacement: false, effect_verdicts: VecDeque::new(), inputs: Vec::new(), effect_checks: 0,
        completion_candidates: VecDeque::new(), completion_checks: 0, waits: Vec::new(), terminal_stages: VecDeque::new(), terminal_inputs: Vec::new(), capture_failure: None, fail_input: false, fail_solver: false,
        stop_on_probe: None, probes_before_stop: 0, stop_on_wait: Some(&stop), waits_before_stop: 0,
        stop_on_capture: None,
        stop_on_diagnostic_capture: None,
    };
    let (result, input_attempted, latest) = exercise(&mut io, draw(), 2, &stop);
    assert!(result.unwrap_err().contains("STOP"));
    assert_eq!(io.trace, ["capture", "probe", "input", "wait"]);
    assert!(input_attempted);
    assert!(latest.is_some());
}


/// Persistent unsupported result scenes exhaust the shared read-only budget.
#[test]
fn unknown_result_scene_stops_without_terminal_inputs() {
    let mut states = vec![(draw(), true)];
    states.extend([(PredictedAction::NoHighlight, false); REOBSERVATION_LIMIT + 1]);
    let mut io = fake(&states);
    let (result, input_attempted, _) = exercise(&mut io, draw(), 2, &AtomicBool::new(false));
    assert!(result.unwrap_err().contains("unsupported Klondike scene"));
    assert_eq!(io.inputs, [draw()]);
    assert_eq!(io.effect_checks, 0);
    assert_eq!(io.trace.iter().filter(|entry| **entry == "capture").count(), REOBSERVATION_LIMIT + 2);
    assert!(!io.trace.contains(&"solver"));
    assert!(io.terminal_inputs.is_empty());
    assert!(input_attempted);
}


/// A settled supported result can prove the original action after a scene gap.
#[test]
fn unsupported_result_recaptures_before_evaluating_the_original_effect() {
    let mut io = fake(&[(draw(), true), (PredictedAction::NoHighlight, false), (draw(), true)]);
    io.proven_effect = true;
    let (result, attempted, latest) = exercise(&mut io, draw(), 1, &AtomicBool::new(false));
    assert!(matches!(result, Ok(RunOutcome::Completed { verified: 1, halo: 0 })));
    assert!(attempted);
    assert_eq!(io.inputs, [draw()]);
    assert_eq!(io.effect_checks, 1);
    assert_eq!(io.trace, ["capture", "probe", "input", "wait", "capture", "wait", "capture"]);
    assert!(!io.trace.contains(&"solver"));
    assert!(latest.unwrap().gameplay_scene);
}


/// A scene gap and later unchanged HALO share one finite budget without replay.
#[test]
fn unsupported_result_does_not_reset_effect_reobservation_budget() {
    let mut states = vec![(draw(), true), (PredictedAction::NoHighlight, false)];
    states.extend([(draw(), true); REOBSERVATION_LIMIT]);
    let mut io = fake(&states);
    let (result, _, _) = exercise(&mut io, draw(), 2, &AtomicBool::new(false));
    assert!(result.unwrap_err().contains("previous effect verified=false"));
    assert_eq!(io.inputs, [draw()]);
    assert_eq!(io.effect_checks, REOBSERVATION_LIMIT);
    assert_eq!(io.trace.iter().filter(|entry| **entry == "capture").count(), REOBSERVATION_LIMIT + 2);
    assert!(!io.trace.contains(&"solver"));
    assert!(io.terminal_inputs.is_empty());
}


/// STOP during unsupported-scene recovery retains the frame and sends no retry.
#[test]
fn stop_interrupts_unsupported_result_reobservation() {
    let stop = AtomicBool::new(false);
    let mut io = fake(&[(draw(), true), (PredictedAction::NoHighlight, false)]);
    io.stop_on_wait = Some(&stop);
    io.waits_before_stop = 1;
    let (result, attempted, latest) = exercise(&mut io, draw(), 2, &stop);
    assert!(result.unwrap_err().contains("STOP"));
    assert!(attempted);
    assert_eq!(io.inputs, [draw()]);
    assert_eq!(io.effect_checks, 0);
    assert_eq!(io.trace, ["capture", "probe", "input", "wait", "capture", "wait"]);
    assert!(!latest.unwrap().gameplay_scene);
    assert!(!io.trace.contains(&"solver"));
    assert!(io.terminal_inputs.is_empty());
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


/// The next Recycle HALO cannot excuse an unchanged preceding RIGHT source in any run mode.
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


/// Solve sends one click, its own delay and bounded diagnostic observations without recovery.
#[test]
fn unresolved_solve_is_requested_once_then_stops_without_effect_or_recovery() {
    let solve = PredictedAction::Action(klondike::canonical_action(KlondikeTarget::Solve).unwrap());


    for limit in [0, 1, 10] {


        for result_scene in [(solve, true), (PredictedAction::NoHighlight, true), (PredictedAction::NoHighlight, false)] {
            let mut states = vec![(solve, true)];
            states.extend([result_scene; SOLVE_COMPLETION_REOBSERVATION_LIMIT + 1]);
            let mut io = fake(&states);
            io.proven_effect = true;
            let (result, input_attempted, latest) = exercise(&mut io, solve, limit, &AtomicBool::new(false));
            assert_eq!(result, Ok(RunOutcome::SolveRequested { previous_verified: 0, previous_halo: 0 }));
            assert_eq!(io.trace.iter().filter(|entry| **entry == "input").count(), 1);
            assert_eq!(io.trace.iter().filter(|entry| **entry == "capture").count(), SOLVE_COMPLETION_REOBSERVATION_LIMIT + 2);
            assert_eq!(io.completion_checks, SOLVE_COMPLETION_REOBSERVATION_LIMIT + 1);
            assert_eq!(io.effect_checks, 0);
            assert_eq!(io.waits[0], AnimationSettleDelays::default().klondike_solve);
            assert!(io.terminal_inputs.is_empty());
            assert!(!io.trace.contains(&"solver"));
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
    let (result, input_attempted, latest) = exercise(&mut io, draw(), 0, &AtomicBool::new(false));
    assert_eq!(result, Ok(RunOutcome::PreviewChanged));
    assert_eq!(io.trace, ["capture"]);
    assert!(!input_attempted);
    assert_eq!(latest.unwrap().prediction, solve);
}


/// A proven move can lead to one Solve request without labelling Solve itself as verified.
#[test]
fn continuous_move_then_solve_reports_only_previous_verified_moves() {
    let solve = PredictedAction::Action(klondike::canonical_action(KlondikeTarget::Solve).unwrap());
    let mut states = vec![(draw(), true), (solve, true)];
    states.extend([(PredictedAction::NoHighlight, false); SOLVE_COMPLETION_REOBSERVATION_LIMIT + 1]);
    let mut io = fake(&states);
    io.proven_effect = true;
    let (result, _, _) = exercise(&mut io, draw(), 0, &AtomicBool::new(false));
    assert_eq!(result, Ok(RunOutcome::SolveRequested { previous_verified: 1, previous_halo: 0 }));
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


/// Undo-related RIGHT-to-tableau drift refreshes the preview without sending either action.
#[test]
fn changed_right_preview_can_be_explicitly_approved_on_the_second_request() {
    let right = PredictedAction::Action(klondike::canonical_action(KlondikeTarget::Waste { offset: 1 }).unwrap());
    let tableau = PredictedAction::Action(klondike::canonical_action(KlondikeTarget::Tableau { column: 5, top: 571, bottom: 761 }).unwrap());
    let mut first = fake(&[(tableau, true)]);
    let (result, input_attempted, latest) = exercise(&mut first, right, 1, &AtomicBool::new(false));
    assert_eq!(result, Ok(RunOutcome::PreviewChanged));
    assert_eq!(first.trace, ["capture"]);
    assert!(!input_attempted);
    let fresh = latest.unwrap().prediction;
    assert_eq!(fresh, tableau);
    let mut second = fake(&[(fresh, true), (draw(), true)]);
    second.proven_effect = true;
    let (result, input_attempted, _) = exercise(&mut second, fresh, 1, &AtomicBool::new(false));
    assert_eq!(result, Ok(RunOutcome::Completed { verified: 1, halo: 0 }));
    assert!(input_attempted);
    assert_eq!(second.trace, ["capture", "probe", "input", "wait", "capture"]);
}


/// A disappeared approved target cannot silently authorise Solver; the no-HALO preview needs approval.
#[test]
fn changed_action_to_no_halo_requires_new_recovery_approval() {
    let mut first = fake(&[(PredictedAction::NoHighlight, true)]);
    let (result, input_attempted, latest) = exercise(&mut first, draw(), 1, &AtomicBool::new(false));
    assert_eq!(result, Ok(RunOutcome::PreviewChanged));
    assert_eq!(first.trace, ["capture"]);
    assert!(!input_attempted);
    assert_eq!(latest.unwrap().prediction, PredictedAction::NoHighlight);
    let mut second = fake(&[(PredictedAction::NoHighlight, true), (draw(), true)]);
    let (result, input_attempted, _) = exercise(&mut second, PredictedAction::NoHighlight, 1, &AtomicBool::new(false));
    assert_eq!(result, Ok(RunOutcome::RecoveryOnly));
    assert!(input_attempted);
    assert_eq!(second.trace, ["capture", "probe", "solver", "capture"]);
}


/// A fabricated changed target cannot be offered as an approved fresh preview.
#[test]
fn malformed_changed_prediction_is_not_published_as_ready() {
    let PredictedAction::Action(mut altered) = draw() else { unreachable!() };
    altered.anchor.x += 1;
    let mut io = fake(&[(PredictedAction::Action(altered), true)]);
    let (result, input_attempted, _) = exercise(&mut io, draw(), 1, &AtomicBool::new(false));
    assert!(result.unwrap_err().contains("Klondike target rejected"));
    assert_eq!(io.trace, ["capture"]);
    assert!(!input_attempted);
}


/// Two fresh positive completion verdicts stop finite Solve runs before terminal input.
#[test]
fn finite_solve_confirms_one_game_without_restart_and_uses_separate_delay() {
    let solve = PredictedAction::Action(klondike::canonical_action(KlondikeTarget::Solve).unwrap());


    for limit in [1, 10] {
        let mut io = fake(&[(solve, true), (PredictedAction::NoHighlight, false), (PredictedAction::NoHighlight, false)]);
        io.completion_candidates = VecDeque::from([true, true]);
        let delays = AnimationSettleDelays::default().with_klondike_solve_millis(1_750);
        let (result, input_attempted, _) = exercise_with_settings(&mut io, solve, StepRunSettings::new(delays, limit), &AtomicBool::new(false));
        assert_eq!(result, Ok(RunOutcome::GameWon { previous_verified: 0, previous_halo: 0 }));
        assert_eq!(io.trace, ["capture", "probe", "input", "wait", "capture", "wait", "capture"]);
        assert_eq!(io.waits, [delays.klondike_solve, delays.klondike_reobserve]);
        assert_eq!(delays.klondike_settle, Duration::from_millis(750));
        assert_eq!(io.effect_checks, 0);
        assert!(io.terminal_inputs.is_empty());
        assert!(input_attempted);
    }
}


/// An interrupted positive sequence requires two new consecutive observations.
#[test]
fn solve_completion_resets_after_an_incomplete_frame() {
    let solve = PredictedAction::Action(klondike::canonical_action(KlondikeTarget::Solve).unwrap());
    let mut states = vec![(solve, true)];
    states.extend([(PredictedAction::NoHighlight, false); REOBSERVATION_LIMIT + 1]);
    let mut io = fake(&states);
    io.completion_candidates = VecDeque::from([true, false, true, true]);
    let (result, _, _) = exercise(&mut io, solve, 1, &AtomicBool::new(false));
    assert_eq!(result, Ok(RunOutcome::GameWon { previous_verified: 0, previous_halo: 0 }));
    assert_eq!(io.completion_checks, 4);
    assert_eq!(io.trace.iter().filter(|entry| **entry == "input").count(), 1);
    assert!(!io.trace.contains(&"solver"));
    assert!(io.terminal_inputs.is_empty());
}


/// One positive completion image cannot establish a win or trigger terminal controls.
#[test]
fn solve_completion_never_accepts_a_single_positive_frame() {
    let solve = PredictedAction::Action(klondike::canonical_action(KlondikeTarget::Solve).unwrap());
    let mut states = vec![(solve, true)];
    states.extend([(PredictedAction::NoHighlight, false); SOLVE_COMPLETION_REOBSERVATION_LIMIT + 1]);
    let mut io = fake(&states);
    io.completion_candidates = VecDeque::from([true, false, false, false]);
    let (result, _, _) = exercise(&mut io, solve, 0, &AtomicBool::new(false));
    assert_eq!(result, Ok(RunOutcome::SolveRequested { previous_verified: 0, previous_halo: 0 }));
    assert_eq!(io.completion_checks, SOLVE_COMPLETION_REOBSERVATION_LIMIT + 1);
    assert!(io.terminal_inputs.is_empty());
}


/// The observed false/true/false/false transition must not consume Solve's entire
/// win budget. Later settled positive frames confirm one game without more input.
#[test]
fn solve_waits_through_observed_transition_before_confirming_settled_win() {
    let solve = PredictedAction::Action(klondike::canonical_action(KlondikeTarget::Solve).unwrap());
    let mut states = vec![(solve, true)];
    states.extend([(PredictedAction::NoHighlight, false); 8]);
    let mut io = fake(&states);
    io.completion_candidates = VecDeque::from([false, true, false, false, false, false, true, true]);
    let delays = AnimationSettleDelays::default().with_klondike_millis(750, 1_500);
    let (result, attempted, latest) = exercise_with_settings(
        &mut io, solve, StepRunSettings::new(delays, 1), &AtomicBool::new(false),
    );
    assert_eq!(result, Ok(RunOutcome::GameWon { previous_verified: 0, previous_halo: 0 }));
    assert_eq!(io.completion_checks, 8);
    assert_eq!(io.inputs, [solve]);
    assert_eq!(io.waits[0], delays.klondike_solve);
    assert!(io.waits[1..].iter().all(|delay| *delay == delays.klondike_reobserve));
    assert!(attempted);
    assert!(latest.is_some());
    assert!(io.terminal_inputs.is_empty());
    assert!(!io.trace.contains(&"solver"));
    assert_eq!(io.effect_checks, 0);
}


/// Isolated positives never extend Solve's total budget or authorise a restart.
#[test]
fn intermittent_win_candidates_cannot_keep_solve_wait_alive_indefinitely() {
    let solve = PredictedAction::Action(klondike::canonical_action(KlondikeTarget::Solve).unwrap());
    let mut states = vec![(solve, true)];
    states.extend([(PredictedAction::NoHighlight, false); SOLVE_COMPLETION_REOBSERVATION_LIMIT + 1]);
    let mut io = fake(&states);
    io.completion_candidates = (0..=SOLVE_COMPLETION_REOBSERVATION_LIMIT)
        .map(|index| index % 2 == 0).collect();
    let (result, _, latest) = exercise(&mut io, solve, 0, &AtomicBool::new(false));
    assert_eq!(result, Ok(RunOutcome::SolveRequested { previous_verified: 0, previous_halo: 0 }));
    assert_eq!(io.completion_checks, SOLVE_COMPLETION_REOBSERVATION_LIMIT + 1);
    assert_eq!(io.inputs, [solve]);
    assert_eq!(io.waits.len(), SOLVE_COMPLETION_REOBSERVATION_LIMIT + 1);
    assert!(io.terminal_inputs.is_empty());
    assert!(!io.trace.contains(&"solver"));
    assert!(latest.is_some());
}


/// Positively classified terminal completion pre-empts no-HALO Solver recovery without input.
#[test]
fn initial_confirmed_win_sends_no_input_in_a_finite_run() {
    let mut io = fake(&[(PredictedAction::NoHighlight, false), (PredictedAction::NoHighlight, false)]);
    io.completion_candidates = VecDeque::from([true, true]);
    let (result, input_attempted, latest) = exercise(&mut io, draw(), 1, &AtomicBool::new(false));
    assert_eq!(result, Ok(RunOutcome::GameWon { previous_verified: 0, previous_halo: 0 }));
    assert_eq!(io.trace, ["capture", "wait", "capture"]);
    assert_eq!(io.completion_checks, 2);
    assert!(!input_attempted);
    assert!(latest.is_some());
    assert!(io.terminal_inputs.is_empty());
}


/// Independent completed-game evidence never relabels an unverified last action as verified.
#[test]
fn ordinary_last_move_preserves_its_effect_verdict_at_the_win_endpoint() {


    for effect_verified in [false, true] {
        let mut io = fake(&[(draw(), true), (PredictedAction::NoHighlight, false), (PredictedAction::NoHighlight, false)]);
        io.proven_effect = effect_verified;
        io.completion_candidates = VecDeque::from([true, true]);
        let (result, input_attempted, _) = exercise(&mut io, draw(), 1, &AtomicBool::new(false));
        assert_eq!(result, Ok(RunOutcome::GameWon { previous_verified: usize::from(effect_verified), previous_halo: 0 }));
        assert!(input_attempted);
        assert_eq!(io.effect_checks, 1);
        assert_eq!(io.trace.iter().filter(|entry| **entry == "input").count(), 1);
        assert!(!io.trace.contains(&"solver"));
        assert!(io.terminal_inputs.is_empty());
    }
}


/// Continuous Solve proceeds once through all five terminal stages and reuses the fresh new board.
#[test]
fn continuous_solve_advances_one_board_terminal_cycle_then_obeys_stop() {
    use klondike::terminal::TerminalStage;
    let solve = PredictedAction::Action(klondike::canonical_action(KlondikeTarget::Solve).unwrap());
    let stop = AtomicBool::new(false);
    let mut states = vec![(solve, true)];
    states.extend([(PredictedAction::NoHighlight, false); 12]);
    states.push((draw(), true));
    let mut io = fake(&states);
    io.completion_candidates = VecDeque::from([false, true, false, false, false, false, true, true]);
    let stages = [TerminalStage::ScoreCounting, TerminalStage::LevelUp, TerminalStage::NewGame, TerminalStage::Play, TerminalStage::SolverReady];
    io.terminal_stages = stages.into_iter().map(Some).collect();
    io.stop_on_probe = Some(&stop);
    io.probes_before_stop = 6;
    let (result, input_attempted, latest) = exercise(&mut io, solve, 0, &stop);
    assert!(result.unwrap_err().contains("STOP"));
    assert!(input_attempted);
    assert_eq!(io.terminal_inputs, stages);
    assert_eq!(io.trace.iter().filter(|entry| **entry == "input").count(), 1);
    assert_eq!(io.trace.iter().filter(|entry| **entry == "capture").count(), 14);
    assert_eq!(io.trace.iter().filter(|entry| **entry == "probe").count(), 7);
    assert_eq!(latest.unwrap().prediction, draw());
    assert!(!io.trace.contains(&"solver"));
    assert!(io.terminal_stages.is_empty());
}


/// Unchanged terminal results are observed input-free and never clicked again.
#[test]
fn terminal_same_stage_is_not_retried() {
    use klondike::terminal::TerminalStage;
    let mut io = fake(&[(PredictedAction::NoHighlight, false); REOBSERVATION_LIMIT + 1]);
    io.terminal_stages = VecDeque::from([Some(TerminalStage::ScoreCounting); REOBSERVATION_LIMIT + 2]);
    let (result, input_attempted, latest) = exercise_terminal(&mut io, &AtomicBool::new(false));
    assert!(result.unwrap_err().contains("advancement exhausted"));
    assert!(input_attempted);
    assert_eq!(io.terminal_inputs, [TerminalStage::ScoreCounting]);
    assert_eq!(io.trace.iter().filter(|entry| **entry == "capture").count(), REOBSERVATION_LIMIT + 1);
    assert!(latest.is_some());
}


/// An out-of-sequence recognised control cannot authorise a later click.
#[test]
fn terminal_unexpected_stage_stops_before_input() {
    use klondike::terminal::TerminalStage;
    let mut io = fake(&[(PredictedAction::NoHighlight, false)]);
    io.terminal_stages = VecDeque::from([Some(TerminalStage::ScoreCounting), Some(TerminalStage::Play)]);
    let (result, _, _) = exercise_terminal(&mut io, &AtomicBool::new(false));
    assert!(result.unwrap_err().contains("unexpected"));
    assert_eq!(io.terminal_inputs, [TerminalStage::ScoreCounting]);
    assert_eq!(io.trace, ["probe", "terminal", "wait", "capture"]);
}


/// An uncertain terminal click stops before result capture or any replay.
#[test]
fn uncertain_terminal_input_is_never_retried() {
    use klondike::terminal::TerminalStage;
    let mut io = fake(&[]);
    io.terminal_stages = VecDeque::from([Some(TerminalStage::ScoreCounting)]);
    io.fail_input = true;
    let (result, input_attempted, latest) = exercise_terminal(&mut io, &AtomicBool::new(false));
    assert!(result.unwrap_err().contains("uncertain terminal"));
    assert!(input_attempted);
    assert_eq!(io.trace, ["probe", "terminal"]);
    assert_eq!(io.terminal_inputs, [TerminalStage::ScoreCounting]);
    assert!(latest.is_some());
}


/// STOP at a terminal probe revokes control authority before input.
#[test]
fn stop_before_terminal_input_sends_nothing() {
    use klondike::terminal::TerminalStage;
    let stop = AtomicBool::new(false);
    let mut io = fake(&[]);
    io.terminal_stages = VecDeque::from([Some(TerminalStage::ScoreCounting)]);
    io.stop_on_probe = Some(&stop);
    let (result, input_attempted, _) = exercise_terminal(&mut io, &stop);
    assert!(result.unwrap_err().contains("STOP"));
    assert!(!input_attempted);
    assert_eq!(io.trace, ["probe"]);
    assert!(io.terminal_inputs.is_empty());
}


/// STOP during a terminal settle preserves the last frame and forbids another input.
#[test]
fn stop_during_terminal_settle_never_replays_the_control() {
    use klondike::terminal::TerminalStage;
    let stop = AtomicBool::new(false);
    let mut io = fake(&[]);
    io.terminal_stages = VecDeque::from([Some(TerminalStage::ScoreCounting)]);
    io.stop_on_wait = Some(&stop);
    let (result, input_attempted, latest) = exercise_terminal(&mut io, &stop);
    assert!(result.unwrap_err().contains("STOP"));
    assert!(input_attempted);
    assert_eq!(io.trace, ["probe", "terminal", "wait"]);
    assert_eq!(io.terminal_inputs, [TerminalStage::ScoreCounting]);
    assert!(latest.is_some());
}


/// A late STOP retains newly decoded terminal pixels while revoking their input authority.
#[test]
fn terminal_late_stop_retains_the_fresh_result() {
    use klondike::terminal::TerminalStage;
    let stop = AtomicBool::new(false);
    let mut io = fake(&[(PredictedAction::NoHighlight, false)]);
    io.frames.front_mut().unwrap().frame.pixels[0] = 213;
    io.terminal_stages = VecDeque::from([Some(TerminalStage::ScoreCounting)]);
    io.stop_on_capture = Some(&stop);
    let (result, input_attempted, latest) = exercise_terminal(&mut io, &stop);
    assert!(result.unwrap_err().contains("STOP"));
    assert!(input_attempted);
    assert_eq!(io.trace, ["probe", "terminal", "wait", "capture"]);
    assert_eq!(latest.unwrap().frame.pixels[0], 213);
    assert_eq!(io.terminal_inputs, [TerminalStage::ScoreCounting]);
}


/// An approved ordinary target drifting to a win cannot authorise terminal inputs in continuous mode.
#[test]
fn initial_changed_target_to_confirmed_win_cannot_restart_continuous_mode() {
    let right = PredictedAction::Action(klondike::canonical_action(KlondikeTarget::Waste { offset: 1 }).unwrap());


    for approved in [draw(), right] {
        let mut io = fake(&[(PredictedAction::NoHighlight, false), (PredictedAction::NoHighlight, false)]);
        io.completion_candidates = VecDeque::from([true, true]);
        let (result, input_attempted, latest) = exercise(&mut io, approved, 0, &AtomicBool::new(false));
        assert_eq!(result, Ok(RunOutcome::GameWon { previous_verified: 0, previous_halo: 0 }));
        assert_eq!(io.trace, ["capture", "wait", "capture"]);
        assert!(!input_attempted);
        assert!(io.terminal_inputs.is_empty());
        assert_eq!(latest.unwrap().prediction, PredictedAction::NoHighlight);
    }
}


/// A changed card may retain the same canonical source. Step Once consumes its
/// one budget slot and reports continuation without asserting destination proof.
#[test]
fn changed_card_with_same_halo_consumes_one_step_and_reports_unverified_continuation() {
    let right = PredictedAction::Action(klondike::canonical_action(KlondikeTarget::Waste { offset: 1 }).unwrap());
    let mut io = fake(&[(right, true), (right, true)]);
    io.proven_source_replacement = true;
    let (events, receiver) = std::sync::mpsc::channel();
    let sink = WorkerEventSink {
        events,
        latest_frame: super::super::LatestFrameSlot::default(),
        capture_context: std::sync::Mutex::new(None),
    };
    let mut latest = None;
    let mut input_attempted = false;
    let result = drive_run(
        &mut io, right, StepRunSettings::new(AnimationSettleDelays::default(), 1),
        &sink, &AtomicBool::new(false), &mut latest, &mut input_attempted,
    );
    assert_eq!(result, Ok(RunOutcome::Completed { verified: 0, halo: 1 }));
    assert!(input_attempted);
    assert_eq!(io.inputs, [right]);
    assert_eq!(io.trace, ["capture", "probe", "input", "wait", "capture"]);
    assert_eq!(latest.unwrap().prediction, right);
    let accepted = receiver.try_iter().filter(|event| matches!(event, WorkerEvent::ActionCompleted { .. })).collect::<Vec<_>>();
    assert_eq!(accepted.len(), 1);
    assert!(matches!(accepted[0], WorkerEvent::ActionCompleted {
        operation_index: 1, operation_limit: 1, before, after, continued_from_halo: true, ..
    } if before == right && after == right));
    assert!(io.terminal_inputs.is_empty());
}


/// Every source class owns its replacement proof; two accepted HALOs consume
/// exactly two actions even when neither prior destination is independently proven.
#[test]
fn changed_card_sources_consume_finite_multi_step_budget() {


    for target in [
        KlondikeTarget::Waste { offset: 1 },
        KlondikeTarget::Tableau { column: 5, top: 571, bottom: 761 },
        KlondikeTarget::Foundation { column: 3 },
    ] {
        let source = PredictedAction::Action(klondike::canonical_action(target).unwrap());
        let mut io = fake(&[(source, true), (source, true), (source, true)]);
        io.proven_source_replacement = true;
        let (result, _, latest) = exercise(&mut io, source, 2, &AtomicBool::new(false));
        assert_eq!(result, Ok(RunOutcome::Completed { verified: 0, halo: 2 }));
        assert_eq!(io.inputs, [source, source]);
        assert_eq!(io.trace.iter().filter(|entry| **entry == "capture").count(), 3);
        assert_eq!(io.trace.iter().filter(|entry| **entry == "probe").count(), 2);
        assert_eq!(latest.unwrap().prediction, source);
        assert!(io.terminal_inputs.is_empty());
    }
}


/// A continuation followed by a proven effect has two budget actions but only
/// one verified action; accepting HALOs must not inflate the verified counter.
#[test]
fn mixed_verified_and_halo_actions_keep_independent_totals() {
    let first = PredictedAction::Action(klondike::canonical_action(KlondikeTarget::Waste { offset: 1 }).unwrap());
    let next = PredictedAction::Action(klondike::canonical_action(KlondikeTarget::Waste { offset: 0 }).unwrap());
    let mut io = fake(&[(first, true), (next, true), (draw(), true)]);
    io.effect_verdicts = VecDeque::from([
        EffectVerdict { verified: false, source_replaced: true },
        EffectVerdict { verified: true, source_replaced: true },
    ]);
    let (result, _, latest) = exercise(&mut io, first, 2, &AtomicBool::new(false));
    assert_eq!(result, Ok(RunOutcome::Completed { verified: 1, halo: 1 }));
    assert_eq!(io.inputs, [first, next]);
    assert_eq!(latest.unwrap().prediction, draw());
}


/// A repeated HALO with identical source content remains unresolved; its one
/// acknowledged input is observed with the finite retry budget and never replayed.
#[test]
fn unchanged_card_with_repeated_halo_cannot_continue() {
    let right = PredictedAction::Action(klondike::canonical_action(KlondikeTarget::Waste { offset: 1 }).unwrap());
    let states = vec![(right, true); REOBSERVATION_LIMIT + 2];
    let mut io = fake(&states);
    let (result, _, latest) = exercise(&mut io, right, 2, &AtomicBool::new(false));
    assert!(result.unwrap_err().contains("previous effect verified=false"));
    assert_eq!(io.inputs, [right]);
    assert_eq!(io.effect_checks, REOBSERVATION_LIMIT + 1);
    assert_eq!(latest.unwrap().prediction, right);
}


/// Fresh replacement authority is unavailable after uncertain input or a STOP
/// at the next probe, even when the adapter would report changed card content.
#[test]
fn continuation_never_bypasses_uncertain_acknowledgement_or_stop() {
    let right = PredictedAction::Action(klondike::canonical_action(KlondikeTarget::Waste { offset: 1 }).unwrap());
    let mut uncertain = fake(&[(right, true)]);
    uncertain.proven_source_replacement = true;
    uncertain.fail_input = true;
    let (result, _, _) = exercise(&mut uncertain, right, 2, &AtomicBool::new(false));
    assert!(result.unwrap_err().contains("uncertain"));
    assert_eq!(uncertain.trace, ["capture", "probe", "input"]);
    assert_eq!(uncertain.inputs, [right]);
    assert_eq!(uncertain.effect_checks, 0);
    let stop = AtomicBool::new(false);
    let mut stopped = fake(&[(right, true), (right, true)]);
    stopped.proven_source_replacement = true;
    stopped.stop_on_probe = Some(&stop);
    stopped.probes_before_stop = 1;
    let (result, _, _) = exercise(&mut stopped, right, 0, &stop);
    assert!(result.unwrap_err().contains("STOP"));
    assert_eq!(stopped.inputs, [right]);
    assert_eq!(stopped.trace, ["capture", "probe", "input", "wait", "capture", "probe"]);
}


/// A fresh independently recognised Solve is a new control, not a replay of
/// unchanged prior card content. Step Once exposes it without clicking it.
#[test]
fn unchanged_card_to_fresh_solve_handoff_finishes_step_once() {
    let right = PredictedAction::Action(klondike::canonical_action(KlondikeTarget::Waste { offset: 1 }).unwrap());
    let solve = PredictedAction::Action(klondike::canonical_action(KlondikeTarget::Solve).unwrap());
    let mut io = fake(&[(right, true), (solve, true)]);
    let (result, _, latest) = exercise(&mut io, right, 1, &AtomicBool::new(false));
    assert_eq!(result, Ok(RunOutcome::Completed { verified: 0, halo: 1 }));
    assert_eq!(io.inputs, [right]);
    assert_eq!(latest.unwrap().prediction, solve);
    assert_eq!(io.effect_checks, 1);
    assert_eq!(io.completion_checks, 0);
    assert!(io.terminal_inputs.is_empty());
    assert!(!io.trace.contains(&"solver"));
}


/// A remaining finite budget or continuous authority may click the new Solve
/// once. Its own delay and bounded completion observations stay independent.
#[test]
fn fresh_solve_handoff_clicks_new_control_once_without_verifying_old_card() {
    let right = PredictedAction::Action(klondike::canonical_action(KlondikeTarget::Waste { offset: 1 }).unwrap());
    let solve = PredictedAction::Action(klondike::canonical_action(KlondikeTarget::Solve).unwrap());


    for limit in [0, 2] {
        let mut states = vec![(right, true), (solve, true)];
        states.extend([(PredictedAction::NoHighlight, false); SOLVE_COMPLETION_REOBSERVATION_LIMIT + 1]);
        let mut io = fake(&states);
        let (result, _, _) = exercise(&mut io, right, limit, &AtomicBool::new(false));
        assert_eq!(result, Ok(RunOutcome::SolveRequested { previous_verified: 0, previous_halo: 1 }));
        assert_eq!(io.inputs, [right, solve]);
        assert_eq!(io.effect_checks, 1);
        assert_eq!(io.waits[0], AnimationSettleDelays::default().klondike_settle);
        assert_eq!(io.waits[1], AnimationSettleDelays::default().klondike_solve);
        assert_eq!(io.completion_checks, SOLVE_COMPLETION_REOBSERVATION_LIMIT + 1);
        assert!(io.terminal_inputs.is_empty());
        assert!(!io.trace.contains(&"solver"));
    }
}


/// Source replacement and new Solve artwork cannot bypass an unknown result
/// scene, or promote a target obtained through intervening Solver recovery.
#[test]
fn continuation_rejects_unknown_scene_and_solver_refreshed_target() {
    let right = PredictedAction::Action(klondike::canonical_action(KlondikeTarget::Waste { offset: 1 }).unwrap());
    let solve = PredictedAction::Action(klondike::canonical_action(KlondikeTarget::Solve).unwrap());
    let mut states = vec![(right, true)];
    states.extend([(solve, false); REOBSERVATION_LIMIT + 1]);
    let mut unknown = fake(&states);
    unknown.proven_source_replacement = true;
    let (result, _, _) = exercise(&mut unknown, right, 2, &AtomicBool::new(false));
    assert!(result.unwrap_err().contains("unsupported Klondike scene"));
    assert_eq!(unknown.inputs, [right]);


    for target in [right, solve] {
        let mut states = vec![(right, true), (PredictedAction::NoHighlight, true)];
        states.extend([(target, true); REOBSERVATION_LIMIT + 1]);
        let mut recovered = fake(&states);
        recovered.proven_source_replacement = true;
        let (result, _, _) = exercise(&mut recovered, right, 2, &AtomicBool::new(false));
        assert!(result.unwrap_err().contains("previous effect verified=false"));
        assert_eq!(recovered.inputs, [right]);
        assert_eq!(recovered.trace.iter().filter(|entry| **entry == "solver").count(), 1);
    }
}


/// Draw and recycle retain their dedicated effect proof even if a fake source
/// verdict or independently recognised Solve is supplied in the result frame.
#[test]
fn draw_and_recycle_sources_cannot_use_fresh_card_continuation() {
    let recycle = PredictedAction::Action(klondike::canonical_action(KlondikeTarget::Recycle).unwrap());
    let solve = PredictedAction::Action(klondike::canonical_action(KlondikeTarget::Solve).unwrap());


    for previous in [draw(), recycle] {


        for fresh in [previous, solve] {
            let mut states = vec![(previous, true)];
            states.extend([(fresh, true); REOBSERVATION_LIMIT + 1]);
            let mut io = fake(&states);
            io.proven_source_replacement = true;
            let (result, _, _) = exercise(&mut io, previous, 2, &AtomicBool::new(false));
            assert!(result.unwrap_err().contains("previous effect verified=false"));
            assert_eq!(io.inputs, [previous]);
            assert!(!io.trace.contains(&"solver"));
        }
    }
}


/// A fabricated result action must fail canonical validation before source
/// replacement or an apparent new control can supply continuation authority.
#[test]
fn continuation_rejects_noncanonical_result_action() {
    let right = PredictedAction::Action(klondike::canonical_action(KlondikeTarget::Waste { offset: 1 }).unwrap());
    let mut altered = klondike::canonical_action(KlondikeTarget::Solve).unwrap();
    altered.anchor.x += 1;
    let mut io = fake(&[(right, true), (PredictedAction::Action(altered), true)]);
    io.proven_source_replacement = true;
    let (result, _, latest) = exercise(&mut io, right, 2, &AtomicBool::new(false));
    assert!(result.unwrap_err().contains("Klondike target rejected"));
    assert_eq!(io.inputs, [right]);
    assert_eq!(io.effect_checks, 0);
    assert_eq!(latest.unwrap().prediction, PredictedAction::Action(altered));
}


/// A fresh-HALO continuation contributes no verified effect or win authority;
/// a later win still needs two separate positive completion observations.
#[test]
fn halo_counts_remain_separate_from_independently_confirmed_win() {
    let right = PredictedAction::Action(klondike::canonical_action(KlondikeTarget::Waste { offset: 1 }).unwrap());
    let mut io = fake(&[
        (right, true), (right, true),
        (PredictedAction::NoHighlight, false), (PredictedAction::NoHighlight, false),
    ]);
    io.effect_verdicts = VecDeque::from([
        EffectVerdict { verified: false, source_replaced: true },
        EffectVerdict { verified: false, source_replaced: false },
    ]);
    io.completion_candidates = VecDeque::from([true, true]);
    let (result, _, _) = exercise(&mut io, right, 2, &AtomicBool::new(false));
    assert_eq!(result, Ok(RunOutcome::GameWon { previous_verified: 0, previous_halo: 1 }));
    assert_eq!(io.inputs, [right, right]);
    assert_eq!(io.completion_checks, 2);
    assert!(io.terminal_inputs.is_empty());
}
