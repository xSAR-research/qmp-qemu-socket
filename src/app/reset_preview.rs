//! Correlation and presentation acknowledgement for a single read-only RESET preview.

use super::{ControllerContext, WorkerState};
use crate::worker::WorkerEvent;


/// One admitted refresh; completion and frame publication may arrive in either order.
pub(super) struct ResetPreview {
    /// Checked monotonic request identity, never gameplay authority.
    pub(super) request_id: u64,
    /// Original socket/game/strategy/generation.
    pub(super) context: ControllerContext,
    /// Successful ordered terminal event has arrived.
    finished: bool,
    /// Preserve a genuine failure across this read-only refresh.
    pub(super) preserves_failure: bool,
    /// RESET may acknowledge historical failure presentation; STOP never masks a real failure.
    pub(super) acknowledges_failure: bool,
    /// Exact token/context frame has been installed successfully.
    frame_installed: bool,
}


impl ResetPreview {
    /// Begin one refresh without changing the controller context.
    pub(super) fn new(request_id: u64, context: ControllerContext) -> Self {
        Self {
            request_id,
            context,
            finished: false,
            preserves_failure: false,
            acknowledges_failure: true,
            frame_installed: false,
        }
    }


    /// Refresh after typed operator cancellation, preserving every genuine diagnostic.
    pub(super) fn after_stop(
        request_id: u64,
        context: ControllerContext,
        state: WorkerState,
    ) -> Self {
        let mut refresh = Self::new(request_id, context);
        refresh.acknowledges_failure = false;
        refresh.preserves_failure = matches!(state, WorkerState::Error | WorkerState::Uncertain);
        refresh
    }


    /// Retain correlation across a new genuine failure and the queued disconnect.
    pub(super) fn preserve_new_failure(&mut self) -> bool {
        if !self.pending() || self.acknowledges_failure {
            return false;
        }
        self.preserves_failure = true;
        true
    }


    /// Ignore obsolete generic progress while retaining genuine STOP-refresh failures.
    pub(super) fn suppress_event(&mut self, event: &WorkerEvent) -> bool {
        if !self.pending() {
            return false;
        }
        match event {
            WorkerEvent::State(WorkerState::Error | WorkerState::Uncertain)
                if !self.acknowledges_failure =>
            {
                self.preserves_failure = true;
                false
            }
            WorkerEvent::Status(_) if self.preserves_failure && !self.acknowledges_failure => false,
            WorkerEvent::State(_) | WorkerEvent::Status(_) => true,
            _ => false,
        }
    }


    /// Remain busy until both signals, then restore any genuine failure presentation.
    pub(super) fn worker_state(&self) -> WorkerState {
        if self.pending() {
            WorkerState::Connecting
        } else if self.preserves_failure {
            WorkerState::Error
        } else {
            WorkerState::Ready
        }
    }


    /// Require both identity and full controller context for correlated evidence.
    pub(super) fn accepts(&self, request_id: u64, context: Option<&ControllerContext>) -> bool {
        self.pending() && self.request_id == request_id && context == Some(&self.context)
    }


    /// Remain pending until both independent success signals have arrived.
    pub(super) fn pending(&self) -> bool {
        !self.finished || !self.frame_installed
    }


    /// Record the successful ordered worker endpoint.
    pub(super) fn finish(&mut self) {
        self.finished = true;
    }


    /// Record installation of the matching decoded frame.
    pub(super) fn install(&mut self) {
        self.frame_installed = true;
    }
}


/// Side effect admitted only by retirement of the exact matching owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CompletionDispatch {
    /// Stale or duplicate completion.
    Ignored,
    /// Ordinary success or genuine failure.
    Finished,
    /// Exactly one read-only STOP preview has been requested.
    StopPreview,
}


/// Invoke the production preview dispatch only after accepted operator cancellation.
pub(super) fn dispatch_completion(
    accepted: bool,
    termination: crate::pyramid_workflow_model::RunTermination,
    stop_preview: impl FnOnce(),
) -> CompletionDispatch {
    if !accepted {
        return CompletionDispatch::Ignored;
    }
    if termination != crate::pyramid_workflow_model::RunTermination::OperatorStopped {
        return CompletionDispatch::Finished;
    }
    stop_preview();
    CompletionDispatch::StopPreview
}


/// Distinguish persistence of an acknowledged failure from a genuinely new failure.
/// UI failure identities survive persistence even when the advisory run identity changes.
pub(super) fn same_failure_update(
    previous: Option<&crate::pyramid_diagnostics::PyramidDiagnostic>,
    incoming: &crate::pyramid_diagnostics::PyramidDiagnostic,
) -> bool {
    let Some(previous) = previous else {
        return false;
    };
    previous.context == incoming.context
        && previous.phase == incoming.phase
        && previous.error == incoming.error
        && match (previous.ui_failure_id, incoming.ui_failure_id) {
            (Some(before), Some(after)) => before == after,
            (None, None) => previous.run_id.is_some() && previous.run_id == incoming.run_id,
            _ => false,
        }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        game::GameMode,
        pyramid_diagnostics::{PyramidDiagnostic, PyramidPhase},
        strategy::SolvingStrategy,
    };
    use std::path::PathBuf;

    /// Stable selected context, without altering an acquisition generation.
    fn context() -> ControllerContext {
        ControllerContext::new(
            PathBuf::from("/reset-test.sock"),
            GameMode::Pyramid,
            SolvingStrategy::ShortestPath,
            7,
        )
    }


    /// Neither event delivery order completes early; retired tokens cannot install again.
    #[test]
    fn reset_176_correlated_preview_requires_both_success_signals() {
        for frame_first in [false, true] {
            let selected = context();
            let mut refresh = ResetPreview::new(41, selected.clone());
            assert!(refresh.accepts(41, Some(&selected)));
            assert!(!refresh.accepts(40, Some(&selected)));
            assert!(!refresh.accepts(41, None));
            let other = ControllerContext::new(
                PathBuf::from("/other.sock"),
                GameMode::Pyramid,
                SolvingStrategy::ShortestPath,
                7,
            );
            assert!(!refresh.accepts(41, Some(&other)));
            if frame_first {
                refresh.install();
            } else {
                refresh.finish();
            }
            assert!(refresh.pending());
            if frame_first {
                refresh.finish();
            } else {
                refresh.install();
            }
            assert!(!refresh.pending());
            assert!(!refresh.accepts(41, Some(&selected)));
            let failed: Option<ResetPreview> = None;
            assert!(
                !failed
                    .as_ref()
                    .is_some_and(|r| r.accepts(41, Some(&selected)))
            );
        }
    }


    /// Persistence metadata is not a new error; missing or different identities are.
    #[test]
    fn reset_176_failure_acknowledgement_preserves_only_the_same_failure() {
        let before = PyramidDiagnostic {
            progress: None,
            context: context(),
            phase: PyramidPhase::Reconnaissance,
            run_id: Some(7),
            ui_failure_id: Some(8),
            error: "original".to_owned(),
            directory: None,
            persistence_error: None,
        };
        let mut incoming = before.clone();
        incoming.run_id = None;
        incoming.directory = Some(PathBuf::from("/packet"));
        incoming.persistence_error = Some("retained".to_owned());
        assert!(same_failure_update(Some(&before), &incoming));
        incoming.error = "new".to_owned();
        assert!(!same_failure_update(Some(&before), &incoming));
        incoming.error = before.error.clone();
        incoming.ui_failure_id = Some(9);
        assert!(!same_failure_update(Some(&before), &incoming));
        let mut run = before.clone();
        run.ui_failure_id = None;
        incoming = run.clone();
        assert!(same_failure_update(Some(&run), &incoming));
        incoming.run_id = Some(9);
        assert!(!same_failure_update(Some(&run), &incoming));
        run.run_id = None;
        incoming = run.clone();
        assert!(!same_failure_update(Some(&run), &incoming));
        assert!(!same_failure_update(None, &incoming));
        incoming = before.clone();
        incoming.phase = PyramidPhase::Planning;
        assert!(!same_failure_update(Some(&before), &incoming));
    }

    /// A genuine failure keeps the token and absorbs the queued disconnect in either order.
    #[test]
    fn stop_183_preview_preserves_failure_through_disconnect_and_capture() {
        for frame_first in [false, true] {
            let selected = context();
            let mut refresh = ResetPreview::after_stop(42, selected.clone(), WorkerState::Ready);
            assert!(refresh.preserve_new_failure());
            assert!(refresh.accepts(42, Some(&selected)));
            assert!(!refresh.accepts(43, Some(&selected)));
            assert!(!refresh.accepts(42, None));
            assert!(refresh.suppress_event(&WorkerEvent::State(WorkerState::Detached)));
            assert!(refresh.suppress_event(&WorkerEvent::State(WorkerState::Ready)));
            assert!(!refresh.suppress_event(&WorkerEvent::State(WorkerState::Error)));
            assert!(!refresh.suppress_event(&WorkerEvent::Status("real failure".to_owned())));
            if frame_first {
                refresh.install();
            } else {
                refresh.finish();
            }
            assert_eq!(refresh.worker_state(), WorkerState::Connecting);
            if frame_first {
                refresh.finish();
            } else {
                refresh.install();
            }
            assert_eq!(refresh.worker_state(), WorkerState::Error);
            assert!(!refresh.accepts(42, Some(&selected)));
        }
        assert!(!ResetPreview::new(43, context()).preserve_new_failure());
    }


    /// Clean cancellation waits for both success signals; genuine prior errors survive.
    #[test]
    fn stop_183_preview_waits_in_both_orders_and_retains_prior_error() {
        for initial in [
            WorkerState::Ready,
            WorkerState::Error,
            WorkerState::Uncertain,
        ] {
            for frame_first in [false, true] {
                let mut refresh = ResetPreview::after_stop(42, context(), initial);
                assert!(!refresh.acknowledges_failure);
                if frame_first {
                    refresh.install();
                } else {
                    refresh.finish();
                }
                assert_eq!(refresh.worker_state(), WorkerState::Connecting);
                if frame_first {
                    refresh.finish();
                } else {
                    refresh.install();
                }
                assert_eq!(
                    refresh.worker_state(),
                    if initial == WorkerState::Ready {
                        WorkerState::Ready
                    } else {
                        WorkerState::Error
                    }
                );
            }
        }
    }
}
