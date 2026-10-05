//! Solver-led Free Cell execution and its ordered one-board restart sequence.
//!
//! One fresh source HALO authorises one click, followed by editable settling and
//! a new frame. Card identity, changed pixels and previous effects are not input
//! conditions. Automatic transfers receive bounded input-free observations.
//! A missing HALO does not establish a win: positive terminal caption/control
//! evidence does. Only continuous runs advance the expected terminal controls.

use std::{path::Path, sync::atomic::{AtomicBool, Ordering}, time::{Duration, Instant}};

use super::{
    FrameObservation, WorkerEvent, WorkerEventSink, WorkerState, capture_screen,
    connect_and_probe, execute_planned_input, format_action_status,
    format_prediction_target, format_step_plan, milliseconds, send_log,
    send_state, send_status, validate_probe, wait_or_stop,
};
use crate::{
    capture::CapturedFrame,
    freecell,
    freecell_terminal::{self, TerminalStage},
    game::{ActionTarget, GameMode},
    parameters::{
        FREECELL_MAX_MULTI_STEP_ACTIONS, LEVEL_UP_APPEAR_DELAY, NOMINAL_FRAME_HEIGHT,
        NOMINAL_FRAME_WIDTH, POST_GAME_MOUSE_HOLD, POST_GAME_STAGE_DELAY,
        SOLVER_MOUSE_HOLD, StepRunSettings,
    },
    qmp::QmpClient,
    stepper::{StepPlan, plan_step},
    tracker::{PredictedAction, TableauScanState},
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


    /// Activate the positively recognised inactive Solver once per context.
    fn solver(&mut self) -> Result<(), String>;


    /// Click exactly one freshly ready expected terminal control.
    fn terminal(&mut self, stage: TerminalStage) -> Result<(), String>;


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
        let (frame, timing) = capture_screen(self.qmp, self.socket_path)?;
        send_log(self.event_tx, format!(
            "PROFILE Free Cell capture: acquisition={:.1} ms; screendump={:.1} ms, decode={:.1} ms.",
            milliseconds(started.elapsed()), milliseconds(timing.screendump), milliseconds(timing.decode),
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


    fn terminal(&mut self, stage: TerminalStage) -> Result<(), String> {
        require_running(self.cancel_requested)?;
        self.qmp.click_with_hold(
            freecell_terminal::click_point(stage), NOMINAL_FRAME_WIDTH,
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
        "Free Cell fresh-HALO execution: action/automatic-transfer settle={} ms, no-HALO observation={} ms, delayed allowance={}. One source click consumes one action; Solver and terminal clicks do not. No card identity, pixel difference, SUIT input, Draw, Recycle or Solve is used. Continuous 0 may restart one-board games through Score, OK, New Game and Play.",
        delays.freecell_settle.as_millis(), delays.freecell_reobserve.as_millis(), settings.freecell_observation_limit(),
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


    loop {
        require_running(cancel_requested)?;
        let observation = current(latest)?;
        let stage = freecell_terminal::classify_win_entry(&observation.frame)
            .map_err(|error| format!("Free Cell win-entry analysis failed: {error}"))?;


        if stage.is_some() {
            send_log(event_tx, format!("Free Cell independent one-board GAME WIN entry={stage:?}; missing HALO alone did not establish completion."));
            return Ok(stage);
        }


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


            if !solver_active && !solver_reserved {
                guard_input(io, cancel_requested)?;
                solver_reserved = true;
                *input_attempted = true;
                send_state(event_tx, WorkerState::Acting);
                send_status(event_tx, "Activating Free Cell Solver once".to_owned());
                io.solver()?;
                io.wait(settings.animation_delays().freecell_settle)?;
                capture_latest(io, latest, event_tx, cancel_requested)?;
                continue;
            }
        }


        if delayed >= settings.freecell_observation_limit() {
            return Err(format!("Free Cell source/win observation allowance exhausted after {delayed} delayed captures; scene={}, Solver activation reserved={solver_reserved}; no speculative input authorised", observation.gameplay_scene));
        }
        delayed += 1;
        send_log(event_tx, format!("Free Cell no-HALO observation {delayed}/{}: scene={}; waiting {} ms, then input-free recapture; active Solver is not clicked again.", settings.freecell_observation_limit(), observation.gameplay_scene, settings.animation_delays().freecell_reobserve.as_millis()));
        io.wait(settings.animation_delays().freecell_reobserve)?;
        capture_latest(io, latest, event_tx, cancel_requested)?;
    }
}


/// Advance only the expected local control, once, after independent win entry.
fn restart_game(
    io: &mut impl FreeCellIo,
    latest: &mut Option<FrameObservation>,
    initial_stage: TerminalStage,
    settings: StepRunSettings,
    event_tx: &WorkerEventSink,
    cancel_requested: &AtomicBool,
    input_attempted: &mut bool,
) -> Result<(), String> {
    let mut stage = initial_stage;


    loop {
        await_expected_control(io, latest, stage, settings, event_tx, cancel_requested)?;
        guard_input(io, cancel_requested)?;
        send_state(event_tx, WorkerState::Acting);
        send_status(event_tx, stage.to_string());
        send_log(event_tx, format!("Free Cell deterministic GAME WIN control: {stage}; one click at {:?}; unrelated panel colours, levels and artwork are not conditions.", freecell_terminal::click_point(stage)));
        *input_attempted = true;
        io.terminal(stage)?;
        io.wait(if stage == TerminalStage::Score { LEVEL_UP_APPEAR_DELAY } else { POST_GAME_STAGE_DELAY })?;
        capture_latest(io, latest, event_tx, cancel_requested)?;


        match stage {
            TerminalStage::Score => stage = TerminalStage::LevelUp,
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


/// Read only the expected caption/button until ready or the finite allowance ends.
fn await_expected_control(
    io: &mut impl FreeCellIo,
    latest: &mut Option<FrameObservation>,
    stage: TerminalStage,
    settings: StepRunSettings,
    event_tx: &WorkerEventSink,
    cancel_requested: &AtomicBool,
) -> Result<(), String> {


    for delayed in 0..=settings.freecell_observation_limit() {
        require_running(cancel_requested)?;


        if freecell_terminal::expected_control_ready(&current(latest)?.frame, stage)
            .map_err(|error| format!("Free Cell expected {stage} readiness failed: {error}"))?
        {
            return Ok(());
        }


        if delayed == settings.freecell_observation_limit() {
            break;
        }
        send_log(event_tx, format!("Free Cell terminal input-free observation {}/{}: waiting only for expected {stage}; prior click is not repeated.", delayed + 1, settings.freecell_observation_limit()));
        io.wait(settings.animation_delays().freecell_reobserve)?;
        capture_latest(io, latest, event_tx, cancel_requested)?;
    }
    Err(format!("Free Cell expected {stage} did not become ready within the bounded observation allowance; no terminal input retried"))
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
    }


    /// I/O records inputs while all native frame decisions use real classifiers.
    struct FakeIo<'a> {
        /// Successive fresh native frames; exhaustion detects an unexpected capture.
        frames: VecDeque<CapturedFrame>,
        /// Attempted source inputs, including an injected uncertain delivery.
        sources: Vec<PredictedAction>,
        /// Attempted one-shot terminal controls in execution order.
        controls: Vec<TerminalStage>,
        /// Attempted activations, independent from the source-action budget.
        solver_clicks: usize,
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
            let frame = self.frames.pop_front().ok_or_else(|| "fixture sequence exhausted".to_owned())?;


            if self.fault == Fault::StopAfterCapture {
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


            if self.fault == Fault::Solver { Err("injected uncertain Solver".to_owned()) } else { Ok(()) }
        }


        fn terminal(&mut self, stage: TerminalStage) -> Result<(), String> {
            self.controls.push(stage);


            if self.fault == Fault::Terminal { Err("injected uncertain terminal".to_owned()) } else { Ok(()) }
        }


        fn wait(&mut self, duration: Duration) -> Result<(), String> {
            self.waits.push(duration);
            require_running(self.cancel)
        }
    }


    /// Decode the original committed PNG bytes through the production decoder.
    fn fixture(number: u8) -> CapturedFrame {
        let path = format!("{}/tests/fixtures/freecell-FC{number:02}.png", env!("CARGO_MANIFEST_DIR"));
        decode_png(&std::fs::read(path).unwrap()).unwrap()
    }


    /// Black native storage supplies no scene, HALO or terminal evidence.
    fn unknown() -> CapturedFrame {
        CapturedFrame { width: 1_920, height: 1_080, stride: 7_680, format: PixelFormat::Rgba8, pixels: vec![0; 7_680 * 1_080] }
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
        FakeIo { frames: frames.into(), sources: Vec::new(), controls: Vec::new(), solver_clicks: 0, captures: 0, waits: Vec::new(), fault: Fault::None, cancel }
    }


    /// Run the exact production driver with isolated bounded event channels.
    fn run(io: &mut FakeIo<'_>, limit: usize, observations: usize, latest: &mut Option<FrameObservation>) -> Result<RunOutcome, String> {
        let (events, _receiver) = std::sync::mpsc::channel();
        let events = WorkerEventSink {
            events,
            latest_frame: super::super::LatestFrameSlot::default(),
            capture_context: std::sync::Mutex::new(None),
        };
        let settings = StepRunSettings::new(AnimationSettleDelays::default(), limit).with_freecell_observation_limit(observations);
        let mut attempted = false;
        let cancel = io.cancel;
        drive_run(io, settings, &events, cancel, latest, &mut attempted)
    }


    /// Finite one permits Solver activation but exactly one source click and result frame.
    #[test]
    fn initial_solver_activation_then_one_action_and_fresh_result() {
        let cancel = AtomicBool::new(false);
        let mut io = fake(vec![fixture(1), fixture(2), fixture(3)], &cancel);
        let mut latest = None;
        assert_eq!(run(&mut io, 1, 3, &mut latest), Ok(RunOutcome::Completed(1)));
        assert_eq!(io.solver_clicks, 1);
        assert_eq!(io.sources.len(), 1);
        assert_eq!(io.captures, 3);
        assert!(io.controls.is_empty());
        assert_eq!(latest.unwrap().frame.pixels, fixture(3).pixels);
    }


    /// CELL sources, single cards and a multi-card run each consume one source action.
    #[test]
    fn native_source_classes_and_runs_use_one_click_without_pixel_proof() {


        for number in [2, 3, 4, 5, 6, 7, 8] {
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
        let mut io = fake(vec![active_without_halo(), active_without_halo(), fixture(7), fixture(8)], &cancel);
        assert_eq!(run(&mut io, 1, 3, &mut None), Ok(RunOutcome::Completed(1)));
        assert_eq!(io.sources.len(), 1);
        assert_eq!(io.solver_clicks, 0);
        assert_eq!(io.captures, 4);
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


    /// An active no-HALO board never spends its allowance on redundant Solver clicks.
    #[test]
    fn active_solver_without_halo_stops_after_read_only_allowance() {
        let cancel = AtomicBool::new(false);
        let mut io = fake(vec![active_without_halo(), active_without_halo(), active_without_halo()], &cancel);
        assert!(run(&mut io, 0, 2, &mut None).unwrap_err().contains("allowance exhausted"));
        assert_eq!(io.captures, 3);
        assert_eq!(io.solver_clicks, 0);
        assert!(io.sources.is_empty() && io.controls.is_empty());
    }


    /// Both terminal-at-start and source-to-win frames drive the actual ordered restart.
    #[test]
    fn native_game_win_score_ok_new_game_play_solver_restart() {


        for prefix in [false, true] {
            let cancel = AtomicBool::new(false);
            let mut frames = Vec::new();


            if prefix { frames.push(fixture(8)); }
            frames.extend([fixture(9), fixture(10), fixture(11), fixture(12), fixture(13), fixture(2), fixture(3)]);
            let mut io = fake(frames, &cancel);
            let mut latest = None;
            assert_eq!(run(&mut io, 0, 3, &mut latest).unwrap_err(), "fixture sequence exhausted");
            assert_eq!(io.controls, [TerminalStage::Score, TerminalStage::LevelUp, TerminalStage::NewGame, TerminalStage::Play]);
            assert_eq!(io.solver_clicks, 1);
            assert_eq!(io.sources.len(), if prefix { 3 } else { 2 });
            assert_eq!(io.waits.iter().filter(|delay| **delay == LEVEL_UP_APPEAR_DELAY).count(), 1);
        }
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
        let mut io = fake(vec![fixture(9), fixture(9), fixture(10), fixture(11), fixture(12), fixture(13), fixture(2)], &cancel);
        assert_eq!(run(&mut io, 0, 3, &mut None).unwrap_err(), "fixture sequence exhausted");
        assert_eq!(io.controls, [TerminalStage::Score, TerminalStage::LevelUp, TerminalStage::NewGame, TerminalStage::Play]);
    }


    /// An unchanged terminal never receives a second click after acknowledgement.
    #[test]
    fn terminal_advancement_bound_never_replays_the_prior_control() {
        let cancel = AtomicBool::new(false);
        let mut io = fake(vec![fixture(9), fixture(9), fixture(9), fixture(9)], &cancel);
        assert!(run(&mut io, 0, 2, &mut None).unwrap_err().contains("expected"));
        assert_eq!(io.controls, [TerminalStage::Score]);
        assert_eq!(io.captures, 4);
        assert_eq!(io.solver_clicks, 0);
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
            let mut io = fake(vec![fixture(number)], &cancel);
            io.fault = fault;
            assert!(run(&mut io, 0, 2, &mut None).unwrap_err().contains("uncertain"));
            assert_eq!(io.sources.len() + io.controls.len() + io.solver_clicks, 1);
            assert_eq!(io.captures, 1);
        }
    }


    /// Invalid source budgets are rejected without touching capture or guest input.
    #[test]
    fn operation_budget_accepts_continuous_and_bounded_limits() {
        assert!(validate_operation_limit(0).is_ok());
        assert!(validate_operation_limit(FREECELL_MAX_MULTI_STEP_ACTIONS).is_ok());
        assert!(validate_operation_limit(FREECELL_MAX_MULTI_STEP_ACTIONS + 1).is_err());
    }
}
