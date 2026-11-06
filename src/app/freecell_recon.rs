//! Identity-bound UI ownership for the read-only, single-frame Free Cell workflow.

use crate::{pyramid_workflow_model::RunTermination, strategy::ControllerContext};

/// The serial never wraps; only the dedicated matching completion retires a request.
#[derive(Default)]
pub(super) struct FreeCellReconState {
    /// Last allocated request identity, including failed dispatches.
    serial: u64,
    /// Exact requested context and token, independent of generic worker state.
    pending: Option<(ControllerContext, u64)>,
    /// Retain acquired diagnostic pixels through the queued STOP disconnect.
    stopped_preview: Option<ControllerContext>,
}

impl FreeCellReconState {
    /// Reserve one explicit request; no capture or publication occurs in this reducer.
    pub(super) fn reserve(&mut self, context: ControllerContext) -> Result<u64, &'static str> {
        if self.pending.is_some() {
            return Err("Free Cell RECON is already active");
        }
        let id = self
            .serial
            .checked_add(1)
            .ok_or("Free Cell request identity exhausted")?;
        self.serial = id;
        self.pending = Some((context, id));
        self.stopped_preview = None;
        Ok(id)
    }

    /// Release a failed dispatch without reusing its token.
    pub(super) fn dispatch_failed(&mut self) {
        self.pending = None;
    }

    /// Consume only the original request identity; STOP never schedules another capture.
    pub(super) fn finish(
        &mut self,
        context: &ControllerContext,
        id: u64,
        _termination: RunTermination,
    ) -> bool {
        if self
            .pending
            .as_ref()
            .is_none_or(|(expected, token)| expected != context || *token != id)
        {
            return false;
        }
        self.pending = None;
        // Any accepted result can race with an already queued STOP disconnect.
        // Keep this frame for inspection until an explicit new request or selection.
        self.stopped_preview = Some(context.clone());
        true
    }

    /// Preserve the same-frame diagnostic through either polling order of Disconnect.
    pub(super) fn retain_after_detach(&self, context: &ControllerContext) -> bool {
        self.stopped_preview.as_ref() == Some(context)
    }

    /// Explicit new capture or selection retires the previous inspection marker.
    pub(super) fn clear_preview(&mut self) {
        self.stopped_preview = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{game::GameMode, strategy::SolvingStrategy};

    /// No stale/duplicate completion can consume another request or initiate recovery I/O.
    #[test]
    fn freecell_300_completion_is_context_bound_and_stop_retains_one_frame() {
        let context = ControllerContext::new(
            "/tmp/freecell-test.sock".into(),
            GameMode::FreeCell,
            SolvingStrategy::ShortestPath,
            6,
        );
        let mut state = FreeCellReconState::default();
        let id = state.reserve(context.clone()).unwrap();
        assert!(state.reserve(context.clone()).is_err());
        let mut changed = context.clone();
        changed.generation += 1;
        assert!(!state.finish(&changed, id, RunTermination::OperatorStopped));
        assert!(!state.finish(&context, id + 1, RunTermination::OperatorStopped));
        assert!(state.finish(&context, id, RunTermination::OperatorStopped));
        assert!(state.retain_after_detach(&context));
        assert!(!state.retain_after_detach(&changed));
        assert!(!state.finish(&context, id, RunTermination::OperatorStopped));
        // Detached can occur before or after mailbox installation; neither consumes this marker.
        assert!(state.retain_after_detach(&context));
        state.clear_preview();
        assert!(!state.retain_after_detach(&context));
        let next = state.reserve(context.clone()).unwrap();
        assert!(next > id);
        assert!(state.finish(&context, next, RunTermination::Failed));
        assert!(state.retain_after_detach(&context));
    }

    /// Failed dispatches and exhausted counters never reuse a request identity.
    #[test]
    fn freecell_300_failed_dispatch_and_serial_exhaustion_are_closed() {
        let context = ControllerContext::new(
            "/tmp/freecell-test.sock".into(),
            GameMode::FreeCell,
            SolvingStrategy::ShortestPath,
            7,
        );
        let mut state = FreeCellReconState::default();
        let first = state.reserve(context.clone()).unwrap();
        state.dispatch_failed();
        assert!(!state.finish(&context, first, RunTermination::Completed));
        let second = state.reserve(context.clone()).unwrap();
        assert_eq!(second, first + 1);
        assert!(state.finish(&context, second, RunTermination::Completed));
        state.serial = u64::MAX;
        assert!(state.reserve(context).is_err());
        assert!(state.pending.is_none());
    }

    /// A genuine failure followed by queued STOP retains the acquired pixels in either poll order.
    #[test]
    fn freecell_300_failure_then_stop_disconnect_keeps_same_context_evidence() {
        let context = ControllerContext::new(
            "/tmp/freecell-race.sock".into(),
            GameMode::FreeCell,
            SolvingStrategy::ShortestPath,
            8,
        );
        for termination in [
            RunTermination::Failed,
            RunTermination::Completed,
            RunTermination::OperatorStopped,
        ] {
            let mut state = FreeCellReconState::default();
            let id = state.reserve(context.clone()).unwrap();
            assert!(state.finish(&context, id, termination));
            // Both pre-mailbox and later-poll Detached checks retain the same inspection marker.
            assert!(state.retain_after_detach(&context));
            assert!(state.retain_after_detach(&context));
            assert!(!state.finish(&context, id, termination));
            let mut changed = context.clone();
            changed.generation += 1;
            assert!(!state.retain_after_detach(&changed));
            state.clear_preview();
            assert!(!state.retain_after_detach(&context));
        }
    }
}
