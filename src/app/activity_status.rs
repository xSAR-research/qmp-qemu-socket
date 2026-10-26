//! Read-only activity presentation from current typed worker ownership.
//!
//! A retained diagnostic or search report never becomes an active operation.
//! This module neither sends worker requests nor grants execution authority.

mod badge;

use super::{ActiveRun, QmpQemuSocketApp, WorkerState};
use crate::{pyramid_diagnostics::PyramidPhase, worker::pyramid_pause};


/// Install the bundled bold face without changing the ordinary UI font families.
pub(super) fn install_fonts(context: &eframe::egui::Context) {
    badge::install_fonts(context);
}


/// Render top-left screen selections beside the fixed activity allocation.
///
/// Snapshot typed presentation evidence before lending the application to the
/// existing selection controls; the returned value belongs to those controls.
pub(super) fn header<R>(
    app: &mut QmpQemuSocketApp,
    ui: &mut eframe::egui::Ui,
    add_contents: impl FnOnce(&mut QmpQemuSocketApp, &mut eframe::egui::Ui) -> R,
) -> R {
    let context = app.current_preview_context();
    let evidence = Evidence {
        worker: app.worker_state,
        owner: app.active_run,
        phase: app.pyramid_workflow.activity_phase(&context),
        failure: app.pyramid_workflow.details_visible(&context),
        pause: app.worker.pause_state(),
        stopping: app.stop_requested,
        exiting: app.exiting,
        choosing: app.strategy_choice_pending,
    };
    badge::header(ui, evidence.status(), |ui| add_contents(app, ui)).0
}


/// Snapshot of presentation facts; none is an input permit or new worker state.
struct Evidence {
    /// Current connection/action state supplied by worker events.
    worker: WorkerState,
    /// UI ownership retained until the matching completion event.
    owner: Option<ActiveRun>,
    /// Latest context-matched manual or automatic operation phase.
    phase: Option<PyramidPhase>,
    /// Current context's failure, including a refusal before worker dispatch.
    failure: bool,
    /// Actual checkpoint acknowledgement, independent of the event queue.
    pause: Option<pyramid_pause::Snapshot>,
    /// Existing STOP latch; presentation never clears it.
    stopping: bool,
    /// Native shutdown is waiting for worker detachment.
    exiting: bool,
    /// Startup still awaits the operator's strategy choice.
    choosing: bool,
}


impl Evidence {
    /// Resolve activity without parsing log/status text or reviving retired work.
    fn status(self) -> badge::Status {
        use badge::Status;
        if self.exiting || self.choosing {
            return Status::Waiting;
        }
        if matches!(self.worker, WorkerState::Error | WorkerState::Uncertain) {
            return Status::Error;
        }
        if self.failure {
            return Status::Error;
        }
        if self.stopping || self.pause.is_some_and(|pause| pause.acknowledged) {
            return Status::Waiting;
        }
        if self.owner.is_none() && !self.worker.is_busy() {
            return Status::Waiting;
        }
        if let Some(pause) = self.pause {
            return match pause.phase {
                pyramid_pause::Phase::Recon => Status::Recon,
                pyramid_pause::Phase::Search => Status::Route,
                pyramid_pause::Phase::Gameplay => Status::Solve,
            };
        }
        match self.owner {
            Some(ActiveRun::Reconnaissance) => Status::Recon,
            Some(ActiveRun::StepOnce | ActiveRun::MultiStep) => Status::Solve,
            Some(ActiveRun::ComputedRoute | ActiveRun::PyramidSession) => match self.phase {
                Some(PyramidPhase::Reconnaissance) => Status::Recon,
                Some(PyramidPhase::Planning) => Status::Route,
                Some(PyramidPhase::Execution) => Status::Solve,
                None if self.owner == Some(ActiveRun::ComputedRoute) => Status::Route,
                None => Status::Recon,
            },
            None => Status::Recon,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    /// A retained route phase cannot pulse after ownership finishes; real pause
    /// acknowledgement wins over the displayed phase, while a request alone does not.
    #[test]
    fn activity_status_tracks_owner_checkpoint_and_error_without_text() {
        let make = || Evidence {
            worker: WorkerState::Validating,
            owner: Some(ActiveRun::PyramidSession),
            phase: Some(PyramidPhase::Reconnaissance),
            failure: false,
            pause: None,
            stopping: false,
            exiting: false,
            choosing: false,
        };
        let mut state = make();
        assert_eq!(state.status(), badge::Status::Recon);
        state = make();
        state.phase = Some(PyramidPhase::Planning);
        assert_eq!(state.status(), badge::Status::Route);
        state = make();
        state.phase = Some(PyramidPhase::Execution);
        assert_eq!(state.status(), badge::Status::Solve);
        state = make();
        state.phase = Some(PyramidPhase::Planning);
        state.owner = None;
        state.worker = WorkerState::Ready;
        assert_eq!(state.status(), badge::Status::Waiting);
        state = make();
        state.owner = None;
        state.worker = WorkerState::Ready;
        state.failure = true;
        assert_eq!(state.status(), badge::Status::Error);
        state = make();
        state.pause = Some(pyramid_pause::Snapshot {
            epoch: 17,
            phase: pyramid_pause::Phase::Search,
            requested: true,
            acknowledged: false,
        });
        assert_eq!(state.status(), badge::Status::Route);
        state = make();
        state.pause = Some(pyramid_pause::Snapshot {
            epoch: 17,
            phase: pyramid_pause::Phase::Search,
            requested: true,
            acknowledged: true,
        });
        assert_eq!(state.status(), badge::Status::Waiting);
        state = make();
        state.worker = WorkerState::Uncertain;
        state.stopping = true;
        assert_eq!(state.status(), badge::Status::Error);
        state = make();
        state.worker = WorkerState::Error;
        state.stopping = true;
        assert_eq!(state.status(), badge::Status::Error);
        state = make();
        state.worker = WorkerState::Uncertain;
        assert_eq!(state.status(), badge::Status::Error);
        state = make();
        state.worker = WorkerState::Ready;
        state.owner = None;
        state.failure = true;
        state.stopping = true;
        assert_eq!(state.status(), badge::Status::Error);
    }
}
