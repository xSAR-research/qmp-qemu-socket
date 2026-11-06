//! One-frame native TCP v3 Free Cell acquisition; no guest input or solver access.
use super::{
    CaptureBackend, WorkerEvent, WorkerEventSink, WorkerState, pyramid_stop::StopTrace,
    selected_capture_backend, tcp_capture,
};
use crate::{
    game::GameMode,
    pyramid_workflow_model::RunTermination,
    strategy::{ControllerContext, SolvingStrategy},
};
use std::{path::PathBuf, sync::atomic::AtomicBool};
use xsar::{freecell_json, games::freecell::Metadata};

/// Explicit output identity; descriptive metadata is never used as a path component.
#[derive(Debug)]
pub(super) struct Request {
    /// Exact operator-selected destination, never overwritten.
    pub destination: PathBuf,
    /// Descriptive acquisition token independent of destination spelling.
    pub deal_id: String,
}
impl Request {
    /// Reject unsupported mode and invalid metadata before acquiring pixels.
    pub fn validate(&self, context: &ControllerContext) -> Result<(), String> {
        context.validate()?;
        if context.game_mode != GameMode::FreeCell
            || context.strategy != SolvingStrategy::ShortestPath
        {
            return Err("Free Cell RECON requires independent Free Cell mode".into());
        }
        if self.destination.as_os_str().is_empty() {
            return Err("Free Cell destination is empty".into());
        }
        self.metadata().map(|_| ())
    }
    /// Build bounded codec metadata without deriving a filename from it.
    fn metadata(&self) -> Result<Metadata, String> {
        Metadata::new(&self.deal_id, "native-freecell-v1", "single-frame-tcp-v3")
            .map_err(|e| e.to_string())
    }
}
/// Successful no-overwrite publication; all identities come from its one acquired frame.
#[derive(Debug)]
pub(crate) struct Report {
    /// Published destination.
    pub destination: PathBuf,
}

/// Typed first failure, formatted only when emitting the completion event.
#[derive(Debug, thiserror::Error)]
pub(super) enum Failure {
    /// Invalid request, before acquisition.
    #[error("Free Cell request: {0}")]
    Request(String),
    /// Unsupported selected transport, without automatic fallback.
    #[error("Free Cell transport: {0}")]
    UnsupportedTransport(String),
    /// Revoked reserved owner or context.
    #[error("Free Cell authority: {0}")]
    Revoked(String),
    /// Original capture cause and typed transport cancellation discriminator.
    #[error("Free Cell capture (cancelled={cancelled}): {message}")]
    Capture {
        /// Original transport/capture diagnostic.
        message: String,
        /// True only for transport's typed Cancelled variant.
        cancelled: bool,
    },
    /// Full profile, physical location, region and first recognition cause.
    #[error("{0}")]
    Recognition(#[from] crate::freecell_recon::RecognitionError),
    /// No-overwrite publication retains its exact completion-status evidence.
    #[error("Free Cell export: {0}")]
    Publication(#[from] freecell_json::PublishError),
}

/// Require the selected v3 transport without guessing, switching or falling back.
fn backend() -> Result<(String, crate::capture::CaptureMode), String> {
    match selected_capture_backend()? {
        CaptureBackend::Tcp { address, mode: mode @ (crate::capture::CaptureMode::BgrV3 | crate::capture::CaptureMode::PersistentBgrV3) } => Ok((address,mode)),
        _ => Err("Free Cell RECON requires selected TCP v3 or v3-persistent; no fallback or capture was attempted".into()),
    }
}
/// Capture and export once under the current reservation; STOP prevents publication.
pub(super) fn run_checked(
    context: &ControllerContext,
    request: &Request,
    events: &WorkerEventSink,
    cancel: &AtomicBool,
    trace: &StopTrace,
) -> Result<Report, Failure> {
    request.validate(context).map_err(Failure::Request)?;
    let (address, mode) = backend().map_err(Failure::UnsupportedTransport)?;
    trace
        .check(&events.latest_frame.reconnaissance, context, cancel)
        .map_err(Failure::Revoked)?;
    let (frame, _timing) = tcp_capture::capture_causal(&address, mode, events).map_err(|e| {
        let cancelled = e.cancelled;
        let message = trace.capture_error(e, &events.latest_frame.reconnaissance, context, cancel);
        Failure::Capture { message, cancelled }
    })?;
    // Retain exactly these pixels even if recognition, context or publication subsequently refuses.
    events
        .latest_frame
        .publish(frame.clone(), None, false, true, Some(context.clone()));
    trace
        .check(&events.latest_frame.reconnaissance, context, cancel)
        .map_err(Failure::Revoked)?;
    let metadata = request.metadata().map_err(Failure::Request)?;
    let deal = crate::freecell_recon::recognise_checked(&frame, metadata, &mut || {
        trace.check(&events.latest_frame.reconnaissance, context, cancel)
    })?;
    trace
        .check(&events.latest_frame.reconnaissance, context, cancel)
        .map_err(Failure::Revoked)?;
    freecell_json::write_new(&request.destination, &deal)?;
    Ok(Report {
        destination: request.destination.clone(),
    })
}
/// Serial worker envelope retains causal cancellation and emits one correlated completion.
pub(super) fn run(
    context: &ControllerContext,
    request_id: u64,
    request: &Request,
    events: &WorkerEventSink,
    cancel: &AtomicBool,
) {
    let trace = StopTrace::default();
    let result = run_checked(context, request, events, cancel, &trace);
    let termination = trace.termination(result.is_err(), true);
    events.latest_frame.reconnaissance.finish();
    let _ = events.send_for_context(
        Some(context.clone()),
        WorkerEvent::State(if termination == RunTermination::Failed {
            WorkerState::Uncertain
        } else {
            WorkerState::Ready
        }),
    );
    let _ = events.send_for_context(
        Some(context.clone()),
        WorkerEvent::FreeCellReconFinished {
            request_id,
            termination,
            result: result.map_err(|e| e.to_string()),
        },
    );
}

/// Explicit preview uses the selected v3 path but never recognises or exports a deal.
pub(super) fn preview(context: &ControllerContext, events: &WorkerEventSink) {
    let result = (|| {
        context.validate()?;
        let (address, mode) = backend()?;
        let (frame, _) =
            tcp_capture::capture_causal(&address, mode, events).map_err(|e| e.message)?;
        events
            .latest_frame
            .publish(frame, None, false, false, Some(context.clone()));
        Ok::<(), String>(())
    })();
    match result {
        Ok(()) => {
            let _ = events.send_for_context(
                Some(context.clone()),
                WorkerEvent::State(WorkerState::Ready),
            );
        }
        Err(error) => {
            let _ = events.send_for_context(
                Some(context.clone()),
                WorkerEvent::State(WorkerState::Error),
            );
            let _ = events.send_for_context(Some(context.clone()), WorkerEvent::Status(error));
        }
    }
}
