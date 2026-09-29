//! Desktop controls, read-only previews and diagnostic presentation for the QMP worker.
//!
//! Worker events update the UI on its own thread. A preview authorises execution
//! only while its mode, socket and dimensions remain current; snapshot bytes and
//! diagnostic frames are retained separately from that authority.

use std::{path::PathBuf, time::Instant};

use eframe::egui::{
    self, Align, Align2, Color32, FontId, Layout, Pos2, Rect, Sense, Stroke, TextBuffer,
};
use egui_winit::clipboard::Clipboard;
use raw_window_handle::HasDisplayHandle;

use crate::{
    capture::{CapturedFrame, PixelFormat},
    game::{ActionTarget, GameMode, InputOperation},
    geometry::{PixelPoint, PixelRect},
    pyramid::{self, PYRAMID_TARGETS},
    session_log::SessionLog,
    snapshot::SnapshotArtifact,
    tracker::PredictedAction,
    worker::{WorkerEvent, WorkerHandle, WorkerState},
};

use crate::parameters::{
    ACTION_CHANGE_CHANNEL_THRESHOLD, ACTION_CURSOR_EXCLUSION_HALF_SIZE,
    BOARD_REDEAL_SETTLE_DELAY_MS, CHALLENGE_COMPLETE_CONTINUE_CONTROL, DEFAULT_MULTI_STEP_ACTIONS,
    DRAW_ANIMATION_SETTLE_DELAY_MS, GOLD_CHANNEL_TOLERANCE, GOLD_RGB_CANDIDATES, KEY_HOLD,
    LEVEL_UP_APPEAR_DELAY, MAX_LOG_LINES, MAXIMUM_ANIMATION_SETTLE_DELAY_MS,
    MIN_PREVIEW_VIEWPORT_HEIGHT_POINTS, MINIMUM_ANIMATION_SETTLE_DELAY_MS,
    MINIMUM_DRAW_CHANGED_PIXELS, MINIMUM_TABLEAU_CHANGED_PIXELS, MOUSE_HOLD,
    MULTI_STEP_INPUT_ENABLED, NO_HIGHLIGHT_REOBSERVE_DELAY, NOMINAL_FRAME_HEIGHT,
    NOMINAL_FRAME_WIDTH, OUTPUT_PANEL_HEIGHT, POINTER_SETTLE_DELAY, POST_GAME_MAX_CLICK_ATTEMPTS,
    POST_GAME_MAX_OBSERVATION_ROUNDS, POST_GAME_STAGE_DELAY, POST_GAME_TARGETS,
    PREVIEW_FOOTER_RESERVE_POINTS, PREVIEW_SCROLLBAR_ALLOWANCE_POINTS,
    PYRAMID_CARD_SETTLE_DELAY_MS, PYRAMID_MOVE_SETTLE_DELAY_MS, PYRAMID_REOBSERVE_DELAY_MS,
    RELEASE_LABEL, SCORE_SKIP_MAX_CLICK_ATTEMPTS, SHARED_TOOLBAR_CONTROLS,
    SNAPSHOT_LABEL_MAX_CHARS, STEP_ONCE_ACTIONS, STEP_ONCE_INPUT_ENABLED,
    TABLEAU_ANIMATION_SETTLE_DELAY_MS, UNBOUNDED_MULTI_STEP_ACTIONS, VISIBLE_LOG_ROLLOVER_GAMES,
};
use crate::parameters::{AnimationSettleDelays, StepRunSettings};
use crate::parameters::{default_qmp_socket_path, session_log_directory, snapshot_directory};

/// Fill colour for setup, capture and snapshot controls.
const SETUP_BLUE: Color32 = Color32::from_rgb(27, 96, 157);
/// Fill colour for controls that start an authorised run.
const ACTION_GREEN: Color32 = Color32::from_rgb(35, 112, 43);
/// Fill colour for stop and exit controls.
const STATE_RED: Color32 = Color32::from_rgb(172, 35, 38);
/// Sub-pixel allowance that corrects f32 round-trip drift at exact pixel boundaries.
const PREVIEW_MAPPING_FLOAT_EPSILON: f32 = 0.0005;

/// UI ownership of a dispatched run until worker completion or cancellation arrives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ActiveRun {
    /// One user-approved input and its resulting observation.
    StepOnce,
    /// A bounded or continuous sequence using accepted result frames.
    MultiStep,
}

/// Deferred clipboard operation selected from the snapshot-label context menu.
enum SnapshotEditAction {
    /// Copy selected text to the clipboard, then remove it from the label.
    Cut,
    /// Copy selected text without changing the label.
    Copy,
    /// Insert sanitised clipboard text within the label length limit.
    Paste,
}

/// Owns UI state and submits bounded requests to the independent QMP worker.
pub struct QmpQemuSocketApp {
    /// Command sender and event receiver for the background QMP worker.
    worker: WorkerHandle,
    /// Latest worker state used to gate controls and describe connection progress.
    worker_state: WorkerState,
    /// Selected calibration and gameplay profile.
    game_mode: GameMode,
    /// Editable local socket pathname; edits revoke preview authority.
    qmp_socket_path: String,
    /// Bounded visible tail of timestamped session output.
    log_lines: Vec<String>,
    /// Whether the detailed-output body occupied space in the previous render.
    output_expanded: bool,
    /// Complete private session history, absent after creation or write failure.
    session_log: Option<SessionLog>,
    /// Successfully restarted games counted for visible-output rollover.
    completed_games: usize,
    /// Monotonic origin for elapsed timestamps in UI-generated log entries.
    started_at: Instant,
    /// Whether pointer movement paints the guest-pixel alignment crosshair.
    show_coordinates: bool,
    /// Whether calibrated target rectangles and click markers are overlaid.
    draw_targets: bool,
    /// Visibility of the editable settings window.
    show_parameters: bool,
    /// Visibility of the original-PNG review and save dialog.
    show_snapshot_dialog: bool,
    /// Optional user text sanitised when reserving a snapshot filename.
    snapshot_label: String,
    /// Wrapping request token used to reject superseded snapshot results.
    snapshot_request_id: u64,
    /// Whether the current snapshot request is awaiting its worker result.
    snapshot_capture_pending: bool,
    /// One-shot request to focus the snapshot label after opening the dialog.
    snapshot_focus_pending: bool,
    /// Character selection retained while the label context menu has focus.
    snapshot_label_selection: Option<egui::text::CCursorRange>,
    /// Current capture or save error displayed without discarding retryable bytes.
    snapshot_error: Option<String>,
    /// Original QMP PNG bytes retained for saving exactly the reviewed capture.
    snapshot_png: Option<Vec<u8>>,
    /// Decoded pixels paired with the retained original PNG.
    snapshot_frame: Option<CapturedFrame>,
    /// Detector result paired with the snapshot frame.
    snapshot_prediction: Option<PredictedAction>,
    /// Socket and mode associated with the snapshot request.
    snapshot_context: Option<(PathBuf, GameMode)>,
    /// Display texture for the snapshot review dialog.
    snapshot_texture: Option<egui::TextureHandle>,
    /// Host clipboard access used by snapshot-label editing.
    clipboard: Clipboard,
    /// Latest main-preview texture, independent of pending snapshot pixels.
    capture_texture: Option<egui::TextureHandle>,
    /// Dimensions of the installed main preview.
    captured_dimensions: Option<(u32, u32)>,
    /// Socket and mode from which the installed main preview originated.
    preview_context: Option<(PathBuf, GameMode)>,
    /// Displayed detector result, subject to fresh worker validation before input.
    prediction: Option<PredictedAction>,
    /// Marks an unverified result frame as inspection-only.
    diagnostic_preview: bool,
    /// TriPeaks draw settling delay copied into the next run settings.
    draw_animation_settle_ms: u64,
    /// TriPeaks tableau settling delay copied into the next run settings.
    tableau_animation_settle_ms: u64,
    /// Redeal settling delay copied into the next run settings.
    board_redeal_settle_ms: u64,
    /// Pyramid Move/Recycle settling delay copied into the next run settings.
    pyramid_move_settle_ms: u64,
    /// Pyramid card-click settling delay copied into the next run settings.
    pyramid_card_settle_ms: u64,
    /// Pyramid no-halo recapture delay copied into the next run settings.
    pyramid_reobserve_ms: u64,
    /// Requested operation limit; the configured zero sentinel means continuous.
    multi_step_actions: usize,
    /// Run ownership retained until the worker reports completion.
    active_run: Option<ActiveRun>,
    /// Cancellation latch that blocks new requests while STOP is in flight.
    stop_requested: bool,
    /// Session advisory board index reported by the worker.
    current_board: usize,
    /// Board count supplied by the selected game profile.
    boards_per_game: usize,
    /// Whether a detected transition has established the board advisory.
    board_position_established: bool,
    /// Concise operational message shown outside the detailed output.
    current_status: String,
}

impl QmpQemuSocketApp {
    /// Creates UI state, starts the worker and requests one read-only capture.
    ///
    /// Uses the display handle for clipboard access. A log-creation failure is
    /// reported in the visible output and does not prevent the application opening.
    pub fn new(creation_context: &eframe::CreationContext<'_>) -> Self {
        let started_at = Instant::now();
        let qmp_socket_path = default_qmp_socket_path().display().to_string();
        let clipboard = Clipboard::new(
            creation_context
                .display_handle()
                .ok()
                .map(|handle| handle.as_raw()),
        );
        let (session_log, session_log_error) = match SessionLog::create() {
            Ok(log) => (Some(log), None),
            Err(error) => (None, Some(error.to_string())),
        };
        let mut app = Self {
            worker: WorkerHandle::spawn(),
            worker_state: WorkerState::Detached,
            game_mode: GameMode::TriPeaks,
            qmp_socket_path,
            log_lines: Vec::new(),
            output_expanded: false,
            session_log,
            completed_games: 0,
            started_at,
            show_coordinates: false,
            draw_targets: true,
            show_parameters: false,
            show_snapshot_dialog: false,
            snapshot_label: String::new(),
            snapshot_request_id: 0,
            snapshot_capture_pending: false,
            snapshot_focus_pending: false,
            snapshot_label_selection: None,
            snapshot_error: None,
            snapshot_png: None,
            snapshot_frame: None,
            snapshot_prediction: None,
            snapshot_context: None,
            snapshot_texture: None,
            clipboard,
            capture_texture: None,
            captured_dimensions: None,
            preview_context: None,
            prediction: None,
            diagnostic_preview: false,
            draw_animation_settle_ms: DRAW_ANIMATION_SETTLE_DELAY_MS,
            tableau_animation_settle_ms: TABLEAU_ANIMATION_SETTLE_DELAY_MS,
            board_redeal_settle_ms: BOARD_REDEAL_SETTLE_DELAY_MS,
            pyramid_move_settle_ms: PYRAMID_MOVE_SETTLE_DELAY_MS,
            pyramid_card_settle_ms: PYRAMID_CARD_SETTLE_DELAY_MS,
            pyramid_reobserve_ms: PYRAMID_REOBSERVE_DELAY_MS,
            multi_step_actions: DEFAULT_MULTI_STEP_ACTIONS,
            active_run: None,
            stop_requested: false,
            current_board: 1,
            boards_per_game: GameMode::TriPeaks.profile().boards_per_game,
            board_position_established: false,
            current_status: "Ready".to_owned(),
        };

        app.push_log(format!(
            "{} {RELEASE_LABEL} started; active game profile={}.",
            crate::parameters::APP_NAME,
            app.game_mode
        ));
        if let Some(path) = app.session_log.as_ref().map(|log| log.path().to_path_buf()) {
            app.push_log(format!(
                "Complete session output is being written to {} with mode 0600.",
                path.display()
            ));
        } else if let Some(error) = session_log_error {
            app.push_log(format!(
                "WARNING: the {} session log could not be created ({error}); Copy Output will use only the visible retained lines.",
                session_log_directory().display(),
            ));
        }
        app.push_log(format!("QMP socket candidate: {}", app.qmp_socket_path));
        app.push_log("Initial Capture Frame is automatic, one-shot and read-only.");
        app.push_log("The preview retains only the latest worker frame; stale full-resolution previews are coalesced while the UI is asleep or occluded.");
        app.push_log("Step Once and Multi-Step freshly validate the initial displayed prediction and reuse each accepted result frame as the next plan. Pyramid can continue a repeated Left–Right pair from fresh settled halos without claiming the prior effect was proven.");
        app.push_log(format!(
            "Execution defaults: one initial planning frame plus fresh result captures after each action, Multi-Step limit {} (0 means continuous). STOP is visible only while a guarded run is active.",
            DEFAULT_MULTI_STEP_ACTIONS,
        ));
        app.request_capture("Startup");
        app
    }

    /// Timestamps a message, appends it to the session file and retains its visible tail.
    ///
    /// A write failure disables further file-backed logging and adds a visible warning.
    fn push_log(&mut self, message: impl Into<String>) {
        let line = format!(
            "[+{:08.3}s] {}",
            self.started_at.elapsed().as_secs_f64(),
            message.into()
        );
        let session_error = self
            .session_log
            .as_mut()
            .and_then(|log| log.append(&line).err());
        if let Some(error) = session_error {
            self.session_log = None;
            self.push_visible_log_line(format!(
                "[+{:08.3}s] WARNING: session log write failed ({error}); retaining output in the visible panel only.",
                self.started_at.elapsed().as_secs_f64()
            ));
        }
        self.push_visible_log_line(line);
    }

    /// Retains the newest line while evicting enough old lines to respect `MAX_LOG_LINES`.
    fn push_visible_log_line(&mut self, line: String) {
        if self.log_lines.len() >= MAX_LOG_LINES {
            let remove_count = self.log_lines.len() + 1 - MAX_LOG_LINES;
            self.log_lines.drain(0..remove_count);
        }
        self.log_lines.push(line);
    }

    /// Timestamps a housekeeping entry without adding it to the visible panel.
    ///
    /// Returns a message on unavailable or failed file logging and disables the failed log.
    fn append_session_only(&mut self, message: impl Into<String>) -> Result<(), String> {
        let line = format!(
            "[+{:08.3}s] {}",
            self.started_at.elapsed().as_secs_f64(),
            message.into()
        );
        let result = match self.session_log.as_mut() {
            Some(log) => log
                .append(&line)
                .map_err(|error| format!("session log write failed: {error}")),
            None => Err("no session log is available".to_owned()),
        };
        if result.is_err() {
            self.session_log = None;
        }
        result
    }

    /// Returns the complete session file, or the visible tail when no file is available.
    ///
    /// A failed read of an existing file is reported rather than silently truncating output.
    fn complete_output(&mut self) -> Result<String, String> {
        match self.session_log.as_mut() {
            Some(log) => log
                .complete_text()
                .map_err(|error| format!("Could not read the complete session output: {error}")),
            None => Ok(self.log_lines.join("\n")),
        }
    }

    /// Counts a restarted game and periodically clears only the rendered log tail.
    ///
    /// Rollover is skipped if its marker cannot be written to the complete session log.
    fn roll_visible_output_after_completed_game(&mut self) {
        self.completed_games = self.completed_games.saturating_add(1);
        if self.completed_games % VISIBLE_LOG_ROLLOVER_GAMES != 0 {
            return;
        }
        let boards_per_game = self.game_mode.profile().boards_per_game;
        if let Err(error) = self.append_session_only(format!(
            "Visible output rollover after {} completed games ({} boards); the complete history remains in the session log.",
            VISIBLE_LOG_ROLLOVER_GAMES,
            VISIBLE_LOG_ROLLOVER_GAMES.saturating_mul(boards_per_game),
        )) {
            self.push_log(format!(
                "WARNING: visible output rollover was skipped because {error}."
            ));
            return;
        }
        self.log_lines.clear();
        self.push_log(format!(
            "Visible output cleared after {} games ({} boards). Copy Output still reads the complete session log.",
            VISIBLE_LOG_ROLLOVER_GAMES,
            VISIBLE_LOG_ROLLOVER_GAMES.saturating_mul(boards_per_game),
        ));
    }

    /// Applies queued worker events and installs only the latest compatible preview.
    ///
    /// Snapshot request tokens reject superseded captures; diagnostic frames remain
    /// inspection-only even when their pixels and predictions are displayed.
    fn poll_worker(&mut self, context: &egui::Context) {
        let events: Vec<_> = self.worker.try_events().collect();
        for event in events {
            match event {
                WorkerEvent::Log(message) => self.push_log(message),
                WorkerEvent::Status(message) => self.current_status = message,
                WorkerEvent::SnapshotPrepared {
                    request_id,
                    context: capture_context,
                    frame,
                    png,
                    prediction,
                } => {
                    if self.show_snapshot_dialog
                        && request_id == self.snapshot_request_id
                        && capture_context == self.current_preview_context()
                    {
                        self.snapshot_capture_pending = false;
                        match frame_to_colour_image(&frame) {
                            Ok(image) => {
                                self.snapshot_texture = Some(context.load_texture(
                                    "qmp-snapshot-to-save",
                                    image,
                                    egui::TextureOptions::LINEAR,
                                ));
                                self.snapshot_context = Some(capture_context);
                                self.snapshot_prediction = Some(prediction);
                                self.snapshot_frame = Some(frame);
                                self.snapshot_png = Some(png);
                                self.snapshot_error = None;
                            }
                            Err(error) => {
                                self.snapshot_error =
                                    Some(format!("Could not preview the captured PNG: {error}"));
                            }
                        }
                    }
                }
                WorkerEvent::SnapshotPreparationFailed { request_id, error } => {
                    if self.show_snapshot_dialog && request_id == self.snapshot_request_id {
                        self.snapshot_capture_pending = false;
                        self.snapshot_error = Some(error);
                    }
                }
                WorkerEvent::BoardProgress {
                    current_board,
                    completed_boards,
                    boards_per_game,
                } => {
                    self.current_board = current_board;
                    self.boards_per_game = boards_per_game;
                    self.board_position_established = completed_boards > 0;
                }
                WorkerEvent::State(state) => {
                    self.worker_state = state;
                    if matches!(state, WorkerState::Detached) {
                        self.stop_requested = false;
                        self.invalidate_preview();
                    }
                    if matches!(
                        state,
                        WorkerState::Detached
                            | WorkerState::Ready
                            | WorkerState::Uncertain
                            | WorkerState::Error
                    ) {
                        self.active_run = None;
                    }
                    if matches!(state, WorkerState::Error) {
                        self.invalidate_preview();
                    }
                }
                WorkerEvent::ActionCompleted {
                    operation_index,
                    operation_limit,
                    before,
                    after,
                    input_commands,
                    input_events,
                    changed_pixels,
                    continued_from_halo,
                } => {
                    self.current_status = display_action(before);
                    let progress = format_operation_progress(operation_index, operation_limit);
                    let acceptance = if continued_from_halo {
                        "continued from fresh halo (prior effect unproven)"
                    } else {
                        "verified"
                    };
                    self.push_log(format!(
                        "Operation {progress} {acceptance}: {} -> {}; changed effect pixels={changed_pixels}; QMP commands={input_commands}, input events={input_events}, guest-input retries=0.",
                        concise_prediction(before),
                        concise_prediction(after),
                    ));
                }
                WorkerEvent::GameCompleted => {
                    self.board_position_established = true;
                    self.push_log(format!(
                        "{}-board game completed and the next Solver board was verified ready.",
                        self.game_mode.profile().boards_per_game,
                    ));
                    self.roll_visible_output_after_completed_game();
                }
                WorkerEvent::RunCompleted {
                    prediction,
                    requested_operations,
                    verified_operations,
                    halo_operations,
                } => {
                    self.active_run = None;
                    self.current_status = "Ready".to_owned();
                    let requested = format_operation_limit(requested_operations);
                    self.push_log(format!(
                        "Execution run completed: verified {verified_operations} operation(s), continued from fresh halo {halo_operations} operation(s), from a {requested} request; final prediction={}; guest-input retries=0.",
                        concise_prediction(prediction),
                    ));
                }
            }
        }

        if let Some(latest) = self.worker.take_latest_frame() {
            if latest.coalesced_frames > 0 {
                self.push_log(format!(
                    "Preview backlog bounded: {} stale full-resolution frame(s) were coalesced; only the newest frame was retained.",
                    latest.coalesced_frames,
                ));
            }
            if latest.context == Some(self.current_preview_context())
                && !matches!(
                    self.worker_state,
                    WorkerState::Detached | WorkerState::Error
                )
                && (self.worker_state != WorkerState::Uncertain || latest.diagnostic)
            {
                self.install_frame(
                    context,
                    latest.frame,
                    latest.prediction,
                    latest.log_prediction,
                    latest.context,
                );
                if latest.diagnostic && self.capture_texture.is_some() {
                    self.diagnostic_preview = true;
                    self.prediction = None;
                    self.push_log("Latest unverified post-action QMP frame is displayed for diagnosis; capture a fresh frame before another run.");
                }
            } else {
                self.push_log("Discarded a preview from a different mode/socket or a detached/uncertain worker state without a diagnostic frame.");
            }
        }
    }

    /// Validates and uploads a captured frame, retaining its prediction and source context.
    ///
    /// When requested, logs the prediction. An invalid layout sets an error state
    /// and revokes the existing preview instead of displaying unchecked pixels.
    fn install_frame(
        &mut self,
        context: &egui::Context,
        frame: CapturedFrame,
        prediction: PredictedAction,
        log_prediction: bool,
        frame_context: Option<(PathBuf, GameMode)>,
    ) {
        let dimensions = (frame.width, frame.height);
        match frame_to_colour_image(&frame) {
            Ok(image) => {
                self.capture_texture = Some(context.load_texture(
                    "qmp-primary-head-0",
                    image,
                    egui::TextureOptions::NEAREST,
                ));
                self.captured_dimensions = Some(dimensions);
                self.preview_context = frame_context;
                self.prediction = Some(prediction);
                self.diagnostic_preview = false;
                if log_prediction {
                    self.push_log(format_prediction(prediction));
                }
            }
            Err(error) => {
                self.worker_state = WorkerState::Error;
                self.invalidate_preview();
                self.push_log(format!("Captured frame rejected by the UI: {error}"));
            }
        }
    }

    /// Returns the trimmed socket pathname and selected mode used to identify captures.
    fn current_preview_context(&self) -> (PathBuf, GameMode) {
        (PathBuf::from(self.qmp_socket_path.trim()), self.game_mode)
    }

    /// Reports whether the displayed frame can authorise a freshly validated run request.
    fn has_current_preview(&self) -> bool {
        preview_is_current(
            self.captured_dimensions,
            self.preview_context.as_ref(),
            &self.current_preview_context(),
            self.capture_texture.is_some(),
            self.stop_requested,
            self.diagnostic_preview,
        )
    }

    /// Drops displayed pixels, prediction and source context so they cannot authorise input.
    fn invalidate_preview(&mut self) {
        self.capture_texture = None;
        self.captured_dimensions = None;
        self.preview_context = None;
        self.prediction = None;
        self.diagnostic_preview = false;
    }

    /// Requests one read-only frame when no run, cancellation or worker operation is active.
    ///
    /// `source` labels the request in the log; dispatch failures become visible UI errors.
    fn request_capture(&mut self, source: &str) {
        if self.worker_state.is_busy() || self.active_run.is_some() || self.stop_requested {
            return;
        }
        self.invalidate_preview();
        self.worker_state = WorkerState::Connecting;
        self.current_status = "Screengrab".to_owned();
        self.push_log(format!(
            "{source}: one read-only {} QMP frame requested; guest input events=0.",
            self.game_mode
        ));
        let (socket_path, mode) = self.current_preview_context();
        if let Err(error) = self.worker.capture_frame(socket_path, mode) {
            self.worker_state = WorkerState::Error;
            self.current_status = "Capture failed".to_owned();
            self.push_log(error);
        }
    }

    /// Starts a fresh snapshot request and clears any superseded review artefact.
    ///
    /// Busy or cancelling runs are left untouched. Dispatch errors remain visible
    /// in the dialog, and the request token prevents old results replacing new ones.
    fn prepare_snapshot(&mut self) {
        if self.worker_state.is_busy() || self.active_run.is_some() || self.stop_requested {
            return;
        }
        self.snapshot_request_id = self.snapshot_request_id.wrapping_add(1);
        self.snapshot_png = None;
        self.snapshot_frame = None;
        self.snapshot_prediction = None;
        self.snapshot_context = None;
        self.snapshot_texture = None;
        self.snapshot_error = None;
        self.snapshot_label_selection = None;
        self.snapshot_capture_pending = true;
        self.worker_state = WorkerState::Connecting;
        self.current_status = "Capturing snapshot".to_owned();
        self.push_log(format!(
            "Snapshot capture requested: one fresh read-only {} QMP PNG for review before saving; guest input events=0.",
            self.game_mode
        ));
        let (socket_path, mode) = self.current_preview_context();
        if let Err(error) =
            self.worker
                .prepare_snapshot(socket_path, mode, self.snapshot_request_id)
        {
            self.worker_state = WorkerState::Error;
            self.snapshot_capture_pending = false;
            self.snapshot_error = Some(error.clone());
            self.current_status = "Snapshot capture failed".to_owned();
            self.push_log(error);
        }
    }

    /// Saves the exact retained PNG only after its source context and readiness checks pass.
    ///
    /// A successful save installs its decoded frame in the main preview. A failed
    /// reservation or write retains the bytes and reports the error for a later retry.
    fn save_prepared_snapshot(&mut self, context: &egui::Context) {
        if self.snapshot_capture_pending
            || !controls_are_mutable(self.active_run, self.worker_state, self.stop_requested)
            || self.snapshot_context.as_ref() != Some(&self.current_preview_context())
            || self.snapshot_texture.is_none()
        {
            self.snapshot_error = Some("Capture is not ready for this game and socket.".to_owned());
            return;
        }
        let Some(png) = self.snapshot_png.as_ref() else {
            self.snapshot_error = Some("No captured PNG is available to save.".to_owned());
            return;
        };
        let png_len = png.len();
        let result = SnapshotArtifact::reserve(&snapshot_directory(), &self.snapshot_label)
            .and_then(|destination| destination.save(png));
        match result {
            Ok(path) => {
                self.push_log(format!(
                    "Original previewed QMP PNG saved: {} ({} bytes); guest input events=0; additional screendumps=0.",
                    path.display(), png_len,
                ));
                if let (Some(frame), Some(prediction), Some(capture_context)) = (
                    self.snapshot_frame.take(),
                    self.snapshot_prediction.take(),
                    self.snapshot_context.take(),
                ) {
                    self.install_frame(context, frame, prediction, false, Some(capture_context));
                }
                self.current_status = "Snapshot saved".to_owned();
                self.close_snapshot_dialog();
            }
            Err(error) => {
                self.snapshot_error = Some(error.clone());
                self.current_status = "Snapshot save failed".to_owned();
                self.push_log(format!(
                    "Snapshot save failed; captured PNG retained for retry: {error}"
                ));
            }
        }
    }

    /// Closes snapshot review and releases its bytes, decoded frame and pending selection.
    fn close_snapshot_dialog(&mut self) {
        self.show_snapshot_dialog = false;
        self.snapshot_capture_pending = false;
        self.snapshot_png = None;
        self.snapshot_frame = None;
        self.snapshot_prediction = None;
        self.snapshot_context = None;
        self.snapshot_texture = None;
        self.snapshot_error = None;
        self.snapshot_label_selection = None;
    }

    /// Snapshots controls and asks the worker to execute the requested operation limit.
    ///
    /// Requires an authorised mode and current actionable preview. STOP and failed
    /// preconditions refuse dispatch; worker-channel errors release UI run ownership.
    fn dispatch_steps(
        &mut self,
        operation_limit: usize,
        request_name: &str,
        active_run: ActiveRun,
    ) {
        if self.stop_requested {
            self.push_log(format!(
                "{request_name} refused: STOP is still being processed; guest input sent=0."
            ));
            return;
        }
        if !self.game_mode.input_authorised() {
            self.worker_state = WorkerState::Ready;
            self.current_status = format!("{} calibration — input disabled", self.game_mode);
            self.push_log(format!(
                "{request_name} refused: {} is read-only calibration mode; guest input sent=0.",
                self.game_mode
            ));
            return;
        }
        let Some(approved_prediction) = self.prediction.filter(|prediction| {
            self.has_current_preview() && prediction_is_actionable(self.game_mode, *prediction)
        }) else {
            self.worker_state = WorkerState::Error;
            self.push_log(format!(
                "{request_name} refused: no approved preview target is available."
            ));
            return;
        };

        let socket_path = PathBuf::from(self.qmp_socket_path.trim());
        let animation_delays = AnimationSettleDelays::from_millis(
            self.draw_animation_settle_ms,
            self.tableau_animation_settle_ms,
            self.board_redeal_settle_ms,
        )
        .with_pyramid_millis(
            self.pyramid_move_settle_ms,
            self.pyramid_card_settle_ms,
            self.pyramid_reobserve_ms,
        );
        let settings = StepRunSettings::new(animation_delays, operation_limit);
        let request_scope = if settings.is_unbounded() {
            "continuously until STOP or a fail-closed anomaly".to_owned()
        } else {
            format!("for up to {} operation(s)", settings.operation_limit())
        };

        self.worker_state = WorkerState::Validating;
        self.active_run = Some(active_run);
        self.current_status = "Validating next move".to_owned();
        self.push_log(format!(
            "{request_name} requested {request_scope}, starting from approved preview target: {}. The initial target receives one fresh validation capture; each accepted result frame then becomes the next planning frame.",
            concise_prediction(approved_prediction),
        ));
        if let Err(error) =
            self.worker
                .execute_steps(socket_path, self.game_mode, approved_prediction, settings)
        {
            self.active_run = None;
            self.worker_state = WorkerState::Error;
            self.current_status = "Run request failed".to_owned();
            self.push_log(error);
        }
    }

    /// Draws mode, capture and execution controls with run-state gating.
    ///
    /// Mode or socket changes invalidate the previous preview before it can authorise input.
    fn top_bar(&mut self, ui: &mut egui::Ui) {
        // A mode/socket change invalidates an old frame before any action can use it.
        if self
            .preview_context
            .as_ref()
            .is_some_and(|context| context != &self.current_preview_context())
        {
            self.invalidate_preview();
        }
        let controls_enabled =
            controls_are_mutable(self.active_run, self.worker_state, self.stop_requested)
                && !self.show_snapshot_dialog;
        let mut preview_available = self.has_current_preview();
        ui.horizontal(|ui| {
            ui.label("Game Type");
            let previous_mode = self.game_mode;
            ui.add_enabled_ui(controls_enabled, |ui| {
                let style = ui.style_mut();
                style.visuals.widgets.inactive.bg_fill = SETUP_BLUE;
                style.visuals.widgets.hovered.bg_fill = SETUP_BLUE.gamma_multiply(1.2);
                style.visuals.widgets.inactive.fg_stroke.color = Color32::WHITE;
                egui::ComboBox::from_id_salt("game-mode")
                    .selected_text(egui::RichText::new(self.game_mode.label()).strong().color(Color32::WHITE))
                    .show_ui(ui, |ui| {
                        for mode in GameMode::AVAILABLE {
                            ui.selectable_value(&mut self.game_mode, mode, egui::RichText::new(mode.label()).strong());
                        }
                    });
            });
            if self.game_mode != previous_mode {
                self.invalidate_preview();
                preview_available = false;
                self.current_board = 1;
                self.board_position_established = false;
                self.boards_per_game = self.game_mode.profile().boards_per_game;
                self.current_status = if self.game_mode.calibration_only() {
                    format!("{} calibration — input disabled", self.game_mode)
                } else {
                    "Ready — capture required".to_owned()
                };
                match self.worker.reset_progress() {
                    Ok(()) => {
                        self.push_log(format!(
                            "Game Type changed to {}; prior prediction and advisory progress were cleared. Capturing a fresh read-only frame; guest input events=0.",
                            self.game_mode,
                        ));
                        self.request_capture("Game Type changed");
                    }
                    Err(error) => {
                        self.worker_state = WorkerState::Error;
                        self.current_status = "Mode change failed".to_owned();
                        self.push_log(format!(
                            "Game Type changed locally, but worker progress reset failed: {error}"
                        ));
                    }
                }
            }

            if bevel_button(ui, "Capture Frame", SETUP_BLUE, false, controls_enabled)
                .on_hover_text("Capture and analyse one frame; send no guest input")
                .clicked()
            {
                self.request_capture("Manual capture");
            }

            if bevel_button(ui, "Track", SETUP_BLUE, self.show_coordinates, controls_enabled).clicked() {
                self.show_coordinates = !self.show_coordinates;
                self.push_log(if self.show_coordinates {
                    "Coordinate reporting enabled: left-click on the preview to log an exact guest pixel; no guest input is sent."
                } else {
                    "Coordinate reporting disabled."
                });
            }

            if preview_available {
                if bevel_button(ui, "Draw Targets", SETUP_BLUE, self.draw_targets, controls_enabled).clicked() {
                    self.draw_targets = !self.draw_targets;
                    self.push_log(if self.draw_targets {
                        "Read-only target outlines shown."
                    } else {
                        "Read-only target outlines hidden."
                    });
                    self.request_capture("Draw Targets changed");
                    preview_available = false;
                }
            }
            if bevel_button(ui, "Capture PNG", SETUP_BLUE, false, controls_enabled)
                .on_hover_text("Capture and preview one fresh original QMP PNG before saving; send no guest input")
                .clicked()
            {
                self.snapshot_label.clear();
                self.show_snapshot_dialog = true;
                self.snapshot_focus_pending = true;
                self.prepare_snapshot();
            }
            ui.separator();
            let state_colour = match self.worker_state {
                WorkerState::Detached => Color32::GRAY,
                WorkerState::Connecting
                | WorkerState::Capturing
                | WorkerState::Validating
                | WorkerState::Acting
                | WorkerState::Verifying => Color32::YELLOW,
                WorkerState::Ready => Color32::LIGHT_GREEN,
                WorkerState::Uncertain => Color32::from_rgb(255, 150, 70),
                WorkerState::Error => Color32::LIGHT_RED,
            };
            ui.colored_label(state_colour, format!("QMP: {}", self.worker_state.label()));

            if self.active_run.is_some() {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if bevel_button(ui, "STOP", STATE_RED, false, true).clicked() {
                        self.stop_requested = true;
                        self.current_status = "Stopping…".to_owned();
                        if let Err(error) = self.worker.disconnect() {
                            self.active_run = None;
                            self.worker_state = WorkerState::Error;
                            self.current_status = "Stop request failed".to_owned();
                            self.push_log(error);
                        }
                    }
                });
            }
        });
        ui.horizontal(|ui| {
            if bevel_button(ui, "Params", SETUP_BLUE, self.show_parameters, controls_enabled).clicked() {
                self.show_parameters = true;
            }
            if preview_available {
                let action_available = controls_enabled
                    && self.game_mode.input_authorised()
                    && matches!(self.worker_state, WorkerState::Ready)
                    && self.prediction.is_some_and(|prediction| prediction_is_actionable(self.game_mode, prediction));
                if bevel_button(ui, "Single Step", ACTION_GREEN, false, STEP_ONCE_INPUT_ENABLED && action_available)
                    .on_hover_text("Execute one freshly validated guarded action; require a changed effect and valid resulting state")
                    .clicked()
                {
                    self.dispatch_steps(STEP_ONCE_ACTIONS, "Single Step", ActiveRun::StepOnce);
                }
                if bevel_button(ui, "Multiple Steps", ACTION_GREEN, false, MULTI_STEP_INPUT_ENABLED && action_available)
                    .on_hover_text(if self.multi_step_actions == UNBOUNDED_MULTI_STEP_ACTIONS {
                        "Run continuously until STOP or the first guarded stop condition"
                    } else {
                        "Run the configured bounded number of actions"
                    })
                    .clicked()
                {
                    self.dispatch_steps(self.multi_step_actions, "Multiple Steps", ActiveRun::MultiStep);
                }
            }
        });
    }

    /// Draws the native-scale capture, optional overlays and guest-coordinate inspection.
    ///
    /// Pointer clicks here log coordinates only; overlays never modify captured
    /// pixels or the original PNG retained by the snapshot dialog.
    fn preview(&mut self, ui: &mut egui::Ui) {
        ui.heading("QMP capture and target preview");
        match self.captured_dimensions {
            Some((width, height)) if self.diagnostic_preview => {
                ui.colored_label(Color32::from_rgb(255, 174, 79), format!(
                    "Last unverified post-action QMP capture: {width}x{height}. For inspection only; use Capture Frame before another run."
                ));
            }
            Some((width, height)) => {
                ui.label(format!(
                    "Last successful primary-display capture, implicit head 0: {width}x{height}. Displayed prediction is advisory."
                ));
            }
            None => {
                ui.label("Waiting for the initial read-only capture; use Capture Frame to retry if it fails.");
            }
        }
        ui.small(format!(
            "Native 1920×1080 pixel preview; scroll to pan. Display scale: {:.2} physical pixels per UI point.",
            ui.pixels_per_point()
        ));
        let mut tracked_point = None;
        let viewport_height = preview_viewport_height(
            ui.available_height(),
            ui.pixels_per_point(),
            self.output_expanded,
        );
        egui::ScrollArea::both()
            .auto_shrink([false, false])
            .max_height(viewport_height)
            .show(ui, |ui| {
                // The logical canvas scales with the host display so its physical
                // extent remains one framebuffer pixel per image pixel.
                let scale = ui.pixels_per_point();
                let canvas_size = egui::vec2(
                    NOMINAL_FRAME_WIDTH as f32 / scale,
                    NOMINAL_FRAME_HEIGHT as f32 / scale,
                );
                let (canvas, response) = ui.allocate_exact_size(canvas_size, Sense::click());
                let painter = ui.painter_at(canvas);
                painter.rect_filled(canvas, 0.0, Color32::from_rgb(24, 43, 37));
                if let Some(texture) = &self.capture_texture {
                    painter.image(
                        texture.id(),
                        canvas,
                        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                        Color32::WHITE,
                    );
                } else {
                    paint_grid(&painter, canvas);
                    painter.text(
                        canvas.center(),
                        Align2::CENTER_CENTER,
                        "No QMP frame captured",
                        FontId::proportional(20.0),
                        Color32::from_gray(175),
                    );
                }
                // Preview-only paint operations. Never modify capture pixels or
                // snapshot_png: saved evidence retains QEMU's original PNG bytes.
                if self.draw_targets {
                    let profile = self.game_mode.profile();
                    for target in profile.preview_targets {
                        paint_target(
                            &painter,
                            canvas,
                            target.bounds,
                            Color32::from_rgb(target.colour[0], target.colour[1], target.colour[2]),
                            target.label,
                        );
                    }
                    if self.game_mode == GameMode::Pyramid {
                        for target in &PYRAMID_TARGETS {
                            paint_labeled_crosshair(
                                &painter,
                                canvas,
                                target.click_point,
                                Color32::from_rgb(255, 100, 210),
                                "",
                            );
                        }
                    }
                    for (label, control) in SHARED_TOOLBAR_CONTROLS {
                        let colour = Color32::from_rgb(120, 200, 255);
                        paint_target(&painter, canvas, control.bounds, colour, label);
                        paint_labeled_crosshair(&painter, canvas, control.click_point, colour, "");
                    }
                }
                if let Some(prediction) = self.prediction {
                    paint_prediction(&painter, canvas, prediction);
                }
                let pointer = response
                    .hovered()
                    .then(|| ui.ctx().pointer_hover_pos())
                    .flatten()
                    .filter(|position| canvas.contains(*position))
                    .map(|position| preview_to_pixel(canvas, position));
                if let Some(point) = pointer {
                    if self.show_coordinates {
                        paint_crosshair(&painter, canvas, point);
                    }
                    if self.show_coordinates && response.clicked() {
                        tracked_point = Some(point);
                    }
                }
            });
        if let Some(point) = tracked_point {
            let targets = self
                .game_mode
                .profile()
                .preview_targets
                .iter()
                .map(|target| format!("{}={}", target.label, target.bounds.contains(point)))
                .collect::<Vec<_>>()
                .join(" | ");
            self.push_log(format!(
                "Left-clicked read-only capture at guest pixel x={}, y={} | {}",
                point.x, point.y, targets
            ));
        }
    }

    /// Draws the collapsible diagnostic tail and complete-history copy controls.
    ///
    /// Records the expanded state so the preview can reclaim space when collapsed.
    /// Manual clearing resets progress only after the worker accepts the reset.
    fn output(&mut self, ui: &mut egui::Ui) {
        ui.separator();
        let output_response = egui::CollapsingHeader::new("Detailed output")
            .default_open(false)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .button("Copy Output")
                        .on_hover_text(
                            "Copy the complete file-backed session output to the clipboard",
                        )
                        .clicked()
                    {
                        match self.complete_output() {
                            Ok(output) => ui.ctx().copy_text(output),
                            Err(error) => self.push_log(error),
                        }
                    }
                    if ui
                        .add_enabled(
                            !self.worker_state.is_busy(),
                            egui::Button::new("Clear Output"),
                        )
                        .on_hover_text(
                            "Clear the visible panel and reset board, game, and row-scan progress; the complete session log is retained",
                        )
                        .clicked()
                    {
                        match self.worker.reset_progress() {
                            Ok(()) => {
                                self.completed_games = 0;
                                self.current_board = 1;
                                self.board_position_established = false;
                                self.current_status = "Ready".to_owned();
                                let clear_result = self.append_session_only(
                                    "Visible output was manually cleared; board, game, and row-scan progress were reset; complete session history retained.",
                                );
                                match clear_result {
                                    Ok(()) => self.log_lines.clear(),
                                    Err(error) => self.push_log(format!(
                                        "Progress tracking was reset, but visible output was retained because {error}."
                                    )),
                                }
                            }
                            Err(error) => self.push_log(format!(
                                "Clear Output refused because progress tracking could not be reset: {error}; visible history was retained."
                            )),
                        }
                    }
                });
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.set_min_height(OUTPUT_PANEL_HEIGHT);
                    egui::ScrollArea::vertical()
                        .stick_to_bottom(true)
                        .max_height(OUTPUT_PANEL_HEIGHT)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            for line in &self.log_lines {
                                ui.label(egui::RichText::new(line).monospace());
                            }
                        });
                });
            });
        self.output_expanded = output_response.body_response.is_some();
    }

    /// Shows the concise operational status and advisory board position outside detailed output.
    fn status_panel(&self, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                if self.game_mode.calibration_only() {
                    ui.strong(format!("{} — calibration only", self.game_mode));
                } else {
                    ui.strong(board_position_label(
                        self.game_mode,
                        self.current_board,
                        self.boards_per_game,
                        self.board_position_established,
                    ));
                }
                ui.separator();
                ui.label(&self.current_status);
            });
        });
    }

    /// Draws settings and calibration information while the Parameters window is open.
    ///
    /// Execution values are editable only while idle and are copied into the next
    /// run; changing the socket invalidates the prior preview.
    fn parameters_window(&mut self, context: &egui::Context) {
        if !self.show_parameters {
            return;
        }

        let mut open = self.show_parameters;
        egui::Window::new("Parameters")
            .open(&mut open)
            .resizable(true)
            .show(context, |ui| {
                ui.strong(RELEASE_LABEL);
                ui.label(format!("Game Type: {}", self.game_mode));
                if self.game_mode.calibration_only() {
                    ui.colored_label(
                        Color32::YELLOW,
                        "Read-only calibration profile — all guest input is disabled",
                    );
                }
                ui.separator();
                ui.label("QMP Unix socket");
                let socket_edit = ui.add_enabled(
                    !self.worker_state.is_busy()
                        && self.active_run.is_none()
                        && !self.show_snapshot_dialog,
                    egui::TextEdit::singleline(&mut self.qmp_socket_path),
                );
                if socket_edit.changed() {
                    self.invalidate_preview();
                    self.current_status = "Socket changed — capture required".to_owned();
                    self.push_log("QMP socket changed locally; prior preview authority was discarded. Capture a fresh read-only frame before input.");
                }
                ui.separator();
                ui.label("Execution settings for the next Step Once or Multi-Step run");
                let execution_enabled = !self.worker_state.is_busy()
                    && self.active_run.is_none()
                    && !self.show_snapshot_dialog;
                if execution_enabled {
                    ui.small("Double-click a number to type milliseconds, or drag to adjust (0–5000 ms).");
                } else {
                    ui.small("Timing controls are locked during a run or capture. STOP and close any snapshot dialog to edit the next run.");
                }
                ui.add_enabled_ui(execution_enabled, |ui| {
                    if self.game_mode == GameMode::Pyramid {
                        ui.horizontal(|ui| {
                            ui.label("After Pyramid MOVE / Recycle click");
                            ui.add(
                                egui::DragValue::new(&mut self.pyramid_move_settle_ms)
                                    .range(
                                        MINIMUM_ANIMATION_SETTLE_DELAY_MS
                                            ..=MAXIMUM_ANIMATION_SETTLE_DELAY_MS,
                                    )
                                    .speed(10.0)
                                    .suffix(" ms"),
                            )
                            .on_hover_text("Wait after clicking MOVE or Recycle before capturing its result");
                        });
                        ui.horizontal(|ui| {
                            ui.label("After Pyramid card / Left / Right click");
                            ui.add(
                                egui::DragValue::new(&mut self.pyramid_card_settle_ms)
                                    .range(
                                        MINIMUM_ANIMATION_SETTLE_DELAY_MS
                                            ..=MAXIMUM_ANIMATION_SETTLE_DELAY_MS,
                                    )
                                    .speed(10.0)
                                    .suffix(" ms"),
                            )
                            .on_hover_text("Wait after a card or pile click, including time for the next MOVE halo to appear");
                        });
                        ui.small("For a late MOVE halo after a card disappears, increase the card / Left / Right delay above.");
                        ui.horizontal(|ui| {
                            ui.label("Pyramid repeat observation settle");
                            ui.add(
                                egui::DragValue::new(&mut self.pyramid_reobserve_ms)
                                    .range(
                                        MINIMUM_ANIMATION_SETTLE_DELAY_MS
                                            ..=MAXIMUM_ANIMATION_SETTLE_DELAY_MS,
                                    )
                                    .speed(10.0)
                                    .suffix(" ms"),
                            )
                            .on_hover_text("Wait before each fresh capture while the action effect or next halo is still unverified; this sends no gameplay input");
                        });
                    } else {
                        ui.horizontal(|ui| {
                            ui.label("Draw-class action settle");
                            ui.add(
                                egui::DragValue::new(&mut self.draw_animation_settle_ms)
                                    .range(
                                        MINIMUM_ANIMATION_SETTLE_DELAY_MS
                                            ..=MAXIMUM_ANIMATION_SETTLE_DELAY_MS,
                                    )
                                    .speed(10.0)
                                    .suffix(" ms"),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label("Tableau move settle");
                            ui.add(
                                egui::DragValue::new(&mut self.tableau_animation_settle_ms)
                                    .range(
                                        MINIMUM_ANIMATION_SETTLE_DELAY_MS
                                            ..=MAXIMUM_ANIMATION_SETTLE_DELAY_MS,
                                    )
                                    .speed(10.0)
                                    .suffix(" ms"),
                            );
                        });
                    }
                    ui.horizontal(|ui| {
                        ui.label("Board redeal settle");
                        ui.add(
                            egui::DragValue::new(&mut self.board_redeal_settle_ms)
                                .range(
                                    MINIMUM_ANIMATION_SETTLE_DELAY_MS
                                        ..=MAXIMUM_ANIMATION_SETTLE_DELAY_MS,
                                )
                                .speed(50.0)
                                .suffix(" ms"),
                        )
                        .on_hover_text(
                            "Wait after a verified final-card move before checking the next board or Level Up dialog",
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Actions per Multi-Step");
                        ui.add(
                            egui::DragValue::new(&mut self.multi_step_actions)
                                .speed(1.0),
                        )
                        .on_hover_text("0 runs continuously across games until STOP or a fail-closed anomaly; positive values are exact gameplay-action limits");
                    });
                    if self.multi_step_actions == UNBOUNDED_MULTI_STEP_ACTIONS {
                        ui.small("Multi-Step is unbounded: run until STOP or a guarded stop condition.");
                    }
                    if ui.button("Restore execution defaults").clicked() {
                        self.draw_animation_settle_ms = DRAW_ANIMATION_SETTLE_DELAY_MS;
                        self.tableau_animation_settle_ms = TABLEAU_ANIMATION_SETTLE_DELAY_MS;
                        self.board_redeal_settle_ms = BOARD_REDEAL_SETTLE_DELAY_MS;
                        self.pyramid_move_settle_ms = PYRAMID_MOVE_SETTLE_DELAY_MS;
                        self.pyramid_card_settle_ms = PYRAMID_CARD_SETTLE_DELAY_MS;
                        self.pyramid_reobserve_ms = PYRAMID_REOBSERVE_DELAY_MS;
                        self.multi_step_actions = DEFAULT_MULTI_STEP_ACTIONS;
                    }
                });
                if self.game_mode == GameMode::Pyramid {
                    ui.small(format!(
                        "Session defaults: Pyramid Move/Recycle {PYRAMID_MOVE_SETTLE_DELAY_MS} ms, Card/Left/Right {PYRAMID_CARD_SETTLE_DELAY_MS} ms, repeat observation {PYRAMID_REOBSERVE_DELAY_MS} ms, board redeal {BOARD_REDEAL_SETTLE_DELAY_MS} ms, Multi-Step {DEFAULT_MULTI_STEP_ACTIONS} (continuous). Changes apply to the next run."
                    ));
                } else {
                    ui.small(format!(
                        "Execution controls are session-only; defaults: draw {DRAW_ANIMATION_SETTLE_DELAY_MS} ms, tableau {TABLEAU_ANIMATION_SETTLE_DELAY_MS} ms, board redeal {BOARD_REDEAL_SETTLE_DELAY_MS} ms, Multi-Step {DEFAULT_MULTI_STEP_ACTIONS} (continuous)."
                    ));
                }
                if let Some(log) = self.session_log.as_ref() {
                    ui.small(format!("Complete output: {}", log.path().display()));
                }
                ui.small(
                    "Current verification: one initial fresh planning capture, then each accepted result frame is reused for the next plan. Pyramid may continue a repeated Left–Right pair from fresh settled halos; this is logged separately from a proven effect. STOP cancels the active run.",
                );
                ui.separator();
                ui.monospace(format!(
                    "nominal-frame = {NOMINAL_FRAME_WIDTH}x{NOMINAL_FRAME_HEIGHT}"
                ));
                let profile = self.game_mode.profile();
                ui.monospace(format!("game-profile = {}", profile.label));
                for target in profile.preview_targets {
                    ui.monospace(format!(
                        "{:<13} = {}",
                        target.label,
                        format_rect(target.bounds)
                    ));
                }
                if self.game_mode == GameMode::Pyramid {
                    let pyramid_cards = PYRAMID_TARGETS
                        .iter()
                        .filter(|target| target.kind.is_card())
                        .count();
                    ui.monospace("guest-input-authorised = true; one guarded click per action");
                    ui.monospace("target-priority = Move, Left, Right, Card");
                    ui.monospace(format!(
                        "target-slots = {} total, {pyramid_cards} cards (expected {})",
                        PYRAMID_TARGETS.len(), pyramid::TABLEAU_CARD_COUNT,
                    ));
                    ui.monospace("halo-probes = fixed 2x2; first eligible target by priority");
                    ui.monospace("card-state = suppress only a clicked card after verified removal");
                    ui.monospace("Move / Left / Right = repeatable; fresh settled Left–Right halos may authorise the next pair");
                    for target in &PYRAMID_TARGETS {
                        ui.monospace(format!(
                            "{}: bounds {}, hit x={}, y={}, HALO {}",
                            target.label,
                            format_rect(target.bounds),
                            target.click_point.x,
                            target.click_point.y,
                            format_rect(crate::pyramid::halo_probe(*target)),
                        ));
                    }
                } else {
                    ui.monospace("guest-input-authorised = true");
                    ui.monospace(format!(
                        "click-offset = ({:+}, {:+})",
                        profile.tableau_click_offset.x, profile.tableau_click_offset.y
                    ));
                }
                for (label, control) in SHARED_TOOLBAR_CONTROLS {
                    ui.monospace(format!(
                        "shared-toolbar-{label}: bounds {}, hit x={}, y={}",
                        format_rect(control.bounds),
                        control.click_point.x,
                        control.click_point.y,
                    ));
                }
                ui.small("Undo All confirmation requires separate calibration; no confirmation click is automated.");
                if self.game_mode == GameMode::Pyramid {
                    ui.monospace("HALO requires all four pixels to match the Pyramid gold predicate");
                } else {
                    ui.monospace(format!("gold-colours = {GOLD_RGB_CANDIDATES:?}"));
                    ui.monospace(format!(
                        "gold-channel-tolerance = ±{GOLD_CHANNEL_TOLERANCE}"
                    ));
                }
                if self.game_mode == GameMode::Pyramid {
                    ui.monospace("no-highlight-recovery = bounded; never implies Move or completion");
                } else {
                    ui.monospace(format!(
                        "no-highlight-reobserve = until target or STOP, {} ms apart",
                        NO_HIGHLIGHT_REOBSERVE_DELAY.as_millis()
                    ));
                }
                ui.monospace(format!(
                    "board-series = {} boards, redeal wait {} ms",
                    profile.boards_per_game,
                    self.board_redeal_settle_ms
                ));
                ui.monospace(format!(
                    "post-game-stage-delay = {} ms",
                    POST_GAME_STAGE_DELAY.as_millis()
                ));
                ui.monospace(format!(
                    "level-up-appear-delay = {} ms after score click",
                    LEVEL_UP_APPEAR_DELAY.as_millis()
                ));
                ui.monospace(format!(
                    "challenge-complete-continue (future) = probe {}, click x={}, y={} (no automatic input)",
                    format_rect(CHALLENGE_COMPLETE_CONTINUE_CONTROL.bounds),
                    CHALLENGE_COMPLETE_CONTINUE_CONTROL.click_point.x,
                    CHALLENGE_COMPLETE_CONTINUE_CONTROL.click_point.y,
                ));
                ui.monospace(format!(
                    "post-game-bounds = {POST_GAME_MAX_OBSERVATION_ROUNDS} observations/stage, {POST_GAME_MAX_CLICK_ATTEMPTS} confirmed clicks/retry context"
                ));
                ui.monospace(format!(
                    "score-skip-bound = {SCORE_SKIP_MAX_CLICK_ATTEMPTS} confirmed centre clicks total (initial included)"
                ));
                for target in POST_GAME_TARGETS {
                    for variant in target.control_variants {
                        ui.monospace(format!(
                            "post-game-{}[{}] = probe {}, click x={}, y={}",
                            target.label,
                            variant.label,
                            format_rect(variant.probe_bounds),
                            variant.click_point.x,
                            variant.click_point.y,
                        ));
                    }
                }
                ui.monospace(format!(
                    "pointer-settle = {} ms",
                    POINTER_SETTLE_DELAY.as_millis()
                ));
                ui.monospace(format!("mouse-hold = {} ms", MOUSE_HOLD.as_millis()));
                ui.monospace(format!("key-hold = {} ms", KEY_HOLD.as_millis()));
                ui.monospace(
                    "capture-verification = 1 initial planning frame + 1 result frame/action",
                );
                ui.monospace(format!(
                    "multi-step-actions = {}",
                    if self.multi_step_actions == UNBOUNDED_MULTI_STEP_ACTIONS {
                        "until STOP".to_owned()
                    } else {
                        self.multi_step_actions.to_string()
                    }
                ));
                if self.game_mode == GameMode::Pyramid {
                    ui.monospace("action-verification = card removal or material lower-pile change; repeated Left–Right pair may continue from fresh settled halos");
                } else {
                    ui.monospace(format!(
                        "action-change = channel >= {ACTION_CHANGE_CHANNEL_THRESHOLD}, draw pixels >= {MINIMUM_DRAW_CHANGED_PIXELS}, tableau pixels >= {MINIMUM_TABLEAU_CHANGED_PIXELS}"
                    ));
                }
                ui.monospace(format!(
                    "cursor-exclusion-half-size = {ACTION_CURSOR_EXCLUSION_HALF_SIZE} px"
                ));
                ui.monospace(format!(
                    "step-once-input-enabled = {STEP_ONCE_INPUT_ENABLED}"
                ));
                ui.monospace(format!(
                    "multi-step-input-enabled = {MULTI_STEP_INPUT_ENABLED}"
                ));
            });
        self.show_parameters = open;
    }

    /// Draws original-PNG review, bounded label editing and save/recapture controls.
    ///
    /// Clipboard insertion strips control characters and honours the character cap;
    /// closing or cancelling releases the retained snapshot through one cleanup path.
    fn snapshot_window(&mut self, context: &egui::Context) {
        if !self.show_snapshot_dialog {
            return;
        }
        let mut open = self.show_snapshot_dialog;
        let mut save = false;
        let mut cancel = false;
        let mut recapture = false;
        egui::Window::new("Capture QMP PNG")
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_width(660.0)
            .show(context, |ui| {
                ui.label("Review this QMP capture; Save writes these exact PNG bytes.");
                if let Some(texture) = &self.snapshot_texture {
                    let width = ui.available_width().clamp(160.0, 640.0);
                    let height = width * NOMINAL_FRAME_HEIGHT as f32 / NOMINAL_FRAME_WIDTH as f32;
                    ui.image((texture.id(), egui::vec2(width, height)));
                    ui.small(format!(
                        "Original: {NOMINAL_FRAME_WIDTH}×{NOMINAL_FRAME_HEIGHT}, {} PNG bytes",
                        self.snapshot_png.as_ref().map_or(0, Vec::len)
                    ));
                } else if self.snapshot_capture_pending {
                    ui.spinner();
                    ui.label("Capturing one fresh frame through QMP…");
                }
                if let Some(error) = &self.snapshot_error {
                    ui.colored_label(Color32::LIGHT_RED, error);
                }
                ui.monospace(snapshot_directory().display().to_string());
                ui.label("Optional identifying words");
                let mut label_output = egui::TextEdit::singleline(&mut self.snapshot_label)
                    .hint_text("e.g. Pyramid B1 Move halo")
                    .char_limit(SNAPSHOT_LABEL_MAX_CHARS)
                    .desired_width(380.0)
                    .show(ui);
                if std::mem::take(&mut self.snapshot_focus_pending) {
                    label_output.response.request_focus();
                }
                let opening_menu = label_output.response.secondary_clicked()
                    || label_output.response.context_menu_opened();
                let retained_selection = self.snapshot_label_selection.as_ref().filter(|range| {
                    let selected = range.as_sorted_char_range();
                    selected.start != selected.end
                        && selected.end <= self.snapshot_label.chars().count().into()
                });
                let selected_range = if opening_menu {
                    retained_selection
                        .cloned()
                        .or_else(|| label_output.cursor_range.clone())
                } else {
                    self.snapshot_label_selection = label_output.cursor_range.clone();
                    label_output.cursor_range.clone()
                };
                let selected_text = selected_range
                    .as_ref()
                    .map(|range| {
                        self.snapshot_label
                            .char_range(range.as_sorted_char_range())
                            .to_owned()
                    })
                    .unwrap_or_default();
                let mut edit_action = None;
                label_output.response.context_menu(|menu| {
                    if menu
                        .add_enabled(!selected_text.is_empty(), egui::Button::new("Cut"))
                        .clicked()
                    {
                        edit_action = Some(SnapshotEditAction::Cut);
                        menu.close();
                    }
                    if menu
                        .add_enabled(!selected_text.is_empty(), egui::Button::new("Copy"))
                        .clicked()
                    {
                        edit_action = Some(SnapshotEditAction::Copy);
                        menu.close();
                    }
                    if menu.button("Paste").clicked() {
                        edit_action = Some(SnapshotEditAction::Paste);
                        menu.close();
                    }
                });
                let menu_used = edit_action.is_some();
                if let Some(action) = edit_action {
                    self.snapshot_label_selection = None;
                    match action {
                        SnapshotEditAction::Copy => self.clipboard.set_text(selected_text),
                        SnapshotEditAction::Cut => {
                            if let Some(range) = selected_range {
                                self.clipboard.set_text(selected_text);
                                let cursor = self.snapshot_label.delete_selected(&range);
                                label_output
                                    .state
                                    .cursor
                                    .set_char_range(Some(egui::text::CCursorRange::two(
                                        cursor, cursor,
                                    )));
                                label_output.state.store(ui.ctx(), label_output.response.id);
                            }
                        }
                        SnapshotEditAction::Paste => {
                            if let Some(pasted) = self.clipboard.get() {
                                let retained_chars = self
                                    .snapshot_label
                                    .chars()
                                    .count()
                                    .saturating_sub(selected_text.chars().count());
                                let available =
                                    SNAPSHOT_LABEL_MAX_CHARS.saturating_sub(retained_chars);
                                let single_line: String = pasted
                                    .chars()
                                    .filter(|ch| !ch.is_control())
                                    .take(available)
                                    .collect();
                                if !single_line.is_empty() {
                                    let mut cursor = if let Some(range) = selected_range.as_ref() {
                                        self.snapshot_label.delete_selected(range)
                                    } else {
                                        egui::text::CCursor::new(self.snapshot_label.chars().count())
                                    };
                                    cursor +=
                                        self.snapshot_label.insert_text(&single_line, cursor.index);
                                    label_output
                                        .state
                                        .cursor
                                        .set_char_range(Some(egui::text::CCursorRange::two(
                                            cursor, cursor,
                                        )));
                                    label_output.state.store(ui.ctx(), label_output.response.id);
                                }
                            }
                        }
                    }
                    label_output.response.request_focus();
                }
                if !menu_used
                    && !label_output.response.context_menu_opened()
                    && label_output.response.lost_focus()
                    && ui.input(|input| input.key_pressed(egui::Key::Enter))
                    && self.snapshot_png.is_some()
                {
                    save = true;
                }
                ui.small("The local date and time lead the filename; unsafe filename characters become separators.");
                ui.horizontal(|ui| {
                    let enabled = !self.snapshot_capture_pending
                        && self.snapshot_png.is_some()
                        && controls_are_mutable(
                            self.active_run,
                            self.worker_state,
                            self.stop_requested,
                        );
                    if bevel_button(ui, "Save PNG", SETUP_BLUE, false, enabled).clicked() {
                        save = true;
                    }
                    if ui
                        .add_enabled(!self.snapshot_capture_pending, egui::Button::new("Recapture"))
                        .clicked()
                    {
                        recapture = true;
                    }
                    if ui.button("Cancel").clicked() {
                        cancel = true;
                    }
                });
            });

        if !open || cancel {
            self.close_snapshot_dialog();
        } else if recapture {
            self.prepare_snapshot();
        } else if save {
            self.save_prepared_snapshot(context);
        }
    }
}

impl eframe::App for QmpQemuSocketApp {
    /// Drains worker events and schedules the next UI update without blocking on QMP I/O.
    fn logic(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_worker(context);
        context.request_repaint_after(std::time::Duration::from_millis(100));
    }

    /// Renders the main controls and dialogs; EXIT requests worker detachment before closing.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            self.top_bar(ui);
            ui.separator();
            self.status_panel(ui);
            ui.separator();
            self.preview(ui);
            self.output(ui);
            ui.add_space(8.0);
            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                if bevel_button(ui, "EXIT", STATE_RED, false, true).clicked() {
                    self.push_log("EXIT requested; detaching locally and leaving QEMU untouched.");
                    let _ = self.worker.disconnect();
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
        });

        self.parameters_window(ui.ctx());
        self.snapshot_window(ui.ctx());
    }
}

/// Adds an enabled or disabled coloured button and returns its interaction response.
///
/// The bevel reverses when selected or pressed; painting does not alter click handling.
fn bevel_button(
    ui: &mut egui::Ui,
    label: &str,
    fill: Color32,
    selected: bool,
    enabled: bool,
) -> egui::Response {
    let button = egui::Button::new(egui::RichText::new(label).strong().color(Color32::WHITE))
        .fill(fill)
        .stroke(Stroke::new(1.0, fill));
    let response = ui.add_enabled(enabled, button);
    if enabled {
        let pressed = selected || response.is_pointer_button_down_on();
        let light = fill.gamma_multiply(1.7);
        let dark = fill.gamma_multiply(0.5);
        let (top_left, bottom_right) = if pressed {
            (dark, light)
        } else {
            (light, dark)
        };
        let rect = response.rect.shrink(1.0);
        let painter = ui.painter();
        painter.line_segment(
            [rect.left_bottom(), rect.left_top()],
            Stroke::new(1.5, top_left),
        );
        painter.line_segment(
            [rect.left_top(), rect.right_top()],
            Stroke::new(1.5, top_left),
        );
        painter.line_segment(
            [rect.right_top(), rect.right_bottom()],
            Stroke::new(1.5, bottom_right),
        );
        painter.line_segment(
            [rect.right_bottom(), rect.left_bottom()],
            Stroke::new(1.5, bottom_right),
        );
    }
    response
}

/// Returns true only after run ownership, worker activity and cancellation have all cleared.
fn controls_are_mutable(
    active_run: Option<ActiveRun>,
    state: WorkerState,
    stop_requested: bool,
) -> bool {
    active_run.is_none() && !state.is_busy() && !stop_requested
}

/// Checks nominal dimensions, texture availability and matching socket/mode authority.
///
/// STOP and diagnostic-only previews reject authority even if their other metadata matches.
fn preview_is_current(
    dimensions: Option<(u32, u32)>,
    preview_context: Option<&(PathBuf, GameMode)>,
    requested_context: &(PathBuf, GameMode),
    has_texture: bool,
    stop_requested: bool,
    diagnostic_preview: bool,
) -> bool {
    dimensions == Some((NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT))
        && preview_context == Some(requested_context)
        && has_texture
        && !stop_requested
        && !diagnostic_preview
}

/// Formats the session board advisory without asserting an unobserved initial board number.
fn board_position_label(
    mode: GameMode,
    current_board: usize,
    boards_per_game: usize,
    established: bool,
) -> String {
    if established {
        format!("{mode} — Board {current_board}/{boards_per_game} (session advisory)")
    } else {
        format!(
            "{mode} — Board ?/{boards_per_game} (session suggests {current_board}/{boards_per_game})"
        )
    }
}

/// Returns preview height in UI points after reserving footer and expanded-output space.
///
/// The height stays between the minimum viewport and native guest height plus
/// scrollbar allowance; collapsing detailed output removes its reservation.
fn preview_viewport_height(
    available_height: f32,
    pixels_per_point: f32,
    output_expanded: bool,
) -> f32 {
    let image_height = NOMINAL_FRAME_HEIGHT as f32 / pixels_per_point;
    let image_and_scrollbar =
        (image_height + PREVIEW_SCROLLBAR_ALLOWANCE_POINTS).max(MIN_PREVIEW_VIEWPORT_HEIGHT_POINTS);
    let reserved_output = if output_expanded {
        OUTPUT_PANEL_HEIGHT
    } else {
        0.0
    };
    (available_height - PREVIEW_FOOTER_RESERVE_POINTS - reserved_output)
        .max(MIN_PREVIEW_VIEWPORT_HEIGHT_POINTS)
        .min(image_and_scrollbar)
}

/// Formats a calibrated pixel rectangle for diagnostic and parameter displays.
fn format_rect(rect: PixelRect) -> String {
    format!(
        "x={}, y={}, width={}, height={}",
        rect.x, rect.y, rect.width, rect.height
    )
}

/// Converts a validated strided RGBA capture to the tightly packed egui layout.
///
/// Returns descriptive errors for invalid buffer layout or checked-size overflow;
/// row padding is omitted from the separate display buffer.
fn frame_to_colour_image(frame: &CapturedFrame) -> Result<egui::ColorImage, String> {
    if !frame.is_layout_valid() {
        return Err("pixel layout is invalid".to_owned());
    }

    let width = usize::try_from(frame.width)
        .map_err(|_| format!("frame width {} does not fit usize", frame.width))?;
    let height = usize::try_from(frame.height)
        .map_err(|_| format!("frame height {} does not fit usize", frame.height))?;
    let row_bytes = width
        .checked_mul(4)
        .ok_or_else(|| "RGBA row length overflowed usize".to_owned())?;
    let total_bytes = row_bytes
        .checked_mul(height)
        .ok_or_else(|| "RGBA frame length overflowed usize".to_owned())?;
    let mut rgba = Vec::with_capacity(total_bytes);

    for y in 0..height {
        let row_start = y
            .checked_mul(frame.stride)
            .ok_or_else(|| "source row offset overflowed usize".to_owned())?;
        let row_end = row_start
            .checked_add(row_bytes)
            .ok_or_else(|| "source row end overflowed usize".to_owned())?;
        let row = frame
            .pixels
            .get(row_start..row_end)
            .ok_or_else(|| "source row lies outside the pixel buffer".to_owned())?;

        match frame.format {
            PixelFormat::Rgba8 => rgba.extend_from_slice(row),
        }
    }

    Ok(egui::ColorImage::from_rgba_unmultiplied(
        [width, height],
        &rgba,
    ))
}

/// Describes detector evidence for the log without claiming an action was executed.
fn format_prediction(prediction: PredictedAction) -> String {
    match prediction {
        PredictedAction::CalibrationOnly { mode } => format!(
            "{mode} calibration capture ready; detector and guest input are disabled pending approved coordinates."
        ),
        PredictedAction::NoHighlight => {
            "WARNING: no calibrated Solver highlight is visible; no action proposed. This may indicate animation, a win, or an unexpected dialog."
                .to_owned()
        }
        PredictedAction::Action(action) => match (action.target, action.operation()) {
            (ActionTarget::Bottom { label, .. }, InputOperation::PressDrawKey) => format!(
                "Prediction: {label} with qcode D; gold anchor=({}, {}); input sent=0.",
                action.anchor.x, action.anchor.y
            ),
            (ActionTarget::Bottom { label, .. }, InputOperation::Click(click_point)) => format!(
                "Prediction: click {label} at ({}, {}); gold anchor=({}, {}); input sent=0.",
                click_point.x, click_point.y, action.anchor.x, action.anchor.y
            ),
            (ActionTarget::Tableau(position), InputOperation::Click(click_point)) => format!(
                "Prediction: click tableau row {}, column {} at ({}, {}); gold anchor=({}, {}); input sent=0.",
                position.row,
                position.column,
                click_point.x,
                click_point.y,
                action.anchor.x,
                action.anchor.y
            ),
            (ActionTarget::Tableau(position), InputOperation::PressDrawKey) => format!(
                "Prediction refused: tableau row {}, column {} has an invalid key-delivery specification; input sent=0.",
                position.row, position.column
            ),
            (ActionTarget::Pyramid(target), InputOperation::Click(click_point)) => format!(
                "Prediction: click {target} at ({}, {}); 2x2 HALO probe=({}, {}); input sent=0.",
                click_point.x, click_point.y, action.anchor.x, action.anchor.y
            ),
            (ActionTarget::Pyramid(target), InputOperation::PressDrawKey) => format!(
                "Prediction refused: {target} requires mouse delivery; input sent=0."
            ),
        },
        PredictedAction::Ambiguous { highlight_count } => format!(
            "Prediction refused: {highlight_count} calibrated highlights were found; input sent=0."
        ),
    }
}

/// Checks that one concrete prediction belongs to the authorised mode and input method.
fn prediction_is_actionable(mode: GameMode, prediction: PredictedAction) -> bool {
    let PredictedAction::Action(action) = prediction else {
        return false;
    };
    mode.input_authorised()
        && action.target.mode() == mode
        && match (action.target, action.operation()) {
            (ActionTarget::Bottom { .. }, _) => true,
            (_, InputOperation::Click(_)) => true,
            (_, InputOperation::PressDrawKey) => false,
        }
}

/// Formats a compact target description for request logs, retaining invalid-action warnings.
fn concise_prediction(prediction: PredictedAction) -> String {
    match prediction {
        PredictedAction::CalibrationOnly { mode } => format!("{mode} calibration only"),
        PredictedAction::NoHighlight => "no highlight".to_owned(),
        PredictedAction::Action(action) => match (action.target, action.operation()) {
            (ActionTarget::Bottom { label, .. }, InputOperation::PressDrawKey) => format!(
                "{label} at gold anchor ({}, {})",
                action.anchor.x, action.anchor.y
            ),
            (ActionTarget::Bottom { label, .. }, InputOperation::Click(point)) => {
                format!("{label} at ({}, {})", point.x, point.y)
            }
            (ActionTarget::Tableau(position), InputOperation::Click(point)) => format!(
                "tableau r{}c{} at ({}, {})",
                position.row, position.column, point.x, point.y
            ),
            (ActionTarget::Tableau(position), InputOperation::PressDrawKey) => {
                format!(
                    "invalid tableau r{}c{} key action",
                    position.row, position.column
                )
            }
            (ActionTarget::Pyramid(target), InputOperation::Click(point)) => {
                format!("{target} at ({}, {})", point.x, point.y)
            }
            (ActionTarget::Pyramid(target), InputOperation::PressDrawKey) => {
                format!("invalid {target} key action")
            }
        },
        PredictedAction::Ambiguous { highlight_count } => {
            format!("ambiguous ({highlight_count} highlights)")
        }
    }
}

/// Formats the next-action label for the UI, including calibration and ambiguous states.
fn display_action(prediction: PredictedAction) -> String {
    match prediction {
        PredictedAction::CalibrationOnly { mode } => format!("{mode} calibration — input disabled"),
        PredictedAction::NoHighlight => "No Solver HALO".to_owned(),
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
            (ActionTarget::Pyramid(target), InputOperation::Click(_)) => {
                format!("{target} click")
            }
            (ActionTarget::Pyramid(target), InputOperation::PressDrawKey) => {
                format!("Invalid {target} key action")
            }
        },
        PredictedAction::Ambiguous { highlight_count } => {
            format!("Ambiguous — {highlight_count} HALOs")
        }
    }
}

/// Names a fixed operation limit or the configured continuous-run sentinel.
fn format_operation_limit(operation_limit: usize) -> String {
    if operation_limit == UNBOUNDED_MULTI_STEP_ACTIONS {
        "continuous until STOP".to_owned()
    } else {
        format!("{operation_limit}-operation")
    }
}

/// Formats an operation index against its fixed limit or an unbounded denominator.
fn format_operation_progress(operation_index: usize, operation_limit: usize) -> String {
    if operation_limit == UNBOUNDED_MULTI_STEP_ACTIONS {
        format!("{operation_index}/unbounded")
    } else {
        format!("{operation_index}/{operation_limit}")
    }
}

/// Maps a non-empty preview canvas position into half-open nominal guest coordinates.
///
/// Clamps edge positions to the final pixel and compensates for f32 round-trip drift.
fn preview_to_pixel(canvas: Rect, position: Pos2) -> PixelPoint {
    let relative_x = ((position.x - canvas.left()) / canvas.width()).clamp(0.0, 1.0);
    let relative_y = ((position.y - canvas.top()) / canvas.height()).clamp(0.0, 1.0);
    // Compensate only sub-thousandth-pixel f32 round-trip drift at exact
    // boundaries; QMP's integer pixel-to-tablet conversion remains unchanged.
    let x = (((relative_x * NOMINAL_FRAME_WIDTH as f32 + PREVIEW_MAPPING_FLOAT_EPSILON).floor()
        as u32)
        .min(NOMINAL_FRAME_WIDTH - 1)) as i32;
    let y = ((relative_y * NOMINAL_FRAME_HEIGHT as f32 + PREVIEW_MAPPING_FLOAT_EPSILON).floor()
        as u32)
        .min(NOMINAL_FRAME_HEIGHT - 1) as i32;
    PixelPoint::new(x, y)
}

/// Maps a nominal guest pixel into UI points within the supplied preview canvas.
fn pixel_to_preview(canvas: Rect, point: PixelPoint) -> Pos2 {
    let x = canvas.left() + point.x as f32 * canvas.width() / NOMINAL_FRAME_WIDTH as f32;
    let y = canvas.top() + point.y as f32 * canvas.height() / NOMINAL_FRAME_HEIGHT as f32;
    Pos2::new(x, y)
}

/// Maps both edges of a nominal guest rectangle into preview UI coordinates.
fn rect_to_preview(canvas: Rect, pixel_rect: PixelRect) -> Rect {
    Rect::from_min_max(
        pixel_to_preview(
            canvas,
            PixelPoint::new(pixel_rect.x as i32, pixel_rect.y as i32),
        ),
        pixel_to_preview(
            canvas,
            PixelPoint::new(pixel_rect.right() as i32, pixel_rect.bottom() as i32),
        ),
    )
}

/// Paints a labelled calibrated rectangle without modifying captured frame pixels.
fn paint_target(
    painter: &egui::Painter,
    canvas: Rect,
    pixel_rect: PixelRect,
    colour: Color32,
    label: &str,
) {
    let target = rect_to_preview(canvas, pixel_rect);
    paint_outline(painter, target, Stroke::new(2.0, colour));
    painter.text(
        target.left_top() + egui::vec2(5.0, 5.0),
        Align2::LEFT_TOP,
        label,
        FontId::monospace(13.0),
        colour,
    );
}

/// Paints read-only detector evidence and proposed click/key markers on the preview.
fn paint_prediction(painter: &egui::Painter, canvas: Rect, prediction: PredictedAction) {
    let gold = Color32::from_rgb(245, 205, 75);
    let proposal = Color32::from_rgb(255, 80, 210);

    match prediction {
        PredictedAction::CalibrationOnly { mode } => {
            painter.text(
                canvas.center_top() + egui::vec2(0.0, 8.0),
                Align2::CENTER_TOP,
                format!("{mode} CALIBRATION — INPUT DISABLED"),
                FontId::monospace(14.0),
                Color32::YELLOW,
            );
        }
        PredictedAction::NoHighlight => {
            painter.text(
                canvas.right_top() + egui::vec2(-8.0, 8.0),
                Align2::RIGHT_TOP,
                "no Solver highlight",
                FontId::monospace(13.0),
                Color32::LIGHT_GRAY,
            );
        }
        PredictedAction::Action(action) => {
            let anchor_label = match action.operation() {
                InputOperation::PressDrawKey => "gold anchor: DRAW (D)",
                InputOperation::Click(_) => "gold anchor",
            };
            paint_labeled_marker(painter, canvas, action.anchor, gold, anchor_label);
            if let InputOperation::Click(click_point) = action.operation() {
                let label = match action.target {
                    ActionTarget::Bottom { label, .. } => format!("proposed {label}"),
                    ActionTarget::Tableau(position) => {
                        format!("proposed r{}c{}", position.row, position.column)
                    }
                    ActionTarget::Pyramid(target) => format!("proposed {target}"),
                };
                paint_labeled_crosshair(painter, canvas, click_point, proposal, &label);
            }
        }
        PredictedAction::Ambiguous { highlight_count } => {
            painter.text(
                canvas.center_top() + egui::vec2(0.0, 8.0),
                Align2::CENTER_TOP,
                format!("AMBIGUOUS: {highlight_count} highlights"),
                FontId::monospace(14.0),
                Color32::LIGHT_RED,
            );
        }
    }
}

/// Paints a small circular marker and adjacent text at a nominal guest pixel.
fn paint_labeled_marker(
    painter: &egui::Painter,
    canvas: Rect,
    point: PixelPoint,
    colour: Color32,
    label: &str,
) {
    let centre = pixel_to_preview(canvas, point);
    let stroke = Stroke::new(2.0, colour);
    painter.circle_stroke(centre, 5.0, stroke);
    painter.text(
        centre + egui::vec2(7.0, -7.0),
        Align2::LEFT_BOTTOM,
        label,
        FontId::monospace(12.0),
        colour,
    );
}

/// Paints a short labelled crosshair at a nominal guest click point.
fn paint_labeled_crosshair(
    painter: &egui::Painter,
    canvas: Rect,
    point: PixelPoint,
    colour: Color32,
    label: &str,
) {
    let centre = pixel_to_preview(canvas, point);
    let stroke = Stroke::new(2.0, colour);
    painter.line_segment(
        [centre - egui::vec2(9.0, 0.0), centre + egui::vec2(9.0, 0.0)],
        stroke,
    );
    painter.line_segment(
        [centre - egui::vec2(0.0, 9.0), centre + egui::vec2(0.0, 9.0)],
        stroke,
    );
    painter.text(
        centre + egui::vec2(10.0, -10.0),
        Align2::LEFT_BOTTOM,
        label,
        FontId::monospace(12.0),
        colour,
    );
}

/// Paints the four edges of a rectangle using the supplied stroke.
fn paint_outline(painter: &egui::Painter, rect: Rect, stroke: Stroke) {
    painter.line_segment([rect.left_top(), rect.right_top()], stroke);
    painter.line_segment([rect.right_top(), rect.right_bottom()], stroke);
    painter.line_segment([rect.right_bottom(), rect.left_bottom()], stroke);
    painter.line_segment([rect.left_bottom(), rect.left_top()], stroke);
}

/// Paints a subdued calibration grid when no capture texture is available.
fn paint_grid(painter: &egui::Painter, canvas: Rect) {
    let stroke = Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 14));
    for division in 1..8 {
        let fraction = division as f32 / 8.0;
        let x = canvas.left() + canvas.width() * fraction;
        painter.line_segment(
            [Pos2::new(x, canvas.top()), Pos2::new(x, canvas.bottom())],
            stroke,
        );
    }
    for division in 1..4 {
        let fraction = division as f32 / 4.0;
        let y = canvas.top() + canvas.height() * fraction;
        painter.line_segment(
            [Pos2::new(canvas.left(), y), Pos2::new(canvas.right(), y)],
            stroke,
        );
    }
}

/// Paints horizontal and vertical alignment guides confined to the preview canvas.
fn paint_crosshair(painter: &egui::Painter, canvas: Rect, point: PixelPoint) {
    let centre = pixel_to_preview(canvas, point);
    let stroke = Stroke::new(1.5, Color32::WHITE);
    painter.line_segment(
        [
            Pos2::new(canvas.left(), centre.y),
            Pos2::new(canvas.right(), centre.y),
        ],
        stroke,
    );
    painter.line_segment(
        [
            Pos2::new(centre.x, canvas.top()),
            Pos2::new(centre.x, canvas.bottom()),
        ],
        stroke,
    );
}

#[cfg(test)]
mod tests {
    //! UI policy and coordinate-mapping regressions independent of a live window.

    use super::*;

    /// Constructs a translated canvas at nominal size to exercise coordinate conversions.
    fn test_canvas() -> Rect {
        Rect::from_min_size(
            Pos2::new(25.0, 40.0),
            egui::vec2(NOMINAL_FRAME_WIDTH as f32, NOMINAL_FRAME_HEIGHT as f32),
        )
    }

    /// Checks that calibrated guest points survive a pixel-to-preview-to-pixel round trip.
    #[test]
    fn preview_mapping_round_trips_calibrated_points() {
        let canvas = test_canvas();
        for point in [
            PixelPoint::new(0, 0),
            PixelPoint::new(842, 870),
            PixelPoint::new(1_662, 630),
            PixelPoint::new(1_723, 533),
            PixelPoint::new(1_919, 1_079),
        ] {
            assert_eq!(
                preview_to_pixel(canvas, pixel_to_preview(canvas, point)),
                point
            );
        }
    }

    /// Checks pixel round trips across fractional and integer host display scales.
    #[test]
    fn native_preview_mapping_round_trips_at_common_wayland_scales() {
        for scale in [1.0_f32, 1.25, 1.5, 1.75, 2.0] {
            let canvas = Rect::from_min_size(
                Pos2::new(25.0, 40.0),
                egui::vec2(
                    NOMINAL_FRAME_WIDTH as f32 / scale,
                    NOMINAL_FRAME_HEIGHT as f32 / scale,
                ),
            );
            for x in [0, 1, 60, 842, 1_320, 1_680, 1_919] {
                for y in [0, 1, 84, 540, 977, 1_079] {
                    let pixel = PixelPoint::new(x, y);
                    assert_eq!(
                        preview_to_pixel(canvas, pixel_to_preview(canvas, pixel)),
                        pixel
                    );
                }
            }
        }
    }

    /// Checks native-height limits and the reclaimed preview space when output is collapsed.
    #[test]
    fn preview_uses_the_full_guest_height_when_the_host_has_room() {
        assert_eq!(preview_viewport_height(1_400.0, 1.0, false), 1_104.0);
        assert_eq!(preview_viewport_height(900.0, 1.0, false), 804.0);
        assert_eq!(preview_viewport_height(1_400.0, 2.0, false), 564.0);
        assert_eq!(preview_viewport_height(900.0, 1.0, true), 519.0);
        assert_eq!(preview_viewport_height(1_400.0, 2.0, true), 564.0);
    }

    /// Checks that the inclusive canvas corner maps to the last valid guest pixel.
    #[test]
    fn preview_mapping_clamps_the_bottom_right_edges() {
        let canvas = test_canvas();

        assert_eq!(
            preview_to_pixel(canvas, canvas.right_bottom()),
            PixelPoint::new(1_919, 1_079)
        );
    }

    /// Checks mode ownership and input-method gating for actionable preview predictions.
    #[test]
    fn only_concrete_single_predictions_enable_step_once() {
        assert!(!prediction_is_actionable(
            GameMode::TriPeaks,
            PredictedAction::NoHighlight
        ));
        assert!(!prediction_is_actionable(
            GameMode::TriPeaks,
            PredictedAction::Ambiguous { highlight_count: 2 }
        ));
        assert!(!prediction_is_actionable(
            GameMode::Pyramid,
            PredictedAction::CalibrationOnly {
                mode: GameMode::Pyramid
            }
        ));
        let profile = GameMode::TriPeaks.profile();
        let draw =
            PredictedAction::Action(profile.bottom_action(0, PixelPoint::new(842, 870)).unwrap());
        let tableau = PredictedAction::Action(
            profile
                .tableau_action(27, PixelPoint::new(1_662, 630))
                .unwrap(),
        );
        assert!(prediction_is_actionable(GameMode::TriPeaks, draw));
        assert!(prediction_is_actionable(GameMode::TriPeaks, tableau));
        assert!(!prediction_is_actionable(GameMode::Pyramid, draw));
        assert!(!prediction_is_actionable(GameMode::Pyramid, tableau));
        let pyramid_action =
            crate::pyramid::action_for_kind(crate::pyramid::PyramidTargetKind::Move).unwrap();
        let pyramid = PredictedAction::Action(pyramid_action);
        assert!(prediction_is_actionable(GameMode::Pyramid, pyramid));
        assert!(!prediction_is_actionable(GameMode::TriPeaks, pyramid));
        let mut invalid_key_action = pyramid_action;
        invalid_key_action.specification.operation = InputOperation::PressDrawKey;
        assert!(!prediction_is_actionable(
            GameMode::Pyramid,
            PredictedAction::Action(invalid_key_action),
        ));
    }

    /// Checks that a zero operation limit is presented as continuous rather than empty.
    #[test]
    fn operation_labels_distinguish_bounded_and_unbounded_runs() {
        assert_eq!(format_operation_limit(0), "continuous until STOP");
        assert_eq!(format_operation_limit(25), "25-operation");
        assert_eq!(format_operation_progress(7, 0), "7/unbounded");
        assert_eq!(format_operation_progress(7, 25), "7/25");
    }

    /// Checks that stale, diagnostic, missing and cancelled previews cannot authorise input.
    #[test]
    fn preview_authority_is_revoked_by_mode_socket_size_or_stop() {
        let tri = (PathBuf::from("/tmp/current.sock"), GameMode::TriPeaks);
        let pyramid = (PathBuf::from("/tmp/current.sock"), GameMode::Pyramid);
        let old_socket = (PathBuf::from("/tmp/old.sock"), GameMode::TriPeaks);
        let size = Some((NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT));
        assert!(preview_is_current(
            size,
            Some(&tri),
            &tri,
            true,
            false,
            false
        ));
        assert!(!preview_is_current(
            size,
            Some(&tri),
            &tri,
            true,
            false,
            true
        ));
        assert!(!preview_is_current(
            size,
            Some(&tri),
            &pyramid,
            true,
            false,
            false
        ));
        assert!(!preview_is_current(
            size,
            Some(&old_socket),
            &tri,
            true,
            false,
            false,
        ));
        assert!(!preview_is_current(
            Some((640, 480)),
            Some(&tri),
            &tri,
            true,
            false,
            false,
        ));
        assert!(!preview_is_current(
            size,
            Some(&tri),
            &tri,
            false,
            false,
            false
        ));
        assert!(!preview_is_current(
            size,
            Some(&tri),
            &tri,
            true,
            true,
            false
        ));
    }

    /// Checks that worker readiness alone cannot clear run ownership or pending cancellation.
    #[test]
    fn run_controls_are_disabled_until_the_run_is_really_stopped() {
        assert!(controls_are_mutable(None, WorkerState::Ready, false));
        assert!(!controls_are_mutable(
            Some(ActiveRun::MultiStep),
            WorkerState::Ready,
            false
        ));
        assert!(!controls_are_mutable(
            Some(ActiveRun::StepOnce),
            WorkerState::Acting,
            false
        ));
        assert!(!controls_are_mutable(None, WorkerState::Ready, true));
    }

    /// Checks that a board position remains explicitly uncertain until a transition establishes it.
    #[test]
    fn board_label_does_not_assert_a_visual_third_from_session_start() {
        assert_eq!(
            board_position_label(GameMode::TriPeaks, 1, 3, false),
            "TriPeaks — Board ?/3 (session suggests 1/3)"
        );
        assert_eq!(
            board_position_label(GameMode::TriPeaks, 2, 3, true),
            "TriPeaks — Board 2/3 (session advisory)"
        );
    }
}
