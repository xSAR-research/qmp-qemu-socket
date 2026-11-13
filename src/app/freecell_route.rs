//! Explicit live Free Cell playback controls, separate from offline calculation.
use super::*;
use crate::{
    pyramid_workflow_model::RunTermination,
    ui_style::PressableButton,
    worker::{freecell_calculation::Identity, freecell_route::Progress},
};


/// UI correlation only; the worker owns the candidate, cursor and live proof.
#[derive(Default)]
pub(super) struct Playback {
    /// Displayed independently replayed calculation, never a live-board receipt.
    pub(super) plan: Option<Identity>,
    /// Exact outstanding invocation until its dedicated completion.
    pending: Option<(Identity, u64)>,
    /// Active MULTI STEP only: false until its resumable stop request is accepted.
    multi_step_stop: Option<bool>,
    /// Coalesced semantic counts, separate from delivery acknowledgement.
    progress: Option<Progress>,
    /// Current result, retained even when the executable candidate is retired.
    message: String,
    /// Application transfer policy copied into the next explicit worker invocation.
    pub(super) mode: crate::freecell_route::AutoMode,
    /// Preserve already acquired pixels through queued STOP disconnection.
    pub(super) retained: Option<ControllerContext>,
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
                "{error}; after manual UNDO ALL, request SINGLE STEP or MULTI STEP again. The cached original route restarts from transfer 1 after fresh recognition of the same 52-card board."
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
        self.multi_step_stop = None;
        self.retained = Some(plan.context.clone());
        true
    }
}


/// Concise display label for [`crate::freecell_route::AutoMode`].
pub(super) const fn auto_mode_label(mode: crate::freecell_route::AutoMode) -> &'static str {
    match mode {
        crate::freecell_route::AutoMode::ObserveGame => "Observe recorded AUTO follow-ups",
        crate::freecell_route::AutoMode::ExecuteRecorded => "Execute each recorded transfer",
    }
}


/// Shared selector widget for Free Cell transfer follow-up handling.
pub(super) fn auto_mode_combo(
    ui: &mut egui::Ui,
    id_salt: &str,
    mode: &mut crate::freecell_route::AutoMode,
) {
    egui::ComboBox::from_id_salt(id_salt)
        .width(220.0)
        .selected_text(auto_mode_label(*mode))
        .show_ui(ui, |ui| {
            ui.selectable_value(
                mode,
                crate::freecell_route::AutoMode::ObserveGame,
                auto_mode_label(crate::freecell_route::AutoMode::ObserveGame),
            );
            ui.selectable_value(
                mode,
                crate::freecell_route::AutoMode::ExecuteRecorded,
                auto_mode_label(crate::freecell_route::AutoMode::ExecuteRecorded),
            );
        });
}


impl QmpQemuSocketApp {
    /// Revoke executable state without releasing a running owner or erasing its report.
    pub(super) fn revoke_freecell_playback(&mut self) {
        self.freecell_playback.revoke();
        self.worker.revoke_freecell_route();
    }


    /// Dispatch one explicit transfer plus proved recorded follow-ups, or the remaining route.
    fn dispatch_freecell_route(&mut self, single_step: bool) {
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
        self.trace_freecell_ui(
            "playback-dispatch",
            serde_json::json!({
            "plan":format!("{plan:?}"),"request_id":plan.request_id,
            "source_generation":plan.source_generation,"single_step":single_step,
            "autoplay_mode":format!("{:?}",self.freecell_playback.mode),
            "diagnostic_drag_mode":"Absolute",
            "gesture_phase_wait_ms":crate::parameters::FREECELL_ROUTE_ABS_GESTURE_INTERVAL.as_millis(),
            "settle_ms":self.freecell_settle_ms}),
        );
        let result = self.worker.execute_freecell_route(
            plan.clone(),
            single_step,
            self.freecell_playback.mode,
            settle,
        );
        match result {
            Ok(run_id) => {
                let preview_released = self
                    .freecell_recon_state
                    .release_for_playback(&plan.context);
                self.trace_freecell_ui(
                    "playback-dispatched",
                    serde_json::json!({
                    "plan":format!("{plan:?}"),"run_id":run_id,"single_step":single_step,
                    "diagnostic_drag_mode":"Absolute",
                            "completed_recon_preview_released":preview_released}),
                );
                self.freecell_playback.pending = Some((plan, run_id));
                self.freecell_playback.multi_step_stop = (!single_step).then_some(false);
                self.freecell_playback.progress = None;
                self.freecell_playback.retained = None;
                self.freecell_playback.message =
                    "Checking the live physical board before route input".into();
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


    /// Request a resumable stop for the exact MULTI STEP owner without latching global STOP.
    fn request_freecell_stop_after_step(&mut self) {
        if self.freecell_playback.multi_step_stop != Some(false) {
            return;
        }
        let Some((plan, run_id)) = self.freecell_playback.pending.clone() else {
            return;
        };
        self.trace_freecell_ui(
            "playback-stop-after-step-intent",
            serde_json::json!({"plan":format!("{plan:?}"),"run_id":run_id}),
        );
        match self.worker.request_freecell_stop_after_step(&plan, run_id) {
            Ok(()) => {
                self.freecell_playback.multi_step_stop = Some(true);
                self.freecell_playback.message =
                    "Stop requested; finishing the current verified step before playback stops"
                        .into();
            }
            Err(error) => {
                self.trace_freecell_ui(
                    "playback-stop-after-step-failed",
                    serde_json::json!({"plan":format!("{plan:?}"),"run_id":run_id,"error":error}),
                );
                self.failure_window.general(
                    "Free Cell stop after step",
                    &error,
                    serde_json::json!({"plan":format!("{plan:?}"),"run_id":run_id,
                        "phase":"stop after step request"}),
                );
                self.freecell_playback.message = error;
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
            auto_mode_combo(
                ui,
                "freecell-autoplay-policy",
                &mut self.freecell_playback.mode,
            );
        });
        ui.label(match self.freecell_playback.mode {
            crate::freecell_route::AutoMode::ObserveGame => {
                "Single Step confirms the current board, credits already observed recorded AUTOs, then sends ONE explicit ABS drag even when the next record is AUTO. It waits for two fresh exact matches of the combined transfer and recorded AUTO follow-ups, credits them and stops."
            }
            crate::freecell_route::AutoMode::ExecuteRecorded => {
                "Single Step confirms the current board, sends ONE explicit recorded ABS transfer, even when marked AUTO, then requires two fresh exact endpoint matches and stops."
            }
        });
        ui.label("Expected AUTOs still require observed board proof. Missing or different effects stop within six observations. After failure or changing transfer policy, manually UNDO ALL before the next explicit request: the cached original route restarts only after fresh recognition of the same 52-card board.");
        ui.label("ABS sends six separately acknowledged events: source, LEFT down, 45 native pixels above the source, the midpoint of the original source and destination (each native axis rounded down), exact destination, LEFT up. Every destination preserves the source pickup inset: 15 native pixels right and 20 down from its proved corner or empty-slot origin. SINGLE STEP and MULTI STEP both use five 100-millisecond intervals (at least 0.5 seconds per transfer). Fresh acquisition and effect proof still apply.");
        ui.label("The original deal must match. Each step verifies the resulting physical board; unsupported or obscured positions stop playback.");
        let active = self.active_run == Some(ActiveRun::FreeCellRoute);
        let mutable = !self.stop_requested && !self.exiting;
        let multi_step_stop = active
            .then_some(self.freecell_playback.multi_step_stop)
            .flatten();
        let pause = active.then(|| self.worker.pause_state()).flatten();
        let paused = pause
            .as_ref()
            .map(|state| state.requested || state.acknowledged);
        let clicked = playback_buttons(ui, ready, multi_step_stop, active, paused, mutable)
            .map(|response| response.is_some_and(|response| response.clicked()));
        let action = playback_action(clicked, active, multi_step_stop.is_some());
        match action {
            Some(PlaybackAction::SingleStep) => self.dispatch_freecell_route(true),
            Some(PlaybackAction::MultiStep) => self.dispatch_freecell_route(false),
            Some(PlaybackAction::StopAfterStep) => self.request_freecell_stop_after_step(),
            _ => {}
        }
        if multi_step_stop.is_some() {
            ui.label("Stop after step finishes the current transfer and required recorded AUTO proof, then keeps the verified route ready to resume. While paused, choose Continue to finish the step. Cancel revokes input authority at the guarded STOP checks.");
        }
        if action == Some(PlaybackAction::Pause)
            && let Some(snapshot) = pause
        {
            let requested = !(snapshot.requested || snapshot.acknowledged);
            self.trace_freecell_ui(
                "playback-pause-intent",
                serde_json::json!({
                    "pending":format!("{:?}",self.freecell_playback.pending),
                    "pause_epoch":snapshot.epoch,"pause_requested":requested}),
            );
            if let Err(error) = self.worker.request_pause(snapshot.epoch, requested) {
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
        if action == Some(PlaybackAction::Cancel) {
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
                    "Free Cell playback cancellation",
                    &error,
                    serde_json::json!({"pending":format!("{:?}",self.freecell_playback.pending),
                        "phase":"STOP request"}),
                );
                self.freecell_playback.message = error;
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


/// A UI intent routed to the existing owner-checked dispatch or interruption path.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PlaybackAction {
    /// Start one independently proved explicit transfer.
    SingleStep,
    /// Start the remaining route using ABS transfers.
    MultiStep,
    /// Finish the current MULTI transfer and proof before retaining a resumable cursor.
    StopAfterStep,
    /// Toggle the current pause epoch without changing route ownership.
    Pause,
    /// Revoke live input authority through the guarded STOP path.
    Cancel,
}


/// Classify real widget clicks while preserving graceful MULTI Stop and immediate SINGLE Stop.
fn playback_action(clicked: [bool; 4], active: bool, multi_step: bool) -> Option<PlaybackAction> {
    match clicked {
        [true, _, _, _] => Some(if active {
            PlaybackAction::Cancel
        } else {
            PlaybackAction::SingleStep
        }),
        [_, true, _, _] => Some(if active && multi_step {
            PlaybackAction::StopAfterStep
        } else {
            PlaybackAction::MultiStep
        }),
        [_, _, true, _] => Some(PlaybackAction::Pause),
        [_, _, _, true] => Some(PlaybackAction::Cancel),
        _ => None,
    }
}


/// Render graceful MULTI Stop after step and immediate SINGLE Stop beside Pause and Cancel.
/// Preallocate the measured group, allowing its controls to wrap in narrow views.
fn playback_buttons(
    ui: &mut egui::Ui,
    ready: bool,
    multi_step_stop: Option<bool>,
    active: bool,
    paused: Option<bool>,
    mutable: bool,
) -> [Option<egui::Response>; 4] {
    ui.horizontal_wrapped(|ui| {
        let active_single = active && multi_step_stop.is_none();
        let single =
            (!active_single).then(|| ui.add_enabled(ready, PressableButton::new("SINGLE STEP")));
        let action_label = if active_single {
            "Stop"
        } else if active {
            "Stop after step"
        } else {
            "MULTI STEP"
        };
        let pause_label = if paused == Some(true) {
            "Continue"
        } else {
            "Pause"
        };
        let mut group_size = PressableButton::new(action_label).desired_size(ui);
        if active {
            for label in [pause_label, "Cancel"] {
                let size = PressableButton::new(label).desired_size(ui);
                group_size.x += ui.spacing().item_spacing.x + size.x;
                group_size.y = group_size.y.max(size.y);
            }
        }
        let action_enabled = mutable
            && if active_single {
                true
            } else {
                multi_step_stop.map_or(ready, |requested| !requested)
            };
        group_size.x = group_size.x.min(ui.max_rect().width());
        let [action, pause, cancel] = ui
            .allocate_ui_with_layout(
                group_size,
                egui::Layout::left_to_right(egui::Align::Center).with_main_wrap(true),
                |ui| {
                    ui.set_row_height(group_size.y);
                    let action = ui
                        .add_enabled(action_enabled, PressableButton::new(action_label))
                        .on_hover_text(if active_single {
                            "Revoke input authority at the guarded STOP checks"
                        } else if active {
                            "Finish the current transfer and required recorded AUTO proof, then retain the verified route for resumption. If paused, choose Continue to finish the step."
                        } else {
                            "Play the remaining route with verified ABS transfers"
                        });
                    let pause = active.then(|| {
                        ui.add_enabled(
                            mutable && paused.is_some(),
                            PressableButton::new(pause_label),
                        )
                    });
                    let cancel = active.then(|| {
                        ui.add_enabled(mutable, PressableButton::new("Cancel"))
                            .on_hover_text("Revoke input authority at the guarded STOP checks")
                    });
                    [Some(action), pause, cancel]
                },
            )
            .inner;
        if active_single {
            let multi = ui.add_enabled(false, PressableButton::new("MULTI STEP"));
            [action, Some(multi), pause, cancel]
        } else {
            [single, action, pause, cancel]
        }
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


    /// Headless state supplied to the actual production playback widgets.
    #[derive(Clone, Copy)]
    struct ButtonState {
        /// Whether an independently replayed route is ready for input.
        ready: bool,
        /// Active MULTI's accepted graceful-stop request, if any.
        multi_step_stop: Option<bool>,
        /// Whether this exact playback invocation owns the UI.
        active: bool,
        /// Current requested or acknowledged pause state, when available.
        paused: Option<bool>,
        /// Whether STOP and exit still permit controls.
        mutable: bool,
    }


    /// Render real widgets and expose their bounds, availability and click state.
    fn playback_button_frame(
        context: &egui::Context,
        state: ButtonState,
        events: Vec<egui::Event>,
        width: f32,
    ) -> [(egui::Rect, bool, bool); 4] {
        crate::ui_style::apply_dark_theme(context);
        let mut buttons = [(egui::Rect::NOTHING, false, false); 4];
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
                    buttons = playback_buttons(
                        ui,
                        state.ready,
                        state.multi_step_stop,
                        state.active,
                        state.paused,
                        state.mutable,
                    )
                    .map(|response| {
                        response.map_or((egui::Rect::NOTHING, false, false), |response| {
                            (response.rect, response.enabled(), response.clicked())
                        })
                    });
                },
            )
            .drop_without_applying_deltas();
        buttons
    }


    /// Click one real widget without application startup, a worker or guest transport.
    fn click_playback_button(
        context: &egui::Context,
        state: ButtonState,
        position: egui::Pos2,
        width: f32,
    ) -> [bool; 4] {
        let pointer = |pressed| egui::Event::PointerButton {
            pos: position,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        playback_button_frame(
            context,
            state,
            vec![egui::Event::PointerMoved(position), pointer(true)],
            width,
        );
        playback_button_frame(context, state, vec![pointer(false)], width)
            .map(|(_, _, clicked)| clicked)
    }


    /// The two normal actions are the only idle widgets and lock during playback.
    #[test]
    fn freecell_3120_actual_playback_buttons_have_two_idle_actions_without_fast_option() {
        for width in [320.0, 640.0, 1920.0] {
            let context = egui::Context::default();
            let idle = ButtonState {
                ready: true,
                multi_step_stop: None,
                active: false,
                paused: None,
                mutable: true,
            };
            let buttons = playback_button_frame(&context, idle, vec![], width);
            assert_eq!(
                buttons.map(|(_, enabled, _)| enabled),
                [true, true, false, false]
            );
            assert!(!buttons[2].0.is_positive());
            assert!(!buttons[3].0.is_positive());
            for (index, (bounds, _, _)) in buttons.into_iter().enumerate().take(2) {
                assert!(bounds.is_positive());
                assert!(bounds.left() >= 0.0 && bounds.right() <= width);
                let clicked = click_playback_button(&context, idle, bounds.center(), width);
                let mut expected = [false; 4];
                expected[index] = true;
                assert_eq!(clicked, expected);
                assert_eq!(
                    playback_action(clicked, false, false),
                    Some(if index == 0 {
                        PlaybackAction::SingleStep
                    } else {
                        PlaybackAction::MultiStep
                    })
                );
            }
            let busy = ButtonState {
                ready: false,
                ..idle
            };
            let buttons = playback_button_frame(&context, busy, vec![], width);
            assert_eq!(buttons.map(|(_, enabled, _)| enabled), [false; 4]);
            for (bounds, _, _) in buttons.into_iter().take(2) {
                assert_eq!(
                    click_playback_button(&context, busy, bounds.center(), width),
                    [false; 4]
                );
            }
        }
    }


    /// Stop after step, Pause/Continue and Cancel wrap within bounds and retain their actions.
    #[test]
    fn freecell_321_active_multi_stop_after_step_pause_cancel_respect_stop_latch() {
        for width in [320.0, 640.0, 1920.0] {
            let context = egui::Context::default();
            let active = ButtonState {
                ready: false,
                multi_step_stop: Some(false),
                active: true,
                paused: Some(false),
                mutable: true,
            };
            let buttons = playback_button_frame(&context, active, vec![], width);
            assert_eq!(
                buttons.map(|(_, enabled, _)| enabled),
                [false, true, true, true]
            );
            for index in 1..4 {
                let bounds = buttons[index].0;
                assert!(bounds.is_positive());
                assert!(bounds.left() >= 0.0 && bounds.right() <= width);
                if width >= 640.0 {
                    assert_eq!(bounds.center().y, buttons[1].0.center().y);
                }
                if index > 1 {
                    let previous = buttons[index - 1].0;
                    assert!(previous.right() < bounds.left() || previous.bottom() < bounds.top());
                }
                let mut expected = [false; 4];
                expected[index] = true;
                let clicked = click_playback_button(&context, active, bounds.center(), width);
                assert_eq!(clicked, expected);
                assert_eq!(
                    playback_action(clicked, true, true),
                    Some(match index {
                        1 => PlaybackAction::StopAfterStep,
                        2 => PlaybackAction::Pause,
                        _ => PlaybackAction::Cancel,
                    })
                );
            }
            let single = ButtonState {
                multi_step_stop: None,
                ..active
            };
            let buttons = playback_button_frame(&context, single, vec![], width);
            assert_eq!(
                buttons.map(|(_, enabled, _)| enabled),
                [true, false, true, true]
            );
            for index in [0, 2, 3] {
                let bounds = buttons[index].0;
                assert!(bounds.left() >= 0.0 && bounds.right() <= width);
                assert_eq!(bounds.center().y, buttons[0].0.center().y);
                let clicked = click_playback_button(&context, single, bounds.center(), width);
                assert_eq!(
                    playback_action(clicked, true, false),
                    Some(if index == 2 {
                        PlaybackAction::Pause
                    } else {
                        PlaybackAction::Cancel
                    })
                );
            }
            for state in [
                ButtonState {
                    multi_step_stop: Some(true),
                    ..active
                },
                ButtonState {
                    multi_step_stop: Some(true),
                    paused: Some(true),
                    ..active
                },
            ] {
                let buttons = playback_button_frame(&context, state, vec![], width);
                assert_eq!(
                    buttons.map(|(_, enabled, _)| enabled),
                    [false, false, true, true]
                );
                for pair in buttons[1..].windows(2) {
                    assert!(pair[0].0.left() >= 0.0 && pair[1].0.right() <= width);
                    if width >= 640.0 {
                        assert_eq!(pair[0].0.center().y, pair[1].0.center().y);
                    } else {
                        assert!(
                            pair[0].0.right() < pair[1].0.left()
                                || pair[0].0.bottom() < pair[1].0.top()
                        );
                    }
                }
            }
            let stopping = ButtonState {
                mutable: false,
                ..active
            };
            let buttons = playback_button_frame(&context, stopping, vec![], width);
            assert_eq!(buttons.map(|(_, enabled, _)| enabled), [false; 4]);
            for (bounds, _, _) in buttons.into_iter().skip(1) {
                assert_eq!(
                    click_playback_button(&context, stopping, bounds.center(), width),
                    [false; 4]
                );
            }
            let queued = ButtonState {
                paused: None,
                ..active
            };
            let buttons = playback_button_frame(&context, queued, vec![], width);
            assert_eq!(
                buttons.map(|(_, enabled, _)| enabled),
                [false, true, false, true]
            );
        }
        let id = plan();
        let mut playback = Playback {
            plan: Some(id.clone()),
            pending: Some((id.clone(), 11)),
            multi_step_stop: Some(true),
            ..Default::default()
        };
        assert!(!playback.finish(&id, 12));
        assert_eq!(playback.multi_step_stop, Some(true));
        assert!(playback.finish(&id, 11));
        assert_eq!(playback.multi_step_stop, None);
        assert_eq!(playback.plan, Some(id.clone()));
        assert!(playback.ready(&id.context, true, |_| true));
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
        let mut enabled = [false; 4];
        context
            .run_ui(egui::RawInput::default(), |ui| {
                let ready = playback.ready(&id.context, true, |_| true);
                enabled = playback_buttons(ui, ready, None, false, None, true)
                    .map(|response| response.is_some_and(|response| response.enabled()));
            })
            .drop_without_applying_deltas();
        assert_eq!(enabled, [true, true, false, false]);
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
}
