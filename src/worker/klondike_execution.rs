//! Klondike execution with bounded recovery and one-shot Solve requests.
//!
//! Zero requests continuous gameplay until STOP, uncertainty or a confirmed game win.
//! Every unresolved context has finite Solver-refresh and observation budgets.
//! An uncertain input is never replayed. An acknowledged card action may continue
//! from a freshly classified HALO only after mode-owned source replacement proof.
//! An independently recognised new Solve control can instead be handed off without
//! claiming the previous effect. Both paths consume the requested action budget.
//! Solve is requested once and followed by
//! bounded read-only completion observations. One board is one Klondike game;
//! completion requires two fresh positive observations. Only an authorised continuous
//! run may advance through the separately calibrated terminal controls.

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


/// Keep complete effect proof separate from the mode-owned source replacement proof.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct EffectVerdict {
    /// Independent source and destination proof established the previous effect.
    verified: bool,
    /// Card content changed under Klondike's source replacement policy alone.
    source_replaced: bool,
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


    /// Measure full effect and source replacement independently of the next target.
    fn effect(
        &mut self,
        before: &CapturedFrame,
        after: &CapturedFrame,
        action: GuidedAction,
    ) -> Result<EffectVerdict, String> {
        klondike::inspect_effect(before, after, action)
            .map(|evidence| EffectVerdict { verified: evidence.verified, source_replaced: evidence.source_replaced })
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


    fn effect(
        &mut self,
        before: &CapturedFrame,
        after: &CapturedFrame,
        action: GuidedAction,
    ) -> Result<EffectVerdict, String> {
        let evidence = klondike::inspect_effect(before, after, action)
            .map_err(|error| format!("Klondike effect analysis failed: {error}"))?;
        send_log(self.event_tx, format!("Klondike effect evidence for {}: {evidence}", action.target));
        Ok(EffectVerdict { verified: evidence.verified, source_replaced: evidence.source_replaced })
    }
}


/// Explicit endpoint separating reviewed predictions, proven gameplay and completion.
#[derive(Debug, PartialEq, Eq)]
enum RunOutcome {
    /// The finite action budget was consumed without implying game completion.
    Completed {
        /// Gameplay effects independently established by source and destination proof.
        verified: usize,
        /// Acknowledged actions accepted only through fresh HALO or new Solve authority.
        halo: usize,
    },
    /// Initial read-only validation found a different canonical target requiring explicit review.
    PreviewChanged,
    /// Solver refresh produced a preview that requires another explicit user run.
    RecoveryOnly,
    /// One board is one game, confirmed by two fresh mode-owned completion observations.
    GameWon {
        /// Ordinary gameplay effects proven before the independent completion endpoint.
        previous_verified: usize,
        /// Previous actions accepted from fresh HALO or new Solve control evidence.
        previous_halo: usize,
    },
    /// Solve was requested once; bounded observations did not establish completion.
    SolveRequested {
        /// Gameplay effects proven before the separate, unverified Solve request.
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
            send_log(event_tx, format!("Klondike bounded run complete: {verified} verified gameplay action(s), {halo} accepted from fresh HALO or new Solve control evidence; no completion or restart inferred."));
        }
        Ok(RunOutcome::PreviewChanged) => {


            if let Some(observation) = latest {
                event_tx.publish_frame(observation.frame, observation.prediction, true);
            }
            send_state(event_tx, WorkerState::Ready);
            send_status(event_tx, "Preview changed — review next move".to_owned());
            send_log(event_tx, "Klondike initial validation found a different current target. Guest inputs sent: 0. The refreshed prediction is displayed for review; press Step Once or Multi-Step again to authorise it.".to_owned());
        }
        Ok(RunOutcome::RecoveryOnly) => {


            if let Some(observation) = latest {
                event_tx.publish_frame(observation.frame, observation.prediction, true);
            }
            send_state(event_tx, WorkerState::Ready);
            send_status(event_tx, "Solver refreshed — review next move".to_owned());
            send_log(event_tx, "Klondike Solver recovery produced a new preview. Gameplay inputs sent: 0. Review the preview and press Step Once or Multi-Step again to authorise that action.".to_owned());
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
        .map_or_else(|| "continuous until STOP, uncertainty or game win".to_owned(), |limit| format!("{limit} gameplay action(s)"));
    send_log(event_tx, format!(
        "Klondike requested {requested_actions}, settle={} ms, Solve animation settle={} ms, reobserve={} ms. Draw uses qcode D; recycle and card sources use one click. Solver refresh: one per unresolved context, followed by at most {REOBSERVATION_LIMIT} delayed observations; the first capture is immediate. Solve is clicked once, separately settled and checked with up to {SOLVE_COMPLETION_REOBSERVATION_LIMIT} delayed input-free completion captures; two consecutive positives are required and card recovery keeps its own shorter budget. One board is one game. Only continuous Multi-Step 0 advances through independently recognised terminal controls.",
        delays.klondike_settle.as_millis(), delays.klondike_solve.as_millis(), delays.klondike_reobserve.as_millis(),
    ));
    send_log(event_tx, format!(
        "Klondike input intervals: pointer settle={} ms, gameplay mouse hold={} ms, Draw key hold={} ms. The editable action settle={} ms begins after input acknowledgement; observation time is additional.",
        POINTER_SETTLE_DELAY.as_millis(), MOUSE_HOLD.as_millis(), KEY_HOLD.as_millis(),
        delays.klondike_settle.as_millis(),
    ));
    send_state(event_tx, WorkerState::Validating);
    send_status(event_tx, "Validating Klondike preview".to_owned());
    capture_latest(io, latest, event_tx)?;
    let fresh_prediction = latest.as_ref().expect("fresh planning capture").prediction;
    let mut solver_refreshes = 0usize;
    let mut initial_restarted = false;


    require_klondike_prediction(fresh_prediction, true)?;
    require_running(cancel_requested)?;


    if matches!(fresh_prediction, PredictedAction::NoHighlight)
        && io.completion(&latest.as_ref().expect("fresh planning capture").frame)?
    {


        if !await_completion(io, latest, settings, event_tx, cancel_requested, CompletionWait::ConfirmCandidate)? {
            return Err("initial completion candidate was not confirmed within bounded read-only observations; no input sent".to_owned());
        }


        if settings.is_unbounded() && fresh_prediction == approved_prediction {
            resume_after_win(io, latest, settings, event_tx, cancel_requested, input_attempted)?;
            initial_restarted = true;
        } else {


            if fresh_prediction != approved_prediction {
                send_log(event_tx, "Klondike completion was confirmed read-only after the approved preview changed. No terminal input is authorised by the stale preview; capture the current completed scene and explicitly approve a new continuous run before advancing terminal controls.".to_owned());
            }
            return Ok(RunOutcome::GameWon { previous_verified: 0, previous_halo: 0 });
        }
    }


    if !initial_restarted {
        require_scene(latest.as_ref().expect("fresh planning capture"))?;


        if fresh_prediction != approved_prediction {
            send_log(event_tx, format!(
                "Klondike preview changed: fresh target {} differs from approved preview {}; guest input sent=0; review the refreshed prediction before another run",
                format_prediction_target(fresh_prediction), format_prediction_target(approved_prediction),
            ));
            return Ok(RunOutcome::PreviewChanged);
        }


        if matches!(fresh_prediction, PredictedAction::NoHighlight) {
            // Recovery is authorised only when the explicit no-HALO preview is reproduced.
            refresh_solver(io, latest, &mut solver_refreshes, settings, event_tx, cancel_requested, input_attempted)?;
            await_target(io, latest, settings, event_tx, cancel_requested)?;
            return Ok(RunOutcome::RecoveryOnly);
        }
    }


    // The budget counts acknowledged logical actions, including unverified ones.
    // Effect and continuation counts remain separate diagnostic outcomes.
    let mut completed_operations = 0usize;
    let mut verified_operations = 0usize;
    let mut halo_operations = 0usize;


    loop {
        require_running(cancel_requested)?;


        if settings.bounded_operation_limit().is_some_and(|limit| completed_operations >= limit) {
            return Ok(RunOutcome::Completed { verified: verified_operations, halo: halo_operations });
        }
        let operation_index = next_counter(completed_operations, "gameplay action")?;
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
        completed_operations = operation_index;
        io.wait(if solve_requested { delays.klondike_solve } else { delays.klondike_settle })?;
        send_state(event_tx, WorkerState::Verifying);


        send_status(event_tx, if solve_requested { "Capturing Solve result" } else { "Verifying Klondike move" }.to_owned());


        if solve_requested {
            // Completion checks cannot authorise another Solve or recovery input.
            let complete = await_completion(
                io, latest, settings, event_tx, cancel_requested, CompletionWait::AfterSolve,
            )?;


            if complete && settings.is_unbounded() {
                resume_after_win(io, latest, settings, event_tx, cancel_requested, input_attempted)?;
                continue;
            }
            return Ok(if complete {
                RunOutcome::GameWon { previous_verified: verified_operations, previous_halo: halo_operations }
            } else {
                RunOutcome::SolveRequested { previous_verified: verified_operations, previous_halo: halo_operations }
            });
        }
        capture_latest(io, latest, event_tx)?;
        require_running(cancel_requested)?;
        let mut context_refreshed = false;
        let mut delayed_observations = 0usize;


        loop {
            require_running(cancel_requested)?;
            let current = latest.as_ref().expect("fresh result capture");
            require_klondike_prediction(current.prediction, true)?;
            let completion_candidate = matches!(current.prediction, PredictedAction::NoHighlight)
                && io.completion(&current.frame)?;


            if !completion_candidate && !current.gameplay_scene {


                if delayed_observations >= REOBSERVATION_LIMIT {
                    return Err(format!(
                        "unsupported Klondike scene persisted through {delayed_observations} delayed input-free result captures; the previous gameplay input was not replayed",
                    ));
                }
                delayed_observations += 1;
                send_log(event_tx, format!(
                    "Klondike unsupported result re-observation {delayed_observations}/{REOBSERVATION_LIMIT}: scene not established; gameplay, Solver and terminal input sent=0; previous effect remains pending",
                ));
                io.wait(delays.klondike_reobserve)?;
                capture_latest(io, latest, event_tx)?;
                continue;
            }
            let effect = io.effect(&before, &current.frame, plan.input().action())?;
            let effect_verified = effect.verified;


            if completion_candidate {
                // Own the final action frame before completion captures replace `latest`.
                let terminal_result = FrameObservation {
                    frame: current.frame.clone(),
                    prediction: current.prediction,
                    observed_rows: current.observed_rows,
                    gameplay_scene: current.gameplay_scene,
                    game_progress: current.game_progress,
                };


                if await_completion(io, latest, settings, event_tx, cancel_requested, CompletionWait::ConfirmCandidate)? {


                    if effect_verified {
                        verified_operations = next_counter(verified_operations, "verified action")?;
                        report_verified_action(event_tx, &before, &terminal_result, plan, operation_index, settings, solver_refreshes)?;
                    } else {
                        send_log(event_tx, "Klondike last gameplay effect remains unverified; GAME WIN evidence was independently confirmed by fresh terminal observations. The previous gameplay input was not replayed or counted as verified.".to_owned());
                    }


                    if settings.is_unbounded() {
                        resume_after_win(io, latest, settings, event_tx, cancel_requested, input_attempted)?;
                        break;
                    }
                    return Ok(RunOutcome::GameWon { previous_verified: verified_operations, previous_halo: halo_operations });
                }
                return Err("completion candidate was not confirmed within bounded read-only observations; no gameplay or Solver input retried".to_owned());
            }


            if matches!(current.prediction, PredictedAction::NoHighlight) && !context_refreshed {
                refresh_solver(io, latest, &mut solver_refreshes, settings, event_tx, cancel_requested, input_attempted)?;
                context_refreshed = true;
                continue;
            }


            if effect_verified && matches!(current.prediction, PredictedAction::Action(_)) {
                verified_operations = next_counter(verified_operations, "verified action")?;
                report_verified_action(event_tx, &before, current, plan, operation_index, settings, solver_refreshes)?;
                break;
            }


            if !context_refreshed && fresh_evidence_continuation(plan.input().action(), current.prediction, effect) {
                halo_operations = next_counter(halo_operations, "fresh-evidence continuation")?;
                report_continued_action(event_tx, &before, current, plan, operation_index, settings, solver_refreshes)?;
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


/// Record a proven ordinary gameplay effect separately from independent completion.
fn report_verified_action(
    event_tx: &WorkerEventSink,
    before: &CapturedFrame,
    current: &FrameObservation,
    plan: StepPlan,
    operation_index: usize,
    settings: StepRunSettings,
    solver_refreshes: usize,
) -> Result<(), String> {
    let changed_pixels = materially_changed_pixels(
        before, &current.frame, plan.input().effect_bounds(),
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
    Ok(())
}


/// Decide fresh target authority without upgrading the previous action to proven.
/// The caller has already received an input acknowledgement, waited the configured
/// settle and validated a new supported scene with canonical Klondike actions.
/// No Solver refresh may intervene. Ordinary HALOs require changed source content;
/// a positively recognised new Solve control is a different action, never a retry
/// of the old card click. Draw, recycle and Solve sources retain their own policy.
fn fresh_evidence_continuation(
    previous: GuidedAction,
    fresh: PredictedAction,
    effect: EffectVerdict,
) -> bool {


    if !matches!(previous.target, ActionTarget::Klondike(
        klondike::KlondikeTarget::Waste { .. } | klondike::KlondikeTarget::Tableau { .. }
            | klondike::KlondikeTarget::Foundation { .. }
    )) {
        return false;
    }


    let PredictedAction::Action(next) = fresh else { return false; };
    matches!(next.target, ActionTarget::Klondike(klondike::KlondikeTarget::Solve))
        || effect.source_replaced
}


/// Record an acknowledged action accepted from fresh evidence, not effect proof.
/// Its budget slot is consumed even for Step Once; the result remains the next
/// planning frame. This changes no card history or game-completion authority.
fn report_continued_action(
    event_tx: &WorkerEventSink,
    before: &CapturedFrame,
    current: &FrameObservation,
    plan: StepPlan,
    operation_index: usize,
    settings: StepRunSettings,
    solver_refreshes: usize,
) -> Result<(), String> {
    let changed_pixels = materially_changed_pixels(
        before, &current.frame, plan.input().effect_bounds(),
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
        continued_from_halo: true,
    });


    let authority = if matches!(current.prediction, PredictedAction::Action(GuidedAction {
        target: ActionTarget::Klondike(klondike::KlondikeTarget::Solve), ..
    })) {
        "independently recognised new Solve control"
    } else {
        "fresh HALO with mode-owned source replacement proof"
    };
    let run_bound = settings.bounded_operation_limit()
        .map_or_else(|| "continuous".to_owned(), |limit| limit.to_string());
    send_log(event_tx, format!(
        "Klondike action {operation_index}/{run_bound} accepted from {authority}; {} -> {}; previous effect remains unverified; changed effect pixels={changed_pixels}; Solver refreshes={solver_refreshes}. One action budget slot consumed; no previous input replay, completion or restart inferred.",
        format_prediction_target(plan.before()), format_prediction_target(current.prediction),
    ));
    Ok(())
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
