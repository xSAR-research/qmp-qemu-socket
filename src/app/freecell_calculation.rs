//! Identity-fenced, plan-only Free Cell controls. No guest/execution permission.

use super::*;
use crate::{
    ui_style::PressableButton,
    worker::freecell_calculation::{Identity, Progress, Report},
};
use std::sync::Arc;


/// Offline source selection and one outstanding request; RECON may select its published file.
#[derive(Default)]
pub(super) struct Calculation {
    /// Exact selected source pathname, supplied manually or by an accepted publication.
    path: String,
    /// Monotonic source edit generation.
    generation: u64,
    /// Last allocated checked request identity.
    serial: u64,
    /// Exact pending source/selection/job triple.
    pending: Option<Identity>,
    /// Advisory latest counters, never completion authority.
    progress: Option<Progress>,
    /// Only an accepted dedicated finish can install a report.
    report: Option<Arc<Report>>,
    /// Local validation/refusal/final summary.
    message: String,
    /// Whether the worker-backed source browser is visible.
    browser: bool,
    /// Literal directory to enumerate on the worker, never on egui.
    directory: String,
    /// Checked source-browser serial, independent of route/calculation identity.
    browser_serial: u64,
    /// Exact outstanding browser invocation; closing retires its result.
    browser_pending: Option<(u64, PathBuf)>,
    /// Known files for this directory, including a successfully published acquisition.
    files: Vec<PathBuf>,
    /// Route visibility is bound to this accepted calculation report.
    route_open: bool,
    /// Local display size in points, independent of LOG and preview scaling.
    route_text_size_points: Option<f32>,
    /// Shared LOG-style minimise, maximise, restore and resize presentation.
    route_chrome: window_chrome::WindowChrome,
}


impl Calculation {
    /// Accept only a still-selected directory response; selection performs no I/O.
    pub(super) fn sources(
        &mut self,
        request_id: u64,
        directory: PathBuf,
        result: Result<Vec<PathBuf>, String>,
    ) -> bool {
        if !self.browser
            || directory != std::path::Path::new(&self.directory)
            || self.browser_pending.as_ref() != Some(&(request_id, directory.clone()))
        {
            return false;
        }
        self.browser_pending = None;
        match result {
            Ok(files) => self.files = files,
            Err(error) => self.message = error,
        }
        true
    }


    /// Change source only while idle; overflow leaves the former source intact.
    fn set_source(&mut self, path: String) -> Result<(), String> {
        if self.pending.is_some() {
            return Err("Calculation source is frozen while active".to_owned());
        }
        let generation = self
            .generation
            .checked_add(1)
            .ok_or("Source generation exhausted")?;
        self.path = path;
        self.generation = generation;
        self.report = None;
        self.route_open = false;
        self.progress = None;
        self.browser = false;
        self.browser_pending = None;
        Ok(())
    }


    /// Select an already published acquisition without opening, listing or calculating it.
    /// Failed pathname/generation checks leave the previous source and browser intact.
    fn select_published(&mut self, path: &std::path::Path) -> Result<(), String> {
        let source = path
            .to_str()
            .ok_or("Published Free Cell filename is not UTF-8")?;
        let parent = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| std::path::Path::new("."));
        let directory = parent
            .to_str()
            .ok_or("Published Free Cell directory is not UTF-8")?;
        self.set_source(source.to_owned())?;
        if std::path::Path::new(&self.directory) != parent {
            self.files.clear();
        }
        self.directory = directory.to_owned();
        if !self.files.iter().any(|known| known == path) {
            self.files.push(path.to_owned());
            self.files.sort();
        }
        self.message = "RECON JSON selected; CALCULATE prepares its route".to_owned();
        Ok(())
    }


    /// Reserve this selected source exactly once, preserving stale-event fences.
    fn reserve(&mut self, context: ControllerContext) -> Result<(Identity, PathBuf), String> {
        if self.pending.is_some() {
            return Err("Calculation already active".to_owned());
        }
        if self.path.is_empty() {
            return Err("Select an exported Free Cell JSON document".to_owned());
        }
        self.serial = self
            .serial
            .checked_add(1)
            .ok_or("Free Cell calculation identity exhausted")?;
        let identity = Identity {
            context,
            request_id: self.serial,
            source_generation: self.generation,
        };
        self.pending = Some(identity.clone());
        self.browser = false;
        self.browser_pending = None;
        self.progress = None;
        self.report = None;
        self.route_open = false;
        self.message = "Calculating full board first; exhaustive maximum removal fallback — no configured search cutoffs".to_owned();
        Ok((identity, PathBuf::from(&self.path)))
    }


    /// Retire exactly one matching result; stale/duplicate work cannot replace it.
    fn finish(&mut self, identity: &Identity, result: Result<Arc<Report>, String>) -> bool {
        if self.pending.as_ref() != Some(identity) || self.generation != identity.source_generation
        {
            return false;
        }
        self.pending = None;
        match result {
            Ok(report) => {
                self.message =
                    crate::worker::freecell_calculation::outcome_summary_message(&report.outcome);
                self.report = Some(report);
            }
            Err(error) => self.message = error,
        }
        true
    }


    /// Accept progress only while its exact calculation remains pending.
    fn progress(&mut self, progress: Progress) {
        if self.pending.as_ref() == Some(&progress.identity)
            && self.generation == progress.identity.source_generation
        {
            self.progress = Some(progress);
        }
    }
}


/// Render live A* calculation counters using the same format across Manual and Session modes.
pub(super) fn render_live_calculation_stats(
    ui: &mut egui::Ui,
    search: &xsar::games::freecell::search::Progress,
    paused: std::time::Duration,
) {
    let stats = &search.search.stats;
    ui.label(format!("Paused {:.1}s", paused.as_secs_f64()));
    ui.label(format!(
        "{:.1}s active · {} expanded · {} generated · {} retained · {} peak frontier · best foundations {}/52{}",
        stats.elapsed.as_secs_f64(),
        stats.expanded,
        stats.generated,
        stats.states,
        stats.peak_frontier,
        search.best_foundation_cards,
        if search.search.finished {
            " · checking final replay"
        } else {
            ""
        }
    ));
}


/// Render final determined route/max-card statistics shared by Manual and Session modes.
pub(super) fn render_determined_calculation_stats(
    ui: &mut egui::Ui,
    report: &crate::worker::freecell_calculation::Report,
) {
    let stats = crate::worker::freecell_calculation::stats(&report.outcome);
    ui.label(format!(
        "Final: {:.2}s active · {} expanded · {} generated · {} retained · {} peak frontier",
        stats.elapsed.as_secs_f64(),
        stats.expanded,
        stats.generated,
        stats.states,
        stats.peak_frontier
    ));
    ui.label(format!("Paused {:.2}s", report.paused.as_secs_f64()));
    ui.label(format!(
        "Frozen source {} · {} bytes · deal {}",
        report.path.display(),
        report.bytes.len(),
        report.deal.metadata().deal_id()
    ));
}


impl QmpQemuSocketApp {
    /// An accepted RECON publication replaces source selection and revokes the former plan.
    /// This grants no capture, calculation or guest-input authority.
    pub(super) fn select_freecell_published_source(
        &mut self,
        request_id: u64,
        destination: &std::path::Path,
    ) {
        self.revoke_freecell_playback();
        match self.freecell_calculation.select_published(destination) {
            Ok(()) => {
                self.trace_freecell_ui(
                    "recon-source-selected",
                    serde_json::json!({
                    "request_id":request_id,"published_destination":destination,
                    "source_generation":self.freecell_calculation.generation,
                    "calculation_started":false,"guest_input_attempted":false}),
                );
                self.push_log(format!(
                    "Free Cell RECON JSON selected for CALCULATE: {}",
                    destination.display()
                ));
            }
            Err(error) => {
                self.trace_freecell_ui(
                    "recon-source-selection-failed",
                    serde_json::json!({
                    "request_id":request_id,"published_destination":destination,
                    "publication":"published","error":error}),
                );
                self.failure_window.general(
                    "Free Cell RECON source selection",
                    &error,
                    serde_json::json!({"request_id":request_id,"published_destination":destination,
                        "publication":"published","guest_input_attempted":false}),
                );
                self.push_log(format!(
                    "Free Cell RECON exported; source selection: {error}"
                ));
            }
        }
    }


    /// Reserve one browser token before dispatch; no directory I/O runs on egui.
    fn request_freecell_sources(&mut self) {
        let mut request_identity = None;
        let result = (|| {
            let request_id = self
                .freecell_calculation
                .browser_serial
                .checked_add(1)
                .ok_or("Free Cell browser request identity exhausted")?;
            self.freecell_calculation.browser_serial = request_id;
            let directory = PathBuf::from(&self.freecell_calculation.directory);
            request_identity = Some((request_id, directory.clone()));
            self.freecell_calculation.browser_pending = Some((request_id, directory.clone()));
            self.trace_freecell_ui(
                "source-browser-dispatch",
                serde_json::json!({
                "request_id":request_id,"directory":directory}),
            );
            self.worker
                .list_freecell_sources(self.current_preview_context(), request_id, directory)
        })();
        if let Err(error) = result {
            self.freecell_calculation.browser_pending = None;
            let details = serde_json::json!({
                "request_id":request_identity.as_ref().map(|(id,_)|*id),
                "directory":request_identity.as_ref().map(|(_,directory)|directory),
                "selected_directory":self.freecell_calculation.directory,
                "context":format!("{:?}",self.current_preview_context()),
                "stage":if request_identity.is_some(){"worker command dispatch"}else{"request allocation"},
                "original_error":error,"guest_input_attempted":false,
            });
            self.trace_freecell_ui("source-browser-dispatch-failure", details.clone());
            self.failure_window
                .general("Free Cell source browser dispatch", &error, details);
            self.freecell_calculation.message = error;
        }
    }


    /// Only an exact active directory request may update source choices or failure details.
    pub(super) fn freecell_sources_listed(
        &mut self,
        request_id: u64,
        directory: PathBuf,
        result: Result<Vec<PathBuf>, String>,
    ) {
        let failure = result.as_ref().err().cloned();
        let accepted = self
            .freecell_calculation
            .sources(request_id, directory.clone(), result);
        self.trace_freecell_ui(
            "source-browser-result",
            serde_json::json!({
            "request_id":request_id,"directory":directory,"accepted":accepted,"error":failure}),
        );
        if accepted && let Some(error) = failure {
            self.failure_window.general("Free Cell source browser", &error,
                serde_json::json!({"request_id":request_id,"directory":directory,
                    "context":format!("{:?}",self.current_preview_context()),"guest_input_attempted":false}));
        }
    }


    /// Freeze a local/offline error with the selected source or exact accepted identity.
    pub(super) fn freecell_calculation_failure(
        &mut self,
        label: &str,
        error: &str,
        identity: Option<&Identity>,
    ) {
        self.trace_freecell_ui(
            "calculation-local-failure",
            serde_json::json!({
            "stage":label,"error":error,"identity":identity.map(|id|format!("{id:?}")),
            "source_path":self.freecell_calculation.path,
            "source_generation":self.freecell_calculation.generation}),
        );
        self.failure_window.general(
            label,
            error,
            serde_json::json!({
                "context":format!("{:?}",self.current_preview_context()),
                "identity":identity.map(|identity|format!("{identity:#?}")),
                "source_path":self.freecell_calculation.path,
                "source_generation":self.freecell_calculation.generation,
                "directory":self.freecell_calculation.directory,
                "guest_input_attempted":false,
            }),
        );
    }


    /// Offer the complete offline calculation independently of live RECON/export.
    pub(super) fn freecell_calculation_controls(&mut self, ui: &mut egui::Ui) {
        let idle = self.active_run.is_none() && !self.worker_state.is_busy() && !self.exiting;
        ui.horizontal_wrapped(|ui| {
            let mut source = self.freecell_calculation.path.clone();
            let changed = ui
                .add_enabled(
                    idle,
                    egui::TextEdit::singleline(&mut source)
                        .hint_text("Free Cell source JSON path")
                        .desired_width(ui.available_width().min(330.0)),
                )
                .changed();
            if changed {
                match self.freecell_calculation.set_source(source) {
                    Ok(()) => {
                        self.trace_freecell_ui(
                            "source-edited",
                            serde_json::json!({
                        "source_path":self.freecell_calculation.path,
                        "source_generation":self.freecell_calculation.generation}),
                        );
                        self.revoke_freecell_playback();
                    }
                    Err(error) => {
                        self.freecell_calculation_failure(
                            "Free Cell source selection",
                            &error,
                            None,
                        );
                        self.freecell_calculation.message = error;
                    }
                }
            }
            if ui
                .add_enabled(idle, PressableButton::new("Select JSON…"))
                .clicked()
            {
                self.freecell_calculation.browser = true;
                if self.freecell_calculation.directory.is_empty() {
                    self.freecell_calculation.directory = "/home/charlie/tmp".to_owned();
                }
                self.request_freecell_sources();
            }
        });
        let active = self.active_run == Some(ActiveRun::FreeCellCalculation);
        let pause = active.then(|| self.worker.pause_state()).flatten();
        let paused = pause
            .as_ref()
            .map(|state| state.requested || state.acknowledged);
        let [action_clicked, pause_clicked] = calculation_buttons(
            ui,
            idle,
            active,
            paused,
            !self.stop_requested && !self.exiting,
        )
        .map(|response| response.is_some_and(|response| response.clicked()));
        if action_clicked && !active {
            self.revoke_freecell_playback();
            let context = self.current_preview_context();
            match self.freecell_calculation.reserve(context) {
                Ok((identity, path)) => {
                    self.trace_freecell_ui("calculation-dispatch", serde_json::json!({
                        "request_id":identity.request_id,"source_generation":identity.source_generation,
                        "source_path":path,"identity":format!("{identity:?}")}));
                    match self.worker.calculate_freecell(
                        identity.context.clone(),
                        identity.request_id,
                        identity.source_generation,
                        path,
                    ) {
                        Ok(()) => {
                            self.active_run = Some(ActiveRun::FreeCellCalculation);
                            self.worker_state = WorkerState::Validating;
                            self.stop_requested = false;
                        }
                        Err(error) => {
                            self.freecell_calculation_failure(
                                "Free Cell calculation dispatch",
                                &error,
                                Some(&identity),
                            );
                            self.freecell_calculation.finish(&identity, Err(error));
                        }
                    }
                }
                Err(error) => {
                    self.freecell_calculation_failure(
                        "Free Cell calculation request",
                        &error,
                        None,
                    );
                    self.freecell_calculation.message = error;
                }
            }
        }
        if self.freecell_calculation.browser && idle {
            egui::Window::new("Select Free Cell JSON source").show(ui.ctx(), |ui| {
                ui.text_edit_singleline(&mut self.freecell_calculation.directory);
                if ui.add(PressableButton::new("List directory")).clicked() {
                    self.request_freecell_sources();
                }
                let mut selected = None;
                egui::ScrollArea::vertical().max_height(250.0).show_rows(
                    ui,
                    ui.text_style_height(&egui::TextStyle::Body),
                    self.freecell_calculation.files.len(),
                    |ui, range| {
                        for index in range {
                            if let Some(path) = self.freecell_calculation.files.get(index)
                                && ui
                                    .selectable_label(
                                        std::path::Path::new(&self.freecell_calculation.path)
                                            == path,
                                        path.display().to_string(),
                                    )
                                    .clicked()
                            {
                                selected = Some(path.clone());
                            }
                        }
                    },
                );
                if let Some(path) = selected {
                    match path.to_str() {
                        Some(path) => match self.freecell_calculation.set_source(path.to_owned()) {
                            Ok(()) => {
                                self.trace_freecell_ui(
                                    "source-selected",
                                    serde_json::json!({
                                    "source_path":self.freecell_calculation.path,
                                    "source_generation":self.freecell_calculation.generation}),
                                );
                                self.revoke_freecell_playback();
                            }
                            Err(error) => {
                                self.freecell_calculation_failure(
                                    "Free Cell source selection",
                                    &error,
                                    None,
                                );
                                self.freecell_calculation.message = error;
                            }
                        },
                        None => {
                            self.freecell_calculation_failure(
                                "Free Cell source selection",
                                "Unsupported filename",
                                None,
                            );
                            self.freecell_calculation.message = "Unsupported filename".to_owned()
                        }
                    }
                }
                if ui.add(PressableButton::new("Close")).clicked() {
                    self.freecell_calculation.browser = false;
                    self.freecell_calculation.browser_pending = None;
                }
            });
        }
        if pause_clicked && let Some(pause) = pause.as_ref() {
            let requested = !(pause.requested || pause.acknowledged);
            self.trace_freecell_ui(
                "calculation-pause-intent",
                serde_json::json!({
                    "identity":format!("{:?}",self.freecell_calculation.pending),
                    "pause_epoch":pause.epoch,"pause_requested":requested}),
            );
            if let Err(error) = self.worker.request_pause(pause.epoch, requested) {
                let identity = self.freecell_calculation.pending.clone();
                self.freecell_calculation_failure(
                    "Free Cell calculation pause",
                    &error,
                    identity.as_ref(),
                );
                self.freecell_calculation.message = error;
            }
        }
        if active && action_clicked {
            self.trace_freecell_ui(
                "calculation-cancel-intent",
                serde_json::json!({
                    "identity":format!("{:?}",self.freecell_calculation.pending),
                    "action":"Stop"}),
            );
            self.worker.cancel_freecell_calculation();
            self.freecell_calculation.message = "Cancelling…".to_owned();
        }
        if active && let Some(pause) = pause.as_ref() {
            if pause.acknowledged {
                ui.label("Paused — frontier retained");
            } else if pause.requested {
                ui.label("Pausing…");
            }
        }
        if let Some(progress) = self
            .freecell_calculation
            .progress
            .as_ref()
            .filter(|_| self.freecell_calculation.pending.is_some())
        {
            render_live_calculation_stats(ui, &progress.search, progress.paused);
        }
        ui.label(&self.freecell_calculation.message);
        if let Some(report) = &self.freecell_calculation.report {
            render_determined_calculation_stats(ui, report);
            if route_moves(&report.outcome).is_some()
                && ui
                    .add(PressableButton::new("Inspect physical route"))
                    .clicked()
            {
                self.freecell_calculation.route_open = true;
                self.freecell_calculation.route_chrome.maximise();
            }
        }
    }


    /// Inspect the accepted immutable route; this window grants no playback authority.
    /// Closing retains its font preference. Source changes and new calculations close it.
    pub(super) fn physical_route_window(&mut self, context: &egui::Context) {
        let calculation = &mut self.freecell_calculation;
        if !calculation.route_open {
            return;
        }
        let Some(report) = calculation.report.as_ref() else {
            calculation.route_open = false;
            return;
        };
        let Some(moves) = route_moves(&report.outcome) else {
            calculation.route_open = false;
            return;
        };
        let mut open = calculation.route_open;
        let route_description = calculation.message.clone();
        calculation.route_chrome.show(
            context,
            window_chrome::WindowAppearance {
                id: egui::Id::new("freecell-physical-route-window"),
                title: "PHYSICAL ROUTE",
                colour: Color32::from_rgb(0, 125, 148),
                default_size: crate::parameters::DETAILS_WINDOW_DEFAULT_SIZE_POINTS.into(),
                min_size: crate::parameters::DETAILS_WINDOW_MIN_SIZE_POINTS.into(),
            },
            &mut open,
            |ui| {
                let mut size = calculation.route_text_size_points.unwrap_or_else(|| {
                    egui::TextStyle::Monospace.resolve(ui.style()).size.clamp(
                        MINIMUM_OUTPUT_TEXT_SIZE_POINTS,
                        MAXIMUM_OUTPUT_TEXT_SIZE_POINTS,
                    )
                });
                ui.label(&route_description);
                ui.horizontal_wrapped(|ui| {
                    ui.label(format!(
                        "{} · {} transfers · inspection only",
                        report.path.display(),
                        moves.len()
                    ));
                    ui.label("Text size");
                    ui.add(
                        egui::DragValue::new(&mut size)
                            .range(
                                MINIMUM_OUTPUT_TEXT_SIZE_POINTS..=MAXIMUM_OUTPUT_TEXT_SIZE_POINTS,
                            )
                            .speed(1.0)
                            .suffix(" pt"),
                    );
                });
                let font = FontId::monospace(size);
                let height = ui.fonts_mut(|fonts| fonts.row_height(&font));
                egui::ScrollArea::both()
                    .id_salt((
                        "physical-route-rows",
                        calculation.generation,
                        calculation.serial,
                    ))
                    .auto_shrink([false, false])
                    .show_rows(ui, height, moves.len(), |ui, range| {
                        for index in range {
                            if let Some(action) = moves.get(index) {
                                ui.label(
                                    egui::RichText::new(format!(
                                        "{}: {} {} → {}{}",
                                        index + 1,
                                        card(action.card_id),
                                        pile(action.from),
                                        pile(action.to),
                                        if action.is_auto { " AUTO" } else { "" }
                                    ))
                                    .font(font.clone()),
                                );
                            }
                        }
                    });
                calculation.route_text_size_points = Some(size);
            },
        );
        calculation.route_open = open;
    }


    /// Receive only the currently selected, exactly matching offline final result.
    pub(super) fn freecell_calculation_finished(
        &mut self,
        identity: Identity,
        result: Result<Arc<Report>, String>,
    ) {
        let failure = match &result {
            Err(error) => Some(error.clone()),
            Ok(report) => match &report.outcome {
                xsar::games::freecell::search::BestEffortOutcome::Failed { failure, .. } => {
                    Some(failure.to_string())
                }
                _ => None,
            },
        };
        let accepted = self.active_run == Some(ActiveRun::FreeCellCalculation)
            && identity.context == self.current_preview_context()
            && self.freecell_calculation.finish(&identity, result);
        self.trace_freecell_ui(
            "calculation-result",
            serde_json::json!({
            "request_id":identity.request_id,"source_generation":identity.source_generation,
            "identity":format!("{identity:?}"),"accepted":accepted,"error":failure,
            "message":self.freecell_calculation.message}),
        );
        if accepted {
            if let Some(error) = failure {
                self.freecell_calculation_failure("Free Cell calculation", &error, Some(&identity));
            }
            let stopped = self.stop_requested;
            if stopped {
                self.revoke_freecell_playback();
            } else if self
                .freecell_calculation
                .report
                .as_ref()
                .is_some_and(|r| route_moves(&r.outcome).is_some_and(|moves| !moves.is_empty()))
            {
                self.freecell_playback.plan = Some(identity.clone());
            }
            self.active_run = None;
            self.stop_requested = false;
            self.worker_state = WorkerState::Ready;
            self.current_status = self.freecell_calculation.message.clone();
            self.push_log(self.current_status.clone());
            if let Some(report) = self.freecell_calculation.report.take() {
                if let Some(moves) = route_moves(&report.outcome) {
                    for (index, action) in moves.iter().enumerate() {
                        self.push_session_telemetry_at(
                            self.started_at.elapsed(),
                            format!(
                                "Free Cell route {}: {} {} → {}{}",
                                index + 1,
                                card(action.card_id),
                                pile(action.from),
                                pile(action.to),
                                if action.is_auto { " AUTO" } else { "" }
                            ),
                        );
                    }
                }
                self.freecell_calculation.report = Some(report);
            }
        }
    }


    /// Drain coalesced progress after ordered final events, never reviving a run.
    pub(super) fn receive_freecell_calculation_progress(&mut self) {
        if let Some(progress) = self.worker.take_freecell_calculation_progress()
            && progress.identity.context == self.current_preview_context()
        {
            self.freecell_calculation.progress(progress);
        }
    }
}


/// Render the offline action and its active controls together without scheduling work.
/// Stop requests cancellation; Pause retains the frontier for Continue.
fn calculation_buttons(
    ui: &mut egui::Ui,
    idle: bool,
    active: bool,
    paused: Option<bool>,
    mutable: bool,
) -> [Option<egui::Response>; 2] {
    ui.horizontal(|ui| {
        let action = ui
            .add_enabled(
                if active { mutable } else { idle },
                PressableButton::new(if active { "Stop" } else { "CALCULATE" }),
            )
            .on_hover_text(if active {
                "Cancel this calculation; no usable route is retained"
            } else {
                "Calculate a route offline without sending guest input"
            });
        let pause = active.then(|| {
            ui.add_enabled(
                mutable && paused.is_some(),
                PressableButton::new(if paused == Some(true) {
                    "Continue"
                } else {
                    "Pause"
                }),
            )
        });
        [Some(action), pause]
    })
    .inner
}


/// Borrow only independently replayed full or exhaustive partial physical transfers.
/// Cancellation and failures contain no route; an empty partial remains inspectable only.
fn route_moves(
    outcome: &xsar::games::freecell::search::BestEffortOutcome,
) -> Option<&[xsar::games::freecell::rules::RecordedMove]> {
    use xsar::games::freecell::search::BestEffortOutcome;
    match outcome {
        BestEffortOutcome::Solved { moves, .. } | BestEffortOutcome::Partial { moves, .. } => {
            Some(moves)
        }
        BestEffortOutcome::Cancelled(_) | BestEffortOutcome::Failed { .. } => None,
    }
}


/// Physical one-based labels for route inspection; foundations use canonical suit IDs.
fn pile(pile: xsar::games::freecell::rules::Pile) -> String {
    use xsar::games::freecell::rules::Pile;
    match pile {
        Pile::Cascade(n) => format!("PLAY {}", u16::from(n) + 1),
        Pile::Cell(n) => format!("CELL {}", u16::from(n) + 1),
        Pile::Foundation(n) => format!(
            "SUIT {}",
            ["Clubs", "Diamonds", "Hearts", "Spades"]
                .get(usize::from(n))
                .copied()
                .unwrap_or("invalid")
        ),
    }
}


/// Render a closed model identity for review without inferring a guest position.
fn card(id: u8) -> String {
    let suit = ["Clubs", "Diamonds", "Hearts", "Spades"]
        .get(usize::from(id / 13))
        .copied()
        .unwrap_or("invalid");
    match xsar::games::cards::Rank::from_value(id % 13 + 1) {
        Some(rank) if id < 52 => format!("{} of {suit}", rank.as_str()),
        _ => "Invalid card".to_owned(),
    }
}


#[cfg(test)]
mod tests {
    use super::*;


    /// Render the production calculation row without a worker, filesystem or guest.
    fn calculation_button_frame(
        context: &egui::Context,
        state: (bool, bool, Option<bool>, bool),
        events: Vec<egui::Event>,
        width: f32,
    ) -> [(egui::Rect, bool, bool); 2] {
        crate::ui_style::apply_dark_theme(context);
        let mut buttons = [(egui::Rect::NOTHING, false, false); 2];
        context
            .run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(width, 300.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ui| {
                    buttons = calculation_buttons(ui, state.0, state.1, state.2, state.3).map(
                        |response| {
                            response.map_or((egui::Rect::NOTHING, false, false), |response| {
                                (response.rect, response.enabled(), response.clicked())
                            })
                        },
                    );
                },
            )
            .drop_without_applying_deltas();
        buttons
    }


    /// Click one real calculation control without dispatching any operation.
    fn click_calculation_button(
        context: &egui::Context,
        state: (bool, bool, Option<bool>, bool),
        position: egui::Pos2,
        width: f32,
    ) -> [bool; 2] {
        let pointer = |pressed| egui::Event::PointerButton {
            pos: position,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        calculation_button_frame(
            context,
            state,
            vec![egui::Event::PointerMoved(position), pointer(true)],
            width,
        );
        calculation_button_frame(context, state, vec![pointer(false)], width)
            .map(|(_, _, clicked)| clicked)
    }


    /// CALCULATE becomes the sole Stop action beside Pause/Continue for its active owner.
    #[test]
    fn freecell_321_actual_calculation_stop_pause_stay_adjacent_and_lock_on_stop() {
        for width in [320.0, 640.0, 1920.0] {
            let context = egui::Context::default();
            let idle = (true, false, None, true);
            let buttons = calculation_button_frame(&context, idle, vec![], width);
            assert_eq!(buttons.map(|(_, enabled, _)| enabled), [true, false]);
            assert!(!buttons[1].0.is_positive());
            assert_eq!(
                click_calculation_button(&context, idle, buttons[0].0.center(), width),
                [true, false]
            );
            for paused in [false, true] {
                let active = (false, true, Some(paused), true);
                let buttons = calculation_button_frame(&context, active, vec![], width);
                assert_eq!(buttons.map(|(_, enabled, _)| enabled), [true; 2]);
                for (index, (bounds, _, _)) in buttons.into_iter().enumerate() {
                    assert!(bounds.is_positive());
                    assert!(bounds.left() >= 0.0 && bounds.right() <= width);
                    assert_eq!(bounds.center().y, buttons[0].0.center().y);
                    let mut expected = [false; 2];
                    expected[index] = true;
                    assert_eq!(
                        click_calculation_button(&context, active, bounds.center(), width),
                        expected
                    );
                }
            }
            let stopping = (false, true, Some(false), false);
            let buttons = calculation_button_frame(&context, stopping, vec![], width);
            assert_eq!(buttons.map(|(_, enabled, _)| enabled), [false; 2]);
            for (bounds, _, _) in buttons {
                assert_eq!(
                    click_calculation_button(&context, stopping, bounds.center(), width),
                    [false; 2]
                );
            }
            let queued =
                calculation_button_frame(&context, (false, true, None, true), vec![], width);
            assert_eq!(queued.map(|(_, enabled, _)| enabled), [true, false]);
        }
    }


    /// A published source retires old inspection/browser results and is the next calculation input.
    #[test]
    fn freecell_315_published_recon_selects_exact_json_and_retires_old_ui_state() {
        let directory = PathBuf::from("/nonexistent-freecell/exports");
        let published = directory.join("new deal.json");
        let mut state = Calculation {
            path: "previous.json".to_owned(),
            generation: 7,
            directory: directory.display().to_string(),
            files: vec![directory.join("older.json")],
            browser: true,
            browser_pending: Some((12, directory.clone())),
            route_open: true,
            route_text_size_points: Some(22.0),
            ..Default::default()
        };
        state.select_published(&published).unwrap();
        assert_eq!(std::path::Path::new(&state.path), published);
        assert_eq!(state.generation, 8);
        assert!(!state.browser);
        assert!(state.browser_pending.is_none());
        assert!(!state.route_open);
        assert_eq!(state.route_text_size_points, Some(22.0));
        assert!(state.files.contains(&published));
        assert_eq!(state.files.len(), 2);
        assert!(!state.sources(12, directory, Err("late listing".into())));
        let (identity, selected) = state.reserve(context()).unwrap();
        assert_eq!(selected, published);
        assert_eq!(identity.source_generation, 8);
    }


    /// Only one exact accepted RECON completion can replace the selected source.
    #[test]
    fn freecell_315_stale_duplicate_and_failed_recon_do_not_select_a_source() {
        use crate::pyramid_workflow_model::RunTermination;
        let selected_context = context();
        let mut recon = super::super::freecell_recon::FreeCellReconState::default();
        let mut calculation = Calculation::default();
        calculation.set_source("retained.json".into()).unwrap();
        let id = recon.reserve(selected_context.clone()).unwrap();
        let mut stale_context = selected_context.clone();
        stale_context.generation += 1;
        for (context, token, termination, published) in [
            (
                &stale_context,
                id,
                RunTermination::Completed,
                Some("stale.json"),
            ),
            (
                &selected_context,
                id + 1,
                RunTermination::Completed,
                Some("wrong.json"),
            ),
            (
                &selected_context,
                id,
                RunTermination::Completed,
                Some("accepted.json"),
            ),
            (
                &selected_context,
                id,
                RunTermination::Completed,
                Some("duplicate.json"),
            ),
        ] {
            if recon.finish(context, token, termination)
                && let Some(path) = published
            {
                calculation
                    .select_published(std::path::Path::new(path))
                    .unwrap();
            }
        }
        assert_eq!(calculation.path, "accepted.json");
        assert_eq!(calculation.generation, 2);
        let next = recon.reserve(selected_context.clone()).unwrap();
        assert!(recon.finish(&selected_context, next, RunTermination::Failed));
        assert_eq!(calculation.path, "accepted.json");
        assert_eq!(calculation.generation, 2);
    }


    /// Generation exhaustion cannot partially replace the source or the cached directory.
    #[test]
    fn freecell_315_published_recon_selection_failure_preserves_previous_source() {
        let mut state = Calculation {
            path: "previous.json".into(),
            generation: u64::MAX,
            directory: "previous-directory".into(),
            files: vec!["previous.json".into()],
            browser: true,
            route_open: true,
            ..Default::default()
        };
        assert_eq!(
            state.select_published(std::path::Path::new("new/deal.json")),
            Err("Source generation exhausted".to_owned())
        );
        assert_eq!(state.path, "previous.json");
        assert_eq!(state.directory, "previous-directory");
        assert_eq!(state.files, vec![PathBuf::from("previous.json")]);
        assert!(state.browser);
        assert!(state.route_open);
    }


    /// Repeated same-directory requests need their own token; a closed browser stays closed.
    #[test]
    fn freecell_313_source_browser_retains_current_failure_against_duplicates() {
        let directory = PathBuf::from("/tmp/freecell-json");
        let mut state = Calculation {
            browser: true,
            directory: directory.display().to_string(),
            browser_pending: Some((2, directory.clone())),
            ..Default::default()
        };
        assert!(!state.sources(1, directory.clone(), Err("old failure".into())));
        assert!(state.sources(2, directory.clone(), Err("current original cause".into())));
        assert_eq!(state.message, "current original cause");
        assert!(!state.sources(2, directory.clone(), Err("duplicate error".into())));
        assert_eq!(state.message, "current original cause");
        state.browser_pending = Some((3, directory.clone()));
        state.browser = false;
        assert!(!state.sources(3, directory, Err("closed browser".into())));
        assert_eq!(state.message, "current original cause");
    }


    /// Correlation context grants no live acquisition or input authority.
    fn context() -> ControllerContext {
        ControllerContext::new(
            PathBuf::from("/nonexistent-offline/qmp.sock"),
            GameMode::FreeCell,
            SolvingStrategy::ShortestPath,
            4,
        )
    }


    /// Source edits and stale progress/completions cannot replace the frozen active job.
    #[test]
    fn freecell_310_source_and_job_fences_retire_once() {
        let mut state = Calculation::default();
        state.set_source("original.json".to_owned()).unwrap();
        state.browser = true;
        let (identity, path) = state.reserve(context()).unwrap();
        assert_eq!(path, PathBuf::from("original.json"));
        assert!(!state.browser);
        assert!(state.set_source("different.json".to_owned()).is_err());
        assert_eq!(state.path, "original.json");
        let mut stale = identity.clone();
        stale.source_generation += 1;
        assert!(!state.finish(&stale, Err("stale".to_owned())));
        assert!(state.pending.is_some());
        let p = Progress {
            identity: stale.clone(),
            search: xsar::games::freecell::search::Progress {
                search: xsar::search::SearchProgress {
                    stats: xsar::search::SearchStats::default(),
                    finished: false,
                },
                best_foundation_cards: 0,
            },
            paused: Duration::ZERO,
        };
        state.progress(p);
        assert!(state.progress.is_none());
        assert!(state.finish(&identity, Err("cancelled at owned checkpoint".to_owned())));
        assert!(!state.finish(&identity, Err("duplicate".to_owned())));
        state.progress(Progress {
            identity: identity.clone(),
            search: xsar::games::freecell::search::Progress {
                search: xsar::search::SearchProgress {
                    stats: xsar::search::SearchStats::default(),
                    finished: true,
                },
                best_foundation_cards: 52,
            },
            paused: Duration::ZERO,
        });
        assert!(state.progress.is_none());
        state.set_source("second.json".to_owned()).unwrap();
        let (next, _) = state.reserve(context()).unwrap();
        assert!(next.request_id > identity.request_id);
        assert!(next.source_generation > identity.source_generation);
    }


    /// Checked ID/source exhaustion leaves the former identity and source unchanged.
    #[test]
    fn freecell_310_source_and_request_exhaustion_fail_closed() {
        let mut state = Calculation {
            path: "saved.json".to_owned(),
            generation: u64::MAX,
            ..Calculation::default()
        };
        assert!(state.set_source("lost.json".to_owned()).is_err());
        assert_eq!(state.path, "saved.json");
        state.serial = u64::MAX;
        assert!(state.reserve(context()).is_err());
        assert!(state.pending.is_none());
    }

    /// Source changes and a new calculation close the inspector while keeping local font size.
    #[test]
    fn freecell_312_route_window_follows_source_lifetime() {
        let mut state = Calculation {
            route_open: true,
            route_text_size_points: Some(22.0),
            ..Calculation::default()
        };
        state.set_source("first.json".into()).unwrap();
        assert!(!state.route_open);
        assert_eq!(state.route_text_size_points, Some(22.0));
        state.route_open = true;
        let (id, _) = state.reserve(context()).unwrap();
        assert!(!state.route_open);
        state.route_open = true;
        assert!(state.set_source("late.json".into()).is_err());
        assert!(state.route_open);
        assert!(state.finish(&id, Err("stopped".into())));
        state.set_source("next.json".into()).unwrap();
        assert!(!state.route_open);
        assert_eq!(state.route_text_size_points, Some(22.0));
    }
}
