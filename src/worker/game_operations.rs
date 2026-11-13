//! Native operation adapter over retained worker capture, QMP and STOP timing.

use super::{
    CaptureTiming, QmpClient, WorkerEventSink, cancellable_wait, capture_screen, validate_probe,
};
use crate::{
    capture::CapturedFrame,
    game_operations::{Input, OperationError, OperationIo},
};
use std::{
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};


/// Borrow existing worker resources; callers supply their exact context/deadline gate.
///
/// No new connection/thread/protocol is created. Selected-game action evidence is
/// checked separately on the captured frame, and Session owns the no-replay latch.
pub(super) struct NativeOperations<'a, A, G> {
    /// Existing QMP transport, with configured click/key intervals.
    pub qmp: &'a mut QmpClient,
    /// Canonical selected socket path for the retained capture backend.
    pub socket: &'a Path,
    /// Existing event and retained TCP capture state.
    pub events: &'a WorkerEventSink,
    /// Existing STOP latch; never cleared by operations or restoration.
    pub cancel: &'a AtomicBool,
    /// Caller-owned exact context, shutdown and deadline check.
    pub active: A,
    /// Caller-owned pure selected-game/scene/action evidence check.
    pub guard: G,
    /// Last capture's existing transport timing, distinct from semantic completion.
    pub last_timing: Option<CaptureTiming>,
}


impl<A, G> OperationIo for NativeOperations<'_, A, G>
where
    A: FnMut() -> Result<(), OperationError>,
    G: FnMut(&CapturedFrame, Input) -> Result<(), OperationError>,
{
    fn check_active(&mut self) -> Result<(), OperationError> {
        if self.cancel.load(Ordering::Acquire) {
            return Err(OperationError::Cancelled);
        }
        (self.active)()
    }

    fn capture(&mut self) -> Result<CapturedFrame, OperationError> {
        self.last_timing = None;
        self.check_active()?;
        let (frame, timing) = capture_screen(self.qmp, self.socket, self.events)
            .map_err(OperationError::Transport)?;
        self.last_timing = Some(timing);
        Ok(frame)
    }

    fn authorise(&mut self, frame: &CapturedFrame, input: Input) -> Result<(), OperationError> {
        (self.guard)(frame, input)
    }

    fn probe(&mut self) -> Result<(), OperationError> {
        let probe = self
            .qmp
            .probe()
            .map_err(|e| OperationError::Transport(e.to_string()))?;
        validate_probe(&probe).map_err(OperationError::Verification)
    }

    fn deliver(&mut self, input: Input) -> Result<(), OperationError> {
        match input {
            Input::MouseLeft(point) => self.qmp.click(point, 1_920, 1_080),
            Input::Key(key) => self.qmp.press_key(key),
        }
        .map_err(|e| OperationError::Transport(e.to_string()))
    }

    fn wait(&mut self, duration: Duration) -> Result<(), OperationError> {
        cancellable_wait(duration, self.cancel)
            .map(|_| ())
            .map_err(|_| OperationError::Cancelled)
    }
}
