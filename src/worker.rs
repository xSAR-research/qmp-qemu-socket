//! Background QMP orchestration, guarded input, capture and UI event delivery.
//!
//! Captures use temporary PNG files that are read, decoded and removed during
//! normal cleanup. Preview frames are coalesced; input is authorised from fresh
//! observations. Game-specific Pyramid results and shared post-game screens
//! are handled by focused child modules.

mod freecell_execution;
mod klondike_execution;
mod post_game;
mod pyramid_execution;

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
    detector::{detect_game_progress_for_profile, pixel_rgb},
    game::{ActionTarget, GameMode, GameProgress, InputOperation},
    geometry::{PixelRect, pixel_point_to_qmp, pixel_rect_to_qmp},
    pyramid,
    qmp::QmpClient,
    stepper::{StepPlan, plan_step, verify_post_action},
    tracker::{
        PredictedAction, RowMask, TableauScanState, analyse_frame_with_state,
        is_gameplay_scene_for_mode,
    },
};

use crate::parameters::StepRunSettings;
use crate::parameters::{
    ACTION_CHANGE_CHANNEL_THRESHOLD, BOARD_TRANSITION_REOBSERVE_DELAY, CANCELLABLE_WAIT_SLICE,
    CAPTURE_NAME_ATTEMPTS, CAPTURE_SEQUENCE, MULTI_STEP_INPUT_ENABLED,
    NO_HIGHLIGHT_REOBSERVE_DELAY, NOMINAL_FRAME_HEIGHT, NOMINAL_FRAME_WIDTH,
    POST_GAME_MAX_OBSERVATION_ROUNDS, POST_GAME_STAGE_DELAY, SHARED_SOLVER_CONTROL,
    SNAPSHOT_MAX_PNG_BYTES, SOLVER_MOUSE_HOLD, STEP_ONCE_ACTIONS, STEP_ONCE_INPUT_ENABLED,
};


/// Maximum ordinary Pyramid planning or result observations before a diagnostic stop.
const PYRAMID_OBSERVATION_LIMIT: usize = 6;


/// Delayed input-free TriPeaks observations before and after one Solver refresh.
const TRIPEAKS_HALO_REOBSERVATION_LIMIT: usize = 3;


/// Next recovery operation after a fresh TriPeaks observation has no HALO.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TriPeaksHaloRecoveryStep {
    /// Wait for a new input-free observation within the current bounded budget.
    Observe {
        /// One-based delayed observation number.
        delayed_round: usize,
        /// Whether the one Solver operation has already been reserved or sent.
        after_solver: bool,
    },
    /// Reserve the context's one Solver refresh on recognised gameplay only.
    RefreshSolver,
    /// Stop without another input after the current observation budget is spent.
    Stop,
}


/// Bounded observation and one-refresh authority for an unresolved TriPeaks action.
#[derive(Default)]
struct TriPeaksHaloRecovery {
    /// Input-free captures scheduled since the action or its Solver operation.
    delayed_observations: usize,
    /// A reserved refresh is consumed even if input delivery becomes uncertain.
    solver_reserved: bool,
}


impl TriPeaksHaloRecovery {


    /// Start a new-board observation context after its authorised Solver input.
    fn after_solver() -> Self {
        Self {
            delayed_observations: 0,
            solver_reserved: true,
        }
    }


    /// Schedule observations first, then at most one positively gated refresh.
    fn next_missing_halo(&mut self, gameplay_scene: bool) -> TriPeaksHaloRecoveryStep {


        if self.delayed_observations < TRIPEAKS_HALO_REOBSERVATION_LIMIT {
            self.delayed_observations += 1;
            return TriPeaksHaloRecoveryStep::Observe {
                delayed_round: self.delayed_observations,
                after_solver: self.solver_reserved,
            };
        }


        if !self.solver_reserved && gameplay_scene {
            self.solver_reserved = true;
            self.delayed_observations = 0;
            TriPeaksHaloRecoveryStep::RefreshSolver
        } else {
            TriPeaksHaloRecoveryStep::Stop
        }
    }
}


/// UI-visible lifecycle state of the background QMP worker.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WorkerState {
    /// No active capture context or retained QMP connection.
    #[default]
    Detached,
    /// Opening and probing the selected QMP socket.
    Connecting,
    /// Waiting for a fresh QMP screendump.
    Capturing,
    /// Checking scene and action preconditions.
    Validating,
    /// Delivering one authorised guest input sequence.
    Acting,
    /// Observing an action or screen transition.
    Verifying,
    /// Idle and available for a new request.
    Ready,
    /// Stopped because observations or input outcome need inspection.
    Uncertain,
    /// A connection, capture or pre-input validation failed.
    Error,
}


impl WorkerState {


    /// Return the concise UI label for this worker state.
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


    /// Report whether a capture, validation, input or result-verification operation is active.
    pub const fn is_busy(self) -> bool {
        matches!(
            self,
            Self::Connecting | Self::Capturing | Self::Validating | Self::Acting | Self::Verifying
        )
    }
}


/// Requests serialised by the background worker command loop.
enum WorkerCommand {
    /// Capture and analyse a read-only advisory preview.
    CaptureFrame {
        /// Selected QMP Unix socket.
        socket_path: PathBuf,
        /// Game profile used for capture and detection.
        mode: GameMode,
    },
    /// Capture original PNG bytes and decoded pixels for manual saving.
    PrepareSnapshot {
        /// Selected QMP Unix socket.
        socket_path: PathBuf,
        /// Game profile used for capture and detection.
        mode: GameMode,
        /// UI request identifier used to reject stale responses.
        request_id: u64,
    },
    /// Run guarded input using an approved preview and fixed settings.
    ExecuteSteps {
        /// Selected QMP Unix socket.
        socket_path: PathBuf,
        /// Game profile used for capture and detection.
        mode: GameMode,
        /// Preview prediction that the first fresh planning frame must reproduce.
        approved_prediction: PredictedAction,
        /// Immutable operation limit and animation timings for this run.
        settings: Box<StepRunSettings>,
    },
    /// Clear local board and scan history without touching the guest.
    ResetProgress,
    /// Stop the current run and discard its capture context.
    Disconnect,
    /// Exit the worker command loop.
    Shutdown,
}


/// Small ordered notifications sent from the worker to the UI.
pub enum WorkerEvent {
    /// Detailed diagnostic text for visible and session logs.
    Log(String),
    /// Concise current-operation text for the status area.
    Status(String),
    /// One fresh original PNG and its decoded preview are ready.
    SnapshotPrepared {
        /// Manual snapshot request identifier.
        request_id: u64,
        /// Socket/game identity under which the snapshot was captured.
        context: (PathBuf, GameMode),
        /// Decoded pixels from this exact capture.
        frame: CapturedFrame,
        /// Original PNG bytes matching the preview frame.
        png: Vec<u8>,
        /// Advisory prediction from the accompanying capture.
        prediction: PredictedAction,
    },
    /// The identified manual snapshot could not be prepared.
    SnapshotPreparationFailed {
        /// Manual snapshot request identifier.
        request_id: u64,
        /// Capture/preparation failure description.
        error: String,
    },
    /// Committed board counters for the current game profile.
    BoardProgress {
        /// Current board number clamped to the profile limit.
        current_board: usize,
        /// Number of positively completed boards in this session.
        completed_boards: usize,
        /// Number of boards defined by the active game profile.
        boards_per_game: usize,
    },
    /// Current lifecycle state of the worker.
    State(WorkerState),
    /// One action was effect-verified or accepted from a fresh repeated halo pair.
    ActionCompleted {
        /// One-based action index within the requested run.
        operation_index: usize,
        /// Requested action count; zero denotes an unbounded run.
        operation_limit: usize,
        /// Planning prediction for the delivered input.
        before: PredictedAction,
        /// Accepted prediction from a fresh result frame.
        after: PredictedAction,
        /// Number of QMP commands in the planned gameplay input.
        input_commands: usize,
        /// Number of guest input events in the planned sequence.
        input_events: usize,
        /// Diagnostic count of materially changed effect pixels.
        changed_pixels: usize,
        /// True when mode-owned fresh evidence authorised continuation without proving the prior effect.
        continued_from_halo: bool,
    },
    /// The shared post-game controller verified a restarted game.
    GameCompleted,
    /// Klondike independently verified one game win and the next actionable Solver board.
    KlondikeGameCompleted,
    /// Free Cell recognised a one-board win; the worker log distinguishes finite stop from restart.
    FreeCellGameCompleted,
    /// A bounded run reached its requested action count.
    RunCompleted {
        /// Advisory prediction from the accompanying capture.
        prediction: PredictedAction,
        /// Requested count for the completed bounded run.
        requested_operations: usize,
        /// Actions accepted through positive effect verification.
        verified_operations: usize,
        /// Actions continued through the selected mode's guarded fresh-evidence policy.
        halo_operations: usize,
    },
}


/// Latest advisory or diagnostic preview transferred through the bounded frame slot.
pub struct LatestWorkerFrame {
    /// Full decoded capture for display.
    pub frame: CapturedFrame,
    /// Advisory target; diagnostic frames carry no action authority.
    pub prediction: PredictedAction,
    /// Whether the UI should log the displayed prediction.
    pub log_prediction: bool,
    /// Whether the frame is retained solely to inspect a failed result.
    pub diagnostic: bool,
    /// Older pending previews replaced since the last UI consumption.
    pub coalesced_frames: usize,
    /// Socket/game identity associated with the frame.
    pub context: Option<(PathBuf, GameMode)>,
}


/// Single pending preview retained until the UI consumes or replaces it.
struct PendingWorkerFrame {
    /// Decoded image retained for the preview.
    frame: CapturedFrame,
    /// Advisory target attached to this frame.
    prediction: PredictedAction,
    /// Request prediction logging when displayed.
    log_prediction: bool,
    /// Mark unverified result pixels for inspection only.
    diagnostic: bool,
    /// Socket/game identity used for the capture.
    context: Option<(PathBuf, GameMode)>,
}


/// Mutable contents protected by the latest-preview mutex.
#[derive(Default)]
struct LatestFrameState {
    /// At most one full-resolution frame awaiting consumption.
    pending: Option<PendingWorkerFrame>,
    /// Number of pending frames replaced since the last take.
    coalesced_frames: usize,
}


/// Shared bounded hand-off slot that prevents preview images accumulating behind a sleeping UI.
#[derive(Clone, Default)]
struct LatestFrameSlot {
    /// Mutex-protected single preview and replacement count.
    state: Arc<Mutex<LatestFrameState>>,
}


impl LatestFrameSlot {


    /// Replace the pending advisory preview and count any frame displaced before the UI consumes it.
    ///
    /// The slot retains at most one full-resolution frame; poisoned locks preserve the
    /// latest recoverable state rather than terminating the worker.
    fn publish(
        &self,
        frame: CapturedFrame,
        prediction: PredictedAction,
        log_prediction: bool,
        diagnostic: bool,
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
                diagnostic,
                context,
            })
            .is_some()
        {
            state.coalesced_frames = state.coalesced_frames.saturating_add(1);
        }
    }


    /// Remove the latest preview and reset its coalesced-frame count; return `None` when empty.
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
            diagnostic: pending.diagnostic,
            coalesced_frames,
            context: pending.context,
        })
    }
}


/// Worker-side delivery of small events and coalesced full-resolution preview frames.
struct WorkerEventSink {
    /// Ordered event channel to the UI.
    events: Sender<WorkerEvent>,
    /// Single-frame slot for high-volume image updates.
    latest_frame: LatestFrameSlot,
    /// Current socket/game identity attached to preview publications.
    capture_context: Mutex<Option<(PathBuf, GameMode)>>,
}


impl WorkerEventSink {


    /// Queue a small worker event; retain failed delivery in a boxed channel error.
    fn send(&self, event: WorkerEvent) -> Result<(), Box<mpsc::SendError<WorkerEvent>>> {
        self.events.send(event).map_err(Box::new)
    }


    /// Publish an advisory frame with its prediction and the current socket/game context.
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
            .publish(frame, prediction, log_prediction, false, context);
    }


    /// Publish the last observed pixels without granting action authority to their prediction.
    fn publish_diagnostic_frame(&self, frame: CapturedFrame) {
        let context = self
            .capture_context
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        // Preserve the exact last observed pixels, without approving a target.
        self.latest_frame
            .publish(frame, PredictedAction::NoHighlight, false, true, context);
    }


    /// Set the socket and game identity attached to subsequent preview frames.
    fn set_capture_context(&self, context: Option<(PathBuf, GameMode)>) {
        *self
            .capture_context
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = context;
    }
}


/// UI-side command, event and cancellation interface to the background QMP thread.
pub struct WorkerHandle {
    /// Queue for serialised worker requests.
    command_tx: Sender<WorkerCommand>,
    /// Receiver for ordered small worker events.
    event_rx: Receiver<WorkerEvent>,
    /// Shared slot for the newest preview.
    latest_frame: LatestFrameSlot,
    /// Thread handle consumed during shutdown.
    join: Option<JoinHandle<()>>,
    /// Atomic STOP flag checked around captures, waits and input.
    cancel_requested: Arc<AtomicBool>,
}


impl WorkerHandle {


    /// Start the QMP worker and create its command, event and bounded-preview channels.
    ///
    /// # Panics
    /// Panics if the operating system cannot create the worker thread.
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


    /// Queue one read-only capture for `socket_path` and `mode`.
    ///
    /// Returns an error if the worker command channel has closed.
    pub fn capture_frame(&self, socket_path: PathBuf, mode: GameMode) -> Result<(), String> {
        // Requests one read-only QMP screendump and frame analysis.
        self.command_tx
            .send(WorkerCommand::CaptureFrame { socket_path, mode })
            .map_err(|error| format!("QMP worker is unavailable: {error}"))
    }


    /// Queue a fresh capture whose original PNG and decoded preview share `request_id`.
    ///
    /// The UI uses the request ID and socket/game context to reject stale responses.
    /// Returns an error if the worker command channel has closed.
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


    /// Queue a guarded run constrained by the approved preview and immutable run settings.
    ///
    /// Clears an earlier STOP request only after mode and compile-time input checks.
    /// Returns an error for disabled input or an unavailable worker; this method
    /// does not itself send guest input.
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


        if mode == GameMode::Klondike {
            klondike_execution::validate_operation_limit(settings.operation_limit())?;
        }


        if mode == GameMode::FreeCell {
            freecell_execution::validate_operation_limit(settings.operation_limit())?;
        }


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
                settings: Box::new(settings),
            })
            .map_err(|error| format!("QMP worker is unavailable: {error}"))
    }


    /// Request cancellation immediately and queue a return to the detached state.
    ///
    /// Returns an error if the worker channel has closed; QEMU remains running.
    pub fn disconnect(&self) -> Result<(), String> {
        // Requests that the worker return to its detached state.
        self.cancel_requested.store(true, Ordering::Release);
        self.command_tx
            .send(WorkerCommand::Disconnect)
            .map_err(|error| format!("QMP worker is unavailable: {error}"))
    }


    /// Queue a local reset of the board counter and scan history without guest input.
    ///
    /// Returns an error if the worker command channel has closed.
    pub fn reset_progress(&self) -> Result<(), String> {
        // Resets the board counter and bounded row scan without touching QMP.
        self.command_tx
            .send(WorkerCommand::ResetProgress)
            .map_err(|error| format!("QMP worker is unavailable: {error}"))
    }


    /// Iterate over currently queued events without blocking the UI thread.
    pub fn try_events(&self) -> impl Iterator<Item = WorkerEvent> + '_ {
        // Returns currently queued worker events without blocking the caller.
        self.event_rx.try_iter()
    }


    /// Take at most one pending preview, including its displaced-frame count.
    pub fn take_latest_frame(&self) -> Option<LatestWorkerFrame> {
        // Takes at most one preview frame, together with the number of stale
        // previews replaced since the UI last consumed one.
        self.latest_frame.take()
    }
}


impl Drop for WorkerHandle {


    /// Request STOP, send shutdown and join the QMP worker before releasing the handle.
    fn drop(&mut self) {
        // Shuts down and joins the worker thread when its handle is released.
        self.cancel_requested.store(true, Ordering::Release);
        let _ = self.command_tx.send(WorkerCommand::Shutdown);


        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}


/// Dispatch queued commands while retaining scan history only for the active socket and game.
///
/// QMP connections belong to individual capture or execution transactions.
/// Shutdown or a closed command channel ends the thread.
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
                run_capture(
                    socket_path,
                    &mut scan_state,
                    &mut completed_boards,
                    &event_tx,
                );
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
                    *settings,
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


/// Reset board and scan history only when the socket path or game mode changes.
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


/// Reset the current game profile to its initial scan state and zero completed boards.
fn reset_progress_tracking(scan_state: &mut TableauScanState, completed_boards: &mut usize) {
    // Returns progress tracking to a new game and the profile's initial row scan.
    *scan_state = TableauScanState::for_mode(scan_state.mode());
    *completed_boards = 0;
}


/// Capture one fresh frame without guest input and publish its advisory analysis.
///
/// A reappeared Pyramid card may clear stale per-board history. Capture errors
/// are reported through worker events rather than returned to the dispatcher.
fn run_capture(
    socket_path: PathBuf,
    scan_state: &mut TableauScanState,
    completed_boards: &mut usize,
    event_tx: &WorkerEventSink,
) {
    // Captures and analyses exactly one guest frame without issuing guest input.
    send_state(event_tx, WorkerState::Connecting);
    send_status(event_tx, "Screengrab".to_owned());
    send_log(
        event_tx,
        format!("Connecting to QMP socket: {}", socket_path.display()),
    );

    let result = capture_and_analyse(&socket_path, scan_state, event_tx).and_then(
        |(mut observation, timing)| {


            if scan_state.mode() == GameMode::Pyramid
                && observation.gameplay_scene
                && matches!(observation.prediction, PredictedAction::NoHighlight)
                && let Some(fresh) = pyramid::reappeared_consumed_card(
                    &observation.frame,
                    &scan_state.pyramid,
                )
                .map_err(|error| format!("Pyramid fresh-capture recovery failed: {error}"))?
            {
                *scan_state = TableauScanState::for_mode(GameMode::Pyramid);
                *completed_boards = 0;
                observation.prediction = fresh.prediction;
                observation.observed_rows = fresh.observed_rows;
                send_board_progress(event_tx, scan_state, *completed_boards);
                send_log(
                    event_tx,
                    format!(
                        "RECOVERED: fresh Pyramid capture found {} on a card positively removed in the previous board record; clearing stale per-board state and rescanning this same frame. No guest input or extra screendump was sent; board number reset to unknown.",
                        format_prediction_target(observation.prediction),
                    ),
                );
            }
            Ok((observation, timing))
        },
    );


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


/// Capture and publish one original PNG with its decoded frame for manual preview.
///
/// Enforces the profile frame contract; classification remains advisory so
/// dialogs can be saved. Reports success or failure with the supplied request ID.
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
            send_log(
                event_tx,
                format!("Read-only snapshot capture failed: {error}"),
            );
        }
    }
}


/// Run the requested number of guarded actions, or continue until STOP when unbounded.
///
/// Reuses accepted result frames for the next plan and keeps effect-verified
/// actions separate from fresh-halo continuations. Failures publish the last
/// observation and stop; completed games enter the shared restart controller.
fn run_execute_steps(
    socket_path: PathBuf,
    approved_prediction: PredictedAction,
    settings: StepRunSettings,
    scan_state: &mut TableauScanState,
    completed_boards: &mut usize,
    event_tx: &WorkerEventSink,
    cancel_requested: &AtomicBool,
) {


    if scan_state.mode() == GameMode::FreeCell {
        freecell_execution::run_freecell_steps(
            &socket_path, approved_prediction, settings, scan_state, event_tx, cancel_requested,
        );
        return;
    }


    if scan_state.mode() == GameMode::Klondike {
        klondike_execution::run_klondike_steps(
            &socket_path,
            approved_prediction,
            settings,
            scan_state,
            event_tx,
            cancel_requested,
        );
        return;
    }

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


    if scan_state.mode() == GameMode::Pyramid {
        let delays = settings.animation_delays();
        send_log(
            event_tx,
            format!(
                "Pyramid timing snapshot: Move/Recycle={} ms, Card/Left/Right={} ms, repeat observation={} ms, board redeal={} ms. Params edits apply to the next run.",
                delays.pyramid_move.as_millis(),
                delays.pyramid_card.as_millis(),
                delays.pyramid_reobserve.as_millis(),
                delays.board_redeal.as_millis(),
            ),
        );
    }


    if scan_state.mode() == GameMode::TriPeaks {
        send_log(
            event_tx,
            format!(
                "TriPeaks timing snapshot: Draw={} ms, tableau={} ms, missing-HALO reobserve={} ms. Missing HALOs receive {TRIPEAKS_HALO_REOBSERVATION_LIMIT} delayed input-free observations before at most one Solver refresh, then {TRIPEAKS_HALO_REOBSERVATION_LIMIT} further delayed observations; the gameplay action is never retried.",
                settings.animation_delays().draw.as_millis(),
                settings.animation_delays().tableau.as_millis(),
                settings.animation_delays().tripeaks_reobserve.as_millis(),
            ),
        );
    }

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
            planning_frame,
            settings,
            scan_state,
            completed_boards,
            GuardedActionContext {
                socket_path: &socket_path,
                event_tx,
                cancel_requested,
            },
        ) {
            Ok(success) => {


                let acceptance = if success.continued_from_halo {
                    run_profile.halo_operations = run_profile.halo_operations.saturating_add(1);
                    "continued from fresh halo"
                } else {
                    verified_operations = verified_operations.saturating_add(1);
                    "verified"
                };
                run_profile.record(success.profile);
                log_action_profile(
                    event_tx,
                    operation_index,
                    operation_limit,
                    acceptance,
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
                    continued_from_halo: success.continued_from_halo,
                });
                send_log(
                    event_tx,
                    format!(
                        "Action {progress} {acceptance} after {} post-action observation round(s): {} -> {}; changed effect pixels={}, effect proven={}, QMP commands={input_commands}, input events={input_events}, guest-input retries=0.",
                        success.observation_rounds,
                        format_prediction_target(success.plan.before()),
                        format_prediction_target(success.after),
                        success.changed_pixels,
                        !success.continued_from_halo,
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


                    match post_game::run_post_game_restart(
                        &mut qmp,
                        &socket_path,
                        (scan_state.mode() == GameMode::Pyramid).then_some(&next_observation),
                        scan_state,
                        completed_boards,
                        event_tx,
                        cancel_requested,
                    ) {
                        Ok(outcome) => {


                            let (observation, game_restarted) = match outcome {
                                post_game::PostGameOutcome::Restarted(observation) => {
                                    (observation, true)
                                }
                                post_game::PostGameOutcome::RecoveredGameplay(observation) => {
                                    (observation, false)
                                }
                            };
                            next_prediction = observation.prediction;
                            next_observation = observation;
                            send_board_progress(event_tx, scan_state, *completed_boards);


                            if game_restarted {
                                let _ = event_tx.send(WorkerEvent::GameCompleted);
                            }
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
                        halo_operations: run_profile.halo_operations,
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
                planning_frame = PlanningFrame::AcceptedResult(next_observation);
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


/// Reset a fully completed game before a new run; report whether a reset occurred.
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


/// Publish an advisory copy of a post-action frame without repeating its prediction log.
fn publish_post_action_observation(event_tx: &WorkerEventSink, observation: &FrameObservation) {
    event_tx.publish_frame(observation.frame.clone(), observation.prediction, false);
}


/// Wait in cancellable slices and return waited time, or the caller-provided STOP message.
fn wait_or_stop(
    duration: Duration,
    cancel_requested: &AtomicBool,
    stopped_message: &str,
) -> Result<Duration, String> {
    cancellable_wait(duration, cancel_requested).map_err(|_| stopped_message.to_owned())
}


/// Per-stage wall times and capture count for QMP PNG acquisition and analysis.
#[derive(Clone, Copy, Debug, Default)]
struct CaptureTiming {
    /// Number of fresh QMP screendumps represented.
    captures: usize,
    /// Time reserving private temporary capture paths.
    reserve: Duration,
    /// Time waiting for QMP to write PNG output.
    screendump: Duration,
    /// Time reading the bounded original PNG bytes.
    file_read: Duration,
    /// Time decoding PNG into RGBA pixels.
    decode: Duration,
    /// Time classifying scene, progress and targets.
    detection: Duration,
}


impl CaptureTiming {


    /// Accumulate capture counts and per-stage durations from another capture measurement.
    fn add_assign(&mut self, other: Self) {
        self.captures = self.captures.saturating_add(other.captures);
        self.reserve += other.reserve;
        self.screendump += other.screendump;
        self.file_read += other.file_read;
        self.decode += other.decode;
        self.detection += other.detection;
    }
}


/// Timing and input-attempt evidence for one guarded action cycle.
#[derive(Clone, Copy, Debug, Default)]
struct ActionProfile {
    /// Accumulated fresh-capture stage times.
    capture: CaptureTiming,
    /// Time evaluating planning, effects and transitions.
    validation_effect: Duration,
    /// Wall time spent delivering input, including deliberate holds.
    input_wall: Duration,
    /// Configured settling and cancellation-aware waits.
    intentional_wait: Duration,
    /// Complete action-cycle wall time.
    total: Duration,
    /// Whether this cycle reached guest input delivery.
    input_attempted: bool,
}


/// Aggregate timing and acceptance statistics for a guarded run.
struct RunProfile {
    /// Initial QMP connection and probe wall time.
    connect_probe: Duration,
    /// Validation/action cycles recorded, including those stopping before input.
    cycles: usize,
    /// Cycles that attempted guest input.
    attempted: usize,
    /// Accepted fresh-halo continuations without proven previous effects.
    halo_operations: usize,
    /// Sum of component timings across all cycles.
    totals: ActionProfile,
    /// Total wall time for cycles that attempted input.
    action_total_sum: Duration,
    /// Shortest attempted-input cycle, absent before any attempt.
    action_total_min: Option<Duration>,
    /// Longest attempted-input cycle, absent before any attempt.
    action_total_max: Option<Duration>,
}


impl RunProfile {


    /// Create empty run accounting with the already-measured QMP connection/probe duration.
    const fn new(connect_probe: Duration) -> Self {
        Self {
            connect_probe,
            cycles: 0,
            attempted: 0,
            halo_operations: 0,
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


    /// Accumulate one validation cycle while measuring min/mean/max only for attempted input.
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


/// Accepted result of one guarded action, preserving effect and halo acceptance separately.
struct ActionSuccess {
    /// Immutable plan whose input was delivered.
    plan: StepPlan,
    /// Accepted next-action prediction.
    after: PredictedAction,
    /// Fresh result frame reused by the next planning cycle.
    observation: FrameObservation,
    /// Diagnostic effect-region change count.
    changed_pixels: usize,
    /// True when continuation was authorised without a proven prior effect.
    continued_from_halo: bool,
    /// Number of post-action captures required for acceptance.
    observation_rounds: usize,
    /// Whether this action positively completed a board.
    board_completed: bool,
    /// Whether the game is ready for terminal-screen handling.
    series_complete: bool,
    /// Committed board count after this result.
    completed_boards: usize,
    /// Measured stages and total action duration.
    profile: ActionProfile,
}


/// Whether a retained failure frame preceded or followed guest input.
#[derive(Clone, Copy)]
enum FailureFramePhase {
    /// Advisory planning frame captured before the attempted action.
    PreAction,
    /// Unverified result frame that must not authorise another action.
    PostAction,
}


/// Stopped action with a diagnostic frame and measured work completed so far.
struct ActionFailure {
    /// UI lifecycle state selected for the failure.
    state: WorkerState,
    /// Detailed stop reason.
    message: String,
    /// Latest available frame, if capture progressed far enough.
    observation: Option<FrameObservation>,
    /// Distinguishes advisory planning from unverified result pixels.
    frame_phase: FailureFramePhase,
    /// Timings including work before the stop.
    profile: ActionProfile,
}


/// Source of planning authority for the next guarded action.
enum PlanningFrame {
    /// The advisory UI prediction is only a constraint. One fresh capture must
    /// reproduce it before the first input in a run.
    InitialConstraint(PredictedAction),
    /// A fresh post-action frame accepted through effect verification or the
    /// settled Pyramid halo rule. It is the next action's immutable plan.
    AcceptedResult(FrameObservation),
}


/// Immutable connection, publication and cancellation references for one guarded action.
#[derive(Clone, Copy)]
struct GuardedActionContext<'a> {
    /// Selected QMP socket used for the action and every result capture.
    socket_path: &'a Path,
    /// Ordered event sink and coalesced latest-frame mailbox.
    event_tx: &'a WorkerEventSink,
    /// Cooperative STOP flag checked before input and during waits.
    cancel_requested: &'a AtomicBool,
}


/// Plan and deliver at most one gameplay input sequence, then observe its result.
///
/// A first planning capture must match the approved preview; later plans reuse
/// accepted fresh results. Pyramid result handling is delegated to its verifier.
/// Returns an accepted action with timings, or a failure retaining the latest
/// frame and whether it preceded input. Uncertain input is never replayed here.
fn execute_guarded_action(
    qmp: &mut QmpClient,
    planning_frame: PlanningFrame,
    settings: StepRunSettings,
    scan_state: &mut TableauScanState,
    completed_boards: &mut usize,
    context: GuardedActionContext<'_>,
) -> Result<ActionSuccess, Box<ActionFailure>> {
    let GuardedActionContext {
        socket_path,
        event_tx,
        cancel_requested,
    } = context;
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


                        if scan_state.mode() == GameMode::Pyramid
                            && pre_action_round >= PYRAMID_OBSERVATION_LIMIT
                        {
                            return Err(action_failure(
                                action_started,
                                profile,
                                WorkerState::Uncertain,
                                format!(
                                    "Pyramid initial planning stopped after {pre_action_round} observations: {error}. Activate Solver and capture a fresh preview; input sent: 0."
                                ),
                                candidate_series.observations.pop(),
                                FailureFramePhase::PreAction,
                            ));
                        }


                        let retry_delay = if scan_state.mode() == GameMode::Pyramid {
                            settings.animation_delays().pyramid_reobserve
                        } else {
                            NO_HIGHLIGHT_REOBSERVE_DELAY
                        };
                        send_log(
                            event_tx,
                            format!(
                                "WAITING: initial planning observation round {pre_action_round} has no stable target ({error}); waiting {} ms, then repeating capture/detection only. Press STOP to end the loop; input sent: 0.",
                                retry_delay.as_millis(),
                            ),
                        );


                        match cancellable_wait(retry_delay, cancel_requested) {
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
        PlanningFrame::AcceptedResult(observation) => {
            send_status(event_tx, "Planning from accepted result".to_owned());
            send_log(
                event_tx,
                "Reusing the prior action's fresh accepted result frame as the next planning frame; redundant pre-action screendump=0."
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


    if plan.input().action().target.mode() != scan_state.mode() {
        return Err(action_failure(
            action_started,
            profile,
            WorkerState::Uncertain,
            "The planned target belongs to a different game mode; input sent: 0.".to_owned(),
            before_series.observations.pop(),
            FailureFramePhase::PreAction,
        ));
    }
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


    if let ActionTarget::Pyramid(kind) = plan.input().action().target {
        send_log(
            event_tx,
            format!(
                "Pyramid {kind:?} input delivered once; waiting {} ms before the first result capture.",
                animation_settle_delay.as_millis(),
            ),
        );
    }


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


    if let ActionTarget::Pyramid(kind) = plan.input().action().target {
        return pyramid_execution::verify_pyramid_action(
            qmp,
            pyramid_execution::PyramidActionVerification {
                before_series,
                plan,
                kind,
                action_started,
                profile,
            },
            settings,
            scan_state,
            completed_boards,
            context,
        );
    }
    let mut observation_round = 1usize;
    let mut board_completion_verified = false;
    let mut board_changed_pixels = 0usize;
    let mut missing_halo_phase = 0u8;
    let mut solver_recovery_clicks = 0usize;
    let mut halo_recovery = TriPeaksHaloRecovery::default();


    let (after, observation, changed_pixels, board_completed, series_complete) = loop {
        send_log(
            event_tx,
            format!(
                "Post-action observation round {observation_round}: capturing one fresh frame; missing-HALO recovery is bounded and never retries the gameplay action."
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


                    match post_game::level_up_overrides_board_progress(
                        board_completion_verified,
                        observation,
                    ) {
                        Ok(visible) => visible,
                        Err(error)


                            if post_game::is_level_up_ambiguity(&error)
                                && observation_round < POST_GAME_MAX_OBSERVATION_ROUNDS =>
                        {
                            profile.validation_effect += verification_started.elapsed();
                            send_log(
                                event_tx,
                                format!(
                                    "WAITING: {error}; repeating the completed-board Level Up capture in {} ms without input.",
                                    BOARD_TRANSITION_REOBSERVE_DELAY.as_millis(),
                                ),
                            );


                            match cancellable_wait(
                                BOARD_TRANSITION_REOBSERVE_DELAY,
                                cancel_requested,
                            ) {
                                Ok(waited) => profile.intentional_wait += waited,
                                Err(waited) => {
                                    profile.intentional_wait += waited;
                                    return Err(action_failure(
                                        action_started,
                                        profile,
                                        WorkerState::Ready,
                                        "STOP was requested during ambiguous Level Up re-observation."
                                            .to_owned(),
                                        no_highlight_observation,
                                        FailureFramePhase::PostAction,
                                    ));
                                }
                            }
                            observation_round = observation_round.saturating_add(1);
                            continue;
                        }
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


                    if let Err(error) = click_shared_solver(qmp, cancel_requested) {
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
                    halo_recovery = TriPeaksHaloRecovery::after_solver();
                    missing_halo_phase = 2;
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


            match halo_recovery.next_missing_halo(recognised_gameplay) {
                TriPeaksHaloRecoveryStep::Observe {
                    delayed_round,
                    after_solver,
                } => {
                    missing_halo_phase = if after_solver
                        || (!recognised_gameplay && delayed_round > 1)
                    {
                        2
                    } else {
                        1
                    };
                    let recovery_delay = if !recognised_gameplay && delayed_round == 2 {
                        BOARD_TRANSITION_REOBSERVE_DELAY
                    } else {
                        settings.animation_delays().tripeaks_reobserve
                    };
                    send_log(
                        event_tx,
                        format!(
                            "TriPeaks missing-HALO re-observation {delayed_round}/{TRIPEAKS_HALO_REOBSERVATION_LIMIT} {}: scene={recognised_gameplay}, exposed rows={}, current scan rows={}..{}; waiting {} ms before a fresh input-free capture. No Draw, card or further Solver input is authorised by this wait.",
                            if after_solver { "after Solver" } else { "before Solver" },
                            !no_rows_remain,
                            scan_state.row_upper(),
                            scan_state.row_lower(),
                            recovery_delay.as_millis(),
                        ),
                    );


                    match cancellable_wait(recovery_delay, cancel_requested) {
                        Ok(waited) => profile.intentional_wait += waited,
                        Err(waited) => {
                            profile.intentional_wait += waited;
                            return Err(action_failure(
                                action_started,
                                profile,
                                WorkerState::Uncertain,
                                "STOP was requested during TriPeaks input-free HALO recovery; no gameplay input was retried."
                                    .to_owned(),
                                no_highlight_observation,
                                FailureFramePhase::PostAction,
                            ));
                        }
                    }
                }
                TriPeaksHaloRecoveryStep::RefreshSolver => {


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


                    if let Err(error) = click_shared_solver(qmp, cancel_requested) {
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
                            "No HALO remained after {TRIPEAKS_HALO_REOBSERVATION_LIMIT} delayed input-free observations; the context's one Solver recovery click {solver_recovery_clicks} was sent at guest pixel ({}, {}) with a {} ms hold. Waiting {} ms before a fresh all-row HALO and card-presence capture; at most {TRIPEAKS_HALO_REOBSERVATION_LIMIT} further delayed observations may follow.",
                            SHARED_SOLVER_CONTROL.click_point.x,
                            SHARED_SOLVER_CONTROL.click_point.y,
                            SOLVER_MOUSE_HOLD.as_millis(),
                            settings.animation_delays().tripeaks_reobserve.as_millis(),
                        ),
                    );
                    send_state(event_tx, WorkerState::Verifying);
                    send_status(
                        event_tx,
                        format!(
                            "Waiting {} ms for Solver HALO",
                            settings.animation_delays().tripeaks_reobserve.as_millis(),
                        ),
                    );


                    match cancellable_wait(
                        settings.animation_delays().tripeaks_reobserve,
                        cancel_requested,
                    ) {
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
                }
                TriPeaksHaloRecoveryStep::Stop => {
                    return Err(action_failure(
                        action_started,
                        profile,
                        WorkerState::Uncertain,
                        format!(
                            "TriPeaks missing-HALO recovery stopped after {TRIPEAKS_HALO_REOBSERVATION_LIMIT} delayed observations {}; recognised gameplay={recognised_gameplay}, exposed rows={}, current scan rows={}..{}, Solver recovery clicks={solver_recovery_clicks}. No gameplay or Solver input was retried; the latest frame is retained for inspection.",
                            if halo_recovery.solver_reserved { "after the one Solver refresh" } else { "on an unsupported scene" },
                            !no_rows_remain,
                            scan_state.row_upper(),
                            scan_state.row_lower(),
                        ),
                        no_highlight_observation,
                        FailureFramePhase::PostAction,
                    ));
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
        continued_from_halo: false,
        board_completed,
        series_complete,
        completed_boards: *completed_boards,
        profile,
    })
}


/// Send one held Solver click unless STOP is already requested.
///
/// Returns input wall time or an uncertain-delivery error without automatic retry.
fn click_shared_solver(
    qmp: &mut QmpClient,
    cancel_requested: &AtomicBool,
) -> Result<Duration, String> {


    if cancel_requested.load(Ordering::Acquire) {
        return Err(
            "STOP was requested before the Solver click; no Solver input was sent".to_owned(),
        );
    }
    let started = Instant::now();
    qmp.click_with_hold(
        SHARED_SOLVER_CONTROL.click_point,
        NOMINAL_FRAME_WIDTH,
        NOMINAL_FRAME_HEIGHT,
        SOLVER_MOUSE_HOLD,
    ).map_err(|error| format!(
        "Solver input result is uncertain: {error}. No automatic retry followed the uncertain delivery."
    ))?;
    Ok(started.elapsed())
}


/// Finish action timing and retain the stop state, reason and optional last observation.
fn action_failure(
    action_started: Instant,
    mut profile: ActionProfile,
    state: WorkerState,
    message: String,
    observation: Option<FrameObservation>,
    frame_phase: FailureFramePhase,
) -> Box<ActionFailure> {
    profile.total = action_started.elapsed();
    Box::new(ActionFailure {
        state,
        message,
        observation,
        frame_phase,
        profile,
    })
}


/// Require a running VM and a current absolute pointer; return the first failed precondition.
fn validate_probe(probe: &crate::qmp::QmpProbe) -> Result<(), String> {


    if !probe.is_running() {
        return Err("VM is not running".to_owned());
    }


    if probe.current_absolute_pointer_name().is_none() {
        return Err("no current absolute pointer was reported".to_owned());
    }
    Ok(())
}


/// Convert a duration to fractional milliseconds for diagnostic output.
fn milliseconds(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1_000.0
}


/// Render zero as an unbounded run and a positive value as its operation limit.
fn format_operation_limit(operation_limit: usize) -> String {


    if operation_limit == 0 {
        "unbounded (until STOP)".to_owned()
    } else {
        operation_limit.to_string()
    }
}


/// Render the current one-based operation index against a bounded or unbounded limit.
fn format_operation_progress(operation_index: usize, operation_limit: usize) -> String {


    if operation_limit == 0 {
        format!("{operation_index}/unbounded")
    } else {
        format!("{operation_index}/{operation_limit}")
    }
}


/// Emit one action timing record, including whether guest input was attempted.
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


/// Emit aggregate timings and separate effect-verified actions from fresh-halo continuations.
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
            "PROFILE run ({status}): requested={requested}, cycles={}, attempted={}, verified={verified}, continued-from-halo={}, captures={}, connect/probe={:.1} ms, reserve={:.1} ms, screendump={:.1} ms, read={:.1} ms, decode={:.1} ms, detection={:.1} ms, validation/effect={:.1} ms, input-wall(inclusive)={:.1} ms, intentional-waits={:.1} ms, action min/mean/max={:.1}/{:.1}/{:.1} ms, wall={:.1} ms.",
            profile.cycles,
            profile.attempted,
            profile.halo_operations,
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


/// Commit row evidence only from one non-ambiguous observation; report whether state changed.
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


/// Require no halo or exposed row after the completion-observation phase.
///
/// A gameplay frame additionally requires an attempted Solver recovery; absent
/// rows alone cannot declare completion while an actionable halo remains.
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


/// Report a guarded-run error known to precede all guest input.
fn step_failed_before_input(event_tx: &WorkerEventSink, reason: String) {
    send_state(event_tx, WorkerState::Error);
    send_log(
        event_tx,
        format!("Guarded run failed closed before input: {reason}"),
    );
}


/// Open a short-lived QMP connection, require a running VM, and analyse one capture.
///
/// Returns the observation and capture timings; connection, capture or detector
/// failures return a labelled error without guest input.
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


/// Connect to the selected Unix socket and return its live QMP client and capability/state probe.
///
/// Connection, handshake and probe failures return a labelled diagnostic error.
fn connect_and_probe(socket_path: &Path) -> Result<(QmpClient, crate::qmp::QmpProbe), String> {
    QmpClient::connect(socket_path)
        .and_then(|mut qmp| {
            let probe = qmp.probe()?;
            Ok((qmp, probe))
        })
        .map_err(|error| format!("QMP probe failed: {error}"))
}


/// Reject calibration-only game modes before any guest input can be requested.
fn ensure_input_authorised(mode: GameMode) -> Result<(), String> {


    if mode.input_authorised() {
        Ok(())
    } else {
        Err(format!(
            "{mode} is available for read-only calibration only; guest input is disabled"
        ))
    }
}


/// Decoded capture and classifications derived from those exact immutable pixels.
struct FrameObservation {
    /// Captured guest pixels.
    frame: CapturedFrame,
    /// Eligible next target or a non-actionable classification.
    prediction: PredictedAction,
    /// Positive row evidence, when available.
    observed_rows: Option<RowMask>,
    /// Whether the calibrated gameplay-scene probes matched.
    gameplay_scene: bool,
    /// Progress-bar classification; never completion authority by itself.
    game_progress: Option<GameProgress>,
}


/// Capture once using an existing QMP connection, then analyse the decoded frame.
///
/// Returns observation and stage timings, propagating file, decode and detector errors.
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


/// Capture one decoded frame and timings, discarding the original PNG bytes from memory.
fn capture_screen(
    qmp: &mut QmpClient,
    socket_path: &Path,
) -> Result<(CapturedFrame, CaptureTiming), String> {
    let (frame, _png, timing) = capture_screen_with_png(qmp, socket_path)?;
    Ok((frame, timing))
}


/// Capture one full-display QMP PNG through a private temporary file and decode its pixels.
///
/// Returns the decoded frame, original bytes and timings for manual preview/save.
/// The file guard removes the temporary PNG on normal cleanup. Empty/oversized
/// files, capture failures, I/O failures and decode errors are rejected.
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


/// Classify one immutable frame under the supplied game and scan state without QMP access.
///
/// Calibration frames receive no input authority. Other modes require scene,
/// progress and halo analysis; invalid dimensions/layout or detector failures
/// return an error. The returned duration measures detection only.
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


    let game_progress = if matches!(scan_state.mode(), GameMode::Klondike | GameMode::FreeCell) {
        None
    } else {
        Some(
            detect_game_progress_for_profile(&frame, profile)
                .map_err(|error| format!("Solver progress analysis failed: {error}"))?,
        )
    };


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


/// Observations and timings passed to shared action validation.
struct CaptureSeries {
    /// Current one-frame capture sequence.
    observations: Vec<FrameObservation>,
    /// Acquisition and analysis timing for the sequence.
    timing: CaptureTiming,
}


/// Capture one observation with STOP checks before and after the operation.
///
/// The series shape supports shared validation logic; it currently contains
/// exactly one frame. Cancellation and capture errors are returned to the caller.
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


/// Count pixels with a channel delta at least `threshold` inside the effect bounds.
///
/// Pixels inside the optional cursor exclusion do not count. Invalid layouts,
/// mismatched dimensions, out-of-frame bounds and unreadable pixels return errors.
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


/// Publish a pre-action advisory frame and request its prediction be logged.
fn send_frame(event_tx: &WorkerEventSink, observation: FrameObservation) {
    event_tx.publish_frame(observation.frame, observation.prediction, true);
}


/// Publish an unverified result frame for inspection without approving another action.
fn send_post_action_frame(event_tx: &WorkerEventSink, observation: FrameObservation) {
    event_tx.publish_diagnostic_frame(observation.frame);
}


/// Describe a predicted target and its input method for detailed diagnostics.
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
            (ActionTarget::Klondike(kind), InputOperation::Click(point)) => {
                format!("CLICK Klondike {kind:?} at ({}, {})", point.x, point.y)
            }
            (ActionTarget::FreeCell(kind), InputOperation::Click(point)) => {
                format!("CLICK Free Cell {kind} at ({}, {})", point.x, point.y)
            }
            (ActionTarget::FreeCell(kind), InputOperation::PressDrawKey) => {
                format!("INVALID Free Cell {kind} key action")
            }
            (ActionTarget::Klondike(kind), InputOperation::PressDrawKey) => {
                format!("Klondike {kind:?} via qcode D; pointer unchanged")
            }
            (ActionTarget::Pyramid(kind), InputOperation::Click(point)) => {
                format!("CLICK Pyramid {kind:?} at ({}, {})", point.x, point.y)
            }
            (ActionTarget::Pyramid(pyramid::PyramidTargetKind::Move), InputOperation::PressDrawKey) => {
                "Pyramid MOVE/Recycle via qcode D; pointer unchanged".to_owned()
            }
            (ActionTarget::Pyramid(kind), InputOperation::PressDrawKey) => {
                format!("INVALID Pyramid {kind:?} key action")
            }
        },
        PredictedAction::Ambiguous { highlight_count } => {
            format!("ambiguous ({highlight_count} highlights)")
        }
    }
}


/// Validate effect/click geometry and describe the exact planned QMP input sequence.
///
/// Returns an error for invalid bounds, coordinates or an unsupported key target.
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
        (ActionTarget::Klondike(crate::klondike::KlondikeTarget::Draw), InputOperation::PressDrawKey) => {
            "Stable plan: Klondike Draw via qcode D; pointer unchanged; commands=2, events=2.".to_owned()
        }
        (ActionTarget::Pyramid(pyramid::PyramidTargetKind::Move), InputOperation::PressDrawKey) => {
            "Stable plan: Pyramid MOVE/Recycle via qcode D; pointer unchanged; commands=2, events=2.".to_owned()
        }
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


/// Send one planned click or draw-key sequence after a final STOP check.
///
/// Returns QMP errors as uncertain action failures; it does not retry delivery.
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


/// Wait up to `duration` while checking STOP between short sleep slices.
///
/// Returns the requested duration on completion or elapsed bounded time on cancellation.
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


/// Guard owning a private temporary QMP PNG path until normal cleanup.
struct CaptureArtifact {
    /// Exclusively reserved path beside the QMP socket.
    file_name: PathBuf,
}


impl CaptureArtifact {


    /// Reserve a private regular PNG file beside the QMP socket using exclusive creation.
    ///
    /// Retries name collisions within a fixed budget; returns errors for a missing
    /// parent, exhausted names or other file-system failures.
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


    /// Borrow the reserved PNG path passed to QMP and subsequent read operations.
    fn file_name(&self) -> &Path {
        &self.file_name
    }
}


impl Drop for CaptureArtifact {


    /// Attempt normal-cleanup removal of the reserved temporary PNG; ignore removal errors.
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.file_name);
    }
}


/// Send a detailed diagnostic message; ignore a closed UI event receiver.
fn send_log(event_tx: &WorkerEventSink, message: String) {
    // Sends a log message to the UI without failing if its receiver has closed.
    let _ = event_tx.send(WorkerEvent::Log(message));
}


/// Send concise status text; detailed evidence remains in the log stream.
fn send_status(event_tx: &WorkerEventSink, message: String) {
    // Status updates are deliberately concise; detailed diagnostics continue
    // through the private and visible logs.
    let _ = event_tx.send(WorkerEvent::Status(message));
}


/// Publish completed boards and a current-board number clamped to the game profile.
fn send_board_progress(
    event_tx: &WorkerEventSink,
    scan_state: &TableauScanState,
    completed_boards: usize,
) {


    if matches!(scan_state.mode(), GameMode::Klondike | GameMode::FreeCell) {
        return;
    }
    let boards_per_game = scan_state.mode().profile().boards_per_game;
    let current_board = completed_boards.saturating_add(1).min(boards_per_game);
    let _ = event_tx.send(WorkerEvent::BoardProgress {
        current_board,
        completed_boards,
        boards_per_game,
    });
}


/// Render concise action status while preserving target and input-method meaning.
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
            (ActionTarget::Klondike(kind), InputOperation::Click(_)) => {
                format!("Click Klondike {kind:?}")
            }
            (ActionTarget::FreeCell(kind), InputOperation::Click(_)) => {
                format!("Click Free Cell {kind}")
            }
            (ActionTarget::FreeCell(kind), InputOperation::PressDrawKey) => {
                format!("Invalid Free Cell {kind} key action")
            }
            (ActionTarget::Klondike(kind), InputOperation::PressDrawKey) => {
                format!("Draw card — Klondike {kind:?}")
            }
            (ActionTarget::Pyramid(kind), InputOperation::Click(_)) => {
                format!("Click Pyramid {kind:?}")
            }
            (ActionTarget::Pyramid(pyramid::PyramidTargetKind::Move), InputOperation::PressDrawKey) => {
                "MOVE/Recycle — Pyramid via D".to_owned()
            }
            (ActionTarget::Pyramid(kind), InputOperation::PressDrawKey) => {
                format!("Invalid Pyramid {kind:?} key action")
            }
        },
        PredictedAction::NoHighlight => "No Solver HALO".to_owned(),
        PredictedAction::Ambiguous { highlight_count } => {
            format!("Ambiguous — {highlight_count} HALOs")
        }
    }
}


/// Publish worker state without treating a closed UI receiver as a worker failure.
fn send_state(event_tx: &WorkerEventSink, state: WorkerState) {
    // Sends a worker state update without failing if its receiver has closed.
    let _ = event_tx.send(WorkerEvent::State(state));
}

#[cfg(test)]
mod tests;
