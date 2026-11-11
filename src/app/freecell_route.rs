//! Explicit live Free Cell playback controls, separate from offline calculation.
use super::*;
use crate::{
    pyramid_workflow_model::RunTermination,
    worker::{freecell_calculation::Identity, freecell_route::Progress},
};


/// UI correlation only; the worker owns the candidate, cursor and live proof.
#[derive(Default)]
pub(super) struct Playback {
    /// Displayed independently replayed calculation, never a live-board receipt.
    pub(super) plan: Option<Identity>,
    /// Exact outstanding invocation until its dedicated completion.
    pending: Option<(Identity, u64)>,
    /// Coalesced semantic counts, separate from delivery acknowledgement.
    progress: Option<Progress>,
    /// Current result, retained even when the executable candidate is retired.
    message: String,
    /// Application transfer policy copied into the next explicit worker invocation.
    mode: crate::freecell_route::AutoMode,
    /// Diagnostic pointer protocol copied into each explicit SINGLE STEP request.
    drag_mode: crate::freecell_route::DragMode,
    /// Preserve already acquired pixels through queued STOP disconnection.
    pub(super) retained: Option<ControllerContext>,
}


/// Show both diagnostic choices without scheduling input or mutating a running request.
fn diagnostic_drag_selector(
    ui: &mut egui::Ui,
    mode: &mut crate::freecell_route::DragMode,
    enabled: bool,
) -> [egui::Response; 2] {
    ui.add_enabled_ui(enabled, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label("Single Step drag:");
            [
                ui.selectable_value(mode, crate::freecell_route::DragMode::Absolute, "ABS")
                    .on_hover_text("Absolute held drag moves 45 native pixels above the source, through the native midpoint of the original source and effective destination, 45 pixels above the destination, then to the exact destination; exact endpoint proof remains required"),
                ui.selectable_value(mode, crate::freecell_route::DragMode::Relative, "REL")
                    .on_hover_text("Ten tiny relative held motions; diagnostic only, retires the route without cursor progress"),
            ]
        })
        .inner
    })
    .inner
}


impl Playback {
    /// Enable explicit requests only for the displayed source and an idle verified cache.
    fn ready(
        &self,
        context: &ControllerContext,
        idle: bool,
        available: impl FnOnce(&Identity) -> bool,
    ) -> bool {
        idle && self.pending.is_none()
            && self
                .plan
                .as_ref()
                .is_some_and(|plan| &plan.context == context && available(plan))
    }


    /// Keep an existing exact cache after failure without ever restoring revoked consent.
    fn failed(&mut self, plan: &Identity, error: &str, available: bool, stopped: bool) -> bool {
        let retry = !stopped && available && self.plan.as_ref() == Some(plan);
        if retry {
            self.message = format!(
                "{error}; after manual UNDO ALL, request Single Step or SOLVE remaining route again. The cached original route restarts from transfer 1 after fresh recognition of the same 52-card board."
            );
        } else {
            self.revoke();
            self.message = error.to_owned();
        }
        retry
    }


    /// Remove executable consent without releasing outstanding completion ownership.
    fn revoke(&mut self) {
        self.plan = None;
    }


    /// Accept progress only for the still-outstanding exact plan and invocation.
    fn progress(&mut self, progress: Progress) {
        if self.pending.as_ref() == Some(&(progress.plan.clone(), progress.run_id)) {
            self.progress = Some(progress);
        }
    }


    /// Consume one terminal identity; duplicate or unrelated work cannot retire it.
    fn finish(&mut self, plan: &Identity, run_id: u64) -> bool {
        if self.pending.as_ref() != Some(&(plan.clone(), run_id)) {
            return false;
        }
        self.pending = None;
        self.retained = Some(plan.context.clone());
        true
    }
}


impl QmpQemuSocketApp {
    /// Revoke executable state without releasing a running owner or erasing its report.
    pub(super) fn revoke_freecell_playback(&mut self) {
        self.freecell_playback.revoke();
        self.worker.revoke_freecell_route();
    }


    /// Dispatch one explicit transfer plus proved recorded follow-ups, or the remaining route.
    fn dispatch_freecell_route(
        &mut self,
        single_step: bool,
        drag_timing: crate::freecell_route::DiagnosticDragTiming,
    ) {
        if !controls_are_mutable(self.active_run, self.worker_state, self.stop_requested)
            || self.exiting
        {
            return;
        }
        let Some(plan) = self
            .freecell_playback
            .plan
            .clone()
            .filter(|p| p.context == self.current_preview_context())
        else {
            return;
        };
        let settle = Duration::from_millis(self.freecell_settle_ms);
        let drag_mode = self.freecell_playback.drag_mode;
        self.trace_freecell_ui(
            "playback-dispatch",
            serde_json::json!({
            "plan":format!("{plan:?}"),"request_id":plan.request_id,
            "source_generation":plan.source_generation,"single_step":single_step,
            "autoplay_mode":format!("{:?}",self.freecell_playback.mode),
            "diagnostic_drag_mode":single_step.then(||format!("{drag_mode:?}")),
            "diagnostic_drag_timing":single_step.then(||format!("{drag_timing:?}")),
            "settle_ms":self.freecell_settle_ms}),
        );
        let result = if single_step {
            self.worker.execute_freecell_route_with_timing(
                plan.clone(),
                true,
                self.freecell_playback.mode,
                drag_mode,
                drag_timing,
                settle,
            )
        } else {
            self.worker.execute_freecell_route(
                plan.clone(),
                false,
                self.freecell_playback.mode,
                drag_mode,
                settle,
            )
        };
        match result {
            Ok(run_id) => {
                self.trace_freecell_ui(
                    "playback-dispatched",
                    serde_json::json!({
                    "plan":format!("{plan:?}"),"run_id":run_id,"single_step":single_step,
                    "diagnostic_drag_mode":single_step.then(||format!("{drag_mode:?}")),
                    "diagnostic_drag_timing":single_step.then(||format!("{drag_timing:?}"))}),
                );
                self.freecell_playback.pending = Some((plan, run_id));
                self.freecell_playback.progress = None;
                self.freecell_playback.retained = None;
                self.freecell_playback.message = if single_step
                    && drag_mode == crate::freecell_route::DragMode::Relative
                {
                    "Checking the live physical board before bounded REL motion probe; route will retire".into()
                } else {
                    "Checking the live physical board before route input".into()
                };
                self.active_run = Some(ActiveRun::FreeCellRoute);
                self.worker_state = WorkerState::Validating;
            }
            Err(error) => {
                self.trace_freecell_ui(
                    "playback-dispatch-failed",
                    serde_json::json!({
                    "plan":format!("{plan:?}"),"error":error}),
                );
                let retry = self.freecell_playback.failed(
                    &plan,
                    &error,
                    self.worker.freecell_route_available(&plan),
                    self.stop_requested,
                );
                if !retry {
                    self.revoke_freecell_playback();
                }
                self.failure_window.general(
                    "Free Cell playback dispatch",
                    &error,
                    serde_json::json!({"context":format!("{plan:?}"),"phase":"dispatch"}),
                );
            }
        }
    }


    /// Explain application transfer policy without claiming to change Solitaire settings.
    pub(super) fn freecell_route_controls(&mut self, ui: &mut egui::Ui) {
        let idle = controls_are_mutable(self.active_run, self.worker_state, self.stop_requested)
            && !self.exiting;
        let ready = self
            .freecell_playback
            .ready(&self.current_preview_context(), idle, |plan| {
                self.worker.freecell_route_available(plan)
            });
        ui.label("Keep Solver off. This transfer policy does not change Solitaire settings.");
        ui.add_enabled_ui(idle, |ui| {
            egui::ComboBox::from_id_salt("freecell-autoplay-policy")
                .width(ui.available_width())
                .selected_text(match self.freecell_playback.mode {
                    crate::freecell_route::AutoMode::ObserveGame => {
                        "Observe recorded AUTO follow-ups"
                    }
                    crate::freecell_route::AutoMode::ExecuteRecorded => {
                        "Execute each recorded transfer"
                    }
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.freecell_playback.mode,
                        crate::freecell_route::AutoMode::ObserveGame,
                        "Observe recorded AUTO follow-ups",
                    );
                    ui.selectable_value(
                        &mut self.freecell_playback.mode,
                        crate::freecell_route::AutoMode::ExecuteRecorded,
                        "Execute each recorded transfer",
                    );
                });
        });
        ui.label("ABS Single Step proves a route move. REL Single Step is a bounded movement probe: no route progress; CALCULATE again before further input.");
        ui.label(match self.freecell_playback.mode {
            crate::freecell_route::AutoMode::ObserveGame => {
                "Single Step confirms the current board, credits already observed recorded AUTOs, then sends ONE explicit drag even when the next record is AUTO. It waits for two fresh exact matches of the combined transfer and recorded AUTO follow-ups, credits them and stops."
            }
            crate::freecell_route::AutoMode::ExecuteRecorded => {
                "Single Step confirms the current board, sends ONE explicit recorded transfer, even when marked AUTO, then requires two fresh exact endpoint matches and stops."
            }
        });
        ui.label("Expected AUTOs still require observed board proof. Missing or different effects stop within six observations. After failure or changing transfer policy, manually UNDO ALL before the next explicit request: the cached original route restarts only after fresh recognition of the same 52-card board.");
        diagnostic_drag_selector(
            ui,
            &mut self.freecell_playback.drag_mode,
            idle && self.freecell_playback.pending.is_none(),
        );
        ui.label("ABS positions the source, waits after its acknowledgement, then sends LEFT down, a point 45 native pixels above the source, the midpoint of the original source and effective destination (each native axis rounded down), a point 45 pixels above the destination, the exact destination and LEFT up. CELL/SUIT destinations use their fixed slot centre. Single Step waits 2 seconds at all six intervals (at least 12 seconds total); Single Step 0.5s waits 0.5 seconds at all six intervals (at least 3 seconds total). The 0.5s button is ABS-only. REL retains its existing 2-second diagnostic waits. No separate source or destination click.");
        ui.label("The original deal must match. Each step verifies the resulting physical board; unsupported or obscured positions stop playback.");
        let buttons = playback_buttons(ui, ready, self.freecell_playback.drag_mode);
        if buttons[0].clicked() {
            self.dispatch_freecell_route(
                true,
                crate::freecell_route::DiagnosticDragTiming::TwoSeconds,
            );
        }
        if buttons[1].clicked() {
            self.dispatch_freecell_route(
                true,
                crate::freecell_route::DiagnosticDragTiming::HalfSecond,
            );
        }
        if buttons[2].clicked() {
            self.dispatch_freecell_route(
                false,
                crate::freecell_route::DiagnosticDragTiming::TwoSeconds,
            );
        }
        if self.active_run == Some(ActiveRun::FreeCellRoute)
            && let Some(snapshot) = self.worker.pause_state()
        {
            let paused = snapshot.requested || snapshot.acknowledged;
            if ui
                .button(if paused {
                    "CONTINUE playback"
                } else {
                    "PAUSE playback"
                })
                .clicked()
            {
                self.trace_freecell_ui(
                    "playback-pause-intent",
                    serde_json::json!({
                    "pending":format!("{:?}",self.freecell_playback.pending),
                    "pause_epoch":snapshot.epoch,"pause_requested":!paused}),
                );
                if let Err(error) = self.worker.request_pause(snapshot.epoch, !paused) {
                    self.trace_freecell_ui(
                        "playback-pause-failed",
                        serde_json::json!({
                        "pending":format!("{:?}",self.freecell_playback.pending),"error":error}),
                    );
                    self.failure_window.general(
                        "Free Cell playback pause",
                        &error,
                        serde_json::json!({"pending":format!("{:?}",self.freecell_playback.pending),
                            "phase":"pause request", "epoch":snapshot.epoch}),
                    );
                    self.freecell_playback.message = error;
                }
            }
            if ui.button("STOP playback").clicked() {
                self.trace_freecell_ui(
                    "playback-stop-intent",
                    serde_json::json!({
                    "pending":format!("{:?}",self.freecell_playback.pending)}),
                );
                if let Err(error) = self.request_stop() {
                    self.trace_freecell_ui(
                        "playback-stop-failed",
                        serde_json::json!({
                        "pending":format!("{:?}",self.freecell_playback.pending),"error":error}),
                    );
                    self.failure_window.general(
                        "Free Cell playback STOP",
                        &error,
                        serde_json::json!({"pending":format!("{:?}",self.freecell_playback.pending),
                            "phase":"STOP request"}),
                    );
                    self.freecell_playback.message = error;
                }
            }
        }
        if let Some(progress) = &self.freecell_playback.progress {
            ui.label(format!(
                "Proved transfers {}/{} · drags attempted {} · acknowledged {}",
                progress.cursor, progress.total, progress.attempted, progress.acknowledged
            ));
        }
        ui.label(&self.freecell_playback.message);
    }


    /// Release only matching dedicated ownership; STOP never schedules another playback capture.
    pub(super) fn freecell_route_finished(
        &mut self,
        plan: Identity,
        run_id: u64,
        result: Result<Progress, Box<crate::worker::freecell_route::Failure>>,
        termination: RunTermination,
    ) {
        if self.active_run != Some(ActiveRun::FreeCellRoute)
            || plan.context != self.current_preview_context()
            || !self.freecell_playback.finish(&plan, run_id)
        {
            self.trace_freecell_ui(
                "playback-result-rejected",
                serde_json::json!({
                "plan":format!("{plan:?}"),"run_id":run_id,
                "termination":format!("{termination:?}"),"reason":"stale or mismatched owner"}),
            );
            return;
        }
        self.trace_freecell_ui(
            "playback-result-accepted",
            serde_json::json!({
            "plan":format!("{plan:?}"),"run_id":run_id,"termination":format!("{termination:?}"),
            "error":result.as_ref().err().map(|failure|&failure.error)}),
        );
        self.active_run = None;
        match result {
            Ok(progress) => {
                self.freecell_playback.message = success_message(&progress, self.stop_requested);
                if progress.completion != Some(crate::freecell_route::Completion::StepVerified) {
                    self.freecell_playback.revoke();
                }
                self.freecell_playback.progress = Some(progress);
                self.worker_state = WorkerState::Ready;
            }
            Err(failure) => {
                let retry = self.freecell_playback.failed(
                    &plan,
                    &failure.error,
                    self.worker.freecell_route_available(&plan),
                    self.stop_requested || termination == RunTermination::OperatorStopped,
                );
                if !retry {
                    self.revoke_freecell_playback();
                }
                if termination != RunTermination::OperatorStopped {
                    self.failure_window.freecell_route(&failure);
                }
                self.worker_state = if retry || termination == RunTermination::OperatorStopped {
                    WorkerState::Ready
                } else {
                    WorkerState::Uncertain
                };
            }
        }
        if self.stop_requested {
            self.revoke_freecell_playback();
        }
        self.current_status = self.freecell_playback.message.clone();
        self.push_log(self.current_status.clone());
    }


    /// Drain after ordered completions, so final events cannot be revived by late progress.
    pub(super) fn receive_freecell_route_progress(&mut self) {
        if let Some(progress) = self.worker.take_freecell_route_progress() {
            self.freecell_playback.progress(progress);
        }
    }
}


/// Render explicit requests; the half-second diagnostic requires idle ABS playback.
fn playback_buttons(
    ui: &mut egui::Ui,
    ready: bool,
    drag_mode: crate::freecell_route::DragMode,
) -> [egui::Response; 3] {
    ui.horizontal_wrapped(|ui| {
        [
            bevel_button(ui, "Single Step", ACTION_GREEN, false, ready),
            bevel_button(
                ui,
                "Single Step 0.5s",
                ACTION_GREEN,
                false,
                ready && drag_mode == crate::freecell_route::DragMode::Absolute,
            ),
            bevel_button(ui, "SOLVE remaining route", ACTION_GREEN, false, ready),
        ]
    })
    .inner
}


/// Describe proved progress without suggesting resumption after STOP.
fn success_message(progress: &Progress, stopped: bool) -> String {
    if stopped {
        "Playback stopped; CALCULATE again before input".into()
    } else {
        match progress.completion {
            Some(crate::freecell_route::Completion::FullBoard) => {
                "Complete route observed: all 52 cards in foundations".into()
            }
            Some(crate::freecell_route::Completion::PartialRouteExhausted) => format!(
                "Maximum partial route observed: {}/52 cards in foundations; route retired",
                progress.foundation_cards
            ),
            Some(crate::freecell_route::Completion::ChallengeComplete) => {
                "Game-reported Challenge Complete; remaining route retired".into()
            }
            Some(crate::freecell_route::Completion::MovementProbe) => {
                let mut message = "REL movement probe acquired; route cursor unchanged, no card effect claimed; route retired".to_owned();
                if let Some(evidence) = &progress.probe_evidence {
                    message.push_str(&format!("; diagnostic frame {}", evidence.capture_serial));
                    if let Some(path) = &evidence.directory {
                        message.push_str(&format!("; evidence {}", path.display()));
                    }
                    if let Some(error) = &evidence.persistence_error {
                        message.push_str(&format!("; evidence persistence: {error}"));
                    }
                }
                message
            }
            Some(crate::freecell_route::Completion::StepVerified) => format!(
                "Step verified at transfer {}; playback stopped, ready for the next planned transfer",
                progress.cursor
            ),
            None => "Playback progress has no final completion receipt".into(),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;


    /// Render actual selector widgets in a headless viewport with supplied pointer events.
    fn drag_selector_frame(
        context: &egui::Context,
        mode: &mut crate::freecell_route::DragMode,
        enabled: bool,
        events: Vec<egui::Event>,
        width: f32,
    ) -> [egui::Rect; 2] {
        let mut bounds = [egui::Rect::NOTHING; 2];
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
                    bounds =
                        diagnostic_drag_selector(ui, mode, enabled).map(|response| response.rect);
                },
            )
            .drop_without_applying_deltas();
        bounds
    }


    /// Click a real selector widget without a worker, transport or application startup.
    fn click_drag_selector(
        context: &egui::Context,
        mode: &mut crate::freecell_route::DragMode,
        enabled: bool,
        position: egui::Pos2,
        width: f32,
    ) {
        let pointer = |pressed| egui::Event::PointerButton {
            pos: position,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        drag_selector_frame(
            context,
            mode,
            enabled,
            vec![egui::Event::PointerMoved(position), pointer(true)],
            width,
        );
        drag_selector_frame(context, mode, enabled, vec![pointer(false)], width);
    }


    /// Both modes are visible and selectable while idle, and a running choice stays frozen.
    #[test]
    fn freecell_317_diagnostic_drag_selector_defaults_absolute_and_locks_during_run() {
        use crate::freecell_route::DragMode;

        for width in [320.0, 640.0, 1920.0] {
            let context = egui::Context::default();
            let mut playback = Playback::default();
            assert_eq!(playback.drag_mode, DragMode::Absolute);
            let bounds =
                drag_selector_frame(&context, &mut playback.drag_mode, true, vec![], width);
            for rect in bounds {
                assert!(rect.is_positive());
                assert!(rect.left() >= 0.0 && rect.right() <= width);
            }
            click_drag_selector(
                &context,
                &mut playback.drag_mode,
                true,
                bounds[1].center(),
                width,
            );
            assert_eq!(playback.drag_mode, DragMode::Relative);
            click_drag_selector(
                &context,
                &mut playback.drag_mode,
                false,
                bounds[0].center(),
                width,
            );
            assert_eq!(playback.drag_mode, DragMode::Relative);
            // Publish the enabled hit-test regions before the next user press.
            let bounds =
                drag_selector_frame(&context, &mut playback.drag_mode, true, vec![], width);
            click_drag_selector(
                &context,
                &mut playback.drag_mode,
                true,
                bounds[0].center(),
                width,
            );
            assert_eq!(playback.drag_mode, DragMode::Absolute);
        }
    }


    /// Render actual playback widgets and report their hit regions and click state.
    fn playback_button_frame(
        context: &egui::Context,
        ready: bool,
        drag_mode: crate::freecell_route::DragMode,
        events: Vec<egui::Event>,
        width: f32,
    ) -> [(egui::Rect, bool, bool); 3] {
        let mut buttons = [(egui::Rect::NOTHING, false, false); 3];
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
                    buttons = playback_buttons(ui, ready, drag_mode)
                        .map(|response| (response.rect, response.enabled(), response.clicked()));
                },
            )
            .drop_without_applying_deltas();
        buttons
    }


    /// Click an actual playback widget without starting the application or a worker.
    fn click_playback_button(
        context: &egui::Context,
        ready: bool,
        drag_mode: crate::freecell_route::DragMode,
        position: egui::Pos2,
        width: f32,
    ) -> [bool; 3] {
        let pointer = |pressed| egui::Event::PointerButton {
            pos: position,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        playback_button_frame(
            context,
            ready,
            drag_mode,
            vec![egui::Event::PointerMoved(position), pointer(true)],
            width,
        );
        playback_button_frame(context, ready, drag_mode, vec![pointer(false)], width)
            .map(|(_, _, clicked)| clicked)
    }


    /// Both ABS buttons work while idle; running requests and REL disable the fast choice.
    #[test]
    fn freecell_3112_actual_single_step_buttons_lock_during_run_and_half_second_is_abs_only() {
        use crate::freecell_route::DragMode;

        for width in [320.0, 640.0, 1920.0] {
            let context = egui::Context::default();
            let buttons = playback_button_frame(&context, true, DragMode::Absolute, vec![], width);
            assert_eq!(buttons.map(|(_, enabled, _)| enabled), [true; 3]);
            for (index, (bounds, _, _)) in buttons.into_iter().enumerate().take(2) {
                assert!(bounds.is_positive());
                assert!(bounds.left() >= 0.0 && bounds.right() <= width);
                let clicked = click_playback_button(
                    &context,
                    true,
                    DragMode::Absolute,
                    bounds.center(),
                    width,
                );
                let mut expected = [false; 3];
                expected[index] = true;
                assert_eq!(clicked, expected);
            }
            let running = playback_button_frame(&context, false, DragMode::Absolute, vec![], width);
            assert_eq!(running.map(|(_, enabled, _)| enabled), [false; 3]);
            for (bounds, _, _) in running.into_iter().take(2) {
                assert_eq!(
                    click_playback_button(
                        &context,
                        false,
                        DragMode::Absolute,
                        bounds.center(),
                        width
                    ),
                    [false; 3],
                );
            }
            let relative = playback_button_frame(&context, true, DragMode::Relative, vec![], width);
            assert_eq!(relative.map(|(_, enabled, _)| enabled), [true, false, true]);
            assert_eq!(
                click_playback_button(
                    &context,
                    true,
                    DragMode::Relative,
                    relative[1].0.center(),
                    width
                ),
                [false; 3],
            );
        }
    }


    /// Construct immutable correlation data without a worker or transport.
    fn plan() -> Identity {
        Identity {
            context: ControllerContext::new(
                PathBuf::from("/nonexistent/qmp.sock"),
                GameMode::FreeCell,
                SolvingStrategy::ShortestPath,
                4,
            ),
            request_id: 7,
            source_generation: 2,
        }
    }


    /// Failure leaves both actual request widgets enabled; revoked consent never reappears.
    #[test]
    fn freecell_3110_failure_keeps_idle_request_buttons_and_original_error() {
        let id = plan();
        let mut playback = Playback {
            plan: Some(id.clone()),
            pending: Some((id.clone(), 11)),
            ..Default::default()
        };
        let failure = crate::worker::freecell_route::Failure {
            error: "No stable physical child after six observations".into(),
            details: serde_json::json!({"attempted":1,"acknowledged":1,"cursor":3}),
            directory: Some(PathBuf::from("/retained/original-failure")),
            persistence_error: None,
        };
        let original = failure.details.clone();
        assert!(!playback.ready(&id.context, true, |_| true));
        assert!(playback.finish(&id, 11));
        assert!(playback.failed(&id, &failure.error, true, false));
        assert!(playback.message.starts_with(&failure.error));
        assert!(playback.message.contains("manual UNDO ALL"));
        assert_eq!(failure.details, original);
        let context = egui::Context::default();
        let mut enabled = [false; 3];
        context
            .run_ui(egui::RawInput::default(), |ui| {
                let ready = playback.ready(&id.context, true, |_| true);
                enabled = playback_buttons(ui, ready, playback.drag_mode)
                    .map(|response| response.enabled());
            })
            .drop_without_applying_deltas();
        assert_eq!(enabled, [true, true, true]);
        assert!(!playback.ready(&id.context, false, |_| true));
        assert!(!playback.ready(&id.context, true, |_| false));
        let mut different = id.context.clone();
        different.generation += 1;
        assert!(!playback.ready(&different, true, |_| true));
        playback.revoke();
        assert!(!playback.failed(&id, &failure.error, true, false));
        assert!(playback.plan.is_none());
        playback.plan = Some(id.clone());
        assert!(!playback.failed(&id, &failure.error, true, true));
        assert!(playback.plan.is_none());
    }


    /// Supply actual-shaped advisory counts for an invocation.
    fn progress(plan: &Identity, run_id: u64, cursor: usize) -> Progress {
        Progress {
            plan: plan.clone(),
            run_id,
            cursor,
            total: 8,
            attempted: 1,
            acknowledged: 1,
            foundation_cards: 3,
            completion: Some(crate::freecell_route::Completion::StepVerified),
            probe_evidence: None,
        }
    }


    /// Wrong request/source/socket/context/run and duplicate finals retain ownership.
    #[test]
    fn freecell_311_playback_exact_owner_and_late_progress() {
        let id = plan();
        let mut state = Playback {
            plan: Some(id.clone()),
            pending: Some((id.clone(), 11)),
            ..Default::default()
        };
        let mut context = id.clone();
        context.context.generation += 1;
        let mut socket = id.clone();
        socket.context.socket_path = PathBuf::from("/other/qmp.sock");
        for stale in [
            Identity {
                request_id: 8,
                ..id.clone()
            },
            Identity {
                source_generation: 3,
                ..id.clone()
            },
            context,
            socket,
        ] {
            state.progress(progress(&stale, 11, 7));
            assert!(state.progress.is_none());
            assert!(!state.finish(&stale, 11));
            assert_eq!(state.pending, Some((id.clone(), 11)));
        }
        state.progress(progress(&id, 12, 7));
        assert!(state.progress.is_none());
        assert!(!state.finish(&id, 12));
        state.progress(progress(&id, 11, 2));
        assert!(state.finish(&id, 11));
        assert!(!state.finish(&id, 11));
        state.progress(progress(&id, 11, 7));
        assert_eq!(state.progress.as_ref().unwrap().cursor, 2);
        assert_eq!(state.retained, Some(id.context.clone()));
        state.pending = Some((id.clone(), 12));
        state.progress(progress(&id, 11, 7));
        assert!(!state.finish(&id, 11));
        state.progress(progress(&id, 12, 3));
        assert_eq!(state.progress.as_ref().unwrap().cursor, 3);
    }


    /// Revocation retains dedicated ownership through every generic worker state.
    #[test]
    fn freecell_311_playback_revoke_and_late_stop_do_not_resume() {
        let id = plan();
        let mut state = Playback {
            plan: Some(id.clone()),
            pending: Some((id.clone(), 11)),
            ..Default::default()
        };
        state.revoke();
        assert!(state.plan.is_none());
        assert_eq!(state.pending, Some((id.clone(), 11)));
        for worker in [
            WorkerState::Detached,
            WorkerState::Ready,
            WorkerState::Uncertain,
            WorkerState::Error,
        ] {
            let owner = active_run_after_worker_state(Some(ActiveRun::FreeCellRoute), worker);
            assert_eq!(owner, Some(ActiveRun::FreeCellRoute));
            assert!(!controls_are_mutable(owner, worker, false));
        }
        assert!(state.finish(&id, 11));
        assert!(state.plan.is_none());
        let mut accepted = progress(&id, 11, 2);
        assert!(success_message(&accepted, false).contains("ready"));
        assert_eq!(
            success_message(&accepted, true),
            "Playback stopped; CALCULATE again before input"
        );
        accepted.cursor = accepted.total;
        accepted.completion = Some(crate::freecell_route::Completion::FullBoard);
        assert_eq!(
            success_message(&accepted, true),
            "Playback stopped; CALCULATE again before input"
        );
        assert!(success_message(&accepted, false).contains("all 52"));
    }


    /// Exhaustion and game-reported completion never masquerade as a full-board proof.
    #[test]
    fn freecell_315_partial_and_terminal_messages_do_not_claim_full_board() {
        let mut result = progress(&plan(), 11, 8);
        result.completion = Some(crate::freecell_route::Completion::PartialRouteExhausted);
        assert!(success_message(&result, false).contains("3/52"));
        assert!(!success_message(&result, false).contains("all 52"));
        result.completion = Some(crate::freecell_route::Completion::ChallengeComplete);
        let message = success_message(&result, false);
        assert!(message.contains("remaining route retired"));
        assert!(!message.contains("all 52") && !message.contains("ready"));
    }

    /// A completed experiment never advertises a card effect, victory or resumable route.
    #[test]
    fn freecell_318_probe_status_is_explicit_and_persistence_is_secondary() {
        let mut result = progress(&plan(), 12, 0);
        result.completion = Some(crate::freecell_route::Completion::MovementProbe);
        result.probe_evidence = Some(crate::worker::freecell_route::ProbeEvidence {
            capture_serial: 5,
            directory: Some(std::path::PathBuf::from("/private/probe")),
            persistence_error: Some("test disk omission".into()),
        });
        let message = success_message(&result, false);
        assert!(message.contains("cursor unchanged") && message.contains("route retired"));
        assert!(message.contains("/private/probe") && message.contains("test disk omission"));
        assert!(!message.contains("all 52") && !message.contains("ready for"));
        assert_eq!(
            success_message(&result, true),
            "Playback stopped; CALCULATE again before input"
        );
    }
}
