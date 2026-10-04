//! Klondike execution with bounded recovery and one-shot Solve requests.
//!
//! Zero requests continuous gameplay until STOP, uncertainty or a confirmed game win.
//! Every unresolved context has finite Solver-refresh and observation budgets.
//! An uncertain input is never replayed. Every acknowledged gameplay action
//! consumes one budget slot, then settles and produces a fresh observation.
//! A canonical fresh Solver HALO authorises the next action, even at the same
//! position; no card matching or pixel-difference threshold gates play. HALOs
//! never prove the preceding effect, a card transfer or game completion.
//! Solve is requested once and followed by
//! bounded read-only completion observations. One board is one Klondike game;
//! completion requires two fresh positive observations. Only an authorised continuous
//! run may advance through the separately calibrated terminal controls.
//! The displayed preview is advisory. Fresh mode-owned scene/target evidence
//! authorises input for both finite and continuous runs. A missing HALO receives
//! bounded input-free recaptures and independent completion checks before one
//! Solver refresh on a positively recognised gameplay scene. The same unresolved
//! context cannot refresh Solver again; uncertainty and unknown scenes stop.
//! Strong, incomplete Solve artwork defers lower tableau/foundation input through
//! at most three input-free observations. It never weakens Solve click authority.

use std::{path::Path, sync::atomic::{AtomicBool, Ordering}, time::{Duration, Instant}};

use super::{
    FrameObservation, WorkerEvent, WorkerEventSink, WorkerState, analyse_captured_frame, capture_screen,
    connect_and_probe, execute_planned_input, format_action_status, format_prediction_target,
    format_step_plan, milliseconds, send_log, send_state,
    send_status, validate_probe, wait_or_stop,
};
use crate::{
    capture::CapturedFrame,
    game::{ActionTarget, GuidedAction},
    klondike,
    parameters::{
        KEY_HOLD, KLONDIKE_MAX_MULTI_STEP_ACTIONS, MOUSE_HOLD,
        NOMINAL_FRAME_HEIGHT, NOMINAL_FRAME_WIDTH, POINTER_SETTLE_DELAY, POST_GAME_MAX_OBSERVATION_ROUNDS,
        SOLVER_MOUSE_HOLD, StepRunSettings,
    },
    qmp::QmpClient,
    stepper::{StepPlan, plan_step},
    tracker::{PredictedAction, TableauScanState},
};


/// Maximum delayed observations after an immediate result or Solver capture.
const REOBSERVATION_LIMIT: usize = 3;


/// An acknowledged Solve starts the existing bounded post-game observation budget.
/// Card recovery retains its shorter limit; no new timing or input is introduced.
const SOLVE_COMPLETION_REOBSERVATION_LIMIT: usize = POST_GAME_MAX_OBSERVATION_ROUNDS;


/// Why the worker is observing completion without sending further input.
#[derive(Clone, Copy)]
enum CompletionWait {
    /// An ordinary observation already supplied the first positive candidate.
    ConfirmCandidate,
    /// Solve was acknowledged once; animation may precede any win evidence.
    AfterSolve,
}


/// Cumulative diagnostics and the one-refresh budget of an unresolved context.
#[derive(Default)]
struct SolverRecovery {
    /// Checked cumulative count; continuous play cannot wrap this counter.
    refreshes: usize,
    /// True after one refresh until an action is acknowledged or a fresh restart completes.
    refreshed: bool,
}


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


    /// Acquire result pixels without action classification or discarding a late STOP frame.
    fn capture_diagnostic(&mut self) -> Result<CapturedFrame, String>;


    /// Read mode-owned completion evidence without granting any guest input authority.
    fn completion(&mut self, frame: &CapturedFrame) -> Result<bool, String> {
        klondike::completion_evidence(frame)
            .map(|evidence| evidence.complete_candidate)
            .map_err(|error| format!("Klondike completion analysis failed: {error}"))
    }


    /// Revalidate VM state and the current QEMU HID Tablet immediately before input.
    fn probe(&mut self) -> Result<(), String>;


    /// Deliver the planned gameplay input once; errors make its outcome uncertain.
    fn input(&mut self, plan: StepPlan) -> Result<(), String>;


    /// Deliver one authorised Solver click; do not add a post-click settle delay.
    fn solver(&mut self) -> Result<(), String>;


    /// Classify a terminal stage from fresh pixels using Klondike's own calibration.
    fn terminal_stage(&mut self, frame: &CapturedFrame) -> Result<Option<klondike::terminal::TerminalStage>, String> {
        klondike::terminal::classify_terminal(frame)
            .map_err(|error| format!("Klondike terminal analysis failed: {error}"))
    }


    /// Send exactly one click on an independently recognised terminal control.
    fn terminal_input(&mut self, stage: klondike::terminal::TerminalStage) -> Result<(), String>;


    /// Wait in cancellable slices when ordinary gameplay or re-observation needs settling.
    fn wait(&mut self, duration: Duration) -> Result<(), String>;
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
    /// Bounded UI output and complete session log for scene and input diagnostics.
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


        if let CaptureResult::Classified(observation) = &result {


            match klondike::inspect_solve_control(&observation.frame) {
                Ok(evidence) => send_log(self.event_tx, format!(
                    "Klondike Solve evidence: {evidence}; gameplay scene={}; selected={}",
                    observation.gameplay_scene, format_prediction_target(observation.prediction),
                )),
                Err(error) => send_log(self.event_tx, format!("Klondike Solve diagnostic unavailable: {error}")),
            }
        }
        Ok(result)
    }


    fn capture_diagnostic(&mut self) -> Result<CapturedFrame, String> {
        require_running(self.cancel_requested)?;
        capture_screen(self.qmp, self.socket_path).map(|(frame, _timing)| frame)
    }


    fn completion(&mut self, frame: &CapturedFrame) -> Result<bool, String> {
        let evidence = klondike::completion_evidence(frame)
            .map_err(|error| format!("Klondike completion analysis failed: {error}"))?;
        send_log(self.event_tx, format!("Klondike completion evidence: {evidence}"));
        Ok(evidence.complete_candidate)
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


    fn terminal_stage(&mut self, frame: &CapturedFrame) -> Result<Option<klondike::terminal::TerminalStage>, String> {
        let stage = klondike::terminal::classify_terminal(frame)
            .map_err(|error| format!("Klondike terminal analysis failed: {error}"))?;


        match klondike::terminal::inspect_terminal_evidence(frame) {
            Ok(evidence) => send_log(self.event_tx, format!(
                "Klondike terminal evidence: {evidence}; selected={stage:?}; guard counts do not grant input authority",
            )),
            Err(error) => send_log(self.event_tx, format!("Klondike terminal diagnostic unavailable: {error}")),
        }
        Ok(stage)
    }


    fn terminal_input(&mut self, stage: klondike::terminal::TerminalStage) -> Result<(), String> {
        require_running(self.cancel_requested)?;
        self.qmp.click_with_hold(
            stage.click_point(), NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT, stage.mouse_hold(),
        ).map_err(|error| format!("Klondike {stage} input outcome is uncertain: {error}; no retry is permitted"))
    }


    fn wait(&mut self, duration: Duration) -> Result<(), String> {
        wait_or_stop(
            duration,
            self.cancel_requested,
            "STOP was requested during Klondike settling; no input was retried",
        ).map(|_| ())
    }
}


/// Explicit endpoint separating acknowledged actions from independent completion.
#[derive(Debug, PartialEq, Eq)]
enum RunOutcome {
    /// The finite action budget was consumed without implying game completion.
    Completed {
        /// Zero under Solver-led play; HALOs never establish a previous effect.
        verified: usize,
        /// Acknowledged actions selected from a freshly classified Solver target.
        halo: usize,
    },
    /// One board is one game, confirmed by two fresh mode-owned completion observations.
    GameWon {
        /// Zero under Solver-led play; completion does not prove earlier card effects.
        previous_verified: usize,
        /// Previous actions accepted from fresh HALO or new Solve control evidence.
        previous_halo: usize,
    },
    /// Solve was requested once; bounded observations did not establish completion.
    SolveRequested {
        /// Zero under Solver-led play; a Solve request does not prove earlier effects.
        previous_verified: usize,
        /// Previous actions accepted from fresh HALO or new Solve control evidence.
        previous_halo: usize,
    },
}


/// Run finite or continuous Klondike play using shared capture and QMP facilities.
///
/// Latest frames are retained on every stop. Unknown scenes cannot authorise
/// input. Missing HALOs alone cannot establish completion or restart; an
/// authorised continuous run may advance a positively verified one-board game.
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
        Ok(RunOutcome::Completed { verified, halo }) => {


            if let Some(observation) = latest {
                let prediction = observation.prediction;
                event_tx.publish_frame(observation.frame, prediction, false);
                let _ = event_tx.send(WorkerEvent::RunCompleted {
                    prediction,
                    requested_operations: settings.operation_limit(),
                    verified_operations: verified,
                    halo_operations: halo,
                });
            }
            send_state(event_tx, WorkerState::Ready);
            send_status(event_tx, "Ready".to_owned());
            send_log(event_tx, format!("Klondike bounded run complete: {halo} acknowledged gameplay action(s) from fresh Solver targets; independently verified effects={verified}. The final fresh frame is retained; no completion or restart inferred."));
        }
        Ok(RunOutcome::GameWon { previous_verified, previous_halo }) => {


            if let Some(observation) = latest {
                event_tx.publish_diagnostic_frame(observation.frame);
            }
            send_state(event_tx, WorkerState::Ready);
            send_status(event_tx, "Klondike game won — no automatic restart".to_owned());
            send_log(event_tx, format!("Klondike GAME WIN confirmed by two fresh mode-owned completion observations after {previous_verified} verified gameplay action(s) and {previous_halo} fresh-evidence continuations. One Klondike board equals one game. This endpoint sent no terminal, restart or new-game input."));
        }
        Ok(RunOutcome::SolveRequested { previous_verified, previous_halo }) => {


            if let Some(observation) = latest {
                event_tx.publish_diagnostic_frame(observation.frame);
            }
            send_state(event_tx, WorkerState::Uncertain);
            send_status(event_tx, "Solve requested — inspect result".to_owned());
            send_log(event_tx, format!("Klondike Solve requested once after {previous_verified} independently verified gameplay action(s) and {previous_halo} fresh-evidence continuations. Completion was not established within the bounded read-only observations; the latest result is diagnostic only. No further guest input was sent."));
        }
        Err(error) => {


            if let Some(observation) = latest {
                event_tx.publish_diagnostic_frame(observation.frame);
            }
            send_state(event_tx, WorkerState::Uncertain);
            send_status(event_tx, "Klondike stopped".to_owned());
            send_log(event_tx, format!("Klondike stopped: {error}; guest input attempted={input_attempted}. No uncertain gameplay, Solver or terminal input was retried; the latest available frame was retained for inspection."));
        }
    }
}


/// Execute Solver-led play against a QMP or deterministic test adapter.
///
/// The preview supplies the selected-mode boundary, not a frozen card coordinate.
/// Every input uses the latest canonical mode-owned target, then settles and
/// captures anew. Acknowledgements consume the finite budget; HALOs do not prove
/// previous effects. Missing targets have bounded read-only recovery and at most
/// one Solver refresh per unresolved context. The latest decoded frame survives
/// STOP, uncertainty and all analysis failures.
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
        .map_or_else(|| "continuous until STOP, uncertainty or game win".to_owned(), |limit| format!("{limit} gameplay action(s)"));
    send_log(event_tx, format!(
        "Klondike requested {requested_actions}, settle={} ms, Solve animation settle={} ms, reobserve={} ms. The preview is advisory: each input requires a fresh supported scene and canonical Solver target. Draw uses qcode D; recycle and card sources use one click. Each acknowledged gameplay action consumes one budget slot; card matching and changed-pixel thresholds are not used. Missing HALOs receive at most {REOBSERVATION_LIMIT} delayed input-free captures, then independent completion review, then at most one Solver refresh on a positively recognised gameplay scene. Solver captures immediately and waits at most {REOBSERVATION_LIMIT} further delayed observations. Solve remains one click plus up to {SOLVE_COMPLETION_REOBSERVATION_LIMIT} delayed completion captures; two consecutive positives establish one-board game win. Only continuous Multi-Step 0 advances recognised terminal controls.",
        delays.klondike_settle.as_millis(), delays.klondike_solve.as_millis(), delays.klondike_reobserve.as_millis(),
    ));
    send_log(event_tx, format!(
        "Klondike input intervals: pointer settle={} ms, gameplay mouse hold={} ms, Draw key hold={} ms. The editable action settle={} ms begins after input acknowledgement; observation time is additional.",
        POINTER_SETTLE_DELAY.as_millis(), MOUSE_HOLD.as_millis(), KEY_HOLD.as_millis(),
        delays.klondike_settle.as_millis(),
    ));
    send_state(event_tx, WorkerState::Validating);
    send_status(event_tx, "Observing fresh Klondike Solver target".to_owned());
    capture_latest(io, latest, event_tx)?;
    let mut recovery = SolverRecovery::default();
    let mut completed_operations = 0usize;
    let mut halo_operations = 0usize;


    loop {
        require_running(cancel_requested)?;


        if settings.bounded_operation_limit().is_some_and(|limit| completed_operations >= limit) {
            return Ok(RunOutcome::Completed { verified: 0, halo: halo_operations });
        }


        if await_playable_context(
            io, latest, &mut recovery, settings,
            event_tx, cancel_requested, input_attempted,
        )? {


            if settings.is_unbounded() {
                resume_after_win(io, latest, settings, event_tx, cancel_requested, input_attempted)?;
                recovery.refreshed = false;
                continue;
            }
            return Ok(RunOutcome::GameWon { previous_verified: 0, previous_halo: halo_operations });
        }
        await_pending_solve(io, latest, settings, event_tx, cancel_requested)?;
        let observation = latest.as_ref().ok_or_else(|| "Klondike planning frame is missing".to_owned())?;
        require_scene(observation)?;
        require_klondike_prediction(observation.prediction, true)?;


        if matches!(observation.prediction, PredictedAction::NoHighlight) {
            // Pending Solve recaptures may replace an actionable planning frame.
            // The existing context flag prevents a second Solver refresh.
            continue;
        }
        let operation_index = next_counter(completed_operations, "gameplay action")?;
        let plan = plan_step(observation.prediction).map_err(|error| format!("Klondike plan rejected: {error}"))?;
        send_log(event_tx, format_step_plan(plan)?);
        let solve_requested = matches!(plan.input().action().target, ActionTarget::Klondike(klondike::KlondikeTarget::Solve));
        io.probe()?;
        require_running(cancel_requested)?;
        send_state(event_tx, WorkerState::Acting);
        send_status(event_tx, format_action_status(plan.before()));
        *input_attempted = true;
        io.input(plan).map_err(|error| format!("gameplay input outcome is uncertain: {error}"))?;
        completed_operations = operation_index;
        recovery.refreshed = false;
        io.wait(if solve_requested { delays.klondike_solve } else { delays.klondike_settle })?;
        send_state(event_tx, WorkerState::Verifying);
        send_status(event_tx, if solve_requested { "Capturing Solve result" } else { "Observing next Klondike Solver target" }.to_owned());


        if solve_requested {
            // Only independent completion evidence can advance this one-shot input.
            let complete = await_completion(
                io, latest, settings, event_tx, cancel_requested, CompletionWait::AfterSolve,
            )?;


            if complete && settings.is_unbounded() {
                resume_after_win(io, latest, settings, event_tx, cancel_requested, input_attempted)?;
                continue;
            }
            return Ok(if complete {
                RunOutcome::GameWon { previous_verified: 0, previous_halo: halo_operations }
            } else {
                RunOutcome::SolveRequested { previous_verified: 0, previous_halo: halo_operations }
            });
        }
        capture_latest(io, latest, event_tx)?;
        require_running(cancel_requested)?;
        halo_operations = next_counter(halo_operations, "acknowledged Solver action")?;
        report_acknowledged_action(
            event_tx, latest.as_ref().expect("fresh acknowledged-action result"), plan,
            operation_index, settings, recovery.refreshes,
        );
    }
}


/// Resolve a missing target without treating missing pixels as input authority.
///
/// The current frame and at most three delayed observations are read-only. Only
/// independent completion can establish a win. Otherwise one Solver refresh is
/// authorised by a positively recognised gameplay scene; its immediate capture
/// and bounded delayed observations cannot authorise another refresh. The caller
/// resets the context only after an acknowledged gameplay action or a positively
/// recognised fresh board at the end of a confirmed game restart.
fn await_playable_context(
    io: &mut impl KlondikeIo,
    latest: &mut Option<FrameObservation>,
    recovery: &mut SolverRecovery,
    settings: StepRunSettings,
    event_tx: &WorkerEventSink,
    cancel_requested: &AtomicBool,
    input_attempted: &mut bool,
) -> Result<bool, String> {


    if await_fresh_target(io, latest, settings, event_tx, cancel_requested)? {
        return Ok(false);
    }


    if io.completion(&latest.as_ref().expect("bounded planning observation").frame)? {


        if await_completion(io, latest, settings, event_tx, cancel_requested, CompletionWait::ConfirmCandidate)? {
            return Ok(true);
        }
        return Err("completion candidate was not confirmed within bounded read-only observations; no gameplay or Solver input retried".to_owned());
    }
    require_scene(latest.as_ref().expect("bounded planning observation"))?;


    if recovery.refreshed {
        return Err("Klondike unresolved context already refreshed Solver once; no further input authorised".to_owned());
    }
    recovery.refreshed = true;
    refresh_solver(io, latest, &mut recovery.refreshes, settings, event_tx, cancel_requested, input_attempted)?;


    if await_fresh_target(io, latest, settings, event_tx, cancel_requested)? {
        return Ok(false);
    }


    if io.completion(&latest.as_ref().expect("bounded Solver observation").frame)? {


        if await_completion(io, latest, settings, event_tx, cancel_requested, CompletionWait::ConfirmCandidate)? {
            return Ok(true);
        }
        return Err("completion after Solver refresh was not confirmed within bounded read-only observations; no further input authorised".to_owned());
    }
    require_scene(latest.as_ref().expect("bounded Solver observation"))?;
    Err("Solver refresh produced no fresh canonical target within its bounded input-free observations; no further Solver or gameplay input authorised".to_owned())
}


/// Observe a canonical fresh Solver target, allowing at most three delayed captures.
/// A supported repeated HALO is a new action recommendation, not prior effect proof.
/// Unknown scenes receive only read-only observations and cannot grant input.
fn await_fresh_target(
    io: &mut impl KlondikeIo,
    latest: &mut Option<FrameObservation>,
    settings: StepRunSettings,
    event_tx: &WorkerEventSink,
    cancel_requested: &AtomicBool,
) -> Result<bool, String> {


    for delayed in 0..=REOBSERVATION_LIMIT {
        require_running(cancel_requested)?;
        let current = latest.as_ref().ok_or_else(|| "Klondike planning context has no fresh frame".to_owned())?;
        require_klondike_prediction(current.prediction, true)?;


        if current.gameplay_scene && matches!(current.prediction, PredictedAction::Action(_)) {
            return Ok(true);
        }


        if delayed == REOBSERVATION_LIMIT {
            return Ok(false);
        }
        send_state(event_tx, WorkerState::Verifying);
        send_status(event_tx, "Waiting for fresh Klondike Solver target".to_owned());
        send_log(event_tx, format!(
            "Klondike target re-observation {}/{REOBSERVATION_LIMIT}: scene={}, target={}; waiting {} ms before a fresh input-free capture; no gameplay, Solver or terminal input authorised",
            delayed + 1, current.gameplay_scene, format_prediction_target(current.prediction),
            settings.animation_delays().klondike_reobserve.as_millis(),
        ));
        io.wait(settings.animation_delays().klondike_reobserve)?;
        capture_latest(io, latest, event_tx)?;
    }
    Ok(false)
}


/// Defer lower card input while strong Solve lettering awaits its existing face guard.
/// Any preceding action has already been acknowledged and counted before this
/// planning context starts. DRAW, RIGHT and a fully recognised Solve retain priority. Each
/// recapture replaces the planning frame. Disappearing candidates grant no immediate
/// input: missing-HALO frames return to bounded recovery without resetting its
/// one-refresh flag. Unknown scenes and malformed storage stop.
fn await_pending_solve(
    io: &mut impl KlondikeIo,
    latest: &mut Option<FrameObservation>,
    settings: StepRunSettings,
    event_tx: &WorkerEventSink,
    cancel_requested: &AtomicBool,
) -> Result<(), String> {
    let mut delayed_observations = 0usize;


    loop {
        require_running(cancel_requested)?;
        let current = latest.as_ref().ok_or_else(|| "Klondike Solve settling has no fresh planning frame".to_owned())?;
        require_scene(current)?;
        require_klondike_prediction(current.prediction, true)?;


        if !matches!(current.prediction, PredictedAction::Action(GuidedAction {
            target: ActionTarget::Klondike(klondike::KlondikeTarget::Tableau { .. }
                | klondike::KlondikeTarget::Foundation { .. }), ..
        })) {
            return Ok(());
        }
        let evidence = klondike::inspect_solve_control(&current.frame)
            .map_err(|error| format!("Klondike pending Solve analysis failed: {error}"))?;


        if !evidence.awaiting_settle() {
            return Ok(());
        }


        if delayed_observations == REOBSERVATION_LIMIT {
            return Err(format!(
                "pending Klondike Solve remained below its input guard after {delayed_observations} delayed input-free planning captures; no lower-priority card or Solve input authorised",
            ));
        }
        send_state(event_tx, WorkerState::Verifying);
        send_status(event_tx, "Waiting for Klondike Solve artwork".to_owned());
        send_log(event_tx, format!(
            "Klondike pending Solve re-observation {}/{REOBSERVATION_LIMIT}: {evidence}; lower-priority target {} deferred; waiting {} ms; guest input sent=0",
            delayed_observations + 1, format_prediction_target(current.prediction),
            settings.animation_delays().klondike_reobserve.as_millis(),
        ));
        delayed_observations += 1;
        io.wait(settings.animation_delays().klondike_reobserve)?;
        capture_latest(io, latest, event_tx)?;
    }
}


/// Advance an authorised continuous run through one-board terminal controls once.
///
/// Each control needs fresh typed scene evidence, a new VM/tablet probe and STOP
/// check. Its single acknowledged input is followed by settling and bounded
/// read-only advancement checks. Only a freshly classified actionable Solver
/// board completes the cycle; no terminal input is retried.
fn resume_after_win(
    io: &mut impl KlondikeIo,
    latest: &mut Option<FrameObservation>,
    settings: StepRunSettings,
    event_tx: &WorkerEventSink,
    cancel_requested: &AtomicBool,
    input_attempted: &mut bool,
) -> Result<(), String> {
    let mut stage = await_terminal_stage(io, latest, settings, event_tx, cancel_requested, None)?;


    while let Some(current) = stage {
        require_running(cancel_requested)?;
        io.probe()?;
        require_running(cancel_requested)?;
        send_state(event_tx, WorkerState::Acting);
        send_status(event_tx, format!("Klondike {current}"));
        send_log(event_tx, format!("Klondike GAME WIN terminal control {current}: one fresh recognised click at {:?}; no retry is authorised", current.click_point()));
        *input_attempted = true;
        io.terminal_input(current)?;
        io.wait(current.settle_delay(settings.animation_delays().board_redeal))?;
        send_state(event_tx, WorkerState::Verifying);
        capture_latest(io, latest, event_tx)?;
        require_running(cancel_requested)?;
        stage = await_terminal_stage(io, latest, settings, event_tx, cancel_requested, Some(current))?;
    }
    send_log(event_tx, "Klondike one-board terminal cycle complete: the next Draw 1 Solver board was freshly classified with a canonical actionable target. Continuous gameplay resumes from that result frame.".to_owned());
    require_running(cancel_requested)?;
    let _ = event_tx.send(WorkerEvent::KlondikeGameCompleted);
    Ok(())
}


/// Wait input-free for the expected next terminal stage or the new Solver board.
///
/// The current fresh frame counts as the initial observation. The prior stage
/// must advance in order; unexpected recognised controls stop immediately rather
/// than authorising a click. Unknown transient frames and an unchanged prior
/// stage receive at most three delayed observations.
fn await_terminal_stage(
    io: &mut impl KlondikeIo,
    latest: &mut Option<FrameObservation>,
    settings: StepRunSettings,
    event_tx: &WorkerEventSink,
    cancel_requested: &AtomicBool,
    previous: Option<klondike::terminal::TerminalStage>,
) -> Result<Option<klondike::terminal::TerminalStage>, String> {
    use klondike::terminal::TerminalStage;
    let expected = previous.map(|stage| match stage {
        TerminalStage::ScoreCounting => Some(TerminalStage::LevelUp),
        TerminalStage::LevelUp => Some(TerminalStage::NewGame),
        TerminalStage::NewGame => Some(TerminalStage::Play),
        TerminalStage::Play => Some(TerminalStage::SolverReady),
        TerminalStage::SolverReady => None,
    });
    let mut delayed_observations = 0usize;


    loop {
        require_running(cancel_requested)?;
        let current = latest.as_ref().ok_or_else(|| "Klondike terminal flow has no fresh frame".to_owned())?;


        if previous == Some(TerminalStage::SolverReady)
            && current.gameplay_scene
            && matches!(current.prediction, PredictedAction::Action(_))
        {
            require_klondike_prediction(current.prediction, false)?;
            return Ok(None);
        }
        let observed = io.terminal_stage(&current.frame)?;


        if let Some(stage) = observed {


            if expected == Some(Some(stage))
                || (previous.is_none() && stage.demonstrates_game_win())
            {
                return Ok(Some(stage));
            }


            if Some(stage) != previous {
                return Err(format!("Klondike terminal flow saw unexpected {stage} after {previous:?}; no new control input authorised"));
            }
        }


        if delayed_observations >= REOBSERVATION_LIMIT {
            return Err(format!("Klondike terminal advancement exhausted {delayed_observations} delayed read-only captures after {previous:?}; the last control was not retried"));
        }
        delayed_observations += 1;
        send_log(event_tx, format!("Klondike terminal re-observation {delayed_observations}/{REOBSERVATION_LIMIT}: prior={previous:?}, observed={observed:?}; guest input sent=0"));
        io.wait(settings.animation_delays().klondike_reobserve)?;
        capture_latest(io, latest, event_tx)?;
    }
}


/// Publish one acknowledged action and its fresh result without measuring effects.
///
/// The shared event's zero changed-pixel value is an unmeasured placeholder, not
/// evidence that no card changed. Fresh Solver source authority is recorded
/// separately from effect verification; a finite endpoint may have no next HALO.
fn report_acknowledged_action(
    event_tx: &WorkerEventSink,
    current: &FrameObservation,
    plan: StepPlan,
    operation_index: usize,
    settings: StepRunSettings,
    solver_refreshes: usize,
) {
    let _ = event_tx.send(WorkerEvent::ActionCompleted {
        operation_index,
        operation_limit: settings.operation_limit(),
        before: plan.before(),
        after: current.prediction,
        input_commands: plan.input().qmp_command_count(),
        input_events: plan.input().qmp_event_count(),
        changed_pixels: 0,
        continued_from_halo: true,
    });
    let run_bound = settings.bounded_operation_limit()
        .map_or_else(|| "continuous".to_owned(), |limit| limit.to_string());
    send_log(event_tx, format!(
        "Klondike action {operation_index}/{run_bound} acknowledged from a fresh canonical Solver target; {} -> {}; pixel difference not measured, previous effect remains unproven; Solver refreshes={solver_refreshes}. One action budget slot consumed. Only a fresh supported target authorises the next action; no card transfer, completion or restart inferred.",
        format_prediction_target(plan.before()), format_prediction_target(current.prediction),
    ));
}


/// Confirm completion from two consecutive fresh frames with a finite read-only budget.
///
/// A known candidate may supply the first observation. Otherwise capture the first
/// result immediately after the caller's Solve settle. Each newly decoded frame
/// replaces `latest` before STOP or analysis errors can end the run. No action is
/// classified or sent, and a non-candidate resets the confirmation count.
/// Acknowledged Solve uses the existing longer post-game observation bound;
/// an ordinary candidate retains the card-recovery bound. Neither budget resets.
fn await_completion(
    io: &mut impl KlondikeIo,
    latest: &mut Option<FrameObservation>,
    settings: StepRunSettings,
    event_tx: &WorkerEventSink,
    cancel_requested: &AtomicBool,
    context: CompletionWait,
) -> Result<bool, String> {


    let (known_candidate, reobservation_limit) = match context {
        CompletionWait::ConfirmCandidate => (true, REOBSERVATION_LIMIT),
        CompletionWait::AfterSolve => (false, SOLVE_COMPLETION_REOBSERVATION_LIMIT),
    };
    let mut consecutive = usize::from(known_candidate);
    let mut delayed_observations = usize::from(known_candidate);


    loop {
        require_running(cancel_requested)?;


        if delayed_observations > 0 {
            io.wait(settings.animation_delays().klondike_reobserve)?;
        }
        let frame = io.capture_diagnostic()?;
        event_tx.publish_diagnostic_frame(frame.clone());
        *latest = Some(diagnostic_observation(frame));
        require_running(cancel_requested)?;
        let candidate = io.completion(&latest.as_ref().expect("fresh completion frame").frame)?;
        consecutive = if candidate { consecutive + 1 } else { 0 };
        send_log(event_tx, format!(
            "Klondike completion observation: candidate={candidate}, consecutive={consecutive}/2; delayed captures={delayed_observations}/{reobservation_limit}; guest input sent=0",
        ));


        if consecutive >= 2 {
            require_running(cancel_requested)?;
            return Ok(true);
        }


        if delayed_observations >= reobservation_limit {
            return Ok(false);
        }
        delayed_observations += 1;
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
