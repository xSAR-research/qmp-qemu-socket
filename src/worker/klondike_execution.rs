//! Klondike execution with bounded recovery and one-shot Solve requests.
//!
//! Zero requests continuous gameplay until STOP, uncertainty or the Solve endpoint.
//! Every unresolved context has finite Solver-refresh and observation budgets.
//! An uncertain input is never replayed. Solve is requested once and its result
//! retained for inspection; this module never infers completion or restart.

use std::{path::Path, sync::atomic::{AtomicBool, Ordering}, time::{Duration, Instant}};

use super::{
    FrameObservation, WorkerEvent, WorkerEventSink, WorkerState, analyse_captured_frame, capture_screen,
    connect_and_probe, execute_planned_input, format_action_status, format_prediction_target,
    format_step_plan, materially_changed_pixels, milliseconds, send_log, send_state,
    send_status, validate_probe, wait_or_stop,
};
use crate::{
    capture::CapturedFrame,
    game::{ActionTarget, GuidedAction},
    klondike,
    parameters::{
        ACTION_CHANGE_CHANNEL_THRESHOLD, KEY_HOLD, KLONDIKE_MAX_MULTI_STEP_ACTIONS, MOUSE_HOLD,
        NOMINAL_FRAME_HEIGHT, NOMINAL_FRAME_WIDTH, POINTER_SETTLE_DELAY, SOLVER_MOUSE_HOLD, StepRunSettings,
    },
    qmp::QmpClient,
    stepper::{StepPlan, plan_step},
    tracker::{PredictedAction, TableauScanState},
};


/// Maximum delayed observations after an immediate result or Solver capture.
const REOBSERVATION_LIMIT: usize = 3;


/// Accept continuous zero or a finite gameplay-action budget before connecting or clearing STOP.
pub(super) fn validate_operation_limit(limit: usize) -> Result<(), String> {


    if limit > KLONDIKE_MAX_MULTI_STEP_ACTIONS {
        return Err(format!(
            "Klondike requires 0 for continuous play or 1..={KLONDIKE_MAX_MULTI_STEP_ACTIONS} gameplay actions; requested {limit}"
        ));
    }
    Ok(())
}


/// Preserve decoded pixels separately from whether their analysis can authorise input.
enum CaptureResult {
    /// The frame was classified successfully with no pending STOP.
    Classified(FrameObservation),
    /// Pixels were decoded, but classification or a late STOP prevents action authority.
    Diagnostic {
        /// Latest decoded pixels retained for inspection even when analysis fails.
        frame: CapturedFrame,
        /// The cause returned after retaining the diagnostic frame.
        reason: String,
    },
}


/// Private boundary for deterministic controller tests using the production state machine.
trait KlondikeIo {
    /// Acquire and classify exactly one new frame, checking cancellation.
    fn capture(&mut self) -> Result<CaptureResult, String>;

    /// Acquire a diagnostic-only Solve result without action classification or discarding a late STOP frame.
    fn capture_diagnostic(&mut self) -> Result<CapturedFrame, String>;

    /// Revalidate VM state and the current QEMU HID Tablet immediately before input.
    fn probe(&mut self) -> Result<(), String>;

    /// Deliver the planned gameplay input once; errors make its outcome uncertain.
    fn input(&mut self, plan: StepPlan) -> Result<(), String>;

    /// Deliver one authorised Solver click; do not add a post-click settle delay.
    fn solver(&mut self) -> Result<(), String>;

    /// Wait in cancellable slices when ordinary gameplay or re-observation needs settling.
    fn wait(&mut self, duration: Duration) -> Result<(), String>;


    /// Establish a material game effect independently of the next HALO prediction.
    fn effect(
        &mut self,
        before: &CapturedFrame,
        after: &CapturedFrame,
        action: GuidedAction,
    ) -> Result<bool, String> {
        klondike::inspect_effect(before, after, action)
            .map(|evidence| evidence.verified)
            .map_err(|error| format!("Klondike effect analysis failed: {error}"))
    }
}


/// Real QMP adapter; the controller never sends host-wide keyboard or mouse input.
struct QmpKlondikeIo<'a> {
    /// Existing QMP connection, owned by this run's worker invocation.
    qmp: &'a mut QmpClient,
    /// Socket directory used by the shared PNG capture guard.
    socket_path: &'a Path,
    /// Immutable mode boundary used for every classification.
    scan_state: &'a TableauScanState,
    /// Cooperative cancellation flag, checked before input and observations.
    cancel_requested: &'a AtomicBool,
    /// Bounded UI output and complete session log for effect-evidence diagnostics.
    event_tx: &'a WorkerEventSink,
}


impl KlondikeIo for QmpKlondikeIo<'_> {


    fn capture(&mut self) -> Result<CaptureResult, String> {
        require_running(self.cancel_requested)?;
        let started = Instant::now();
        let (frame, timing) = capture_screen(self.qmp, self.socket_path)?;
        let result = classify_captured_result(frame, self.scan_state, self.cancel_requested);
        send_log(self.event_tx, format!(
            "PROFILE Klondike observation: acquisition and classification={:.1} ms; screendump={:.1} ms, decode={:.1} ms.",
            milliseconds(started.elapsed()), milliseconds(timing.screendump), milliseconds(timing.decode),
        ));
        Ok(result)
    }


    fn capture_diagnostic(&mut self) -> Result<CapturedFrame, String> {
        require_running(self.cancel_requested)?;
        capture_screen(self.qmp, self.socket_path).map(|(frame, _timing)| frame)
    }


    fn probe(&mut self) -> Result<(), String> {
        require_running(self.cancel_requested)?;
        let probe = self.qmp.probe().map_err(|error| format!("Klondike pre-input probe failed: {error}"))?;
        validate_probe(&probe)?;


        if probe.current_absolute_pointer_name() != Some("QEMU HID Tablet") {
            return Err("Klondike input requires the current absolute QEMU HID Tablet".to_owned());
        }
        require_running(self.cancel_requested)
    }


    fn input(&mut self, plan: StepPlan) -> Result<(), String> {
        let started = Instant::now();
        let result = execute_planned_input(self.qmp, plan, self.cancel_requested);
        send_log(self.event_tx, format!(
            "PROFILE Klondike input: {:.1} ms through QMP response handling; acknowledged={}; guest effect requires the next observation.",
            milliseconds(started.elapsed()), result.is_ok(),
        ));
        result
    }


    fn solver(&mut self) -> Result<(), String> {
        require_running(self.cancel_requested)?;
        self.qmp.click_with_hold(
            klondike::SOLVER_CLICK,
            NOMINAL_FRAME_WIDTH,
            NOMINAL_FRAME_HEIGHT,
            SOLVER_MOUSE_HOLD,
        ).map_err(|error| format!("Klondike Solver input is uncertain: {error}; no retry is permitted"))
    }


    fn wait(&mut self, duration: Duration) -> Result<(), String> {
        wait_or_stop(
            duration,
            self.cancel_requested,
            "STOP was requested during Klondike settling; no input was retried",
        ).map(|_| ())
    }


    fn effect(
        &mut self,
        before: &CapturedFrame,
        after: &CapturedFrame,
        action: GuidedAction,
    ) -> Result<bool, String> {
        let evidence = klondike::inspect_effect(before, after, action)
            .map_err(|error| format!("Klondike effect analysis failed: {error}"))?;
        send_log(self.event_tx, format!("Klondike effect evidence for {}: {evidence}", action.target));
        Ok(evidence.verified)
    }
}


/// Explicit endpoint without inferring the guest game has completed.
#[derive(Debug, PartialEq, Eq)]
enum RunOutcome {
    /// All requested gameplay actions have independently verified effects.
    Completed(usize),
    /// Solver refresh produced a preview that requires another explicit user run.
    RecoveryOnly,
    /// Solve was requested once; its newly captured result remains unverified.
    SolveRequested {
        /// Gameplay effects proven before the separate, unverified Solve request.
        previous_verified: usize,
    },
}


/// Run finite or continuous Klondike play using shared capture and QMP facilities.
///
/// Latest frames are retained on every stop. Unknown scenes cannot authorise
/// input, and no completion or restart is inferred from a missing HALO.
pub(super) fn run_klondike_steps(
    socket_path: &Path,
    approved_prediction: PredictedAction,
    settings: StepRunSettings,
    scan_state: &TableauScanState,
    event_tx: &WorkerEventSink,
    cancel_requested: &AtomicBool,
) {
    let mut latest = None;
    let mut input_attempted = false;
    let result = (|| {
        validate_operation_limit(settings.operation_limit())?;
        require_running(cancel_requested)?;
        send_state(event_tx, WorkerState::Connecting);
        let (mut qmp, probe) = connect_and_probe(socket_path)?;
        validate_probe(&probe)?;
        send_log(event_tx, probe.summary());
        let mut io = QmpKlondikeIo { qmp: &mut qmp, socket_path, scan_state, cancel_requested, event_tx };
        drive_run(
            &mut io, approved_prediction, settings, event_tx, cancel_requested,
            &mut latest, &mut input_attempted,
        )
    })();


    match result {
        Ok(RunOutcome::Completed(verified)) => {


            if let Some(observation) = latest {
                let prediction = observation.prediction;
                event_tx.publish_frame(observation.frame, prediction, false);
                let _ = event_tx.send(WorkerEvent::RunCompleted {
                    prediction,
                    requested_operations: settings.operation_limit(),
                    verified_operations: verified,
                    halo_operations: 0,
                });
            }
            send_state(event_tx, WorkerState::Ready);
            send_status(event_tx, "Ready".to_owned());
            send_log(event_tx, format!("Klondike bounded run complete: {verified} verified gameplay action(s); no completion or restart inferred."));
        }
        Ok(RunOutcome::RecoveryOnly) => {


            if let Some(observation) = latest {
                event_tx.publish_frame(observation.frame, observation.prediction, true);
            }
            send_state(event_tx, WorkerState::Ready);
            send_status(event_tx, "Solver refreshed — review next move".to_owned());
            send_log(event_tx, "Klondike Solver recovery produced a new preview. Gameplay inputs sent: 0. Review the preview and press Step Once or Multi-Step again to authorise that action.".to_owned());
        }
        Ok(RunOutcome::SolveRequested { previous_verified }) => {


            if let Some(observation) = latest {
                event_tx.publish_diagnostic_frame(observation.frame);
            }
            send_state(event_tx, WorkerState::Uncertain);
            send_status(event_tx, "Solve requested — inspect result".to_owned());
            send_log(event_tx, format!("Klondike Solve requested once after {previous_verified} independently verified gameplay action(s). The settled result is diagnostic only: Solve effect, win and completion remain unverified. No further input was sent."));
        }
        Err(error) => {


            if let Some(observation) = latest {
                event_tx.publish_diagnostic_frame(observation.frame);
            }
            send_state(event_tx, WorkerState::Uncertain);
            send_status(event_tx, "Klondike stopped".to_owned());
            send_log(event_tx, format!("Klondike stopped: {error}; guest input attempted={input_attempted}. No uncertain gameplay or Solver input was retried; no completion or restart was inferred."));
        }
    }
}


/// Execute the production controller against a QMP or deterministic test adapter.
///
/// The same result frame is reused for the next plan. `latest` is updated only
/// after a successful capture so an observation failure preserves prior pixels.
fn drive_run(
    io: &mut impl KlondikeIo,
    approved_prediction: PredictedAction,
    settings: StepRunSettings,
    event_tx: &WorkerEventSink,
    cancel_requested: &AtomicBool,
    latest: &mut Option<FrameObservation>,
    input_attempted: &mut bool,
) -> Result<RunOutcome, String> {
    validate_operation_limit(settings.operation_limit())?;
    require_running(cancel_requested)?;
    require_klondike_prediction(approved_prediction, true)?;
    let delays = settings.animation_delays();
    let requested_actions = settings.bounded_operation_limit()
        .map_or_else(|| "continuous until STOP, uncertainty or Solve".to_owned(), |limit| format!("{limit} gameplay action(s)"));
    send_log(event_tx, format!(
        "Klondike requested {requested_actions}, settle={} ms, reobserve={} ms. Draw uses qcode D; recycle and card sources use one click. Solver refresh: one per unresolved context, followed by at most {REOBSERVATION_LIMIT} delayed observations; the first capture is immediate. Solve is clicked once, settled and captured, then execution stops with its result unverified.",
        delays.klondike_settle.as_millis(), delays.klondike_reobserve.as_millis(),
    ));
    send_log(event_tx, format!(
        "Klondike input intervals: pointer settle={} ms, gameplay mouse hold={} ms, Draw key hold={} ms. The editable action settle={} ms begins after input acknowledgement; observation time is additional.",
        POINTER_SETTLE_DELAY.as_millis(), MOUSE_HOLD.as_millis(), KEY_HOLD.as_millis(),
        delays.klondike_settle.as_millis(),
    ));
    send_state(event_tx, WorkerState::Validating);
    send_status(event_tx, "Validating Klondike preview".to_owned());
    capture_latest(io, latest, event_tx)?;
    require_scene(latest.as_ref().expect("fresh planning capture"))?;
    let fresh_prediction = latest.as_ref().expect("fresh planning capture").prediction;
    let mut solver_refreshes = 0usize;


    if matches!(fresh_prediction, PredictedAction::NoHighlight) {
        // A changed/missing preview target cannot silently select another move.
        // The explicit run still authorises the user's bounded Solver refresh.
        refresh_solver(io, latest, &mut solver_refreshes, settings, event_tx, cancel_requested, input_attempted)?;
        await_target(io, latest, settings, event_tx, cancel_requested)?;
        return Ok(RunOutcome::RecoveryOnly);
    }


    if fresh_prediction != approved_prediction {
        return Err(format!(
            "fresh target {} differs from approved preview {}; inspect the refreshed frame before another run",
            format_prediction_target(fresh_prediction), format_prediction_target(approved_prediction),
        ));
    }


    let mut verified_operations = 0usize;


    loop {
        require_running(cancel_requested)?;


        if settings.bounded_operation_limit().is_some_and(|limit| verified_operations >= limit) {
            return Ok(RunOutcome::Completed(verified_operations));
        }
        let operation_index = next_counter(verified_operations, "gameplay action")?;
        let observation = latest.as_ref().ok_or_else(|| "Klondike planning frame is missing".to_owned())?;
        require_scene(observation)?;
        require_klondike_prediction(observation.prediction, false)?;
        let plan = plan_step(observation.prediction).map_err(|error| format!("Klondike plan rejected: {error}"))?;
        send_log(event_tx, format_step_plan(plan)?);
        let before = observation.frame.clone();
        let solve_requested = matches!(plan.input().action().target, ActionTarget::Klondike(klondike::KlondikeTarget::Solve));
        io.probe()?;
        require_running(cancel_requested)?;
        send_state(event_tx, WorkerState::Acting);
        send_status(event_tx, format_action_status(plan.before()));
        *input_attempted = true;
        io.input(plan).map_err(|error| format!("gameplay input outcome is uncertain: {error}"))?;
        io.wait(delays.klondike_settle)?;
        send_state(event_tx, WorkerState::Verifying);


        send_status(event_tx, if solve_requested { "Capturing Solve result" } else { "Verifying Klondike move" }.to_owned());


        if solve_requested {
            // The observed control authorises this click, not a predicted end screen.
            // Retain the decoded result before checking late STOP, without analysis.
            let frame = io.capture_diagnostic()?;
            event_tx.publish_diagnostic_frame(frame.clone());
            *latest = Some(diagnostic_observation(frame));
            require_running(cancel_requested)?;
            return Ok(RunOutcome::SolveRequested { previous_verified: verified_operations });
        }
        capture_latest(io, latest, event_tx)?;
        require_running(cancel_requested)?;
        let mut context_refreshed = false;
        let mut delayed_observations = 0usize;


        loop {
            require_running(cancel_requested)?;
            let current = latest.as_ref().expect("fresh result capture");
            require_scene(current)?;
            require_klondike_prediction(current.prediction, true)?;
            let effect_verified = io.effect(&before, &current.frame, plan.input().action())?;


            if matches!(current.prediction, PredictedAction::NoHighlight) && !context_refreshed {
                refresh_solver(io, latest, &mut solver_refreshes, settings, event_tx, cancel_requested, input_attempted)?;
                context_refreshed = true;
                continue;
            }


            if effect_verified && matches!(current.prediction, PredictedAction::Action(_)) {
                let changed_pixels = materially_changed_pixels(
                    &before, &current.frame, plan.input().effect_bounds(),
                    plan.input().effect_exclusion_bounds(), ACTION_CHANGE_CHANNEL_THRESHOLD,
                )?;
                let _ = event_tx.send(WorkerEvent::ActionCompleted {
                    operation_index,
                    operation_limit: settings.operation_limit(),
                    before: plan.before(),
                    after: current.prediction,
                    input_commands: plan.input().qmp_command_count(),
                    input_events: plan.input().qmp_event_count(),
                    changed_pixels,
                    continued_from_halo: false,
                });
                let run_bound = settings.bounded_operation_limit()
                    .map_or_else(|| "continuous".to_owned(), |limit| limit.to_string());
                send_log(event_tx, format!(
                    "Klondike action {operation_index}/{run_bound} effect verified; {} -> {}; changed effect pixels={changed_pixels}; Solver refreshes={solver_refreshes}. Fresh HALO alone never verifies the previous action.",
                    format_prediction_target(plan.before()), format_prediction_target(current.prediction),
                ));
                verified_operations = operation_index;
                break;
            }


            if delayed_observations >= REOBSERVATION_LIMIT {
                return Err(format!(
                    "bounded result observations exhausted after {delayed_observations} delayed captures; previous effect verified={effect_verified}, next target={}; gameplay input was sent once",
                    format_prediction_target(current.prediction),
                ));
            }
            delayed_observations += 1;
            io.wait(delays.klondike_reobserve)?;
            capture_latest(io, latest, event_tx)?;
        }
    }
}


/// Advance a cumulative diagnostic counter without permitting wraparound during continuous play.
fn next_counter(current: usize, label: &str) -> Result<usize, String> {
    current.checked_add(1)
        .ok_or_else(|| format!("Klondike {label} counter exhausted; stopping before further input"))
}


/// Send at most one Solver click per unresolved context and capture immediately.
///
/// The caller controls its per-context flag; finite runs also enforce a total bound.
/// Continuous runs retain checked cumulative counters and bounded per-context recovery.
/// Fresh scene, cancellation and tablet probes precede input.
fn refresh_solver(
    io: &mut impl KlondikeIo,
    latest: &mut Option<FrameObservation>,
    refreshes: &mut usize,
    settings: StepRunSettings,
    event_tx: &WorkerEventSink,
    cancel_requested: &AtomicBool,
    input_attempted: &mut bool,
) -> Result<(), String> {
    let current = latest.as_ref().ok_or_else(|| "Solver refresh has no fresh frame".to_owned())?;
    require_scene(current)?;


    if !matches!(current.prediction, PredictedAction::NoHighlight) {
        return Err("Solver refresh requires a fresh no-HALO classification".to_owned());
    }


    if settings.bounded_operation_limit().is_some_and(|limit| *refreshes > limit) {
        return Err("Klondike total Solver-refresh budget exhausted".to_owned());
    }
    let next_refresh = next_counter(*refreshes, "Solver refresh")?;
    require_running(cancel_requested)?;
    io.probe()?;
    require_running(cancel_requested)?;
    *refreshes = next_refresh;
    *input_attempted = true;
    send_state(event_tx, WorkerState::Acting);
    send_status(event_tx, "Refreshing Klondike Solver".to_owned());
    io.solver()?;
    send_log(event_tx, format!("Klondike Solver refresh {refreshes}: acknowledged once; capturing immediately, with no post-click settle delay."));
    send_state(event_tx, WorkerState::Verifying);
    capture_latest(io, latest, event_tx)?;
    Ok(())
}


/// Observe a recovered initial target without sending gameplay or another Solver click.
fn await_target(
    io: &mut impl KlondikeIo,
    latest: &mut Option<FrameObservation>,
    settings: StepRunSettings,
    event_tx: &WorkerEventSink,
    cancel_requested: &AtomicBool,
) -> Result<(), String> {


    for attempt in 0..=REOBSERVATION_LIMIT {
        require_running(cancel_requested)?;
        let current = latest.as_ref().ok_or_else(|| "Solver recovery has no captured frame".to_owned())?;
        require_scene(current)?;
        require_klondike_prediction(current.prediction, true)?;


        if matches!(current.prediction, PredictedAction::Action(_)) {
            return Ok(());
        }


        if attempt == REOBSERVATION_LIMIT {
            break;
        }
        io.wait(settings.animation_delays().klondike_reobserve)?;
        capture_latest(io, latest, event_tx)?;
    }
    Err("Solver refresh produced no target within its bounded input-free observations".to_owned())
}


/// Classify decoded pixels while preserving them if analysis fails or STOP arrives late.
///
/// Classification owns its frame, so one temporary clone retains the original
/// pixels for the error path. This boundary is local to Klondike.
fn classify_captured_result(
    frame: CapturedFrame,
    scan_state: &TableauScanState,
    cancel_requested: &AtomicBool,
) -> CaptureResult {


    if let Err(reason) = require_running(cancel_requested) {
        return CaptureResult::Diagnostic { frame, reason };
    }
    let analysed = analyse_captured_frame(frame.clone(), scan_state);


    match analysed {
        Ok((observation, _detection)) => {


            if let Err(reason) = require_running(cancel_requested) {
                CaptureResult::Diagnostic { frame, reason }
            } else {
                CaptureResult::Classified(observation)
            }
        }
        Err(reason) => CaptureResult::Diagnostic { frame, reason },
    }
}


/// Wrap diagnostic pixels without a prediction, gameplay scene or completion authority.
fn diagnostic_observation(frame: CapturedFrame) -> FrameObservation {
    FrameObservation {
        frame,
        prediction: PredictedAction::NoHighlight,
        observed_rows: None,
        gameplay_scene: false,
        game_progress: None,
    }
}


/// Retain and publish every decoded frame before propagating analysis or late STOP errors.
///
/// Acquisition/decode failures leave the prior frame intact because no newer
/// usable pixels exist. Diagnostic results never grant input authority.
fn capture_latest(
    io: &mut impl KlondikeIo,
    latest: &mut Option<FrameObservation>,
    event_tx: &WorkerEventSink,
) -> Result<(), String> {


    let (observation, result) = match io.capture()? {
        CaptureResult::Classified(observation) => (observation, Ok(())),
        CaptureResult::Diagnostic { frame, reason } => (diagnostic_observation(frame), Err(reason)),
    };
    event_tx.publish_diagnostic_frame(observation.frame.clone());
    *latest = Some(observation);
    result
}


/// Refuse unknown scenes before both gameplay and Solver recovery.
fn require_scene(observation: &FrameObservation) -> Result<(), String> {


    if !observation.gameplay_scene {
        return Err("unsupported Klondike scene; latest frame retained for inspection".to_owned());
    }
    Ok(())
}


/// Reject ambiguous targets, other modes and fabricated plans before input.
fn require_klondike_prediction(prediction: PredictedAction, allow_missing: bool) -> Result<(), String> {


    match prediction {
        PredictedAction::Action(action) if matches!(action.target, ActionTarget::Klondike(_)) => {
            plan_step(prediction).map(|_| ()).map_err(|error| format!("Klondike target rejected: {error}"))
        }
        PredictedAction::NoHighlight if allow_missing => Ok(()),
        _ => Err(format!("Klondike prediction has no input authority: {}", format_prediction_target(prediction))),
    }
}


/// Check STOP at controller boundaries as well as inside real I/O operations.
fn require_running(cancel_requested: &AtomicBool) -> Result<(), String> {


    if cancel_requested.load(Ordering::Acquire) {
        return Err("STOP was requested".to_owned());
    }
    Ok(())
}


#[cfg(test)]
mod tests;
