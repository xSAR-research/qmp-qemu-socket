//! Run-local causal cancellation, separate from text and the final shared STOP latch.

use std::cell::Cell;

use super::{
    pyramid_reconnaissance::{AuthorityError, ReconControl},
    *,
};
use crate::pyramid_workflow_model::RunTermination;


/// The exact failing boundary can identify deliberate cancellation without hiding other errors.
#[derive(Default)]
pub(super) struct StopTrace {
    /// Set only by an authority refusal caused by operator STOP for this run.
    operator: Cell<bool>,
    /// A propagated genuine failure dominates any later cancellation observation.
    fatal: Cell<bool>,
}


impl StopTrace {
    /// Check reserved authority, retaining the typed cause before legacy string propagation.
    pub(super) fn check(
        &self,
        control: &ReconControl,
        context: &ControllerContext,
        cancel: &AtomicBool,
    ) -> Result<(), String> {
        self.retain(control.check_active_causal(context, cancel))
    }


    /// Preserve the cause from one atomic active-owner/receipt-epoch check.
    pub(super) fn check_receipt(
        &self,
        control: &ReconControl,
        context: &ControllerContext,
        cancel: &AtomicBool,
        epoch: u64,
    ) -> Result<(), String> {
        self.retain(control.check_receipt_causal(context, cancel, Some(epoch)))
    }


    /// Record only an error that is being propagated from this exact boundary.
    fn retain(&self, result: Result<(), AuthorityError>) -> Result<(), String> {
        result.map_err(|error| {
            if matches!(error, AuthorityError::OperatorStop) {
                self.operator.set(true);
            } else {
                self.fatal.set(true);
            }
            error.to_string()
        })
    }


    /// Classify only transport's typed cancellation, never a coincident genuine capture error.
    pub(super) fn capture_error(
        &self,
        error: CaptureFailure,
        control: &ReconControl,
        context: &ControllerContext,
        cancel: &AtomicBool,
    ) -> String {
        if error.cancelled {
            let _ = self.check(control, context, cancel);
        } else {
            self.fatal.set(true);
        }
        error.message
    }


    /// Carry the consumed nested execution's causal result into its owning session.
    pub(super) fn adopt(&self, termination: RunTermination) {
        match termination {
            RunTermination::Failed => self.fatal.set(true),
            RunTermination::OperatorStopped => self.operator.set(true),
            RunTermination::Completed => {}
        }
    }


    /// An unacknowledged attempt always remains a failure, even during deliberate STOP.
    pub(super) fn termination(&self, failed: bool, balanced: bool) -> RunTermination {
        if !balanced || self.fatal.get() {
            RunTermination::Failed
        } else if !failed {
            RunTermination::Completed
        } else if balanced && self.operator.get() {
            RunTermination::OperatorStopped
        } else {
            RunTermination::Failed
        }
    }
}


/// Capture failure retains only the transport's explicit cancellation discriminator.
pub(super) struct CaptureFailure {
    /// Original diagnostic used by existing callers.
    pub(super) message: String,
    /// True only for the TCP protocol's typed Cancelled result.
    pub(super) cancelled: bool,
}


impl From<String> for CaptureFailure {
    /// Non-cancellation backend, layout, PNG and recording errors remain genuine failures.
    fn from(message: String) -> Self {
        Self {
            message,
            cancelled: false,
        }
    }
}


/// Causal route failure; its message remains the existing diagnostic for compatibility.
pub(super) struct ExecutionFailure {
    /// Original finite-route diagnostic, including actual delivery counts.
    pub(super) message: String,
    /// Exact causal outcome, independent of a later user action.
    pub(super) termination: RunTermination,
}
