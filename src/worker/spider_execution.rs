//! Solver-led Spider execution and its ordered one-board restart sequence.
//!
//! One fresh source HALO authorises one click or DRAW key press, followed by editable settling and
//! a new frame. An acknowledged, released source click parks the pointer on felt before settling.
//! DRAW has a separate editable deal interval. Card identity, changed pixels and previous effects are not input
//! conditions. Automatic transfers receive bounded input-free observations.
//! A missing HALO does not establish a win: positive terminal caption/control
//! evidence does. Only continuous runs advance the expected terminal controls.
//! After score skip, Level Up is optional: inspect local OK and New Game
//! readiness on each fresh frame and accept exactly one ready control. Both
//! candidates share three searches and at most two input-free recaptures; a third
//! miss retains that frame.
//! An inactive Solver board receives an editable start delay and fresh capture
//! before activation once per unresolved context. Active no-HALO gameplay receives
//! bounded input-free observations, with no Solver refresh or allowance reset.

use std::{path::Path, sync::atomic::{AtomicBool, Ordering}, time::{Duration, Instant}};

use super::{
    FrameObservation, WorkerEvent, WorkerEventSink, WorkerState, capture_screen,
    connect_and_probe, control_search::ControlLookup, execute_planned_input, format_action_status,
    format_prediction_target, format_step_plan, milliseconds, send_log,
    send_state, send_status, validate_probe, wait_or_stop,
};
use crate::{
    capture::CapturedFrame,
    spider,
    spider_terminal::{self, TerminalStage},
    game::{ActionTarget, GameMode, InputOperation},
    geometry::PixelPoint,
    parameters::{
        SPIDER_MAX_MULTI_STEP_ACTIONS, LEVEL_UP_APPEAR_DELAY, NOMINAL_FRAME_HEIGHT,
        NOMINAL_FRAME_WIDTH, POST_GAME_MOUSE_HOLD, POST_GAME_STAGE_DELAY,
        SOLVER_MOUSE_HOLD, TERMINAL_CONTROL_SEARCH_ATTEMPTS, StepRunSettings,
    },
    qmp::QmpClient,
    stepper::{StepPlan, plan_step},
    tracker::{PredictedAction, TableauScanState},
    terminal_controls::{Control, profiles::SPIDER_CONTROLS},
};


/// Reject invalid finite budgets before connecting or changing cancellation.
pub(super) fn validate_operation_limit(limit: usize) -> Result<(), String> {


    if limit > SPIDER_MAX_MULTI_STEP_ACTIONS {
        return Err(format!("Spider requires 0 for continuous play or 1..={SPIDER_MAX_MULTI_STEP_ACTIONS} actions; requested {limit}"));
    }
    Ok(())
}


/// I/O injection leaves all gameplay and terminal authority in production classifiers.
trait SpiderIo {


    /// Acquire one new decoded frame; no classifier result is fabricated here.
    fn capture(&mut self) -> Result<CapturedFrame, String>;


    /// Probe the running VM and current absolute tablet before each input.
    fn probe(&mut self) -> Result<(), String>;


    /// Acknowledge one source click or DRAW key through release without replay on uncertainty.
    fn input(&mut self, plan: StepPlan) -> Result<(), String>;


    /// Move the released source-click pointer to validated felt without clicking.
    fn park_pointer(&mut self) -> Result<(), String>;


    /// Activate inactive Solver once from fresh positively recognised gameplay.
    fn solver(&mut self) -> Result<(), String>;


    /// Click exactly one freshly ready expected terminal control.
    fn terminal(&mut self, stage: TerminalStage, point: PixelPoint) -> Result<(), String>;


    /// Perform an interruptible settle or observation delay.
    fn wait(&mut self, duration: Duration) -> Result<(), String>;
}


/// QMP adapter reusing capture, input, logging and cooperative cancellation.
struct QmpSpiderIo<'a> {
    /// Existing connection local to this execution run.
    qmp: &'a mut QmpClient,
    /// Shared capture artefacts are reserved beside this socket.
    socket_path: &'a Path,
    /// Cooperative STOP and mode/socket invalidation flag.
    cancel_requested: &'a AtomicBool,
    /// Bounded UI output and complete session log.
    event_tx: &'a WorkerEventSink,
}


impl SpiderIo for QmpSpiderIo<'_> {


    fn capture(&mut self) -> Result<CapturedFrame, String> {
        require_running(self.cancel_requested)?;
        let started = Instant::now();
        let (frame, timing) = capture_screen(self.qmp, self.socket_path, self.event_tx)?;
        send_log(self.event_tx, format!(
            "PROFILE Spider capture: acquisition={:.1} ms; request={:.1} ms, receive/read={:.1} ms, convert/decode={:.1} ms, png-encode={:.1} ms.",
            milliseconds(started.elapsed()), milliseconds(timing.screendump), milliseconds(timing.file_read), milliseconds(timing.decode), milliseconds(timing.png_encode),
        ));
        Ok(frame)
    }


    fn probe(&mut self) -> Result<(), String> {
        require_running(self.cancel_requested)?;
        let probe = self.qmp.probe().map_err(|error| format!("Spider pre-input probe failed: {error}"))?;
        validate_probe(&probe)?;


        if probe.current_absolute_pointer_name() != Some("QEMU HID Tablet") {
            return Err("Spider input requires the current absolute QEMU HID Tablet".to_owned());
        }
        require_running(self.cancel_requested)
    }


    fn input(&mut self, plan: StepPlan) -> Result<(), String> {
        execute_planned_input(self.qmp, plan, self.cancel_requested)
    }


    fn park_pointer(&mut self) -> Result<(), String> {
        require_running(self.cancel_requested)?;
        self.qmp.move_pointer(
            spider::POINTER_PARK, NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT,
        ).map_err(|error| format!("Spider pointer parking delivery is uncertain: {error}; no retry permitted"))
    }


    fn solver(&mut self) -> Result<(), String> {
        require_running(self.cancel_requested)?;
        self.qmp.click_with_hold(
            spider::SOLVER_CLICK, NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT, SOLVER_MOUSE_HOLD,
        ).map_err(|error| format!("Spider Solver delivery is uncertain: {error}; no retry permitted"))
    }


    fn terminal(&mut self, stage: TerminalStage, point: PixelPoint) -> Result<(), String> {
        require_running(self.cancel_requested)?;
        if point == PixelPoint::new(0, 0) {
            return Err("Spider terminal control has no detected click point; no input sent".to_owned());
        }
        self.qmp.click_with_hold(
            point, NOMINAL_FRAME_WIDTH,
            NOMINAL_FRAME_HEIGHT, POST_GAME_MOUSE_HOLD,
        ).map_err(|error| format!("Spider {stage} delivery is uncertain: {error}; no retry permitted"))
    }


    fn wait(&mut self, duration: Duration) -> Result<(), String> {
        wait_or_stop(duration, self.cancel_requested, "STOP during Spider settling; no input replayed").map(|_| ())
    }
}


/// Finite-budget completion differs from independent one-board game completion.
#[derive(Debug, PartialEq, Eq)]
enum RunOutcome {
    /// Acknowledged source actions consumed the finite budget.
    Completed(usize),
    /// Positive terminal entry ended a finite run without restart inputs.
    GameWon(usize),
}


/// Run fresh-evidence Spider play using shared worker facilities.
pub(super) fn run_spider_steps(
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


        if scan_state.mode() != GameMode::Spider {
            return Err("Spider run received another mode's scan state".to_owned());
        }
        require_preview_mode(approved_prediction)?;
        send_state(event_tx, WorkerState::Connecting);
        let (mut qmp, probe) = connect_and_probe(socket_path)?;
        validate_probe(&probe)?;
        send_log(event_tx, probe.summary());
        let mut io = QmpSpiderIo { qmp: &mut qmp, socket_path, cancel_requested, event_tx };
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
            send_log(event_tx, format!("Spider finite run complete: {actions} acknowledged fresh-HALO actions; card effects were not measured. Latest result retained."));
        }
        Ok(RunOutcome::GameWon(actions)) => {


            if let Some(observation) = latest {
                event_tx.publish_diagnostic_frame(observation.frame);
            }
            let _ = event_tx.send(WorkerEvent::SpiderGameCompleted);
            send_state(event_tx, WorkerState::Ready);
            send_status(event_tx, "Spider game won — no automatic restart".to_owned());
            send_log(event_tx, format!("Spider one-board GAME WIN recognised after {actions} source actions; finite run sent no terminal or restart input."));
        }
        Err(error) => {


            if let Some(observation) = latest {
                event_tx.publish_diagnostic_frame(observation.frame);
            }
            send_state(event_tx, WorkerState::Uncertain);
            send_status(event_tx, "Spider stopped".to_owned());
            send_log(event_tx, format!("Spider stopped: {error}; guest input attempted={input_attempted}. No uncertain source, pointer movement, Solver or terminal input was replayed; latest available frame retained."));
        }
    }
}


/// Drive one fresh source action at a time; the preview never freezes coordinates.
fn drive_run(
    io: &mut impl SpiderIo,
    settings: StepRunSettings,
    event_tx: &WorkerEventSink,
    cancel_requested: &AtomicBool,
    latest: &mut Option<FrameObservation>,
    input_attempted: &mut bool,
) -> Result<RunOutcome, String> {
    validate_operation_limit(settings.operation_limit())?;
    let delays = settings.animation_delays();
    send_log(event_tx, format!(
        "Spider fresh-HALO execution: game-start delay={} ms, card/automatic-pack settle={} ms, DRAW deal settle={} ms, no-HALO observation={} ms, delayed allowance={}. Initial inactive Solver receives one start delay and fresh capture before one activation per unresolved context. After Play, the same game-start delay runs once before observing the new board. Active no-HALO gameplay receives bounded input-free observations with no Solver refresh; activation does not reset the delayed allowance. One source click or DRAW key press consumes one action; Solver and terminal clicks do not. Each acknowledged released source click parks the pointer on validated felt before settling: one separate QMP movement command with two absolute events, no click and no extra action. DRAW uses qcode D, has its own deal wait and does not park. No card identity, pixel difference, COLLAPSED SUITS input, Recycle or Solve is used. Continuous 0 may restart one-board games through Score, optional Level Up OK, New Game and Play. After score skip, exactly one ready local OK or New Game control selects the next stage. When neither control is ready, at most two fresh input-free captures allow three total searches; a third miss stops with the latest frame retained; when both are ready, execution stops without input.",
        delays.spider_game_start.as_millis(), delays.spider_settle.as_millis(), delays.spider_draw_settle.as_millis(), delays.spider_reobserve.as_millis(), settings.spider_observation_limit(),
    ));
    capture_latest(io, latest, event_tx, cancel_requested)?;
    let mut actions = 0usize;
    let mut pointer_parks = 0usize;


    loop {
        require_running(cancel_requested)?;


        if settings.bounded_operation_limit().is_some_and(|limit| actions >= limit) {
            return Ok(RunOutcome::Completed(actions));
        }


        if let Some(stage) = await_context(io, latest, settings, event_tx, cancel_requested, input_attempted, false)? {


            if !settings.is_unbounded() {
                let _game_win_images = super::begin_game_win_images(event_tx)?;
                return Ok(RunOutcome::GameWon(actions));
            }
            restart_game(io, latest, stage, settings, event_tx, cancel_requested, input_attempted)?;
            continue;
        }
        let prediction = current(latest)?.prediction;
        let plan = plan_step(prediction).map_err(|error| format!("Spider plan rejected: {error}"))?;
        let next_actions = actions.checked_add(1).ok_or_else(|| "Spider action counter exhausted".to_owned())?;
        send_log(event_tx, format_step_plan(plan)?);
        guard_input(io, cancel_requested)?;
        send_state(event_tx, WorkerState::Acting);
        send_status(event_tx, format_action_status(prediction));
        *input_attempted = true;
        io.input(plan).map_err(|error| format!("Spider source input outcome uncertain: {error}; no replay permitted"))?;
        // The released source consumes its slot even if parking or later observation fails.
        actions = next_actions;
        send_log(event_tx, format!("Spider source action {actions} acknowledged through release: input_commands={}, input_events={}; source action budget consumed; pointer movement counted separately.", plan.input().qmp_command_count(), plan.input().qmp_event_count()));


        if matches!(plan.input().operation(), InputOperation::Click(_)) {
            guard_input(io, cancel_requested)?;
            io.park_pointer().map_err(|error| format!("Spider pointer parking outcome uncertain: {error}; no source or park replay permitted"))?;
            pointer_parks += 1;
            send_log(event_tx, format!("Spider pointer park {pointer_parks} acknowledged after released source action {actions}: {:?}; separate input_commands=1, input_events=2 (absolute x/y), no action-budget slot consumed.", spider::POINTER_PARK));
        }
        io.wait(plan.input().animation_settle_delay(delays))?;
        capture_latest(io, latest, event_tx, cancel_requested)?;
        let after = current(latest)?.prediction;
        let _ = event_tx.send(WorkerEvent::ActionCompleted {
            operation_index: actions, operation_limit: settings.operation_limit(),
            before: prediction, after, input_commands: plan.input().qmp_command_count(),
            input_events: plan.input().qmp_event_count(), changed_pixels: 0, continued_from_halo: true,
        });
        send_log(event_tx, format!(
            "Spider action {actions} acknowledged once: {} -> {}; card effects and pixel differences not measured. Next input requires a new supported source HALO, including at the same position.",
            format_prediction_target(prediction), format_prediction_target(after),
        ));
    }
}


/// Observe automatic transfers, terminal entry or one supported source without speculation.
fn await_context(
    io: &mut impl SpiderIo,
    latest: &mut Option<FrameObservation>,
    settings: StepRunSettings,
    event_tx: &WorkerEventSink,
    cancel_requested: &AtomicBool,
    input_attempted: &mut bool,
    game_start_settled: bool,
) -> Result<Option<TerminalStage>, String> {
    let mut activation_reserved = false;
    let mut delayed = 0usize;
    let mut start_observed = game_start_settled;


    loop {
        require_running(cancel_requested)?;
        let observation = current(latest)?;
        let stage = spider_terminal::classify_win_entry(&observation.frame)
            .map_err(|error| format!("Spider win-entry analysis failed: {error}"))?;


        if stage.is_some() {
            send_log(event_tx, format!("Spider independent one-board GAME WIN entry={stage:?}; missing HALO alone did not establish completion."));
            return Ok(stage);
        }
        let mut observed_solver = "unavailable: unsupported scene";


        if observation.gameplay_scene {


            if let PredictedAction::Action(_) = observation.prediction {
                require_source(observation.prediction)?;
                return Ok(None);
            }


            if !matches!(observation.prediction, PredictedAction::NoHighlight) {
                return Err("Spider scene has an uncertain or unsupported source recommendation".to_owned());
            }
            let solver_active = spider::solver_active(&observation.frame)
                .map_err(|error| format!("Spider Solver-state analysis failed: {error}"))?;
            observed_solver = if solver_active { "active" } else { "inactive" };


            if !solver_active && !activation_reserved {


                if !start_observed {
                    start_observed = true;
                    send_status(event_tx, "Settling Spider board before Solver activation".to_owned());
                    send_log(event_tx, format!("Spider Solver observed inactive with no HALO: waiting game-start delay={} ms once, then fresh input-free capture before deciding activation; guest input sent=0.", settings.animation_delays().spider_game_start.as_millis()));
                    io.wait(settings.animation_delays().spider_game_start)?;
                    capture_latest(io, latest, event_tx, cancel_requested)?;
                    continue;
                }


                guard_input(io, cancel_requested)?;
                activation_reserved = true;
                *input_attempted = true;
                send_state(event_tx, WorkerState::Acting);
                send_status(event_tx, "Spider Solver activation once".to_owned());
                send_log(event_tx, format!("Spider Solver activation: fresh supported no-HALO board, observed Solver={observed_solver}; one click consumes this unresolved context's activation reserve; delayed observations already used={delayed}, allowance is not reset."));
                io.solver()?;
                io.wait(settings.animation_delays().spider_settle)?;
                capture_latest(io, latest, event_tx, cancel_requested)?;
                continue;
            }
        }


        if delayed >= settings.spider_observation_limit() {
            return Err(format!("Spider source/win observation allowance exhausted after {delayed} delayed captures; scene={}, observed Solver={observed_solver}, activation reserved={activation_reserved}; no further input authorised", observation.gameplay_scene));
        }
        delayed += 1;
        send_log(event_tx, format!("Spider no-HALO observation {delayed}/{}: scene={}, observed Solver={observed_solver}, activation reserved={activation_reserved}; waiting {} ms, then input-free recapture; active Solver is not refreshed.", settings.spider_observation_limit(), observation.gameplay_scene, settings.animation_delays().spider_reobserve.as_millis()));
        io.wait(settings.animation_delays().spider_reobserve)?;
        capture_latest(io, latest, event_tx, cancel_requested)?;
    }
}


/// Advance each ready local control once after independent win entry.
/// Score skip can lead to optional Level Up OK or directly to New Game.
fn restart_game(
    io: &mut impl SpiderIo,
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
            return Err("Spider expected terminal control returned no match; no input authorised".to_owned());
        }
        guard_input(io, cancel_requested)?;
        send_state(event_tx, WorkerState::Acting);
        send_status(event_tx, stage.to_string());
        send_log(event_tx, format!("Spider deterministic GAME WIN control: {stage}; one click at {:?}; unrelated panel colours, levels and artwork are not conditions.", point));
        *input_attempted = true;
        io.terminal(stage, point)?;
        let settle = match stage {
            TerminalStage::Score => LEVEL_UP_APPEAR_DELAY,
            TerminalStage::Play => settings.animation_delays().spider_game_start,
            TerminalStage::LevelUp | TerminalStage::NewGame => POST_GAME_STAGE_DELAY,
        };
        io.wait(settle)?;
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


    for delayed in 0..=settings.spider_observation_limit() {
        require_running(cancel_requested)?;


        if current(latest)?.gameplay_scene {


            if await_context(io, latest, settings, event_tx, cancel_requested, input_attempted, true)?.is_some() {
                return Err("Spider restart encountered another terminal entry instead of a fresh Solver board".to_owned());
            }
            let _ = event_tx.send(WorkerEvent::SpiderGameCompleted);
            send_log(event_tx, "Spider one-board restart complete: fresh gameplay and supported Solver source recognised; continuous play resumes.".to_owned());
            return Ok(());
        }


        if delayed == settings.spider_observation_limit() {
            break;
        }
        io.wait(settings.animation_delays().spider_reobserve)?;
        capture_latest(io, latest, event_tx, cancel_requested)?;
    }
    Err("Spider Play acknowledgement was not followed by a recognised fresh board within bounded input-free observations; Play was not retried".to_owned())
}


/// Resolve optional OK and New Game from the same fresh frame.
/// Both candidates share at most two recaptures; an ambiguous pair sends no input.
/// A visible OK waits for settled readiness within that same three-frame budget.
fn await_post_score_control(
    io: &mut impl SpiderIo,
    latest: &mut Option<FrameObservation>,
    settings: StepRunSettings,
    event_tx: &WorkerEventSink,
    cancel_requested: &AtomicBool,
) -> Result<(TerminalStage, PixelPoint), String> {
    let mut ok_lookup = ControlLookup::new("Spider", &SPIDER_CONTROLS, Control::Ok, event_tx);
    let mut new_game_lookup = ControlLookup::new("Spider", &SPIDER_CONTROLS, Control::NewGame, event_tx);
    for attempt in 1..=TERMINAL_CONTROL_SEARCH_ATTEMPTS {
        require_running(cancel_requested)?;
        let frame = &current(latest)?.frame;
        let ok_point = ok_lookup.search(frame)?;
        let new_game_point = new_game_lookup.search(frame)?;
        match (ok_point != PixelPoint::new(0, 0), new_game_point != PixelPoint::new(0, 0)) {
            (true, false) => {
                let ready = ok_lookup.confirm_ok(frame, ok_point)?;
                if ready != PixelPoint::new(0, 0) {
                    ok_lookup.set_outcome("matched");
                    new_game_lookup.set_outcome("optional absent");
                    send_log(event_tx, "Spider post-score branch: settled OK ready; optional Level Up selected.".to_owned());
                    return Ok((TerminalStage::LevelUp, ready));
                }
            }
            (false, true) => {
                ok_lookup.confirm_ok(frame, ok_point)?;
                if !ok_lookup.has_ok_panel() {
                    ok_lookup.set_outcome("optional absent");
                    new_game_lookup.set_outcome("matched");
                    send_log(event_tx, "Spider post-score branch: local New Game ready; no Level Up step required.".to_owned());
                    return Ok((TerminalStage::NewGame, new_game_point));
                }
            }
            (true, true) => {
                ok_lookup.set_outcome("ambiguous");
                new_game_lookup.set_outcome("ambiguous");
                return Err("Spider post-score controls are uncertain: both OK and New Game are ready; no branch input authorised".to_owned());
            }
            (false, false) => { ok_lookup.confirm_ok(frame, ok_point)?; }
        }
        if attempt == TERMINAL_CONTROL_SEARCH_ATTEMPTS {
            break;
        }
        send_log(event_tx, format!("Spider post-score OK/New Game absent or OK not settled after search {attempt}/{TERMINAL_CONTROL_SEARCH_ATTEMPTS}: input-free recapture {attempt}/{} and repeat search; score skip is not repeated.", TERMINAL_CONTROL_SEARCH_ATTEMPTS - 1));
        if let Err(error) = io.wait(settings.animation_delays().spider_reobserve) {
            ok_lookup.set_outcome("wait stopped");
            new_game_lookup.set_outcome("wait stopped");
            return Err(error);
        }
        ok_lookup.note_recapture();
        new_game_lookup.note_recapture();
        if let Err(error) = capture_latest(io, latest, event_tx, cancel_requested) {
            ok_lookup.set_outcome("capture failed or stopped");
            new_game_lookup.set_outcome("capture failed or stopped");
            return Err(error);
        }
    }
    ok_lookup.set_outcome("not found");
    new_game_lookup.set_outcome("not found");
    Err(format!("Spider expected settled OK or New Game did not become ready after {TERMINAL_CONTROL_SEARCH_ATTEMPTS} searches and {} fresh recaptures; latest failed frame retained for preview; score skip was not retried", TERMINAL_CONTROL_SEARCH_ATTEMPTS - 1))
}


/// Each expected menu control receives at most three searches and two fresh recaptures.
/// OK also requires fresh settled readiness before its single permitted click.
/// Score skip retains its existing caption recognition and observation allowance.
fn await_expected_control(
    io: &mut impl SpiderIo,
    latest: &mut Option<FrameObservation>,
    stage: TerminalStage,
    settings: StepRunSettings,
    event_tx: &WorkerEventSink,
    cancel_requested: &AtomicBool,
) -> Result<PixelPoint, String> {
    let mut lookup = spider_terminal::control(stage)
        .map(|control| ControlLookup::new("Spider", &SPIDER_CONTROLS, control, event_tx));
    let retries = if lookup.is_some() { TERMINAL_CONTROL_SEARCH_ATTEMPTS - 1 } else { settings.spider_observation_limit() };
    for delayed in 0..=retries {
        require_running(cancel_requested)?;
        let point = if let Some(lookup) = lookup.as_mut() {
            let frame = &current(latest)?.frame;
            let candidate = lookup.search(frame)?;
            lookup.confirm_ok(frame, candidate)?
        } else {
            spider_terminal::expected_control_point(&current(latest)?.frame, stage)
                .map_err(|error| format!("Spider expected {stage} readiness failed: {error}"))?
        };
        if point != PixelPoint::new(0, 0) {
            if let Some(lookup) = lookup.as_mut() { lookup.set_outcome("matched"); }
            return Ok(point);
        }
        if delayed == retries { break; }
        send_log(event_tx, format!("Spider terminal input-free observation {}/{}: waiting only for expected {stage}; prior click is not repeated.", delayed + 1, retries));
        if let Err(error) = io.wait(settings.animation_delays().spider_reobserve) {
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
    Err(format!("Spider expected {stage} did not become ready after {} searches and {retries} fresh recaptures; latest failed frame retained for preview; no terminal input retried", retries + 1))
}


/// Retain decoded pixels before any late STOP or classifier failure is returned.
fn capture_latest(
    io: &mut impl SpiderIo,
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
    let observation = latest.as_mut().ok_or_else(|| "Spider decoded frame was not retained".to_owned())?;
    observation.gameplay_scene = spider::is_gameplay_scene(&observation.frame)
        .map_err(|error| format!("Spider scene analysis failed: {error}"))?;
    observation.prediction = spider::analyse(&observation.frame)
        .map_err(|error| format!("Spider source analysis failed: {error}"))?.prediction;
    require_running(cancel_requested)
}


/// Require fresh Spider action metadata and the shared canonical input plan.
fn require_source(prediction: PredictedAction) -> Result<(), String> {


    match prediction {
        PredictedAction::Action(action) if matches!(action.target, ActionTarget::Spider(_)) => {
            plan_step(prediction).map(|_| ()).map_err(|error| format!("Spider source rejected: {error}"))
        }
        _ => Err("Fresh recommendation is not a canonical Spider source".to_owned()),
    }
}


/// Advisory previews may be empty, but another mode's action cannot cross this boundary.
fn require_preview_mode(prediction: PredictedAction) -> Result<(), String> {


    match prediction {
        PredictedAction::NoHighlight | PredictedAction::CalibrationOnly { mode: GameMode::Spider } => Ok(()),
        PredictedAction::Action(action) if matches!(action.target, ActionTarget::Spider(_)) => Ok(()),
        _ => Err("Spider run received an unsupported advisory preview".to_owned()),
    }
}


/// Access the latest immutable observation without unchecked frame assumptions.
fn current(latest: &Option<FrameObservation>) -> Result<&FrameObservation, String> {
    latest.as_ref().ok_or_else(|| "Spider run has no fresh observation".to_owned())
}


/// Probe and recheck STOP immediately before authorised clicks, keys or pointer movement.
fn guard_input(io: &mut impl SpiderIo, cancel_requested: &AtomicBool) -> Result<(), String> {
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
    //! Real native frames drive decisions; injected I/O tests only delivery and STOP.

    use std::collections::VecDeque;

    use super::*;
    use crate::{capture::{PixelFormat, decode_png},
        parameters::AnimationSettleDelays};


    /// Faults model uncertain delivery or cancellation, not classifier answers.
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Fault {
        /// Normal acknowledged delivery.
        None,
        /// Source input may already have reached the guest.
        Source,
        /// Pointer movement may already have reached the guest.
        Park,
        /// Solver input may already have reached the guest.
        Solver,
        /// Terminal input may already have reached the guest.
        Terminal,
        /// Cancellation at the final pre-input probe boundary.
        StopAtProbe,
        /// Cancellation after a source click acknowledges its release.
        StopAfterSource,
        /// Cancellation at the second probe, immediately before parking.
        StopAtParkProbe,
        /// Cancellation after fresh pixels have been acquired.
        StopAfterCapture,
        /// Cancellation during a requested delay.
        StopOnWait,
        /// Cancellation during the second terminal recapture wait.
        StopOnSecondWait,
    }


    /// Call ordering proves that movement precedes settling and recapture.
    #[derive(Debug, PartialEq, Eq)]
    enum IoCall {
        /// Acquire one fresh native frame.
        Capture,
        /// Check the current VM and absolute pointer before input.
        Probe,
        /// Deliver one canonical gameplay click or DRAW key.
        Source(InputOperation),
        /// Move the released pointer without a button or key event.
        Park,
        /// Activate an observed inactive Solver control once.
        Solver,
        /// Click one ready expected terminal control.
        Terminal(TerminalStage),
        /// Observe the selected settle interval without input.
        Wait(Duration),
    }


    /// Native decoded frames are consumed afresh; actions and waits are recorded.
    struct FakeIo<'a> {
        /// Successive fresh native observations.
        frames: VecDeque<CapturedFrame>,
        /// Attempted canonical source inputs, including uncertain delivery.
        sources: Vec<StepPlan>,
        /// Attempted movement-only pointer parks, separate from source inputs.
        pointer_parks: usize,
        /// Delivery, delay and capture order at the adapter boundary.
        calls: Vec<IoCall>,
        /// Attempted terminal controls in order.
        controls: Vec<TerminalStage>,
        /// Measured full-frame coordinates passed to the terminal adapter.
        control_points: Vec<PixelPoint>,
        /// Attempted Solver inputs, independent from the action budget.
        solver_clicks: usize,
        /// Acquisition count immediately before each Solver input.
        solver_capture_counts: Vec<usize>,
        /// Number of acquired observations.
        captures: usize,
        /// Snapshotted intervals actually requested by the driver.
        waits: Vec<Duration>,
        /// Transport/cancellation fault, without fabricated detector authority.
        fault: Fault,
        /// The production cooperative cancellation flag.
        cancel: &'a AtomicBool,
    }


    impl SpiderIo for FakeIo<'_> {


        fn capture(&mut self) -> Result<CapturedFrame, String> {
            self.calls.push(IoCall::Capture);
            self.captures += 1;
            let frame = self.frames.pop_front().ok_or_else(|| "fixture sequence exhausted".to_owned())?;


            if self.fault == Fault::StopAfterCapture {
                self.cancel.store(true, Ordering::Release);
            }
            Ok(frame)
        }


        fn probe(&mut self) -> Result<(), String> {
            self.calls.push(IoCall::Probe);


            if self.fault == Fault::StopAtProbe
                || (self.fault == Fault::StopAtParkProbe && !self.sources.is_empty())
            {
                self.cancel.store(true, Ordering::Release);
            }
            Ok(())
        }


        fn input(&mut self, plan: StepPlan) -> Result<(), String> {
            self.calls.push(IoCall::Source(plan.input().operation()));
            self.sources.push(plan);


            if self.fault == Fault::StopAfterSource {
                self.cancel.store(true, Ordering::Release);
            }
            if self.fault == Fault::Source { Err("injected uncertain source".to_owned()) } else { Ok(()) }
        }


        fn park_pointer(&mut self) -> Result<(), String> {
            self.calls.push(IoCall::Park);
            self.pointer_parks += 1;


            if self.fault == Fault::Park { Err("injected uncertain movement".to_owned()) } else { Ok(()) }
        }


        fn solver(&mut self) -> Result<(), String> {
            self.calls.push(IoCall::Solver);
            self.solver_clicks += 1;
            self.solver_capture_counts.push(self.captures);


            if self.fault == Fault::Solver { Err("injected uncertain Solver".to_owned()) } else { Ok(()) }
        }


        fn terminal(&mut self, stage: TerminalStage, point: PixelPoint) -> Result<(), String> {
            self.calls.push(IoCall::Terminal(stage));
            assert_ne!(point, PixelPoint::new(0, 0), "no-match sentinel must never reach input");
            self.controls.push(stage);
            self.control_points.push(point);


            if self.fault == Fault::Terminal { Err("injected uncertain terminal".to_owned()) } else { Ok(()) }
        }


        fn wait(&mut self, duration: Duration) -> Result<(), String> {
            self.calls.push(IoCall::Wait(duration));
            self.waits.push(duration);


            if self.fault == Fault::StopOnWait
                || (self.fault == Fault::StopOnSecondWait && self.waits.len() == 2)
            {
                self.cancel.store(true, Ordering::Release);
            }
            require_running(self.cancel)
        }
    }


    /// Production decoding keeps the original PNG evidence and RGBA contract.
    fn fixture(number: u8) -> CapturedFrame {
        let path = format!("{}/tests/fixtures/spider-SP{number:02}.png", env!("CARGO_MANIFEST_DIR"));
        decode_png(&std::fs::read(path).unwrap()).unwrap()
    }


    /// Optional Level Up was confirmed to use the existing local OK control.
    fn level_up() -> CapturedFrame {
        decode_png(include_bytes!("../../tests/fixtures/freecell-FC10.png")).unwrap()
    }


    /// Byte-identical native Spider level-505 screenshot supplied by Charlie.
    fn level_up_505() -> CapturedFrame {
        decode_png(include_bytes!("../../tests/fixtures/spider-level-up-505.png")).unwrap()
    }


    /// Decode an unchanged capture from the Spider failure recorded on 15 October.
    fn recorded_game_win(name: &str) -> CapturedFrame {
        let path = format!("{}/tests/fixtures/spider-game-win-20261015/{name}.png", env!("CARGO_MANIFEST_DIR"));
        decode_png(&std::fs::read(path).unwrap()).unwrap()
    }


    /// A native-sized flat frame supplies no independent win or scene authority.
    /// Exact black pixels alone can satisfy the context-dependent OK run rule.
    fn unknown() -> CapturedFrame {
        CapturedFrame { width: 1_920, height: 1_080, stride: 7_680,
            format: PixelFormat::Rgba8, pixels: vec![0; 7_680 * 1_080] }
    }


    /// Remove only the highlighted column from active SP02, retaining the scene
    /// and active Solver banner. No fabricated classification result is injected.
    fn active_without_halo() -> CapturedFrame {
        let mut frame = fixture(2);


        for y in 90..spider::TOOLBAR_TOP {


            for x in 280..445 {
                let offset = y as usize * frame.stride + x * 4;
                frame.pixels[offset..offset + 3].copy_from_slice(&[20, 100, 75]);
            }
        }
        assert!(spider::is_gameplay_scene(&frame).unwrap());
        assert!(spider::solver_active(&frame).unwrap());
        assert_eq!(spider::analyse(&frame).unwrap().prediction, PredictedAction::NoHighlight);
        frame
    }


    /// Exhaustion deliberately reveals unexpected captures or repeated inputs.
    fn fake(frames: Vec<CapturedFrame>, cancel: &AtomicBool) -> FakeIo<'_> {
        FakeIo { frames: frames.into(), sources: Vec::new(), pointer_parks: 0,
            calls: Vec::new(), controls: Vec::new(), control_points: Vec::new(),
            solver_clicks: 0, solver_capture_counts: Vec::new(), captures: 0,
            waits: Vec::new(), fault: Fault::None, cancel }
    }


    /// Use the production driver and immutable timing/budget snapshot.
    fn run(
        io: &mut FakeIo<'_>,
        limit: usize,
        latest: &mut Option<FrameObservation>,
    ) -> Result<RunOutcome, String> {
        let settings = StepRunSettings::new(AnimationSettleDelays::default(), limit)
            .with_spider_observation_limit(2);
        run_with_settings(io, settings, latest)
    }


    /// Isolate logs/UI events without replacing any production classifier.
    fn run_with_settings(
        io: &mut FakeIo<'_>,
        settings: StepRunSettings,
        latest: &mut Option<FrameObservation>,
    ) -> Result<RunOutcome, String> {
        run_with_events(io, settings, latest).0
    }


    /// Inspect emitted source counts and separate park logging when relevant.
    fn run_with_events(
        io: &mut FakeIo<'_>,
        settings: StepRunSettings,
        latest: &mut Option<FrameObservation>,
    ) -> (Result<RunOutcome, String>, Vec<WorkerEvent>) {
        let (events, receiver) = std::sync::mpsc::channel();
        let events = WorkerEventSink {
            events, latest_frame: super::super::LatestFrameSlot::default(),
            capture_context: std::sync::Mutex::new(None),
        };
        let mut attempted = false;
        let cancel = io.cancel;
        let result = drive_run(io, settings, &events, cancel, latest, &mut attempted);
        (result, receiver.try_iter().map(|notification| notification.event).collect())
    }


    /// One complete run is one source click even when several rounded borders
    /// overlap; identical settled frames do not trigger card-pixel effect proof.
    #[test]
    fn fresh_native_single_cards_runs_and_same_position_consume_one_action() {


        for number in [2, 3, 4, 5, 6, 7, 10, 11, 12, 13, 14] {
            let cancel = AtomicBool::new(false);
            let mut io = fake(vec![fixture(number), fixture(number)], &cancel);
            assert_eq!(run(&mut io, 1, &mut None), Ok(RunOutcome::Completed(1)), "SP{number:02}");
            assert_eq!(io.sources.len(), 1);
            assert!(matches!(io.sources[0].input().operation(), InputOperation::Click { .. }));
            assert_eq!(io.pointer_parks, 1);
            assert_eq!(io.solver_clicks, 0);
        }
        let cancel = AtomicBool::new(false);
        let mut io = fake(vec![fixture(3), fixture(3), fixture(3)], &cancel);
        assert_eq!(run(&mut io, 2, &mut None), Ok(RunOutcome::Completed(2)));
        assert_eq!(io.sources[0].before(), io.sources[1].before());
        assert_eq!(io.pointer_parks, 2);
    }


    /// Source command accounting excludes the separately logged movement; the
    /// park is guarded after acknowledged release and precedes settle/capture.
    #[test]
    fn acknowledged_source_click_parks_before_settle_and_fresh_capture() {
        let cancel = AtomicBool::new(false);
        let mut io = fake(vec![fixture(3), fixture(4)], &cancel);
        let settings = StepRunSettings::new(AnimationSettleDelays::default(), 1);
        let (result, events) = run_with_events(&mut io, settings, &mut None);
        assert_eq!(result, Ok(RunOutcome::Completed(1)));
        assert_eq!(io.calls, [IoCall::Capture, IoCall::Probe,
            IoCall::Source(io.sources[0].input().operation()), IoCall::Probe,
            IoCall::Park, IoCall::Wait(settings.animation_delays().spider_settle), IoCall::Capture]);
        let source_counts: Vec<_> = events.iter().filter_map(|event| match event {
            WorkerEvent::ActionCompleted { operation_index, input_commands, input_events, .. } =>
                Some((*operation_index, *input_commands, *input_events)),
            _ => None,
        }).collect();
        assert_eq!(source_counts, [(1, 3, 4)]);
        assert!(events.iter().any(|event| matches!(event, WorkerEvent::Log(message)
            if message.contains("pointer park 1 acknowledged")
                && message.contains("input_commands=1, input_events=2"))));
    }


    /// DRAW acknowledges only its two key commands, then the deal interval and
    /// recapture; it never probes for or delivers a pointer park.
    #[test]
    fn draw_never_parks_pointer_or_consumes_an_extra_action() {
        let cancel = AtomicBool::new(false);
        let mut io = fake(vec![fixture(8), fixture(9)], &cancel);
        let settings = StepRunSettings::new(AnimationSettleDelays::default(), 1);
        let (result, events) = run_with_events(&mut io, settings, &mut None);
        assert_eq!(result, Ok(RunOutcome::Completed(1)));
        assert_eq!(io.pointer_parks, 0);
        assert_eq!(io.calls, [IoCall::Capture, IoCall::Probe,
            IoCall::Source(InputOperation::PressDrawKey),
            IoCall::Wait(settings.animation_delays().spider_draw_settle), IoCall::Capture]);
        assert!(events.iter().any(|event| matches!(event, WorkerEvent::ActionCompleted {
            operation_index: 1, input_commands: 2, input_events: 2, ..
        })));
    }


    /// Uncertain movement and STOP after release or at its probe stop without
    /// repeating the acknowledged source, settling or recapturing.
    #[test]
    fn parking_failure_and_stop_after_source_do_not_replay_source() {


        for fault in [Fault::Park, Fault::StopAfterSource, Fault::StopAtParkProbe] {
            let cancel = AtomicBool::new(false);
            let mut io = fake(vec![fixture(3)], &cancel);
            io.fault = fault;
            let settings = StepRunSettings::new(AnimationSettleDelays::default(), 1);
            let (result, events) = run_with_events(&mut io, settings, &mut None);
            let error = result.unwrap_err();
            assert!(error.contains(if fault == Fault::Park { "uncertain" } else { "STOP" }));
            assert_eq!(io.sources.len(), 1);
            assert_eq!(io.pointer_parks, usize::from(fault == Fault::Park));
            assert_eq!(io.captures, 1);
            assert!(io.waits.is_empty() && io.controls.is_empty());
            assert_eq!(io.solver_clicks, 0);
            assert!(!events.iter().any(|event| matches!(event, WorkerEvent::ActionCompleted { .. })));
            assert!(events.iter().any(|event| matches!(event, WorkerEvent::Log(message)
                if message.contains("source action 1 acknowledged through release"))));
        }
    }


    /// SP08 sends D once and waits its separate editable deal interval before
    /// SP09; card inputs use the other snapshotted settle value.
    #[test]
    fn draw_key_deal_and_card_delays_are_separate_and_editable() {
        let cancel = AtomicBool::new(false);
        let mut io = fake(vec![fixture(8), fixture(9), fixture(10)], &cancel);
        let delays = AnimationSettleDelays::default().with_spider_millis(650, 2_250, 1_000);
        let settings = StepRunSettings::new(delays, 2).with_spider_observation_limit(2);
        assert_eq!(run_with_settings(&mut io, settings, &mut None), Ok(RunOutcome::Completed(2)));
        assert_eq!(io.sources.len(), 2);
        assert_eq!(io.sources[0].input().operation(), InputOperation::PressDrawKey);
        assert!(matches!(io.sources[1].input().operation(), InputOperation::Click { .. }));
        assert_eq!(io.waits, [Duration::from_millis(2_250), Duration::from_millis(650)]);
        assert_eq!(io.captures, 3);
        assert_eq!(io.solver_clicks, 0);
        assert_eq!(io.pointer_parks, 1);
    }


    /// A fresh deal receives its start delay and fresh observation before one
    /// activation; a source already appearing during the wait skips activation.
    #[test]
    fn fresh_deal_wait_precedes_solver_and_fresh_capture_can_skip_activation() {


        for activate in [true, false] {
            let cancel = AtomicBool::new(false);
            let mut frames = vec![fixture(1)];


            if activate { frames.push(fixture(1)); }
            frames.extend([fixture(2), fixture(3)]);
            let mut io = fake(frames, &cancel);
            assert_eq!(run(&mut io, 1, &mut None), Ok(RunOutcome::Completed(1)));
            assert_eq!(io.solver_clicks, usize::from(activate));
            assert_eq!(io.waits[0], AnimationSettleDelays::default().spider_game_start);


            if activate { assert_eq!(io.solver_capture_counts, [2]); }
            assert_eq!(io.sources.len(), 1);
        }
    }


    /// Active no-HALO boards receive only bounded captures. Each can recognise
    /// a source or independent win; continued absence sends no Solver input.
    #[test]
    fn active_no_halo_recaptures_without_solver_input_and_checks_completion() {
        let cancel = AtomicBool::new(false);
        let mut io = fake(vec![active_without_halo(), active_without_halo(), fixture(3), fixture(4)], &cancel);
        assert_eq!(run(&mut io, 1, &mut None), Ok(RunOutcome::Completed(1)));
        assert_eq!(io.solver_clicks, 0);
        assert_eq!(io.waits[0], AnimationSettleDelays::default().spider_reobserve);
        let mut io = fake(vec![active_without_halo(), active_without_halo(), fixture(15)], &cancel);
        assert_eq!(run(&mut io, 1, &mut None), Ok(RunOutcome::GameWon(0)));
        assert!(io.sources.is_empty() && io.controls.is_empty());
        assert_eq!(io.solver_clicks, 0);
        assert_eq!(io.pointer_parks, 0);
        let mut io = fake(vec![active_without_halo(); 3], &cancel);
        assert!(run(&mut io, 0, &mut None).unwrap_err().contains("allowance exhausted"));
        assert_eq!(io.solver_clicks, 0);
        assert_eq!(io.captures, 3);
        assert_eq!(io.waits, [AnimationSettleDelays::default().spider_reobserve; 2]);
        assert_eq!(io.pointer_parks, 0);
        assert!(io.sources.is_empty() && io.controls.is_empty());
    }


    /// A still inactive board cannot reuse its activation; delayed captures
    /// already consumed before activation remain charged to this context.
    #[test]
    fn inactive_solver_activates_once_without_resetting_observation_allowance() {


        for delayed_before_activation in [false, true] {
            let cancel = AtomicBool::new(false);
            let frames = if delayed_before_activation {
                vec![active_without_halo(), fixture(1), fixture(1), active_without_halo(), active_without_halo()]
            } else {
                vec![fixture(1); 5]
            };
            let mut io = fake(frames, &cancel);
            assert!(run(&mut io, 0, &mut None).unwrap_err().contains("allowance exhausted"));
            assert_eq!(io.solver_clicks, 1);
            assert_eq!(io.solver_capture_counts, [if delayed_before_activation { 3 } else { 2 }]);
            assert_eq!(io.captures, 5);
            let delays = AnimationSettleDelays::default();
            let expected_waits = if delayed_before_activation {
                vec![delays.spider_reobserve, delays.spider_game_start, delays.spider_settle, delays.spider_reobserve]
            } else {
                vec![delays.spider_game_start, delays.spider_settle, delays.spider_reobserve, delays.spider_reobserve]
            };
            assert_eq!(io.waits, expected_waits);
            assert!(io.sources.is_empty() && io.controls.is_empty());
            assert_eq!(io.pointer_parks, 0);
        }
    }


    /// Unknown scenes receive only the bounded input-free captures and retain
    /// the last decoded frame, without Solver, gameplay or false-win input.
    #[test]
    fn unsupported_scene_is_bounded_input_free_and_retains_latest() {
        let cancel = AtomicBool::new(false);
        let mut io = fake(vec![unknown(), unknown(), unknown()], &cancel);
        let mut latest = None;
        assert!(run(&mut io, 0, &mut latest).unwrap_err().contains("allowance exhausted"));
        assert_eq!(io.captures, 3);
        assert!(io.sources.is_empty() && io.controls.is_empty());
        assert_eq!(io.solver_clicks, 0);
        assert!(latest.is_some());
    }


    /// Legacy, SP20 and level-505 optional OK, plus direct New Game, reach Spider's
    /// own Play location. Ordered waits distinguish score settling from the
    /// editable deal; only an inactive Solver receives activation.
    #[test]
    fn continuous_win_restart_handles_optional_level_up_and_spider_play() {


        for level_up_fixture in [Some("FC10"), Some("SP20"), Some("505"), None] {
            let with_level_up = level_up_fixture.is_some();


            for solver_already_active in [false, true] {
                let branch = format!("level_up_fixture={level_up_fixture:?}, solver_already_active={solver_already_active}");
                let cancel = AtomicBool::new(false);
                let delays = AnimationSettleDelays::default().with_spider_game_start_millis(3_500);
                let settings = StepRunSettings::new(delays, 0).with_spider_observation_limit(2);
                let mut frames = vec![fixture(15)];


                if let Some(name) = level_up_fixture {
                    let ready = match name {
                        "FC10" => level_up(),
                        "SP20" => fixture(20),
                        _ => level_up_505(),
                    };
                    frames.extend([ready.clone(), ready]);
                }
                frames.extend([fixture(17), fixture(18)]);


                frames.push(if solver_already_active { active_without_halo() } else { fixture(19) });
                frames.push(fixture(2));
                let mut io = fake(frames, &cancel);
                assert_eq!(run_with_settings(&mut io, settings, &mut None).unwrap_err(),
                    "fixture sequence exhausted", "{branch}");
                let mut controls = vec![TerminalStage::Score];
                let mut waits = vec![LEVEL_UP_APPEAR_DELAY];


                if with_level_up {
                    controls.push(TerminalStage::LevelUp);
                    waits.extend([delays.spider_reobserve, POST_GAME_STAGE_DELAY]);
                }
                controls.extend([TerminalStage::NewGame, TerminalStage::Play]);
                waits.extend([POST_GAME_STAGE_DELAY, delays.spider_game_start]);


                waits.push(if solver_already_active { delays.spider_reobserve } else { delays.spider_settle });
                waits.push(delays.spider_settle);
                let solver_capture_counts = if solver_already_active {
                    Vec::new()
                } else {
                    vec![if with_level_up { 6 } else { 4 }]
                };
                assert_eq!(io.controls, controls, "{branch}");
                if let Some(name) = level_up_fixture {
                    let expected = match name {
                        "FC10" => PixelPoint::new(941, 822),
                        "SP20" => PixelPoint::new(942, 803),
                        _ => PixelPoint::new(944, 771),
                    };
                    assert_eq!(io.control_points[1], expected, "{branch}");
                }
                assert_eq!(io.control_points[io.control_points.len() - 2..],
                    [PixelPoint::new(863, 862), PixelPoint::new(679, 862)], "{branch}");
                assert_eq!(io.solver_clicks, usize::from(!solver_already_active), "{branch}");
                assert_eq!(io.solver_capture_counts, solver_capture_counts, "{branch}");
                assert_eq!(io.sources.len(), 1, "{branch}");
                assert_eq!(io.pointer_parks, 1, "{branch}");
                assert_eq!(io.captures, 6 + 2 * usize::from(with_level_up), "{branch}");
                assert_eq!(io.waits, waits, "{branch}");
            }
        }
    }


    /// Bounded operation requests recognise a win without any restart clicks.
    /// A standalone OK remains insufficient for independent win entry.
    #[test]
    fn finite_win_stops_and_standalone_ok_does_not_create_completion() {
        let cancel = AtomicBool::new(false);
        let mut io = fake(vec![fixture(15)], &cancel);
        assert_eq!(run(&mut io, 1, &mut None), Ok(RunOutcome::GameWon(0)));
        assert!(io.sources.is_empty() && io.controls.is_empty());


        for frame in [level_up(), fixture(20), level_up_505()] {
            let mut io = fake(vec![frame.clone(), frame.clone(), frame], &cancel);
            assert!(run(&mut io, 0, &mut None).unwrap_err().contains("allowance exhausted"));
            assert!(io.sources.is_empty() && io.controls.is_empty());
            assert_eq!(io.solver_clicks, 0);
        }
    }


    /// A score screen that never advances stops after two fresh recaptures rather
    /// than repeating its acknowledged score-skip input.
    #[test]
    fn stalled_terminal_transition_never_repeats_prior_click() {
        let cancel = AtomicBool::new(false);
        let mut io = fake(vec![fixture(15); 4], &cancel);
        assert!(run(&mut io, 0, &mut None).unwrap_err().contains("expected"));
        assert_eq!(io.controls, [TerminalStage::Score]);
        assert_eq!(io.solver_clicks, 0);
        assert!(io.sources.is_empty());
    }


    /// Two settled OK observations can finish on the second or third search.
    #[test]
    fn post_score_retries_find_level_up_and_use_that_frames_coordinates() {
        for attempt in 2..=TERMINAL_CONTROL_SEARCH_ATTEMPTS {
            let cancel = AtomicBool::new(false);
            let mut frames = vec![fixture(15); attempt - 1];
            frames.extend([level_up_505(), level_up_505(), fixture(17), fixture(18), fixture(19), fixture(2)]);
            let mut io = fake(frames, &cancel);
            let settings = StepRunSettings::new(AnimationSettleDelays::default(), 0)
                .with_spider_observation_limit(20);
            let (result, events) = run_with_events(&mut io, settings, &mut None);
            assert_eq!(result.unwrap_err(), "fixture sequence exhausted");
            assert_eq!(io.controls, [TerminalStage::Score, TerminalStage::LevelUp,
                TerminalStage::NewGame, TerminalStage::Play]);
            assert_eq!(io.control_points[1], PixelPoint::new(944, 771));
            let searches: Vec<_> = events.iter().filter_map(|event| match event {
                WorkerEvent::Log(message) if message.starts_with("Spider OK search ") && !message.contains("search summary:") => Some(message),
                _ => None,
            }).collect();
            assert_eq!(searches.len(), attempt);
            assert!(searches[..attempt - 2].iter().all(|search| search.contains("result=(0, 0)")));
            assert!(searches[attempt - 2].contains("result=(944, 771)"));
            assert!(searches[attempt - 1].contains(&format!("search {attempt}:"))
                && searches[attempt - 1].contains("result=(944, 771)"));
            let bounds = SPIDER_CONTROLS.spec(Control::Ok).bounds;
            let logged_bounds = format!("X={}..={}, Y={}..={}",
                bounds.x, bounds.right() - 1, bounds.y, bounds.bottom() - 1);
            assert!(searches.iter().all(|message| message.contains(&logged_bounds)));
            let metrics = format!("searches={attempt}; recaptures={}; second_grab=true", attempt - 1);
            assert!(events.iter().any(|event| matches!(event,
                WorkerEvent::Log(message) if message.starts_with("Spider OK search summary:")
                    && message.contains(&metrics)
                    && message.contains("wall_ms=") && message.ends_with("outcome=matched."))));
            assert!(events.iter().any(|event| matches!(event,
                WorkerEvent::Log(message) if message.starts_with("Spider New Game search summary:")
                    && message.contains(&metrics)
                    && message.ends_with("outcome=optional absent."))));
        }
    }


    /// A third missing result stops after exactly two recaptures and publishes its
    /// own pixels for the egui diagnostic preview, without sending the sentinel.
    #[test]
    fn post_score_third_miss_stops_and_retains_latest_failed_preview() {
        let cancel = AtomicBool::new(false);
        let mut failed = fixture(15);
        failed.pixels[..4].copy_from_slice(&[17, 23, 31, 255]);
        let mut io = fake(vec![fixture(15), fixture(15), fixture(15), failed.clone(), level_up_505()], &cancel);
        let settings = StepRunSettings::new(AnimationSettleDelays::default(), 0)
            .with_spider_observation_limit(20);
        let (sender, receiver) = std::sync::mpsc::channel();
        let slot = super::super::LatestFrameSlot::default();
        let events = WorkerEventSink {
            events: sender, latest_frame: slot.clone(),
            capture_context: std::sync::Mutex::new(None),
        };
        let mut latest = None;
        let mut input_attempted = false;
        let result = drive_run(&mut io, settings, &events, &cancel, &mut latest, &mut input_attempted);
        assert!(result.unwrap_err().contains("after 3 searches and 2 fresh recaptures"));
        assert_eq!(io.captures, 4);
        assert_eq!(io.frames.len(), 1, "a fourth post-score frame must remain unread");
        assert_eq!(io.controls, [TerminalStage::Score]);
        assert_eq!(io.control_points, [PixelPoint::new(960, 550)]);
        assert!(io.sources.is_empty());
        assert_eq!(io.solver_clicks, 0);
        assert_eq!(io.waits, [LEVEL_UP_APPEAR_DELAY, settings.animation_delays().spider_reobserve,
            settings.animation_delays().spider_reobserve]);
        assert_eq!(latest.unwrap().frame.pixels, failed.pixels);
        let preview = slot.take().unwrap();
        assert!(preview.diagnostic);
        assert_eq!(preview.prediction, None);
        assert_eq!(preview.frame.pixels, failed.pixels);
        let searches: Vec<_> = receiver.try_iter().filter_map(|notification| match notification.event {
            WorkerEvent::Log(message) if message.starts_with("Spider OK search ") && !message.contains("search summary:") => Some(message),
            _ => None,
        }).collect();
        assert_eq!(searches.len(), 3);
        assert!(searches.iter().all(|message| message.contains("result=(0, 0)")));
    }


    /// Cancellation at the last probe or after capture sends no input and
    /// preserves the acquired frame; STOP during start delay prevents Solver.
    #[test]
    fn cancellation_preserves_fresh_frames_and_input_boundaries() {


        for fault in [Fault::StopAtProbe, Fault::StopAfterCapture, Fault::StopOnWait] {
            let cancel = AtomicBool::new(false);
            let number = if fault == Fault::StopOnWait { 1 } else { 2 };
            let mut io = fake(vec![fixture(number)], &cancel);
            io.fault = fault;
            let mut latest = None;
            assert!(run(&mut io, 0, &mut latest).unwrap_err().contains("STOP"));
            assert!(io.sources.is_empty() && io.controls.is_empty());
            assert_eq!(io.solver_clicks, 0);
            assert_eq!(latest.unwrap().frame.pixels, fixture(number).pixels);
        }
    }


    /// Source, Solver and terminal delivery failures may already have reached
    /// the guest; all stop after exactly one attempt, without a retry capture.
    #[test]
    fn uncertain_input_classes_are_attempted_once_without_replay() {


        for (fault, number) in [(Fault::Source, 2), (Fault::Solver, 1), (Fault::Terminal, 15)] {
            let cancel = AtomicBool::new(false);
            let mut frames = vec![fixture(number)];


            if fault == Fault::Solver { frames.push(fixture(number)); }
            let mut io = fake(frames, &cancel);
            io.fault = fault;
            assert!(run(&mut io, 0, &mut None).unwrap_err().contains("uncertain"));
            assert_eq!(io.sources.len() + io.controls.len() + io.solver_clicks, 1);
            assert_eq!(io.captures, if fault == Fault::Solver { 2 } else { 1 });
            assert_eq!(io.pointer_parks, 0);
        }
    }


    /// Invalid finite budgets and another mode's previews never cross the
    /// Spider execution boundary, while an empty Spider preview is advisory.
    #[test]
    fn limits_and_advisory_preview_mode_are_checked() {
        assert!(validate_operation_limit(0).is_ok());
        assert!(validate_operation_limit(SPIDER_MAX_MULTI_STEP_ACTIONS).is_ok());
        assert!(validate_operation_limit(SPIDER_MAX_MULTI_STEP_ACTIONS + 1).is_err());
        assert!(require_preview_mode(PredictedAction::NoHighlight).is_ok());
        assert!(require_preview_mode(PredictedAction::CalibrationOnly { mode: GameMode::FreeCell }).is_err());
    }

    /// Ordinary menus can match immediately; OK needs two fresh settled observations.
    #[test]
    fn bounded_control_search_metrics_cover_all_three_success_attempts() {
        for (number, stage, expected) in [
            (20, TerminalStage::LevelUp, PixelPoint::new(942, 803)),
            (17, TerminalStage::NewGame, PixelPoint::new(863, 862)),
            (18, TerminalStage::Play, PixelPoint::new(679, 862))
        ] {
            let confirmations = if stage == TerminalStage::LevelUp { 2 } else { 1 };
            for attempt in confirmations..=TERMINAL_CONTROL_SEARCH_ATTEMPTS {
                let cancel = AtomicBool::new(false);
                let mut frames = vec![fixture(15); attempt - confirmations];
                frames.extend(vec![fixture(number); confirmations]);
                frames.push(fixture(15));
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
                assert_eq!(io.waits, vec![settings.animation_delays().spider_reobserve; attempt - 1]);
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
            let mut second = fixture(15);
            second.pixels[..4].copy_from_slice(&[11, 13, 19, 255]);
            let mut failed = fixture(15);
            failed.pixels[..4].copy_from_slice(&[17, 23, 31, 255]);
            let mut io = fake(vec![fixture(15), second, failed.clone(), fixture(15)], &cancel);
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
            assert_eq!(io.waits, vec![settings.animation_delays().spider_reobserve; 2]);
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
            let mut second = fixture(15);
            second.pixels[..4].copy_from_slice(&[11, 13, 19, 255]);
            let mut io = fake(vec![fixture(15), second.clone(), fixture(20)], &cancel);
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
            assert_eq!(io.waits, vec![settings.animation_delays().spider_reobserve; 2]);
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
            (level_up_505(), TerminalStage::LevelUp, PixelPoint::new(944, 771)),
            (fixture(17), TerminalStage::NewGame, PixelPoint::new(863, 862))
        ] {
            let confirmations = if expected_stage == TerminalStage::LevelUp { 2 } else { 1 };
            for attempt in confirmations..=TERMINAL_CONTROL_SEARCH_ATTEMPTS {
                let cancel = AtomicBool::new(false);
                let mut frames = vec![fixture(15); attempt - confirmations];
                frames.extend(vec![ready.clone(); confirmations]);
                frames.push(fixture(15));
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
                assert_eq!(io.waits, vec![settings.animation_delays().spider_reobserve; attempt - 1]);
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


    /// An ambiguous optional pair stops on its current frame without a retry or input.
    #[test]
    fn simultaneous_post_score_ok_and_new_game_stop_without_recapture() {
        let cancel = AtomicBool::new(false);
        let mut both = fixture(17);
        let ok = level_up_505();
        let bounds = SPIDER_CONTROLS.spec(Control::Ok).bounds;
        for y in bounds.y..bounds.bottom() {
            let start = y as usize * both.stride + bounds.x as usize * 4;
            let end = start + bounds.width as usize * 4;
            both.pixels[start..end].copy_from_slice(&ok.pixels[start..end]);
        }
        assert_ne!(spider_terminal::expected_control_point(&both, TerminalStage::LevelUp).unwrap(),
            PixelPoint::new(0, 0));
        assert_ne!(spider_terminal::expected_control_point(&both, TerminalStage::NewGame).unwrap(),
            PixelPoint::new(0, 0));
        let mut io = fake(vec![fixture(15), both.clone(), level_up_505()], &cancel);
        let mut latest = None;
        assert!(run(&mut io, 0, &mut latest).unwrap_err().contains("both OK and New Game"));
        assert_eq!(io.controls, [TerminalStage::Score]);
        assert_eq!(io.captures, 2);
        assert_eq!(io.frames.len(), 1);
        assert_eq!(io.waits, [LEVEL_UP_APPEAR_DELAY]);
        assert!(io.sources.is_empty());
        assert_eq!(io.solver_clicks, 0);
        assert_eq!(latest.unwrap().frame.pixels, both.pixels);
    }

    /// A first OK match on the last observation cannot bypass fresh confirmation.
    #[test]
    fn third_observation_first_ok_match_stops_without_input_or_extra_capture() {
        for optional_pair in [false, true] {
            let cancel = AtomicBool::new(false);
            let ready = level_up_505();
            let mut io = fake(vec![fixture(15), fixture(15), ready.clone(), ready.clone()], &cancel);
            let settings = StepRunSettings::new(AnimationSettleDelays::default(), 0);
            let (sender, _receiver) = std::sync::mpsc::channel();
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
            assert!(result.unwrap_err().contains("latest failed frame retained"));
            assert_eq!(io.captures, TERMINAL_CONTROL_SEARCH_ATTEMPTS);
            assert_eq!(io.frames.len(), 1, "confirmation cannot acquire a fourth frame");
            assert!(io.controls.is_empty() && io.sources.is_empty());
            assert_eq!(io.solver_clicks, 0);
            assert_eq!(latest.unwrap().frame.pixels, ready.pixels);
            assert_eq!(slot.take().unwrap().frame.pixels, ready.pixels);
        }
    }

    /// A visible static OK remains input-free if STOP interrupts its confirmation.
    #[test]
    fn stop_during_ok_readiness_confirmation_sends_no_terminal_input() {
        for optional_pair in [false, true] {
            let cancel = AtomicBool::new(false);
            let ready = level_up_505();
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
            assert_eq!(io.waits, [settings.animation_delays().spider_reobserve]);
            assert!(io.controls.is_empty() && io.sources.is_empty());
            assert_eq!(io.solver_clicks, 0);
            assert_eq!(latest.unwrap().frame.pixels, ready.pixels);
        }
    }

    /// An acknowledged OK followed by a retained modal never authorises another OK.
    #[test]
    fn settled_ok_acknowledgement_is_not_replayed_when_modal_remains_visible() {
        let cancel = AtomicBool::new(false);
        let ready = level_up_505();
        let mut frames = vec![fixture(15)];
        frames.extend(vec![ready.clone(); 6]);
        let mut io = fake(frames, &cancel);
        let mut latest = None;
        assert!(run(&mut io, 0, &mut latest).unwrap_err().contains("New Game"));
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
        let menu = fixture(17);
        let new_game = SPIDER_CONTROLS.spec(Control::NewGame).bounds;
        for y in new_game.y..new_game.bottom() {
            let start = y as usize * foreground.stride + new_game.x as usize * 4;
            let end = start + new_game.width as usize * 4;
            foreground.pixels[start..end].copy_from_slice(&menu.pixels[start..end]);
        }
        let ok = SPIDER_CONTROLS.spec(Control::Ok).bounds;
        for y in ok.y..ok.bottom() {
            for x in ok.x..ok.right() {
                let offset = y as usize * foreground.stride + x as usize * 4;
                if foreground.pixels[offset..offset + 3] == [0, 0, 0] {
                    foreground.pixels[offset..offset + 3].copy_from_slice(&[35, 35, 35]);
                }
            }
        }
        assert_eq!(crate::terminal_controls::control_point(&foreground, SPIDER_CONTROLS.spec(Control::Ok)).unwrap(),
            PixelPoint::new(0, 0));
        assert_ne!(crate::terminal_controls::control_point(&foreground, SPIDER_CONTROLS.spec(Control::NewGame)).unwrap(),
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

    /// The actual Spider failure now clicks the third-frame glyph despite its grey header.
    /// A retained modal after that acknowledgement still cannot replay OK.
    #[test]
    fn recorded_spider_grey_header_allows_one_third_frame_ok_without_replay() {
        let score = recorded_game_win("score");
        let entering = recorded_game_win("entering");
        let near_black = recorded_game_win("near-black");
        let grey = recorded_game_win("grey-caption");
        assert_eq!(spider_terminal::classify_win_entry(&score).unwrap(), Some(TerminalStage::Score),
            "the native score capture supplies independent Spider GAME WIN entry");
        assert_eq!(spider_terminal::classify_win_entry(&grey).unwrap(), None,
            "a generic OK frame cannot independently declare a Spider win");
        assert_eq!(spider_terminal::expected_control_point(&near_black, TerminalStage::LevelUp).unwrap(),
            PixelPoint::new(0, 0), "near-black ink cannot substitute for an exact-black run");
        let cancel = AtomicBool::new(false);
        let mut io = fake(vec![score, entering, near_black, grey.clone(),
            grey.clone(), grey.clone(), grey.clone(), fixture(17)], &cancel);
        let settings = StepRunSettings::new(AnimationSettleDelays::default(), 0)
            .with_spider_observation_limit(20);
        let mut latest = None;
        let (result, events) = run_with_events(&mut io, settings, &mut latest);
        assert!(result.unwrap_err().contains("New Game"));
        assert_eq!(io.controls, [TerminalStage::Score, TerminalStage::LevelUp]);
        assert_eq!(io.control_points, [PixelPoint::new(960, 550), PixelPoint::new(943, 770)]);
        assert_eq!(io.captures, 7);
        assert_eq!(io.frames.len(), 1, "a fourth post-OK New Game capture is not permitted");
        assert!(io.sources.is_empty());
        assert_eq!(io.solver_clicks, 0);
        assert_eq!(io.pointer_parks, 0);
        let ok_index = io.calls.iter().position(|call|
            *call == IoCall::Terminal(TerminalStage::LevelUp)).unwrap();
        assert_eq!(io.calls[..ok_index].iter().filter(|call| **call == IoCall::Capture).count(), 4,
            "the score entry plus three fresh OK observations precede the sole OK click");
        let readiness: Vec<_> = events.iter().filter_map(|event| match event {
            WorkerEvent::Log(message) if message.starts_with("Spider OK readiness:") => Some(message),
            _ => None,
        }).collect();
        assert_eq!(readiness.len(), TERMINAL_CONTROL_SEARCH_ATTEMPTS);
        assert!(readiness[..2].iter().all(|message| message.contains("ready=false")));
        assert!(readiness[2].contains("caption_present=false")
            && readiness[2].contains("geometry_stable=true")
            && readiness[2].contains("selected=(943, 770)")
            && readiness[2].contains("ready=true")
            && readiness[2].contains("reason=settled-button-and-glyph"));
        assert!(events.iter().any(|event| matches!(event, WorkerEvent::Log(message)
            if message.starts_with("Spider OK search summary: searches=3; recaptures=2;")
                && message.ends_with("outcome=matched."))));
        let delays = settings.animation_delays();
        assert_eq!(io.waits, [LEVEL_UP_APPEAR_DELAY, delays.spider_reobserve,
            delays.spider_reobserve, POST_GAME_STAGE_DELAY, delays.spider_reobserve,
            delays.spider_reobserve]);
        assert_eq!(latest.unwrap().frame.pixels, grey.pixels);
    }

    /// Diagnostic-only header brightness does not relax three observations or STOP.
    #[test]
    fn recorded_spider_missing_glyph_and_stop_preserve_readiness_bounds() {
        for stop_in_confirmation in [false, true] {
            let entering = recorded_game_win("entering");
            let near_black = recorded_game_win("near-black");
            let grey = recorded_game_win("grey-caption");
            let cancel = AtomicBool::new(false);
            let mut io = fake(vec![entering, near_black.clone(), near_black.clone(), grey], &cancel);
            if stop_in_confirmation { io.fault = Fault::StopOnSecondWait; }
            let settings = StepRunSettings::new(AnimationSettleDelays::default(), 0);
            let (sender, _receiver) = std::sync::mpsc::channel();
            let slot = super::super::LatestFrameSlot::default();
            let events = WorkerEventSink {
                events: sender, latest_frame: slot.clone(),
                capture_context: std::sync::Mutex::new(None),
            };
            let mut latest = None;
            capture_latest(&mut io, &mut latest, &events, &cancel).unwrap();
            let error = await_post_score_control(&mut io, &mut latest, settings, &events, &cancel)
                .unwrap_err();
            if stop_in_confirmation {
                assert!(error.contains("STOP"));
                assert_eq!(io.captures, 2);
                assert_eq!(io.frames.len(), 2);
            } else {
                assert!(error.contains("after 3 searches and 2 fresh recaptures"));
                assert_eq!(io.captures, 3);
                assert_eq!(io.frames.len(), 1, "a fourth readiness capture is not permitted");
            }
            assert!(io.controls.is_empty() && io.sources.is_empty());
            assert_eq!(io.solver_clicks, 0);
            assert_eq!(latest.unwrap().frame.pixels, near_black.pixels);
            assert_eq!(slot.take().unwrap().frame.pixels, near_black.pixels);
        }
    }

}
