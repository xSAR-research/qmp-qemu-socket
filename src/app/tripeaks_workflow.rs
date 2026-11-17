//! Independent TriPeaks operator UI across `Continuous`, `Single board`, and `Manual` modes.
//!
//! Renders plain card-knowledge status (`Known / Unknown / Unreachable`, `Probes used`),
//! active objective, live A* search statistics, and Manual `RECON` / `CALCULATE` / `VIEW ROUTE` /
//! `SINGLE STEP` / `MULTI STEP` / `UNDO ALL` controls without blocking the UI thread or
//! granting guest input authority directly.

use std::{path::PathBuf, sync::Arc, time::Duration};

use super::*;
use crate::{
    pyramid_diagnostics::PyramidPhase,
    pyramid_workflow_model::{RunTermination, SessionPhase},
    tripeaks_model::TriPeaksMove,
    tripeaks_progress::TriPeaksChallenge,
    tripeaks_workflow_model::{
        SlotKnowledgeCounts, TriPeaksManualReconSummary, TriPeaksPlanningSnapshot,
        TriPeaksPlayStyle, TriPeaksSessionCompletion, TriPeaksSessionProgress,
        TriPeaksSessionRequest, format_tripeaks_objective,
    },
    ui_style::PressableButton,
    worker::{
        tripeaks_calculation,
        tripeaks_reconnaissance::{TriPeaksReconConfig, TriPeaksStepTiming},
        tripeaks_route,
    },
};

/// Operator-owned UI state for both Automatic (`Continuous` / `Single board`) and `Manual` TriPeaks.
#[derive(Default)]
pub(super) struct TriPeaksWorkflowState {
    /// Selected TriPeaks mode rendered in the top workflow bar.
    pub(super) play_style: TriPeaksPlayStyle,
    /// Active controller identity bound to session status fields.
    context: Option<ControllerContext>,
    /// Monotonic session run counter (checked arithmetic, never reset).
    next_run_id: u64,
    /// Active automatic session identity while [`ActiveRun::TriPeaksSession`] owns the UI.
    pub(super) active_run_id: Option<u64>,
    /// One-based board cycle index reported by the running session.
    deal_generation: u64,
    /// Cumulative verified boards cleared in the session.
    completed_boards: usize,
    /// Cumulative full 3-board games / Challenges completed in the session.
    completed_games: usize,
    /// Latest session phase while running.
    phase: Option<SessionPhase>,
    /// Latest session card-knowledge counts (`Known / Unknown / Unreachable`).
    counts: Option<SlotKnowledgeCounts>,
    /// Latest session `Undo All` probe cycle count.
    probes_used: usize,
    /// Plain-English active objective description.
    objective_label: Option<String>,
    /// Execution cursor `(completed_moves, total_moves)`.
    route_cursor: Option<(usize, usize)>,
    /// Expanded states from latest planning pass.
    expanded_states: Option<u64>,
    /// Live replaceable planning snapshot during `SessionPhase::Planning`.
    planning_progress: Option<TriPeaksPlanningSnapshot>,
    /// Final determined A* calculation report for statistics inspection.
    calculation_report: Option<Arc<tripeaks_calculation::Report>>,
    /// Present when an unsolvable board halted safely without clicking Redeal.
    unsolvable_max_cleared: Option<u8>,
    /// Most recent exported `52`-slot JSON path.
    export_path: Option<PathBuf>,
    /// Context retaining diagnostic preview across detached state transitions.
    retained_preview: Option<ControllerContext>,
    /// Final session completion or stop message.
    report: Option<String>,

    /// Optional explicit export destination input for Manual `RECON / Export 52-slot deal`.
    export_path_input: String,
    /// Monotonic Manual `RECON` request serial.
    recon_serial: u64,
    /// Active Manual `RECON` owner until `TriPeaksReconFinished` arrives.
    active_recon: Option<(ControllerContext, u64)>,
    /// Card counts observed in Manual mode.
    manual_counts: Option<SlotKnowledgeCounts>,
    /// `Undo All` probe count observed in Manual `RECON` or loaded JSON.
    manual_probes_used: usize,
    /// Plain objective label observed in Manual `RECON` or loaded JSON.
    manual_objective_label: Option<String>,
    /// Directory typed or opened in the Manual JSON source browser.
    source_dir_input: String,
    /// Whether the inline JSON source browser window is open.
    source_browser_open: bool,
    /// Monotonic source-listing request serial.
    source_list_serial: u64,
    /// Outstanding source-listing request ID.
    source_list_pending: Option<u64>,
    /// Enumerated `.json` candidate paths from the selected directory.
    listed_sources: Vec<PathBuf>,
    /// Active source JSON path edited by operator or populated by `RECON` / browser.
    source_path_input: String,
    /// Monotonic counter bumped on every source path edit or publication.
    source_generation: u64,
    /// Monotonic Manual calculation request serial.
    calc_serial: u64,
    /// Outstanding Manual offline calculation identity.
    active_calc: Option<tripeaks_calculation::Identity>,
    /// Latest live search progress for Manual `CALCULATE`.
    live_calc_progress: Option<tripeaks_calculation::Progress>,
    /// Identity of the currently cached Manual calculation candidate, if valid.
    prepared_plan: Option<tripeaks_calculation::Identity>,
    /// Frozen determined report from Manual `CALCULATE`.
    manual_report: Option<Arc<tripeaks_calculation::Report>>,
    /// Outstanding Manual route invocation `(plan_identity, route_run_id)`.
    pending_route: Option<(tripeaks_calculation::Identity, u64)>,
    /// Non-`None` while `MultiStep` is active (`true` once `Stop after step` was requested).
    multi_step_stop_requested: Option<bool>,
    /// Coalesced live execution progress for Manual playback.
    live_route_progress: Option<tripeaks_route::Progress>,
    /// Concise Manual status message.
    manual_message: String,
    /// Toggles the `VIEW ROUTE` step-by-step route breakdown panel.
    show_route_inspector: bool,
}

impl TriPeaksWorkflowState {
    /// Reset displayed state cleanly when switching controller contexts while idle.
    pub(super) fn bind(&mut self, context: &ControllerContext) {
        if self.context.as_ref() != Some(context) {
            self.context = Some(context.clone());
            self.active_run_id = None;
            self.deal_generation = 0;
            self.completed_boards = 0;
            self.completed_games = 0;
            self.phase = None;
            self.counts = None;
            self.probes_used = 0;
            self.objective_label = None;
            self.route_cursor = None;
            self.expanded_states = None;
            self.planning_progress = None;
            self.calculation_report = None;
            self.unsolvable_max_cleared = None;
            self.retained_preview = None;
            self.report = None;
            self.active_recon = None;
            self.active_calc = None;
            self.prepared_plan = None;
            self.pending_route = None;
            self.multi_step_stop_requested = None;
            self.live_route_progress = None;
        }
    }

    /// Preserve diagnostic pixels after clean session or recon detach under the same context.
    pub(super) fn retain_after_detach(&self, context: &ControllerContext) -> bool {
        self.retained_preview.as_ref() == Some(context)
    }

    /// Clear retained diagnostic preview marker when an explicit new preview or switch starts.
    pub(super) fn clear_preview(&mut self) {
        self.retained_preview = None;
    }

    /// Retire Manual playback selection cleanly when context changes or stops.
    fn clear_manual_playback(&mut self) {
        self.prepared_plan = None;
        self.live_route_progress = None;
    }

    /// Map active session lifecycle phase for the top header badge.
    pub(super) fn activity_phase(&self, context: &ControllerContext) -> Option<PyramidPhase> {
        if self.context.as_ref() != Some(context) {
            return None;
        }
        self.phase.map(|phase| match phase {
            SessionPhase::Entry | SessionPhase::Reconnaissance => PyramidPhase::Reconnaissance,
            SessionPhase::Planning => PyramidPhase::Planning,
            SessionPhase::Execution
            | SessionPhase::Transition
            | SessionPhase::Completion
            | SessionPhase::Restart => PyramidPhase::Execution,
        })
    }

    /// Allocate a fresh run identity and clear per-run session display counters.
    fn begin_session_run(&mut self, context: &ControllerContext) -> Result<u64, &'static str> {
        if self.active_run_id.is_some() {
            return Err("TriPeaks automatic session is already active");
        }
        let run_id = self
            .next_run_id
            .checked_add(1)
            .ok_or("TriPeaks session run identity exhausted")?;
        self.next_run_id = run_id;
        self.bind(context);
        self.active_run_id = Some(run_id);
        self.deal_generation = 0;
        self.completed_boards = 0;
        self.completed_games = 0;
        self.phase = Some(SessionPhase::Entry);
        self.counts = None;
        self.probes_used = 0;
        self.objective_label = None;
        self.route_cursor = None;
        self.expanded_states = None;
        self.planning_progress = None;
        self.calculation_report = None;
        self.unsolvable_max_cleared = None;
        self.retained_preview = None;
        self.report = None;
        Ok(run_id)
    }

    /// Apply one context-checked worker session progress update.
    fn receive_session_progress(
        &mut self,
        progress: &TriPeaksSessionProgress,
        context: &ControllerContext,
        owner: Option<ActiveRun>,
    ) -> bool {
        if owner != Some(ActiveRun::TriPeaksSession)
            || &progress.context != context
            || self.context.as_ref() != Some(context)
            || self.active_run_id != Some(progress.run_id)
        {
            return false;
        }
        if progress.deal_generation != self.deal_generation {
            self.route_cursor = None;
            self.expanded_states = None;
            self.planning_progress = None;
            self.calculation_report = None;
        }
        self.phase = Some(progress.phase);
        self.deal_generation = progress.deal_generation;
        self.completed_boards = progress.completed_boards;
        self.completed_games = progress.completed_games;
        if progress.counts.is_some() {
            self.counts = progress.counts;
        }
        self.probes_used = progress.probes_used;
        if progress.objective_label.is_some() {
            self.objective_label = progress.objective_label.clone();
        }
        if progress.route_cursor.is_some() {
            self.route_cursor = progress.route_cursor;
        }
        if progress.expanded_states.is_some() {
            self.expanded_states = progress.expanded_states;
        }
        if let Some(snapshot) = &progress.planning_progress {
            self.planning_progress = Some(snapshot.clone());
        }
        if let Some(report) = &progress.calculation_report {
            self.calculation_report = Some(Arc::clone(report));
            self.planning_progress = None;
        }
        if progress.phase != SessionPhase::Planning {
            self.planning_progress = None;
        }
        if progress.export_path.is_some() {
            self.export_path = progress.export_path.clone();
        }
        true
    }

    /// Retire matching session ownership upon [`WorkerEvent::TriPeaksSessionFinished`].
    fn complete_session(
        &mut self,
        owner: &mut Option<ActiveRun>,
        state: &mut WorkerState,
        completion: &TriPeaksSessionCompletion,
        context: &ControllerContext,
        stopped: bool,
    ) -> bool {
        if completion.context != *context
            || self.context.as_ref() != Some(context)
            || *owner != Some(ActiveRun::TriPeaksSession)
            || self.active_run_id != Some(completion.run_id)
        {
            return false;
        }
        self.active_run_id = None;
        self.phase = None;
        self.planning_progress = None;
        self.completed_boards = completion.completed_boards;
        self.completed_games = completion.completed_games;
        if completion.counts.is_some() {
            self.counts = completion.counts;
        }
        if completion.objective_label.is_some() {
            self.objective_label = completion.objective_label.clone();
        }
        if let Some(report) = &completion.calculation_report {
            self.calculation_report = Some(Arc::clone(report));
        }
        if completion.export_path.is_some() {
            self.export_path = completion.export_path.clone();
        }
        self.unsolvable_max_cleared = completion.unsolvable_max_cleared;
        self.retained_preview = (!stopped).then(|| context.clone());
        self.report = Some(completion.result.as_ref().map_or_else(
            |err| format!("TriPeaks session stopped: {err}"),
            Clone::clone,
        ));
        *owner = None;
        *state = super::pyramid_workflow::next_session_worker_state(
            *state,
            completion.termination,
            completion.result.is_err(),
            stopped,
        );
        true
    }
}

/// Format one [`TriPeaksMove`] in plain human-readable notation (`Tap R4C3` / `Draw (D)`).
pub(super) fn format_tripeaks_move(mv: TriPeaksMove) -> String {
    match mv {
        TriPeaksMove::TapTableau { row, column } => format!(
            "Tap R{}C{}",
            usize::from(row).saturating_add(1),
            usize::from(column).saturating_add(1)
        ),
        TriPeaksMove::Draw => "Draw (D)".to_owned(),
    }
}

/// Render plain operator knowledge summary and objective banner whenever known.
fn render_knowledge_and_objective(
    ui: &mut egui::Ui,
    counts: Option<SlotKnowledgeCounts>,
    probes_used: usize,
    objective_label: Option<&str>,
) {
    if let Some(c) = counts {
        ui.strong(c.summary_line(probes_used));
    }
    if let Some(label) = objective_label {
        ui.label(format!("Objective: {label}"));
    }
}

/// Render live search stats from `TriPeaksPlanningSnapshot` or `tripeaks_calculation::Progress`.
fn render_tripeaks_live_search_stats(
    ui: &mut egui::Ui,
    search: &xsar::search::SearchProgress,
    paused: Duration,
) {
    let stats = &search.stats;
    ui.label(format!("Paused {:.1}s", paused.as_secs_f64()));
    ui.label(format!(
        "{:.1}s active · {} expanded · {} generated · {} retained · {} peak frontier{}",
        stats.elapsed.as_secs_f64(),
        stats.expanded,
        stats.generated,
        stats.states,
        stats.peak_frontier,
        if search.finished {
            " · verifying replay"
        } else {
            ""
        }
    ));
}

/// Render determined calculation stats and optional step-by-step route inspector.
fn render_tripeaks_determined_report(
    ui: &mut egui::Ui,
    report: &tripeaks_calculation::Report,
    show_route_inspector: Option<&mut bool>,
    id_prefix: &str,
) {
    let stats = tripeaks_calculation::stats(&report.outcome);
    ui.label(format!(
        "Final: {:.2}s active · {} expanded · {} generated · {} retained · {} peak frontier",
        stats.elapsed.as_secs_f64(),
        stats.expanded,
        stats.generated,
        stats.states,
        stats.peak_frontier
    ));
    ui.label(format!(
        "Source: {} ({} bytes, paused {:.2}s)",
        report.path.display(),
        report.bytes.len(),
        report.paused.as_secs_f64()
    ));

    if let Some(toggle) = show_route_inspector {
        ui.horizontal(|ui| {
            let label = if *toggle { "HIDE ROUTE" } else { "VIEW ROUTE" };
            if bevel_button(ui, label, SETUP_BLUE, *toggle, true).clicked() {
                *toggle = !*toggle;
            }
        });
        if *toggle {
            let doc_counts = SlotKnowledgeCounts::from_slots(&report.document.slots);
            ui.small(doc_counts.summary_line(report.document.probes_used));
            match &report.outcome {
                tripeaks_calculation::Outcome::Solved { moves, .. }
                | tripeaks_calculation::Outcome::Partial { moves, .. } => {
                    egui::ScrollArea::vertical()
                        .id_salt(format!("{id_prefix}-route-scroll"))
                        .max_height(140.0)
                        .show(ui, |ui| {
                            if moves.is_empty() {
                                ui.small("Zero post-recon moves queued.");
                            } else {
                                for (idx, mv) in moves.iter().enumerate() {
                                    ui.monospace(format!(
                                        "{:>2}. {}",
                                        idx + 1,
                                        format_tripeaks_move(*mv)
                                    ));
                                }
                            }
                        });
                }
                tripeaks_calculation::Outcome::Cancelled(_) => {
                    ui.small("Calculation cancelled; no route stored.");
                }
                tripeaks_calculation::Outcome::Failed { failure, .. } => {
                    ui.small(format!("No route available: {failure}"));
                }
            }
        }
    }
}

impl QmpQemuSocketApp {
    /// Assemble TriPeaks settle timings from current application parameters.
    pub(super) fn tripeaks_step_timing(&self) -> TriPeaksStepTiming {
        let mut timing = TriPeaksStepTiming::from(&TriPeaksReconConfig::default());
        let move_ms = self
            .tableau_animation_settle_ms
            .max(self.draw_animation_settle_ms)
            .max(1);
        timing.move_settle_delay = Duration::from_millis(move_ms);
        timing.reobserve_delay = Duration::from_millis(self.tripeaks_reobserve_ms.max(1));
        timing
    }

    /// Revoke any idle Manual TriPeaks route candidate cleanly.
    pub(super) fn revoke_tripeaks_playback(&mut self) {
        self.tripeaks_workflow.clear_manual_playback();
        self.worker.revoke_tripeaks_route();
    }

    /// Render the independent TriPeaks workflow controls inside the main panel.
    pub(super) fn tripeaks_workflow_body(&mut self, ui: &mut egui::Ui, idle: bool) {
        let context = self.current_preview_context();
        self.tripeaks_workflow.bind(&context);

        ui.horizontal_wrapped(|ui| {
            ui.label("Mode");
            ui.add_enabled_ui(idle, |ui| {
                egui::ComboBox::from_id_salt("tripeaks-play-style")
                    .selected_text(self.tripeaks_workflow.play_style.label())
                    .show_ui(ui, |ui| {
                        for option in TriPeaksPlayStyle::ALL {
                            ui.selectable_value(
                                &mut self.tripeaks_workflow.play_style,
                                option,
                                option.label(),
                            );
                        }
                    });
            });
        });
        ui.small(self.tripeaks_workflow.play_style.description());

        ui.horizontal(|ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                self.independent_stop_exit(ui);
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    if self.tripeaks_workflow.play_style.uses_session() {
                        if self.active_run == Some(ActiveRun::TriPeaksSession) {
                            if bevel_button(ui, "Stop", STATE_RED, false, !self.stop_requested)
                                .clicked()
                            {
                                self.request_independent_stop();
                            }
                        } else if bevel_button(ui, "START", ACTION_GREEN, false, idle).clicked() {
                            self.press_tripeaks_session();
                        }
                        self.session_pause_cancel_controls(ui);
                    }
                    if bevel_button(ui, "LOG", SETUP_BLUE, false, true).clicked() {
                        self.show_output = true;
                    }
                    if bevel_button(
                        ui,
                        "FAILURE DETAILS",
                        SETUP_BLUE,
                        false,
                        self.failure_window.available(),
                    )
                    .clicked()
                    {
                        self.failure_window.reopen();
                    }
                });
            });
        });

        if self.tripeaks_workflow.play_style.uses_session() {
            ui.label(&self.current_status);
            render_knowledge_and_objective(
                ui,
                self.tripeaks_workflow.counts,
                self.tripeaks_workflow.probes_used,
                self.tripeaks_workflow.objective_label.as_deref(),
            );
            if let Some(phase) = self.tripeaks_workflow.phase {
                let cursor_text = self.tripeaks_workflow.route_cursor.map_or_else(
                    || {
                        self.tripeaks_workflow
                            .expanded_states
                            .map_or_else(String::new, |states| {
                                format!(" · {states} states expanded")
                            })
                    },
                    |(done, total)| format!(" · {done}/{total} moves"),
                );
                ui.small(format!(
                    "Cycle {} · {} · boards cleared: {} · completed games: {}{cursor_text}",
                    self.tripeaks_workflow.deal_generation,
                    phase.label(),
                    self.tripeaks_workflow.completed_boards,
                    self.tripeaks_workflow.completed_games,
                ));
            } else if self.tripeaks_workflow.completed_boards > 0
                || self.tripeaks_workflow.completed_games > 0
            {
                ui.small(format!(
                    "Boards cleared: {} · completed games: {}",
                    self.tripeaks_workflow.completed_boards, self.tripeaks_workflow.completed_games,
                ));
            }

            if self.tripeaks_workflow.phase == Some(SessionPhase::Planning)
                && let Some(snapshot) = &self.tripeaks_workflow.planning_progress
            {
                render_tripeaks_live_search_stats(ui, &snapshot.search, snapshot.paused);
            }
            if let Some(report) = &self.tripeaks_workflow.calculation_report {
                let report = Arc::clone(report);
                let summary = tripeaks_calculation::outcome_summary_message(&report.outcome);
                if self.current_status != summary {
                    ui.label(summary);
                }
                render_tripeaks_determined_report(ui, &report, None, "tripeaks-session");
            }
            if let Some(cards) = self.tripeaks_workflow.unsolvable_max_cleared {
                ui.colored_label(
                    Color32::from_rgb(240, 196, 80),
                    format!(
                        "Unsolvable deal notice (max {cards}/28 tableau clearable; zero Redeal clicks sent). Deal a new game and press START."
                    ),
                );
            }
            if let Some(path) = &self.tripeaks_workflow.export_path {
                ui.small(format!("Last exported deal: {}", path.display()));
            }
            if self.worker_state != WorkerState::Error
                && let Some(report) = &self.tripeaks_workflow.report
            {
                ui.small(report.as_str());
            }
        } else {
            self.tripeaks_manual_controls(ui, &context, idle);
            ui.label(&self.current_status);
        }

        ui.separator();
        self.preview(ui);
    }

    /// Render the Manual 3-stage TriPeaks panel (`RECON`, `CALCULATE`/`VIEW ROUTE`, Playback/`UNDO ALL`).
    fn tripeaks_manual_controls(
        &mut self,
        ui: &mut egui::Ui,
        context: &ControllerContext,
        idle: bool,
    ) {
        render_knowledge_and_objective(
            ui,
            self.tripeaks_workflow.manual_counts,
            self.tripeaks_workflow.manual_probes_used,
            self.tripeaks_workflow.manual_objective_label.as_deref(),
        );

        freecell_control_group(ui, "Initial deal acquisition", |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.add_enabled(
                    idle,
                    egui::TextEdit::singleline(&mut self.tripeaks_workflow.export_path_input)
                        .hint_text("Export path (blank = unique temp file)")
                        .desired_width(260.0),
                );
                if bevel_button(ui, "RECON / Export 52-slot deal", SETUP_BLUE, false, idle)
                    .clicked()
                {
                    self.request_tripeaks_recon();
                }
            });
        });

        freecell_control_group(ui, "Offline route calculation", |ui| {
            ui.horizontal_wrapped(|ui| {
                let mut edited = self.tripeaks_workflow.source_path_input.clone();
                if ui
                    .add_enabled(
                        idle,
                        egui::TextEdit::singleline(&mut edited)
                            .hint_text("TriPeaks 52-slot JSON source path")
                            .desired_width(ui.available_width().min(320.0)),
                    )
                    .changed()
                {
                    self.tripeaks_workflow.source_path_input = edited;
                    self.tripeaks_workflow.source_generation =
                        self.tripeaks_workflow.source_generation.wrapping_add(1);
                    self.revoke_tripeaks_playback();
                }
                if ui
                    .add_enabled(idle, PressableButton::new("Browse…"))
                    .clicked()
                {
                    self.tripeaks_workflow.source_browser_open = true;
                    if self.tripeaks_workflow.source_dir_input.is_empty() {
                        self.tripeaks_workflow.source_dir_input =
                            std::env::temp_dir().to_string_lossy().into_owned();
                    }
                    self.request_tripeaks_sources();
                }
            });

            let calc_active = self.active_run == Some(ActiveRun::TriPeaksCalculation);
            let mutable = !self.stop_requested && !self.exiting;
            ui.horizontal_wrapped(|ui| {
                let can_calc = idle && !self.tripeaks_workflow.source_path_input.trim().is_empty();
                if bevel_button(ui, "CALCULATE", ACTION_GREEN, false, can_calc).clicked() {
                    self.dispatch_tripeaks_calculation();
                }
                if bevel_button(
                    ui,
                    "Cancel calculation",
                    STATE_RED,
                    false,
                    calc_active && mutable,
                )
                .clicked()
                {
                    let _ = self.request_stop();
                }
                if calc_active && let Some(pause) = self.worker.pause_state() {
                    let paused = pause.requested || pause.acknowledged;
                    let label = if paused { "Continue" } else { "Pause" };
                    if bevel_button(ui, label, SETUP_BLUE, paused, mutable).clicked()
                        && let Err(err) = self.worker.request_pause(pause.epoch, !paused)
                    {
                        self.push_log(err);
                    }
                }
            });

            if calc_active && let Some(progress) = &self.tripeaks_workflow.live_calc_progress {
                render_tripeaks_live_search_stats(ui, &progress.search, progress.paused);
            }
            if let Some(report) = &self.tripeaks_workflow.manual_report {
                let report = Arc::clone(report);
                let mut inspector = self.tripeaks_workflow.show_route_inspector;
                render_tripeaks_determined_report(
                    ui,
                    &report,
                    Some(&mut inspector),
                    "tripeaks-manual",
                );
                self.tripeaks_workflow.show_route_inspector = inspector;
            }
        });

        freecell_control_group(ui, "Live route playback", |ui| {
            let route_active = self.active_run == Some(ActiveRun::TriPeaksRoute);
            let plan = self
                .tripeaks_workflow
                .prepared_plan
                .clone()
                .filter(|p| &p.context == context);
            let step_ready = idle
                && plan
                    .as_ref()
                    .is_some_and(|p| self.worker.tripeaks_route_available(p));
            let undo_ready = idle
                && plan
                    .as_ref()
                    .is_some_and(|p| self.worker.tripeaks_undo_available(p));
            let stopping_after_step =
                self.tripeaks_workflow.multi_step_stop_requested == Some(true);
            let mutable = !self.stop_requested && !self.exiting;

            ui.horizontal_wrapped(|ui| {
                if bevel_button(ui, "SINGLE STEP", ACTION_GREEN, false, step_ready).clicked() {
                    self.dispatch_tripeaks_playback(tripeaks_route::PlaybackAction::SingleStep);
                }
                if bevel_button(ui, "MULTI STEP", ACTION_GREEN, false, step_ready).clicked() {
                    self.dispatch_tripeaks_playback(tripeaks_route::PlaybackAction::MultiStep);
                }
                if route_active && self.tripeaks_workflow.multi_step_stop_requested == Some(false) {
                    if bevel_button(ui, "Stop after step", SETUP_BLUE, false, mutable).clicked() {
                        self.request_tripeaks_stop_after_step();
                    }
                } else if stopping_after_step {
                    ui.small("Stopping after current verified step…");
                }
                if bevel_button(ui, "UNDO ALL", SETUP_BLUE, false, undo_ready).clicked() {
                    self.dispatch_tripeaks_playback(tripeaks_route::PlaybackAction::UndoAll);
                }
            });

            if let Some(progress) = &self.tripeaks_workflow.live_route_progress {
                ui.label(format!(
                    "Steps {}/{} · {}/28 tableau cleared · +{} pts · deliveries {}/{}",
                    progress.cursor,
                    progress.total,
                    progress.cards_cleared,
                    progress.score_delta,
                    progress.attempted,
                    progress.acknowledged
                ));
            }
            if !self.tripeaks_workflow.manual_message.is_empty() {
                ui.small(&self.tripeaks_workflow.manual_message);
            }
        });

        if self.tripeaks_workflow.source_browser_open && idle {
            egui::Window::new("Select TriPeaks JSON source").show(ui.ctx(), |ui| {
                ui.text_edit_singleline(&mut self.tripeaks_workflow.source_dir_input);
                if ui.add(PressableButton::new("List directory")).clicked() {
                    self.request_tripeaks_sources();
                }
                let mut picked = None;
                egui::ScrollArea::vertical().max_height(240.0).show_rows(
                    ui,
                    ui.text_style_height(&egui::TextStyle::Body),
                    self.tripeaks_workflow.listed_sources.len(),
                    |ui, range| {
                        for idx in range {
                            if let Some(path) = self.tripeaks_workflow.listed_sources.get(idx)
                                && ui
                                    .selectable_label(
                                        std::path::Path::new(
                                            &self.tripeaks_workflow.source_path_input,
                                        ) == path,
                                        path.display().to_string(),
                                    )
                                    .clicked()
                            {
                                picked = Some(path.clone());
                            }
                        }
                    },
                );
                if let Some(path) = picked {
                    self.tripeaks_workflow.source_path_input = path.to_string_lossy().into_owned();
                    self.tripeaks_workflow.source_generation =
                        self.tripeaks_workflow.source_generation.wrapping_add(1);
                    self.tripeaks_workflow.source_browser_open = false;
                    self.revoke_tripeaks_playback();
                }
                if ui.button("Close").clicked() {
                    self.tripeaks_workflow.source_browser_open = false;
                }
            });
        }
    }

    /// Dispatch automatic TriPeaks session (`Continuous` or `Single board`).
    fn press_tripeaks_session(&mut self) {
        let context = self.current_preview_context();
        if !self.tripeaks_workflow.play_style.uses_session() {
            return;
        }
        let run_id = match self.tripeaks_workflow.begin_session_run(&context) {
            Ok(id) => id,
            Err(error) => {
                self.failure_window.general(
                    "TriPeaks session",
                    error,
                    serde_json::json!({"mode": self.tripeaks_workflow.play_style.label()}),
                );
                return;
            }
        };
        self.active_run = Some(ActiveRun::TriPeaksSession);
        self.stop_requested = false;
        self.worker_state = WorkerState::Validating;
        self.current_status = format!(
            "Starting TriPeaks {}…",
            self.tripeaks_workflow.play_style.label()
        );
        self.revoke_tripeaks_playback();
        self.invalidate_preview();
        let request = TriPeaksSessionRequest {
            run_id,
            play_style: self.tripeaks_workflow.play_style,
            step_timing: self.tripeaks_step_timing(),
            board_redeal_settle: Duration::from_millis(self.board_redeal_settle_ms),
        };
        self.push_log(format!(
            "Explicit TriPeaks {} start requested (move settle {} ms).",
            request.play_style.label(),
            request.step_timing.move_settle_delay.as_millis()
        ));
        if let Err(error) = self.worker.start_tripeaks_session(context, request) {
            self.tripeaks_workflow.active_run_id = None;
            self.tripeaks_workflow.phase = None;
            self.active_run = None;
            self.worker_state = WorkerState::Error;
            self.current_status = "TriPeaks session dispatch failed".to_owned();
            let msg = format!("TriPeaks session request refused: {error}");
            self.tripeaks_workflow.report = Some(msg.clone());
            self.push_log(msg.clone());
            self.failure_window.general(
                "TriPeaks session",
                &msg,
                serde_json::json!({"mode": self.tripeaks_workflow.play_style.label()}),
            );
        }
    }

    /// Dispatch Manual `RECON / Export 52-slot deal`.
    fn request_tripeaks_recon(&mut self) {
        if !controls_are_mutable(self.active_run, self.worker_state, self.stop_requested)
            || self.exiting
        {
            return;
        }
        let context = self.current_preview_context();
        let Some(next_id) = self.tripeaks_workflow.recon_serial.checked_add(1) else {
            self.failure_window.general(
                "TriPeaks RECON",
                "RECON request serial exhausted",
                serde_json::json!({}),
            );
            return;
        };
        self.tripeaks_workflow.recon_serial = next_id;
        let trimmed = self.tripeaks_workflow.export_path_input.trim();
        let destination = if trimmed.is_empty() {
            std::env::temp_dir().join(format!(
                "tripeaks-manual-{}-{next_id}.json",
                std::process::id()
            ))
        } else {
            PathBuf::from(trimmed)
        };
        self.revoke_tripeaks_playback();
        self.invalidate_preview();
        self.tripeaks_workflow.active_recon = Some((context.clone(), next_id));
        match self
            .worker
            .reconnoitre_tripeaks(context.clone(), next_id, destination.clone())
        {
            Ok(()) => {
                self.active_run = Some(ActiveRun::TriPeaksRecon);
                self.stop_requested = false;
                self.worker_state = WorkerState::Connecting;
                self.current_status = "TriPeaks: running live RECON and Stock sweep…".to_owned();
                self.tripeaks_workflow.manual_counts = None;
                self.tripeaks_workflow.manual_probes_used = 0;
                self.tripeaks_workflow.manual_objective_label = None;
                self.tripeaks_workflow.manual_message =
                    format!("Export destination: {}", destination.display());
            }
            Err(error) => {
                self.tripeaks_workflow.active_recon = None;
                self.worker_state = WorkerState::Error;
                self.current_status = error.clone();
                self.tripeaks_workflow.manual_message = error.clone();
                self.failure_window.general(
                    "TriPeaks RECON dispatch",
                    &error,
                    serde_json::json!({"request_id": next_id, "destination": destination}),
                );
            }
        }
    }

    /// Dispatch background JSON directory enumeration for the Manual browser window.
    fn request_tripeaks_sources(&mut self) {
        let Some(next_id) = self.tripeaks_workflow.source_list_serial.checked_add(1) else {
            return;
        };
        self.tripeaks_workflow.source_list_serial = next_id;
        self.tripeaks_workflow.source_list_pending = Some(next_id);
        let dir = PathBuf::from(self.tripeaks_workflow.source_dir_input.trim());
        if let Err(err) =
            self.worker
                .list_tripeaks_sources(self.current_preview_context(), next_id, dir)
        {
            self.tripeaks_workflow.source_list_pending = None;
            self.tripeaks_workflow.manual_message = err;
        }
    }

    /// Dispatch offline `CALCULATE` against the selected JSON deal file.
    fn dispatch_tripeaks_calculation(&mut self) {
        if !controls_are_mutable(self.active_run, self.worker_state, self.stop_requested)
            || self.exiting
        {
            return;
        }
        let Some(req_id) = self.tripeaks_workflow.calc_serial.checked_add(1) else {
            return;
        };
        self.tripeaks_workflow.calc_serial = req_id;
        let source_gen = self.tripeaks_workflow.source_generation;
        let path = PathBuf::from(self.tripeaks_workflow.source_path_input.trim());
        let context = self.current_preview_context();
        let identity = tripeaks_calculation::Identity {
            context: context.clone(),
            request_id: req_id,
            source_generation: source_gen,
        };
        self.revoke_tripeaks_playback();
        self.tripeaks_workflow.manual_report = None;
        self.tripeaks_workflow.live_calc_progress = None;
        self.tripeaks_workflow.active_calc = Some(identity.clone());
        match self
            .worker
            .calculate_tripeaks(context, req_id, source_gen, path.clone())
        {
            Ok(()) => {
                self.active_run = Some(ActiveRun::TriPeaksCalculation);
                self.worker_state = WorkerState::Validating;
                self.stop_requested = false;
                self.current_status = "Calculating TriPeaks route…".to_owned();
                self.tripeaks_workflow.manual_message =
                    format!("Calculating from {}", path.display());
            }
            Err(error) => {
                self.tripeaks_workflow.active_calc = None;
                self.worker_state = WorkerState::Error;
                self.current_status = error.clone();
                self.tripeaks_workflow.manual_message = error.clone();
                self.failure_window.general(
                    "TriPeaks calculation dispatch",
                    &error,
                    serde_json::json!({"path": path}),
                );
            }
        }
    }

    /// Reserve and dispatch one Manual live playback or `UNDO ALL` action.
    fn dispatch_tripeaks_playback(&mut self, action: tripeaks_route::PlaybackAction) {
        if !controls_are_mutable(self.active_run, self.worker_state, self.stop_requested)
            || self.exiting
        {
            return;
        }
        let Some(plan) = self
            .tripeaks_workflow
            .prepared_plan
            .clone()
            .filter(|p| p.context == self.current_preview_context())
        else {
            return;
        };
        let timing = self.tripeaks_step_timing();
        match self
            .worker
            .execute_tripeaks_route(plan.clone(), action, timing)
        {
            Ok(run_id) => {
                self.active_run = Some(ActiveRun::TriPeaksRoute);
                self.stop_requested = false;
                self.worker_state = WorkerState::Acting;
                self.tripeaks_workflow.pending_route = Some((plan, run_id));
                self.tripeaks_workflow.multi_step_stop_requested =
                    (action == tripeaks_route::PlaybackAction::MultiStep).then_some(false);
                self.tripeaks_workflow.live_route_progress = None;
                self.current_status = match action {
                    tripeaks_route::PlaybackAction::SingleStep => {
                        "Executing single TriPeaks step…".to_owned()
                    }
                    tripeaks_route::PlaybackAction::MultiStep => {
                        "Executing TriPeaks route…".to_owned()
                    }
                    tripeaks_route::PlaybackAction::UndoAll => {
                        "Restoring TriPeaks board via Undo All…".to_owned()
                    }
                };
            }
            Err(error) => {
                self.tripeaks_workflow.manual_message = error.clone();
                self.failure_window.general(
                    "TriPeaks route dispatch",
                    &error,
                    serde_json::json!({"action": format!("{action:?}")}),
                );
            }
        }
    }

    /// Ask the active Manual `MULTI STEP` invocation to stop cleanly after its current step.
    fn request_tripeaks_stop_after_step(&mut self) {
        let Some((plan, run_id)) = self.tripeaks_workflow.pending_route.clone() else {
            return;
        };
        match self.worker.request_tripeaks_stop_after_step(&plan, run_id) {
            Ok(()) => {
                self.tripeaks_workflow.multi_step_stop_requested = Some(true);
            }
            Err(error) => {
                self.push_log(error);
            }
        }
    }

    /// Apply directory enumeration results only for the matching browser request.
    pub(super) fn tripeaks_sources_listed(
        &mut self,
        request_id: u64,
        _directory: PathBuf,
        result: Result<Vec<PathBuf>, String>,
    ) {
        if self.tripeaks_workflow.source_list_pending != Some(request_id) {
            return;
        }
        self.tripeaks_workflow.source_list_pending = None;
        match result {
            Ok(files) => self.tripeaks_workflow.listed_sources = files,
            Err(error) => {
                self.tripeaks_workflow.manual_counts = None;
                self.tripeaks_workflow.manual_message = error;
            }
        }
    }

    /// Update live Manual `RECON` knowledge counters and objective preview.
    pub(super) fn tripeaks_recon_progress(
        &mut self,
        request_id: u64,
        counts: SlotKnowledgeCounts,
        probes_used: usize,
        objective: Option<crate::recon::Objective>,
        challenge: Option<TriPeaksChallenge>,
    ) {
        let context = self.current_preview_context();
        if self.active_run != Some(ActiveRun::TriPeaksRecon)
            || self.tripeaks_workflow.active_recon.as_ref() != Some(&(context, request_id))
        {
            return;
        }
        self.tripeaks_workflow.manual_counts = Some(counts);
        self.tripeaks_workflow.manual_probes_used = probes_used;
        if let Some(obj) = objective {
            self.tripeaks_workflow.manual_objective_label =
                Some(format_tripeaks_objective(obj, challenge));
        }
    }

    /// Handle completion of Manual `RECON / Export 52-slot deal`.
    pub(super) fn tripeaks_recon_finished(
        &mut self,
        request_id: u64,
        termination: RunTermination,
        result: Result<TriPeaksManualReconSummary, String>,
    ) {
        let context = self.current_preview_context();
        if self.tripeaks_workflow.active_recon.as_ref() != Some(&(context.clone(), request_id)) {
            return;
        }
        self.tripeaks_workflow.active_recon = None;
        if self.active_run == Some(ActiveRun::TriPeaksRecon) {
            self.active_run = None;
        }
        let stopped = termination == RunTermination::OperatorStopped || self.stop_requested;
        self.stop_requested = false;
        self.tripeaks_workflow.retained_preview = Some(context);
        match result {
            Ok(summary) => {
                self.tripeaks_workflow.manual_counts = Some(summary.counts);
                self.tripeaks_workflow.manual_probes_used = summary.probes_used;
                self.tripeaks_workflow.manual_objective_label = Some(format_tripeaks_objective(
                    summary.objective,
                    summary.challenge,
                ));
                self.tripeaks_workflow.source_path_input =
                    summary.path.to_string_lossy().into_owned();
                self.tripeaks_workflow.source_generation =
                    self.tripeaks_workflow.source_generation.wrapping_add(1);
                self.tripeaks_workflow.manual_message = summary.summary.clone();
                self.push_log(format!(
                    "{} (root hash {:#018x}, completed_live={}).",
                    summary.summary, summary.root_hash, summary.completed_during_recon
                ));
                self.current_status = summary.summary;
                self.worker_state = WorkerState::Ready;
            }
            Err(error) => {
                let msg = if stopped {
                    format!("TriPeaks RECON stopped: {error}")
                } else {
                    format!("TriPeaks RECON failed: {error}")
                };
                self.tripeaks_workflow.manual_message = msg.clone();
                self.current_status = msg.clone();
                if !stopped {
                    self.failure_window.general(
                        "TriPeaks RECON",
                        &error,
                        serde_json::json!({"request_id": request_id}),
                    );
                    self.worker_state = WorkerState::Uncertain;
                } else if !matches!(
                    self.worker_state,
                    WorkerState::Error | WorkerState::Uncertain
                ) {
                    self.worker_state = WorkerState::Ready;
                }
            }
        }
    }

    /// Consume latest-only live progress emitted by Manual `CALCULATE`.
    pub(super) fn receive_tripeaks_calculation_progress(&mut self) {
        let Some(progress) = self.worker.take_tripeaks_calculation_progress() else {
            return;
        };
        if self.active_run == Some(ActiveRun::TriPeaksCalculation)
            && self.tripeaks_workflow.active_calc.as_ref() == Some(&progress.identity)
        {
            self.tripeaks_workflow.live_calc_progress = Some(progress);
        }
    }

    /// Handle completion of Manual `CALCULATE`.
    pub(super) fn tripeaks_calculation_finished(
        &mut self,
        identity: tripeaks_calculation::Identity,
        result: Result<Arc<tripeaks_calculation::Report>, String>,
    ) {
        if self.tripeaks_workflow.active_calc.as_ref() != Some(&identity) {
            return;
        }
        self.tripeaks_workflow.active_calc = None;
        self.tripeaks_workflow.live_calc_progress = None;
        if self.active_run == Some(ActiveRun::TriPeaksCalculation) {
            self.active_run = None;
        }
        self.stop_requested = false;
        self.worker_state = WorkerState::Ready;
        match result {
            Ok(report) => {
                let doc_counts = SlotKnowledgeCounts::from_slots(&report.document.slots);
                self.tripeaks_workflow.manual_counts = Some(doc_counts);
                self.tripeaks_workflow.manual_probes_used = report.document.probes_used;
                self.tripeaks_workflow.manual_objective_label =
                    Some(format_tripeaks_objective(report.document.objective, None));
                let summary = tripeaks_calculation::outcome_summary_message(&report.outcome);
                self.tripeaks_workflow.manual_message = summary.clone();
                self.current_status = summary;
                let has_candidate = matches!(
                    &report.outcome,
                    tripeaks_calculation::Outcome::Solved { .. }
                        | tripeaks_calculation::Outcome::Partial { .. }
                );
                self.tripeaks_workflow.prepared_plan = has_candidate.then_some(identity);
                self.tripeaks_workflow.manual_report = Some(report);
            }
            Err(error) => {
                self.tripeaks_workflow.prepared_plan = None;
                self.tripeaks_workflow.manual_message = error.clone();
                self.current_status = error.clone();
                self.failure_window.general(
                    "TriPeaks calculation",
                    &error,
                    serde_json::json!({"request_id": identity.request_id}),
                );
            }
        }
    }

    /// Consume coalesced live execution counts during Manual route playback.
    pub(super) fn receive_tripeaks_route_progress(&mut self) {
        let Some(progress) = self.worker.take_tripeaks_route_progress() else {
            return;
        };
        if self.active_run == Some(ActiveRun::TriPeaksRoute)
            && self.tripeaks_workflow.pending_route.as_ref()
                == Some(&(progress.plan.clone(), progress.run_id))
        {
            self.tripeaks_workflow.live_route_progress = Some(progress);
        }
    }

    /// Handle completion of Manual `SINGLE STEP`, `MULTI STEP`, or `UNDO ALL`.
    pub(super) fn tripeaks_route_finished(
        &mut self,
        plan: tripeaks_calculation::Identity,
        run_id: u64,
        result: Result<tripeaks_route::Progress, Box<tripeaks_route::Failure>>,
        termination: RunTermination,
    ) {
        if self.tripeaks_workflow.pending_route.as_ref() != Some(&(plan.clone(), run_id)) {
            return;
        }
        self.tripeaks_workflow.pending_route = None;
        self.tripeaks_workflow.multi_step_stop_requested = None;
        if self.active_run == Some(ActiveRun::TriPeaksRoute) {
            self.active_run = None;
        }
        let stopped = termination == RunTermination::OperatorStopped || self.stop_requested;
        self.stop_requested = false;
        self.tripeaks_workflow.retained_preview = Some(plan.context.clone());

        match result {
            Ok(progress) => {
                self.tripeaks_workflow.live_route_progress = Some(progress.clone());
                let obj = self
                    .tripeaks_workflow
                    .manual_report
                    .as_ref()
                    .map_or(crate::recon::Objective::ClearBoard, |r| {
                        r.document.objective
                    });
                let msg = tripeaks_route::progress_summary_message(&progress, obj);
                self.tripeaks_workflow.manual_message = msg.clone();
                self.current_status = msg;
                if matches!(
                    progress.completion,
                    Some(
                        tripeaks_route::Completion::ObjectiveReached
                            | tripeaks_route::Completion::BoardCleared
                            | tripeaks_route::Completion::NextBoardDealt { .. }
                            | tripeaks_route::Completion::BoardWinOverlay
                            | tripeaks_route::Completion::ChallengeComplete
                            | tripeaks_route::Completion::PartialComplete
                    )
                ) {
                    self.tripeaks_workflow.prepared_plan = None;
                }
                self.worker_state = WorkerState::Ready;
            }
            Err(failure) => {
                self.tripeaks_workflow.manual_message = failure.error.clone();
                self.current_status = failure.error.clone();
                if !stopped {
                    self.failure_window.general(
                        "TriPeaks playback",
                        &failure.error,
                        serde_json::json!({
                            "directory": failure.directory,
                            "persistence_error": failure.persistence_error,
                            "run_id": run_id,
                        }),
                    );
                    self.worker_state = WorkerState::Uncertain;
                } else if !matches!(
                    self.worker_state,
                    WorkerState::Error | WorkerState::Uncertain
                ) {
                    self.worker_state = WorkerState::Ready;
                }
            }
        }
    }

    /// Handle advisory progress emitted during an automatic TriPeaks session.
    pub(super) fn tripeaks_session_progress(&mut self, progress: TriPeaksSessionProgress) {
        let context = self.current_preview_context();
        if self
            .tripeaks_workflow
            .receive_session_progress(&progress, &context, self.active_run)
        {
            self.current_status = progress.message;
        }
    }

    /// Retire active automatic TriPeaks session cleanly and report final results.
    pub(super) fn tripeaks_session_finished(&mut self, completion: TriPeaksSessionCompletion) {
        let context = self.current_preview_context();
        let stopped =
            completion.termination == RunTermination::OperatorStopped || self.stop_requested;
        if !self.tripeaks_workflow.complete_session(
            &mut self.active_run,
            &mut self.worker_state,
            &completion,
            &context,
            stopped,
        ) {
            return;
        }
        if let Some(msg) = &self.tripeaks_workflow.report {
            self.current_status = msg.clone();
        }
        if !stopped && let Err(error) = &completion.result {
            self.failure_window.general(
                "TriPeaks session",
                error,
                serde_json::json!({
                    "run_id": completion.run_id,
                    "completed_boards": completion.completed_boards,
                    "completed_games": completion.completed_games,
                    "export_path": completion.export_path,
                    "failure_directory": completion.failure_directory,
                }),
            );
        }
        self.stop_requested = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        recon::Objective,
        tripeaks_model::ReplayReport,
        worker::{tripeaks_calculation, tripeaks_route},
    };
    use xsar::games::cards::Rank;

    /// Construct a TriPeaks controller context for UI unit state tests.
    fn test_context(gen_num: u64) -> ControllerContext {
        ControllerContext::new(
            PathBuf::from("/tmp/tripeaks-ui-test.sock"),
            GameMode::TriPeaks,
            SolvingStrategy::ShortestPath,
            gen_num,
        )
    }

    /// Mode switching, session lifecycle transitions, and human-readable move formatting stay consistent.
    #[test]
    fn mode_switching_session_progress_completion_and_move_formatting() {
        let mut wf = TriPeaksWorkflowState::default();
        assert_eq!(wf.play_style, TriPeaksPlayStyle::Continuous);

        let ctx = test_context(1);
        let run_id = wf.begin_session_run(&ctx).expect("begin first session run");
        assert!(wf.begin_session_run(&ctx).is_err());

        let mut owner = Some(ActiveRun::TriPeaksSession);
        assert_eq!(wf.activity_phase(&ctx), Some(PyramidPhase::Reconnaissance));

        let counts = SlotKnowledgeCounts {
            known: 42,
            unknown: 10,
            unreachable: 0,
        };
        assert!(wf.receive_session_progress(
            &TriPeaksSessionProgress {
                context: ctx.clone(),
                run_id,
                deal_generation: 1,
                completed_boards: 1,
                completed_games: 0,
                phase: SessionPhase::Execution,
                counts: Some(counts),
                probes_used: 3,
                objective_label: Some("Clear board (all 28 tableau cards)".to_owned()),
                route_cursor: Some((5, 28)),
                expanded_states: Some(128),
                planning_progress: None,
                calculation_report: None,
                export_path: Some(PathBuf::from("/tmp/tp-1.json")),
                message: "Executing TriPeaks route".to_owned(),
            },
            &ctx,
            owner,
        ));
        assert_eq!(wf.activity_phase(&ctx), Some(PyramidPhase::Execution));
        assert_eq!(wf.route_cursor, Some((5, 28)));
        assert_eq!(wf.probes_used, 3);

        let mut state = WorkerState::Acting;
        assert!(wf.complete_session(
            &mut owner,
            &mut state,
            &TriPeaksSessionCompletion {
                context: ctx.clone(),
                run_id,
                completed_boards: 3,
                completed_games: 1,
                termination: RunTermination::Completed,
                counts: Some(counts),
                objective_label: Some("Clear board".to_owned()),
                unsolvable_max_cleared: None,
                calculation_report: None,
                export_path: Some(PathBuf::from("/tmp/tp-1.json")),
                failure_directory: None,
                result: Ok("Completed 3/3 normal TriPeaks boards.".to_owned()),
            },
            &ctx,
            false,
        ));
        assert!(owner.is_none());
        assert_eq!(state, WorkerState::Ready);
        assert_eq!(wf.completed_games, 1);
        assert!(wf.retain_after_detach(&ctx));

        // Context generation bump clears prior session summary cleanly.
        let ctx2 = test_context(2);
        wf.bind(&ctx2);
        assert_eq!(wf.completed_games, 0);
        assert!(wf.activity_phase(&ctx2).is_none());

        assert_eq!(
            format_tripeaks_move(TriPeaksMove::TapTableau { row: 3, column: 0 }),
            "Tap R4C1"
        );
        assert_eq!(format_tripeaks_move(TriPeaksMove::Draw), "Draw (D)");
    }

    /// Operator labels, progress summaries, and objective strings must never contain banned jargon.
    #[test]
    fn operator_copy_contains_zero_jargon_occurrences() {
        let banned_bytes = [0x65_u8, 0x70, 0x69, 0x73, 0x74, 0x65, 0x6d, 0x69, 0x63];
        let banned_word = std::str::from_utf8(&banned_bytes).expect("ascii");
        let mut samples = Vec::new();
        for style in TriPeaksPlayStyle::ALL {
            samples.push(style.label().to_owned());
            samples.push(style.description().to_owned());
        }
        let counts = SlotKnowledgeCounts {
            known: 38,
            unknown: 14,
            unreachable: 0,
        };
        samples.push(counts.summary_line(4));
        for (obj, ch) in [
            (Objective::ClearBoard, None),
            (
                Objective::ClearRankQuota {
                    rank: 3,
                    remaining: 8,
                },
                Some(TriPeaksChallenge::ClearRank {
                    rank: Rank::Four,
                    current: 4,
                    target: 12,
                }),
            ),
            (
                Objective::ReachScoreDelta {
                    points_needed: 1500,
                },
                Some(TriPeaksChallenge::Score {
                    current: 500,
                    target: 2000,
                }),
            ),
        ] {
            samples.push(format_tripeaks_objective(obj, ch));
        }

        let slots = [crate::recon::CardSlot::Unknown; crate::tripeaks_model::DEAL_SLOT_COUNT];
        let empty_pos = crate::tripeaks_model::TriPeaksPosition {
            tableau_mask: crate::tripeaks_model::FULL_TABLEAU_MASK,
            stock_cursor: 0,
            waste_card: crate::recon::Card(0),
            streak: 0,
            score_delta: 0,
            cleared_by_rank: [0; 13],
        };
        let _ = slots;
        let dummy_replay = ReplayReport {
            final_position: empty_pos,
            steps_executed: 0,
            newly_uncovered_unknown_slots: Vec::new(),
        };
        samples.push(tripeaks_calculation::outcome_summary_message(
            &tripeaks_calculation::Outcome::Solved {
                objective: Objective::ClearBoard,
                moves: Vec::new(),
                replay: dummy_replay.clone(),
                stats: xsar::search::SearchStats::default(),
                escalated_clear_board: false,
            },
        ));
        samples.push(tripeaks_calculation::outcome_summary_message(
            &tripeaks_calculation::Outcome::Partial {
                objective: Objective::ClearBoard,
                moves: Vec::new(),
                replay: dummy_replay,
                max_tableau_cleared: 0,
                banked_rank_count: 0,
                stats: xsar::search::SearchStats::default(),
            },
        ));
        let identity = tripeaks_calculation::Identity {
            context: test_context(9),
            request_id: 1,
            source_generation: 1,
        };
        for comp in [
            tripeaks_route::Completion::Step,
            tripeaks_route::Completion::ObjectiveReached,
            tripeaks_route::Completion::BoardCleared,
            tripeaks_route::Completion::NextBoardDealt { root_hash: 1 },
            tripeaks_route::Completion::BoardWinOverlay,
            tripeaks_route::Completion::ChallengeComplete,
            tripeaks_route::Completion::PartialComplete,
            tripeaks_route::Completion::RestoredToRoot,
        ] {
            samples.push(tripeaks_route::progress_summary_message(
                &tripeaks_route::Progress {
                    plan: identity.clone(),
                    run_id: 1,
                    cursor: 1,
                    total: 10,
                    attempted: 1,
                    acknowledged: 1,
                    score_delta: 100,
                    cards_cleared: 1,
                    completion: Some(comp),
                },
                Objective::ClearBoard,
            ));
        }

        for sample in samples {
            assert!(
                !sample.to_ascii_lowercase().contains(banned_word),
                "unexpected jargon in UI text: {sample}"
            );
        }
    }
}
