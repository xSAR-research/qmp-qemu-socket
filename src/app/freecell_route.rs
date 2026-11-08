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
    /// Policy matching the game setting, frozen by the first worker invocation.
    mode: crate::freecell_route::AutoMode,
    /// Preserve already acquired pixels through queued STOP disconnection.
    pub(super) retained: Option<ControllerContext>,
}


impl Playback {
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


    /// Dispatch one deliberate move plus observed autos, or the remaining complete route.
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
        match self.worker.execute_freecell_route(
            plan.clone(),
            single_step,
            self.freecell_playback.mode,
            settle,
        ) {
            Ok(run_id) => {
                self.freecell_playback.pending = Some((plan, run_id));
                self.freecell_playback.progress = None;
                self.freecell_playback.retained = None;
                self.freecell_playback.message =
                    "Checking the live physical board before route input".into();
                self.active_run = Some(ActiveRun::FreeCellRoute);
                self.worker_state = WorkerState::Validating;
            }
            Err(error) => {
                self.revoke_freecell_playback();
                self.failure_window.general(
                    "Free Cell playback dispatch",
                    &error,
                    serde_json::json!({"context":format!("{plan:?}"),"phase":"dispatch"}),
                );
                self.freecell_playback.message = error;
            }
        }
    }


    /// Keep every input control explicit; automatic records are observed rather than clicked.
    pub(super) fn freecell_route_controls(&mut self, ui: &mut egui::Ui) {
        let idle = controls_are_mutable(self.active_run, self.worker_state, self.stop_requested)
            && !self.exiting;
        let ready = idle
            && self.freecell_playback.plan.as_ref().is_some_and(|p| {
                p.context == self.current_preview_context()
                    && self.worker.freecell_route_available(p)
            });
        ui.separator();
        ui.label("Live route playback · keep Solver off and match the game autoplay setting.");
        ui.add_enabled_ui(idle, |ui| {
            egui::ComboBox::from_id_salt("freecell-autoplay-policy")
                .selected_text(match self.freecell_playback.mode {
                    crate::freecell_route::AutoMode::ObserveGame => {
                        "Game autoplay ON: observe automatic transfers"
                    }
                    crate::freecell_route::AutoMode::ExecuteRecorded => {
                        "Game autoplay OFF: execute every transfer"
                    }
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.freecell_playback.mode,
                        crate::freecell_route::AutoMode::ObserveGame,
                        "Game autoplay ON: observe automatic transfers",
                    );
                    ui.selectable_value(
                        &mut self.freecell_playback.mode,
                        crate::freecell_route::AutoMode::ExecuteRecorded,
                        "Game autoplay OFF: execute every transfer",
                    );
                });
        });
        ui.label("Single Step includes leading and trailing automatic transfers. After playback begins, changing this policy requires CALCULATE again.");
        ui.label("The original deal must match. Each step verifies the resulting physical board; unsupported or obscured positions stop playback.");
        ui.horizontal(|ui| {
            if bevel_button(ui, "Single Step", ACTION_GREEN, false, ready).clicked() {
                self.dispatch_freecell_route(true);
            }
            if bevel_button(ui, "SOLVE remaining route", ACTION_GREEN, false, ready).clicked() {
                self.dispatch_freecell_route(false);
            }
        });
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
                && let Err(error) = self.worker.request_pause(snapshot.epoch, !paused)
            {
                self.freecell_playback.message = error;
            }
            if ui.button("STOP playback").clicked()
                && let Err(error) = self.request_stop()
            {
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
            return;
        }
        self.active_run = None;
        match result {
            Ok(progress) => {
                self.freecell_playback.message = success_message(&progress, self.stop_requested);
                self.freecell_playback.progress = Some(progress);
                self.worker_state = WorkerState::Ready;
            }
            Err(failure) => {
                self.revoke_freecell_playback();
                if termination != RunTermination::OperatorStopped {
                    self.failure_window.freecell_route(&failure);
                }
                self.freecell_playback.message = failure.error;
                self.worker_state = if termination == RunTermination::OperatorStopped {
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


/// Describe proved progress without suggesting resumption after STOP.
fn success_message(progress: &Progress, stopped: bool) -> String {
    if stopped {
        "Playback stopped; CALCULATE again before input".into()
    } else if progress.cursor == progress.total {
        "Complete route observed: all 52 cards in foundations".into()
    } else {
        format!(
            "Step verified at transfer {}; ready for the next deliberate move",
            progress.cursor
        )
    }
}


#[cfg(test)]
mod tests {
    use super::*;


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


    /// Supply actual-shaped advisory counts for an invocation.
    fn progress(plan: &Identity, run_id: u64, cursor: usize) -> Progress {
        Progress {
            plan: plan.clone(),
            run_id,
            cursor,
            total: 8,
            attempted: 1,
            acknowledged: 1,
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
        assert_eq!(
            success_message(&accepted, true),
            "Playback stopped; CALCULATE again before input"
        );
        assert!(success_message(&accepted, false).contains("all 52"));
    }
}
