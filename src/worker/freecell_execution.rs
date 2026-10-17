//! Solver-led Free Cell execution and its ordered one-board restart sequence.
//!
//! One fresh source HALO authorises one click, followed by editable settling and
//! a new frame. Card identity, changed pixels and previous effects are not input
//! conditions. Automatic transfers receive bounded input-free observations.
//! A missing HALO does not establish a win: positive terminal caption/control
//! evidence does. Only continuous runs advance the expected terminal controls.
//! After score skip, Level Up is optional: inspect local OK and New Game
//! readiness on each fresh frame and accept exactly one ready control. Both
//! candidates share three normal observations. Eligible clean or interrupted
//! third-frame OK evidence may reserve one final fresh observation of OK alone.
//! An inactive Solver board receives an editable start delay and fresh capture
//! before activation. Active no-HALO gameplay receives one delayed observation
//! before one Solver refresh. Both share one input reserve per unresolved context.

use std::{path::Path, sync::atomic::{AtomicBool, Ordering}, time::{Duration, Instant}};

use super::{
    FrameObservation, WorkerEvent, WorkerEventSink, WorkerState, capture_screen,
    connect_and_probe, control_search::ControlLookup, execute_planned_input, format_action_status,
    format_prediction_target, format_step_plan, milliseconds, send_log,
    send_state, send_status, validate_probe, wait_or_stop,
};
use crate::{
    capture::CapturedFrame,
    freecell,
    freecell_terminal::{self, TerminalStage},
    game::{ActionTarget, GameMode},
    geometry::PixelPoint,
    parameters::{
        FREECELL_MAX_MULTI_STEP_ACTIONS, LEVEL_UP_APPEAR_DELAY, NOMINAL_FRAME_HEIGHT,
        NOMINAL_FRAME_WIDTH, POST_GAME_MOUSE_HOLD, POST_GAME_STAGE_DELAY,
        SOLVER_MOUSE_HOLD, TERMINAL_CONTROL_SEARCH_ATTEMPTS, TERMINAL_OK_FINAL_CONFIRMATIONS, StepRunSettings,
    },
    qmp::QmpClient,
    stepper::{StepPlan, plan_step},
    tracker::{PredictedAction, TableauScanState},
    terminal_controls::{Control, profiles::FREECELL_CONTROLS},
};


/// Reject invalid finite budgets before connecting or changing cancellation.
pub(super) fn validate_operation_limit(limit: usize) -> Result<(), String> {


    if limit > FREECELL_MAX_MULTI_STEP_ACTIONS {
        return Err(format!("Free Cell requires 0 for continuous play or 1..={FREECELL_MAX_MULTI_STEP_ACTIONS} actions; requested {limit}"));
    }
    Ok(())
}


/// I/O injection leaves all gameplay and terminal authority in production classifiers.
trait FreeCellIo {


    /// Acquire one new decoded frame; no classifier result is fabricated here.
    fn capture(&mut self) -> Result<CapturedFrame, String>;


    /// Probe the running VM and current absolute tablet before each input.
    fn probe(&mut self) -> Result<(), String>;


    /// Deliver exactly one source action without replay on uncertainty.
    fn input(&mut self, plan: StepPlan) -> Result<(), String>;


    /// Activate or refresh Solver once from fresh positively recognised gameplay.
    fn solver(&mut self) -> Result<(), String>;


    /// Click exactly one freshly ready expected terminal control.
    fn terminal(&mut self, stage: TerminalStage, point: PixelPoint) -> Result<(), String>;


    /// Perform an interruptible settle or observation delay.
    fn wait(&mut self, duration: Duration) -> Result<(), String>;
}


/// QMP adapter reusing capture, input, logging and cooperative cancellation.
struct QmpFreeCellIo<'a> {
    /// Existing connection local to this execution run.
    qmp: &'a mut QmpClient,
    /// Shared capture artefacts are reserved beside this socket.
    socket_path: &'a Path,
    /// Cooperative STOP and mode/socket invalidation flag.
    cancel_requested: &'a AtomicBool,
    /// Bounded UI output and complete session log.
    event_tx: &'a WorkerEventSink,
}


impl FreeCellIo for QmpFreeCellIo<'_> {


    fn capture(&mut self) -> Result<CapturedFrame, String> {
        require_running(self.cancel_requested)?;
        let started = Instant::now();
        let (frame, timing) = capture_screen(self.qmp, self.socket_path, self.event_tx)?;
        send_log(self.event_tx, format!(
            "PROFILE Free Cell capture: acquisition={:.1} ms; request={:.1} ms, receive/read={:.1} ms, convert/decode={:.1} ms, png-encode={:.1} ms.",
            milliseconds(started.elapsed()), milliseconds(timing.screendump), milliseconds(timing.file_read), milliseconds(timing.decode), milliseconds(timing.png_encode),
        ));
        Ok(frame)
    }


    fn probe(&mut self) -> Result<(), String> {
        require_running(self.cancel_requested)?;
        let probe = self.qmp.probe().map_err(|error| format!("Free Cell pre-input probe failed: {error}"))?;
        validate_probe(&probe)?;


        if probe.current_absolute_pointer_name() != Some("QEMU HID Tablet") {
            return Err("Free Cell input requires the current absolute QEMU HID Tablet".to_owned());
        }
        require_running(self.cancel_requested)
    }


    fn input(&mut self, plan: StepPlan) -> Result<(), String> {
        execute_planned_input(self.qmp, plan, self.cancel_requested)
    }


    fn solver(&mut self) -> Result<(), String> {
        require_running(self.cancel_requested)?;
        self.qmp.click_with_hold(
            freecell::SOLVER_CLICK, NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT, SOLVER_MOUSE_HOLD,
        ).map_err(|error| format!("Free Cell Solver delivery is uncertain: {error}; no retry permitted"))
    }


    fn terminal(&mut self, stage: TerminalStage, point: PixelPoint) -> Result<(), String> {
        require_running(self.cancel_requested)?;
        if point == PixelPoint::new(0, 0) {
            return Err("Free Cell terminal control has no detected click point; no input sent".to_owned());
        }
        self.qmp.click_with_hold(
            point, NOMINAL_FRAME_WIDTH,
            NOMINAL_FRAME_HEIGHT, POST_GAME_MOUSE_HOLD,
        ).map_err(|error| format!("Free Cell {stage} delivery is uncertain: {error}; no retry permitted"))
    }


    fn wait(&mut self, duration: Duration) -> Result<(), String> {
        wait_or_stop(duration, self.cancel_requested, "STOP during Free Cell settling; no input replayed").map(|_| ())
    }
}


/// Finite-budget completion differs from independent one-board game completion.
#[derive(Debug, PartialEq, Eq)]
enum RunOutcome {
    /// Acknowledged source clicks consumed the finite budget.
    Completed(usize),
    /// Positive terminal entry ended a finite run without restart inputs.
    GameWon(usize),
}


/// Run fresh-evidence Free Cell play using shared worker facilities.
pub(super) fn run_freecell_steps(
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


        if scan_state.mode() != GameMode::FreeCell {
            return Err("Free Cell run received another mode's scan state".to_owned());
        }
        require_preview_mode(approved_prediction)?;
        send_state(event_tx, WorkerState::Connecting);
        let (mut qmp, probe) = connect_and_probe(socket_path)?;
        validate_probe(&probe)?;
        send_log(event_tx, probe.summary());
        let mut io = QmpFreeCellIo { qmp: &mut qmp, socket_path, cancel_requested, event_tx };
        drive_run(&mut io, settings, event_tx, cancel_requested, &mut latest, &mut input_attempted)
    })();


    match result {
        Ok(RunOutcome::Completed(actions)) => {


            if let Some(observation) = latest {
                let prediction = observation.prediction;
                event_tx.publish_frame(observation.frame, prediction, false);
                let _ = event_tx.send(WorkerEvent::RunCompleted {
                    prediction, requested_operations: settings.operation_limit(),
                    verified_operations: 0, halo_operations: actions,
                });
            }
            send_state(event_tx, WorkerState::Ready);
            send_status(event_tx, "Ready".to_owned());
            send_log(event_tx, format!("Free Cell finite run complete: {actions} acknowledged fresh-HALO actions; card effects were not measured. Latest result retained."));
        }
        Ok(RunOutcome::GameWon(actions)) => {


            if let Some(observation) = latest {
                event_tx.publish_diagnostic_frame(observation.frame);
            }
            let _ = event_tx.send(WorkerEvent::FreeCellGameCompleted);
            send_state(event_tx, WorkerState::Ready);
            send_status(event_tx, "Free Cell game won — no automatic restart".to_owned());
            send_log(event_tx, format!("Free Cell one-board GAME WIN recognised after {actions} source actions; finite run sent no terminal or restart input."));
        }
        Err(error) => {


            if let Some(observation) = latest {
                event_tx.publish_diagnostic_frame(observation.frame);
            }
            send_state(event_tx, WorkerState::Uncertain);
            send_status(event_tx, "Free Cell stopped".to_owned());
            send_log(event_tx, format!("Free Cell stopped: {error}; guest input attempted={input_attempted}. No uncertain source, Solver or terminal input was replayed; latest available frame retained."));
        }
    }
}


/// Drive one fresh source click at a time; the preview never freezes coordinates.
fn drive_run(
    io: &mut impl FreeCellIo,
    settings: StepRunSettings,
    event_tx: &WorkerEventSink,
    cancel_requested: &AtomicBool,
    latest: &mut Option<FrameObservation>,
    input_attempted: &mut bool,
) -> Result<RunOutcome, String> {
    validate_operation_limit(settings.operation_limit())?;
    let delays = settings.animation_delays();
    send_log(event_tx, format!(
        "Free Cell fresh-HALO execution: game-start delay={} ms, action/automatic-transfer settle={} ms, no-HALO observation={} ms, delayed allowance={}. Inactive Solver receives one start delay and fresh capture before activation. Active no-HALO gameplay receives one delayed input-free observation before one refresh; activation and refresh share one reserve per unresolved context. One source click consumes one action; Solver and terminal clicks do not. No card identity, pixel difference, SUIT input, Draw, Recycle or Solve is used. Continuous 0 may restart one-board games through Score, optional Level Up OK, New Game and Play. After score skip, exactly one ready local OK or New Game control selects the next stage; a missing pair receives three normal observations, and an ambiguous pair stops without input. Only a settled-size OK with a current exact-black central glyph awaiting geometry on observation three may reserve one final fresh 200 ms OK confirmation. New Game and Play remain limited to three observations; no control input is replayed.",
        delays.freecell_game_start.as_millis(), delays.freecell_settle.as_millis(), delays.freecell_reobserve.as_millis(), settings.freecell_observation_limit(),
    ));
    capture_latest(io, latest, event_tx, cancel_requested)?;
    let mut actions = 0usize;


    loop {
        require_running(cancel_requested)?;


        if settings.bounded_operation_limit().is_some_and(|limit| actions >= limit) {
            return Ok(RunOutcome::Completed(actions));
        }


        if let Some(stage) = await_context(io, latest, settings, event_tx, cancel_requested, input_attempted)? {


            if !settings.is_unbounded() {
                let _game_win_images = super::begin_game_win_images(event_tx)?;
                return Ok(RunOutcome::GameWon(actions));
            }
            restart_game(io, latest, stage, settings, event_tx, cancel_requested, input_attempted)?;
            continue;
        }
        let prediction = current(latest)?.prediction;
        let plan = plan_step(prediction).map_err(|error| format!("Free Cell plan rejected: {error}"))?;
        let next_actions = actions.checked_add(1).ok_or_else(|| "Free Cell action counter exhausted".to_owned())?;
        send_log(event_tx, format_step_plan(plan)?);
        guard_input(io, cancel_requested)?;
        send_state(event_tx, WorkerState::Acting);
        send_status(event_tx, format_action_status(prediction));
        *input_attempted = true;
        io.input(plan).map_err(|error| format!("Free Cell source input outcome uncertain: {error}; no replay permitted"))?;
        actions = next_actions;
        io.wait(delays.freecell_settle)?;
        capture_latest(io, latest, event_tx, cancel_requested)?;
        let after = current(latest)?.prediction;
        let _ = event_tx.send(WorkerEvent::ActionCompleted {
            operation_index: actions, operation_limit: settings.operation_limit(),
            before: prediction, after, input_commands: plan.input().qmp_command_count(),
            input_events: plan.input().qmp_event_count(), changed_pixels: 0, continued_from_halo: true,
        });
        send_log(event_tx, format!(
            "Free Cell action {actions} acknowledged once: {} -> {}; card effects and pixel differences not measured. Next input requires a new supported source HALO, including at the same position.",
            format_prediction_target(prediction), format_prediction_target(after),
        ));
    }
}


/// Observe automatic transfers, terminal entry or one supported source without speculation.
fn await_context(
    io: &mut impl FreeCellIo,
    latest: &mut Option<FrameObservation>,
    settings: StepRunSettings,
    event_tx: &WorkerEventSink,
    cancel_requested: &AtomicBool,
    input_attempted: &mut bool,
) -> Result<Option<TerminalStage>, String> {
    let mut solver_reserved = false;
    let mut delayed = 0usize;
    let mut start_observed = false;
    let mut active_reobserved = false;


    loop {
        require_running(cancel_requested)?;
        let observation = current(latest)?;
        let stage = freecell_terminal::classify_win_entry(&observation.frame)
            .map_err(|error| format!("Free Cell win-entry analysis failed: {error}"))?;


        if stage.is_some() {
            send_log(event_tx, format!("Free Cell independent one-board GAME WIN entry={stage:?}; missing HALO alone did not establish completion."));
            return Ok(stage);
        }
        let mut observed_solver = "unavailable: unsupported scene";


        if observation.gameplay_scene {


            if let PredictedAction::Action(_) = observation.prediction {
                require_source(observation.prediction)?;
                return Ok(None);
            }


            if !matches!(observation.prediction, PredictedAction::NoHighlight) {
                return Err("Free Cell scene has an uncertain or unsupported source recommendation".to_owned());
            }
            let solver_active = freecell::solver_active(&observation.frame)
                .map_err(|error| format!("Free Cell Solver-state analysis failed: {error}"))?;
            observed_solver = if solver_active { "active" } else { "inactive" };


            if !solver_reserved {


                if !solver_active && !start_observed {
                    start_observed = true;
                    send_status(event_tx, "Settling Free Cell board before Solver activation".to_owned());
                    send_log(event_tx, format!("Free Cell Solver observed inactive with no HALO: waiting game-start delay={} ms once, then fresh input-free capture before deciding activation; guest input sent=0.", settings.animation_delays().freecell_game_start.as_millis()));
                    io.wait(settings.animation_delays().freecell_game_start)?;
                    capture_latest(io, latest, event_tx, cancel_requested)?;
                    continue;
                }


                if solver_active && !active_reobserved {


                    if delayed >= settings.freecell_observation_limit() {
                        return Err(format!("Free Cell active no-HALO observation allowance exhausted after {delayed} delayed captures before Solver refresh; no further input authorised"));
                    }
                    active_reobserved = true;
                    delayed += 1;
                    send_log(event_tx, format!("Free Cell Solver observed active with no HALO: input-free re-observation {delayed}/{} after {} ms allows automatic transfers before considering one refresh.", settings.freecell_observation_limit(), settings.animation_delays().freecell_reobserve.as_millis()));
                    io.wait(settings.animation_delays().freecell_reobserve)?;
                    capture_latest(io, latest, event_tx, cancel_requested)?;
                    continue;
                }
                guard_input(io, cancel_requested)?;
                solver_reserved = true;
                *input_attempted = true;
                send_state(event_tx, WorkerState::Acting);
                let operation = if solver_active { "refresh" } else { "activation" };
                send_status(event_tx, format!("Free Cell Solver {operation} once"));
                send_log(event_tx, format!("Free Cell Solver {operation}: fresh supported no-HALO board, observed Solver={observed_solver}; one click consumes this unresolved context's shared activation/refresh reserve."));
                io.solver()?;
                io.wait(settings.animation_delays().freecell_settle)?;
                capture_latest(io, latest, event_tx, cancel_requested)?;
                delayed = 0;
                continue;
            }
        }


        if delayed >= settings.freecell_observation_limit() {
            return Err(format!("Free Cell source/win observation allowance exhausted after {delayed} delayed captures; scene={}, observed Solver={observed_solver}, activation/refresh reserved={solver_reserved}; no further input authorised", observation.gameplay_scene));
        }
        delayed += 1;
        send_log(event_tx, format!("Free Cell no-HALO observation {delayed}/{}: scene={}, observed Solver={observed_solver}, activation/refresh reserved={solver_reserved}; waiting {} ms, then input-free recapture.", settings.freecell_observation_limit(), observation.gameplay_scene, settings.animation_delays().freecell_reobserve.as_millis()));
        io.wait(settings.animation_delays().freecell_reobserve)?;
        capture_latest(io, latest, event_tx, cancel_requested)?;
    }
}


/// Advance each ready local control once after independent win entry.
/// Score skip can lead to optional Level Up OK or directly to New Game.
fn restart_game(
    io: &mut impl FreeCellIo,
    latest: &mut Option<FrameObservation>,
    initial_stage: TerminalStage,
    settings: StepRunSettings,
    event_tx: &WorkerEventSink,
    cancel_requested: &AtomicBool,
    input_attempted: &mut bool,
) -> Result<(), String> {
    let _game_win_images = super::begin_game_win_images(event_tx)?;
    let mut stage = initial_stage;
    let mut ready_point = None;


    loop {
        let point = if let Some(point) = ready_point.take() {
            point
        } else {
            await_expected_control(io, latest, stage, settings, event_tx, cancel_requested)?
        };
        if point == PixelPoint::new(0, 0) {
            return Err("Free Cell expected terminal control returned no match; no input authorised".to_owned());
        }
        guard_input(io, cancel_requested)?;
        send_state(event_tx, WorkerState::Acting);
        send_status(event_tx, stage.to_string());
        send_log(event_tx, format!("Free Cell deterministic GAME WIN control: {stage}; one click at {:?}; unrelated panel colours, levels and artwork are not conditions.", point));
        *input_attempted = true;
        io.terminal(stage, point)?;
        io.wait(if stage == TerminalStage::Score { LEVEL_UP_APPEAR_DELAY } else { POST_GAME_STAGE_DELAY })?;
        capture_latest(io, latest, event_tx, cancel_requested)?;


        match stage {
            TerminalStage::Score => {
                let (next_stage, point) = await_post_score_control(
                    io, latest, settings, event_tx, cancel_requested,
                )?;
                stage = next_stage;
                ready_point = Some(point);
            },
            TerminalStage::LevelUp => stage = TerminalStage::NewGame,
            TerminalStage::NewGame => stage = TerminalStage::Play,
            TerminalStage::Play => break,
        }
    }


    for delayed in 0..=settings.freecell_observation_limit() {
        require_running(cancel_requested)?;


        if current(latest)?.gameplay_scene {


            if await_context(io, latest, settings, event_tx, cancel_requested, input_attempted)?.is_some() {
                return Err("Free Cell restart encountered another terminal entry instead of a fresh Solver board".to_owned());
            }
            let _ = event_tx.send(WorkerEvent::FreeCellGameCompleted);
            send_log(event_tx, "Free Cell one-board restart complete: fresh gameplay and supported Solver source recognised; continuous play resumes.".to_owned());
            return Ok(());
        }


        if delayed == settings.freecell_observation_limit() {
            break;
        }
        io.wait(settings.animation_delays().freecell_reobserve)?;
        capture_latest(io, latest, event_tx, cancel_requested)?;
    }
    Err("Free Cell Play acknowledgement was not followed by a recognised fresh board within bounded input-free observations; Play was not retried".to_owned())
}


/// Resolve optional OK and New Game from the same fresh frame.
/// Both candidates share three normal observations; an ambiguous pair sends no input.
/// Only an eligible third-frame OK may reserve one final fresh confirmation.
/// That fourth frame checks OK alone and cannot authorise a New Game alternative.
fn await_post_score_control(
    io: &mut impl FreeCellIo,
    latest: &mut Option<FrameObservation>,
    settings: StepRunSettings,
    event_tx: &WorkerEventSink,
    cancel_requested: &AtomicBool,
) -> Result<(TerminalStage, PixelPoint), String> {
    let mut ok_lookup = ControlLookup::new("Free Cell", &FREECELL_CONTROLS, Control::Ok, event_tx);
    let mut new_game_lookup = ControlLookup::new("Free Cell", &FREECELL_CONTROLS, Control::NewGame, event_tx);
    let mut observation_limit = TERMINAL_CONTROL_SEARCH_ATTEMPTS;
    let mut observations = 0;
    for attempt in 1..=TERMINAL_CONTROL_SEARCH_ATTEMPTS + TERMINAL_OK_FINAL_CONFIRMATIONS {
        observations = attempt;
        require_running(cancel_requested)?;
        if current(latest)?.gameplay_scene {
            ok_lookup.forget_ok_geometry();
            ok_lookup.set_outcome("rejected-gameplay");
            new_game_lookup.set_outcome("rejected-gameplay");
        } else {
            let frame = &current(latest)?.frame;
            let ok_point = ok_lookup.search(frame)?;
            let new_game_point = if attempt <= TERMINAL_CONTROL_SEARCH_ATTEMPTS {
                new_game_lookup.search(frame)?
            } else {
                PixelPoint::new(0, 0)
            };
            match (ok_point != PixelPoint::new(0, 0), new_game_point != PixelPoint::new(0, 0)) {
                (true, false) => {
                    let ready = ok_lookup.confirm_ok(frame, ok_point)?;
                    if ready != PixelPoint::new(0, 0) {
                        ok_lookup.set_outcome("matched");
                        new_game_lookup.set_outcome("optional absent");
                        send_log(event_tx, "Free Cell post-score branch: settled OK ready; optional Level Up selected.".to_owned());
                        return Ok((TerminalStage::LevelUp, ready));
                    }
                }
                (false, true) => {
                    ok_lookup.confirm_ok(frame, ok_point)?;
                    if !ok_lookup.has_ok_panel() {
                        ok_lookup.set_outcome("optional absent");
                        new_game_lookup.set_outcome("matched");
                        send_log(event_tx, "Free Cell post-score branch: local New Game ready; no Level Up step required.".to_owned());
                        return Ok((TerminalStage::NewGame, new_game_point));
                    }
                }
                (true, true) => {
                    ok_lookup.set_outcome("ambiguous");
                    new_game_lookup.set_outcome("ambiguous");
                    return Err("Free Cell post-score controls are uncertain: both OK and New Game are ready; no branch input authorised".to_owned());
                }
                (false, false) => { ok_lookup.confirm_ok(frame, ok_point)?; }
            }
        }
        if attempt == observation_limit {
            if !ok_lookup.reserve_final_ok_confirmation(attempt) { break; }
            observation_limit += TERMINAL_OK_FINAL_CONFIRMATIONS;
            send_log(event_tx, format!("Free Cell reserved one final input-free OK confirmation after {attempt} normal observations; observation {observation_limit} checks only the fresh OK witness. New Game, score skip and prior controls are not retried."));
        }
        let retry_delay = ok_lookup.retry_delay(settings.animation_delays().freecell_reobserve);
        send_log(event_tx, format!("Free Cell post-score OK/New Game absent or OK not settled after search {attempt}/{observation_limit}: waiting {} ms before input-free recapture {attempt}/{} and repeat search; score skip is not repeated.", retry_delay.as_millis(), observation_limit - 1));
        if let Err(error) = io.wait(retry_delay) {
            ok_lookup.set_outcome("wait stopped");
            new_game_lookup.set_outcome("wait stopped");
            return Err(error);
        }
        ok_lookup.note_recapture();
        if attempt < TERMINAL_CONTROL_SEARCH_ATTEMPTS { new_game_lookup.note_recapture(); }
        if let Err(error) = capture_latest(io, latest, event_tx, cancel_requested) {
            ok_lookup.set_outcome("capture failed or stopped");
            new_game_lookup.set_outcome("capture failed or stopped");
            return Err(error);
        }
    }
    ok_lookup.set_outcome("not found");
    new_game_lookup.set_outcome("not found");
    Err(format!("Free Cell expected settled OK or New Game did not become ready after {observations} searches and {} fresh recaptures; New Game was limited to the first {TERMINAL_CONTROL_SEARCH_ATTEMPTS} observations; latest failed frame retained for preview; score skip was not retried", observations - 1))
}


/// Expected menus receive three normal observations without replaying prior input.
/// Only eligible third-frame OK evidence may reserve one final fresh confirmation.
/// Score skip retains its existing caption recognition and observation allowance.
fn await_expected_control(
    io: &mut impl FreeCellIo,
    latest: &mut Option<FrameObservation>,
    stage: TerminalStage,
    settings: StepRunSettings,
    event_tx: &WorkerEventSink,
    cancel_requested: &AtomicBool,
) -> Result<PixelPoint, String> {
    let mut lookup = freecell_terminal::control(stage)
        .map(|control| ControlLookup::new("Free Cell", &FREECELL_CONTROLS, control, event_tx));
    let mut observation_limit = if lookup.is_some() { TERMINAL_CONTROL_SEARCH_ATTEMPTS } else { settings.freecell_observation_limit() + 1 };
    let maximum_observations = observation_limit
        + if stage == TerminalStage::LevelUp { TERMINAL_OK_FINAL_CONFIRMATIONS } else { 0 };
    let mut observations = 0;
    for attempt in 1..=maximum_observations {
        observations = attempt;
        require_running(cancel_requested)?;
        let point = if let Some(lookup) = lookup.as_mut() {
            let frame = &current(latest)?.frame;
            if stage == TerminalStage::LevelUp && current(latest)?.gameplay_scene {
                lookup.forget_ok_geometry();
                lookup.set_outcome("rejected-gameplay");
                PixelPoint::new(0, 0)
            } else {
                let candidate = lookup.search(frame)?;
                lookup.confirm_ok(frame, candidate)?
            }
        } else {
            freecell_terminal::expected_control_point(&current(latest)?.frame, stage)
                .map_err(|error| format!("Free Cell expected {stage} readiness failed: {error}"))?
        };
        if point != PixelPoint::new(0, 0) {
            if let Some(lookup) = lookup.as_mut() { lookup.set_outcome("matched"); }
            return Ok(point);
        }
        if attempt == observation_limit {
            if !lookup.as_mut().is_some_and(|lookup| lookup.reserve_final_ok_confirmation(attempt)) { break; }
            observation_limit += TERMINAL_OK_FINAL_CONFIRMATIONS;
            send_log(event_tx, format!("Free Cell reserved one final input-free OK confirmation after {attempt} normal observations; observation {observation_limit} requires a fresh settled OK witness and never repeats a click."));
        }
        let normal_delay = settings.animation_delays().freecell_reobserve;
        let retry_delay = lookup.as_ref().map_or(normal_delay, |lookup| lookup.retry_delay(normal_delay));
        send_log(event_tx, format!("Free Cell terminal input-free recapture {attempt}/{}: waiting {} ms only for expected {stage}; prior click is not repeated.", observation_limit - 1, retry_delay.as_millis()));
        if let Err(error) = io.wait(retry_delay) {
            if let Some(lookup) = lookup.as_mut() { lookup.set_outcome("wait stopped"); }
            return Err(error);
        }
        if let Some(lookup) = lookup.as_mut() { lookup.note_recapture(); }
        if let Err(error) = capture_latest(io, latest, event_tx, cancel_requested) {
            if let Some(lookup) = lookup.as_mut() { lookup.set_outcome("capture failed or stopped"); }
            return Err(error);
        }
    }
    if let Some(lookup) = lookup.as_mut() { lookup.set_outcome("not found"); }
    Err(format!("Free Cell expected {stage} did not become ready after {observations} searches and {} fresh recaptures; latest failed frame retained for preview; no terminal input retried", observations - 1))
}


/// Retain decoded pixels before any late STOP or classifier failure is returned.
fn capture_latest(
    io: &mut impl FreeCellIo,
    latest: &mut Option<FrameObservation>,
    event_tx: &WorkerEventSink,
    cancel_requested: &AtomicBool,
) -> Result<(), String> {
    require_running(cancel_requested)?;
    send_state(event_tx, WorkerState::Capturing);
    let frame = io.capture()?;
    event_tx.publish_diagnostic_frame(frame.clone());
    *latest = Some(FrameObservation {
        frame, prediction: PredictedAction::NoHighlight, observed_rows: None,
        gameplay_scene: false, game_progress: None,
    });
    require_running(cancel_requested)?;
    let observation = latest.as_mut().ok_or_else(|| "Free Cell decoded frame was not retained".to_owned())?;
    observation.gameplay_scene = freecell::is_gameplay_scene(&observation.frame)
        .map_err(|error| format!("Free Cell scene analysis failed: {error}"))?;
    observation.prediction = freecell::analyse(&observation.frame)
        .map_err(|error| format!("Free Cell source analysis failed: {error}"))?.prediction;
    require_running(cancel_requested)
}


/// Require fresh Free Cell action metadata and the shared canonical input plan.
fn require_source(prediction: PredictedAction) -> Result<(), String> {


    match prediction {
        PredictedAction::Action(action) if matches!(action.target, ActionTarget::FreeCell(_)) => {
            plan_step(prediction).map(|_| ()).map_err(|error| format!("Free Cell source rejected: {error}"))
        }
        _ => Err("Fresh recommendation is not a canonical Free Cell source".to_owned()),
    }
}


/// Advisory previews may be empty, but another mode's action cannot cross this boundary.
fn require_preview_mode(prediction: PredictedAction) -> Result<(), String> {


    match prediction {
        PredictedAction::NoHighlight | PredictedAction::CalibrationOnly { mode: GameMode::FreeCell } => Ok(()),
        PredictedAction::Action(action) if matches!(action.target, ActionTarget::FreeCell(_)) => Ok(()),
        _ => Err("Free Cell run received an unsupported advisory preview".to_owned()),
    }
}


/// Access the latest immutable observation without unchecked frame assumptions.
fn current(latest: &Option<FrameObservation>) -> Result<&FrameObservation, String> {
    latest.as_ref().ok_or_else(|| "Free Cell run has no fresh observation".to_owned())
}


/// Probe and recheck STOP immediately before each authorised input class.
fn guard_input(io: &mut impl FreeCellIo, cancel_requested: &AtomicBool) -> Result<(), String> {
    require_running(cancel_requested)?;
    io.probe()?;
    require_running(cancel_requested)
}


/// STOP also covers mode/socket invalidation; uncertain inputs are never replayed.
fn require_running(cancel_requested: &AtomicBool) -> Result<(), String> {


    if cancel_requested.load(Ordering::Acquire) {
        return Err("STOP requested".to_owned());
    }
    Ok(())
}


#[cfg(test)]
mod tests {
    //! Native evidence drives the production controller, not mocked detection authority.

    use std::collections::VecDeque;

    use super::*;
    use crate::{capture::{PixelFormat, decode_png}, parameters::AnimationSettleDelays};


    /// One typed injected failure prevents accidental ambiguity about replay tests.
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Fault {
        /// No failure injection.
        None,
        /// Source delivery may have reached the guest before its acknowledgement fails.
        Source,
        /// Solver delivery may have reached the guest before its acknowledgement fails.
        Solver,
        /// Terminal delivery may have reached the guest before its acknowledgement fails.
        Terminal,
        /// Set STOP at the final probe boundary.
        StopAtProbe,
        /// Set STOP after acquiring the first decoded frame.
        StopAfterCapture,
        /// Set STOP during the first requested settle or observation wait.
        StopOnWait,
        /// Cancellation during the second terminal recapture wait.
        StopOnSecondWait,
        /// Cancellation during the reserved final OK confirmation wait.
        StopOnThirdWait,
        /// Cancellation after decoding the reserved fourth observation.
        StopAfterFourthCapture,
        /// Fail acquisition of the reserved fourth observation before decoding.
        FourthCapture,
        /// Only the Level Up acknowledgement is uncertain; earlier Score succeeds.
        LevelUpTerminal,
    }


    /// I/O records inputs while all native frame decisions use real classifiers.
    struct FakeIo<'a> {
        /// Successive fresh native frames; exhaustion detects an unexpected capture.
        frames: VecDeque<CapturedFrame>,
        /// Attempted source inputs, including an injected uncertain delivery.
        sources: Vec<PredictedAction>,
        /// Attempted one-shot terminal controls in execution order.
        controls: Vec<TerminalStage>,
        /// Detected full-frame coordinates delivered to the terminal adapter.
        control_points: Vec<PixelPoint>,
        /// Attempted activations, independent from the source-action budget.
        solver_clicks: usize,
        /// Actual native banner state observed at each attempted Solver input.
        solver_states: Vec<Option<bool>>,
        /// Acquisition count at Solver delivery proves a new capture preceded it.
        solver_capture_counts: Vec<usize>,
        /// Banner classification from the latest acquired pixels, not a mocked state.
        last_solver_state: Option<bool>,
        /// Total acquisitions, distinguishing immediate from delayed observations.
        captures: usize,
        /// Requested settle and input-free observation intervals.
        waits: Vec<Duration>,
        /// Inject only transport or cancellation failures, never classification authority.
        fault: Fault,
        /// Shared controller cancellation boundary.
        cancel: &'a AtomicBool,
    }


    impl FreeCellIo for FakeIo<'_> {
        fn capture(&mut self) -> Result<CapturedFrame, String> {
            self.captures += 1;
            if self.fault == Fault::FourthCapture && self.captures == 4 {
                return Err("injected fourth capture failure".to_owned());
            }
            let frame = self.frames.pop_front().ok_or_else(|| "fixture sequence exhausted".to_owned())?;
            self.last_solver_state = freecell::solver_active(&frame).ok();


            if self.fault == Fault::StopAfterCapture
                || (self.fault == Fault::StopAfterFourthCapture && self.captures == 4)
            {
                self.cancel.store(true, Ordering::Release);
            }
            Ok(frame)
        }


        fn probe(&mut self) -> Result<(), String> {


            if self.fault == Fault::StopAtProbe {
                self.cancel.store(true, Ordering::Release);
            }
            Ok(())
        }


        fn input(&mut self, plan: StepPlan) -> Result<(), String> {
            self.sources.push(plan.before());


            if self.fault == Fault::Source { Err("injected uncertain source".to_owned()) } else { Ok(()) }
        }


        fn solver(&mut self) -> Result<(), String> {
            self.solver_clicks += 1;
            self.solver_states.push(self.last_solver_state);
            self.solver_capture_counts.push(self.captures);


            if self.fault == Fault::Solver { Err("injected uncertain Solver".to_owned()) } else { Ok(()) }
        }


        fn terminal(&mut self, stage: TerminalStage, point: PixelPoint) -> Result<(), String> {
            assert_ne!(point, PixelPoint::new(0, 0));
            self.controls.push(stage);
            self.control_points.push(point);


            if self.fault == Fault::Terminal
                || (self.fault == Fault::LevelUpTerminal && stage == TerminalStage::LevelUp)
            {
                Err("injected uncertain terminal".to_owned())
            } else {
                Ok(())
            }
        }


        fn wait(&mut self, duration: Duration) -> Result<(), String> {
            self.waits.push(duration);


            if self.fault == Fault::StopOnWait
                || (self.fault == Fault::StopOnSecondWait && self.waits.len() == 2)
                || (self.fault == Fault::StopOnThirdWait && self.waits.len() == 3)
            {
                self.cancel.store(true, Ordering::Release);
            }
            require_running(self.cancel)
        }
    }


    /// Decode the original committed PNG bytes through the production decoder.
    fn fixture(number: u8) -> CapturedFrame {
        let path = format!("{}/tests/fixtures/freecell-FC{number:02}.png", env!("CARGO_MANIFEST_DIR"));
        decode_png(&std::fs::read(path).unwrap()).unwrap()
    }


    /// Decode an unchanged native frame from the earlier late-geometry episode.
    fn final_confirmation_fixture(name: &str) -> CapturedFrame {
        let path = format!("{}/tests/fixtures/freecell-confirmation-20261016/{name}.png", env!("CARGO_MANIFEST_DIR"));
        decode_png(&std::fs::read(path).unwrap()).unwrap()
    }


    /// Model RGB(2,2,2) glyph washout in a native frame without changing its body or alpha.
    /// This controlled derivative is not the unavailable recorded frame 2450.
    fn interrupted_ok_fixture() -> CapturedFrame {
        let mut frame = final_confirmation_fixture("clean");
        let bounds = FREECELL_CONTROLS.spec(Control::Ok).bounds;
        for y in bounds.y..bounds.bottom() {
            for x in bounds.x..bounds.right() {
                let offset = y as usize * frame.stride + x as usize * 4;
                if frame.pixels[offset..offset + 3] == [0, 0, 0] {
                    assert_eq!(frame.pixels[offset + 3], 255);
                    frame.pixels[offset..offset + 3].fill(2);
                }
            }
        }
        assert_eq!(
            freecell_terminal::expected_control_point(&frame, TerminalStage::LevelUp).unwrap(),
            PixelPoint::new(0, 0)
        );
        frame
    }


    /// Near-black native storage supplies no scene, HALO or black-run evidence.
    fn unknown() -> CapturedFrame {
        CapturedFrame { width: 1_920, height: 1_080, stride: 7_680, format: PixelFormat::Rgba8, pixels: vec![1; 7_680 * 1_080] }
    }


    /// Remove lower source outlines while preserving the native board and active banner.
    fn active_without_halo() -> CapturedFrame {
        let mut frame = fixture(2);


        for y in 350..freecell::TOOLBAR_TOP {


            for x in 200..1_718 {
                let offset = y as usize * frame.stride + x * 4;
                frame.pixels[offset..offset + 3].copy_from_slice(&[20, 100, 75]);
            }
        }
        assert!(freecell::is_gameplay_scene(&frame).unwrap());
        assert!(freecell::solver_active(&frame).unwrap());
        assert_eq!(freecell::analyse(&frame).unwrap().prediction, PredictedAction::NoHighlight);
        frame
    }


    /// Construct a deterministic adapter with no fabricated classifier outputs.
    fn fake(frames: Vec<CapturedFrame>, cancel: &AtomicBool) -> FakeIo<'_> {
        FakeIo {
            frames: frames.into(), sources: Vec::new(), controls: Vec::new(), control_points: Vec::new(),
            solver_clicks: 0, solver_states: Vec::new(), solver_capture_counts: Vec::new(),
            last_solver_state: None, captures: 0, waits: Vec::new(), fault: Fault::None, cancel,
        }
    }


    /// Run the exact production driver with isolated bounded event channels.
    fn run(io: &mut FakeIo<'_>, limit: usize, observations: usize, latest: &mut Option<FrameObservation>) -> Result<RunOutcome, String> {
        let settings = StepRunSettings::new(AnimationSettleDelays::default(), limit).with_freecell_observation_limit(observations);
        run_with_settings(io, settings, latest)
    }


    /// Preserve custom snapshotted timing while exercising the production controller.
    fn run_with_settings(io: &mut FakeIo<'_>, settings: StepRunSettings, latest: &mut Option<FrameObservation>) -> Result<RunOutcome, String> {
        let (events, _receiver) = std::sync::mpsc::channel();
        let events = WorkerEventSink {
            events,
            latest_frame: super::super::LatestFrameSlot::default(),
            capture_context: std::sync::Mutex::new(None),
        };
        let mut attempted = false;
        let cancel = io.cancel;
        drive_run(io, settings, &events, cancel, latest, &mut attempted)
    }


    /// Finite one permits Solver activation but exactly one source click and result frame.
    #[test]
    fn initial_solver_activation_then_one_action_and_fresh_result() {
        let cancel = AtomicBool::new(false);
        let mut io = fake(vec![fixture(1), fixture(1), fixture(2), fixture(3)], &cancel);
        let mut latest = None;
        assert_eq!(run(&mut io, 1, 3, &mut latest), Ok(RunOutcome::Completed(1)));
        assert_eq!(io.solver_clicks, 1);
        assert_eq!(io.sources.len(), 1);
        assert_eq!(io.captures, 4);
        assert_eq!(io.solver_states, [Some(false)]);
        assert_eq!(io.solver_capture_counts, [2]);
        assert_eq!(io.waits[0], AnimationSettleDelays::default().freecell_game_start);
        assert!(io.controls.is_empty());
        assert_eq!(latest.unwrap().frame.pixels, fixture(3).pixels);
    }


    /// The editable start delay precedes a new capture, then the inactive-state click.
    #[test]
    fn editable_start_delay_is_snapshotted_before_fresh_solver_activation() {
        let cancel = AtomicBool::new(false);
        let mut io = fake(vec![fixture(1), fixture(1), fixture(2), fixture(3)], &cancel);
        let delays = AnimationSettleDelays::default().with_freecell_game_start_millis(1_750);
        let settings = StepRunSettings::new(delays, 1).with_freecell_observation_limit(3);
        assert_eq!(run_with_settings(&mut io, settings, &mut None), Ok(RunOutcome::Completed(1)));
        assert_eq!(io.waits, [Duration::from_millis(1_750), delays.freecell_settle, delays.freecell_settle]);
        assert_eq!(io.solver_states, [Some(false)]);
        assert_eq!(io.solver_capture_counts, [2]);
    }


    /// A source appearing during the start wait prevents an obsolete Solver click.
    #[test]
    fn start_delay_fresh_capture_uses_appearing_halo_without_solver_activation() {
        let cancel = AtomicBool::new(false);
        let mut io = fake(vec![fixture(1), fixture(2), fixture(3)], &cancel);
        assert_eq!(run(&mut io, 1, 3, &mut None), Ok(RunOutcome::Completed(1)));
        assert_eq!(io.solver_clicks, 0);
        assert_eq!(io.sources.len(), 1);
        assert_eq!(io.captures, 3);
        assert_eq!(io.waits[0], AnimationSettleDelays::default().freecell_game_start);
    }


    /// CELL sources, single cards and a multi-card run each consume one source action.
    #[test]
    fn native_source_classes_and_runs_use_one_click_without_pixel_proof() {


        for number in [2, 3, 4, 5, 6, 7, 8, 14, 16, 17] {
            let cancel = AtomicBool::new(false);
            let mut io = fake(vec![fixture(number), fixture(number)], &cancel);
            assert_eq!(run(&mut io, 1, 3, &mut None), Ok(RunOutcome::Completed(1)), "FC{number:02}");
            assert_eq!(io.sources.len(), 1);
            assert_eq!(io.solver_clicks, 0);
        }
    }


    /// A repeated fresh canonical source is a new recommendation, not rejected effect proof.
    #[test]
    fn same_fresh_target_can_authorise_consecutive_actions() {
        let cancel = AtomicBool::new(false);
        let mut io = fake(vec![fixture(2), fixture(2), fixture(2)], &cancel);
        assert_eq!(run(&mut io, 2, 3, &mut None), Ok(RunOutcome::Completed(2)));
        assert_eq!(io.sources.len(), 2);
        assert_eq!(io.sources[0], io.sources[1]);
    }


    /// Cards in flight or missing targets produce observations, not speculative Solver clicks.
    #[test]
    fn missing_halo_then_native_source_resumes_without_input_during_gap() {
        let cancel = AtomicBool::new(false);
        let mut io = fake(vec![active_without_halo(), fixture(7), fixture(8)], &cancel);
        assert_eq!(run(&mut io, 1, 3, &mut None), Ok(RunOutcome::Completed(1)));
        assert_eq!(io.sources.len(), 1);
        assert_eq!(io.solver_clicks, 0);
        assert_eq!(io.captures, 3);
    }


    /// Unsupported frames stop at the exact delayed allowance with no input or false win.
    #[test]
    fn unsupported_scene_has_bounded_input_free_observations() {
        let cancel = AtomicBool::new(false);
        let mut io = fake(vec![unknown(), unknown(), unknown()], &cancel);
        let mut latest = None;
        assert!(run(&mut io, 0, 2, &mut latest).unwrap_err().contains("allowance exhausted"));
        assert_eq!(io.captures, 3);
        assert!(io.sources.is_empty() && io.controls.is_empty());
        assert_eq!(io.solver_clicks, 0);
        assert!(latest.is_some());
    }


    /// Active no-HALO gameplay refreshes once, then exhausts bounded read-only recovery.
    #[test]
    fn active_solver_without_halo_refreshes_once_then_stops_after_allowance() {
        let cancel = AtomicBool::new(false);
        let mut io = fake(vec![active_without_halo(), active_without_halo(), active_without_halo(), active_without_halo(), active_without_halo()], &cancel);
        assert!(run(&mut io, 0, 2, &mut None).unwrap_err().contains("allowance exhausted"));
        assert_eq!(io.captures, 5);
        assert_eq!(io.solver_clicks, 1);
        assert_eq!(io.solver_states, [Some(true)]);
        assert_eq!(io.solver_capture_counts, [2]);
        assert!(io.sources.is_empty() && io.controls.is_empty());
    }


    /// One fresh no-HALO recapture permits one active-state refresh and fresh action.
    #[test]
    fn active_no_halo_refresh_follows_recapture_then_fresh_supported_source() {
        let cancel = AtomicBool::new(false);
        let mut io = fake(vec![active_without_halo(), active_without_halo(), fixture(7), fixture(8)], &cancel);
        assert_eq!(run(&mut io, 1, 3, &mut None), Ok(RunOutcome::Completed(1)));
        assert_eq!(io.solver_clicks, 1);
        assert_eq!(io.solver_states, [Some(true)]);
        assert_eq!(io.solver_capture_counts, [2]);
        assert_eq!(io.sources.len(), 1);
        assert_eq!(io.captures, 4);
        assert_eq!(io.waits, [AnimationSettleDelays::default().freecell_reobserve,
            AnimationSettleDelays::default().freecell_settle, AnimationSettleDelays::default().freecell_settle]);
    }


    /// A refresh that leaves Solver off cannot spend a second activation reserve.
    #[test]
    fn solver_refresh_to_inactive_state_never_activates_again_in_same_context() {
        let cancel = AtomicBool::new(false);
        let mut io = fake(vec![active_without_halo(), active_without_halo(), fixture(1), fixture(1), fixture(1)], &cancel);
        assert!(run(&mut io, 0, 2, &mut None).unwrap_err().contains("allowance exhausted"));
        assert_eq!(io.solver_clicks, 1);
        assert_eq!(io.solver_states, [Some(true)]);
        assert_eq!(io.solver_capture_counts, [2]);
        assert_eq!(io.captures, 5);
        assert!(io.sources.is_empty() && io.controls.is_empty());
    }


    /// Win entry after either pre-Solver wait takes priority over activation or refresh.
    #[test]
    fn win_during_start_or_active_observation_wait_prevents_solver_input() {


        for initial in [fixture(1), active_without_halo()] {
            let cancel = AtomicBool::new(false);
            let mut io = fake(vec![initial, fixture(9)], &cancel);
            assert_eq!(run(&mut io, 1, 3, &mut None), Ok(RunOutcome::GameWon(0)));
            assert_eq!(io.solver_clicks, 0);
            assert_eq!(io.captures, 2);
            assert!(io.sources.is_empty() && io.controls.is_empty());
        }
    }


    /// STOP during either pre-Solver wait prevents input and retains its last frame.
    #[test]
    fn stop_during_start_or_active_observation_wait_sends_no_solver_input() {


        for initial in [fixture(1), active_without_halo()] {
            let cancel = AtomicBool::new(false);
            let expected_pixels = initial.pixels.clone();
            let mut io = fake(vec![initial], &cancel);
            io.fault = Fault::StopOnWait;
            let mut latest = None;
            assert!(run(&mut io, 0, 3, &mut latest).unwrap_err().contains("STOP"));
            assert_eq!(io.solver_clicks, 0);
            assert_eq!(io.captures, 1);
            assert!(io.sources.is_empty() && io.controls.is_empty());
            assert_eq!(latest.unwrap().frame.pixels, expected_pixels);
        }
    }


    /// Both terminal-at-start and source-to-win frames drive the actual ordered restart.
    #[test]
    fn native_game_win_score_ok_new_game_play_solver_restart() {


        for prefix in [false, true] {
            let cancel = AtomicBool::new(false);
            let mut frames = Vec::new();


            if prefix { frames.push(fixture(8)); }
            frames.extend([fixture(9), fixture(10), fixture(10), fixture(11), fixture(12), fixture(13), fixture(13), fixture(2), fixture(3)]);
            let mut io = fake(frames, &cancel);
            let mut latest = None;
            assert_eq!(run(&mut io, 0, 3, &mut latest).unwrap_err(), "fixture sequence exhausted");
            assert_eq!(io.controls, [TerminalStage::Score, TerminalStage::LevelUp, TerminalStage::NewGame, TerminalStage::Play]);
            assert_eq!(io.control_points, [PixelPoint::new(960, 550),
                PixelPoint::new(941, 822), PixelPoint::new(810, 861), PixelPoint::new(680, 762)]);
            assert_eq!(io.solver_clicks, 1);
            assert_eq!(io.solver_states, [Some(false)]);
            assert_eq!(io.solver_capture_counts, [if prefix { 8 } else { 7 }]);
            assert_eq!(io.sources.len(), if prefix { 3 } else { 2 });
            let delays = AnimationSettleDelays::default();
            let mut expected_waits = Vec::new();


            if prefix { expected_waits.push(delays.freecell_settle); }
            expected_waits.extend([
                LEVEL_UP_APPEAR_DELAY, Duration::from_millis(200),
                POST_GAME_STAGE_DELAY, POST_GAME_STAGE_DELAY, POST_GAME_STAGE_DELAY,
                delays.freecell_game_start,
                delays.freecell_settle, delays.freecell_settle, delays.freecell_settle,
            ]);
            assert_eq!(io.waits, expected_waits);
        }
    }


    /// A score skip can reach New Game directly, immediately or after animation.
    #[test]
    fn native_no_level_up_restart_accepts_all_three_new_game_attempts() {


        for attempt in 1..=TERMINAL_CONTROL_SEARCH_ATTEMPTS {
            let cancel = AtomicBool::new(false);
            let mut frames = vec![fixture(9); attempt];
            frames.extend([fixture(15), fixture(12), fixture(13), fixture(13), fixture(2)]);
            let mut io = fake(frames, &cancel);
            assert_eq!(run(&mut io, 0, 3, &mut None).unwrap_err(), "fixture sequence exhausted");
            assert_eq!(io.controls, [TerminalStage::Score, TerminalStage::NewGame, TerminalStage::Play]);
            assert_eq!(io.solver_clicks, 1);
            assert_eq!(io.sources.len(), 1);
        }
    }


    /// The exact stopped native New Game frame can resume a continuous restart.
    #[test]
    fn native_new_game_entry_resumes_without_score_or_level_up_input() {
        let cancel = AtomicBool::new(false);
        let mut io = fake(vec![fixture(15), fixture(12), fixture(13), fixture(13), fixture(2)], &cancel);
        assert_eq!(run(&mut io, 0, 3, &mut None).unwrap_err(), "fixture sequence exhausted");
        assert_eq!(io.controls, [TerminalStage::NewGame, TerminalStage::Play]);
        assert_eq!(io.solver_clicks, 1);
        assert_eq!(io.sources.len(), 1);
        let mut io = fake(vec![fixture(15)], &cancel);
        assert_eq!(run(&mut io, 1, 3, &mut None), Ok(RunOutcome::GameWon(0)));
        assert!(io.controls.is_empty() && io.sources.is_empty());
        assert_eq!(io.solver_clicks, 0);
    }


    /// Contradictory local controls stop after score skip, without choosing either.
    #[test]
    fn simultaneous_post_score_ok_and_new_game_are_uncertain() {
        let cancel = AtomicBool::new(false);
        let mut both = fixture(15);
        let ok = fixture(10);


        for y in 788..838 {
            let start = y * both.stride + 848 * 4;
            let end = start + 223 * 4;
            both.pixels[start..end].copy_from_slice(&ok.pixels[start..end]);
        }
        assert!(freecell_terminal::expected_control_ready(&both, TerminalStage::LevelUp).unwrap());
        assert!(freecell_terminal::expected_control_ready(&both, TerminalStage::NewGame).unwrap());
        let mut io = fake(vec![fixture(9), both], &cancel);
        let mut latest = None;
        assert!(run(&mut io, 0, 3, &mut latest).unwrap_err().contains("both OK and New Game"));
        assert_eq!(io.controls, [TerminalStage::Score]);
        assert_eq!(io.captures, 2);
        assert!(io.sources.is_empty());
        assert_eq!(io.solver_clicks, 0);
        assert!(latest.is_some());
    }


    /// Finite terminal entry never sends restart controls; standalone OK is not a win.
    #[test]
    fn finite_win_and_unconfirmed_level_up_remain_input_free() {
        let cancel = AtomicBool::new(false);
        let mut io = fake(vec![fixture(9)], &cancel);
        assert_eq!(run(&mut io, 1, 2, &mut None), Ok(RunOutcome::GameWon(0)));
        assert!(io.controls.is_empty() && io.sources.is_empty());
        let mut io = fake(vec![fixture(10), fixture(10), fixture(10)], &cancel);
        assert!(run(&mut io, 0, 2, &mut None).unwrap_err().contains("allowance exhausted"));
        assert!(io.controls.is_empty());
    }


    /// Delayed readiness observes the expected region rather than repeating the prior click.
    #[test]
    fn terminal_animation_recaptures_without_replaying_score_input() {
        let cancel = AtomicBool::new(false);
        let mut io = fake(vec![fixture(9), fixture(9), fixture(10), fixture(10), fixture(11), fixture(12), fixture(13), fixture(13), fixture(2)], &cancel);
        assert_eq!(run(&mut io, 0, 3, &mut None).unwrap_err(), "fixture sequence exhausted");
        assert_eq!(io.controls, [TerminalStage::Score, TerminalStage::LevelUp, TerminalStage::NewGame, TerminalStage::Play]);
    }


    /// Three failed post-score frames retain the last preview without repeating Score.
    #[test]
    fn terminal_third_miss_retains_latest_preview_without_replaying_prior_control() {
        let cancel = AtomicBool::new(false);
        let mut failed = fixture(9);
        failed.pixels[..4].copy_from_slice(&[17, 23, 31, 255]);
        let mut io = fake(vec![fixture(9), fixture(9), fixture(9), failed.clone(), fixture(10)], &cancel);
        let settings = StepRunSettings::new(AnimationSettleDelays::default(), 0)
            .with_freecell_observation_limit(20);
        let (sender, receiver) = std::sync::mpsc::channel();
        let slot = super::super::LatestFrameSlot::default();
        let events = WorkerEventSink {
            events: sender, latest_frame: slot.clone(),
            capture_context: std::sync::Mutex::new(None),
        };
        let mut latest = None;
        let mut input_attempted = false;
        assert!(drive_run(&mut io, settings, &events, &cancel, &mut latest, &mut input_attempted)
            .unwrap_err().contains("after 3 searches and 2 fresh recaptures"));
        assert_eq!(io.controls, [TerminalStage::Score]);
        assert_eq!(io.control_points, [PixelPoint::new(960, 550)]);
        assert_eq!(io.captures, 4);
        assert_eq!(io.frames.len(), 1, "a fourth post-score frame must remain unread");
        assert_eq!(io.waits, [LEVEL_UP_APPEAR_DELAY, settings.animation_delays().freecell_reobserve,
            settings.animation_delays().freecell_reobserve]);
        assert_eq!(io.solver_clicks, 0);
        assert!(io.sources.is_empty());
        assert_eq!(latest.unwrap().frame.pixels, failed.pixels);
        let preview = slot.take().unwrap();
        assert!(preview.diagnostic);
        assert_eq!(preview.prediction, None);
        assert_eq!(preview.frame.pixels, failed.pixels);
        let summaries: Vec<_> = receiver.try_iter().filter_map(|notification| match notification.event {
            WorkerEvent::Log(message) if message.contains("search summary:") => Some(message),
            _ => None,
        }).collect();
        assert_eq!(summaries.len(), 2);
        assert!(summaries.iter().all(|summary|
            summary.contains("searches=3; recaptures=2; second_grab=true")
                && summary.ends_with("outcome=not found.")));
    }


    /// STOP at the probe prevents input, while late acquisition STOP retains its new frame.
    #[test]
    fn stop_and_late_capture_stop_preserve_input_boundaries() {


        for fault in [Fault::StopAtProbe, Fault::StopAfterCapture] {
            let cancel = AtomicBool::new(false);
            let mut io = fake(vec![fixture(2)], &cancel);
            io.fault = fault;
            let mut latest = None;
            assert!(run(&mut io, 1, 2, &mut latest).unwrap_err().contains("STOP"));
            assert!(io.sources.is_empty());
            assert_eq!(latest.unwrap().frame.pixels, fixture(2).pixels);
        }
    }


    /// Malformed decoded storage survives classifier rejection without becoming authority.
    #[test]
    fn classifier_failure_retains_the_latest_diagnostic_pixels() {
        let cancel = AtomicBool::new(false);
        let mut frame = fixture(2);
        frame.pixels.truncate(12);
        let mut io = fake(vec![frame], &cancel);
        let mut latest = None;
        assert!(run(&mut io, 0, 2, &mut latest).unwrap_err().contains("analysis failed"));
        assert_eq!(latest.unwrap().frame.pixels.len(), 12);
        assert!(io.sources.is_empty() && io.controls.is_empty());
    }


    /// Each uncertain input class is attempted once and never followed by a replacement input.
    #[test]
    fn uncertain_source_solver_and_terminal_are_not_replayed() {


        for (fault, number) in [(Fault::Source, 2), (Fault::Solver, 1), (Fault::Terminal, 9)] {
            let cancel = AtomicBool::new(false);
            let mut frames = vec![fixture(number)];


            if fault == Fault::Solver { frames.push(fixture(number)); }
            let mut io = fake(frames, &cancel);
            io.fault = fault;
            assert!(run(&mut io, 0, 2, &mut None).unwrap_err().contains("uncertain"));
            assert_eq!(io.sources.len() + io.controls.len() + io.solver_clicks, 1);
            assert_eq!(io.captures, if fault == Fault::Solver { 2 } else { 1 });
        }
    }


    /// An uncertain active-state refresh consumes its reserve without later captures.
    #[test]
    fn uncertain_active_solver_refresh_is_attempted_once_without_reobservation() {
        let cancel = AtomicBool::new(false);
        let mut io = fake(vec![active_without_halo(), active_without_halo(), fixture(7)], &cancel);
        io.fault = Fault::Solver;
        assert!(run(&mut io, 0, 3, &mut None).unwrap_err().contains("uncertain"));
        assert_eq!(io.solver_clicks, 1);
        assert_eq!(io.solver_states, [Some(true)]);
        assert_eq!(io.solver_capture_counts, [2]);
        assert_eq!(io.captures, 2);
        assert_eq!(io.frames.len(), 1);
        assert!(io.sources.is_empty() && io.controls.is_empty());
    }


    /// Invalid source budgets are rejected without touching capture or guest input.
    #[test]
    fn operation_budget_accepts_continuous_and_bounded_limits() {
        assert!(validate_operation_limit(0).is_ok());
        assert!(validate_operation_limit(FREECELL_MAX_MULTI_STEP_ACTIONS).is_ok());
        assert!(validate_operation_limit(FREECELL_MAX_MULTI_STEP_ACTIONS + 1).is_err());
    }

    /// Ordinary menus can match immediately; OK needs two fresh settled observations.
    #[test]
    fn bounded_control_search_metrics_cover_all_three_success_attempts() {
        for (number, stage, expected) in [
            (10, TerminalStage::LevelUp, PixelPoint::new(941, 822)),
            (11, TerminalStage::NewGame, PixelPoint::new(810, 861)),
            (12, TerminalStage::Play, PixelPoint::new(680, 762))
        ] {
            let confirmations = if stage == TerminalStage::LevelUp { 2 } else { 1 };
            for attempt in confirmations..=TERMINAL_CONTROL_SEARCH_ATTEMPTS {
                let cancel = AtomicBool::new(false);
                let mut frames = vec![fixture(9); attempt - confirmations];
                frames.extend(vec![fixture(number); confirmations]);
                frames.push(fixture(9));
                let mut io = fake(frames, &cancel);
                let settings = StepRunSettings::new(AnimationSettleDelays::default(), 0);
                let (sender, receiver) = std::sync::mpsc::channel();
                let events = WorkerEventSink {
                    events: sender, latest_frame: super::super::LatestFrameSlot::default(),
                    capture_context: std::sync::Mutex::new(None),
                };
                let mut latest = None;
                capture_latest(&mut io, &mut latest, &events, &cancel).unwrap();
                let point = await_expected_control(&mut io, &mut latest, stage, settings, &events, &cancel).unwrap();
                assert_eq!(point, expected);
                assert_eq!(io.captures, attempt);
                assert_eq!(io.frames.len(), 1, "no capture follows a successful search");
                let mut expected_waits = vec![settings.animation_delays().freecell_reobserve; attempt - 1];
                if stage == TerminalStage::LevelUp { *expected_waits.last_mut().unwrap() = Duration::from_millis(200); }
                assert_eq!(io.waits, expected_waits);
                io.terminal(stage, point).unwrap();
                assert_eq!(io.controls, [stage]);
                assert_eq!(io.control_points, [expected]);
                let logs: Vec<_> = receiver.try_iter().filter_map(|event| match event.event {
                    WorkerEvent::Log(message) => Some(message), _ => None,
                }).collect();
                let summary = logs.iter().find(|log| log.contains("search summary:")).unwrap();
                assert!(summary.contains(&format!(
                    "searches={attempt}; recaptures={}; second_grab={}", attempt - 1, attempt > 1)));
                assert!(summary.contains("scan_total_ms=") && summary.contains("wall_ms="));
                assert!(summary.ends_with("outcome=matched."));
                assert_eq!(logs.iter().filter(|log| log.contains("search summary:")).count(), 1);
            }
        }
    }


    /// Three missing menu frames retain the third preview and leave a fourth unread.
    #[test]
    fn every_control_third_miss_retains_the_failed_frame_without_input() {
        for stage in [TerminalStage::LevelUp, TerminalStage::NewGame, TerminalStage::Play] {
            let cancel = AtomicBool::new(false);
            let mut second = fixture(9);
            second.pixels[..4].copy_from_slice(&[11, 13, 19, 255]);
            let mut failed = fixture(9);
            failed.pixels[..4].copy_from_slice(&[17, 23, 31, 255]);
            let mut io = fake(vec![fixture(9), second, failed.clone(), fixture(9)], &cancel);
            let settings = StepRunSettings::new(AnimationSettleDelays::default(), 0);
            let (sender, receiver) = std::sync::mpsc::channel();
            let slot = super::super::LatestFrameSlot::default();
            let events = WorkerEventSink {
                events: sender, latest_frame: slot.clone(),
                capture_context: std::sync::Mutex::new(None),
            };
            let mut latest = None;
            capture_latest(&mut io, &mut latest, &events, &cancel).unwrap();
            assert!(await_expected_control(&mut io, &mut latest, stage, settings, &events, &cancel)
                .unwrap_err().contains("latest failed frame retained"));
            assert_eq!(io.captures, 3);
            assert_eq!(io.frames.len(), 1, "a fourth frame must remain unread");
            assert_eq!(io.waits, vec![settings.animation_delays().freecell_reobserve; 2]);
            assert!(io.controls.is_empty() && io.sources.is_empty());
            assert_eq!(io.solver_clicks, 0);
            assert_eq!(latest.unwrap().frame.pixels, failed.pixels);
            let preview = slot.take().unwrap();
            assert!(preview.diagnostic);
            assert_eq!(preview.frame.pixels, failed.pixels);
            let logs: Vec<_> = receiver.try_iter().filter_map(|event| match event.event {
                WorkerEvent::Log(message) => Some(message), _ => None,
            }).collect();
            let summary = logs.iter().find(|log| log.contains("search summary:")).unwrap();
            assert!(summary.contains("searches=3; recaptures=2; second_grab=true"));
            assert!(summary.ends_with("outcome=not found."));
        }
    }


    /// STOP in the second recapture wait prevents a third search and all menu input.
    #[test]
    fn cancellation_before_third_terminal_search_retains_second_frame() {
        for optional_pair in [false, true] {
            let cancel = AtomicBool::new(false);
            let mut second = fixture(9);
            second.pixels[..4].copy_from_slice(&[11, 13, 19, 255]);
            let mut io = fake(vec![fixture(9), second.clone(), fixture(10)], &cancel);
            io.fault = Fault::StopOnSecondWait;
            let settings = StepRunSettings::new(AnimationSettleDelays::default(), 0);
            let (sender, receiver) = std::sync::mpsc::channel();
            let slot = super::super::LatestFrameSlot::default();
            let events = WorkerEventSink {
                events: sender, latest_frame: slot.clone(),
                capture_context: std::sync::Mutex::new(None),
            };
            let mut latest = None;
            capture_latest(&mut io, &mut latest, &events, &cancel).unwrap();
            let result = if optional_pair {
                await_post_score_control(&mut io, &mut latest, settings, &events, &cancel).map(|_| ())
            } else {
                await_expected_control(&mut io, &mut latest, TerminalStage::LevelUp,
                    settings, &events, &cancel).map(|_| ())
            };
            assert!(result.unwrap_err().contains("STOP"));
            assert_eq!(io.captures, 2);
            assert_eq!(io.frames.len(), 1, "STOP must prevent the third capture");
            assert_eq!(io.waits, vec![settings.animation_delays().freecell_reobserve; 2]);
            assert!(io.controls.is_empty() && io.sources.is_empty());
            assert_eq!(io.solver_clicks, 0);
            assert_eq!(latest.unwrap().frame.pixels, second.pixels);
            assert_eq!(slot.take().unwrap().frame.pixels, second.pixels);
            let summaries: Vec<_> = receiver.try_iter().filter_map(|event| match event.event {
                WorkerEvent::Log(message) if message.contains("search summary:") => Some(message),
                _ => None,
            }).collect();
            assert_eq!(summaries.len(), if optional_pair { 2 } else { 1 });
            assert!(summaries.iter().all(|summary|
                summary.contains("searches=2; recaptures=1; second_grab=true")
                    && summary.ends_with("outcome=wait stopped.")));
        }
    }


    /// Optional New Game can match immediately; OK needs fresh settled confirmation.
    #[test]
    fn post_score_optional_pair_accepts_each_branch_on_all_three_attempts() {
        for (ready, expected_stage, expected_point) in [
            (fixture(10), TerminalStage::LevelUp, PixelPoint::new(941, 822)),
            (fixture(15), TerminalStage::NewGame, PixelPoint::new(863, 861))
        ] {
            let confirmations = if expected_stage == TerminalStage::LevelUp { 2 } else { 1 };
            for attempt in confirmations..=TERMINAL_CONTROL_SEARCH_ATTEMPTS {
                let cancel = AtomicBool::new(false);
                let mut frames = vec![fixture(9); attempt - confirmations];
                frames.extend(vec![ready.clone(); confirmations]);
                frames.push(fixture(9));
                let mut io = fake(frames, &cancel);
                let settings = StepRunSettings::new(AnimationSettleDelays::default(), 0);
                let (sender, receiver) = std::sync::mpsc::channel();
                let events = WorkerEventSink {
                    events: sender, latest_frame: super::super::LatestFrameSlot::default(),
                    capture_context: std::sync::Mutex::new(None),
                };
                let mut latest = None;
                capture_latest(&mut io, &mut latest, &events, &cancel).unwrap();
                let (stage, point) = await_post_score_control(
                    &mut io, &mut latest, settings, &events, &cancel).unwrap();
                assert_eq!((stage, point), (expected_stage, expected_point));
                assert_eq!(io.captures, attempt);
                assert_eq!(io.frames.len(), 1);
                let mut expected_waits = vec![settings.animation_delays().freecell_reobserve; attempt - 1];
                if expected_stage == TerminalStage::LevelUp { *expected_waits.last_mut().unwrap() = Duration::from_millis(200); }
                assert_eq!(io.waits, expected_waits);
                assert!(io.controls.is_empty(), "observation alone must not deliver menu input");
                io.terminal(stage, point).unwrap();
                assert_eq!(io.controls, [expected_stage]);
                assert_eq!(io.control_points, [expected_point]);
                assert!(io.sources.is_empty());
                assert_eq!(io.solver_clicks, 0);
                let summaries: Vec<_> = receiver.try_iter().filter_map(|notification| match notification.event {
                    WorkerEvent::Log(message) if message.contains("search summary:") => Some(message),
                    _ => None,
                }).collect();
                assert_eq!(summaries.len(), 2);
                let metrics = format!("searches={attempt}; recaptures={}; second_grab={}",
                    attempt - 1, attempt > 1);
                assert!(summaries.iter().all(|summary| summary.contains(&metrics)));
                assert_eq!(summaries.iter().filter(|summary| summary.ends_with("outcome=matched.")).count(), 1);
                assert_eq!(summaries.iter().filter(|summary| summary.ends_with("outcome=optional absent.")).count(), 1);
            }
        }
    }

    /// A clean third observation gets one controlled fresh witness, using its current coordinate.
    #[test]
    fn third_clean_ok_reserves_one_final_fresh_confirmation_and_latest_point() {
        for optional_pair in [false, true] {
            let entering = final_confirmation_fixture("entering");
            let tinted = final_confirmation_fixture("tinted");
            let clean = final_confirmation_fixture("clean");
            let previous_point = freecell_terminal::expected_control_point(&clean, TerminalStage::LevelUp).unwrap();
            let mut fresh = clean.clone();
            let expected = PixelPoint::new(FREECELL_CONTROLS.spec(Control::Ok).bounds.x as i32, previous_point.y);
            for x in expected.x..expected.x + 5 {
                let offset = expected.y as usize * fresh.stride + x as usize * 4;
                fresh.pixels[offset..offset + 3].fill(0);
            }
            assert_eq!(previous_point, PixelPoint::new(942, 770));
            assert_eq!(freecell_terminal::expected_control_point(&fresh, TerminalStage::LevelUp).unwrap(), expected);
            assert_ne!(expected, previous_point);
            // The fourth observation is a controlled native-frame derivative, not
            // a recorded live response acquired 200 ms after the third capture.
            let cancel = AtomicBool::new(false);
            let mut io = fake(vec![entering, tinted, clean, fresh.clone(), fixture(9)], &cancel);
            let settings = StepRunSettings::new(AnimationSettleDelays::default(), 0);
            let (sender, receiver) = std::sync::mpsc::channel();
            let events = WorkerEventSink {
                events: sender, latest_frame: super::super::LatestFrameSlot::default(),
                capture_context: std::sync::Mutex::new(None),
            };
            let mut latest = None;
            capture_latest(&mut io, &mut latest, &events, &cancel).unwrap();
            let point = if optional_pair {
                let (stage, point) = await_post_score_control(
                    &mut io, &mut latest, settings, &events, &cancel).unwrap();
                assert_eq!(stage, TerminalStage::LevelUp);
                point
            } else {
                await_expected_control(&mut io, &mut latest, TerminalStage::LevelUp,
                    settings, &events, &cancel).unwrap()
            };
            assert_eq!(point, expected);
            assert_eq!(io.captures, 4);
            assert_eq!(io.frames.len(), 1, "a fifth observation is never needed");
            assert_eq!(io.waits, [settings.animation_delays().freecell_reobserve,
                settings.animation_delays().freecell_reobserve, Duration::from_millis(200)]);
            assert!(io.controls.is_empty() && io.sources.is_empty());
            io.terminal(TerminalStage::LevelUp, point).unwrap();
            assert_eq!(io.controls, [TerminalStage::LevelUp]);
            assert_eq!(io.control_points, [expected]);
            assert_eq!(io.solver_clicks, 0);
            assert_eq!(latest.unwrap().frame.pixels, fresh.pixels);
            let logs: Vec<_> = receiver.try_iter().filter_map(|event| match event.event {
                WorkerEvent::Log(message) => Some(message), _ => None,
            }).collect();
            assert!(logs.iter().any(|message| message.starts_with("Free Cell OK search summary: searches=4; recaptures=3;")
                && message.ends_with("outcome=matched.")));
            if optional_pair {
                assert!(logs.iter().any(|message| message.starts_with("Free Cell New Game search summary: searches=3; recaptures=2;")));
            }
        }
    }

    /// A visible static OK remains input-free if STOP interrupts its confirmation.
    #[test]
    fn stop_during_ok_readiness_confirmation_sends_no_terminal_input() {
        for optional_pair in [false, true] {
            let cancel = AtomicBool::new(false);
            let ready = fixture(10);
            let mut io = fake(vec![ready.clone(), ready.clone()], &cancel);
            io.fault = Fault::StopOnWait;
            let settings = StepRunSettings::new(AnimationSettleDelays::default(), 0);
            let (sender, _receiver) = std::sync::mpsc::channel();
            let events = WorkerEventSink {
                events: sender, latest_frame: super::super::LatestFrameSlot::default(),
                capture_context: std::sync::Mutex::new(None),
            };
            let mut latest = None;
            capture_latest(&mut io, &mut latest, &events, &cancel).unwrap();
            let result = if optional_pair {
                await_post_score_control(&mut io, &mut latest, settings, &events, &cancel).map(|_| ())
            } else {
                await_expected_control(&mut io, &mut latest, TerminalStage::LevelUp,
                    settings, &events, &cancel).map(|_| ())
            };
            assert!(result.unwrap_err().contains("STOP"));
            assert_eq!(io.captures, 1);
            assert_eq!(io.frames.len(), 1);
            assert_eq!(io.waits, [Duration::from_millis(200)]);
            assert!(io.controls.is_empty() && io.sources.is_empty());
            assert_eq!(io.solver_clicks, 0);
            assert_eq!(latest.unwrap().frame.pixels, ready.pixels);
        }
    }

    /// An acknowledged OK followed by a retained modal never authorises another OK.
    #[test]
    fn settled_ok_acknowledgement_is_not_replayed_when_modal_remains_visible() {
        let cancel = AtomicBool::new(false);
        let ready = fixture(10);
        let mut frames = vec![fixture(9)];
        frames.extend(vec![ready.clone(); 6]);
        let mut io = fake(frames, &cancel);
        let mut latest = None;
        assert!(run(&mut io, 0, 3, &mut latest).unwrap_err().contains("New Game"));
        assert_eq!(io.controls, [TerminalStage::Score, TerminalStage::LevelUp]);
        assert_eq!(io.captures, 6);
        assert_eq!(io.frames.len(), 1);
        assert!(io.sources.is_empty());
        assert_eq!(io.solver_clicks, 0);
        assert_eq!(latest.unwrap().frame.pixels, ready.pixels);
    }

    /// Recorded entrance geometry and missing exact-black glyphs defer OK until frame three.
    #[test]
    fn captured_entrance_fading_and_settled_ok_use_latest_caption_point() {
        for optional_pair in [false, true] {
            let early = decode_png(include_bytes!("../../tests/fixtures/game-win-readiness/early.png")).unwrap();
            let fading = decode_png(include_bytes!("../../tests/fixtures/game-win-readiness/fading.png")).unwrap();
            let settled = decode_png(include_bytes!("../../tests/fixtures/game-win-readiness/settled.png")).unwrap();
            let cancel = AtomicBool::new(false);
            let mut io = fake(vec![early, fading, settled.clone(), settled.clone()], &cancel);
            let settings = StepRunSettings::new(AnimationSettleDelays::default(), 0);
            let (sender, _receiver) = std::sync::mpsc::channel();
            let events = WorkerEventSink {
                events: sender, latest_frame: super::super::LatestFrameSlot::default(),
                capture_context: std::sync::Mutex::new(None),
            };
            let mut latest = None;
            capture_latest(&mut io, &mut latest, &events, &cancel).unwrap();
            let point = if optional_pair {
                let (stage, point) = await_post_score_control(
                    &mut io, &mut latest, settings, &events, &cancel).unwrap();
                assert_eq!(stage, TerminalStage::LevelUp);
                point
            } else {
                await_expected_control(&mut io, &mut latest, TerminalStage::LevelUp,
                    settings, &events, &cancel).unwrap()
            };
            assert_eq!(point, PixelPoint::new(941, 770), "use the current caption, not the cursor outline");
            assert_eq!(io.captures, 3);
            assert_eq!(io.frames.len(), 1);
            assert!(io.controls.is_empty() && io.sources.is_empty());
            io.terminal(TerminalStage::LevelUp, point).unwrap();
            assert_eq!(io.controls, [TerminalStage::LevelUp]);
            assert_eq!(io.control_points, [point]);
            assert_eq!(io.solver_clicks, 0);
            assert_eq!(latest.unwrap().frame.pixels, settled.pixels);
        }
    }

    /// Missing OK ink cannot expose a visible New Game under its foreground button.
    #[test]
    fn foreground_ok_with_missing_black_run_blocks_new_game_underlay() {
        let mut foreground = decode_png(include_bytes!("../../tests/fixtures/game-win-readiness/settled.png")).unwrap();
        let menu = fixture(15);
        let new_game = FREECELL_CONTROLS.spec(Control::NewGame).bounds;
        for y in new_game.y..new_game.bottom() {
            let start = y as usize * foreground.stride + new_game.x as usize * 4;
            let end = start + new_game.width as usize * 4;
            foreground.pixels[start..end].copy_from_slice(&menu.pixels[start..end]);
        }
        let ok = FREECELL_CONTROLS.spec(Control::Ok).bounds;
        for y in ok.y..ok.bottom() {
            for x in ok.x..ok.right() {
                let offset = y as usize * foreground.stride + x as usize * 4;
                if foreground.pixels[offset..offset + 3] == [0, 0, 0] {
                    foreground.pixels[offset..offset + 3].copy_from_slice(&[35, 35, 35]);
                }
            }
        }
        assert_eq!(crate::terminal_controls::control_point(&foreground, FREECELL_CONTROLS.spec(Control::Ok)).unwrap(),
            PixelPoint::new(0, 0));
        assert_ne!(crate::terminal_controls::control_point(&foreground, FREECELL_CONTROLS.spec(Control::NewGame)).unwrap(),
            PixelPoint::new(0, 0));
        let cancel = AtomicBool::new(false);
        let mut io = fake(vec![foreground.clone(), foreground.clone(), foreground.clone(), menu], &cancel);
        let settings = StepRunSettings::new(AnimationSettleDelays::default(), 0);
        let (sender, _receiver) = std::sync::mpsc::channel();
        let events = WorkerEventSink {
            events: sender, latest_frame: super::super::LatestFrameSlot::default(),
            capture_context: std::sync::Mutex::new(None),
        };
        let mut latest = None;
        capture_latest(&mut io, &mut latest, &events, &cancel).unwrap();
        assert!(await_post_score_control(&mut io, &mut latest, settings, &events, &cancel)
            .unwrap_err().contains("after 3 searches and 2 fresh recaptures"));
        assert_eq!(io.captures, 3);
        assert_eq!(io.frames.len(), 1);
        assert!(io.controls.is_empty() && io.sources.is_empty());
        assert_eq!(io.solver_clicks, 0);
        assert_eq!(latest.unwrap().frame.pixels, foreground.pixels);
    }

    /// Entrance uses the ordinary delay; a full-size glyph gets one 200 ms fresh confirmation.
    #[test]
    fn full_size_ok_confirmation_uses_200ms_and_the_latest_frames_point() {
        for optional_pair in [false, true] {
            let entering = decode_png(include_bytes!("../../tests/fixtures/game-win-readiness/early.png")).unwrap();
            let clean = fixture(10);
            let first_point = freecell_terminal::expected_control_point(&clean, TerminalStage::LevelUp).unwrap();
            let mut fresh = clean.clone();
            let latest_point = PixelPoint::new(FREECELL_CONTROLS.spec(Control::Ok).bounds.x as i32, first_point.y);
            for x in latest_point.x..latest_point.x + 5 {
                let offset = latest_point.y as usize * fresh.stride + x as usize * 4;
                fresh.pixels[offset..offset + 3].fill(0);
            }
            assert_ne!(latest_point, first_point);
            assert_eq!(freecell_terminal::expected_control_point(&fresh, TerminalStage::LevelUp).unwrap(), latest_point);
            // This fake supplies a new native-frame derivative after the requested
            // 200 ms interval; it does not claim a recorded live 200 ms response.
            let cancel = AtomicBool::new(false);
            let mut io = fake(vec![entering, clean, fresh.clone(), fixture(9)], &cancel);
            let settings = StepRunSettings::new(AnimationSettleDelays::default()
                .with_freecell_millis(1_000, 2_300), 0);
            let (sender, _receiver) = std::sync::mpsc::channel();
            let events = WorkerEventSink {
                events: sender, latest_frame: super::super::LatestFrameSlot::default(),
                capture_context: std::sync::Mutex::new(None),
            };
            let mut latest = None;
            capture_latest(&mut io, &mut latest, &events, &cancel).unwrap();
            let point = if optional_pair {
                let (stage, point) = await_post_score_control(
                    &mut io, &mut latest, settings, &events, &cancel).unwrap();
                assert_eq!(stage, TerminalStage::LevelUp);
                point
            } else {
                await_expected_control(&mut io, &mut latest, TerminalStage::LevelUp,
                    settings, &events, &cancel).unwrap()
            };
            assert_eq!(io.waits, [Duration::from_millis(2_300), Duration::from_millis(200)]);
            assert_eq!(io.captures, TERMINAL_CONTROL_SEARCH_ATTEMPTS);
            assert_eq!(io.frames.len(), 1, "fresh confirmation consumes the existing third observation");
            assert_eq!(point, latest_point, "the earlier valid candidate is never replayed");
            assert!(io.controls.is_empty() && io.sources.is_empty());
            io.terminal(TerminalStage::LevelUp, point).unwrap();
            assert_eq!(io.controls, [TerminalStage::LevelUp]);
            assert_eq!(io.control_points, [latest_point]);
            assert_eq!(io.solver_clicks, 0);
            assert_eq!(latest.unwrap().frame.pixels, fresh.pixels);
        }
    }

    /// Missing current glyphs clear short-wait permission; missing body also clears geometry.
    #[test]
    fn a_missing_ok_observation_restores_normal_delay_and_bounds_final_confirmation() {
        for optional_pair in [false, true] {
            for missing_body in [false, true] {
                let clean = fixture(10);
                let mut interrupted = if missing_body { fixture(9) } else { clean.clone() };
                if !missing_body {
                    let bounds = FREECELL_CONTROLS.spec(Control::Ok).bounds;
                    for y in bounds.y..bounds.bottom() {
                        for x in bounds.x..bounds.right() {
                            let offset = y as usize * interrupted.stride + x as usize * 4;
                            if interrupted.pixels[offset..offset + 3] == [0, 0, 0] {
                                interrupted.pixels[offset..offset + 3].fill(1);
                            }
                        }
                    }
                }
                assert_eq!(freecell_terminal::expected_control_point(&interrupted, TerminalStage::LevelUp).unwrap(),
                    PixelPoint::new(0, 0));
                let expected = freecell_terminal::expected_control_point(&clean, TerminalStage::LevelUp).unwrap();
                let cancel = AtomicBool::new(false);
                let mut frames = vec![clean.clone(), interrupted, clean.clone(), fixture(9)];
                if missing_body { frames.push(clean.clone()); }
                let mut io = fake(frames, &cancel);
                let settings = StepRunSettings::new(AnimationSettleDelays::default()
                    .with_freecell_millis(1_000, 2_300), 0);
                let (sender, _receiver) = std::sync::mpsc::channel();
                let events = WorkerEventSink {
                    events: sender, latest_frame: super::super::LatestFrameSlot::default(),
                    capture_context: std::sync::Mutex::new(None),
                };
                let mut latest = None;
                capture_latest(&mut io, &mut latest, &events, &cancel).unwrap();
                let result = if optional_pair {
                    await_post_score_control(&mut io, &mut latest, settings, &events, &cancel)
                        .map(|(_, point)| point)
                } else {
                    await_expected_control(&mut io, &mut latest, TerminalStage::LevelUp,
                        settings, &events, &cancel)
                };
                if missing_body {
                    assert!(result.unwrap_err().contains("latest failed frame retained"));
                } else {
                    assert_eq!(result.unwrap(), expected, "only the fresh current glyph permits readiness");
                }
                let expected_waits = if missing_body {
                    vec![Duration::from_millis(200), Duration::from_millis(2_300), Duration::from_millis(200)]
                } else {
                    vec![Duration::from_millis(200), Duration::from_millis(2_300)]
                };
                assert_eq!(io.waits, expected_waits);
                assert_eq!(io.captures, if missing_body { 4 } else { 3 });
                assert_eq!(io.frames.len(), 1, "lost geometry cannot grant a fifth confirmation");
                assert!(io.controls.is_empty() && io.sources.is_empty());
                assert_eq!(io.solver_clicks, 0);
                let expected_pixels = if missing_body { fixture(9).pixels } else { clean.pixels };
                assert_eq!(latest.unwrap().frame.pixels, expected_pixels);
            }
        }
    }

    /// A retained pointer below the caption cannot trigger the 200 ms confirmation interval.
    #[test]
    fn pointer_only_ok_runs_keep_the_normal_input_free_retry_delay() {
        let pointer = decode_png(include_bytes!("../../tests/fixtures/game-win-readiness/fading.png")).unwrap();
        assert_eq!(freecell_terminal::expected_control_point(&pointer, TerminalStage::LevelUp).unwrap(),
            PixelPoint::new(945, 792));
        for optional_pair in [false, true] {
            let cancel = AtomicBool::new(false);
            let mut io = fake(vec![pointer.clone(), pointer.clone(), pointer.clone(), fixture(10)], &cancel);
            let settings = StepRunSettings::new(AnimationSettleDelays::default()
                .with_freecell_millis(1_000, 2_300), 0);
            let (sender, _receiver) = std::sync::mpsc::channel();
            let events = WorkerEventSink {
                events: sender, latest_frame: super::super::LatestFrameSlot::default(),
                capture_context: std::sync::Mutex::new(None),
            };
            let mut latest = None;
            capture_latest(&mut io, &mut latest, &events, &cancel).unwrap();
            let result = if optional_pair {
                await_post_score_control(&mut io, &mut latest, settings, &events, &cancel).map(|_| ())
            } else {
                await_expected_control(&mut io, &mut latest, TerminalStage::LevelUp,
                    settings, &events, &cancel).map(|_| ())
            };
            assert!(result.unwrap_err().contains("latest failed frame retained"));
            assert_eq!(io.waits, [Duration::from_millis(2_300); 2]);
            assert_eq!(io.captures, TERMINAL_CONTROL_SEARCH_ATTEMPTS);
            assert_eq!(io.frames.len(), 1);
            assert!(io.controls.is_empty() && io.sources.is_empty());
            assert_eq!(io.solver_clicks, 0);
            assert_eq!(latest.unwrap().frame.pixels, pointer.pixels);
        }
    }

    /// The final OK observation cannot renew itself or select a fourth-frame New Game.
    #[test]
    fn final_ok_confirmation_misses_stop_without_a_fifth_capture_or_alternative_input() {
        for optional_pair in [false, true] {
            for fourth_kind in ["tinted", "pointer-only", "new-game", "moved-button"] {
                let clean = final_confirmation_fixture("clean");
                let mut fourth = match fourth_kind {
                    "tinted" => final_confirmation_fixture("tinted"),
                    "new-game" => fixture(15),
                    "moved-button" => fixture(10),
                    _ => clean.clone(),
                };
                if fourth_kind == "pointer-only" {
                    let bounds = FREECELL_CONTROLS.spec(Control::Ok).bounds;
                    for y in bounds.y..bounds.bottom() {
                        for x in bounds.x..bounds.right() {
                            let offset = y as usize * fourth.stride + x as usize * 4;
                            if fourth.pixels[offset..offset + 3] == [0, 0, 0] {
                                fourth.pixels[offset..offset + 3].fill(1);
                            }
                        }
                    }
                    for x in 945..950 {
                        let offset = 792 * fourth.stride + x * 4;
                        fourth.pixels[offset..offset + 3].fill(0);
                    }
                }
                let cancel = AtomicBool::new(false);
                let mut io = fake(vec![final_confirmation_fixture("entering"),
                    final_confirmation_fixture("tinted"), clean.clone(), fourth.clone(), clean], &cancel);
                let settings = StepRunSettings::new(AnimationSettleDelays::default(), 0);
                let (sender, receiver) = std::sync::mpsc::channel();
                let slot = super::super::LatestFrameSlot::default();
                let events = WorkerEventSink {
                    events: sender, latest_frame: slot.clone(),
                    capture_context: std::sync::Mutex::new(None),
                };
                let mut latest = None;
                capture_latest(&mut io, &mut latest, &events, &cancel).unwrap();
                let error = if optional_pair {
                    await_post_score_control(&mut io, &mut latest, settings, &events, &cancel)
                        .unwrap_err()
                } else {
                    await_expected_control(&mut io, &mut latest, TerminalStage::LevelUp,
                        settings, &events, &cancel).unwrap_err()
                };
                assert!(error.contains("after 4 searches and 3 fresh recaptures"), "{fourth_kind}: {error}");
                assert_eq!(io.captures, 4);
                assert_eq!(io.frames.len(), 1, "{fourth_kind}: a fifth capture is forbidden");
                assert_eq!(io.waits, [settings.animation_delays().freecell_reobserve,
                    settings.animation_delays().freecell_reobserve, Duration::from_millis(200)]);
                assert!(io.controls.is_empty() && io.sources.is_empty());
                assert_eq!(io.solver_clicks, 0);
                assert_eq!(latest.unwrap().frame.pixels, fourth.pixels);
                assert_eq!(slot.take().unwrap().frame.pixels, fourth.pixels);
                let logs: Vec<_> = receiver.try_iter().filter_map(|event| match event.event {
                    WorkerEvent::Log(message) => Some(message), _ => None,
                }).collect();
                assert!(logs.iter().any(|message| message.starts_with("Free Cell OK search summary: searches=4; recaptures=3;")));
                if optional_pair {
                    assert!(logs.iter().any(|message| message.starts_with("Free Cell New Game search summary: searches=3; recaptures=2;")));
                    assert!(!logs.iter().any(|message| message.starts_with("Free Cell New Game search 4:")));
                }
            }
        }
    }

    /// STOP during the reserved 200 ms wait retains observation three and sends no input.
    #[test]
    fn stop_during_final_ok_confirmation_prevents_the_fourth_capture() {
        for optional_pair in [false, true] {
            let clean = final_confirmation_fixture("clean");
            let cancel = AtomicBool::new(false);
            let mut io = fake(vec![final_confirmation_fixture("entering"),
                final_confirmation_fixture("tinted"), clean.clone(), clean.clone()], &cancel);
            io.fault = Fault::StopOnThirdWait;
            let settings = StepRunSettings::new(AnimationSettleDelays::default(), 0);
            let (sender, _receiver) = std::sync::mpsc::channel();
            let slot = super::super::LatestFrameSlot::default();
            let events = WorkerEventSink {
                events: sender, latest_frame: slot.clone(),
                capture_context: std::sync::Mutex::new(None),
            };
            let mut latest = None;
            capture_latest(&mut io, &mut latest, &events, &cancel).unwrap();
            let error = if optional_pair {
                await_post_score_control(&mut io, &mut latest, settings, &events, &cancel)
                    .unwrap_err()
            } else {
                await_expected_control(&mut io, &mut latest, TerminalStage::LevelUp,
                    settings, &events, &cancel).unwrap_err()
            };
            assert!(error.contains("STOP"));
            assert_eq!(io.captures, 3);
            assert_eq!(io.frames.len(), 1, "STOP prevents the reserved fourth capture");
            assert_eq!(io.waits, [settings.animation_delays().freecell_reobserve,
                settings.animation_delays().freecell_reobserve, Duration::from_millis(200)]);
            assert!(io.controls.is_empty() && io.sources.is_empty());
            assert_eq!(io.solver_clicks, 0);
            assert_eq!(latest.unwrap().frame.pixels, clean.pixels);
            assert_eq!(slot.take().unwrap().frame.pixels, clean.pixels);
        }
    }

    /// A fourth-frame menu match cannot extend New Game or Play's three-observation budget.
    #[test]
    fn new_game_and_play_never_receive_a_final_ok_confirmation_capture() {
        for (stage, menu) in [(TerminalStage::NewGame, fixture(15)),
            (TerminalStage::Play, fixture(12))]
        {
            let cancel = AtomicBool::new(false);
            let mut io = fake(vec![fixture(9), fixture(9), fixture(9), menu], &cancel);
            let settings = StepRunSettings::new(AnimationSettleDelays::default(), 0);
            let (sender, _receiver) = std::sync::mpsc::channel();
            let events = WorkerEventSink {
                events: sender, latest_frame: super::super::LatestFrameSlot::default(),
                capture_context: std::sync::Mutex::new(None),
            };
            let mut latest = None;
            capture_latest(&mut io, &mut latest, &events, &cancel).unwrap();
            assert!(await_expected_control(&mut io, &mut latest, stage,
                settings, &events, &cancel).unwrap_err().contains("after 3 searches and 2 fresh recaptures"));
            assert_eq!(io.captures, 3);
            assert_eq!(io.frames.len(), 1, "the fourth menu frame remains unread");
            assert_eq!(io.waits, [settings.animation_delays().freecell_reobserve; 2]);
            assert!(io.controls.is_empty() && io.sources.is_empty());
            assert_eq!(io.solver_clicks, 0);
        }
    }

    /// Acknowledging the fourth-frame OK does not renew OK or the next menu's budget.
    #[test]
    fn acknowledged_final_ok_is_not_replayed_when_the_modal_remains_visible() {
        let clean = final_confirmation_fixture("clean");
        let cancel = AtomicBool::new(false);
        // Repeated native pixels model controlled new captures; a live fourth
        // response at 200 ms was not supplied with the recorded failing episode.
        let mut frames = vec![fixture(9), final_confirmation_fixture("entering"),
            final_confirmation_fixture("tinted")];
        frames.extend(vec![clean.clone(); 5]);
        frames.push(fixture(15));
        let mut io = fake(frames, &cancel);
        let mut latest = None;
        assert!(run(&mut io, 0, 3, &mut latest).unwrap_err().contains("New Game"));
        assert_eq!(io.controls, [TerminalStage::Score, TerminalStage::LevelUp]);
        assert_eq!(io.control_points, [PixelPoint::new(960, 550), PixelPoint::new(942, 770)]);
        assert_eq!(io.captures, 8);
        assert_eq!(io.frames.len(), 1, "no fourth New Game capture or extra OK replay is permitted");
        assert!(io.sources.is_empty());
        assert_eq!(io.solver_clicks, 0);
        let normal = AnimationSettleDelays::default().freecell_reobserve;
        assert_eq!(io.waits, [LEVEL_UP_APPEAR_DELAY, normal, normal,
            Duration::from_millis(200), POST_GAME_STAGE_DELAY, normal, normal]);
        assert_eq!(latest.unwrap().frame.pixels, clean.pixels);
    }


    /// Stable third-frame washout permits one observation; only its fresh fourth glyph can click.
    #[test]
    fn interrupted_ok_confirmation_uses_one_fresh_fourth_point() {
        for optional_pair in [false, true] {
            let clean = final_confirmation_fixture("clean");
            let previous_point =
                freecell_terminal::expected_control_point(&clean, TerminalStage::LevelUp).unwrap();
            let mut fourth = clean.clone();
            let expected = PixelPoint::new(930, previous_point.y);
            for x in expected.x..expected.x + 5 {
                let offset = expected.y as usize * fourth.stride + x as usize * 4;
                fourth.pixels[offset..offset + 3].fill(0);
            }
            assert_ne!(expected, previous_point);
            let cancel = AtomicBool::new(false);
            let mut io = fake(
                vec![
                    final_confirmation_fixture("entering"),
                    clean,
                    interrupted_ok_fixture(),
                    fourth.clone(),
                    fixture(9),
                ],
                &cancel,
            );
            let settings = StepRunSettings::new(
                AnimationSettleDelays::default().with_freecell_millis(1_000, 2_300),
                0,
            );
            let (sender, receiver) = std::sync::mpsc::channel();
            let events = WorkerEventSink {
                events: sender,
                latest_frame: super::super::LatestFrameSlot::default(),
                capture_context: std::sync::Mutex::new(None),
            };
            let mut latest = None;
            capture_latest(&mut io, &mut latest, &events, &cancel).unwrap();
            let point = if optional_pair {
                let (stage, point) =
                    await_post_score_control(&mut io, &mut latest, settings, &events, &cancel)
                        .unwrap();
                assert_eq!(stage, TerminalStage::LevelUp);
                point
            } else {
                await_expected_control(
                    &mut io,
                    &mut latest,
                    TerminalStage::LevelUp,
                    settings,
                    &events,
                    &cancel,
                )
                .unwrap()
            };
            assert_eq!(point, expected);
            assert_eq!(io.captures, 4);
            assert_eq!(io.frames.len(), 1);
            assert_eq!(
                io.waits,
                [
                    Duration::from_millis(2_300),
                    Duration::from_millis(200),
                    Duration::from_millis(200)
                ]
            );
            assert!(io.controls.is_empty() && io.sources.is_empty());
            guard_input(&mut io, &cancel).unwrap();
            io.terminal(TerminalStage::LevelUp, point).unwrap();
            assert_eq!(io.control_points, [expected]);
            assert_eq!(io.controls, [TerminalStage::LevelUp]);
            assert_eq!(io.solver_clicks, 0);
            assert_eq!(latest.unwrap().frame.pixels, fourth.pixels);
            let logs: Vec<_> = receiver
                .try_iter()
                .filter_map(|event| match event.event {
                    WorkerEvent::Log(message) => Some(message),
                    _ => None,
                })
                .collect();
            assert!(
                logs.iter()
                    .any(|message| message.contains("interrupted_confirmation=true"))
            );
            assert!(
                !logs
                    .iter()
                    .any(|message| message.starts_with("Free Cell New Game search 4:"))
            );
        }
    }


    /// A failed fourth witness retains diagnostics and cannot select another stage or a fifth frame.
    #[test]
    fn interrupted_ok_confirmation_failures_keep_the_single_reserve_bound() {
        for optional_pair in [false, true] {
            for kind in [
                "washout",
                "no-glyph",
                "new-game",
                "play",
                "gameplay",
                "moved-body",
                "malformed",
                "capture-error",
                "stop-wait",
                "stop-capture",
            ] {
                let clean = final_confirmation_fixture("clean");
                let interrupted = interrupted_ok_fixture();
                let mut fourth = match kind {
                    "new-game" => fixture(15),
                    "play" => fixture(12),
                    "gameplay" => fixture(2),
                    "moved-body" => fixture(10),
                    "no-glyph" => final_confirmation_fixture("tinted"),
                    "malformed" => clean.clone(),
                    _ => interrupted.clone(),
                };
                if kind == "malformed" {
                    fourth.pixels.truncate(4);
                }
                let cancel = AtomicBool::new(false);
                let mut io = fake(
                    vec![
                        final_confirmation_fixture("entering"),
                        clean.clone(),
                        interrupted.clone(),
                        fourth.clone(),
                        clean,
                    ],
                    &cancel,
                );
                io.fault = match kind {
                    "capture-error" => Fault::FourthCapture,
                    "stop-wait" => Fault::StopOnThirdWait,
                    "stop-capture" => Fault::StopAfterFourthCapture,
                    _ => Fault::None,
                };
                let settings = StepRunSettings::new(AnimationSettleDelays::default(), 0);
                let (sender, receiver) = std::sync::mpsc::channel();
                let slot = super::super::LatestFrameSlot::default();
                let events = WorkerEventSink {
                    events: sender,
                    latest_frame: slot.clone(),
                    capture_context: std::sync::Mutex::new(None),
                };
                let mut latest = None;
                capture_latest(&mut io, &mut latest, &events, &cancel).unwrap();
                let result = if optional_pair {
                    await_post_score_control(&mut io, &mut latest, settings, &events, &cancel)
                        .map(|_| ())
                } else {
                    await_expected_control(
                        &mut io,
                        &mut latest,
                        TerminalStage::LevelUp,
                        settings,
                        &events,
                        &cancel,
                    )
                    .map(|_| ())
                };
                assert!(result.is_err(), "{kind}");
                assert_eq!(
                    io.captures,
                    if kind == "stop-wait" { 3 } else { 4 },
                    "{kind}"
                );
                assert_eq!(
                    io.frames.len(),
                    if matches!(kind, "stop-wait" | "capture-error") {
                        2
                    } else {
                        1
                    },
                    "{kind}"
                );
                assert_eq!(io.waits.last(), Some(&Duration::from_millis(200)), "{kind}");
                assert!(io.controls.is_empty() && io.sources.is_empty(), "{kind}");
                assert_eq!(io.solver_clicks, 0);
                let retained = if matches!(kind, "stop-wait" | "capture-error") {
                    interrupted
                } else {
                    fourth
                };
                assert_eq!(latest.unwrap().frame.pixels, retained.pixels, "{kind}");
                assert_eq!(slot.take().unwrap().frame.pixels, retained.pixels, "{kind}");
                let logs: Vec<_> = receiver
                    .try_iter()
                    .filter_map(|event| match event.event {
                        WorkerEvent::Log(message) => Some(message),
                        _ => None,
                    })
                    .collect();
                assert!(
                    !logs
                        .iter()
                        .any(|message| message.starts_with("Free Cell New Game search 4:"))
                );
            }
        }
    }


    /// A current washout with no valid immediate predecessor cannot borrow an older glyph.
    #[test]
    fn interrupted_ok_requires_the_immediate_valid_predecessor() {
        for optional_pair in [false, true] {
            for previous in [interrupted_ok_fixture(), fixture(2), fixture(9)] {
                let interrupted = interrupted_ok_fixture();
                let cancel = AtomicBool::new(false);
                let mut io = fake(
                    vec![
                        final_confirmation_fixture("clean"),
                        previous,
                        interrupted.clone(),
                        final_confirmation_fixture("clean"),
                    ],
                    &cancel,
                );
                let settings = StepRunSettings::new(AnimationSettleDelays::default(), 0);
                let (sender, _receiver) = std::sync::mpsc::channel();
                let events = WorkerEventSink {
                    events: sender,
                    latest_frame: super::super::LatestFrameSlot::default(),
                    capture_context: std::sync::Mutex::new(None),
                };
                let mut latest = None;
                capture_latest(&mut io, &mut latest, &events, &cancel).unwrap();
                let result = if optional_pair {
                    await_post_score_control(&mut io, &mut latest, settings, &events, &cancel)
                        .map(|_| ())
                } else {
                    await_expected_control(
                        &mut io,
                        &mut latest,
                        TerminalStage::LevelUp,
                        settings,
                        &events,
                        &cancel,
                    )
                    .map(|_| ())
                };
                assert!(result.is_err());
                assert_eq!(io.captures, 3);
                assert_eq!(
                    io.frames.len(),
                    1,
                    "there is no eligible fourth observation"
                );
                assert!(io.controls.is_empty() && io.sources.is_empty());
                assert_eq!(io.solver_clicks, 0);
                assert_eq!(latest.unwrap().frame.pixels, interrupted.pixels);
            }
        }
    }


    /// The guarded restart sends Score once and one fresh OK, including on uncertain OK delivery.
    #[test]
    fn interrupted_ok_restart_never_replays_score_or_uncertain_ok() {
        for uncertain in [false, true] {
            let clean = final_confirmation_fixture("clean");
            let cancel = AtomicBool::new(false);
            let mut io = fake(
                vec![
                    fixture(9),
                    final_confirmation_fixture("entering"),
                    clean.clone(),
                    interrupted_ok_fixture(),
                    clean.clone(),
                    fixture(15),
                    fixture(12),
                    fixture(2),
                ],
                &cancel,
            );
            if uncertain {
                io.fault = Fault::LevelUpTerminal;
            }
            let settings = StepRunSettings::new(AnimationSettleDelays::default(), 0);
            let (sender, _receiver) = std::sync::mpsc::channel();
            let events = WorkerEventSink {
                events: sender,
                latest_frame: super::super::LatestFrameSlot::default(),
                capture_context: std::sync::Mutex::new(None),
            };
            let mut latest = None;
            capture_latest(&mut io, &mut latest, &events, &cancel).unwrap();
            let mut input_attempted = false;
            let result = restart_game(
                &mut io,
                &mut latest,
                TerminalStage::Score,
                settings,
                &events,
                &cancel,
                &mut input_attempted,
            );
            assert!(input_attempted);
            if uncertain {
                assert_eq!(result.unwrap_err(), "injected uncertain terminal");
                assert_eq!(io.controls, [TerminalStage::Score, TerminalStage::LevelUp]);
                assert_eq!(io.captures, 5);
                assert_eq!(io.frames.len(), 3);
                assert_eq!(latest.unwrap().frame.pixels, clean.pixels);
            } else {
                result.unwrap();
                assert_eq!(
                    io.controls,
                    [
                        TerminalStage::Score,
                        TerminalStage::LevelUp,
                        TerminalStage::NewGame,
                        TerminalStage::Play
                    ]
                );
                assert_eq!(io.captures, 8);
                assert!(io.frames.is_empty());
            }
            assert_eq!(io.waits[0], LEVEL_UP_APPEAR_DELAY);
            assert_eq!(io.waits[2..4], [Duration::from_millis(200); 2]);
            assert!(io.sources.is_empty());
            assert_eq!(io.solver_clicks, 0);
        }
    }
}
