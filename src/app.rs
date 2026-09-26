use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    os::unix::fs::OpenOptionsExt,
    path::PathBuf,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use eframe::egui::{
    self, Align, Align2, Color32, FontId, Layout, Pos2, Rect, Sense, Stroke, TextBuffer,
};
use egui_winit::clipboard::Clipboard;
use raw_window_handle::HasDisplayHandle;

use crate::{
    capture::{CapturedFrame, PixelFormat},
    game::{ActionTarget, GameMode, InputOperation},
    geometry::{PixelPoint, PixelRect},
    pyramid::{PYRAMID_TARGETS, TABLEAU_CARD_COUNT as PYRAMID_TABLEAU_CARD_COUNT},
    snapshot::SnapshotArtifact,
    tracker::PredictedAction,
    worker::{WorkerEvent, WorkerHandle, WorkerState},
};

use crate::parameters::{
    ACTION_CHANGE_CHANNEL_THRESHOLD, ACTION_CURSOR_EXCLUSION_HALF_SIZE,
    BOARD_REDEAL_SETTLE_DELAY_MS, CHALLENGE_COMPLETE_CONTINUE_CONTROL, DEFAULT_MULTI_STEP_ACTIONS,
    DRAW_ANIMATION_SETTLE_DELAY_MS, GOLD_CHANNEL_TOLERANCE, GOLD_RGB_CANDIDATES, KEY_HOLD,
    LEVEL_UP_APPEAR_DELAY, MAX_LOG_LINES,
    MAXIMUM_ANIMATION_SETTLE_DELAY_MS, MIN_PREVIEW_VIEWPORT_HEIGHT_POINTS,
    MINIMUM_ANIMATION_SETTLE_DELAY_MS, MINIMUM_DRAW_CHANGED_PIXELS, MINIMUM_TABLEAU_CHANGED_PIXELS,
    MOUSE_HOLD, MULTI_STEP_INPUT_ENABLED, NO_HIGHLIGHT_REOBSERVE_DELAY, NOMINAL_FRAME_HEIGHT,
    NOMINAL_FRAME_WIDTH, OUTPUT_PANEL_HEIGHT, POINTER_SETTLE_DELAY, POST_GAME_MAX_CLICK_ATTEMPTS,
    POST_GAME_MAX_OBSERVATION_ROUNDS, POST_GAME_STAGE_DELAY, POST_GAME_TARGETS,
    PREVIEW_FOOTER_RESERVE_POINTS, PREVIEW_SCROLLBAR_ALLOWANCE_POINTS, RELEASE_LABEL,
    SCORE_SKIP_MAX_CLICK_ATTEMPTS, SESSION_LOG_FILE_PREFIX, SESSION_LOG_FILE_SUFFIX,
    SESSION_LOG_MODE, SESSION_LOG_NAME_ATTEMPTS, SNAPSHOT_LABEL_MAX_CHARS, SHARED_TOOLBAR_CONTROLS,
    STEP_ONCE_ACTIONS, STEP_ONCE_INPUT_ENABLED, TABLEAU_ANIMATION_SETTLE_DELAY_MS,
    UNBOUNDED_MULTI_STEP_ACTIONS, VISIBLE_LOG_ROLLOVER_GAMES,
};
use crate::parameters::{AnimationSettleDelays, StepRunSettings};
use crate::parameters::{default_qmp_socket_path, session_log_directory, snapshot_directory};

struct SessionLog {
    path: PathBuf,
    file: File,
}

const SETUP_BLUE: Color32 = Color32::from_rgb(27, 96, 157);
const ACTION_GREEN: Color32 = Color32::from_rgb(35, 112, 43);
const STATE_RED: Color32 = Color32::from_rgb(172, 35, 38);
const PREVIEW_MAPPING_FLOAT_EPSILON: f32 = 0.0005;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ActiveRun {
    StepOnce,
    MultiStep,
}

enum SnapshotEditAction {
    Cut,
    Copy,
    Paste,
}

impl SessionLog {
    fn create() -> io::Result<Self> {
        // Creates a private, timestamped backing file for the complete session.
        // The visible panel can then roll over without losing diagnostic history.
        let directory = session_log_directory();
        fs::create_dir_all(&directory)?;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_millis();

        for attempt in 0..SESSION_LOG_NAME_ATTEMPTS {
            let collision_suffix = if attempt == 0 {
                String::new()
            } else {
                format!("-{attempt}")
            };
            let path = directory.join(format!(
                "{SESSION_LOG_FILE_PREFIX}{timestamp}{collision_suffix}{SESSION_LOG_FILE_SUFFIX}"
            ));
            match OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(SESSION_LOG_MODE)
                .open(&path)
            {
                Ok(file) => return Ok(Self { path, file }),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "could not reserve a unique session log name",
        ))
    }

    fn append(&mut self, line: &str) -> io::Result<()> {
        writeln!(self.file, "{line}")
    }

    fn complete_text(&mut self) -> io::Result<String> {
        self.file.flush()?;
        fs::read_to_string(&self.path)
    }
}

pub struct QmpQemuSocketApp {
    worker: WorkerHandle,
    worker_state: WorkerState,
    game_mode: GameMode,
    qmp_socket_path: String,
    log_lines: Vec<String>,
    session_log: Option<SessionLog>,
    completed_games: usize,
    started_at: Instant,
    show_coordinates: bool,
    draw_targets: bool,
    show_parameters: bool,
    show_snapshot_dialog: bool,
    snapshot_label: String,
    snapshot_request_id: u64,
    snapshot_capture_pending: bool,
    snapshot_focus_pending: bool,
    snapshot_label_selection: Option<egui::text::CCursorRange>,
    snapshot_error: Option<String>,
    snapshot_png: Option<Vec<u8>>,
    snapshot_frame: Option<CapturedFrame>,
    snapshot_prediction: Option<PredictedAction>,
    snapshot_context: Option<(PathBuf, GameMode)>,
    snapshot_texture: Option<egui::TextureHandle>,
    clipboard: Clipboard,
    capture_texture: Option<egui::TextureHandle>,
    captured_dimensions: Option<(u32, u32)>,
    preview_context: Option<(PathBuf, GameMode)>,
    prediction: Option<PredictedAction>,
    draw_animation_settle_ms: u64,
    tableau_animation_settle_ms: u64,
    board_redeal_settle_ms: u64,
    multi_step_actions: usize,
    active_run: Option<ActiveRun>,
    stop_requested: bool,
    current_board: usize,
    boards_per_game: usize,
    board_position_established: bool,
    current_status: String,
}

impl QmpQemuSocketApp {
    pub fn new(creation_context: &eframe::CreationContext<'_>) -> Self {
        // Creates the initial UI state and starts its background worker.
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
            draw_animation_settle_ms: DRAW_ANIMATION_SETTLE_DELAY_MS,
            tableau_animation_settle_ms: TABLEAU_ANIMATION_SETTLE_DELAY_MS,
            board_redeal_settle_ms: BOARD_REDEAL_SETTLE_DELAY_MS,
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
        if let Some(path) = app.session_log.as_ref().map(|log| log.path.clone()) {
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
        app.push_log("Step Once and Multi-Step freshly validate the initial displayed prediction, reuse each verified result frame as the next plan, and never retry uncertain guest input.");
        app.push_log(format!(
            "Execution defaults: one initial planning frame plus one fresh verified result frame per action, Multi-Step limit {} (0 means continuous). STOP is visible only while a guarded run is active.",
            DEFAULT_MULTI_STEP_ACTIONS,
        ));
        app.request_capture("Startup");
        app
    }

    fn push_log(&mut self, message: impl Into<String>) {
        // Appends every line to the private session file and keeps only a
        // bounded working set in the rendered panel.
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

    fn push_visible_log_line(&mut self, line: String) {
        if self.log_lines.len() >= MAX_LOG_LINES {
            let remove_count = self.log_lines.len() + 1 - MAX_LOG_LINES;
            self.log_lines.drain(0..remove_count);
        }
        self.log_lines.push(line);
    }

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

    fn complete_output(&mut self) -> Result<String, String> {
        match self.session_log.as_mut() {
            Some(log) => log
                .complete_text()
                .map_err(|error| format!("Could not read the complete session output: {error}")),
            None => Ok(self.log_lines.join("\n")),
        }
    }

    fn roll_visible_output_after_completed_game(&mut self) {
        // Counts successfully restarted games and periodically
        // clears only the rendered panel while preserving the backing file.
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

    fn poll_worker(&mut self, context: &egui::Context) {
        // Applies pending worker state and log events to the UI state.
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
                                self.snapshot_error = Some(format!(
                                    "Could not preview the captured PNG: {error}"
                                ));
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
                } => {
                    self.current_status = display_action(before);
                    let progress = format_operation_progress(operation_index, operation_limit);
                    self.push_log(format!(
                        "Operation {progress} verified: {} -> {}; changed effect pixels={changed_pixels}; QMP commands={input_commands}, input events={input_events}, guest-input retries=0.",
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
                } => {
                    self.active_run = None;
                    self.current_status = "Ready".to_owned();
                    let requested = format_operation_limit(requested_operations);
                    self.push_log(format!(
                        "Execution run completed: verified {verified_operations} operation(s) from a {requested} request; final prediction={}; guest-input retries=0.",
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
                    WorkerState::Detached | WorkerState::Error | WorkerState::Uncertain
                )
            {
                self.install_frame(
                    context,
                    latest.frame,
                    latest.prediction,
                    latest.log_prediction,
                    latest.context,
                );
            } else {
                self.push_log("Discarded a preview from a different mode/socket or a detached/uncertain worker state.");
            }
        }
    }

    fn install_frame(
        &mut self,
        context: &egui::Context,
        frame: CapturedFrame,
        prediction: PredictedAction,
        log_prediction: bool,
        frame_context: Option<(PathBuf, GameMode)>,
    ) {
        // Uploads a worker-owned frame to egui only after validating its layout.
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

    fn current_preview_context(&self) -> (PathBuf, GameMode) {
        (PathBuf::from(self.qmp_socket_path.trim()), self.game_mode)
    }

    fn has_current_preview(&self) -> bool {
        preview_is_current(
            self.captured_dimensions,
            self.preview_context.as_ref(),
            &self.current_preview_context(),
            self.capture_texture.is_some(),
            self.stop_requested,
        )
    }

    fn invalidate_preview(&mut self) {
        self.capture_texture = None;
        self.captured_dimensions = None;
        self.preview_context = None;
        self.prediction = None;
    }

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
        if let Err(error) = self
            .worker
            .prepare_snapshot(socket_path, mode, self.snapshot_request_id)
        {
            self.worker_state = WorkerState::Error;
            self.snapshot_capture_pending = false;
            self.snapshot_error = Some(error.clone());
            self.current_status = "Snapshot capture failed".to_owned();
            self.push_log(error);
        }
    }

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

    fn dispatch_steps(
        &mut self,
        operation_limit: usize,
        request_name: &str,
        active_run: ActiveRun,
    ) {
        // Snapshots all execution controls before handing the run to the worker.
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
        let Some(approved_prediction) = self.prediction.filter(|_| self.has_current_preview())
        else {
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
            "{request_name} requested {request_scope}, starting from approved preview target: {}. The initial target receives one fresh validation capture; each verified result frame then becomes the next planning frame.",
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
                        "Run the configured bounded number of verified actions"
                    })
                    .clicked()
                {
                    self.dispatch_steps(self.multi_step_actions, "Multiple Steps", ActiveRun::MultiStep);
                }
            }
        });
    }

    fn preview(&mut self, ui: &mut egui::Ui) {
        // Renders the latest QMP capture and reports guest-pixel coordinates.
        ui.heading("QMP capture and target preview");
        match self.captured_dimensions {
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
        let viewport_height = preview_viewport_height(ui.available_height(), ui.pixels_per_point());
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
                        paint_labeled_crosshair(
                            &painter,
                            canvas,
                            control.click_point,
                            colour,
                            "",
                        );
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

    fn output(&mut self, ui: &mut egui::Ui) {
        // Keeps complete diagnostics available without making them the primary
        // operational display.
        ui.separator();
        egui::CollapsingHeader::new("Detailed output")
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
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            for line in &self.log_lines {
                                ui.label(egui::RichText::new(line).monospace());
                            }
                        });
                });
            });
    }

    fn status_panel(&self, ui: &mut egui::Ui) {
        // Operational state remains short and stable while the detailed trace
        // continues in the bounded panel and private session file.
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

    fn parameters_window(&mut self, context: &egui::Context) {
        // Shows the editable QMP path and current tuning parameters when requested.
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
                ui.add_enabled_ui(execution_enabled, |ui| {
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
                        self.multi_step_actions = DEFAULT_MULTI_STEP_ACTIONS;
                    }
                });
                ui.small(format!(
                    "Execution controls are session-only; defaults: draw {DRAW_ANIMATION_SETTLE_DELAY_MS} ms, tableau {TABLEAU_ANIMATION_SETTLE_DELAY_MS} ms, board redeal {BOARD_REDEAL_SETTLE_DELAY_MS} ms, Multi-Step {DEFAULT_MULTI_STEP_ACTIONS} (continuous)."
                ));
                if let Some(log) = self.session_log.as_ref() {
                    ui.small(format!("Complete output: {}", log.path.display()));
                }
                ui.small(
                    "Current verification: one initial fresh planning capture, then each fresh verified result frame is reused for the next plan. Material effect verification remains mandatory. STOP cancels the active run.",
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
                if self.game_mode.calibration_only() {
                    let pyramid_cards = PYRAMID_TARGETS
                        .iter()
                        .filter(|target| target.kind.is_card())
                        .count();
                    ui.monospace("guest-input-authorised = false");
                    ui.monospace("target-priority = Move, Left, Right, Card");
                    ui.monospace(format!(
                        "target-slots = {} total, {pyramid_cards} cards (expected {PYRAMID_TABLEAU_CARD_COUNT})",
                        PYRAMID_TARGETS.len(),
                    ));
                    ui.monospace("halo-probes = not validated; detection disabled");
                    ui.monospace("click-points = screenshot geometry; input disabled");
                    for target in &PYRAMID_TARGETS {
                        ui.monospace(format!(
                            "{}: bounds {}, hit x={}, y={}",
                            target.label,
                            format_rect(target.bounds),
                            target.click_point.x,
                            target.click_point.y,
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
                ui.monospace(format!("gold-colours = {GOLD_RGB_CANDIDATES:?}"));
                ui.monospace(format!(
                    "gold-channel-tolerance = ±{GOLD_CHANNEL_TOLERANCE}"
                ));
                ui.monospace(format!(
                    "no-highlight-reobserve = until target or STOP, {} ms apart",
                    NO_HIGHLIGHT_REOBSERVE_DELAY.as_millis()
                ));
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
                ui.monospace(format!(
                    "action-change = channel >= {ACTION_CHANGE_CHANNEL_THRESHOLD}, draw pixels >= {MINIMUM_DRAW_CHANGED_PIXELS}, tableau pixels >= {MINIMUM_TABLEAU_CHANGED_PIXELS}"
                ));
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
    fn logic(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        // Polls worker events and schedules regular UI repainting.
        self.poll_worker(context);
        context.request_repaint_after(std::time::Duration::from_millis(100));
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Renders the main application panel and optional parameters window.
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

fn controls_are_mutable(
    active_run: Option<ActiveRun>,
    state: WorkerState,
    stop_requested: bool,
) -> bool {
    active_run.is_none() && !state.is_busy() && !stop_requested
}

fn preview_is_current(
    dimensions: Option<(u32, u32)>,
    preview_context: Option<&(PathBuf, GameMode)>,
    requested_context: &(PathBuf, GameMode),
    has_texture: bool,
    stop_requested: bool,
) -> bool {
    dimensions == Some((NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT))
        && preview_context == Some(requested_context)
        && has_texture
        && !stop_requested
}

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

fn preview_viewport_height(available_height: f32, pixels_per_point: f32) -> f32 {
    let image_height = NOMINAL_FRAME_HEIGHT as f32 / pixels_per_point;
    let image_and_scrollbar =
        (image_height + PREVIEW_SCROLLBAR_ALLOWANCE_POINTS).max(MIN_PREVIEW_VIEWPORT_HEIGHT_POINTS);
    (available_height - PREVIEW_FOOTER_RESERVE_POINTS)
        .max(MIN_PREVIEW_VIEWPORT_HEIGHT_POINTS)
        .min(image_and_scrollbar)
}

fn format_rect(rect: PixelRect) -> String {
    // Formats a pixel rectangle for log and parameter displays.
    format!(
        "x={}, y={}, width={}, height={}",
        rect.x, rect.y, rect.width, rect.height
    )
}

fn frame_to_colour_image(frame: &CapturedFrame) -> Result<egui::ColorImage, String> {
    // Converts a validated strided capture into the tightly packed RGBA layout egui expects.
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
            PixelFormat::Bgra8 => {
                for pixel in row.chunks_exact(4) {
                    rgba.extend_from_slice(&[pixel[2], pixel[1], pixel[0], pixel[3]]);
                }
            }
        }
    }

    Ok(egui::ColorImage::from_rgba_unmultiplied(
        [width, height],
        &rgba,
    ))
}

fn format_prediction(prediction: PredictedAction) -> String {
    // Summarises one read-only detector result without implying that it was executed.
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
        },
        PredictedAction::Ambiguous { highlight_count } => format!(
            "Prediction refused: {highlight_count} calibrated highlights were found; input sent=0."
        ),
    }
}

fn prediction_is_actionable(mode: GameMode, prediction: PredictedAction) -> bool {
    mode.input_authorised() && matches!(prediction, PredictedAction::Action(_))
}

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
        },
        PredictedAction::Ambiguous { highlight_count } => {
            format!("ambiguous ({highlight_count} highlights)")
        }
    }
}

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
        },
        PredictedAction::Ambiguous { highlight_count } => {
            format!("Ambiguous — {highlight_count} HALOs")
        }
    }
}

fn format_operation_limit(operation_limit: usize) -> String {
    if operation_limit == UNBOUNDED_MULTI_STEP_ACTIONS {
        "continuous until STOP".to_owned()
    } else {
        format!("{operation_limit}-operation")
    }
}

fn format_operation_progress(operation_index: usize, operation_limit: usize) -> String {
    if operation_limit == UNBOUNDED_MULTI_STEP_ACTIONS {
        format!("{operation_index}/unbounded")
    } else {
        format!("{operation_index}/{operation_limit}")
    }
}

fn preview_to_pixel(canvas: Rect, position: Pos2) -> PixelPoint {
    // Maps a preview position to half-open nominal-frame pixel coordinates.
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

fn pixel_to_preview(canvas: Rect, point: PixelPoint) -> Pos2 {
    // Maps nominal-frame pixel coordinates to a position on the preview canvas.
    let x = canvas.left() + point.x as f32 * canvas.width() / NOMINAL_FRAME_WIDTH as f32;
    let y = canvas.top() + point.y as f32 * canvas.height() / NOMINAL_FRAME_HEIGHT as f32;
    Pos2::new(x, y)
}

fn rect_to_preview(canvas: Rect, pixel_rect: PixelRect) -> Rect {
    // Maps a nominal-frame pixel rectangle onto the preview canvas.
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

fn paint_target(
    painter: &egui::Painter,
    canvas: Rect,
    pixel_rect: PixelRect,
    colour: Color32,
    label: &str,
) {
    // Draws a labelled target rectangle on the preview canvas.
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

fn paint_prediction(painter: &egui::Painter, canvas: Rect, prediction: PredictedAction) {
    // Draws detector evidence and the proposed action while remaining read-only.
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

fn paint_outline(painter: &egui::Painter, rect: Rect, stroke: Stroke) {
    // Draws the four edges of a rectangular outline.
    painter.line_segment([rect.left_top(), rect.right_top()], stroke);
    painter.line_segment([rect.right_top(), rect.right_bottom()], stroke);
    painter.line_segment([rect.right_bottom(), rect.left_bottom()], stroke);
    painter.line_segment([rect.left_bottom(), rect.left_top()], stroke);
}

fn paint_grid(painter: &egui::Painter, canvas: Rect) {
    // Draws a subdued calibration grid across the preview canvas.
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

fn paint_crosshair(painter: &egui::Painter, canvas: Rect, point: PixelPoint) {
    // Draws a full-span alignment guide inside the guest frame only.
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
    use super::*;

    fn test_canvas() -> Rect {
        Rect::from_min_size(
            Pos2::new(25.0, 40.0),
            egui::vec2(NOMINAL_FRAME_WIDTH as f32, NOMINAL_FRAME_HEIGHT as f32),
        )
    }

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

    #[test]
    fn preview_uses_the_full_guest_height_when_the_host_has_room() {
        assert_eq!(preview_viewport_height(1_400.0, 1.0), 1_104.0);
        assert_eq!(preview_viewport_height(900.0, 1.0), 804.0);
        assert_eq!(preview_viewport_height(1_400.0, 2.0), 564.0);
    }

    #[test]
    fn preview_mapping_clamps_the_bottom_right_edges() {
        let canvas = test_canvas();

        assert_eq!(
            preview_to_pixel(canvas, canvas.right_bottom()),
            PixelPoint::new(1_919, 1_079)
        );
    }

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
    }

    #[test]
    fn operation_labels_distinguish_bounded_and_unbounded_runs() {
        assert_eq!(format_operation_limit(0), "continuous until STOP");
        assert_eq!(format_operation_limit(25), "25-operation");
        assert_eq!(format_operation_progress(7, 0), "7/unbounded");
        assert_eq!(format_operation_progress(7, 25), "7/25");
    }

    #[test]
    fn preview_authority_is_revoked_by_mode_socket_size_or_stop() {
        let tri = (PathBuf::from("/tmp/current.sock"), GameMode::TriPeaks);
        let pyramid = (PathBuf::from("/tmp/current.sock"), GameMode::Pyramid);
        let old_socket = (PathBuf::from("/tmp/old.sock"), GameMode::TriPeaks);
        let size = Some((NOMINAL_FRAME_WIDTH, NOMINAL_FRAME_HEIGHT));
        assert!(preview_is_current(size, Some(&tri), &tri, true, false));
        assert!(!preview_is_current(size, Some(&tri), &pyramid, true, false));
        assert!(!preview_is_current(
            size,
            Some(&old_socket),
            &tri,
            true,
            false
        ));
        assert!(!preview_is_current(
            Some((640, 480)),
            Some(&tri),
            &tri,
            true,
            false
        ));
        assert!(!preview_is_current(size, Some(&tri), &tri, false, false));
        assert!(!preview_is_current(size, Some(&tri), &tri, true, true));
    }

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
