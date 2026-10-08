//! Control-flow regressions run against the production Klondike controller.
//!
//! The fake adapter supplies fresh observations and records one-shot inputs so
//! cancellation, uncertainty and recovery budgets can be injected deterministically.
//! Native fixtures use the production scene/HALO classifier. Independent detector
//! effect diagnostics are deliberately outside the controller input boundary.

use std::{collections::VecDeque, sync::atomic::AtomicBool};

use super::*;
use crate::{capture::PixelFormat, game::InputOperation, geometry::PixelPoint, klondike::KlondikeTarget, parameters::AnimationSettleDelays};


/// Deterministic adapter recording order while independently injecting I/O failures.
struct FakeIo<'a> {
    /// Fresh captures supplied in order; exhaustion indicates an unexpected recapture.
    frames: VecDeque<FrameObservation>,
    /// Ordered I/O trace, used to verify no delay between Solver and the next capture.
    trace: Vec<&'static str>,
    /// Use real terminal/completion pixel classification for full restart evidence.
    native_terminal_analysis: bool,
    /// Acknowledged or attempted gameplay targets, distinguishing a new action from replay.
    inputs: Vec<PredictedAction>,
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
    /// Fail one specified terminal stage without failing earlier input.
    fail_terminal_stage: Option<klondike::terminal::TerminalStage>,
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
    /// Permit this many captures before a capture publishes diagnostic STOP pixels.
    captures_before_stop: usize,
    /// Set STOP after the diagnostic pixels are acquired but before the controller receives them.
    stop_on_diagnostic_capture: Option<&'a AtomicBool>,
}


impl KlondikeIo for FakeIo<'_> {


    fn capture(&mut self) -> Result<CaptureResult, String> {
        self.trace.push("capture");
        let observation = self.frames.pop_front().ok_or_else(|| "unexpected capture".to_owned())?;


        if let Some(stop) = self.stop_on_capture
            && self.trace.iter().filter(|entry| **entry == "capture").count() > self.captures_before_stop
        {
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


    fn completion(&mut self, frame: &CapturedFrame) -> Result<bool, String> {
        self.completion_checks += 1;


        if self.native_terminal_analysis {
            return klondike::completion_evidence(frame)
                .map(|evidence| evidence.complete_candidate)
                .map_err(|error| format!("native test completion analysis failed: {error}"));
        }
        Ok(self.completion_candidates.pop_front().unwrap_or(false))
    }


    fn terminal_stage(&mut self, frame: &CapturedFrame) -> Result<Option<klondike::terminal::TerminalStage>, String> {


        if self.native_terminal_analysis {
            return klondike::terminal::classify_confirmed_win_entry(frame)
                .map_err(|error| format!("native test terminal analysis failed: {error}"));
        }
        Ok(self.terminal_stages.pop_front().unwrap_or(None))
    }


    fn expected_terminal_control(&mut self, frame: &CapturedFrame, stage: klondike::terminal::TerminalStage) -> Result<bool, String> {


        if self.native_terminal_analysis {
            return klondike::terminal::expected_control_ready(frame, stage)
                .map_err(|error| format!("native test expected control analysis failed: {error}"));
        }
        Ok(self.terminal_stages.pop_front().flatten() == Some(stage))
    }


    fn terminal_input(&mut self, stage: klondike::terminal::TerminalStage) -> Result<(), String> {
        self.trace.push("terminal");
        self.terminal_inputs.push(stage);


        if self.fail_input || self.fail_terminal_stage == Some(stage) {
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
        native_terminal_analysis: false,
        inputs: Vec::new(),
        completion_candidates: VecDeque::new(),
        completion_checks: 0,
        waits: Vec::new(),
        terminal_stages: VecDeque::new(),
        terminal_inputs: Vec::new(),
        capture_failure: None,
        fail_input: false,
        fail_terminal_stage: None,
        fail_solver: false,
        stop_on_probe: None,
        probes_before_stop: 0,
        stop_on_wait: None,
        waits_before_stop: 0,
        stop_on_capture: None,
        captures_before_stop: 0,
        stop_on_diagnostic_capture: None,
    }
}


/// Preserve native K48 pixels while withdrawing 24 stable non-glyph samples.
/// The unretained early worker pixels are unknown; this controlled derivative
/// reproduces only the logged 459/483 face and 342/342 glyph counts. Production
/// scene and target classification supplies both observations used by the worker.
fn pending_solve_pair() -> (FrameObservation, FrameObservation) {
    let bytes = include_bytes!("../../../tests/fixtures/klondike/K48.png");
    let settled = crate::capture::decode_png(bytes).expect("decode native Solve fixture");
    let mut pending = settled.clone();
    let mut changed = 0;


    for y in (143..256).step_by(4) {


        for x in (562..686).step_by(4) {


            if changed < 24 && (578..670).contains(&x) && (159..240).contains(&y)
                && !((590..660).contains(&x) && (160..236).contains(&y))
            {
                let offset = y as usize * pending.stride + x as usize * 4;
                pending.pixels[offset..offset + 3].copy_from_slice(&[255, 255, 255]);
                changed += 1;
            }
        }
    }
    assert_eq!(changed, 24);
    let evidence = klondike::inspect_solve_control(&pending).unwrap();
    assert_eq!((evidence.interior_matched, evidence.glyph_matched), (459, 342));
    assert!(evidence.awaiting_settle() && !evidence.available);
    (native_observation(pending), native_observation(settled))
}


/// Classify unchanged native geometry without replacing pixel guards with a mock.
fn native_observation(frame: CapturedFrame) -> FrameObservation {
    FrameObservation {
        prediction: klondike::analyse(&frame).expect("classify evidence pixels").prediction,
        gameplay_scene: klondike::is_gameplay_scene(&frame).expect("validate evidence scene"),
        frame,
        observed_rows: None,
        game_progress: None,
    }
}


/// Duplicate a test observation without changing shared worker ownership contracts.
fn copy_observation(source: &FrameObservation) -> FrameObservation {
    FrameObservation {
        frame: source.frame.clone(),
        prediction: source.prediction,
        observed_rows: source.observed_rows,
        gameplay_scene: source.gameplay_scene,
        game_progress: source.game_progress,
    }
}


/// Remove only the pending button to model a disappearing candidate on a fresh frame.
/// This is an explicitly controlled derivative, not a captured live transition.
fn absent_solve_observation(pending: &FrameObservation) -> FrameObservation {
    let mut frame = pending.frame.clone();


    for y in 143..256 {


        for x in 562..686 {
            let offset = y * frame.stride + x * 4;
            frame.pixels[offset..offset + 3].copy_from_slice(&[12, 82, 45]);
        }
    }
    assert!(!klondike::inspect_solve_control(&frame).unwrap().awaiting_settle());
    native_observation(frame)
}


/// Supply already classified full frames without inventing earlier worker captures.
fn fake_frames<'a>(frames: Vec<FrameObservation>) -> FakeIo<'a> {
    let mut io = fake(&[]);
    io.frames = frames.into();
    io
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



/// Every logical source/stock class uses the same fresh-capture action budget.
fn canonical(target: KlondikeTarget) -> PredictedAction {
    PredictedAction::Action(klondike::canonical_action(target).expect("evidenced canonical target"))
}


/// Supply all bounded no-HALO observations before and after one Solver refresh.
fn unresolved_context() -> Vec<(PredictedAction, bool)> {
    vec![(PredictedAction::NoHighlight, true); 2 * (REOBSERVATION_LIMIT + 1)]
}


/// The preview is advisory; the new valid source is the only input authority.
#[test]
fn changed_advisory_preview_uses_the_fresh_canonical_source() {
    let fresh = canonical(KlondikeTarget::Waste { offset: 2 });


    for approved in [PredictedAction::NoHighlight, draw(), canonical(KlondikeTarget::Foundation { column: 2 })] {
        let mut io = fake(&[(fresh, true), (draw(), true)]);
        let (result, attempted, latest) = exercise(&mut io, approved, 1, &AtomicBool::new(false));
        assert_eq!(result, Ok(RunOutcome::Completed { verified: 0, halo: 1 }));
        assert!(attempted);
        assert_eq!(io.inputs, [fresh]);
        assert_eq!(io.trace, ["capture", "probe", "input", "wait", "capture"]);
        assert_eq!(latest.unwrap().prediction, draw());
    }
}


/// A one-row native HALO phase change cannot revive the stale preview coordinates.
#[test]
fn native_one_row_preview_change_uses_the_fresh_source_geometry() {
    let thick = native_hint_phase(74);
    let thin = native_hint_phase(75);
    assert_ne!(thick.prediction, thin.prediction);
    let approved = thick.prediction;
    let expected = thin.prediction;
    let mut io = fake_frames(vec![copy_observation(&thin), thin]);
    let (result, attempted, _) = exercise(&mut io, approved, 1, &AtomicBool::new(false));
    assert_eq!(result, Ok(RunOutcome::Completed { verified: 0, halo: 1 }));
    assert!(attempted);
    assert_eq!(io.inputs, [expected]);
    assert_eq!(io.trace, ["capture", "probe", "input", "wait", "capture"]);
}


/// Fresh source classes, including repeated DRAW and Recycle, consume new actions.
#[test]
fn fresh_repeated_halos_count_actions_without_card_or_move_matching() {


    for target in [
        KlondikeTarget::Draw, KlondikeTarget::Recycle, KlondikeTarget::Waste { offset: 1 },
        KlondikeTarget::Foundation { column: 2 },
        KlondikeTarget::Tableau { column: 4, top: 808, bottom: 947 },
    ] {
        let predicted = canonical(target);
        let mut io = fake(&[(predicted, true); 3]);
        let (result, attempted, _) = exercise(&mut io, predicted, 2, &AtomicBool::new(false));
        assert_eq!(result, Ok(RunOutcome::Completed { verified: 0, halo: 2 }));
        assert!(attempted);
        assert_eq!(io.inputs, [predicted, predicted]);
        assert_eq!(io.trace, ["capture", "probe", "input", "wait", "capture", "probe", "input", "wait", "capture"]);
        assert!(io.terminal_inputs.is_empty());
    }
}


/// A fresh different source does not require any before/result artwork correlation.
#[test]
fn fresh_different_halos_reuse_each_result_frame_and_probe_each_action() {
    let first = canonical(KlondikeTarget::Foundation { column: 2 });
    let second = canonical(KlondikeTarget::Waste { offset: 2 });
    let third = canonical(KlondikeTarget::Recycle);
    let mut io = fake(&[(first, true), (second, true), (third, true)]);
    let (result, _, latest) = exercise(&mut io, draw(), 2, &AtomicBool::new(false));
    assert_eq!(result, Ok(RunOutcome::Completed { verified: 0, halo: 2 }));
    assert_eq!(io.inputs, [first, second]);
    assert_eq!(latest.unwrap().prediction, third);
    assert_eq!(io.trace.iter().filter(|entry| **entry == "capture").count(), 3);
    assert_eq!(io.trace.iter().filter(|entry| **entry == "probe").count(), 2);
    assert!(!io.trace.contains(&"solver"));
}


/// Step Once ends after its fresh result; no missing-HALO recovery adds more input.
#[test]
fn finite_budget_stops_after_one_fresh_result_even_without_a_supported_target() {


    for (result_target, scene) in [(draw(), true), (PredictedAction::NoHighlight, true), (PredictedAction::NoHighlight, false)] {
        let mut io = fake(&[(draw(), true), (result_target, scene)]);
        let delays = AnimationSettleDelays::default().with_klondike_millis(750, 250);
        let (result, attempted, latest) = exercise_with_settings(
            &mut io, draw(), StepRunSettings::new(delays, 1), &AtomicBool::new(false),
        );
        assert_eq!(result, Ok(RunOutcome::Completed { verified: 0, halo: 1 }));
        assert!(attempted);
        assert_eq!(io.inputs, [draw()]);
        assert_eq!(io.trace, ["capture", "probe", "input", "wait", "capture"]);
        assert_eq!(io.waits, [delays.klondike_settle]);
        assert_eq!(latest.unwrap().prediction, result_target);
        assert!(io.terminal_inputs.is_empty());
    }
}


/// Fresh-HALO reporting counts the sent action and never claims a proven transfer.
#[test]
fn halo_action_events_keep_verified_totals_zero() {
    let right = canonical(KlondikeTarget::Waste { offset: 1 });
    let mut io = fake(&[(right, true), (right, true)]);
    let (events, receiver) = std::sync::mpsc::channel();
    let sink = WorkerEventSink {
        events,
        latest_frame: super::super::LatestFrameSlot::default(),
        capture_context: std::sync::Mutex::new(None),
    };
    let mut latest = None;
    let mut attempted = false;
    let result = drive_run(&mut io, draw(), StepRunSettings::new(AnimationSettleDelays::default(), 1),
        &sink, &AtomicBool::new(false), &mut latest, &mut attempted);
    assert_eq!(result, Ok(RunOutcome::Completed { verified: 0, halo: 1 }));
    let accepted: Vec<_> = receiver.try_iter().map(|notification| notification.event)
        .filter(|event| matches!(event, WorkerEvent::ActionCompleted { .. })).collect();
    assert_eq!(accepted.len(), 1);
    assert!(matches!(accepted[0], WorkerEvent::ActionCompleted {
        operation_index: 1, operation_limit: 1, before, after, continued_from_halo: true, ..
    } if before == right && after == right));
    assert!(attempted && io.terminal_inputs.is_empty());
}


/// Zero removes the action budget but STOP is still probed before every fresh action.
#[test]
fn continuous_repeated_halo_stops_at_the_next_fresh_probe() {
    let stop = AtomicBool::new(false);
    let mut io = fake(&[(draw(), true); 3]);
    io.stop_on_probe = Some(&stop);
    io.probes_before_stop = 2;
    let (result, attempted, latest) = exercise(&mut io, draw(), 0, &stop);
    assert!(result.unwrap_err().contains("STOP"));
    assert!(attempted);
    assert_eq!(io.inputs, [draw(), draw()]);
    assert_eq!(io.trace, ["capture", "probe", "input", "wait", "capture", "probe", "input", "wait", "capture", "probe"]);
    assert_eq!(latest.unwrap().prediction, draw());
}


/// No HALO gets bounded read-only captures before the one supported-scene Solver click.
#[test]
fn missing_halo_context_has_finite_observations_and_one_solver_refresh() {


    for limit in [0, 1] {
        let mut io = fake(&unresolved_context());
        let (result, attempted, latest) = exercise(&mut io, PredictedAction::NoHighlight, limit, &AtomicBool::new(false));
        assert!(result.is_err());
        assert!(attempted);
        assert!(io.inputs.is_empty() && io.terminal_inputs.is_empty());
        assert_eq!(io.trace.iter().filter(|entry| **entry == "solver").count(), 1);
        assert_eq!(io.trace.iter().filter(|entry| **entry == "capture").count(), 2 * (REOBSERVATION_LIMIT + 1));
        let first_solver = io.trace.iter().position(|entry| *entry == "solver").unwrap();
        assert_eq!(io.trace[..first_solver].iter().filter(|entry| **entry == "capture").count(), REOBSERVATION_LIMIT + 1);
        assert_eq!(io.trace[first_solver + 1], "capture", "Solver has no speculative post-click sleep");
        assert_eq!(latest.unwrap().prediction, PredictedAction::NoHighlight);
    }
}


/// A delayed HALO appearing before recovery requires no Solver click.
#[test]
fn missing_halo_recapture_can_find_a_target_before_solver_activation() {
    let mut io = fake(&[(PredictedAction::NoHighlight, true), (draw(), true), (draw(), true)]);
    let (result, attempted, _) = exercise(&mut io, PredictedAction::NoHighlight, 1, &AtomicBool::new(false));
    assert_eq!(result, Ok(RunOutcome::Completed { verified: 0, halo: 1 }));
    assert!(attempted);
    assert_eq!(io.inputs, [draw()]);
    assert_eq!(io.trace, ["capture", "wait", "capture", "probe", "input", "wait", "capture"]);
    assert!(!io.trace.contains(&"solver"));
}


/// An initial Step Once may activate Solver, observe its fresh recommendation and send one action.
#[test]
fn initial_no_halo_step_once_plays_one_fresh_recovered_target() {
    let mut states = vec![(PredictedAction::NoHighlight, true); REOBSERVATION_LIMIT + 1];
    states.extend([(draw(), true), (draw(), true)]);
    let mut io = fake(&states);
    let (result, attempted, latest) = exercise(&mut io, PredictedAction::NoHighlight, 1, &AtomicBool::new(false));
    assert_eq!(result, Ok(RunOutcome::Completed { verified: 0, halo: 1 }));
    assert!(attempted);
    assert_eq!(io.inputs, [draw()]);
    assert_eq!(io.trace.iter().filter(|entry| **entry == "solver").count(), 1);
    assert_eq!(io.trace.iter().filter(|entry| **entry == "capture").count(), REOBSERVATION_LIMIT + 3);
    assert_eq!(latest.unwrap().prediction, draw());
}


/// The one Solver refresh remains followed by bounded fresh delayed observations.
#[test]
fn late_recovered_target_uses_editable_input_free_waits_before_gameplay() {
    let mut states = vec![(PredictedAction::NoHighlight, true); REOBSERVATION_LIMIT + 1];
    states.extend([(PredictedAction::NoHighlight, true), (PredictedAction::NoHighlight, true), (draw(), true), (draw(), true)]);
    let mut io = fake(&states);
    let delays = AnimationSettleDelays::default().with_klondike_millis(750, 250);
    let (result, _, _) = exercise_with_settings(&mut io, draw(), StepRunSettings::new(delays, 1), &AtomicBool::new(false));
    assert_eq!(result, Ok(RunOutcome::Completed { verified: 0, halo: 1 }));
    assert_eq!(io.inputs, [draw()]);
    assert_eq!(io.waits.last(), Some(&delays.klondike_settle));
    assert!(io.waits[..io.waits.len() - 1].iter().all(|delay| *delay == delays.klondike_reobserve));
    assert_eq!(io.trace.iter().filter(|entry| **entry == "solver").count(), 1);
}


/// Unsupported pixels cannot grant Solver or gameplay input after the read-only budget.
#[test]
fn unknown_scenes_never_authorise_solver_even_with_an_injected_target() {


    for prediction in [PredictedAction::NoHighlight, draw()] {
        let mut io = fake(&[(prediction, false); REOBSERVATION_LIMIT + 1]);
        let (result, attempted, latest) = exercise(&mut io, draw(), 0, &AtomicBool::new(false));
        assert!(result.is_err());
        assert!(!attempted && io.inputs.is_empty() && io.terminal_inputs.is_empty());
        assert!(!io.trace.contains(&"solver") && !io.trace.contains(&"probe"));
        assert_eq!(io.trace.iter().filter(|entry| **entry == "capture").count(), REOBSERVATION_LIMIT + 1);
        assert!(latest.is_some());
    }
}


/// An ambiguous or forged target never becomes a logical action at initial or result planning.
#[test]
fn initial_and_result_targets_must_be_unique_and_canonical() {
    let mut forged = klondike::canonical_action(KlondikeTarget::Solve).unwrap();
    forged.anchor.x += 1;


    for rejected in [PredictedAction::Ambiguous { highlight_count: 2 }, PredictedAction::Action(forged)] {
        let mut initial = fake(&[(rejected, true)]);
        let (result, attempted, _) = exercise(&mut initial, draw(), 0, &AtomicBool::new(false));
        assert!(result.is_err());
        assert!(!attempted && initial.inputs.is_empty() && !initial.trace.contains(&"solver"));
        let mut result_io = fake(&[(draw(), true), (rejected, true)]);
        let (result, attempted, latest) = exercise(&mut result_io, draw(), 2, &AtomicBool::new(false));
        assert!(result.is_err());
        assert!(attempted);
        assert_eq!(result_io.inputs, [draw()]);
        assert_eq!(latest.unwrap().prediction, rejected);
        assert!(!result_io.trace.contains(&"solver"));
    }
}


/// Unknown result scenes receive read-only observations while a fresh later HALO can resume.
#[test]
fn unsupported_result_recapture_can_resume_from_a_supported_fresh_halo() {
    let mut io = fake(&[(draw(), true), (PredictedAction::NoHighlight, false), (draw(), true), (draw(), true)]);
    let (result, attempted, latest) = exercise(&mut io, draw(), 2, &AtomicBool::new(false));
    assert_eq!(result, Ok(RunOutcome::Completed { verified: 0, halo: 2 }));
    assert!(attempted);
    assert_eq!(io.inputs, [draw(), draw()]);
    assert_eq!(latest.unwrap().prediction, draw());
    assert!(!io.trace.contains(&"solver"));
    assert_eq!(io.trace.iter().filter(|entry| **entry == "capture").count(), 4);
}


/// Persistent unsupported result pixels stop without speculative Solver or terminal control.
#[test]
fn unsupported_result_exhausts_input_free_observations_without_replaying_gameplay() {
    let mut states = vec![(draw(), true)];
    states.extend([(PredictedAction::NoHighlight, false); REOBSERVATION_LIMIT + 1]);
    let mut io = fake(&states);
    let (result, attempted, _) = exercise(&mut io, draw(), 2, &AtomicBool::new(false));
    assert!(result.is_err());
    assert!(attempted);
    assert_eq!(io.inputs, [draw()]);
    assert!(!io.trace.contains(&"solver") && io.terminal_inputs.is_empty());
    assert_eq!(io.trace.iter().filter(|entry| **entry == "capture").count(), REOBSERVATION_LIMIT + 2);
}


/// A recovered target authorises one next logical action; its context can refresh only once.
#[test]
fn continuous_missing_halo_recovery_resets_after_a_new_acknowledged_action() {
    let stop = AtomicBool::new(false);
    let mut states = vec![(draw(), true)];
    states.extend([(PredictedAction::NoHighlight, true); REOBSERVATION_LIMIT + 1]);
    states.push((draw(), true));
    states.extend([(PredictedAction::NoHighlight, true); REOBSERVATION_LIMIT + 1]);
    states.push((draw(), true));
    let mut io = fake(&states);
    io.stop_on_probe = Some(&stop);
    io.probes_before_stop = 4;
    let (result, _, _) = exercise(&mut io, draw(), 0, &stop);
    assert!(result.unwrap_err().contains("STOP"));
    assert_eq!(io.inputs, [draw(), draw()]);
    assert_eq!(io.trace.iter().filter(|entry| **entry == "solver").count(), 2);
    assert_eq!(io.trace.iter().filter(|entry| **entry == "capture").count(), 2 * REOBSERVATION_LIMIT + 5);
}


/// STOP interrupts no-HALO waits and the final Solver probe before recovery input.
#[test]
fn missing_halo_recovery_obeys_stop_before_solver_or_gameplay_input() {
    let stop = AtomicBool::new(false);
    let mut waiting = fake(&[(PredictedAction::NoHighlight, true)]);
    waiting.stop_on_wait = Some(&stop);
    let (result, attempted, _) = exercise(&mut waiting, draw(), 0, &stop);
    assert!(result.unwrap_err().contains("STOP"));
    assert!(!attempted && waiting.inputs.is_empty());
    assert_eq!(waiting.trace, ["capture", "wait"]);
    let stop = AtomicBool::new(false);
    let mut probing = fake(&[(PredictedAction::NoHighlight, true); REOBSERVATION_LIMIT + 1]);
    probing.stop_on_probe = Some(&stop);
    let (result, attempted, _) = exercise(&mut probing, draw(), 0, &stop);
    assert!(result.unwrap_err().contains("STOP"));
    assert!(!attempted && !probing.trace.contains(&"solver"));
}


/// STOP after one Solver activation still prevents the recovered source input.
#[test]
fn recovered_target_keeps_its_own_final_stop_probe() {
    let stop = AtomicBool::new(false);
    let mut states = vec![(PredictedAction::NoHighlight, true); REOBSERVATION_LIMIT + 1];
    states.push((draw(), true));
    let mut io = fake(&states);
    io.stop_on_probe = Some(&stop);
    io.probes_before_stop = 1;
    let (result, attempted, latest) = exercise(&mut io, draw(), 0, &stop);
    assert!(result.unwrap_err().contains("STOP"));
    assert!(attempted && io.inputs.is_empty());
    assert_eq!(io.trace.iter().filter(|entry| **entry == "solver").count(), 1);
    assert_eq!(latest.unwrap().prediction, draw());
}


/// Uncertain Solver delivery stops without another capture or any retry.
#[test]
fn uncertain_solver_stops_after_one_attempt_without_capture_or_replay() {
    let mut io = fake(&[(PredictedAction::NoHighlight, true); REOBSERVATION_LIMIT + 1]);
    io.fail_solver = true;
    let (result, attempted, _) = exercise(&mut io, draw(), 0, &AtomicBool::new(false));
    assert!(result.unwrap_err().contains("uncertain Solver"));
    assert!(attempted && io.inputs.is_empty());
    assert_eq!(io.trace.iter().filter(|entry| **entry == "solver").count(), 1);
    assert_eq!(io.trace.last(), Some(&"solver"));
    assert!(io.frames.is_empty());
}


/// Ordinary acknowledged actions remain unverified when independent completion follows.
#[test]
fn ordinary_action_can_reach_independent_game_win_without_a_transfer_claim() {
    let mut states = vec![(draw(), true)];
    states.extend([(PredictedAction::NoHighlight, false); REOBSERVATION_LIMIT + 2]);
    let mut io = fake(&states);
    io.completion_candidates = VecDeque::from([true, true]);
    let (result, attempted, _) = exercise(&mut io, draw(), 2, &AtomicBool::new(false));
    assert_eq!(result, Ok(RunOutcome::GameWon { previous_verified: 0, previous_halo: 1 }));
    assert!(attempted);
    assert_eq!(io.inputs, [draw()]);
    assert_eq!(io.completion_checks, 2);
    assert!(io.terminal_inputs.is_empty() && !io.trace.contains(&"solver"));
}


/// Completion has its own consecutive evidence requirement before any Solver recovery.
#[test]
fn initial_confirmed_win_preempts_solver_after_bounded_read_only_observation() {
    let mut io = fake(&[(PredictedAction::NoHighlight, false); REOBSERVATION_LIMIT + 2]);
    io.completion_candidates = VecDeque::from([true, true]);
    let (result, attempted, latest) = exercise(&mut io, draw(), 1, &AtomicBool::new(false));
    assert_eq!(result, Ok(RunOutcome::GameWon { previous_verified: 0, previous_halo: 0 }));
    assert!(!attempted && io.inputs.is_empty() && io.terminal_inputs.is_empty());
    assert!(!io.trace.contains(&"solver"));
    assert_eq!(io.completion_checks, 2);
    assert!(latest.is_some());
}


/// Fresh Solve can replace a stale preview and retains its one-shot completion path.
#[test]
fn fresh_initial_solve_is_authorised_from_the_current_canonical_frame() {
    let solve = canonical(KlondikeTarget::Solve);
    let mut io = fake(&[(solve, true), (PredictedAction::NoHighlight, false), (PredictedAction::NoHighlight, false)]);
    io.completion_candidates = VecDeque::from([true, true]);
    let (result, attempted, _) = exercise(&mut io, draw(), 1, &AtomicBool::new(false));
    assert_eq!(result, Ok(RunOutcome::GameWon { previous_verified: 0, previous_halo: 0 }));
    assert!(attempted);
    assert_eq!(io.inputs, [solve]);
    assert!(io.terminal_inputs.is_empty() && !io.trace.contains(&"solver"));
}


/// A card-to-Solve handoff counts the preceding action without claiming its effect.
#[test]
fn fresh_solve_after_card_reports_unverified_action_before_one_solve_request() {
    let solve = canonical(KlondikeTarget::Solve);
    let mut states = vec![(draw(), true), (solve, true)];
    states.extend([(PredictedAction::NoHighlight, false); SOLVE_COMPLETION_REOBSERVATION_LIMIT + 1]);
    let mut io = fake(&states);
    let (result, _, _) = exercise(&mut io, draw(), 2, &AtomicBool::new(false));
    assert_eq!(result, Ok(RunOutcome::SolveRequested { previous_verified: 0, previous_halo: 1 }));
    assert_eq!(io.inputs, [draw(), solve]);
    assert!(!io.trace.contains(&"solver") && io.terminal_inputs.is_empty());
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
        trace: Vec::new(), native_terminal_analysis: false, inputs: Vec::new(),
        completion_candidates: VecDeque::new(), completion_checks: 0, waits: Vec::new(), terminal_stages: VecDeque::new(), terminal_inputs: Vec::new(), capture_failure: None, fail_input: false, fail_terminal_stage: None, fail_solver: false,
        stop_on_probe: Some(&stop), probes_before_stop: 0, stop_on_wait: None, waits_before_stop: 0,
        stop_on_capture: None,
        captures_before_stop: 0,
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
        trace: Vec::new(), native_terminal_analysis: false, inputs: Vec::new(),
        completion_candidates: VecDeque::new(), completion_checks: 0, waits: Vec::new(), terminal_stages: VecDeque::new(), terminal_inputs: Vec::new(), capture_failure: None, fail_input: false, fail_terminal_stage: None, fail_solver: false,
        stop_on_probe: None, probes_before_stop: 0, stop_on_wait: Some(&stop), waits_before_stop: 0,
        stop_on_capture: None,
        captures_before_stop: 0,
        stop_on_diagnostic_capture: None,
    };
    let (result, input_attempted, latest) = exercise(&mut io, draw(), 2, &stop);
    assert!(result.unwrap_err().contains("STOP"));
    assert_eq!(io.trace, ["capture", "probe", "input", "wait"]);
    assert!(input_attempted);
    assert!(latest.is_some());
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

    assert_eq!(io.trace, ["capture", "probe", "input", "wait", "capture", "wait"]);
    assert!(!latest.unwrap().gameplay_scene);
    assert!(!io.trace.contains(&"solver"));
    assert!(io.terminal_inputs.is_empty());
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

            let (result, input_attempted, latest) = exercise(&mut io, solve, limit, &AtomicBool::new(false));
            assert_eq!(result, Ok(RunOutcome::SolveRequested { previous_verified: 0, previous_halo: 0 }));
            assert_eq!(io.trace.iter().filter(|entry| **entry == "input").count(), 1);
            assert_eq!(io.trace.iter().filter(|entry| **entry == "capture").count(), SOLVE_COMPLETION_REOBSERVATION_LIMIT + 2);
            assert_eq!(io.completion_checks, SOLVE_COMPLETION_REOBSERVATION_LIMIT + 1);

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


/// Uncertain Solve delivery stops immediately and is never replayed or followed by recovery.
#[test]
fn uncertain_solve_input_stops_without_recapture_or_retry() {
    let solve = PredictedAction::Action(klondike::canonical_action(KlondikeTarget::Solve).unwrap());
    let mut io = fake(&[(solve, true)]);
    io.fail_input = true;
    let (result, _, _) = exercise(&mut io, solve, 0, &AtomicBool::new(false));
    assert!(result.unwrap_err().contains("uncertain"));
    assert_eq!(io.trace, ["capture", "probe", "input"]);

}


/// Both continuous diagnostic counters fail before a wrapping increment can authorise input.
#[test]
fn cumulative_counters_reject_overflow() {
    assert_eq!(next_counter(0, "gameplay action").unwrap(), 1);
    assert_eq!(next_counter(usize::MAX - 1, "Solver refresh").unwrap(), usize::MAX);
    assert!(next_counter(usize::MAX, "gameplay action").unwrap_err().contains("before further input"));
    assert!(next_counter(usize::MAX, "Solver refresh").unwrap_err().contains("before further input"));
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
    let mut io = fake(&[(PredictedAction::NoHighlight, false); POST_GAME_MAX_OBSERVATION_ROUNDS + 1]);
    io.terminal_stages = VecDeque::from([Some(TerminalStage::ScoreCounting); POST_GAME_MAX_OBSERVATION_ROUNDS + 2]);
    let (result, input_attempted, latest) = exercise_terminal(&mut io, &AtomicBool::new(false));
    assert!(result.unwrap_err().contains("advancement exhausted"));
    assert!(input_attempted);
    assert_eq!(io.terminal_inputs, [TerminalStage::ScoreCounting]);
    assert_eq!(io.trace.iter().filter(|entry| **entry == "capture").count(), POST_GAME_MAX_OBSERVATION_ROUNDS + 1);
    assert!(latest.is_some());
}


/// An out-of-sequence recognised control cannot authorise a later click.
#[test]
fn terminal_unexpected_stage_stops_before_input() {
    use klondike::terminal::TerminalStage;
    let mut io = fake(&[(PredictedAction::NoHighlight, false); POST_GAME_MAX_OBSERVATION_ROUNDS + 1]);
    io.terminal_stages = VecDeque::from([Some(TerminalStage::ScoreCounting), Some(TerminalStage::Play)]);
    let (result, _, _) = exercise_terminal(&mut io, &AtomicBool::new(false));
    assert!(result.unwrap_err().contains("advancement exhausted"));
    assert_eq!(io.terminal_inputs, [TerminalStage::ScoreCounting]);
    assert_eq!(io.trace.iter().filter(|entry| **entry == "probe").count(), 1);
    assert_eq!(io.trace.iter().filter(|entry| **entry == "capture").count(), POST_GAME_MAX_OBSERVATION_ROUNDS + 1);
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


/// Three read-only planning captures settle the logged pending counts before Solve.
/// The prior Draw action is counted before waiting; no lower tableau input occurs.
#[test]
fn pending_solve_settles_before_lower_tableau_input() {
    let (pending, settled) = pending_solve_pair();
    let solve = settled.prediction;
    let mut io = fake_frames(vec![
        observation(draw(), true), copy_observation(&pending), copy_observation(&pending), pending,
        settled, observation(PredictedAction::NoHighlight, false),
        observation(PredictedAction::NoHighlight, false),
    ]);

    io.completion_candidates = VecDeque::from([true, true]);
    let delays = AnimationSettleDelays::default().with_klondike_millis(750, 1_250);
    let (result, input_attempted, _) = exercise_with_settings(
        &mut io, draw(), StepRunSettings::new(delays, 2), &AtomicBool::new(false),
    );
    assert_eq!(result, Ok(RunOutcome::GameWon { previous_verified: 0, previous_halo: 1 }));
    assert_eq!(io.inputs, [draw(), solve]);

    assert_eq!(io.waits, [
        delays.klondike_settle, delays.klondike_reobserve, delays.klondike_reobserve,
        delays.klondike_reobserve, delays.klondike_solve, delays.klondike_reobserve,
    ]);
    assert!(input_attempted && io.frames.is_empty());
    assert!(!io.trace.contains(&"solver") && io.terminal_inputs.is_empty());
}


/// Persistent partial artwork exhausts one three-capture planning budget without input.
#[test]
fn persistent_pending_solve_stops_without_lower_card_or_solve_input() {
    let (pending, _) = pending_solve_pair();
    let lower = pending.prediction;
    let mut io = fake_frames(vec![
        observation(draw(), true), copy_observation(&pending), copy_observation(&pending), copy_observation(&pending), pending,
    ]);

    let (result, input_attempted, latest) = exercise(&mut io, draw(), 0, &AtomicBool::new(false));
    assert!(result.unwrap_err().contains("after 3 delayed input-free planning captures"));
    assert_eq!(io.inputs, [draw()]);

    assert_eq!(io.waits.len(), 4);
    assert_eq!(latest.unwrap().prediction, lower);
    assert!(input_attempted && io.frames.is_empty() && !io.trace.contains(&"solver"));
}


/// A disappeared candidate resumes only from its newly classified lower target.
#[test]
fn disappearing_pending_solve_uses_fresh_lower_target_without_solver_refresh() {
    let (pending, _) = pending_solve_pair();
    let absent = absent_solve_observation(&pending);
    let lower = absent.prediction;
    assert!(matches!(lower, PredictedAction::Action(GuidedAction {
        target: ActionTarget::Klondike(KlondikeTarget::Tableau { .. }), ..
    })));
    let mut io = fake_frames(vec![observation(draw(), true), pending, absent, observation(draw(), true)]);

    let (result, _, latest) = exercise(&mut io, draw(), 2, &AtomicBool::new(false));
    assert_eq!(result, Ok(RunOutcome::Completed { verified: 0, halo: 2 }));
    assert_eq!(io.inputs, [draw(), lower]);

    assert_eq!(io.waits.len(), 3);
    assert_eq!(latest.unwrap().prediction, draw());
    assert!(!io.trace.contains(&"solver"));
}


/// Typed DRAW/Recycle/RIGHT priority is retained even beside pending Solve pixels.
/// These adapters inject the higher target separately; detector priority has its own tests.
#[test]
fn pending_solve_never_defers_higher_priority_draw_recycle_or_right() {
    let (pending, _) = pending_solve_pair();


    for target in [KlondikeTarget::Draw, KlondikeTarget::Recycle, KlondikeTarget::Waste { offset: 1 }] {
        let higher = PredictedAction::Action(klondike::canonical_action(target).unwrap());
        let mut planning = copy_observation(&pending);
        planning.prediction = higher;
        let mut io = fake_frames(vec![planning, observation(draw(), true)]);

        let (result, _, _) = exercise(&mut io, higher, 1, &AtomicBool::new(false));
        assert_eq!(result, Ok(RunOutcome::Completed { verified: 0, halo: 1 }));
        assert_eq!(io.inputs, [higher]);
        assert_eq!(io.waits, [AnimationSettleDelays::default().klondike_settle]);
        assert!(io.frames.is_empty());
    }
}


/// A foundation source is also lower priority than stable pending Solve lettering.
#[test]
fn pending_solve_defers_foundation_source_input() {
    let (mut pending, _) = pending_solve_pair();
    let foundation = PredictedAction::Action(klondike::canonical_action(KlondikeTarget::Foundation { column: 3 }).unwrap());
    pending.prediction = foundation;
    let mut io = fake_frames(vec![copy_observation(&pending), copy_observation(&pending), copy_observation(&pending), pending]);
    let (result, input_attempted, _) = exercise(&mut io, foundation, 1, &AtomicBool::new(false));
    assert!(result.unwrap_err().contains("pending Klondike Solve"));
    assert!(!input_attempted && io.inputs.is_empty());
    assert_eq!(io.waits.len(), REOBSERVATION_LIMIT);
}


/// Finite action budgets finish before planning-only Solve observations are started.
#[test]
fn final_step_does_not_wait_for_pending_solve_after_its_budget_is_consumed() {
    let (pending, _) = pending_solve_pair();
    let lower = pending.prediction;
    let mut io = fake_frames(vec![observation(draw(), true), pending]);

    let (result, _, latest) = exercise(&mut io, draw(), 1, &AtomicBool::new(false));
    assert_eq!(result, Ok(RunOutcome::Completed { verified: 0, halo: 1 }));
    assert_eq!(io.inputs, [draw()]);
    assert_eq!(io.waits, [AnimationSettleDelays::default().klondike_settle]);
    assert_eq!(latest.unwrap().prediction, lower);
    assert!(io.frames.is_empty());
}


/// STOP during pending artwork observation or the final fresh probe forbids Solve.
#[test]
fn pending_solve_wait_and_final_probe_obey_stop_without_replaying_input() {


    for stop_during_wait in [true, false] {
        let stop = AtomicBool::new(false);
        let (pending, settled) = pending_solve_pair();
        let mut io = fake_frames(vec![observation(draw(), true), pending, settled]);



        if stop_during_wait {
            io.stop_on_wait = Some(&stop);
            io.waits_before_stop = 1;
        } else {
            io.stop_on_probe = Some(&stop);
            io.probes_before_stop = 1;
        }
        let (result, _, _) = exercise(&mut io, draw(), 0, &stop);
        assert!(result.unwrap_err().contains("STOP"));
        assert_eq!(io.inputs, [draw()]);

        assert!(!io.trace.contains(&"solver"));
    }
}


/// Malformed recapture storage is retained and cannot authorise a lower card click.
#[test]
fn pending_solve_recapture_with_malformed_storage_stops_with_latest_pixels() {
    let (pending, _) = pending_solve_pair();
    let mut malformed = copy_observation(&pending);
    malformed.frame.pixels.truncate(1);
    let mut io = fake_frames(vec![observation(draw(), true), pending, malformed]);

    let (result, _, latest) = exercise(&mut io, draw(), 0, &AtomicBool::new(false));
    assert!(result.unwrap_err().contains("pending Solve analysis failed"));
    assert_eq!(io.inputs, [draw()]);
    assert_eq!(latest.unwrap().frame.pixels.len(), 1);
    assert!(!io.trace.contains(&"solver"));
}


/// Captured pixels survive an analysis failure encountered during pending settling.
#[test]
fn pending_solve_capture_analysis_error_retains_the_new_diagnostic_frame() {
    let (pending, settled) = pending_solve_pair();
    let mut io = fake_frames(vec![settled]);
    io.capture_failure = Some("simulated pending recapture analysis failure");
    let mut latest = Some(pending);
    let result = await_pending_solve(
        &mut io, &mut latest, StepRunSettings::new(AnimationSettleDelays::default(), 0),
        &event_sink(), &AtomicBool::new(false),
    );
    assert!(result.unwrap_err().contains("simulated pending recapture analysis failure"));
    assert!(latest.unwrap().frame.is_layout_valid());
    assert_eq!(io.trace, ["wait", "capture"]);
    assert!(io.inputs.is_empty());
}


/// Pending recaptures preserve the acknowledged HALO count without transfer proof.
#[test]
fn pending_solve_wait_keeps_the_prior_unverified_source_verdict_separate() {
    let (pending, settled) = pending_solve_pair();
    let right = PredictedAction::Action(klondike::canonical_action(KlondikeTarget::Waste { offset: 1 }).unwrap());
    let solve = settled.prediction;
    let mut io = fake_frames(vec![
        observation(right, true), pending, settled,
        observation(PredictedAction::NoHighlight, false), observation(PredictedAction::NoHighlight, false),
    ]);

    io.completion_candidates = VecDeque::from([true, true]);
    let (result, _, _) = exercise(&mut io, right, 2, &AtomicBool::new(false));
    assert_eq!(result, Ok(RunOutcome::GameWon { previous_verified: 0, previous_halo: 1 }));
    assert_eq!(io.inputs, [right, solve]);

    assert!(!io.trace.contains(&"solver"));
}


/// A recognised new-game cycle retains the active continuous request after settling.
/// Its new Solve target is not compared against the previous game's approved preview.
#[test]
fn restarted_continuous_game_can_settle_pending_solve_before_its_next_action() {
    use klondike::terminal::TerminalStage;
    let stop = AtomicBool::new(false);
    let (pending, settled) = pending_solve_pair();
    let solve = settled.prediction;
    let mut frames = vec![observation(solve, true)];
    frames.extend((0..6).map(|_| observation(PredictedAction::NoHighlight, false)));
    frames.extend([
        pending, settled, observation(PredictedAction::NoHighlight, false),
        observation(PredictedAction::NoHighlight, false),
    ]);
    let mut io = fake_frames(frames);
    io.completion_candidates = VecDeque::from([true, true, true, true]);
    let stages = [TerminalStage::ScoreCounting, TerminalStage::LevelUp, TerminalStage::NewGame,
        TerminalStage::Play, TerminalStage::SolverReady];
    io.terminal_stages = stages.into_iter().chain([TerminalStage::ScoreCounting]).map(Some).collect();
    io.stop_on_probe = Some(&stop);
    io.probes_before_stop = 7;
    let (result, input_attempted, _) = exercise(&mut io, solve, 0, &stop);
    assert!(result.unwrap_err().contains("STOP"));
    assert!(input_attempted);
    assert_eq!(io.inputs, [solve, solve]);
    assert_eq!(io.terminal_inputs, stages);

    assert!(io.frames.is_empty() && !io.trace.contains(&"solver"));
}


/// Production completion/terminal classifiers drive the medal-layout Level Up
/// through the established restart stages. Each control is sent once; STOP
/// precedes the first action on the next deal, with no mock stage approvals.
#[test]
fn native_pro_level_up_restart_cycle_recognises_each_control_once() {
    use klondike::terminal::TerminalStage;
    let native = |number: u8| {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("tests/fixtures/klondike/K{number:02}.png"));
        native_observation(crate::capture::decode_png(&std::fs::read(path).unwrap()).unwrap())
    };
    let initial = native(25);
    let approved = initial.prediction;
    assert_eq!(approved, PredictedAction::Action(klondike::canonical_action(KlondikeTarget::Solve).unwrap()));
    let mut io = fake_frames(vec![
        initial, native(26), native(26), native(67), native(28), native(29), native(30), native(2),
    ]);
    let stop = AtomicBool::new(false);
    io.native_terminal_analysis = true;
    io.stop_on_probe = Some(&stop);
    io.probes_before_stop = 6;
    let (result, input_attempted, latest) = exercise(&mut io, approved, 0, &stop);
    assert!(result.unwrap_err().contains("STOP"));
    assert!(input_attempted);
    assert_eq!(io.inputs, [approved]);
    assert_eq!(io.terminal_inputs, [TerminalStage::ScoreCounting, TerminalStage::LevelUp,
        TerminalStage::NewGame, TerminalStage::Play, TerminalStage::SolverReady]);
    assert_eq!(io.completion_checks, 2);

    assert!(io.frames.is_empty() && !io.trace.contains(&"solver"));
    assert!(matches!(latest.unwrap().prediction, PredictedAction::Action(_)));
}


/// The native level-101 OK, New Game and Play frames advance in deterministic
/// order after independent completion. Former theme signatures are not consulted;
/// each control is sent once and STOP prevents gameplay on the next fresh deal.
#[test]
fn native_grandmaster_buttons_restart_after_confirmed_win_without_theme_matching() {
    use klondike::terminal::TerminalStage;
    let initial = native_hint_phase(25);
    let approved = initial.prediction;
    let mut io = fake_frames(vec![
        initial, native_hint_phase(26), native_hint_phase(26), native_hint_phase(91),
        native_hint_phase(92), native_hint_phase(93), native_hint_phase(77), native_hint_phase(2),
    ]);
    let stop = AtomicBool::new(false);
    io.native_terminal_analysis = true;
    io.stop_on_probe = Some(&stop);
    io.probes_before_stop = 6;
    let (result, attempted, latest) = exercise(&mut io, approved, 0, &stop);
    assert!(result.unwrap_err().contains("STOP"));
    assert!(attempted);
    assert_eq!(io.inputs, [approved]);
    assert_eq!(io.terminal_inputs, [TerminalStage::ScoreCounting, TerminalStage::LevelUp,
        TerminalStage::NewGame, TerminalStage::Play, TerminalStage::SolverReady]);
    assert_eq!(io.completion_checks, 2);
    assert!(io.frames.is_empty() && !io.trace.contains(&"solver"));
    assert_eq!(latest.unwrap().prediction, native_hint_phase(2).prediction);
}


/// Five unchanged score panels exceed the former short wait but do not replay
/// score input. The existing terminal budget permits the delayed local OK frame,
/// then one acknowledged click at every expected control, retaining editable waits.
#[test]
fn confirmed_win_waits_for_late_expected_ok_without_repeating_score_input() {
    use klondike::terminal::TerminalStage;
    let initial = native_hint_phase(25);
    let approved = initial.prediction;
    let mut frames = vec![initial, native_hint_phase(26), native_hint_phase(26)];
    frames.extend((0..5).map(|_| native_hint_phase(26)));
    frames.extend([native_hint_phase(91), native_hint_phase(92), native_hint_phase(93),
        native_hint_phase(77), native_hint_phase(2)]);
    let stop = AtomicBool::new(false);
    let mut io = fake_frames(frames);
    io.native_terminal_analysis = true;
    io.stop_on_probe = Some(&stop);
    io.probes_before_stop = 6;
    let (result, _, _) = exercise(&mut io, approved, 0, &stop);
    assert!(result.unwrap_err().contains("STOP"));
    assert_eq!(io.terminal_inputs, [TerminalStage::ScoreCounting, TerminalStage::LevelUp,
        TerminalStage::NewGame, TerminalStage::Play, TerminalStage::SolverReady]);
    assert_eq!(io.trace.iter().filter(|entry| **entry == "capture").count(), 13);
    assert!(!io.trace.contains(&"solver"));
}


/// The old OK remains visible after acknowledgement but cannot authorise New Game.
/// Bounded observations preserve the last screen without replaying either click.
#[test]
fn confirmed_win_unchanged_native_ok_cannot_authorise_next_control() {
    use klondike::terminal::TerminalStage;
    let initial = native_hint_phase(25);
    let approved = initial.prediction;
    let mut frames = vec![initial, native_hint_phase(26), native_hint_phase(26), native_hint_phase(91)];
    frames.extend((0..=POST_GAME_MAX_OBSERVATION_ROUNDS).map(|_| native_hint_phase(91)));
    let mut io = fake_frames(frames);
    io.native_terminal_analysis = true;
    let (result, attempted, latest) = exercise(&mut io, approved, 0, &AtomicBool::new(false));
    assert!(result.unwrap_err().contains("terminal advancement exhausted"));
    assert!(attempted);
    assert_eq!(io.inputs, [approved]);
    assert_eq!(io.terminal_inputs, [TerminalStage::ScoreCounting, TerminalStage::LevelUp]);
    assert!(io.frames.is_empty() && !io.trace.contains(&"solver"));
    assert_eq!(latest.unwrap().frame.pixels, native_hint_phase(91).frame.pixels);
}


/// An uncertain expected OK delivery stops without result capture or New Game.
#[test]
fn confirmed_win_expected_ok_uncertainty_does_not_replay_or_advance() {
    use klondike::terminal::TerminalStage;
    let initial = native_hint_phase(25);
    let approved = initial.prediction;
    let mut io = fake_frames(vec![initial, native_hint_phase(26), native_hint_phase(26),
        native_hint_phase(91), native_hint_phase(92)]);
    io.native_terminal_analysis = true;
    io.fail_terminal_stage = Some(TerminalStage::LevelUp);
    let (result, attempted, latest) = exercise(&mut io, approved, 0, &AtomicBool::new(false));
    assert!(result.unwrap_err().contains("uncertain terminal input"));
    assert!(attempted);
    assert_eq!(io.terminal_inputs, [TerminalStage::ScoreCounting, TerminalStage::LevelUp]);
    assert_eq!(io.frames.len(), 1);
    assert_eq!(latest.unwrap().frame.pixels, native_hint_phase(91).frame.pixels);
}


/// K86 supplies the native Master Level Up screen after the score-skip click.
/// Separate captured stages compose a controlled restart replay, not the
/// unretained historical frames. Each terminal control is acknowledged once.
#[test]
fn native_master_level_up_restart_cycle_recognises_ok_then_new_game() {
    use klondike::terminal::TerminalStage;
    let initial = native_hint_phase(25);
    let approved = initial.prediction;
    let mut io = fake_frames(vec![
        initial, native_hint_phase(26), native_hint_phase(26), native_hint_phase(86),
        native_hint_phase(28), native_hint_phase(29), native_hint_phase(30), native_hint_phase(2),
    ]);
    let stop = AtomicBool::new(false);
    io.native_terminal_analysis = true;
    io.stop_on_probe = Some(&stop);
    io.probes_before_stop = 6;
    let (result, attempted, latest) = exercise(&mut io, approved, 0, &stop);
    assert!(result.unwrap_err().contains("STOP"));
    assert!(attempted);
    assert_eq!(io.inputs, [approved]);
    assert_eq!(io.terminal_inputs, [TerminalStage::ScoreCounting, TerminalStage::LevelUp,
        TerminalStage::NewGame, TerminalStage::Play, TerminalStage::SolverReady]);
    assert_eq!(io.completion_checks, 2);
    assert!(io.frames.is_empty() && !io.trace.contains(&"solver"));
    assert_eq!(latest.unwrap().prediction, native_hint_phase(2).prediction);
}


/// Positive K77 deal geometry advances once through SolverReady, then an
/// unchanged Solver-off observation receives only a delayed fresh capture.
/// A separately supplied native HALO exercises resumption while STOP prevents
/// gameplay on that fresh board; no historic frame sequence is asserted.
#[test]
fn native_new_deal_activates_solver_once_then_waits_for_fresh_halo() {
    use klondike::terminal::TerminalStage;
    let (approved, mut frames) = native_restart_to_solver_off_deal();
    frames.push(native_hint_phase(77));
    let next = native_hint_phase(2);
    assert!(matches!(next.prediction, PredictedAction::Action(_)));
    let next_prediction = next.prediction;
    let next_pixels = next.frame.pixels.clone();
    frames.push(next);
    let stop = AtomicBool::new(false);
    let mut io = fake_frames(frames);
    io.native_terminal_analysis = true;
    io.stop_on_probe = Some(&stop);
    io.probes_before_stop = 6;
    let (result, input_attempted, latest) = exercise(&mut io, approved, 0, &stop);
    assert!(result.unwrap_err().contains("STOP"));
    assert!(input_attempted);
    assert_eq!(io.inputs, [approved]);
    assert_eq!(io.terminal_inputs, [TerminalStage::ScoreCounting, TerminalStage::LevelUp,
        TerminalStage::NewGame, TerminalStage::Play, TerminalStage::SolverReady]);
    assert_eq!(io.completion_checks, 2);

    assert_eq!(io.trace.iter().filter(|entry| **entry == "capture").count(), 9);
    assert_eq!(io.trace.iter().filter(|entry| **entry == "probe").count(), 7);
    assert!(io.frames.is_empty() && !io.trace.contains(&"solver"));
    assert_eq!(latest.as_ref().unwrap().prediction, next_prediction);
    assert_eq!(latest.unwrap().frame.pixels, next_pixels);
}


/// No-HALO after acknowledged SolverReady does not establish the next board's
/// gameplay authority. Four unchanged native K77 observations exhaust the
/// existing read-only budget without replaying the Solver toolbar click.
#[test]
fn native_new_deal_without_fresh_halo_stops_after_one_solver_ready_input() {
    use klondike::terminal::TerminalStage;
    let (approved, mut frames) = native_restart_to_solver_off_deal();
    frames.extend((0..4).map(|_| native_hint_phase(77)));
    let mut io = fake_frames(frames);
    io.native_terminal_analysis = true;
    let (result, input_attempted, latest) = exercise(&mut io, approved, 0, &AtomicBool::new(false));
    assert!(result.unwrap_err().contains("terminal advancement exhausted"));
    assert!(input_attempted);
    assert_eq!(io.inputs, [approved]);
    assert_eq!(io.terminal_inputs, [TerminalStage::ScoreCounting, TerminalStage::LevelUp,
        TerminalStage::NewGame, TerminalStage::Play, TerminalStage::SolverReady]);

    assert!(io.frames.is_empty() && !io.trace.contains(&"solver"));
    assert_eq!(latest.as_ref().unwrap().prediction, PredictedAction::NoHighlight);
    assert_eq!(latest.unwrap().frame.pixels, native_hint_phase(77).frame.pixels);
}


/// A recognised new deal cannot bypass STOP or an uncertain SolverReady input.
/// The separately supplied actionable result stays unread on uncertain delivery.
#[test]
fn native_new_deal_keeps_solver_ready_stop_and_uncertainty_boundaries() {
    use klondike::terminal::TerminalStage;
    let (approved, frames) = native_restart_to_solver_off_deal();
    let stop = AtomicBool::new(false);
    let mut stopped = fake_frames(frames);
    stopped.native_terminal_analysis = true;
    stopped.stop_on_probe = Some(&stop);
    stopped.probes_before_stop = 5;
    let (result, input_attempted, latest) = exercise(&mut stopped, approved, 0, &stop);
    assert!(result.unwrap_err().contains("STOP"));
    assert!(input_attempted, "only the earlier acknowledged controls were attempted");
    assert_eq!(stopped.inputs, [approved]);
    assert_eq!(stopped.terminal_inputs, [TerminalStage::ScoreCounting, TerminalStage::LevelUp,
        TerminalStage::NewGame, TerminalStage::Play]);

    assert!(stopped.frames.is_empty() && !stopped.trace.contains(&"solver"));
    assert_eq!(latest.unwrap().frame.pixels, native_hint_phase(77).frame.pixels);

    let (_, mut frames) = native_restart_to_solver_off_deal();
    frames.push(native_hint_phase(2));
    let mut uncertain = fake_frames(frames);
    uncertain.native_terminal_analysis = true;
    uncertain.fail_terminal_stage = Some(TerminalStage::SolverReady);
    let (result, input_attempted, latest) = exercise(&mut uncertain, approved, 0, &AtomicBool::new(false));
    assert!(result.unwrap_err().contains("uncertain terminal input"));
    assert!(input_attempted);
    assert_eq!(uncertain.inputs, [approved]);
    assert_eq!(uncertain.terminal_inputs, [TerminalStage::ScoreCounting, TerminalStage::LevelUp,
        TerminalStage::NewGame, TerminalStage::Play, TerminalStage::SolverReady]);

    assert_eq!(uncertain.frames.len(), 1);
    assert!(!uncertain.trace.contains(&"solver"));
    assert_eq!(latest.unwrap().frame.pixels, native_hint_phase(77).frame.pixels);
}


/// Read a complete original frame for the same-card Hint-overlay regressions.
/// The later manual phases do not recreate any unsaved historical result frame.
fn native_hint_phase(number: u8) -> FrameObservation {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("tests/fixtures/klondike/K{number:02}.png"));
    native_observation(crate::capture::decode_png(&std::fs::read(path).unwrap()).unwrap())
}


/// The native red deal is recognised with Solver off; the production controller
/// waits through its existing read-only budget, then follows fresh K97 evidence.
/// Repeated fixture observations model recovery flow, not measured live timing.
#[test]
fn native_red_back_deal_activates_solver_once_then_uses_fresh_source() {
    let off = native_hint_phase(96);
    let on = native_hint_phase(97);
    assert!(off.gameplay_scene && on.gameplay_scene);
    assert_eq!(off.prediction, PredictedAction::NoHighlight);
    assert_eq!(on.prediction, canonical(KlondikeTarget::Tableau { column: 7, top: 443, bottom: 632 }));


    for limit in [0, 1] {
        let mut frames = (0..=REOBSERVATION_LIMIT).map(|_| copy_observation(&off)).collect::<Vec<_>>();
        frames.extend([copy_observation(&on), copy_observation(&on)]);
        let stop = AtomicBool::new(false);
        let mut io = fake_frames(frames);
        io.native_terminal_analysis = true;


        if limit == 0 {
            io.stop_on_probe = Some(&stop);
            io.probes_before_stop = 2;
        }
        let (result, attempted, latest) = exercise(&mut io, off.prediction, limit, &stop);


        if limit == 1 {
            assert_eq!(result, Ok(RunOutcome::Completed { verified: 0, halo: 1 }));
        } else {
            assert!(result.unwrap_err().contains("STOP"));
        }
        assert!(attempted && io.frames.is_empty());
        assert_eq!(io.inputs, [on.prediction]);
        assert!(io.terminal_inputs.is_empty());
        assert_eq!(io.trace.iter().filter(|entry| **entry == "solver").count(), 1);
        let solver = io.trace.iter().position(|entry| *entry == "solver").unwrap();
        assert_eq!(io.trace[..solver].iter().filter(|entry| **entry == "capture").count(), REOBSERVATION_LIMIT + 1);
        assert_eq!(io.trace[solver + 1], "capture", "Solver is followed immediately by a fresh capture");
        assert_eq!(latest.unwrap().prediction, on.prediction);
    }
}


/// A fresh native red-backed source goes directly to its canonical action;
/// an already active Solver must not be toggled during this Step Once request.
#[test]
fn native_red_back_halo_plays_one_source_without_solver_refresh() {
    let on = native_hint_phase(97);
    let mut io = fake_frames(vec![copy_observation(&on), copy_observation(&on)]);
    io.native_terminal_analysis = true;
    let (result, attempted, latest) = exercise(&mut io, PredictedAction::NoHighlight, 1, &AtomicBool::new(false));
    assert_eq!(result, Ok(RunOutcome::Completed { verified: 0, halo: 1 }));
    assert!(attempted && io.frames.is_empty());
    assert_eq!(io.inputs, [on.prediction]);
    assert_eq!(io.trace, ["capture", "probe", "input", "wait", "capture"]);
    assert!(!io.trace.contains(&"solver") && io.terminal_inputs.is_empty());
    assert_eq!(latest.unwrap().prediction, on.prediction);
}


/// Persistent native red no-HALO pixels remain bounded after one activation;
/// absent recommendations never become speculative gameplay or terminal input.
#[test]
fn native_red_back_no_halo_stops_after_one_bounded_solver_refresh() {
    let off = native_hint_phase(96);


    for limit in [0, 1] {
        let frames = (0..2 * (REOBSERVATION_LIMIT + 1)).map(|_| copy_observation(&off)).collect();
        let mut io = fake_frames(frames);
        io.native_terminal_analysis = true;
        let (result, attempted, latest) = exercise(&mut io, off.prediction, limit, &AtomicBool::new(false));
        assert!(result.is_err() && attempted && io.frames.is_empty());
        assert!(io.inputs.is_empty() && io.terminal_inputs.is_empty());
        assert_eq!(io.trace.iter().filter(|entry| **entry == "solver").count(), 1);
        assert_eq!(io.trace.iter().filter(|entry| **entry == "capture").count(), 2 * (REOBSERVATION_LIMIT + 1));
        assert_eq!(latest.unwrap().prediction, PredictedAction::NoHighlight);
    }
}




/// Assemble native terminal artwork from separate original fixtures. This
/// deterministic composition is not Charlie's unsaved live restart sequence.
/// K77 supplies the newly dealt Solver-off board only after acknowledged Play.
fn native_restart_to_solver_off_deal() -> (PredictedAction, Vec<FrameObservation>) {
    let frames: Vec<_> = [25, 26, 26, 67, 28, 29, 77].into_iter().map(native_hint_phase).collect();
    let approved = frames[0].prediction;
    assert_eq!(approved, PredictedAction::Action(klondike::canonical_action(KlondikeTarget::Solve).unwrap()));
    assert_eq!(frames.last().unwrap().prediction, PredictedAction::NoHighlight);
    assert_eq!(klondike::terminal::classify_terminal(&frames.last().unwrap().frame).unwrap(),
        Some(klondike::terminal::TerminalStage::SolverReady));
    (approved, frames)
}





/// Manual source phases are distinct valid recommendations but not a proved transfer.
#[test]
fn native_same_card_phase_continues_only_from_each_fresh_canonical_halo() {
    let thin = native_hint_phase(75);
    let thick = native_hint_phase(74);


    for (before, after) in [(&thin, &thick), (&thick, &thin)] {
        let PredictedAction::Action(planned) = before.prediction else { panic!("native source must be actionable"); };
        let evidence = klondike::inspect_effect(&before.frame, &after.frame, planned).unwrap();
        assert!(!evidence.verified && !evidence.source_replaced, "phase is not transfer proof: {evidence}");
        let mut io = fake_frames(vec![copy_observation(before), copy_observation(after), copy_observation(after)]);
        let (result, attempted, latest) = exercise(&mut io, before.prediction, 2, &AtomicBool::new(false));
        assert_eq!(result, Ok(RunOutcome::Completed { verified: 0, halo: 2 }));
        assert!(attempted);
        assert_eq!(io.inputs, [before.prediction, after.prediction]);
        assert_eq!(latest.unwrap().frame.pixels, after.frame.pixels);
        assert!(!io.trace.contains(&"solver") && io.terminal_inputs.is_empty());
    }
}


/// The new sparse SUIT return pair supplies a valid next RIGHT recommendation.
/// Its manual captures are regression inputs, not asserted historical worker bytes.
#[test]
fn native_sparse_suit_return_does_not_gate_the_next_halo_on_old_print_matching() {
    let before = native_hint_phase(78);
    let after = native_hint_phase(79);
    let PredictedAction::Action(planned) = before.prediction else { panic!("native SUIT source must be actionable"); };
    assert_eq!(planned.target, ActionTarget::Klondike(KlondikeTarget::Foundation { column: 2 }));
    assert_eq!(after.prediction, canonical(KlondikeTarget::Waste { offset: 2 }));
    let evidence = klondike::inspect_effect(&before.frame, &after.frame, planned).unwrap();
    assert!(!evidence.verified, "old proof remains diagnostic only: {evidence}");
    let mut io = fake_frames(vec![copy_observation(&before), copy_observation(&after), copy_observation(&after)]);
    let (result, attempted, latest) = exercise(&mut io, before.prediction, 2, &AtomicBool::new(false));
    assert_eq!(result, Ok(RunOutcome::Completed { verified: 0, halo: 2 }));
    assert!(attempted);
    assert_eq!(io.inputs, [before.prediction, after.prediction]);
    assert_eq!(io.trace, ["capture", "probe", "input", "wait", "capture", "probe", "input", "wait", "capture"]);
    assert_eq!(latest.unwrap().frame.pixels, after.frame.pixels);
    assert_eq!(io.completion_checks, 0);
    assert!(!io.trace.contains(&"solver") && io.terminal_inputs.is_empty());
}


/// The failure snapshot and later Undo reconstruction supply native target
/// evidence for a controlled replay, not the unsaved historical worker frames.
/// Fresh long-source authority does not prove either acknowledged action's effect.
#[test]
fn native_long_column_seven_source_continues_from_acknowledged_waste_without_effect_proof() {
    let before = native_hint_phase(81);
    let after = native_hint_phase(80);
    assert!(before.gameplay_scene && after.gameplay_scene);
    assert_eq!(before.prediction, canonical(KlondikeTarget::Waste { offset: 1 }));
    assert_eq!(after.prediction, canonical(KlondikeTarget::Tableau { column: 7, top: 372, bottom: 947 }));


    for limit in [1, 2] {
        let mut frames = vec![copy_observation(&before), copy_observation(&after)];
        let mut expected_inputs = vec![before.prediction];


        if limit == 2 {
            frames.push(copy_observation(&after));
            expected_inputs.push(after.prediction);
        }
        let mut io = fake_frames(frames);
        let (events, receiver) = std::sync::mpsc::channel();
        let sink = WorkerEventSink {
            events,
            latest_frame: super::super::LatestFrameSlot::default(),
            capture_context: std::sync::Mutex::new(None),
        };
        let mut latest = None;
        let mut attempted = false;
        let result = drive_run(
            &mut io, before.prediction, StepRunSettings::new(AnimationSettleDelays::default(), limit),
            &sink, &AtomicBool::new(false), &mut latest, &mut attempted,
        );
        assert_eq!(result, Ok(RunOutcome::Completed { verified: 0, halo: limit }));
        assert!(attempted);
        assert_eq!(io.inputs, expected_inputs);
        assert_eq!(io.trace.iter().filter(|entry| **entry == "capture").count(), limit + 1);
        assert_eq!(io.trace.iter().filter(|entry| **entry == "probe").count(), limit);
        assert_eq!(io.trace.iter().filter(|entry| **entry == "input").count(), limit);
        assert_eq!(io.trace.iter().filter(|entry| **entry == "wait").count(), limit);
        assert!(io.frames.is_empty());
        assert_eq!(latest.unwrap().frame.pixels, after.frame.pixels);
        assert_eq!(io.completion_checks, 0);
        assert!(!io.trace.contains(&"solver") && io.terminal_inputs.is_empty());
        let accepted: Vec<_> = receiver.try_iter().map(|notification| notification.event)
            .filter(|event| matches!(event, WorkerEvent::ActionCompleted { .. })).collect();
        assert_eq!(accepted.len(), limit);


        for (index, event) in accepted.into_iter().enumerate() {
            assert!(matches!(event, WorkerEvent::ActionCompleted {
                operation_index, operation_limit, before: input, after: result,
                input_commands: 3, input_events: 4, changed_pixels: 0, continued_from_halo: true,
            } if operation_index == index + 1 && operation_limit == limit
                && input == expected_inputs[index] && result == after.prediction));
        }
    }
}


/// The later Undo/manual pairs reproduce native recommendation transitions
/// without asserting identity with the unsaved worker captures. A long source
/// crossing the old scene gutter remains one fresh action, with no effect gate.
#[test]
fn native_column_five_sources_continue_with_exact_finite_action_budgets() {
    let cases = [
        (83, 82, KlondikeTarget::Waste { offset: 2 }, KlondikeTarget::Tableau { column: 5, top: 407, bottom: 947 }),
        (85, 84, KlondikeTarget::Tableau { column: 2, top: 336, bottom: 527 },
            KlondikeTarget::Tableau { column: 5, top: 389, bottom: 947 }),
        (88, 87, KlondikeTarget::Waste { offset: 0 },
            KlondikeTarget::Tableau { column: 5, top: 610, bottom: 909 }),
    ];


    for (before_number, after_number, before_target, after_target) in cases {
        let before = native_hint_phase(before_number);
        let after = native_hint_phase(after_number);
        assert!(before.gameplay_scene && after.gameplay_scene);
        assert_eq!(before.prediction, canonical(before_target));
        assert_eq!(after.prediction, canonical(after_target));


        for limit in [1, 2] {
            let mut frames = vec![copy_observation(&before), copy_observation(&after)];
            let mut expected_inputs = vec![before.prediction];


            if limit == 2 {
                frames.push(copy_observation(&after));
                expected_inputs.push(after.prediction);
            }
            let mut io = fake_frames(frames);
            let (result, attempted, latest) = exercise(&mut io, before.prediction, limit, &AtomicBool::new(false));
            assert_eq!(result, Ok(RunOutcome::Completed { verified: 0, halo: limit }));
            assert!(attempted);
            assert_eq!(io.inputs, expected_inputs);
            assert_eq!(io.trace.iter().filter(|entry| **entry == "capture").count(), limit + 1);
            assert_eq!(io.trace.iter().filter(|entry| **entry == "probe").count(), limit);
            assert_eq!(io.waits.len(), limit);
            assert!(io.frames.is_empty());
            assert_eq!(latest.unwrap().frame.pixels, after.frame.pixels);
            assert_eq!(io.completion_checks, 0);
            assert!(!io.trace.contains(&"solver") && io.terminal_inputs.is_empty());
        }
    }
}


/// STOP and uncertain delivery remain input boundaries even when the next HALO is valid.
#[test]
fn native_sparse_suit_return_preserves_stop_and_uncertain_input_guards() {
    let before = native_hint_phase(78);
    let after = native_hint_phase(79);
    let stop = AtomicBool::new(false);
    let mut stopped = fake_frames(vec![copy_observation(&before), copy_observation(&after)]);
    stopped.stop_on_probe = Some(&stop);
    stopped.probes_before_stop = 1;
    let (result, attempted, latest) = exercise(&mut stopped, before.prediction, 0, &stop);
    assert!(result.unwrap_err().contains("STOP"));
    assert!(attempted);
    assert_eq!(stopped.inputs, [before.prediction]);
    assert_eq!(latest.unwrap().frame.pixels, after.frame.pixels);
    let mut uncertain = fake_frames(vec![copy_observation(&before), after]);
    uncertain.fail_input = true;
    let (result, attempted, _) = exercise(&mut uncertain, before.prediction, 0, &AtomicBool::new(false));
    assert!(result.unwrap_err().contains("uncertain"));
    assert!(attempted);
    assert_eq!(uncertain.inputs, [before.prediction]);
    assert_eq!(uncertain.trace, ["capture", "probe", "input"]);
    assert_eq!(uncertain.frames.len(), 1);
}


/// A displaced preview remains uncalibrated despite the revised controller policy.
#[test]
fn native_displaced_hint_copy_stays_input_free_after_one_solver_refresh() {
    let duplicate = native_hint_phase(76);
    assert!(duplicate.gameplay_scene);
    assert_eq!(duplicate.prediction, PredictedAction::NoHighlight);
    let mut io = fake_frames((0..2 * (REOBSERVATION_LIMIT + 1)).map(|_| copy_observation(&duplicate)).collect());
    io.native_terminal_analysis = true;
    let (result, attempted, latest) = exercise(&mut io, duplicate.prediction, 0, &AtomicBool::new(false));
    assert!(result.is_err());
    assert!(attempted);
    assert!(io.inputs.is_empty() && io.terminal_inputs.is_empty());
    assert_eq!(io.trace.iter().filter(|entry| **entry == "solver").count(), 1);
    assert!(io.frames.is_empty());
    assert_eq!(latest.unwrap().frame.pixels, duplicate.frame.pixels);
}


/// Native Hint source detection cannot bypass the final STOP probe or uncertain delivery.
#[test]
fn native_hint_source_keeps_stop_and_uncertain_delivery_guards() {
    let thin = native_hint_phase(75);
    let thick = native_hint_phase(74);
    let approved = thin.prediction;
    let stop = AtomicBool::new(false);
    let mut stopped = fake_frames(vec![copy_observation(&thin), copy_observation(&thick)]);
    stopped.stop_on_probe = Some(&stop);
    let (result, attempted, latest) = exercise(&mut stopped, approved, 0, &stop);
    assert!(result.unwrap_err().contains("STOP"));
    assert!(!attempted && stopped.inputs.is_empty());
    assert_eq!(stopped.trace, ["capture", "probe"]);
    assert_eq!(stopped.frames.len(), 1);
    assert_eq!(latest.unwrap().frame.pixels, thin.frame.pixels);
    let mut uncertain = fake_frames(vec![thin, thick]);
    uncertain.fail_input = true;
    let (result, attempted, _) = exercise(&mut uncertain, approved, 0, &AtomicBool::new(false));
    assert!(result.unwrap_err().contains("uncertain"));
    assert!(attempted);
    assert_eq!(uncertain.trace, ["capture", "probe", "input"]);
    assert_eq!(uncertain.inputs, [approved]);
    assert_eq!(uncertain.frames.len(), 1);
}


/// A freshly settled Solve replaces the advisory lower source only after positive artwork.
#[test]
fn initial_pending_solve_uses_fresh_settled_control_without_stale_preview_approval() {
    let (pending, settled) = pending_solve_pair();
    let approved = pending.prediction;
    let solve = settled.prediction;
    let mut io = fake_frames(vec![pending, settled,
        observation(PredictedAction::NoHighlight, false), observation(PredictedAction::NoHighlight, false)]);
    io.completion_candidates = VecDeque::from([true, true]);
    let (result, attempted, _) = exercise(&mut io, approved, 1, &AtomicBool::new(false));
    assert_eq!(result, Ok(RunOutcome::GameWon { previous_verified: 0, previous_halo: 0 }));
    assert!(attempted);
    assert_eq!(io.inputs, [solve]);
    assert_eq!(io.trace, ["capture", "wait", "capture", "probe", "input", "wait", "capture", "wait", "capture"]);
    assert!(!io.trace.contains(&"solver"));
}


/// Changing source coordinates does not authorise another mode's stale preview.
#[test]
fn advisory_and_fresh_predictions_keep_the_selected_mode_boundary() {
    let PredictedAction::Action(mut foreign) = draw() else { unreachable!(); };
    foreign.target = ActionTarget::Bottom { index: 0, label: "DRAW" };
    let foreign = PredictedAction::Action(foreign);
    let mut initial = fake(&[(draw(), true)]);
    let (result, attempted, latest) = exercise(&mut initial, foreign, 1, &AtomicBool::new(false));
    assert!(result.is_err());
    assert!(!attempted && initial.trace.is_empty() && latest.is_none());
    let mut changed = fake(&[(draw(), true), (foreign, true)]);
    let (result, attempted, latest) = exercise(&mut changed, draw(), 2, &AtomicBool::new(false));
    assert!(result.is_err());
    assert!(attempted);
    assert_eq!(changed.inputs, [draw()]);
    assert_eq!(latest.unwrap().prediction, foreign);
    assert!(!changed.trace.contains(&"solver"));
}


/// Late STOP retains the newly captured result pixels without authorising another action.
#[test]
fn ordinary_result_capture_late_stop_retains_pixels_and_never_replays_input() {
    let stop = AtomicBool::new(false);
    let mut io = fake(&[(draw(), true), (draw(), true)]);
    io.frames.back_mut().unwrap().frame.pixels[0] = 213;
    io.stop_on_capture = Some(&stop);
    io.captures_before_stop = 1;
    let (result, attempted, latest) = exercise(&mut io, draw(), 2, &stop);
    assert!(result.unwrap_err().contains("STOP"));
    assert!(attempted);
    assert_eq!(io.inputs, [draw()]);
    assert_eq!(io.trace, ["capture", "probe", "input", "wait", "capture"]);
    let latest = latest.unwrap();
    assert_eq!(latest.frame.pixels[0], 213);
    assert_eq!(latest.prediction, PredictedAction::NoHighlight);
    assert!(!latest.gameplay_scene);
    assert!(io.terminal_inputs.is_empty());
}


/// Later Undo and manual FAIL frames reproduce a fresh long-source recommendation.
/// The run consumes each acknowledged action once and applies no card-pixel gate;
/// these frames do not assert historical worker identity or a proven game effect.
#[test]
fn native_hint_shadow_long_run_uses_fresh_targets_and_finite_budgets() {
    let before = native_hint_phase(90);
    let after = native_hint_phase(89);
    assert_eq!(before.prediction, canonical(KlondikeTarget::Tableau {
        column: 5, top: 337, bottom: 909,
    }));
    assert_eq!(after.prediction, canonical(KlondikeTarget::Tableau {
        column: 4, top: 372, bottom: 947,
    }));


    for limit in [1, 2] {
        let mut frames = vec![copy_observation(&before), copy_observation(&after)];
        let mut expected_inputs = vec![before.prediction];


        if limit == 2 {
            frames.push(copy_observation(&after));
            expected_inputs.push(after.prediction);
        }
        let mut io = fake_frames(frames);
        let (result, attempted, latest) = exercise(&mut io, before.prediction, limit, &AtomicBool::new(false));
        assert_eq!(result, Ok(RunOutcome::Completed { verified: 0, halo: limit }));
        assert!(attempted);
        assert_eq!(io.inputs, expected_inputs);
        assert_eq!(io.trace.iter().filter(|entry| **entry == "capture").count(), limit + 1);
        assert_eq!(io.trace.iter().filter(|entry| **entry == "probe").count(), limit);
        assert_eq!(io.waits.len(), limit);
        assert!(io.frames.is_empty());
        assert_eq!(latest.unwrap().frame.pixels, after.frame.pixels);
        assert_eq!(io.completion_checks, 0);
        assert!(!io.trace.contains(&"solver") && io.terminal_inputs.is_empty());
    }
}


/// The original K95 frame exposes a five-card source only above the toolbar.
/// Step Once groups that source, sends its one safe header click and retains the
/// fresh result without inspecting the covered lower border or proving an effect.
#[test]
fn native_toolbar_clipped_five_card_source_sends_one_step_once_click() {
    let before = native_hint_phase(95);
    let expected = canonical(KlondikeTarget::Tableau {
        column: 6, top: 571, bottom: 947,
    });
    assert!(before.gameplay_scene);
    assert_eq!(before.prediction, expected);
    let PredictedAction::Action(action) = expected else { panic!("native source must be actionable"); };
    assert_eq!(action.operation(), InputOperation::Click(PixelPoint::new(1296, 611)));
    let mut io = fake_frames(vec![copy_observation(&before), copy_observation(&before)]);
    let delays = AnimationSettleDelays::default().with_klondike_millis(750, 250);
    let (result, attempted, latest) = exercise_with_settings(
        &mut io, PredictedAction::NoHighlight, StepRunSettings::new(delays, 1), &AtomicBool::new(false),
    );
    assert_eq!(result, Ok(RunOutcome::Completed { verified: 0, halo: 1 }));
    assert!(attempted);
    assert_eq!(io.inputs, [expected]);
    assert_eq!(io.trace, ["capture", "probe", "input", "wait", "capture"]);
    assert_eq!(io.waits, [delays.klondike_settle]);
    assert_eq!(io.completion_checks, 0);
    assert!(io.frames.is_empty() && io.terminal_inputs.is_empty());
    assert_eq!(latest.unwrap().frame.pixels, before.frame.pixels);
}
