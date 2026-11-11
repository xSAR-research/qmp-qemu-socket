//! Passive development tracing for every A* Free Cell worker command.
//!
//! Scope selection uses the command's immutable context, never the latest preview.
//! Events use the existing session channel; tracing adds no I/O or gameplay authority.

use super::*;
use serde_json::{Value, json};


/// Select only the Free Cell A* command owner, including offline and read-only work.
fn identity(command: &WorkerCommand) -> Option<(&ControllerContext, Value)> {
    let (context, detail) = match command {
        WorkerCommand::CalculateFreeCell { identity, path, .. } => (
            &identity.context,
            json!({"command":"CALCULATE", "request_id":identity.request_id,
                "source_generation":identity.source_generation, "source":path}),
        ),
        WorkerCommand::ExecuteFreeCellRoute {
            plan,
            run_id,
            single_step,
            drag_mode,
            drag_timing,
            settle,
        } => (
            &plan.context,
            json!({"command":"playback", "request_id":plan.request_id,
                "source_generation":plan.source_generation,"run_id":run_id,
                "single_step":single_step,
                "diagnostic_drag_mode":single_step.then(||format!("{drag_mode:?}")),
                "diagnostic_drag_timing":single_step.then(||format!("{drag_timing:?}")),
                "settle_ms":settle.as_millis()}),
        ),
        WorkerCommand::ReconnoitreFreeCell {
            context,
            request_id,
            request,
        } => (
            context,
            json!({"command":"RECON", "request_id":request_id,
                "export":request.destination}),
        ),
        WorkerCommand::ListFreeCellSources {
            context,
            request_id,
            directory,
        } => (
            context,
            json!({"command":"list-sources","request_id":request_id,"directory":directory}),
        ),
        WorkerCommand::CaptureFrame { context } => (context, json!({"command":"capture-frame"})),
        WorkerCommand::CaptureResetPreview {
            context,
            request_id,
        } => (
            context,
            json!({"command":"reset-preview","request_id":request_id}),
        ),
        WorkerCommand::PrepareSnapshot {
            context,
            request_id,
        } => (
            context,
            json!({"command":"snapshot","request_id":request_id}),
        ),
        WorkerCommand::RecordPyramidFailure { context, phase, .. } => (
            context,
            json!({"command":"persist-failure","phase":format!("{phase:?}")}),
        ),
        WorkerCommand::ExecuteSteps { context, .. } => {
            (context, json!({"command":"legacy-execute-request"}))
        }
        WorkerCommand::ResetProgress(context) => (context, json!({"command":"reset-progress"})),
        WorkerCommand::Disconnect(context) => (context, json!({"command":"disconnect"})),
        _ => return None,
    };
    (context.game_mode == GameMode::FreeCell && context.strategy == SolvingStrategy::ShortestPath)
        .then_some((context, detail))
}


/// Install one serial command's trace without altering the capture context or STOP latch.
pub(super) fn scope(
    command: &WorkerCommand,
    events: &WorkerEventSink,
) -> Option<crate::single_step_telemetry::Scope> {
    let (context, detail) = identity(command)?;
    let sender = events.events.clone();
    let owner = context.clone();
    Some(crate::single_step_telemetry::Scope::freecell(
        json!({"context":format!("{context:?}"),"request":detail}),
        move |message| {
            let _ = sender.send(ContextualWorkerEvent {
                context: Some(owner.clone()),
                emitted_at: worker_elapsed(),
                event: WorkerEvent::SessionTelemetry(message),
            });
        },
    ))
}


/// Record existing ordered results before UI polling; telemetry events never recurse.
pub(super) fn event(event: &WorkerEvent) {
    if !crate::single_step_telemetry::is_freecell() {
        return;
    }
    crate::single_step_telemetry::record(|| match event {
        WorkerEvent::Log(text) | WorkerEvent::Status(text) => {
            json!({"event":"application-stage","message":text})
        }
        WorkerEvent::State(state) => json!({"event":"worker-state","state":state.label()}),
        WorkerEvent::FreeCellSourcesListed {
            request_id,
            directory,
            result,
        } => json!({"event":"sources-result","request_id":request_id,"directory":directory,
                "file_count":result.as_ref().ok().map(Vec::len),"error":result.as_ref().err()}),
        WorkerEvent::FreeCellCalculationFinished { identity, result } => {
            json!({"event":"calculation-result","request_id":identity.request_id,
                "source_generation":identity.source_generation,"completed":result.is_ok(),
                "outcome":result.as_ref().ok().map(|report|match &report.outcome {
                    xsar::games::freecell::search::BestEffortOutcome::Solved{..}=>"Solved",
                    xsar::games::freecell::search::BestEffortOutcome::Cancelled{..}=>"Cancelled",
                    xsar::games::freecell::search::BestEffortOutcome::Partial{..}=>"ProvedMaximumPartial",
                    xsar::games::freecell::search::BestEffortOutcome::Failed{..}=>"Failed",
                }),
                "stats":result.as_ref().ok().map(|report|format!("{:?}",freecell_calculation::stats(&report.outcome))),
                "error":result.as_ref().err(),"input_authorised":false})
        }
        WorkerEvent::FreeCellReconFinished {
            request_id,
            termination,
            result,
            ..
        } => json!({"event":"recon-result","request_id":request_id,
                "termination":format!("{termination:?}"),"completed":result.is_ok()}),
        WorkerEvent::FreeCellRouteFinished {
            plan,
            run_id,
            termination,
            result,
        } => json!({"event":"playback-result","request_id":plan.request_id,"run_id":run_id,
                "termination":format!("{termination:?}"),"completed":result.is_ok(),
                "progress":result.as_ref().ok().map(|p|json!({"proved_cursor":p.cursor,
                    "total":p.total,"attempted":p.attempted,"acknowledged":p.acknowledged,
                    "completion":p.completion.map(|reason|format!("{reason:?}")),
                    "probe_evidence":p.probe_evidence.as_ref().map(|evidence|json!({
                        "capture_serial":evidence.capture_serial,"directory":evidence.directory,
                        "persistence_error":evidence.persistence_error}))})),
                "error":result.as_ref().err().map(|failure| &failure.error)}),
        WorkerEvent::SnapshotPrepared {
            request_id,
            frame,
            png,
            ..
        } => json!({"event":"snapshot-result","request_id":request_id,
                "layout":[frame.width,frame.height,frame.stride],"png_bytes":png.len()}),
        WorkerEvent::SnapshotPreparationFailed { request_id, error } => {
            json!({"event":"snapshot-result","request_id":request_id,"error":error})
        }
        _ => json!({"event":"worker-notification"}),
    });
}


#[cfg(test)]
mod tests {
    //! Prove scope isolation and source identity without capture, QMP or filesystem work.

    use super::*;


    /// All other game and strategy combinations stay outside enhanced Free Cell tracing.
    #[test]
    fn freecell_313_telemetry_is_limited_to_astar_freecell() {
        for game_mode in GameMode::AVAILABLE {
            for strategy in SolvingStrategy::AVAILABLE {
                let context = ControllerContext {
                    socket_path: PathBuf::from("/tmp/passive-telemetry.sock"),
                    game_mode,
                    strategy,
                    generation: 41,
                };
                let command = WorkerCommand::CaptureFrame {
                    context: context.clone(),
                };
                assert_eq!(
                    identity(&command).is_some(),
                    game_mode == GameMode::FreeCell && strategy == SolvingStrategy::ShortestPath
                );
            }
        }
    }
}
