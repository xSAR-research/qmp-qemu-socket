//! Identity-fenced, plan-only Free Cell controls. No guest/execution permission.

use super::*;
use crate::worker::freecell_calculation::{Identity, Progress, Report};
use std::sync::Arc;


/// Offline source selection and one outstanding request, independent of RECON.
#[derive(Default)]
pub(super) struct Calculation {
    /// Operator's exact source pathname.
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
    /// Latest browser result bound to its directory.
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
    pub(super) fn sources(&mut self, directory: PathBuf, result: Result<Vec<PathBuf>, String>) {
        if directory != std::path::Path::new(&self.directory) {
            return;
        }
        match result {
            Ok(files) => self.files = files,
            Err(error) => self.message = error,
        }
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
        self.progress = None;
        self.report = None;
        self.route_open = false;
        self.message = "Calculating complete route — no configured search cutoffs".to_owned();
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
                use xsar::games::freecell::search::Outcome;
                self.message = match &report.outcome {
                    Outcome::Solved { moves, replay, .. } => format!(
                        "Complete route independently replayed: {} transfers; {} deliberate, {} automatic; 52 foundation cards",
                        moves.len(),
                        replay.deliberate,
                        replay.automatic
                    ),
                    Outcome::Exhausted(_) => "Search graph exhausted: no complete route".to_owned(),
                    Outcome::Cancelled(_) => "Calculation cancelled; no usable route".to_owned(),
                    Outcome::Failed { failure, .. } => format!("Calculation failed: {failure}"),
                };
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


impl QmpQemuSocketApp {
    /// Offer the complete offline calculation independently of live RECON/export.
    pub(super) fn freecell_calculation_controls(&mut self, ui: &mut egui::Ui) {
        let idle = self.active_run.is_none() && !self.worker_state.is_busy() && !self.exiting;
        let mut source = self.freecell_calculation.path.clone();
        let changed = ui
            .add_enabled(
                idle,
                egui::TextEdit::singleline(&mut source)
                    .hint_text("Free Cell source JSON path")
                    .desired_width(330.0),
            )
            .changed();
        if changed {
            match self.freecell_calculation.set_source(source) {
                Ok(()) => self.revoke_freecell_playback(),
                Err(error) => self.freecell_calculation.message = error,
            }
        }
        if bevel_button(ui, "Select JSON…", SETUP_BLUE, false, idle).clicked() {
            self.freecell_calculation.browser = true;
            if self.freecell_calculation.directory.is_empty() {
                self.freecell_calculation.directory = "/home/charlie/tmp".to_owned();
            }
            if let Err(error) = self.worker.list_freecell_sources(
                self.current_preview_context(),
                PathBuf::from(&self.freecell_calculation.directory),
            ) {
                self.freecell_calculation.message = error;
            }
        }
        if self.freecell_calculation.browser && idle {
            egui::Window::new("Select Free Cell JSON source").show(ui.ctx(), |ui| {
                ui.text_edit_singleline(&mut self.freecell_calculation.directory);
                if ui.button("List directory").clicked()
                    && let Err(error) = self.worker.list_freecell_sources(
                        self.current_preview_context(),
                        PathBuf::from(&self.freecell_calculation.directory),
                    )
                {
                    self.freecell_calculation.message = error;
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
                                    .selectable_label(false, path.display().to_string())
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
                            Ok(()) => self.revoke_freecell_playback(),
                            Err(error) => self.freecell_calculation.message = error,
                        },
                        None => {
                            self.freecell_calculation.message = "Unsupported filename".to_owned()
                        }
                    }
                }
                if ui.button("Close").clicked() {
                    self.freecell_calculation.browser = false;
                }
            });
        }
        if bevel_button(ui, "CALCULATE", SETUP_BLUE, false, idle).clicked() {
            self.revoke_freecell_playback();
            let context = self.current_preview_context();
            match self.freecell_calculation.reserve(context) {
                Ok((identity, path)) => {
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
                            self.freecell_calculation.finish(&identity, Err(error));
                        }
                    }
                }
                Err(error) => self.freecell_calculation.message = error,
            }
        }
        if self.active_run == Some(ActiveRun::FreeCellCalculation) {
            if let Some(pause) = self.worker.pause_state() {
                let requested = pause.requested || pause.acknowledged;
                if ui
                    .button(if requested { "CONTINUE" } else { "PAUSE" })
                    .clicked()
                    && let Err(error) = self.worker.request_pause(pause.epoch, !requested)
                {
                    self.freecell_calculation.message = error;
                }
                if pause.acknowledged {
                    ui.label("Paused — frontier retained");
                } else if pause.requested {
                    ui.label("Pausing…");
                }
            }
            if ui.button("CANCEL").clicked() {
                self.worker.cancel_freecell_calculation();
                self.freecell_calculation.message = "Cancelling…".to_owned();
            }
        }
        if let Some(progress) = self
            .freecell_calculation
            .progress
            .as_ref()
            .filter(|_| self.freecell_calculation.pending.is_some())
        {
            let stats = &progress.search.search.stats;
            ui.label(format!("Paused {:.1}s", progress.paused.as_secs_f64()));
            ui.label(format!("{:.1}s active · {} expanded · {} generated · {} retained · {} peak frontier · best foundations {}/52{}", stats.elapsed.as_secs_f64(), stats.expanded, stats.generated, stats.states, stats.peak_frontier, progress.search.best_foundation_cards, if progress.search.search.finished { " · checking final replay" } else { "" }));
        }
        ui.label(&self.freecell_calculation.message);
        if let Some(report) = &self.freecell_calculation.report {
            let stats = crate::worker::freecell_calculation::stats(&report.outcome);
            ui.label(format!("Final: {:.2}s active · {} expanded · {} generated · {} retained · {} peak frontier", stats.elapsed.as_secs_f64(), stats.expanded, stats.generated, stats.states, stats.peak_frontier));
            ui.label(format!("Paused {:.2}s", report.paused.as_secs_f64()));
            ui.label(format!(
                "Frozen source {} · {} bytes · deal {}",
                report.path.display(),
                report.bytes.len(),
                report.deal.metadata().deal_id()
            ));
            if matches!(
                &report.outcome,
                xsar::games::freecell::search::Outcome::Solved { .. }
            ) && ui.button("Inspect physical route").clicked()
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
        let xsar::games::freecell::search::Outcome::Solved { moves, .. } = &report.outcome else {
            calculation.route_open = false;
            return;
        };
        let mut open = calculation.route_open;
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
        if self.active_run == Some(ActiveRun::FreeCellCalculation)
            && identity.context == self.current_preview_context()
            && self.freecell_calculation.finish(&identity, result)
        {
            let stopped = self.stop_requested;
            if stopped {
                self.revoke_freecell_playback();
            } else if self.freecell_calculation.report.as_ref().is_some_and(|r| {
                matches!(
                    &r.outcome,
                    xsar::games::freecell::search::Outcome::Solved { .. }
                )
            }) {
                self.freecell_playback.plan = Some(identity.clone());
            }
            self.active_run = None;
            self.stop_requested = false;
            self.worker_state = WorkerState::Ready;
            self.current_status = self.freecell_calculation.message.clone();
            self.push_log(self.current_status.clone());
            if let Some(report) = self.freecell_calculation.report.take() {
                if let xsar::games::freecell::search::Outcome::Solved { moves, .. } =
                    &report.outcome
                {
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
