use std::{
    fs::{self, OpenOptions},
    io::{ErrorKind, Read},
    os::unix::fs::OpenOptionsExt,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use crate::{
    capture::{CapturedFrame, decode_png},
    detector::{detect_game_progress_for_profile, has_dialog_gold_button, pixel_rgb},
    game::{ActionTarget, GameMode, GameProgress, InputOperation},
    geometry::{PixelRect, pixel_point_to_qmp, pixel_rect_to_qmp},
    qmp::QmpClient,
    stepper::{StepPlan, plan_step, verify_post_action},
    tracker::{
        PredictedAction, RowMask, TableauScanState, analyse_frame_with_state,
        is_gameplay_scene_for_mode,
    },
};

use crate::parameters::{
    ACTION_CHANGE_CHANNEL_THRESHOLD, BOARD_TRANSITION_REOBSERVE_DELAY, CANCELLABLE_WAIT_SLICE,
    CAPTURE_NAME_ATTEMPTS, CAPTURE_SEQUENCE, LEVEL_UP_APPEAR_DELAY, MULTI_STEP_INPUT_ENABLED,
    NO_HIGHLIGHT_REOBSERVE_DELAY, NOMINAL_FRAME_HEIGHT, NOMINAL_FRAME_WIDTH,
    POST_GAME_MAX_CLICK_ATTEMPTS, POST_GAME_MAX_OBSERVATION_ROUNDS, POST_GAME_MOUSE_HOLD,
    POST_GAME_STAGE_DELAY, POST_GAME_TARGETS, SCORE_SKIP_CONTROL, SCORE_SKIP_MAX_CLICK_ATTEMPTS,
    SNAPSHOT_MAX_PNG_BYTES, SHARED_SOLVER_CONTROL, SOLVER_MOUSE_HOLD, STEP_ONCE_ACTIONS,
    STEP_ONCE_INPUT_ENABLED,
};
use crate::parameters::{PostGameControlVariant, PostGameStage, PostGameTarget, StepRunSettings};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WorkerState {
    #[default]
    Detached,
    Connecting,
    Capturing,
    Validating,
    Acting,
    Verifying,
    Ready,
    Uncertain,
    Error,
}

impl WorkerState {
    pub const fn label(self) -> &'static str {
        // Returns the concise display label for a worker state.
        match self {
            Self::Detached => "Detached",
            Self::Connecting => "Connecting",
            Self::Capturing => "Capturing",
            Self::Validating => "Validating",
            Self::Acting => "Acting",
            Self::Verifying => "Verifying",
            Self::Ready => "Ready",
            Self::Uncertain => "Uncertain",
            Self::Error => "Error",
        }
    }

    pub const fn is_busy(self) -> bool {
        matches!(
            self,
            Self::Connecting | Self::Capturing | Self::Validating | Self::Acting | Self::Verifying
        )
    }
}

enum WorkerCommand {
    CaptureFrame {
        socket_path: PathBuf,
        mode: GameMode,
    },
    PrepareSnapshot {
        socket_path: PathBuf,
        mode: GameMode,
        request_id: u64,
    },
    ExecuteSteps {
        socket_path: PathBuf,
        mode: GameMode,
        approved_prediction: PredictedAction,
        settings: StepRunSettings,
    },
    ResetProgress,
    Disconnect,
    Shutdown,
}

pub enum WorkerEvent {
    Log(String),
    Status(String),
    SnapshotPrepared {
        request_id: u64,
        context: (PathBuf, GameMode),
        frame: CapturedFrame,
        png: Vec<u8>,
        prediction: PredictedAction,
    },
    SnapshotPreparationFailed {
        request_id: u64,
        error: String,
    },
    BoardProgress {
        current_board: usize,
        completed_boards: usize,
        boards_per_game: usize,
    },
    State(WorkerState),
    ActionCompleted {
        operation_index: usize,
        operation_limit: usize,
        before: PredictedAction,
        after: PredictedAction,
        input_commands: usize,
        input_events: usize,
        changed_pixels: usize,
    },
    GameCompleted,
    RunCompleted {
        prediction: PredictedAction,
        requested_operations: usize,
        verified_operations: usize,
    },
}

pub struct LatestWorkerFrame {
    pub frame: CapturedFrame,
    pub prediction: PredictedAction,
    pub log_prediction: bool,
    pub coalesced_frames: usize,
    pub context: Option<(PathBuf, GameMode)>,
}

struct PendingWorkerFrame {
    frame: CapturedFrame,
    prediction: PredictedAction,
    log_prediction: bool,
    context: Option<(PathBuf, GameMode)>,
}

#[derive(Default)]
struct LatestFrameState {
    pending: Option<PendingWorkerFrame>,
    coalesced_frames: usize,
}

#[derive(Clone, Default)]
struct LatestFrameSlot {
    state: Arc<Mutex<LatestFrameState>>,
}

impl LatestFrameSlot {
    fn publish(
        &self,
        frame: CapturedFrame,
        prediction: PredictedAction,
        log_prediction: bool,
        context: Option<(PathBuf, GameMode)>,
    ) {
        // Preview frames are advisory. Replacing a stale pending frame keeps
        // an occluded or sleeping UI from accumulating full-resolution images.
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state
            .pending
            .replace(PendingWorkerFrame {
                frame,
                prediction,
                log_prediction,
                context,
            })
            .is_some()
        {
            state.coalesced_frames = state.coalesced_frames.saturating_add(1);
        }
    }

    fn take(&self) -> Option<LatestWorkerFrame> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let pending = state.pending.take()?;
        let coalesced_frames = std::mem::take(&mut state.coalesced_frames);
        Some(LatestWorkerFrame {
            frame: pending.frame,
            prediction: pending.prediction,
            log_prediction: pending.log_prediction,
            coalesced_frames,
            context: pending.context,
        })
    }
}

struct WorkerEventSink {
    events: Sender<WorkerEvent>,
    latest_frame: LatestFrameSlot,
    capture_context: Mutex<Option<(PathBuf, GameMode)>>,
}

impl WorkerEventSink {
    fn send(&self, event: WorkerEvent) -> Result<(), mpsc::SendError<WorkerEvent>> {
        self.events.send(event)
    }

    fn publish_frame(
        &self,
        frame: CapturedFrame,
        prediction: PredictedAction,
        log_prediction: bool,
    ) {
        let context = self
            .capture_context
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        self.latest_frame
            .publish(frame, prediction, log_prediction, context);
    }

    fn set_capture_context(&self, context: Option<(PathBuf, GameMode)>) {
        *self
            .capture_context
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = context;
    }
}

pub struct WorkerHandle {
    command_tx: Sender<WorkerCommand>,
    event_rx: Receiver<WorkerEvent>,
    latest_frame: LatestFrameSlot,
    join: Option<JoinHandle<()>>,
    cancel_requested: Arc<AtomicBool>,
}

impl WorkerHandle {
    pub fn spawn() -> Self {
        // Starts the QMP worker thread and creates its command and event channels.
        let (command_tx, command_rx) = mpsc::channel();
        let (event_tx, event_rx) = mpsc::channel();
        let latest_frame = LatestFrameSlot::default();
        let event_sink = WorkerEventSink {
            events: event_tx,
            latest_frame: latest_frame.clone(),
            capture_context: Mutex::new(None),
        };
        let cancel_requested = Arc::new(AtomicBool::new(false));
        let worker_cancel = Arc::clone(&cancel_requested);
        let join = thread::Builder::new()
            .name("qmp-socket".to_owned())
            .spawn(move || run_worker(command_rx, event_sink, worker_cancel))
            .expect("failed to spawn QMP worker thread");

        Self {
            command_tx,
            event_rx,
            latest_frame,
            join: Some(join),
            cancel_requested,
        }
    }

    pub fn capture_frame(&self, socket_path: PathBuf, mode: GameMode) -> Result<(), String> {
        // Requests one read-only QMP screendump and frame analysis.
        self.command_tx
            .send(WorkerCommand::CaptureFrame { socket_path, mode })
            .map_err(|error| format!("QMP worker is unavailable: {error}"))
    }

    pub fn prepare_snapshot(
        &self,
        socket_path: PathBuf,
        mode: GameMode,
        request_id: u64,
    ) -> Result<(), String> {
        self.command_tx
            .send(WorkerCommand::PrepareSnapshot {
                socket_path,
                mode,
                request_id,
            })
            .map_err(|error| format!("QMP worker is unavailable: {error}"))
    }

    pub fn execute_steps(
        &self,
        socket_path: PathBuf,
        mode: GameMode,
        approved_prediction: PredictedAction,
        settings: StepRunSettings,
    ) -> Result<(), String> {
        // Requests one guarded run. Every operation requires fresh
        // validation and emits at most one input sequence.
        ensure_input_authorised(mode)?;
        if !STEP_ONCE_INPUT_ENABLED {
            return Err("guarded input is disabled at compile time".to_owned());
        }
        if (settings.is_unbounded() || settings.operation_limit() > STEP_ONCE_ACTIONS)
            && !MULTI_STEP_INPUT_ENABLED
        {
            return Err("Multi-Step input is disabled at compile time".to_owned());
        }
        self.cancel_requested.store(false, Ordering::Release);
        self.command_tx
            .send(WorkerCommand::ExecuteSteps {
                socket_path,
                mode,
                approved_prediction,
                settings,
            })
            .map_err(|error| format!("QMP worker is unavailable: {error}"))
    }

    pub fn disconnect(&self) -> Result<(), String> {
        // Requests that the worker return to its detached state.
        self.cancel_requested.store(true, Ordering::Release);
        self.command_tx
            .send(WorkerCommand::Disconnect)
            .map_err(|error| format!("QMP worker is unavailable: {error}"))
    }

    pub fn reset_progress(&self) -> Result<(), String> {
        // Resets the board counter and bounded row scan without touching QMP.
        self.command_tx
            .send(WorkerCommand::ResetProgress)
            .map_err(|error| format!("QMP worker is unavailable: {error}"))
    }

    pub fn try_events(&self) -> impl Iterator<Item = WorkerEvent> + '_ {
        // Returns currently queued worker events without blocking the caller.
        self.event_rx.try_iter()
    }

    pub fn take_latest_frame(&self) -> Option<LatestWorkerFrame> {
        // Takes at most one preview frame, together with the number of stale
        // previews replaced since the UI last consumed one.
        self.latest_frame.take()
    }
}

impl Drop for WorkerHandle {
    fn drop(&mut self) {
        // Shuts down and joins the worker thread when its handle is released.
        self.cancel_requested.store(true, Ordering::Release);
        let _ = self.command_tx.send(WorkerCommand::Shutdown);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

fn run_worker(
    command_rx: Receiver<WorkerCommand>,
    event_tx: WorkerEventSink,
    cancel_requested: Arc<AtomicBool>,
) {
    // Processes bounded commands without retaining QMP between transactions.
    let mut scan_state = TableauScanState::initial();
    let mut scan_context = None;
    let mut completed_boards = 0usize;
    while let Ok(command) = command_rx.recv() {
        match command {
            WorkerCommand::CaptureFrame { socket_path, mode } => {
                event_tx.set_capture_context(Some((socket_path.clone(), mode)));
                reset_scan_state_for_context(
                    &socket_path,
                    mode,
                    &mut scan_context,
                    &mut scan_state,
                    &mut completed_boards,
                );
                send_board_progress(&event_tx, &scan_state, completed_boards);
                run_capture(socket_path, &scan_state, &event_tx);
            }
            WorkerCommand::PrepareSnapshot {
                socket_path,
                mode,
                request_id,
            } => {
                event_tx.set_capture_context(Some((socket_path.clone(), mode)));
                reset_scan_state_for_context(
                    &socket_path,
                    mode,
                    &mut scan_context,
                    &mut scan_state,
                    &mut completed_boards,
                );
                send_board_progress(&event_tx, &scan_state, completed_boards);
                run_prepare_snapshot(socket_path, &scan_state, request_id, &event_tx);
            }
            WorkerCommand::ExecuteSteps {
                socket_path,
                mode,
                approved_prediction,
                settings,
            } => {
                event_tx.set_capture_context(Some((socket_path.clone(), mode)));
                reset_scan_state_for_context(
                    &socket_path,
                    mode,
                    &mut scan_context,
                    &mut scan_state,
                    &mut completed_boards,
                );
                send_board_progress(&event_tx, &scan_state, completed_boards);
                run_execute_steps(
                    socket_path,
                    approved_prediction,
                    settings,
                    &mut scan_state,
                    &mut completed_boards,
                    &event_tx,
                    &cancel_requested,
                );
            }
            WorkerCommand::ResetProgress => {
                reset_progress_tracking(&mut scan_state, &mut completed_boards);
                send_board_progress(&event_tx, &scan_state, completed_boards);
                send_status(&event_tx, "Ready".to_owned());
            }
            WorkerCommand::Disconnect => {
                event_tx.set_capture_context(None);
                reset_progress_tracking(&mut scan_state, &mut completed_boards);
                scan_context = None;
                send_board_progress(&event_tx, &scan_state, completed_boards);
                send_state(&event_tx, WorkerState::Detached);
                send_status(&event_tx, "Stopped".to_owned());
                send_log(
                    &event_tx,
                    "Detached. No QMP connection is retained between captures; QEMU and Windows were left running."
                        .to_owned(),
                );
            }
            WorkerCommand::Shutdown => break,
        }
    }
}

fn reset_scan_state_for_context(
    socket_path: &Path,
    mode: GameMode,
    scan_context: &mut Option<(PathBuf, GameMode)>,
    scan_state: &mut TableauScanState,
    completed_boards: &mut usize,
) {
    if scan_context
        .as_ref()
        .map(|(path, active_mode)| (path.as_path(), *active_mode))
        != Some((socket_path, mode))
    {
        *scan_state = TableauScanState::for_mode(mode);
        *completed_boards = 0;
        *scan_context = Some((socket_path.to_path_buf(), mode));
    }
}

fn reset_progress_tracking(scan_state: &mut TableauScanState, completed_boards: &mut usize) {
    // Returns progress tracking to a new game and the profile's initial row scan.
    *scan_state = TableauScanState::for_mode(scan_state.mode());
    *completed_boards = 0;
}

fn run_capture(socket_path: PathBuf, scan_state: &TableauScanState, event_tx: &WorkerEventSink) {
    // Captures and analyses exactly one guest frame without issuing guest input.
    send_state(event_tx, WorkerState::Connecting);
    send_status(event_tx, "Screengrab".to_owned());
    send_log(
        event_tx,
        format!("Connecting to QMP socket: {}", socket_path.display()),
    );

    let result = capture_and_analyse(&socket_path, scan_state, event_tx);
    match result {
        Ok((observation, timing)) => {
            let dimensions = format!("{}x{}", observation.frame.width, observation.frame.height);
            if scan_state.mode().calibration_only() {
                send_log(
                    event_tx,
                    format!(
                        "{} read-only calibration capture retained; detector and guest input remain disabled pending approved coordinates.",
                        scan_state.mode()
                    ),
                );
            } else if !observation.gameplay_scene {
                send_log(
                    event_tx,
                    "WARNING: captured frame is not a recognised gameplay scene; any gold dialog decoration was ignored."
                        .to_owned(),
                );
            }
            event_tx.publish_frame(observation.frame, observation.prediction, true);
            send_state(event_tx, WorkerState::Ready);
            send_status(
                event_tx,
                if scan_state.mode().calibration_only() {
                    format!("{} calibration capture ready", scan_state.mode())
                } else {
                    "Capture ready".to_owned()
                },
            );
            send_log(
                event_tx,
                format!("Read-only QMP capture complete ({dimensions}); input events sent: 0."),
            );
            send_log(
                event_tx,
                format!(
                    "PROFILE capture: reserve={:.1} ms, screendump={:.1} ms, read={:.1} ms, decode={:.1} ms, detection={:.1} ms.",
                    milliseconds(timing.reserve),
                    milliseconds(timing.screendump),
                    milliseconds(timing.file_read),
                    milliseconds(timing.decode),
                    milliseconds(timing.detection),
                ),
            );
        }
        Err(error) => {
            send_state(event_tx, WorkerState::Error);
            send_status(event_tx, "Capture failed".to_owned());
            send_log(event_tx, format!("Read-only capture failed: {error}"));
        }
    }
}

fn run_prepare_snapshot(
    socket_path: PathBuf,
    scan_state: &TableauScanState,
    request_id: u64,
    event_tx: &WorkerEventSink,
) {
    send_state(event_tx, WorkerState::Connecting);
    send_status(event_tx, "Capturing snapshot".to_owned());

    let result = (|| {
        let (mut qmp, probe) = connect_and_probe(&socket_path)?;
        if !probe.is_running() {
            return Err("snapshot refused because the VM is not running".to_owned());
        }
        send_state(event_tx, WorkerState::Capturing);
        let (frame, png, timing) = capture_screen_with_png(&mut qmp, &socket_path)?;
        let profile = scan_state.mode().profile();
        if (frame.width, frame.height) != (profile.frame_width, profile.frame_height)
            || !frame.is_layout_valid()
        {
            return Err(format!(
                "snapshot frame must be exactly {}x{} with a valid layout; got {}x{}",
                profile.frame_width, profile.frame_height, frame.width, frame.height
            ));
        }

        // Recognition is advisory for saving: dialogs must still be
        // capturable, and an unrecognised scene never authorises input.
        let prediction = match analyse_captured_frame(frame.clone(), scan_state) {
            Ok((observation, _)) => observation.prediction,
            Err(error) => {
                send_log(
                    event_tx,
                    format!(
                        "Snapshot scene could not be classified ({error}); preview has no actionable target."
                    ),
                );
                PredictedAction::NoHighlight
            }
        };
        Ok::<_, String>((frame, prediction, png, timing))
    })();

    match result {
        Ok((frame, prediction, png, timing)) => {
            let png_len = png.len();
            let _ = event_tx.send(WorkerEvent::SnapshotPrepared {
                request_id,
                context: (socket_path, scan_state.mode()),
                frame,
                png,
                prediction,
            });
            send_state(event_tx, WorkerState::Ready);
            send_status(event_tx, "Snapshot ready to save".to_owned());
            send_log(
                event_tx,
                format!(
                    "Original QMP PNG prepared for preview ({} bytes); guest input events=0; one screendump={:.1} ms. Saving this capture requires no second screendump.",
                    png_len,
                    milliseconds(timing.screendump),
                ),
            );
        }
        Err(error) => {
            let _ = event_tx.send(WorkerEvent::SnapshotPreparationFailed {
                request_id,
                error: error.clone(),
            });
            send_state(event_tx, WorkerState::Error);
            send_status(event_tx, "Snapshot capture failed".to_owned());
            send_log(event_tx, format!("Read-only snapshot capture failed: {error}"));
        }
    }
}

fn run_execute_steps(
    socket_path: PathBuf,
    approved_prediction: PredictedAction,
    settings: StepRunSettings,
    scan_state: &mut TableauScanState,
    completed_boards: &mut usize,
    event_tx: &WorkerEventSink,
    cancel_requested: &AtomicBool,
) {
    let run_started = Instant::now();
    let game_profile = scan_state.mode().profile();
    if let Err(error) = ensure_input_authorised(scan_state.mode()) {
        send_status(event_tx, "Input disabled".to_owned());
        step_failed_before_input(event_tx, format!("{error}; input sent: 0"));
        return;
    }
    let boards_per_game = game_profile.boards_per_game;
    let operation_limit = settings.operation_limit();
    let operation_scope = format_operation_limit(operation_limit);
    send_state(event_tx, WorkerState::Connecting);
    send_board_progress(event_tx, scan_state, *completed_boards);
    send_status(event_tx, "Validating next move".to_owned());
    send_log(
        event_tx,
        format!(
            "Guarded {} run requested: operations={operation_scope}, capture model=one initial planning frame plus one fresh result frame per action, QMP socket={}, approved preview constraint={}.",
            game_profile.label,
            socket_path.display(),
            format_prediction_target(approved_prediction),
        ),
    );

    let connect_started = Instant::now();
    let (mut qmp, probe) = match connect_and_probe(&socket_path) {
        Ok(connected) => connected,
        Err(error) => {
            step_failed_before_input(event_tx, error);
            return;
        }
    };
    let connect_probe = connect_started.elapsed();
    send_log(event_tx, probe.summary());
    if let Err(error) = validate_probe(&probe) {
        step_failed_before_input(event_tx, format!("{error}; input sent: 0"));
        return;
    }

    if reset_completed_series_for_new_run(scan_state, completed_boards) {
        send_log(
            event_tx,
            format!(
                "A new actionable run followed a completed {boards_per_game}-board game; the board counter and bounded row scan were reset for the new game."
            ),
        );
    }

    let mut planning_frame = PlanningFrame::InitialConstraint(approved_prediction);
    let mut run_profile = RunProfile::new(connect_probe);
    let mut verified_operations = 0usize;
    let mut operation_index = 1usize;

    loop {
        let progress = format_operation_progress(operation_index, operation_limit);
        if cancel_requested.load(Ordering::Acquire) {
            send_state(event_tx, WorkerState::Uncertain);
            send_status(event_tx, "Stopped".to_owned());
            send_log(
                event_tx,
                "Guarded run stopped before the next action because STOP was requested; no further input was sent."
                    .to_owned(),
            );
            log_run_profile(
                event_tx,
                "stopped",
                operation_limit,
                verified_operations,
                run_started.elapsed(),
                &run_profile,
            );
            return;
        }

        match execute_guarded_action(
            &mut qmp,
            &socket_path,
            planning_frame,
            settings,
            scan_state,
            completed_boards,
            event_tx,
            cancel_requested,
        ) {
            Ok(success) => {
                verified_operations = verified_operations.saturating_add(1);
                run_profile.record(success.profile);
                log_action_profile(
                    event_tx,
                    operation_index,
                    operation_limit,
                    "verified",
                    success.profile,
                );
                let input_commands = success.plan.input().qmp_command_count();
                let input_events = success.plan.input().qmp_event_count();
                let _ = event_tx.send(WorkerEvent::ActionCompleted {
                    operation_index,
                    operation_limit,
                    before: success.plan.before(),
                    after: success.after,
                    input_commands,
                    input_events,
                    changed_pixels: success.changed_pixels,
                });
                send_log(
                    event_tx,
                    format!(
                        "Action {progress} verified after {} post-action observation round(s): {} -> {}; changed effect pixels={}, required={}, QMP commands={input_commands}, input events={input_events}, guest-input retries=0.",
                        success.observation_rounds,
                        format_prediction_target(success.plan.before()),
                        format_prediction_target(success.after),
                        success.changed_pixels,
                        success.plan.input().minimum_changed_pixels(),
                    ),
                );

                if success.board_completed {
                    send_board_progress(event_tx, scan_state, success.completed_boards);
                    send_log(
                        event_tx,
                        format!(
                            "Board counter: {}/{} completed in this solver session{}.",
                            success.completed_boards,
                            boards_per_game,
                            if success.series_complete {
                                "; game complete"
                            } else {
                                "; automatic redeal verified"
                            },
                        ),
                    );
                }

                let mut next_observation = success.observation;
                let mut next_prediction = success.after;
                if success.series_complete {
                    match run_post_game_restart(
                        &mut qmp,
                        &socket_path,
                        scan_state,
                        completed_boards,
                        event_tx,
                        cancel_requested,
                    ) {
                        Ok(observation) => {
                            next_prediction = observation.prediction;
                            next_observation = observation;
                            send_board_progress(event_tx, scan_state, *completed_boards);
                            let _ = event_tx.send(WorkerEvent::GameCompleted);
                        }
                        Err(error) => {
                            let state = if cancel_requested.load(Ordering::Acquire) {
                                WorkerState::Ready
                            } else {
                                WorkerState::Uncertain
                            };
                            send_state(event_tx, state);
                            send_status(event_tx, "Post-game recovery stopped".to_owned());
                            send_log(event_tx, error);
                            log_run_profile(
                                event_tx,
                                "post-game restart stopped",
                                operation_limit,
                                verified_operations,
                                run_started.elapsed(),
                                &run_profile,
                            );
                            return;
                        }
                    }
                }

                if settings
                    .bounded_operation_limit()
                    .is_some_and(|limit| operation_index >= limit)
                {
                    event_tx.publish_frame(next_observation.frame, next_prediction, false);
                    let _ = event_tx.send(WorkerEvent::RunCompleted {
                        prediction: next_prediction,
                        requested_operations: operation_limit,
                        verified_operations,
                    });
                    send_state(event_tx, WorkerState::Ready);
                    send_status(event_tx, "Ready".to_owned());
                    log_run_profile(
                        event_tx,
                        "completed",
                        operation_limit,
                        verified_operations,
                        run_started.elapsed(),
                        &run_profile,
                    );
                    return;
                }
                operation_index = operation_index.saturating_add(1);
                planning_frame = PlanningFrame::VerifiedResult(next_observation);
            }
            Err(failure) => {
                run_profile.record(failure.profile);
                log_action_profile(
                    event_tx,
                    operation_index,
                    operation_limit,
                    "stopped",
                    failure.profile,
                );
                if let Some(observation) = failure.observation {
                    match failure.frame_phase {
                        FailureFramePhase::PreAction => send_frame(event_tx, observation),
                        FailureFramePhase::PostAction => {
                            send_post_action_frame(event_tx, observation);
                        }
                    }
                }
                send_state(event_tx, failure.state);
                send_status(event_tx, "Run stopped".to_owned());
                send_log(event_tx, failure.message);
                log_run_profile(
                    event_tx,
                    "stopped",
                    operation_limit,
                    verified_operations,
                    run_started.elapsed(),
                    &run_profile,
                );
                return;
            }
        }
    }
}

fn reset_completed_series_for_new_run(
    scan_state: &mut TableauScanState,
    completed_boards: &mut usize,
) -> bool {
    if *completed_boards < scan_state.mode().profile().boards_per_game {
        return false;
    }
    *completed_boards = 0;
    *scan_state = TableauScanState::for_mode(scan_state.mode());
    true
}

fn run_post_game_restart(
    qmp: &mut QmpClient,
    socket_path: &Path,
    scan_state: &mut TableauScanState,
    completed_boards: &mut usize,
    event_tx: &WorkerEventSink,
    cancel_requested: &AtomicBool,
) -> Result<FrameObservation, String> {
    // Advances the verified end-of-game UI through OK, New Game, Play, and Solver.
    // Every control uses a deliberate hold; an earlier dialog is retried if a
    // fresh capture proves that its prior click was not accepted. Observation
    // and confirmed-delivery click retries are both explicitly bounded.
    let transition_started = Instant::now();
    let mut capture_timing = CaptureTiming::default();
    let mut intentional_wait = Duration::ZERO;
    let mut input_wall = Duration::ZERO;
    let boards_per_game = scan_state.mode().profile().boards_per_game;

    *scan_state = TableauScanState::for_mode(scan_state.mode());
    send_state(event_tx, WorkerState::Verifying);
    send_status(
        event_tx,
        format!(
            "Detected end of game — waiting {} ms",
            POST_GAME_STAGE_DELAY.as_millis()
        ),
    );
    send_log(
        event_tx,
        format!(
            "The {boards_per_game}-board game is complete; waiting {} ms before clicking the centre score panel to skip score counting.",
            POST_GAME_STAGE_DELAY.as_millis(),
        ),
    );
    intentional_wait += wait_or_stop(
        POST_GAME_STAGE_DELAY,
        cancel_requested,
        "STOP was requested before the score-skip click.",
    )?;

    let mut score_skip_click_attempts = 0usize;
    send_state(event_tx, WorkerState::Acting);
    send_status(event_tx, "Skip score counting".to_owned());
    input_wall += send_score_skip_click(qmp, cancel_requested, score_skip_click_attempts)?;
    score_skip_click_attempts = score_skip_click_attempts.saturating_add(1);
    send_log(
        event_tx,
        format!(
            "Score-counting panel click attempt {score_skip_click_attempts}/{SCORE_SKIP_MAX_CLICK_ATTEMPTS} sent at guest pixel ({}, {}) with a {} ms hold; waiting {} ms before classifying the Level Up screen.",
            SCORE_SKIP_CONTROL.click_point.x,
            SCORE_SKIP_CONTROL.click_point.y,
            POST_GAME_MOUSE_HOLD.as_millis(),
            LEVEL_UP_APPEAR_DELAY.as_millis(),
        ),
    );
    send_state(event_tx, WorkerState::Verifying);
    send_status(
        event_tx,
        format!("Waiting {} ms for Level Up", LEVEL_UP_APPEAR_DELAY.as_millis()),
    );
    intentional_wait += wait_or_stop(
        LEVEL_UP_APPEAR_DELAY,
        cancel_requested,
        "STOP was requested during the post-score-skip wait.",
    )?;

    let mut click_attempts = [0usize; POST_GAME_TARGETS.len()];
    let mut recovery_solver_click_attempts = 0usize;
    let mut target_index = 0usize;
    while let Some(target) = POST_GAME_TARGETS.get(target_index).copied() {
        let mut observation_round = 1usize;
        let matched_variant = loop {
            send_status(event_tx, format!("Waiting for {}", target.label));
            let (observation, timing) =
                capture_with_client(qmp, socket_path, scan_state).map_err(|error| {
                    format!(
                        "Post-game {} capture failed before input: {error}.",
                        target.label
                    )
                })?;
            capture_timing.add_assign(timing);
            publish_post_action_observation(event_tx, &observation);

            if observation.gameplay_scene
                && matches!(observation.prediction, PredictedAction::Action(_))
            {
                *completed_boards = 0;
                *scan_state = TableauScanState::for_mode(scan_state.mode());
                let _ = scan_state.reconcile_observation(observation.observed_rows);
                send_log(
                    event_tx,
                    format!(
                        "RECOVERED: expected post-game stage {} but an actionable all-row HALO was found; abandoning the stale board counter and resuming normal play.",
                        target.label,
                    ),
                );
                return Ok(observation);
            }

            if let Some((stale_index, stale_target, stale_variant)) =
                earlier_visible_post_game_target(&observation, target_index)?
            {
                require_post_game_observation_retry_budget(observation_round, target.label)?;
                require_post_game_click_retry_budget(
                    click_attempts[stale_index],
                    stale_target.label,
                )?;
                let reprobe = qmp.probe().map_err(|error| {
                    format!(
                        "Post-game stale {} variant {} QMP probe failed while waiting for {}: {error}",
                        stale_target.label, stale_variant.label, target.label,
                    )
                })?;
                validate_probe(&reprobe).map_err(|error| {
                    format!(
                        "Post-game stale {} variant {} retry refused by the fresh QMP probe: {error}",
                        stale_target.label, stale_variant.label,
                    )
                })?;

                send_state(event_tx, WorkerState::Acting);
                send_status(
                    event_tx,
                    post_game_click_status(stale_target.stage).to_owned(),
                );
                let input_started = Instant::now();
                qmp.click_with_hold(
                    stale_variant.click_point,
                    NOMINAL_FRAME_WIDTH,
                    NOMINAL_FRAME_HEIGHT,
                    POST_GAME_MOUSE_HOLD,
                )
                .map_err(|error| {
                    format!(
                        "Post-game stale {} variant {} retry result is uncertain: {error}. No further input followed the uncertain QMP result.",
                        stale_target.label, stale_variant.label,
                    )
                })?;
                input_wall += input_started.elapsed();
                click_attempts[stale_index] = click_attempts[stale_index].saturating_add(1);
                send_log(
                    event_tx,
                    format!(
                        "RETRY: post-game stage {} variant {} remains visible while waiting for {}; click attempt {} sent at guest pixel ({}, {}) with a {} ms hold. Awaiting a verified screen transition.",
                        stale_target.label,
                        stale_variant.label,
                        target.label,
                        click_attempts[stale_index],
                        stale_variant.click_point.x,
                        stale_variant.click_point.y,
                        POST_GAME_MOUSE_HOLD.as_millis(),
                    ),
                );
                send_state(event_tx, WorkerState::Verifying);
                intentional_wait += wait_or_stop(
                    POST_GAME_STAGE_DELAY,
                    cancel_requested,
                    "STOP was requested after retrying a stale post-game stage.",
                )?;
                observation_round = observation_round.saturating_add(1);
                continue;
            }

            if let Some(variant) = resolve_post_game_target(&observation, target)? {
                break Some(variant);
            }

            if target.stage == PostGameStage::LevelUpOk
                && new_game_visible_while_awaiting_level_up(&observation)?
            {
                send_log(
                    event_tx,
                    "RECOVERED: Level Up OK is absent but the New Game control is visible in a fresh non-gameplay frame. Waiting for a second capture to verify New Game before sending input."
                        .to_owned(),
                );
                intentional_wait += wait_or_stop(
                    POST_GAME_STAGE_DELAY,
                    cancel_requested,
                    "STOP was requested while verifying the New Game transition.",
                )?;
                break None;
            }

            if observation.gameplay_scene
                && matches!(observation.prediction, PredictedAction::NoHighlight)
                && target.stage != PostGameStage::Solver
            {
                require_post_game_observation_retry_budget(observation_round, target.label)?;
                require_post_game_click_retry_budget(
                    recovery_solver_click_attempts,
                    "recovery Solver",
                )?;
                let reprobe = qmp.probe().map_err(|error| {
                    format!("Post-game recovery Solver QMP probe failed: {error}")
                })?;
                validate_probe(&reprobe)
                    .map_err(|error| format!("Post-game recovery Solver click refused: {error}"))?;
                send_state(event_tx, WorkerState::Acting);
                send_status(event_tx, "Click Solver".to_owned());
                let input_started = Instant::now();
                qmp.click_with_hold(
                    SHARED_SOLVER_CONTROL.click_point,
                    NOMINAL_FRAME_WIDTH,
                    NOMINAL_FRAME_HEIGHT,
                    SOLVER_MOUSE_HOLD,
                )
                .map_err(|error| {
                    format!(
                        "Post-game recovery Solver click is uncertain: {error}. No automatic input retry followed the uncertain QMP result."
                    )
                })?;
                input_wall += input_started.elapsed();
                recovery_solver_click_attempts = recovery_solver_click_attempts.saturating_add(1);
                send_log(
                    event_tx,
                    format!(
                        "RECOVERY: expected post-game stage {} but found a gameplay board without a HALO; Solver click attempt {} sent with a {} ms hold before re-orientation.",
                        target.label,
                        recovery_solver_click_attempts,
                        SOLVER_MOUSE_HOLD.as_millis(),
                    ),
                );
                send_state(event_tx, WorkerState::Verifying);
                intentional_wait += wait_or_stop(
                    POST_GAME_STAGE_DELAY,
                    cancel_requested,
                    "STOP was requested after the post-game recovery Solver click.",
                )?;
                observation_round = observation_round.saturating_add(1);
                continue;
            }

            require_post_game_observation_retry_budget(observation_round, target.label)?;
            if score_skip_retry_is_authorised(target.stage, observation.gameplay_scene) {
                send_state(event_tx, WorkerState::Acting);
                send_status(event_tx, "Skip score counting".to_owned());
                input_wall +=
                    send_score_skip_click(qmp, cancel_requested, score_skip_click_attempts)?;
                score_skip_click_attempts = score_skip_click_attempts.saturating_add(1);
                send_log(
                    event_tx,
                    format!(
                        "RETRY: post-game stage {} was not recognised on observation round {observation_round}; score-skip centre click attempt {score_skip_click_attempts}/{SCORE_SKIP_MAX_CLICK_ATTEMPTS} was sent at guest pixel ({}, {}) with a {} ms hold. Waiting {} ms before the next capture; press STOP to end the loop.",
                        target.label,
                        SCORE_SKIP_CONTROL.click_point.x,
                        SCORE_SKIP_CONTROL.click_point.y,
                        POST_GAME_MOUSE_HOLD.as_millis(),
                        LEVEL_UP_APPEAR_DELAY.as_millis(),
                    ),
                );
                send_state(event_tx, WorkerState::Verifying);
                intentional_wait += wait_or_stop(
                    LEVEL_UP_APPEAR_DELAY,
                    cancel_requested,
                    "STOP was requested while waiting after a score-skip retry.",
                )?;
                observation_round = observation_round.saturating_add(1);
                continue;
            }

            send_log(
                event_tx,
                format!(
                    "WAITING: post-game stage {} was not recognised on observation round {observation_round}; allowing the guest to animate for {} ms before the next capture. Press STOP to end the loop.",
                    target.label,
                    BOARD_TRANSITION_REOBSERVE_DELAY.as_millis(),
                ),
            );
            intentional_wait += wait_or_stop(
                BOARD_TRANSITION_REOBSERVE_DELAY,
                cancel_requested,
                "STOP was requested while waiting for the next post-game screen.",
            )?;
            observation_round = observation_round.saturating_add(1);
        };

        let Some(matched_variant) = matched_variant else {
            target_index += 1;
            continue;
        };

        let reprobe = qmp
            .probe()
            .map_err(|error| format!("Post-game {} QMP probe failed: {error}", target.label))?;
        validate_probe(&reprobe).map_err(|error| {
            format!(
                "Post-game {} click refused by the fresh QMP probe: {error}",
                target.label
            )
        })?;

        send_state(event_tx, WorkerState::Acting);
        send_status(event_tx, post_game_click_status(target.stage).to_owned());
        let input_started = Instant::now();
        let mouse_hold = if target.stage == PostGameStage::Solver {
            SOLVER_MOUSE_HOLD
        } else {
            POST_GAME_MOUSE_HOLD
        };
        qmp.click_with_hold(
            matched_variant.click_point,
            NOMINAL_FRAME_WIDTH,
            NOMINAL_FRAME_HEIGHT,
            mouse_hold,
        )
        .map_err(|error| {
            format!(
                "Post-game {} click result is uncertain: {error}. The click was not retried.",
                target.label
            )
        })?;
        input_wall += input_started.elapsed();
        click_attempts[target_index] = click_attempts[target_index].saturating_add(1);
        if target.stage == PostGameStage::Solver {
            send_log(
                event_tx,
                format!(
                    "Post-game stage Solver variant {} recognised; click attempt {} sent at guest pixel ({}, {}) with a {} ms hold; QMP commands=3, input events=4. Awaiting HALO confirmation.",
                    matched_variant.label,
                    click_attempts[target_index],
                    matched_variant.click_point.x,
                    matched_variant.click_point.y,
                    SOLVER_MOUSE_HOLD.as_millis(),
                ),
            );
        } else {
            send_log(
                event_tx,
                format!(
                    "Post-game stage {} variant {} recognised; click attempt {} sent at guest pixel ({}, {}) with a {} ms hold; QMP commands=3, input events=4. Awaiting a fresh capture proving the next stage.",
                    target.label,
                    matched_variant.label,
                    click_attempts[target_index],
                    matched_variant.click_point.x,
                    matched_variant.click_point.y,
                    POST_GAME_MOUSE_HOLD.as_millis(),
                ),
            );
        }

        send_state(event_tx, WorkerState::Verifying);
        send_status(
            event_tx,
            format!(
                "Waiting {} ms for the next game state",
                POST_GAME_STAGE_DELAY.as_millis()
            ),
        );
        intentional_wait += wait_or_stop(
            POST_GAME_STAGE_DELAY,
            cancel_requested,
            "STOP was requested during the one-second post-game stage wait.",
        )?;
        target_index += 1;
    }

    let solver_target = POST_GAME_TARGETS
        .iter()
        .find(|target| target.stage == PostGameStage::Solver)
        .copied()
        .ok_or_else(|| "Post-game Solver target is not configured.".to_owned())?;
    let [solver_variant] = solver_target.control_variants else {
        return Err("Post-game Solver must have exactly one control variant.".to_owned());
    };
    let solver_variant = *solver_variant;
    let mut halo_round = 1usize;
    let mut solver_click_attempts = 1usize;
    let observation = loop {
        let (observation, timing) = capture_with_client(qmp, socket_path, scan_state)
            .map_err(|error| format!("First post-Solver HALO capture failed: {error}"))?;
        capture_timing.add_assign(timing);
        publish_post_action_observation(event_tx, &observation);

        if observation.gameplay_scene
            && matches!(observation.prediction, PredictedAction::Action(_))
        {
            break observation;
        }

        if observation.gameplay_scene
            && matches!(observation.prediction, PredictedAction::NoHighlight)
        {
            require_post_game_observation_retry_budget(halo_round, "post-Solver HALO")?;
            require_post_game_click_retry_budget(solver_click_attempts, "Solver")?;
            let reprobe = qmp
                .probe()
                .map_err(|error| format!("Post-game Solver retry QMP probe failed: {error}"))?;
            validate_probe(&reprobe).map_err(|error| {
                format!("Post-game Solver retry refused by the fresh QMP probe: {error}")
            })?;

            send_state(event_tx, WorkerState::Acting);
            send_status(event_tx, "Click Solver".to_owned());
            let input_started = Instant::now();
            qmp.click_with_hold(
                solver_variant.click_point,
                NOMINAL_FRAME_WIDTH,
                NOMINAL_FRAME_HEIGHT,
                SOLVER_MOUSE_HOLD,
            )
            .map_err(|error| {
                format!(
                    "Post-game Solver retry result is uncertain: {error}. No automatic input retry followed this uncertain QMP result."
                )
            })?;
            input_wall += input_started.elapsed();
            solver_click_attempts = solver_click_attempts.saturating_add(1);
            send_log(
                event_tx,
                format!(
                    "No actionable HALO was visible on observation round {halo_round}; Solver click attempt {solver_click_attempts} sent at guest pixel ({}, {}) with a {} ms hold. Awaiting HALO confirmation; press STOP to end the loop.",
                    solver_variant.click_point.x,
                    solver_variant.click_point.y,
                    SOLVER_MOUSE_HOLD.as_millis(),
                ),
            );

            send_state(event_tx, WorkerState::Verifying);
            intentional_wait += wait_or_stop(
                POST_GAME_STAGE_DELAY,
                cancel_requested,
                "STOP was requested while waiting after a Solver retry.",
            )?;
        } else {
            require_post_game_observation_retry_budget(halo_round, "post-Solver HALO")?;
            send_log(
                event_tx,
                format!(
                    "WAITING: no actionable HALO is visible on observation round {halo_round}, but the Solver control is not safe to retry in this frame; allowing {} ms before the next transition capture. Press STOP to end the loop.",
                    BOARD_TRANSITION_REOBSERVE_DELAY.as_millis(),
                ),
            );
            intentional_wait += wait_or_stop(
                BOARD_TRANSITION_REOBSERVE_DELAY,
                cancel_requested,
                "STOP was requested while waiting for a safe Solver retry frame.",
            )?;
        }

        halo_round = halo_round.saturating_add(1);
    };

    *completed_boards = 0;
    *scan_state = TableauScanState::for_mode(scan_state.mode());
    let _ = scan_state.reconcile_observation(observation.observed_rows);
    send_board_progress(event_tx, scan_state, *completed_boards);
    send_status(event_tx, "Board 1 ready".to_owned());
    send_log(
        event_tx,
        format!(
            "Post-game restart verified: first HALO acquired after {halo_round} post-Solver observation round(s) and {solver_click_attempts} Solver click attempt(s); board counter reset to 0/{boards_per_game}."
        ),
    );
    send_log(
        event_tx,
        format!(
            "PROFILE post-game restart: captures={}, reserve={:.1} ms, screendump={:.1} ms, read={:.1} ms, decode={:.1} ms, detection={:.1} ms, input-wall(inclusive)={:.1} ms, intentional-waits={:.1} ms, total={:.1} ms.",
            capture_timing.captures,
            milliseconds(capture_timing.reserve),
            milliseconds(capture_timing.screendump),
            milliseconds(capture_timing.file_read),
            milliseconds(capture_timing.decode),
            milliseconds(capture_timing.detection),
            milliseconds(input_wall),
            milliseconds(intentional_wait),
            milliseconds(transition_started.elapsed()),
        ),
    );

    Ok(observation)
}

fn resolve_post_game_target(
    observation: &FrameObservation,
    target: PostGameTarget,
) -> Result<Option<PostGameControlVariant>, String> {
    // Resolve the exact control variant that supplied fresh visual authority.
    // Multiple matching variants are ambiguous and must never select a click.
    if target.requires_gameplay_scene {
        let [variant] = target.control_variants else {
            return Err(format!(
                "{} must have exactly one gameplay control variant",
                target.label
            ));
        };
        return Ok((observation.gameplay_scene
            && target.stage == PostGameStage::Solver
            && matches!(observation.prediction, PredictedAction::NoHighlight))
        .then_some(*variant));
    }

    if observation.gameplay_scene {
        return Ok(None);
    }

    if target.control_variants.is_empty() {
        return Err(format!(
            "{} has no configured control variants",
            target.label
        ));
    }

    let mut matched_variant: Option<PostGameControlVariant> = None;
    for variant in target.control_variants.iter().copied() {
        let is_visible =
            has_dialog_gold_button(&observation.frame, variant.probe_bounds).map_err(|error| {
                format!(
                    "{} variant {} target analysis failed: {error}",
                    target.label, variant.label
                )
            })?;
        if !is_visible {
            continue;
        }

        if let Some(previous) = matched_variant {
            return Err(format!(
                "{} target is ambiguous: variants {} and {} both matched; no guest input was sent",
                target.label, previous.label, variant.label
            ));
        }
        matched_variant = Some(variant);
    }

    Ok(matched_variant)
}

fn earlier_visible_post_game_target(
    observation: &FrameObservation,
    expected_target_index: usize,
) -> Result<Option<(usize, PostGameTarget, PostGameControlVariant)>, String> {
    // A QMP acknowledgement proves delivery, not that the guest accepted the
    // click. Re-detect earlier controls before authorising a later-stage click.
    let earlier_targets = POST_GAME_TARGETS
        .get(..expected_target_index)
        .ok_or_else(|| format!("invalid post-game target index: {expected_target_index}"))?;
    for (index, target) in earlier_targets.iter().copied().enumerate().rev() {
        if let Some(variant) = resolve_post_game_target(observation, target)? {
            return Ok(Some((index, target, variant)));
        }
    }

    Ok(None)
}

fn level_up_overrides_board_progress(
    board_completion_verified: bool,
    observation: &FrameObservation,
) -> Result<bool, String> {
    // A recognised Level Up dialog is stronger evidence than the earlier
    // progress-bar branch and prevents a stale redeal state from looping.
    if !board_completion_verified {
        return Ok(false);
    }

    Ok(resolve_post_game_target(observation, POST_GAME_TARGETS[0])?.is_some())
}

fn new_game_visible_while_awaiting_level_up(
    observation: &FrameObservation,
) -> Result<bool, String> {
    Ok(resolve_post_game_target(observation, POST_GAME_TARGETS[1])?.is_some())
}

fn require_post_game_observation_retry_budget(
    observation_round: usize,
    target_label: &str,
) -> Result<(), String> {
    if observation_round < POST_GAME_MAX_OBSERVATION_ROUNDS {
        return Ok(());
    }

    Err(format!(
        "Post-game {target_label} was not safely resolved after {observation_round} observation rounds; the bounded retry budget is exhausted and no further guest input was sent."
    ))
}

fn require_post_game_click_retry_budget(
    completed_attempts: usize,
    target_label: &str,
) -> Result<(), String> {
    if completed_attempts < POST_GAME_MAX_CLICK_ATTEMPTS {
        return Ok(());
    }

    Err(format!(
        "Post-game {target_label} remained visible after {completed_attempts} confirmed-delivery click attempts; the bounded click budget is exhausted and no further guest input was sent."
    ))
}

fn require_score_skip_click_budget(completed_attempts: usize) -> Result<(), String> {
    if completed_attempts < SCORE_SKIP_MAX_CLICK_ATTEMPTS {
        return Ok(());
    }

    Err(format!(
        "Level Up remained unrecognised after {completed_attempts} confirmed-delivery score-skip centre clicks; the bounded score-skip budget is exhausted and no further guest input was sent."
    ))
}

// Context may authorise another centre click only while waiting for Level Up
// on a non-gameplay frame. An ambiguous gameplay frame must remain input-free.
fn score_skip_retry_is_authorised(expected_stage: PostGameStage, gameplay_scene: bool) -> bool {
    expected_stage == PostGameStage::LevelUpOk && !gameplay_scene
}

fn send_score_skip_click(
    qmp: &mut QmpClient,
    cancel_requested: &AtomicBool,
    completed_attempts: usize,
) -> Result<Duration, String> {
    require_score_skip_click_budget(completed_attempts)?;
    let attempt = completed_attempts.saturating_add(1);

    if cancel_requested.load(Ordering::Acquire) {
        return Err(format!(
            "STOP was requested before score-skip click attempt {attempt}; no guest input was sent."
        ));
    }

    let reprobe = qmp
        .probe()
        .map_err(|error| format!("Score-skip click attempt {attempt} QMP probe failed: {error}"))?;
    validate_probe(&reprobe).map_err(|error| {
        format!("Score-skip click attempt {attempt} was refused by the fresh QMP probe: {error}")
    })?;

    if cancel_requested.load(Ordering::Acquire) {
        return Err(format!(
            "STOP was requested after the score-skip probe and before click attempt {attempt}; no guest input was sent."
        ));
    }

    let input_started = Instant::now();
    qmp.click_with_hold(
        SCORE_SKIP_CONTROL.click_point,
        NOMINAL_FRAME_WIDTH,
        NOMINAL_FRAME_HEIGHT,
        POST_GAME_MOUSE_HOLD,
    )
    .map_err(|error| {
        format!(
            "Score-skip click attempt {attempt} result is uncertain: {error}. No automatic retry followed the uncertain QMP result."
        )
    })?;
    Ok(input_started.elapsed())
}

fn publish_post_action_observation(event_tx: &WorkerEventSink, observation: &FrameObservation) {
    event_tx.publish_frame(observation.frame.clone(), observation.prediction, false);
}

fn wait_or_stop(
    duration: Duration,
    cancel_requested: &AtomicBool,
    stopped_message: &str,
) -> Result<Duration, String> {
    cancellable_wait(duration, cancel_requested).map_err(|_| stopped_message.to_owned())
}

#[derive(Clone, Copy, Debug, Default)]
struct CaptureTiming {
    captures: usize,
    reserve: Duration,
    screendump: Duration,
    file_read: Duration,
    decode: Duration,
    detection: Duration,
}

impl CaptureTiming {
    fn add_assign(&mut self, other: Self) {
        self.captures = self.captures.saturating_add(other.captures);
        self.reserve += other.reserve;
        self.screendump += other.screendump;
        self.file_read += other.file_read;
        self.decode += other.decode;
        self.detection += other.detection;
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct ActionProfile {
    capture: CaptureTiming,
    validation_effect: Duration,
    input_wall: Duration,
    intentional_wait: Duration,
    total: Duration,
    input_attempted: bool,
}

struct RunProfile {
    connect_probe: Duration,
    cycles: usize,
    attempted: usize,
    totals: ActionProfile,
    action_total_sum: Duration,
    action_total_min: Option<Duration>,
    action_total_max: Option<Duration>,
}

impl RunProfile {
    const fn new(connect_probe: Duration) -> Self {
        Self {
            connect_probe,
            cycles: 0,
            attempted: 0,
            totals: ActionProfile {
                capture: CaptureTiming {
                    captures: 0,
                    reserve: Duration::ZERO,
                    screendump: Duration::ZERO,
                    file_read: Duration::ZERO,
                    decode: Duration::ZERO,
                    detection: Duration::ZERO,
                },
                validation_effect: Duration::ZERO,
                input_wall: Duration::ZERO,
                intentional_wait: Duration::ZERO,
                total: Duration::ZERO,
                input_attempted: false,
            },
            action_total_sum: Duration::ZERO,
            action_total_min: None,
            action_total_max: None,
        }
    }

    fn record(&mut self, profile: ActionProfile) {
        self.cycles = self.cycles.saturating_add(1);
        if profile.input_attempted {
            self.attempted = self.attempted.saturating_add(1);
            self.action_total_sum += profile.total;
            self.action_total_min = Some(
                self.action_total_min
                    .map_or(profile.total, |current| current.min(profile.total)),
            );
            self.action_total_max = Some(
                self.action_total_max
                    .map_or(profile.total, |current| current.max(profile.total)),
            );
        }
        self.totals.capture.add_assign(profile.capture);
        self.totals.validation_effect += profile.validation_effect;
        self.totals.input_wall += profile.input_wall;
        self.totals.intentional_wait += profile.intentional_wait;
        self.totals.total += profile.total;
    }
}

struct ActionSuccess {
    plan: StepPlan,
    after: PredictedAction,
    observation: FrameObservation,
    changed_pixels: usize,
    observation_rounds: usize,
    board_completed: bool,
    series_complete: bool,
    completed_boards: usize,
    profile: ActionProfile,
}

#[derive(Clone, Copy)]
enum FailureFramePhase {
    PreAction,
    PostAction,
}

struct ActionFailure {
    state: WorkerState,
    message: String,
    observation: Option<FrameObservation>,
    frame_phase: FailureFramePhase,
    profile: ActionProfile,
}

enum PlanningFrame {
    /// The advisory UI prediction is only a constraint. One fresh capture must
    /// reproduce it before the first input in a run.
    InitialConstraint(PredictedAction),
    /// A fresh post-action frame whose effect and resulting state were already
    /// verified. It is the next action's immutable planning frame.
    VerifiedResult(FrameObservation),
}

fn execute_guarded_action(
    qmp: &mut QmpClient,
    socket_path: &Path,
    planning_frame: PlanningFrame,
    settings: StepRunSettings,
    scan_state: &mut TableauScanState,
    completed_boards: &mut usize,
    event_tx: &WorkerEventSink,
    cancel_requested: &AtomicBool,
) -> Result<ActionSuccess, ActionFailure> {
    let action_started = Instant::now();
    let mut profile = ActionProfile::default();
    let boards_per_game = scan_state.mode().profile().boards_per_game;

    if cancel_requested.load(Ordering::Acquire) {
        return Err(action_failure(
            action_started,
            profile,
            WorkerState::Ready,
            "STOP was requested before action planning; input sent: 0.".to_owned(),
            None,
            FailureFramePhase::PreAction,
        ));
    }

    send_state(event_tx, WorkerState::Validating);
    let (mut before_series, plan) = match planning_frame {
        PlanningFrame::InitialConstraint(expected_prediction) => {
            let mut pre_action_round = 1usize;
            loop {
                send_status(event_tx, "Screengrab — validating first move".to_owned());
                let pre_action_scan_state = *scan_state;
                let mut candidate_series = match capture_series(
                    qmp,
                    socket_path,
                    &pre_action_scan_state,
                    cancel_requested,
                ) {
                    Ok(series) => series,
                    Err(error) => {
                        let state = if cancel_requested.load(Ordering::Acquire) {
                            WorkerState::Ready
                        } else {
                            WorkerState::Error
                        };
                        return Err(action_failure(
                            action_started,
                            profile,
                            state,
                            format!(
                                "Initial planning capture failed closed: {error}; input sent: 0."
                            ),
                            None,
                            FailureFramePhase::PreAction,
                        ));
                    }
                };
                profile.capture.add_assign(candidate_series.timing);

                let validation_started = Instant::now();
                if candidate_series
                    .observations
                    .iter()
                    .any(|observation| !observation.gameplay_scene)
                {
                    profile.validation_effect += validation_started.elapsed();
                    let observation = candidate_series.observations.pop();
                    return Err(action_failure(
                        action_started,
                        profile,
                        WorkerState::Uncertain,
                        "Initial planning frame is not a recognised gameplay scene; run stopped and input sent: 0."
                            .to_owned(),
                        observation,
                        FailureFramePhase::PreAction,
                    ));
                }
                let candidate_prediction = candidate_series.observations[0].prediction;
                let candidate_plan = plan_step(candidate_prediction);
                let plan = match candidate_plan {
                    Ok(plan) => plan,
                    Err(error) => {
                        reconcile_observation_series(scan_state, &candidate_series.observations);
                        profile.validation_effect += validation_started.elapsed();
                        send_log(
                            event_tx,
                            format!(
                                "WAITING: initial planning observation round {pre_action_round} has no stable target ({error}); waiting {} ms, then repeating capture/detection only. Press STOP to end the loop; input sent: 0.",
                                NO_HIGHLIGHT_REOBSERVE_DELAY.as_millis(),
                            ),
                        );
                        match cancellable_wait(NO_HIGHLIGHT_REOBSERVE_DELAY, cancel_requested) {
                            Ok(waited) => profile.intentional_wait += waited,
                            Err(waited) => {
                                profile.intentional_wait += waited;
                                let observation = candidate_series.observations.pop();
                                return Err(action_failure(
                                    action_started,
                                    profile,
                                    WorkerState::Ready,
                                    "STOP was requested during initial planning re-observation; input sent: 0."
                                        .to_owned(),
                                    observation,
                                    FailureFramePhase::PreAction,
                                ));
                            }
                        }
                        pre_action_round = pre_action_round.saturating_add(1);
                        continue;
                    }
                };
                if plan.before() != expected_prediction {
                    profile.validation_effect += validation_started.elapsed();
                    let observation = candidate_series.observations.pop();
                    return Err(action_failure(
                        action_started,
                        profile,
                        WorkerState::Ready,
                        format!(
                            "Fresh initial target {} differs from approved preview target {}; run stopped and input sent: 0.",
                            format_prediction_target(plan.before()),
                            format_prediction_target(expected_prediction),
                        ),
                        observation,
                        FailureFramePhase::PreAction,
                    ));
                }
                profile.validation_effect += validation_started.elapsed();
                break (candidate_series, plan);
            }
        }
        PlanningFrame::VerifiedResult(observation) => {
            send_status(event_tx, "Planning from verified result".to_owned());
            send_log(
                event_tx,
                "Reusing the prior action's fresh verified result frame as the next planning frame; redundant pre-action screendump=0."
                    .to_owned(),
            );
            let validation_started = Instant::now();
            let mut candidate_series = CaptureSeries {
                observations: vec![observation],
                timing: CaptureTiming::default(),
            };
            if !candidate_series.observations[0].gameplay_scene {
                profile.validation_effect += validation_started.elapsed();
                let observation = candidate_series.observations.pop();
                return Err(action_failure(
                    action_started,
                    profile,
                    WorkerState::Uncertain,
                    "Verified result frame is no longer an actionable gameplay state; run stopped and input sent: 0."
                        .to_owned(),
                    observation,
                    FailureFramePhase::PreAction,
                ));
            }
            let candidate_prediction = candidate_series.observations[0].prediction;
            let plan = match plan_step(candidate_prediction) {
                Ok(plan) => plan,
                Err(error) => {
                    profile.validation_effect += validation_started.elapsed();
                    let observation = candidate_series.observations.pop();
                    return Err(action_failure(
                        action_started,
                        profile,
                        WorkerState::Uncertain,
                        format!(
                            "Verified result frame cannot authorise the next action ({error}); run stopped and input sent: 0."
                        ),
                        observation,
                        FailureFramePhase::PreAction,
                    ));
                }
            };
            profile.validation_effect += validation_started.elapsed();
            (candidate_series, plan)
        }
    };

    let effect_bounds = plan.input().effect_bounds();
    let exclusion_bounds = plan.input().effect_exclusion_bounds();
    let required_changed_pixels = plan.input().minimum_changed_pixels();
    let validation_started = Instant::now();
    reconcile_observation_series(scan_state, &before_series.observations);
    let plan_description = match format_step_plan(plan) {
        Ok(description) => description,
        Err(error) => {
            profile.validation_effect += validation_started.elapsed();
            let observation = before_series.observations.pop();
            return Err(action_failure(
                action_started,
                profile,
                WorkerState::Error,
                format!("Action geometry refused: {error}; input sent: 0."),
                observation,
                FailureFramePhase::PreAction,
            ));
        }
    };
    profile.validation_effect += validation_started.elapsed();
    send_log(event_tx, plan_description);

    if cancel_requested.load(Ordering::Acquire) {
        let observation = before_series.observations.pop();
        return Err(action_failure(
            action_started,
            profile,
            WorkerState::Ready,
            "STOP was requested before input; input sent: 0.".to_owned(),
            observation,
            FailureFramePhase::PreAction,
        ));
    }

    let reprobe_started = Instant::now();
    let reprobe = qmp
        .probe()
        .map_err(|error| format!("QMP pre-input probe failed: {error}"));
    profile.validation_effect += reprobe_started.elapsed();
    let reprobe = match reprobe {
        Ok(probe) => probe,
        Err(error) => {
            let observation = before_series.observations.pop();
            return Err(action_failure(
                action_started,
                profile,
                WorkerState::Error,
                format!("{error}; input sent: 0."),
                observation,
                FailureFramePhase::PreAction,
            ));
        }
    };
    if let Err(error) = validate_probe(&reprobe) {
        let observation = before_series.observations.pop();
        return Err(action_failure(
            action_started,
            profile,
            WorkerState::Ready,
            format!("Pre-input probe refused the action: {error}; input sent: 0."),
            observation,
            FailureFramePhase::PreAction,
        ));
    }
    if cancel_requested.load(Ordering::Acquire) {
        let observation = before_series.observations.pop();
        return Err(action_failure(
            action_started,
            profile,
            WorkerState::Ready,
            "STOP was requested after the pre-input probe; input sent: 0.".to_owned(),
            observation,
            FailureFramePhase::PreAction,
        ));
    }

    send_state(event_tx, WorkerState::Acting);
    send_status(event_tx, format_action_status(plan.before()));
    profile.input_attempted = true;
    let input_started = Instant::now();
    let input_result = execute_planned_input(qmp, plan, cancel_requested);
    profile.input_wall += input_started.elapsed();
    if let Err(error) = input_result {
        return Err(action_failure(
            action_started,
            profile,
            WorkerState::Uncertain,
            format!(
                "Input result is uncertain: {error}. No retry was attempted; inspect the guest before another run."
            ),
            None,
            FailureFramePhase::PostAction,
        ));
    }
    profile.intentional_wait += plan.input().intentional_input_wait();

    let animation_settle_delay = plan
        .input()
        .animation_settle_delay(settings.animation_delays());
    match cancellable_wait(animation_settle_delay, cancel_requested) {
        Ok(waited) => profile.intentional_wait += waited,
        Err(waited) => {
            profile.intentional_wait += waited;
            return Err(action_failure(
                action_started,
                profile,
                WorkerState::Uncertain,
                "STOP was requested during the animation-settle wait. Input was sent once and was not retried."
                    .to_owned(),
                None,
                FailureFramePhase::PostAction,
            ));
        }
    }

    send_state(event_tx, WorkerState::Verifying);
    send_status(event_tx, "Verifying move".to_owned());
    let mut observation_round = 1usize;
    let mut board_completion_verified = false;
    let mut board_changed_pixels = 0usize;
    let mut missing_halo_phase = 0u8;
    let mut solver_recovery_clicks = 0usize;
    let (after, observation, changed_pixels, board_completed, series_complete) = loop {
        send_log(
            event_tx,
            format!(
                "Post-action observation round {observation_round}: capturing one fresh frame; read-only retries continue until a verified target or STOP."
            ),
        );
        let post_action_scan_state = *scan_state;
        let mut after_series = match capture_series(
            qmp,
            socket_path,
            &post_action_scan_state,
            cancel_requested,
        ) {
            Ok(series) => series,
            Err(error) => {
                return Err(action_failure(
                    action_started,
                    profile,
                    WorkerState::Uncertain,
                    format!(
                        "Post-action capture failed after input: {error}. Guest input was not retried."
                    ),
                    None,
                    FailureFramePhase::PostAction,
                ));
            }
        };
        profile.capture.add_assign(after_series.timing);

        let verification_started = Instant::now();
        let recognised_gameplay = after_series
            .observations
            .iter()
            .all(|observation| observation.gameplay_scene);
        let after_predictions: Vec<_> = after_series
            .observations
            .iter()
            .map(|observation| observation.prediction)
            .collect();
        if after_predictions
            .iter()
            .any(|prediction| matches!(prediction, PredictedAction::NoHighlight))
        {
            reconcile_observation_series(scan_state, &after_series.observations);
            let no_highlight_observation = after_series.observations.pop();

            let level_up_visible = match no_highlight_observation.as_ref() {
                Some(observation) => {
                    match level_up_overrides_board_progress(board_completion_verified, observation)
                    {
                        Ok(visible) => visible,
                        Err(error) => {
                            profile.validation_effect += verification_started.elapsed();
                            return Err(action_failure(
                                action_started,
                                profile,
                                WorkerState::Uncertain,
                                format!(
                                    "Level Up recovery analysis failed after a completed board: {error}"
                                ),
                                no_highlight_observation,
                                FailureFramePhase::PostAction,
                            ));
                        }
                    }
                }
                None => false,
            };
            if level_up_visible {
                *completed_boards = boards_per_game;
                profile.validation_effect += verification_started.elapsed();
                send_log(
                    event_tx,
                    "RECOVERED: Level Up OK is visible during the expected redeal; overriding the stale board-progress classification and entering the game-win sequence."
                        .to_owned(),
                );
                let Some(observation) = no_highlight_observation else {
                    return Err(action_failure(
                        action_started,
                        profile,
                        WorkerState::Uncertain,
                        "Level Up recovery produced no display frame.".to_owned(),
                        None,
                        FailureFramePhase::PostAction,
                    ));
                };
                break (
                    PredictedAction::NoHighlight,
                    observation,
                    board_changed_pixels,
                    true,
                    true,
                );
            }

            let no_rows_remain = no_highlight_observation
                .as_ref()
                .is_some_and(|observation| observation.observed_rows.is_none());
            let completion_check_authorised = !board_completion_verified
                && no_highlight_observation
                    .as_ref()
                    .is_some_and(|observation| {
                        settled_completion_evidence(
                            observation.prediction,
                            observation.observed_rows,
                            observation.gameplay_scene,
                            missing_halo_phase,
                            solver_recovery_clicks,
                        )
                    });
            if completion_check_authorised {
                let completion_changed = match before_series
                    .observations
                    .last()
                    .zip(no_highlight_observation.as_ref())
                    .ok_or_else(|| {
                        "board-completion capture result was unexpectedly empty".to_owned()
                    })
                    .and_then(|(before, after)| {
                        materially_changed_pixels(
                            &before.frame,
                            &after.frame,
                            effect_bounds,
                            exclusion_bounds,
                            ACTION_CHANGE_CHANNEL_THRESHOLD,
                        )
                    }) {
                    Ok(changed) => changed,
                    Err(error) => {
                        profile.validation_effect += verification_started.elapsed();
                        return Err(action_failure(
                            action_started,
                            profile,
                            WorkerState::Uncertain,
                            format!(
                                "Board-completion effect comparison failed: {error}. Guest input was not retried."
                            ),
                            no_highlight_observation,
                            FailureFramePhase::PostAction,
                        ));
                    }
                };
                if completion_changed >= required_changed_pixels {
                    let game_progress = no_highlight_observation
                        .as_ref()
                        .and_then(|observation| observation.game_progress)
                        .unwrap_or(GameProgress::AnotherBoard);
                    let series_complete = game_progress == GameProgress::GameComplete;
                    *completed_boards = if series_complete {
                        boards_per_game
                    } else {
                        (*completed_boards)
                            .saturating_add(1)
                            .min(boards_per_game.saturating_sub(1))
                    };
                    board_completion_verified = true;
                    board_changed_pixels = completion_changed;
                    send_board_progress(event_tx, scan_state, *completed_boards);
                    send_log(
                        event_tx,
                        format!(
                            "Board {}/{} completion verified only after an all-row HALO search, a 500 ms Solver recovery click, and a settled no-card observation; progress bar={game_progress:?}, changed effect pixels={completion_changed} (required={required_changed_pixels}).",
                            *completed_boards, boards_per_game,
                        ),
                    );
                    send_status(
                        event_tx,
                        format!(
                            "Waiting {} ms for board {}",
                            settings.animation_delays().board_redeal.as_millis(),
                            completed_boards.saturating_add(1),
                        ),
                    );
                    if series_complete {
                        profile.validation_effect += verification_started.elapsed();
                        let Some(observation) = no_highlight_observation else {
                            return Err(action_failure(
                                action_started,
                                profile,
                                WorkerState::Uncertain,
                                "Board completion produced no display frame. Guest input was not retried."
                                    .to_owned(),
                                None,
                                FailureFramePhase::PostAction,
                            ));
                        };
                        break (
                            PredictedAction::NoHighlight,
                            observation,
                            completion_changed,
                            true,
                            true,
                        );
                    }

                    *scan_state = TableauScanState::for_mode(scan_state.mode());
                    profile.validation_effect += verification_started.elapsed();
                    send_log(
                        event_tx,
                        format!(
                            "Board {}/{} complete; waiting {} ms for the automatic redeal before capture/detection resumes.",
                            *completed_boards,
                            boards_per_game,
                            settings.animation_delays().board_redeal.as_millis(),
                        ),
                    );
                    match cancellable_wait(
                        settings.animation_delays().board_redeal,
                        cancel_requested,
                    ) {
                        Ok(waited) => profile.intentional_wait += waited,
                        Err(waited) => {
                            profile.intentional_wait += waited;
                            return Err(action_failure(
                                action_started,
                                profile,
                                WorkerState::Ready,
                                "STOP was requested during the board-redeal wait; the completed-board move was not retried."
                                    .to_owned(),
                                no_highlight_observation,
                                FailureFramePhase::PostAction,
                            ));
                        }
                    }

                    let reprobe = match qmp.probe() {
                        Ok(probe) => probe,
                        Err(error) => {
                            return Err(action_failure(
                                action_started,
                                profile,
                                WorkerState::Error,
                                format!("New-board Solver QMP probe failed: {error}"),
                                no_highlight_observation,
                                FailureFramePhase::PostAction,
                            ));
                        }
                    };
                    if let Err(error) = validate_probe(&reprobe) {
                        return Err(action_failure(
                            action_started,
                            profile,
                            WorkerState::Ready,
                            format!("New-board Solver click refused: {error}"),
                            None,
                            FailureFramePhase::PostAction,
                        ));
                    }

                    send_state(event_tx, WorkerState::Acting);
                    send_status(event_tx, "Click Solver".to_owned());
                    let input_started = Instant::now();
                    if let Err(error) = qmp.click_with_hold(
                        SHARED_SOLVER_CONTROL.click_point,
                        NOMINAL_FRAME_WIDTH,
                        NOMINAL_FRAME_HEIGHT,
                        SOLVER_MOUSE_HOLD,
                    ) {
                        profile.input_wall += input_started.elapsed();
                        return Err(action_failure(
                            action_started,
                            profile,
                            WorkerState::Uncertain,
                            format!("New-board Solver click is uncertain: {error}"),
                            None,
                            FailureFramePhase::PostAction,
                        ));
                    }
                    profile.input_wall += input_started.elapsed();
                    send_log(
                        event_tx,
                        format!(
                            "Automatic redeal wait complete; Solver clicked at the start of the new board with a {} ms hold. Waiting {} ms before the fresh all-row HALO capture.",
                            SOLVER_MOUSE_HOLD.as_millis(),
                            POST_GAME_STAGE_DELAY.as_millis(),
                        ),
                    );
                    send_state(event_tx, WorkerState::Verifying);
                    send_status(event_tx, "Waiting 1000 ms for Solver HALO".to_owned());
                    match cancellable_wait(POST_GAME_STAGE_DELAY, cancel_requested) {
                        Ok(waited) => profile.intentional_wait += waited,
                        Err(waited) => {
                            profile.intentional_wait += waited;
                            return Err(action_failure(
                                action_started,
                                profile,
                                WorkerState::Ready,
                                "STOP was requested after the new-board Solver click.".to_owned(),
                                None,
                                FailureFramePhase::PostAction,
                            ));
                        }
                    }
                    observation_round = observation_round.saturating_add(1);
                    continue;
                }
            }

            profile.validation_effect += verification_started.elapsed();
            if !recognised_gameplay && missing_halo_phase == 1 {
                missing_halo_phase = 2;
                send_log(
                    event_tx,
                    format!(
                        "The board has left the gameplay scene after an all-row HALO miss; allowing {} ms without screendumps before one settled transition capture.",
                        BOARD_TRANSITION_REOBSERVE_DELAY.as_millis(),
                    ),
                );
                match cancellable_wait(BOARD_TRANSITION_REOBSERVE_DELAY, cancel_requested) {
                    Ok(waited) => profile.intentional_wait += waited,
                    Err(waited) => {
                        profile.intentional_wait += waited;
                        return Err(action_failure(
                            action_started,
                            profile,
                            WorkerState::Ready,
                            "STOP was requested during the board-transition wait.".to_owned(),
                            no_highlight_observation,
                            FailureFramePhase::PostAction,
                        ));
                    }
                }
                observation_round = observation_round.saturating_add(1);
                continue;
            }

            if recognised_gameplay && missing_halo_phase == 1 {
                if cancel_requested.load(Ordering::Acquire) {
                    return Err(action_failure(
                        action_started,
                        profile,
                        WorkerState::Ready,
                        "STOP was requested before the Solver recovery click; the card action was not retried."
                            .to_owned(),
                        no_highlight_observation,
                        FailureFramePhase::PostAction,
                    ));
                }

                let reprobe_started = Instant::now();
                let reprobe = qmp.probe().map_err(|error| {
                    format!("Post-action Solver recovery QMP probe failed: {error}")
                });
                profile.validation_effect += reprobe_started.elapsed();
                let reprobe = match reprobe {
                    Ok(probe) => probe,
                    Err(error) => {
                        return Err(action_failure(
                            action_started,
                            profile,
                            WorkerState::Error,
                            error,
                            no_highlight_observation,
                            FailureFramePhase::PostAction,
                        ));
                    }
                };
                if let Err(error) = validate_probe(&reprobe) {
                    return Err(action_failure(
                        action_started,
                        profile,
                        WorkerState::Ready,
                        format!("Post-action Solver recovery click refused: {error}"),
                        no_highlight_observation,
                        FailureFramePhase::PostAction,
                    ));
                }

                send_state(event_tx, WorkerState::Acting);
                send_status(event_tx, "Click Solver".to_owned());
                let input_started = Instant::now();
                if let Err(error) = qmp.click_with_hold(
                    SHARED_SOLVER_CONTROL.click_point,
                    NOMINAL_FRAME_WIDTH,
                    NOMINAL_FRAME_HEIGHT,
                    SOLVER_MOUSE_HOLD,
                ) {
                    profile.input_wall += input_started.elapsed();
                    return Err(action_failure(
                        action_started,
                        profile,
                        WorkerState::Uncertain,
                        format!(
                            "Post-action Solver recovery click is uncertain: {error}. The card action was not retried."
                        ),
                        no_highlight_observation,
                        FailureFramePhase::PostAction,
                    ));
                }
                profile.input_wall += input_started.elapsed();
                solver_recovery_clicks = solver_recovery_clicks.saturating_add(1);
                missing_halo_phase = 2;
                send_log(
                    event_tx,
                    format!(
                        "No HALO remained after a fresh retry; Solver recovery click {solver_recovery_clicks} was sent at guest pixel ({}, {}) with a {} ms hold. Waiting {} ms before repeating the all-row HALO and card-presence checks.",
                        SHARED_SOLVER_CONTROL.click_point.x,
                        SHARED_SOLVER_CONTROL.click_point.y,
                        SOLVER_MOUSE_HOLD.as_millis(),
                        POST_GAME_STAGE_DELAY.as_millis(),
                    ),
                );
                send_state(event_tx, WorkerState::Verifying);
                send_status(event_tx, "Waiting 1000 ms for Solver HALO".to_owned());
                match cancellable_wait(POST_GAME_STAGE_DELAY, cancel_requested) {
                    Ok(waited) => profile.intentional_wait += waited,
                    Err(waited) => {
                        profile.intentional_wait += waited;
                        return Err(action_failure(
                            action_started,
                            profile,
                            WorkerState::Ready,
                            "STOP was requested while waiting after the Solver recovery click; the card action was not retried."
                                .to_owned(),
                            no_highlight_observation,
                            FailureFramePhase::PostAction,
                        ));
                    }
                }
            } else {
                if missing_halo_phase == 2 {
                    send_log(
                        event_tx,
                        if no_rows_remain {
                            format!(
                                "No HALO or exposed card was found after Solver, but completion evidence is not yet settled; repeating the transition capture in {} ms.",
                                BOARD_TRANSITION_REOBSERVE_DELAY.as_millis(),
                            )
                        } else {
                            format!(
                                "No HALO followed the Solver recovery click, but exposed cards remain within rows {}..{}; board completion is forbidden. Restarting recovery in {} ms.",
                                scan_state.row_upper(),
                                scan_state.row_lower(),
                                NO_HIGHLIGHT_REOBSERVE_DELAY.as_millis(),
                            )
                        },
                    );
                    missing_halo_phase = if no_rows_remain { 2 } else { 0 };
                } else {
                    send_log(
                        event_tx,
                        format!(
                            "WAITING: no stable calibrated target is available after post-action round {observation_round}; retrying one fresh capture in {} ms before using Solver. Press STOP to end the loop; the card action will not be retried.",
                            NO_HIGHLIGHT_REOBSERVE_DELAY.as_millis(),
                        ),
                    );
                    missing_halo_phase = 1;
                }

                let recovery_delay = if missing_halo_phase == 2 && no_rows_remain {
                    BOARD_TRANSITION_REOBSERVE_DELAY
                } else {
                    NO_HIGHLIGHT_REOBSERVE_DELAY
                };
                match cancellable_wait(recovery_delay, cancel_requested) {
                    Ok(waited) => profile.intentional_wait += waited,
                    Err(waited) => {
                        profile.intentional_wait += waited;
                        return Err(action_failure(
                            action_started,
                            profile,
                            WorkerState::Uncertain,
                            "STOP was requested during post-action HALO recovery; the card action was not retried."
                                .to_owned(),
                            no_highlight_observation,
                            FailureFramePhase::PostAction,
                        ));
                    }
                }
            }
            observation_round = observation_round.saturating_add(1);
            continue;
        }
        let changed_pixels = match before_series
            .observations
            .last()
            .zip(after_series.observations.last())
            .ok_or_else(|| "capture result was unexpectedly empty".to_owned())
            .and_then(|(before, after)| {
                materially_changed_pixels(
                    &before.frame,
                    &after.frame,
                    effect_bounds,
                    exclusion_bounds,
                    ACTION_CHANGE_CHANNEL_THRESHOLD,
                )
            }) {
            Ok(changed_pixels) => changed_pixels,
            Err(error) => {
                profile.validation_effect += verification_started.elapsed();
                let observation = after_series.observations.pop();
                return Err(action_failure(
                    action_started,
                    profile,
                    WorkerState::Uncertain,
                    format!(
                        "Post-action effect comparison failed: {error}. Input was not retried."
                    ),
                    observation,
                    FailureFramePhase::PostAction,
                ));
            }
        };
        let effect_changed = changed_pixels >= required_changed_pixels;
        if !effect_changed {
            profile.validation_effect += verification_started.elapsed();
            send_log(
                event_tx,
                format!(
                    "WAITING: post-action effect is not visible yet (changed pixels={changed_pixels}, required={required_changed_pixels}); repeating capture/detection only."
                ),
            );
            match cancellable_wait(NO_HIGHLIGHT_REOBSERVE_DELAY, cancel_requested) {
                Ok(waited) => profile.intentional_wait += waited,
                Err(waited) => {
                    profile.intentional_wait += waited;
                    let observation = after_series.observations.pop();
                    return Err(action_failure(
                        action_started,
                        profile,
                        WorkerState::Uncertain,
                        "STOP was requested while waiting for the post-action effect; guest input was not retried."
                            .to_owned(),
                        observation,
                        FailureFramePhase::PostAction,
                    ));
                }
            }
            observation_round = observation_round.saturating_add(1);
            continue;
        }

        if board_completion_verified {
            let after = match plan_step(after_predictions[0]) {
                Ok(next_plan) => next_plan.before(),
                Err(error) => {
                    profile.validation_effect += verification_started.elapsed();
                    send_log(
                        event_tx,
                        format!(
                            "WAITING: automatic redeal has not produced a stable next target ({error}); repeating capture/detection only."
                        ),
                    );
                    match cancellable_wait(NO_HIGHLIGHT_REOBSERVE_DELAY, cancel_requested) {
                        Ok(waited) => profile.intentional_wait += waited,
                        Err(waited) => {
                            profile.intentional_wait += waited;
                            let observation = after_series.observations.pop();
                            return Err(action_failure(
                                action_started,
                                profile,
                                WorkerState::Ready,
                                "STOP was requested while waiting for the redealt board target; guest input was not retried."
                                    .to_owned(),
                                observation,
                                FailureFramePhase::PostAction,
                            ));
                        }
                    }
                    observation_round = observation_round.saturating_add(1);
                    continue;
                }
            };
            reconcile_observation_series(scan_state, &after_series.observations);
            profile.validation_effect += verification_started.elapsed();
            let Some(observation) = after_series.observations.pop() else {
                return Err(action_failure(
                    action_started,
                    profile,
                    WorkerState::Uncertain,
                    "Automatic redeal verification produced no display frame.".to_owned(),
                    None,
                    FailureFramePhase::PostAction,
                ));
            };
            break (after, observation, board_changed_pixels, true, false);
        }

        let after = match verify_post_action(&plan, after_predictions[0], effect_changed) {
            Ok(after) => after,
            Err(error) => {
                profile.validation_effect += verification_started.elapsed();
                send_log(
                    event_tx,
                    format!(
                        "WAITING: post-action target is not yet verified ({error}); repeating capture/detection only. Guest input will not be retried."
                    ),
                );
                match cancellable_wait(NO_HIGHLIGHT_REOBSERVE_DELAY, cancel_requested) {
                    Ok(waited) => profile.intentional_wait += waited,
                    Err(waited) => {
                        profile.intentional_wait += waited;
                        let observation = after_series.observations.pop();
                        return Err(action_failure(
                            action_started,
                            profile,
                            WorkerState::Uncertain,
                            "STOP was requested during post-action target verification; guest input was not retried."
                                .to_owned(),
                            observation,
                            FailureFramePhase::PostAction,
                        ));
                    }
                }
                observation_round = observation_round.saturating_add(1);
                continue;
            }
        };

        reconcile_observation_series(scan_state, &after_series.observations);
        profile.validation_effect += verification_started.elapsed();
        let observation = match after_series.observations.pop() {
            Some(observation) => observation,
            None => {
                return Err(action_failure(
                    action_started,
                    profile,
                    WorkerState::Uncertain,
                    "Post-action verification produced no display frame. Guest input was not retried."
                        .to_owned(),
                    None,
                    FailureFramePhase::PostAction,
                ));
            }
        };
        break (after, observation, changed_pixels, false, false);
    };
    profile.total = action_started.elapsed();
    Ok(ActionSuccess {
        plan,
        after,
        observation,
        changed_pixels,
        observation_rounds: observation_round,
        board_completed,
        series_complete,
        completed_boards: *completed_boards,
        profile,
    })
}

fn action_failure(
    action_started: Instant,
    mut profile: ActionProfile,
    state: WorkerState,
    message: String,
    observation: Option<FrameObservation>,
    frame_phase: FailureFramePhase,
) -> ActionFailure {
    profile.total = action_started.elapsed();
    ActionFailure {
        state,
        message,
        observation,
        frame_phase,
        profile,
    }
}

fn validate_probe(probe: &crate::qmp::QmpProbe) -> Result<(), String> {
    if !probe.is_running() {
        return Err("VM is not running".to_owned());
    }
    if probe.current_absolute_pointer_name().is_none() {
        return Err("no current absolute pointer was reported".to_owned());
    }
    Ok(())
}

fn milliseconds(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1_000.0
}

fn format_operation_limit(operation_limit: usize) -> String {
    if operation_limit == 0 {
        "unbounded (until STOP)".to_owned()
    } else {
        operation_limit.to_string()
    }
}

fn format_operation_progress(operation_index: usize, operation_limit: usize) -> String {
    if operation_limit == 0 {
        format!("{operation_index}/unbounded")
    } else {
        format!("{operation_index}/{operation_limit}")
    }
}

fn log_action_profile(
    event_tx: &WorkerEventSink,
    operation_index: usize,
    operation_limit: usize,
    status: &str,
    profile: ActionProfile,
) {
    let progress = format_operation_progress(operation_index, operation_limit);
    send_log(
        event_tx,
        format!(
            "PROFILE action {progress} ({status}): input-attempted={}, captures={}, reserve={:.1} ms, screendump={:.1} ms, read={:.1} ms, decode={:.1} ms, detection={:.1} ms, validation/effect={:.1} ms, input-wall(inclusive)={:.1} ms, intentional-waits={:.1} ms, total={:.1} ms.",
            if profile.input_attempted { "yes" } else { "no" },
            profile.capture.captures,
            milliseconds(profile.capture.reserve),
            milliseconds(profile.capture.screendump),
            milliseconds(profile.capture.file_read),
            milliseconds(profile.capture.decode),
            milliseconds(profile.capture.detection),
            milliseconds(profile.validation_effect),
            milliseconds(profile.input_wall),
            milliseconds(profile.intentional_wait),
            milliseconds(profile.total),
        ),
    );
}

fn log_run_profile(
    event_tx: &WorkerEventSink,
    status: &str,
    requested: usize,
    verified: usize,
    wall: Duration,
    profile: &RunProfile,
) {
    let requested = format_operation_limit(requested);
    let mean = if profile.attempted == 0 {
        Duration::ZERO
    } else {
        Duration::from_secs_f64(profile.action_total_sum.as_secs_f64() / profile.attempted as f64)
    };
    send_log(
        event_tx,
        format!(
            "PROFILE run ({status}): requested={requested}, cycles={}, attempted={}, verified={verified}, captures={}, connect/probe={:.1} ms, reserve={:.1} ms, screendump={:.1} ms, read={:.1} ms, decode={:.1} ms, detection={:.1} ms, validation/effect={:.1} ms, input-wall(inclusive)={:.1} ms, intentional-waits={:.1} ms, action min/mean/max={:.1}/{:.1}/{:.1} ms, wall={:.1} ms.",
            profile.cycles,
            profile.attempted,
            profile.totals.capture.captures,
            milliseconds(profile.connect_probe),
            milliseconds(profile.totals.capture.reserve),
            milliseconds(profile.totals.capture.screendump),
            milliseconds(profile.totals.capture.file_read),
            milliseconds(profile.totals.capture.decode),
            milliseconds(profile.totals.capture.detection),
            milliseconds(profile.totals.validation_effect),
            milliseconds(profile.totals.input_wall),
            milliseconds(profile.totals.intentional_wait),
            milliseconds(profile.action_total_min.unwrap_or(Duration::ZERO)),
            milliseconds(mean),
            milliseconds(profile.action_total_max.unwrap_or(Duration::ZERO)),
            milliseconds(wall),
        ),
    );
}

fn reconcile_observation_series(
    scan_state: &mut TableauScanState,
    observations: &[FrameObservation],
) -> bool {
    match observations {
        [single] if !matches!(single.prediction, PredictedAction::Ambiguous { .. }) => {
            scan_state.reconcile_observation(single.observed_rows)
        }
        _ => false,
    }
}

fn settled_completion_evidence(
    prediction: PredictedAction,
    observed_rows: Option<RowMask>,
    gameplay_scene: bool,
    missing_halo_phase: u8,
    solver_recovery_clicks: usize,
) -> bool {
    // Board completion is authorised only after Solver recovery and a fresh
    // all-row observation finds neither a HALO nor any exposed tableau row.
    matches!(prediction, PredictedAction::NoHighlight)
        && observed_rows.is_none()
        && missing_halo_phase == 2
        && (solver_recovery_clicks > 0 || !gameplay_scene)
}

fn step_failed_before_input(event_tx: &WorkerEventSink, reason: String) {
    send_state(event_tx, WorkerState::Error);
    send_log(
        event_tx,
        format!("Guarded run failed closed before input: {reason}"),
    );
}

fn capture_and_analyse(
    socket_path: &Path,
    scan_state: &TableauScanState,
    event_tx: &WorkerEventSink,
) -> Result<(FrameObservation, CaptureTiming), String> {
    let (mut qmp, probe) = connect_and_probe(socket_path)?;

    send_log(event_tx, probe.summary());
    if !probe.is_running() {
        return Err("capture refused because the VM is not running".to_owned());
    }

    send_state(event_tx, WorkerState::Capturing);
    let (observation, timing) = capture_with_client(&mut qmp, socket_path, scan_state)?;

    // `qmp` and the capture guard both drop here. No connection or image file is
    // retained after the event has been constructed.
    Ok((observation, timing))
}

fn connect_and_probe(socket_path: &Path) -> Result<(QmpClient, crate::qmp::QmpProbe), String> {
    QmpClient::connect(socket_path)
        .and_then(|mut qmp| {
            let probe = qmp.probe()?;
            Ok((qmp, probe))
        })
        .map_err(|error| format!("QMP probe failed: {error}"))
}

fn ensure_input_authorised(mode: GameMode) -> Result<(), String> {
    if mode.input_authorised() {
        Ok(())
    } else {
        Err(format!(
            "{mode} is available for read-only calibration only; guest input is disabled"
        ))
    }
}

struct FrameObservation {
    frame: CapturedFrame,
    prediction: PredictedAction,
    observed_rows: Option<RowMask>,
    gameplay_scene: bool,
    game_progress: Option<GameProgress>,
}

fn capture_with_client(
    qmp: &mut QmpClient,
    socket_path: &Path,
    scan_state: &TableauScanState,
) -> Result<(FrameObservation, CaptureTiming), String> {
    let (frame, mut timing) = capture_screen(qmp, socket_path)?;
    let (observation, detection) = analyse_captured_frame(frame, scan_state)?;
    timing.detection = detection;

    Ok((observation, timing))
}

fn capture_screen(
    qmp: &mut QmpClient,
    socket_path: &Path,
) -> Result<(CapturedFrame, CaptureTiming), String> {
    let (frame, _png, timing) = capture_screen_with_png(qmp, socket_path)?;
    Ok((frame, timing))
}

fn capture_screen_with_png(
    qmp: &mut QmpClient,
    socket_path: &Path,
) -> Result<(CapturedFrame, Vec<u8>, CaptureTiming), String> {
    // Perform the one unavoidable full-display QMP screendump. Game-specific
    // regions are applied only after decode because QMP offers no crop input.
    let mut timing = CaptureTiming {
        captures: 1,
        ..CaptureTiming::default()
    };

    let reserve_started = Instant::now();
    let capture_artifact = CaptureArtifact::reserve_beside(socket_path)?;
    timing.reserve = reserve_started.elapsed();

    let screendump_started = Instant::now();
    qmp.screendump_png(capture_artifact.file_name())
        .map_err(|error| format!("QMP screendump failed: {error}"))?;
    timing.screendump = screendump_started.elapsed();

    let read_started = Instant::now();
    let png_size = fs::metadata(capture_artifact.file_name())
        .map_err(|error| format!("PNG size check failed: {error}"))?
        .len();
    if png_size == 0 || png_size > SNAPSHOT_MAX_PNG_BYTES as u64 {
        return Err(format!(
            "QMP PNG is outside the 1..={SNAPSHOT_MAX_PNG_BYTES} byte limit: {png_size}"
        ));
    }
    let mut png = Vec::with_capacity(png_size as usize);
    fs::File::open(capture_artifact.file_name())
        .map_err(|error| format!("PNG open failed: {error}"))?
        .take(SNAPSHOT_MAX_PNG_BYTES as u64 + 1)
        .read_to_end(&mut png)
        .map_err(|error| format!("PNG read failed: {error}"))?;
    if png.is_empty() || png.len() > SNAPSHOT_MAX_PNG_BYTES {
        return Err("QMP PNG size changed outside the capture limit".to_owned());
    }
    timing.file_read = read_started.elapsed();

    let decode_started = Instant::now();
    let frame = decode_png(&png).map_err(|error| format!("PNG decode failed: {error}"))?;
    timing.decode = decode_started.elapsed();

    Ok((frame, png, timing))
}

fn analyse_captured_frame(
    frame: CapturedFrame,
    scan_state: &TableauScanState,
) -> Result<(FrameObservation, Duration), String> {
    // Analyse one immutable frame; this atom has no QMP access and therefore
    // cannot send input or capture an implicit replacement frame.
    let detection_started = Instant::now();
    let profile = scan_state.mode().profile();
    if scan_state.mode().calibration_only() {
        if (frame.width, frame.height) != (profile.frame_width, profile.frame_height) {
            return Err(format!(
                "calibration frame must be exactly {}x{}, got {}x{}",
                profile.frame_width, profile.frame_height, frame.width, frame.height
            ));
        }
        if !frame.is_layout_valid() {
            return Err("calibration frame has an invalid pixel layout".to_owned());
        }
        let detection = detection_started.elapsed();
        return Ok((
            FrameObservation {
                frame,
                prediction: PredictedAction::CalibrationOnly {
                    mode: scan_state.mode(),
                },
                observed_rows: None,
                gameplay_scene: false,
                game_progress: None,
            },
            detection,
        ));
    }
    let gameplay_scene = is_gameplay_scene_for_mode(&frame, scan_state.mode())
        .map_err(|error| format!("gameplay-scene analysis failed: {error}"))?;
    let game_progress = Some(
        detect_game_progress_for_profile(&frame, profile)
            .map_err(|error| format!("Solver progress analysis failed: {error}"))?,
    );
    let (prediction, observed_rows) = if gameplay_scene {
        let analysis = analyse_frame_with_state(&frame, scan_state)
            .map_err(|error| format!("frame analysis failed: {error}"))?;
        (analysis.prediction, analysis.observed_rows)
    } else {
        (PredictedAction::NoHighlight, None)
    };
    let detection = detection_started.elapsed();

    Ok((
        FrameObservation {
            frame,
            prediction,
            observed_rows,
            gameplay_scene,
            game_progress,
        },
        detection,
    ))
}

struct CaptureSeries {
    observations: Vec<FrameObservation>,
    timing: CaptureTiming,
}

fn capture_series(
    qmp: &mut QmpClient,
    socket_path: &Path,
    scan_state: &TableauScanState,
    cancel_requested: &AtomicBool,
) -> Result<CaptureSeries, String> {
    if cancel_requested.load(Ordering::Acquire) {
        return Err("STOP was requested before the next capture".to_owned());
    }
    let (observation, timing) = capture_with_client(qmp, socket_path, scan_state)?;
    if cancel_requested.load(Ordering::Acquire) {
        return Err("STOP was requested after capture".to_owned());
    }

    Ok(CaptureSeries {
        observations: vec![observation],
        timing,
    })
}

fn materially_changed_pixels(
    before: &CapturedFrame,
    after: &CapturedFrame,
    bounds: PixelRect,
    exclusion: Option<PixelRect>,
    threshold: u8,
) -> Result<usize, String> {
    if !before.is_layout_valid() || !after.is_layout_valid() {
        return Err("effect comparison received an invalid frame layout".to_owned());
    }
    if (before.width, before.height) != (after.width, after.height) {
        return Err(format!(
            "effect comparison dimensions differ: {}x{} versus {}x{}",
            before.width, before.height, after.width, after.height
        ));
    }
    if bounds.is_empty() || bounds.right() > before.width || bounds.bottom() > before.height {
        return Err(format!(
            "effect ROI ({}, {}, {}, {}) lies outside {}x{}",
            bounds.x, bounds.y, bounds.width, bounds.height, before.width, before.height
        ));
    }

    let mut changed = 0usize;
    for y in bounds.y..bounds.bottom() {
        for x in bounds.x..bounds.right() {
            let point = crate::geometry::PixelPoint::new(x as i32, y as i32);
            if exclusion.is_some_and(|excluded| excluded.contains(point)) {
                continue;
            }
            let before_rgb = pixel_rgb(before, x, y)
                .ok_or_else(|| format!("could not read before pixel ({x}, {y})"))?;
            let after_rgb = pixel_rgb(after, x, y)
                .ok_or_else(|| format!("could not read after pixel ({x}, {y})"))?;
            if before_rgb
                .into_iter()
                .zip(after_rgb)
                .any(|(left, right)| left.abs_diff(right) >= threshold)
            {
                changed = changed.saturating_add(1);
            }
        }
    }
    Ok(changed)
}

fn send_frame(event_tx: &WorkerEventSink, observation: FrameObservation) {
    event_tx.publish_frame(observation.frame, observation.prediction, true);
}

fn send_post_action_frame(event_tx: &WorkerEventSink, observation: FrameObservation) {
    event_tx.publish_frame(observation.frame, observation.prediction, false);
}

fn format_prediction_target(prediction: PredictedAction) -> String {
    match prediction {
        PredictedAction::CalibrationOnly { mode } => format!("{mode} calibration only"),
        PredictedAction::NoHighlight => "no highlight".to_owned(),
        PredictedAction::Action(action) => match (action.target, action.operation()) {
            (ActionTarget::Bottom { label, .. }, InputOperation::PressDrawKey) => format!(
                "{label} via qcode D at gold anchor ({}, {})",
                action.anchor.x, action.anchor.y
            ),
            (ActionTarget::Bottom { label, .. }, InputOperation::Click(point)) => {
                format!("CLICK {label} at ({}, {})", point.x, point.y)
            }
            (ActionTarget::Tableau(position), InputOperation::Click(point)) => format!(
                "CLICK tableau row {}, column {} at ({}, {})",
                position.row, position.column, point.x, point.y
            ),
            (ActionTarget::Tableau(position), InputOperation::PressDrawKey) => format!(
                "INVALID tableau row {}, column {} key action",
                position.row, position.column
            ),
        },
        PredictedAction::Ambiguous { highlight_count } => {
            format!("ambiguous ({highlight_count} highlights)")
        }
    }
}

fn format_step_plan(plan: StepPlan) -> Result<String, String> {
    let input = plan.input();
    pixel_rect_to_qmp(
        input.effect_bounds(),
        NOMINAL_FRAME_WIDTH,
        NOMINAL_FRAME_HEIGHT,
    )
    .map_err(|error| format!("invalid action-effect ROI: {error}"))?;

    let action = input.action();
    Ok(match (action.target, input.operation()) {
        (ActionTarget::Bottom { label, .. }, InputOperation::PressDrawKey) => format!(
            "Stable plan: {label} via qcode D; pointer unchanged; gold anchor=({}, {}); commands=2, events=2.",
            action.anchor.x, action.anchor.y
        ),
        (target, InputOperation::Click(click_point)) => {
            let qmp = pixel_point_to_qmp(click_point, NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT)
                .map_err(|error| error.to_string())?;
            format!(
                "Stable plan: CLICK {target}; pixel=({}, {}), QMP=({}, {}); gold anchor=({}, {}); commands=3, events=4.",
                click_point.x, click_point.y, qmp.x, qmp.y, action.anchor.x, action.anchor.y
            )
        }
        (target, InputOperation::PressDrawKey) => {
            return Err(format!(
                "profile target {target} cannot use qcode D in this context"
            ));
        }
    })
}

fn execute_planned_input(
    qmp: &mut QmpClient,
    plan: StepPlan,
    cancel_requested: &AtomicBool,
) -> Result<(), String> {
    if cancel_requested.load(Ordering::Acquire) {
        return Err("STOP was requested before input".to_owned());
    }

    match plan.input().operation() {
        InputOperation::Click(point) => qmp
            .click(point, NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT)
            .map_err(|error| format!("QMP click failed: {error}"))?,
        InputOperation::PressDrawKey => qmp
            .press_draw_key()
            .map_err(|error| format!("QMP draw key failed: {error}"))?,
    }
    Ok(())
}

fn cancellable_wait(
    duration: Duration,
    cancel_requested: &AtomicBool,
) -> Result<Duration, Duration> {
    let started = Instant::now();
    loop {
        if cancel_requested.load(Ordering::Acquire) {
            return Err(started.elapsed().min(duration));
        }
        let elapsed = started.elapsed();
        if elapsed >= duration {
            return Ok(duration);
        }
        thread::sleep((duration - elapsed).min(CANCELLABLE_WAIT_SLICE));
    }
}

struct CaptureArtifact {
    file_name: PathBuf,
}

impl CaptureArtifact {
    fn reserve_beside(socket_path: &Path) -> Result<Self, String> {
        // Reserve a private, collision-resistant regular file in the QMP socket directory.
        let directory = socket_path.parent().ok_or_else(|| {
            format!(
                "QMP socket has no parent directory: {}",
                socket_path.display()
            )
        })?;

        for _ in 0..CAPTURE_NAME_ATTEMPTS {
            let sequence = CAPTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let file_name = directory.join(format!(
                ".qmp-qemu-socket-capture-{}-{sequence}.png",
                std::process::id()
            ));
            match OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&file_name)
            {
                Ok(file) => {
                    drop(file);
                    return Ok(Self { file_name });
                }
                Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
                Err(error) => {
                    return Err(format!(
                        "could not reserve capture file {}: {error}",
                        file_name.display()
                    ));
                }
            }
        }

        Err(format!(
            "could not reserve a unique capture file in {} after {CAPTURE_NAME_ATTEMPTS} attempts",
            directory.display()
        ))
    }

    fn file_name(&self) -> &Path {
        &self.file_name
    }
}

impl Drop for CaptureArtifact {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.file_name);
    }
}

fn send_log(event_tx: &WorkerEventSink, message: String) {
    // Sends a log message to the UI without failing if its receiver has closed.
    let _ = event_tx.send(WorkerEvent::Log(message));
}

fn send_status(event_tx: &WorkerEventSink, message: String) {
    // Status updates are deliberately concise; detailed diagnostics continue
    // through the private and visible logs.
    let _ = event_tx.send(WorkerEvent::Status(message));
}

fn send_board_progress(
    event_tx: &WorkerEventSink,
    scan_state: &TableauScanState,
    completed_boards: usize,
) {
    let boards_per_game = scan_state.mode().profile().boards_per_game;
    let current_board = completed_boards.saturating_add(1).min(boards_per_game);
    let _ = event_tx.send(WorkerEvent::BoardProgress {
        current_board,
        completed_boards,
        boards_per_game,
    });
}

fn format_action_status(prediction: PredictedAction) -> String {
    match prediction {
        PredictedAction::CalibrationOnly { mode } => format!("{mode} calibration — input disabled"),
        PredictedAction::Action(action) => match (action.target, action.operation()) {
            (ActionTarget::Bottom { label, .. }, InputOperation::PressDrawKey) => {
                format!("Draw card — {label}")
            }
            (ActionTarget::Bottom { label, .. }, InputOperation::Click(_)) => {
                format!("Click {label}")
            }
            (ActionTarget::Tableau(position), InputOperation::Click(_)) => {
                format!("R{}C{} click", position.row, position.column)
            }
            (ActionTarget::Tableau(position), InputOperation::PressDrawKey) => {
                format!("Invalid R{}C{} key action", position.row, position.column)
            }
        },
        PredictedAction::NoHighlight => "No Solver HALO".to_owned(),
        PredictedAction::Ambiguous { highlight_count } => {
            format!("Ambiguous — {highlight_count} HALOs")
        }
    }
}

const fn post_game_click_status(stage: PostGameStage) -> &'static str {
    match stage {
        PostGameStage::LevelUpOk => "Click Level Up OK",
        PostGameStage::NewGame => "Start New Game",
        PostGameStage::Play => "Click Play",
        PostGameStage::Solver => "Click Solver",
    }
}

fn send_state(event_tx: &WorkerEventSink, state: WorkerState) {
    // Sends a worker state update without failing if its receiver has closed.
    let _ = event_tx.send(WorkerEvent::State(state));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parameters::{LEVEL_UP_CONTROL_VARIANTS, NEW_GAME_CONTROL_VARIANTS};

    fn blank_frame(width: u32, height: u32) -> CapturedFrame {
        let stride = width as usize * 4;
        CapturedFrame {
            width,
            height,
            stride,
            format: crate::capture::PixelFormat::Rgba8,
            pixels: vec![0; stride * height as usize],
            cursor: None,
        }
    }

    fn draw_prediction(anchor: crate::geometry::PixelPoint) -> PredictedAction {
        PredictedAction::Action(
            GameMode::TriPeaks
                .profile()
                .bottom_action(0, anchor)
                .unwrap(),
        )
    }

    #[test]
    fn concise_status_labels_preserve_action_and_post_game_meaning() {
        assert_eq!(
            format_action_status(draw_prediction(crate::geometry::PixelPoint::new(842, 870))),
            "Draw card — TriPeaks stock DRAW"
        );
        assert_eq!(
            post_game_click_status(PostGameStage::LevelUpOk),
            "Click Level Up OK"
        );
        assert_eq!(
            post_game_click_status(PostGameStage::NewGame),
            "Start New Game"
        );
        assert_eq!(post_game_click_status(PostGameStage::Play), "Click Play");
        assert_eq!(
            post_game_click_status(PostGameStage::Solver),
            "Click Solver"
        );
    }

    #[test]
    fn pyramid_is_rejected_before_any_worker_command_or_qmp_connection() {
        assert_eq!(ensure_input_authorised(GameMode::TriPeaks), Ok(()));
        assert_eq!(
            ensure_input_authorised(GameMode::Pyramid),
            Err(
                "Pyramid is available for read-only calibration only; guest input is disabled"
                    .to_owned()
            )
        );
    }

    #[test]
    fn pyramid_capture_returns_only_calibration_metadata() {
        let scan_state = TableauScanState::for_mode(GameMode::Pyramid);
        let (observation, _) = analyse_captured_frame(
            blank_frame(NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT),
            &scan_state,
        )
        .unwrap();

        assert_eq!(
            observation.prediction,
            PredictedAction::CalibrationOnly {
                mode: GameMode::Pyramid,
            }
        );
        assert_eq!(observation.observed_rows, None);
        assert!(!observation.gameplay_scene);
        assert_eq!(observation.game_progress, None);
    }

    #[test]
    fn pyramid_calibration_still_requires_the_exact_frame_contract() {
        let scan_state = TableauScanState::for_mode(GameMode::Pyramid);
        let error = analyse_captured_frame(blank_frame(1_280, 720), &scan_state)
            .err()
            .unwrap();

        assert_eq!(
            error,
            "calibration frame must be exactly 1920x1080, got 1280x720"
        );
    }

    fn row_mask(row: u8) -> RowMask {
        RowMask::single_for(row, GameMode::TriPeaks.profile().tableau_rows.len() as u8).unwrap()
    }

    fn row_observation(
        prediction: PredictedAction,
        observed_rows: Option<RowMask>,
    ) -> FrameObservation {
        FrameObservation {
            frame: CapturedFrame {
                width: 1,
                height: 1,
                stride: 4,
                format: crate::capture::PixelFormat::Rgba8,
                pixels: vec![0; 4],
                cursor: None,
            },
            prediction,
            observed_rows,
            gameplay_scene: true,
            game_progress: Some(GameProgress::AnotherBoard),
        }
    }

    fn dialog_observation(variants: &[PostGameControlVariant]) -> FrameObservation {
        // Build a non-gameplay frame whose selected dialog-control probes are gold.
        let width = NOMINAL_FRAME_WIDTH as usize;
        let height = NOMINAL_FRAME_HEIGHT as usize;
        let mut frame = CapturedFrame {
            width: NOMINAL_FRAME_WIDTH,
            height: NOMINAL_FRAME_HEIGHT,
            stride: width * 4,
            format: crate::capture::PixelFormat::Rgba8,
            pixels: vec![0; width * height * 4],
            cursor: None,
        };

        for variant in variants {
            let probe = variant.probe_bounds;
            for y in probe.y..probe.bottom() {
                for x in probe.x..probe.right() {
                    let offset = y as usize * frame.stride + x as usize * 4;
                    frame.pixels[offset..offset + 4].copy_from_slice(&[220, 170, 80, 255]);
                }
            }
        }

        FrameObservation {
            frame,
            prediction: PredictedAction::NoHighlight,
            observed_rows: None,
            gameplay_scene: false,
            game_progress: Some(GameProgress::GameComplete),
        }
    }

    #[test]
    fn latest_frame_slot_replaces_stale_full_resolution_previews() {
        let slot = LatestFrameSlot::default();
        let first = row_observation(PredictedAction::NoHighlight, None);
        let second_prediction = draw_prediction(crate::geometry::PixelPoint::new(7, 9));
        let mut second = row_observation(second_prediction, None);
        second.frame.pixels.fill(2);

        slot.publish(
            first.frame,
            first.prediction,
            false,
            Some((PathBuf::from("/tmp/old-qmp.sock"), GameMode::TriPeaks)),
        );
        slot.publish(
            second.frame,
            second.prediction,
            true,
            Some((PathBuf::from("/tmp/new-qmp.sock"), GameMode::Pyramid)),
        );

        let latest = slot.take().expect("latest preview");
        assert_eq!(latest.frame.pixels, vec![2; 4]);
        assert_eq!(latest.prediction, second_prediction);
        assert!(latest.log_prediction);
        assert_eq!(latest.coalesced_frames, 1);
        assert_eq!(
            latest.context,
            Some((PathBuf::from("/tmp/new-qmp.sock"), GameMode::Pyramid))
        );
        assert!(slot.take().is_none());
    }

    #[test]
    fn board_progress_reports_committed_completion_without_waiting_for_redeal() {
        let (tx, rx) = mpsc::channel();
        let sink = WorkerEventSink {
            events: tx,
            latest_frame: LatestFrameSlot::default(),
            capture_context: Mutex::new(None),
        };
        let scan_state = TableauScanState::initial();
        send_board_progress(&sink, &scan_state, 1);
        match rx.try_recv().expect("board-completion event") {
            WorkerEvent::BoardProgress {
                current_board,
                completed_boards,
                boards_per_game,
            } => {
                assert_eq!(
                    (current_board, completed_boards, boards_per_game),
                    (2, 1, 3)
                );
            }
            _ => panic!("expected a board-completion event"),
        }
    }

    #[test]
    fn draw_plan_reports_key_only_input_without_a_pointer_target() {
        let prediction = draw_prediction(crate::geometry::PixelPoint::new(842, 870));
        let plan = plan_step(prediction).expect("key-only draw plan");

        assert_eq!(
            format_step_plan(plan),
            Ok("Stable plan: TriPeaks stock DRAW via qcode D; pointer unchanged; gold anchor=(842, 870); commands=2, events=2.".to_owned())
        );
    }

    #[test]
    fn capture_artifact_is_private_beside_socket_and_removed_on_drop() {
        let test_directory = std::env::temp_dir().join(format!(
            "solitaire-worker-test-{}-{}",
            std::process::id(),
            CAPTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&test_directory).expect("create worker test directory");
        let socket_path = test_directory.join("qmp.sock");

        let artifact =
            CaptureArtifact::reserve_beside(&socket_path).expect("reserve capture artifact");
        let capture_file = artifact.file_name().to_owned();
        assert_eq!(capture_file.parent(), Some(test_directory.as_path()));
        assert!(capture_file.exists());

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(&capture_file)
                .expect("capture metadata")
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(mode, 0o600);
        }

        drop(artifact);
        assert!(!capture_file.exists());
        fs::remove_dir(&test_directory).expect("remove worker test directory");
    }

    #[test]
    fn effect_comparison_applies_threshold_and_cursor_exclusion() {
        let before = CapturedFrame {
            width: 3,
            height: 2,
            stride: 12,
            format: crate::capture::PixelFormat::Rgba8,
            pixels: vec![0; 24],
            cursor: None,
        };
        let mut after = before.clone();
        after.pixels[0] = 19;
        after.pixels[4] = 20;
        after.pixels[8] = 100;
        after.pixels[12 + 2] = 21;

        assert_eq!(
            materially_changed_pixels(
                &before,
                &after,
                PixelRect::new(0, 0, 3, 2),
                Some(PixelRect::new(2, 0, 1, 1)),
                20,
            ),
            Ok(2)
        );
    }

    #[test]
    fn row_three_halo_forbids_completion_when_row_one_is_empty() {
        let row_three_halo = PredictedAction::Action(
            GameMode::TriPeaks
                .profile()
                .tableau_action(9, crate::geometry::PixelPoint::new(296, 630))
                .unwrap(),
        );

        assert!(!settled_completion_evidence(
            row_three_halo,
            Some(row_mask(3)),
            true,
            2,
            1,
        ));
        assert!(!settled_completion_evidence(
            PredictedAction::NoHighlight,
            Some(row_mask(3)),
            true,
            2,
            1,
        ));
        assert!(settled_completion_evidence(
            PredictedAction::NoHighlight,
            None,
            true,
            2,
            1,
        ));
        assert!(!settled_completion_evidence(
            PredictedAction::NoHighlight,
            None,
            true,
            1,
            0,
        ));
        assert!(settled_completion_evidence(
            PredictedAction::NoHighlight,
            None,
            false,
            2,
            0,
        ));
    }

    #[test]
    fn level_up_dialog_overrides_only_a_verified_redeal_state() {
        let observation = dialog_observation(&[LEVEL_UP_CONTROL_VARIANTS[1]]);

        assert_eq!(
            level_up_overrides_board_progress(false, &observation),
            Ok(false)
        );
        assert_eq!(
            level_up_overrides_board_progress(true, &observation),
            Ok(true)
        );
    }

    #[test]
    fn dialog_probes_cannot_authorise_a_different_post_game_stage() {
        let dialog_targets = &POST_GAME_TARGETS[..3];

        for (expected_index, expected_target) in dialog_targets.iter().copied().enumerate() {
            for expected_variant in expected_target.control_variants.iter().copied() {
                let observation = dialog_observation(&[expected_variant]);

                for (actual_index, actual_target) in dialog_targets.iter().copied().enumerate() {
                    assert_eq!(
                        resolve_post_game_target(&observation, actual_target),
                        Ok((expected_index == actual_index).then_some(expected_variant))
                    );
                }
            }
        }
    }

    #[test]
    fn level_up_layout_ambiguity_fails_closed() {
        let observation = dialog_observation(&LEVEL_UP_CONTROL_VARIANTS);
        let error = resolve_post_game_target(&observation, POST_GAME_TARGETS[0])
            .expect_err("both Level Up layouts must be ambiguous");

        assert!(error.contains("legacy-low"));
        assert!(error.contains("raised"));
        assert!(error.contains("no guest input was sent"));
    }

    #[test]
    fn raised_level_up_resolves_its_own_click_point() {
        let raised = LEVEL_UP_CONTROL_VARIANTS[1];
        let observation = dialog_observation(&[raised]);

        assert_eq!(
            resolve_post_game_target(&observation, POST_GAME_TARGETS[0]),
            Ok(Some(raised))
        );
        assert_eq!(
            raised.click_point,
            crate::geometry::PixelPoint::new(960, 770)
        );
    }

    #[test]
    fn new_game_can_be_verified_when_level_up_is_missing() {
        let new_game = dialog_observation(&[NEW_GAME_CONTROL_VARIANTS[0]]);
        assert_eq!(resolve_post_game_target(&new_game, POST_GAME_TARGETS[0]), Ok(None));
        assert_eq!(new_game_visible_while_awaiting_level_up(&new_game), Ok(true));

        let level_up = dialog_observation(&[LEVEL_UP_CONTROL_VARIANTS[1]]);
        assert_eq!(new_game_visible_while_awaiting_level_up(&level_up), Ok(false));
    }

    #[test]
    fn stale_dialog_is_reported_before_the_expected_later_stage() {
        let raised = LEVEL_UP_CONTROL_VARIANTS[1];
        let level_up = dialog_observation(&[raised]);
        assert_eq!(
            earlier_visible_post_game_target(&level_up, 1),
            Ok(Some((0, POST_GAME_TARGETS[0], raised)))
        );

        let new_game_variant = NEW_GAME_CONTROL_VARIANTS[0];
        let new_game = dialog_observation(&[new_game_variant]);
        assert_eq!(earlier_visible_post_game_target(&new_game, 1), Ok(None));
        assert_eq!(
            earlier_visible_post_game_target(&new_game, 2),
            Ok(Some((1, POST_GAME_TARGETS[1], new_game_variant)))
        );
    }

    #[test]
    fn post_game_retry_budgets_stop_at_the_configured_limits() {
        assert_eq!(
            require_post_game_observation_retry_budget(
                POST_GAME_MAX_OBSERVATION_ROUNDS - 1,
                "test"
            ),
            Ok(())
        );
        assert!(
            require_post_game_observation_retry_budget(POST_GAME_MAX_OBSERVATION_ROUNDS, "test")
                .is_err()
        );

        assert_eq!(
            require_post_game_click_retry_budget(POST_GAME_MAX_CLICK_ATTEMPTS - 1, "test"),
            Ok(())
        );
        assert!(
            require_post_game_click_retry_budget(POST_GAME_MAX_CLICK_ATTEMPTS, "test").is_err()
        );
    }

    #[test]
    fn score_skip_budget_counts_the_initial_click_and_stops_before_a_fourth() {
        for completed_attempts in 0..SCORE_SKIP_MAX_CLICK_ATTEMPTS {
            assert_eq!(require_score_skip_click_budget(completed_attempts), Ok(()));
        }

        let error = require_score_skip_click_budget(SCORE_SKIP_MAX_CLICK_ATTEMPTS)
            .expect_err("a fourth score-skip click must be refused");
        assert!(error.contains("confirmed-delivery score-skip centre clicks"));
        assert!(error.contains("no further guest input was sent"));
    }

    #[test]
    fn score_skip_retry_is_limited_to_non_gameplay_frames_awaiting_level_up() {
        assert!(score_skip_retry_is_authorised(
            PostGameStage::LevelUpOk,
            false
        ));
        assert!(!score_skip_retry_is_authorised(
            PostGameStage::LevelUpOk,
            true
        ));
        assert!(!score_skip_retry_is_authorised(
            PostGameStage::NewGame,
            false
        ));
    }

    #[test]
    fn effect_comparison_rejects_mismatched_dimensions_and_bounds() {
        let before = CapturedFrame {
            width: 2,
            height: 1,
            stride: 8,
            format: crate::capture::PixelFormat::Rgba8,
            pixels: vec![0; 8],
            cursor: None,
        };
        let mut after = before.clone();
        after.width = 1;
        after.stride = 4;

        assert!(
            materially_changed_pixels(&before, &after, PixelRect::new(0, 0, 1, 1), None, 20,)
                .is_err()
        );
        assert!(
            materially_changed_pixels(&before, &before, PixelRect::new(1, 0, 2, 1), None, 20,)
                .is_err()
        );
    }

    #[test]
    fn cursor_sized_change_does_not_meet_the_material_effect_floor() {
        let width = 64;
        let height = 64;
        let before = CapturedFrame {
            width,
            height,
            stride: width as usize * 4,
            format: crate::capture::PixelFormat::Rgba8,
            pixels: vec![0; width as usize * height as usize * 4],
            cursor: None,
        };
        let mut after = before.clone();
        for y in 0..48usize {
            for x in 0..32usize {
                let offset = y * after.stride + x * 4;
                after.pixels[offset..offset + 3].fill(255);
            }
        }

        let changed = materially_changed_pixels(
            &before,
            &after,
            PixelRect::new(0, 0, width, height),
            None,
            ACTION_CHANGE_CHANNEL_THRESHOLD,
        )
        .unwrap();
        assert_eq!(changed, 32 * 48);
        assert!(changed < crate::parameters::MINIMUM_TABLEAU_CHANGED_PIXELS);
    }

    #[test]
    fn run_profile_distinguishes_validation_cycles_from_input_attempts() {
        let mut profile = RunProfile::new(Duration::from_millis(7));
        profile.record(ActionProfile {
            total: Duration::from_millis(10),
            ..ActionProfile::default()
        });
        profile.record(ActionProfile {
            total: Duration::from_millis(20),
            input_attempted: true,
            ..ActionProfile::default()
        });

        assert_eq!(profile.cycles, 2);
        assert_eq!(profile.attempted, 1);
        assert_eq!(profile.action_total_sum, Duration::from_millis(20));
        assert_eq!(profile.action_total_min, Some(Duration::from_millis(20)));
        assert_eq!(profile.action_total_max, Some(Duration::from_millis(20)));
    }

    #[test]
    fn cancellable_wait_observes_an_existing_stop_request() {
        let cancelled = AtomicBool::new(true);
        let result = cancellable_wait(Duration::from_secs(1), &cancelled);

        assert!(result.is_err());
        assert!(result.unwrap_err() < Duration::from_secs(1));
    }

    #[test]
    fn single_capture_can_commit_non_authoritative_row_evidence() {
        let mut state = TableauScanState::initial();
        let row_three = row_mask(3);
        let observations = [row_observation(
            PredictedAction::NoHighlight,
            Some(row_three),
        )];

        assert!(reconcile_observation_series(&mut state, &observations));
        assert_eq!(state.active_rows(), row_three);
    }

    #[test]
    fn ambiguous_single_capture_does_not_change_row_state() {
        let initial = TableauScanState::initial();
        let row_three = row_mask(3);

        let mut ambiguous_state = initial;
        let ambiguous = [row_observation(
            PredictedAction::Ambiguous { highlight_count: 2 },
            Some(row_three),
        )];
        assert!(!reconcile_observation_series(
            &mut ambiguous_state,
            &ambiguous
        ));
        assert_eq!(ambiguous_state, initial);
    }

    #[test]
    fn socket_or_mode_change_resets_the_row_hint_but_same_context_preserves_it() {
        let first_socket = PathBuf::from("/tmp/solitaire-a.sock");
        let second_socket = PathBuf::from("/tmp/solitaire-b.sock");
        let row_three = row_mask(3);
        let mut state = TableauScanState::from_active_rows(row_three).unwrap();
        let mut remembered_context = None;
        let mut completed_boards = 2;

        reset_scan_state_for_context(
            &first_socket,
            GameMode::TriPeaks,
            &mut remembered_context,
            &mut state,
            &mut completed_boards,
        );
        assert_eq!(state, TableauScanState::initial());
        assert_eq!(completed_boards, 0);
        assert_eq!(
            remembered_context
                .as_ref()
                .map(|(path, mode)| (path.as_path(), *mode)),
            Some((first_socket.as_path(), GameMode::TriPeaks))
        );

        state = TableauScanState::from_active_rows(row_three).unwrap();
        completed_boards = 1;
        reset_scan_state_for_context(
            &first_socket,
            GameMode::TriPeaks,
            &mut remembered_context,
            &mut state,
            &mut completed_boards,
        );
        assert_eq!(state.active_rows(), row_three);
        assert_eq!(completed_boards, 1);

        reset_scan_state_for_context(
            &second_socket,
            GameMode::TriPeaks,
            &mut remembered_context,
            &mut state,
            &mut completed_boards,
        );
        assert_eq!(state, TableauScanState::initial());
        assert_eq!(completed_boards, 0);
        assert_eq!(
            remembered_context
                .as_ref()
                .map(|(path, mode)| (path.as_path(), *mode)),
            Some((second_socket.as_path(), GameMode::TriPeaks))
        );
    }

    #[test]
    fn manual_progress_reset_clears_board_and_row_tracking() {
        let row_two = row_mask(2);
        let mut state = TableauScanState::from_active_rows(row_two).unwrap();
        let mut completed_boards = 2usize;

        reset_progress_tracking(&mut state, &mut completed_boards);

        assert_eq!(state, TableauScanState::initial());
        assert_eq!(completed_boards, 0);
    }

    #[test]
    fn new_actionable_run_resets_only_a_completed_three_board_series() {
        let row_three = row_mask(3);
        let mut state = TableauScanState::from_active_rows(row_three).unwrap();
        let boards_per_game = state.mode().profile().boards_per_game;
        let mut completed_boards = boards_per_game - 1;

        assert!(!reset_completed_series_for_new_run(
            &mut state,
            &mut completed_boards,
        ));
        assert_eq!(state.active_rows(), row_three);
        assert_eq!(completed_boards, boards_per_game - 1);

        completed_boards = boards_per_game;
        assert!(reset_completed_series_for_new_run(
            &mut state,
            &mut completed_boards,
        ));
        assert_eq!(state, TableauScanState::initial());
        assert_eq!(completed_boards, 0);
    }
}
