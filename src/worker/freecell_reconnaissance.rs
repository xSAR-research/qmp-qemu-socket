//! One-frame native TCP v3 Free Cell acquisition; no guest input or solver access.
use super::{
    CaptureBackend, WorkerEvent, WorkerEventSink, WorkerState, pyramid_stop::StopTrace,
    selected_capture_backend, tcp_capture,
};
use crate::capture::CapturedFrame;
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
    /// Honest classification of this frame's objective; unresolved grants no route authority.
    pub objective: &'static str,
    /// Wire identity and dimensions copied from the same successful acquisition.
    pub capture: serde_json::Value,
    /// Same-frame objective evidence, when a challenge remains unresolved.
    pub acquisition_directory: Option<PathBuf>,
    /// Evidence omissions never change the successful JSON publication result.
    pub persistence_error: Option<String>,
}


/// One explicit request's acquired pixels and last attempted worker stage.
pub(super) struct Acquisition {
    /// Immutable UI identity; neither telemetry nor evidence grants input authority.
    request_id: u64,
    /// Exact successful capture, retained even when later recognition refuses.
    pub frame: Option<CapturedFrame>,
    /// Protocol identifiers returned alongside exactly these pixels.
    identity: Option<super::CaptureIdentity>,
    /// Stage reached before the original error occurred.
    stage: &'static str,
}


impl Acquisition {
    /// Start without pixels; only the one requested transport call can supply them.
    pub(super) fn new(request_id: u64) -> Self {
        Self {
            request_id,
            frame: None,
            identity: None,
            stage: "request validation",
        }
    }


    /// Describe an existing stage without acquiring evidence or changing authority.
    fn stage(&mut self, stage: &'static str) {
        self.stage = stage;
        crate::single_step_telemetry::record(|| {
            serde_json::json!({
                "event":"recon-stage", "request_id":self.request_id, "stage":stage,
            })
        });
    }


    /// Retain dimensions and wire identifiers together without serialising pixel bytes.
    fn capture_details(&self) -> serde_json::Value {
        self.frame.as_ref().map_or(serde_json::Value::Null, |frame| serde_json::json!({
            "width":frame.width,"height":frame.height,"stride_bytes":frame.stride,
            "format":format!("{:?}",frame.format),"pixel_bytes":frame.pixels.len(),
            "source":"same explicit RECON acquisition","request_id":self.request_id,
            "transport_identity":self.identity.map(|identity|serde_json::json!({
                "protocol_version":identity.protocol_version,"session_id":identity.session_id,
                "request_id":identity.request_id,"frame_id":identity.frame_id,
                "display_generation":identity.display_generation,
                "reused_connection":identity.reused_connection,
            })),
        }))
    }
}


/// Immutable primary failure, with independently reported evidence persistence.
#[derive(Debug)]
pub(crate) struct FailureReport {
    /// Original worker error, never replaced by packet or UI formatting failures.
    pub error: String,
    /// Original request, stage, recognition region and acquired-frame metadata.
    pub details: serde_json::Value,
    /// Reserved private packet, including partial writes.
    pub directory: Option<PathBuf>,
    /// Secondary persistence errors or explicit evidence omissions.
    pub persistence_error: Option<String>,
}


/// Preserve only evidence already acquired for this request; no retry or input.
pub(super) fn failure_report(
    context: &ControllerContext,
    request: &Request,
    acquisition: &Acquisition,
    failure: Failure,
) -> FailureReport {
    let error = failure.to_string();
    let recognition = match &failure {
        Failure::Recognition(error) => Some(serde_json::json!({
            "profile":error.profile, "place":format!("{:?}",error.place),
            "region":format!("{:?}",error.region), "bounds":format!("{:?}",error.bounds),
            "kind":format!("{:?}",error.kind), "cause":error.cause,
        })),
        _ => None,
    };
    let details = serde_json::json!({
        "operation":"Free Cell RECON", "request_id":acquisition.request_id,
        "context":format!("{context:#?}"), "deal_id":request.deal_id,
        "destination":request.destination, "stage":acquisition.stage,
        "recognition":recognition,
        "capture":acquisition.capture_details(),
        "guest_input_attempted":false,
    });
    let saved =
        crate::pyramid_diagnostics::persist_failure(crate::pyramid_diagnostics::FailureEvidence {
            progress: None,
            context,
            phase: crate::pyramid_diagnostics::PyramidPhase::Reconnaissance,
            run_id: Some(acquisition.request_id),
            error: &error,
            details: details.clone(),
            source_document: None,
            first_frame: acquisition.frame.as_ref(),
            latest_frame: None,
        });
    crate::single_step_telemetry::record(|| {
        serde_json::json!({
            "event":"recon-failure-evidence", "request_id":acquisition.request_id,
            "error":error, "stage":acquisition.stage,
            "directory":saved.directory, "persistence_error":saved.persistence_error,
        })
    });
    FailureReport {
        error,
        details,
        directory: saved.directory,
        persistence_error: saved.persistence_error,
    }
}


/// Keep unresolved objective pixels in a separately labelled acquisition packet.
/// Successful JSON publication remains independent of secondary evidence failures.
fn retain_objective_evidence(
    context: &ControllerContext,
    acquisition: &Acquisition,
    deal: xsar::games::freecell::InitialDeal,
) -> (
    xsar::games::freecell::InitialDeal,
    Option<PathBuf>,
    Option<String>,
) {
    use crate::freecell_recon::objective::{self, Objective};
    if objective::from_metadata(deal.metadata()) != Objective::Unresolved {
        return (deal, None, None);
    }
    let saved = crate::pyramid_diagnostics::persist_failure(
        crate::pyramid_diagnostics::FailureEvidence {
            progress: None,
            context,
            phase: crate::pyramid_diagnostics::PyramidPhase::Reconnaissance,
            run_id: Some(acquisition.request_id),
            error: "Free Cell RECON acquisition evidence: recognition succeeded; challenge objective unresolved",
            details: serde_json::json!({"artifact_kind":"successful acquisition evidence",
                "recognition":"succeeded", "request_id":acquisition.request_id,
                "objective":"unresolved", "game_effect_proved":false,
                "capture":acquisition.capture_details()}),
            source_document: None,
            first_frame: acquisition.frame.as_ref(),
            latest_frame: None,
        },
    );
    let mut persistence_error = saved.persistence_error;
    let labelled = saved
        .directory
        .as_ref()
        .and_then(|directory| directory.file_name())
        .and_then(|name| name.to_str())
        .map(|name| objective::attach_evidence(&deal, name));
    let deal = match labelled {
        Some(Ok(labelled)) => labelled,
        Some(Err(error)) => {
            persistence_error = Some(format!(
                "{}Objective evidence reference: {error}",
                persistence_error
                    .as_ref()
                    .map_or(String::new(), |previous| format!("{previous}; "))
            ));
            deal
        }
        None => deal,
    };
    crate::single_step_telemetry::record(|| {
        serde_json::json!({
            "event":"recon-objective-acquisition-evidence", "request_id":acquisition.request_id,
            "recognition":"succeeded", "directory":saved.directory,
            "persistence_error":persistence_error,
        })
    });
    (deal, saved.directory, persistence_error)
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
pub(super) fn backend() -> Result<(String, crate::capture::CaptureMode), String> {
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
    acquisition: &mut Acquisition,
) -> Result<Report, Failure> {
    run_checked_with_deal(context, request, events, cancel, trace, acquisition, |_| {})
        .map(|(report, _deal)| report)
}

/// Capture, recognise, and export once under the current reservation, retaining the
/// validated immutable deal for worker-owned sessions and notifying `on_frame` as soon
/// as a raw board screenshot is acquired.
pub(super) fn run_checked_with_deal(
    context: &ControllerContext,
    request: &Request,
    events: &WorkerEventSink,
    cancel: &AtomicBool,
    trace: &StopTrace,
    acquisition: &mut Acquisition,
    on_frame: impl FnOnce(&CapturedFrame),
) -> Result<(Report, xsar::games::freecell::InitialDeal), Failure> {
    acquisition.stage("request validation");
    request.validate(context).map_err(Failure::Request)?;
    acquisition.stage("transport selection");
    let (address, mode) = backend().map_err(Failure::UnsupportedTransport)?;
    acquisition.stage("capture authority");
    trace
        .check(&events.latest_frame.reconnaissance, context, cancel)
        .map_err(Failure::Revoked)?;
    acquisition.stage("fresh screenshot acquisition");
    let (frame, timing) = tcp_capture::capture_causal(&address, mode, events).map_err(|e| {
        let cancelled = e.cancelled;
        let message = trace.capture_error(e, &events.latest_frame.reconnaissance, context, cancel);
        Failure::Capture { message, cancelled }
    })?;
    // Retain exactly these pixels even if recognition, context or publication subsequently refuses.
    acquisition.frame = Some(frame.clone());
    acquisition.identity = timing.identity;
    on_frame(&frame);
    acquisition.stage("post-capture authority");
    trace
        .check(&events.latest_frame.reconnaissance, context, cancel)
        .map_err(Failure::Revoked)?;
    let metadata = request.metadata().map_err(Failure::Request)?;
    acquisition.stage("recognition");
    let deal = crate::freecell_recon::recognise_checked(&frame, metadata, &mut || {
        trace.check(&events.latest_frame.reconnaissance, context, cancel)
    })?;
    acquisition.stage("objective acquisition evidence");
    let (deal, acquisition_directory, persistence_error) =
        retain_objective_evidence(context, acquisition, deal);
    acquisition.stage("publication authority");
    trace
        .check(&events.latest_frame.reconnaissance, context, cancel)
        .map_err(Failure::Revoked)?;
    acquisition.stage("no-overwrite JSON publication");
    if let Err(error) = freecell_json::write_new(&request.destination, &deal) {
        crate::single_step_telemetry::record(|| {
            serde_json::json!({
                "event":"recon-export", "request_id":acquisition.request_id,
                "destination":request.destination,"completed":false,"error":error.to_string(),
                "capture":acquisition.capture_details(),
            })
        });
        return Err(Failure::Publication(error));
    }
    crate::single_step_telemetry::record(|| {
        serde_json::json!({
            "event":"recon-export", "request_id":acquisition.request_id,
            "destination":request.destination,"completed":true,"publication":"published",
            "capture":acquisition.capture_details(),
        })
    });
    acquisition.stage("completed");
    let report = Report {
        destination: request.destination.clone(),
        objective: crate::freecell_recon::objective::from_metadata(deal.metadata()).label(),
        capture: acquisition.capture_details(),
        acquisition_directory,
        persistence_error,
    };
    Ok((report, deal))
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
    let mut acquisition = Acquisition::new(request_id);
    let result = run_checked(context, request, events, cancel, &trace, &mut acquisition);
    let termination = trace.termination(result.is_err(), true);
    let result =
        result.map_err(|failure| Box::new(failure_report(context, request, &acquisition, failure)));
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
            frame: acquisition.frame,
            termination,
            result,
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
