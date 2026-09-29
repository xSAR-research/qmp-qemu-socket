//! Bounded Klondike execution and one-shot Solver refresh from fresh scene evidence.
//!
//! Gameplay actions, Solver refreshes and read-only observations have separate
//! finite budgets. An uncertain gameplay input is never replayed. This module
//! deliberately has no game-completion, restart or three-board progression path.

use std::{path::Path, sync::atomic::{AtomicBool, Ordering}, time::Duration};

use super::{
    FrameObservation, WorkerEvent, WorkerEventSink, WorkerState, capture_series,
    connect_and_probe, execute_planned_input, format_action_status, format_prediction_target,
    format_step_plan, materially_changed_pixels, send_log, send_state,
    send_status, validate_probe, wait_or_stop,
};
use crate::{
    capture::CapturedFrame,
    game::{ActionTarget, GuidedAction},
    klondike,
    parameters::{
        ACTION_CHANGE_CHANNEL_THRESHOLD, KLONDIKE_MAX_MULTI_STEP_ACTIONS,
        NOMINAL_FRAME_HEIGHT, NOMINAL_FRAME_WIDTH, SOLVER_MOUSE_HOLD, StepRunSettings,
    },
    qmp::QmpClient,
    stepper::{StepPlan, plan_step},
    tracker::{PredictedAction, TableauScanState},
};


/// Maximum delayed observations after an immediate result or Solver capture.
const REOBSERVATION_LIMIT: usize = 3;


/// Validate the finite gameplay-action budget before connecting or clearing STOP.
pub(super) fn validate_operation_limit(limit: usize) -> Result<(), String> {


    if !(1..=KLONDIKE_MAX_MULTI_STEP_ACTIONS).contains(&limit) {
        return Err(format!(
            "Klondike requires 1..={KLONDIKE_MAX_MULTI_STEP_ACTIONS} gameplay actions; requested {limit}"
        ));
    }
    Ok(())
}


/// Private boundary for deterministic controller tests using the production state machine.
trait KlondikeIo {
    /// Acquire and classify exactly one new frame, checking cancellation.
    fn capture(&mut self) -> Result<FrameObservation, String>;

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
        klondike::verify_effect(before, after, action)
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
}


impl KlondikeIo for QmpKlondikeIo<'_> {


    fn capture(&mut self) -> Result<FrameObservation, String> {
        capture_series(self.qmp, self.socket_path, self.scan_state, self.cancel_requested)?
            .observations
            .pop()
            .ok_or_else(|| "Klondike capture returned no frame".to_owned())
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
        execute_planned_input(self.qmp, plan, self.cancel_requested)
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
}


/// Successful endpoint: a bounded action count or a recovery-only fresh preview.
#[derive(Debug, PartialEq, Eq)]
enum RunOutcome {
    /// All requested gameplay actions have independently verified effects.
    Completed(usize),
    /// Solver refresh produced a preview that requires another explicit user run.
    RecoveryOnly,
}


/// Run Klondike under an independent finite policy using shared capture and QMP facilities.
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
        let mut io = QmpKlondikeIo { qmp: &mut qmp, socket_path, scan_state, cancel_requested };
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
    send_log(event_tx, format!(
        "Klondike requested {} gameplay action(s), settle={} ms, reobserve={} ms. Draw uses qcode D; recycle and card sources use one click. Solver refresh: one per unresolved context, at most {} per run; next capture is immediate, without a deliberate settle delay.",
        settings.operation_limit(), delays.klondike_settle.as_millis(),
        delays.klondike_reobserve.as_millis(), settings.operation_limit() + 1,
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


    for operation_index in 1..=settings.operation_limit() {
        require_running(cancel_requested)?;
        let observation = latest.as_ref().ok_or_else(|| "Klondike planning frame is missing".to_owned())?;
        require_scene(observation)?;
        require_klondike_prediction(observation.prediction, false)?;
        let plan = plan_step(observation.prediction).map_err(|error| format!("Klondike plan rejected: {error}"))?;
        send_log(event_tx, format_step_plan(plan)?);
        let before = observation.frame.clone();
        io.probe()?;
        require_running(cancel_requested)?;
        send_state(event_tx, WorkerState::Acting);
        send_status(event_tx, format_action_status(plan.before()));
        *input_attempted = true;
        io.input(plan).map_err(|error| format!("gameplay input outcome is uncertain: {error}"))?;
        io.wait(delays.klondike_settle)?;
        send_state(event_tx, WorkerState::Verifying);
        send_status(event_tx, "Verifying Klondike move".to_owned());
        capture_latest(io, latest, event_tx)?;
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
                send_log(event_tx, format!(
                    "Klondike action {operation_index}/{} effect verified; {} -> {}; changed effect pixels={changed_pixels}; Solver refreshes={solver_refreshes}. Fresh HALO alone never verifies the previous action.",
                    settings.operation_limit(), format_prediction_target(plan.before()),
                    format_prediction_target(current.prediction),
                ));
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
    Ok(RunOutcome::Completed(settings.operation_limit()))
}


/// Send at most one Solver click per unresolved context and capture immediately.
///
/// The caller controls its per-context flag; the additional total bound guards
/// accidental future calls. Fresh scene, cancellation and tablet probes precede input.
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


    if *refreshes > settings.operation_limit() {
        return Err("Klondike total Solver-refresh budget exhausted".to_owned());
    }
    require_running(cancel_requested)?;
    io.probe()?;
    require_running(cancel_requested)?;
    *refreshes += 1;
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


/// Publish every fresh observation for responsive preview without granting new input authority.
///
/// A failed capture leaves the prior observation intact. The single-frame mailbox
/// coalesces previews, while the run retains its exact source/result frames.
fn capture_latest(
    io: &mut impl KlondikeIo,
    latest: &mut Option<FrameObservation>,
    event_tx: &WorkerEventSink,
) -> Result<(), String> {
    *latest = Some(io.capture()?);


    if let Some(observation) = latest.as_ref() {
        event_tx.publish_diagnostic_frame(observation.frame.clone());
    }
    Ok(())
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
