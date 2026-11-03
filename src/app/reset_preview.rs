//! Correlation and presentation acknowledgement for a single read-only RESET preview.

use super::ControllerContext;


/// One admitted refresh; completion and frame publication may arrive in either order.
pub(super) struct ResetPreview {
    /// Checked monotonic request identity, never gameplay authority.
    pub(super) request_id: u64,
    /// Original socket/game/strategy/generation.
    pub(super) context: ControllerContext,
    /// Successful ordered terminal event has arrived.
    finished: bool,
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
            frame_installed: false,
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
}
