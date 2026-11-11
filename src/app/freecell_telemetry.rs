//! Passive UI intent records for the temporary A* Free Cell development trace.
//!
//! UI control methods are not on the worker thread and cannot inherit its TLS scope.
//! Records go directly to the existing private session logger; they do not enter the
//! visible log, create files, acquire frames, dispatch commands or grant authority.

use super::*;
use serde_json::{Value, json};


/// Format one context-bound UI stage only for the selected A* Free Cell workflow.
fn ui_record(
    context: &ControllerContext,
    owner: Option<ActiveRun>,
    stage: &str,
    details: Value,
) -> Option<String> {
    if context.strategy != SolvingStrategy::ShortestPath || context.game_mode != GameMode::FreeCell
    {
        return None;
    }
    Some(format!(
        "FREE_CELL {}",
        json!({
            "event":"ui-operation", "stage":stage,
            "context":format!("{context:?}"), "owner":format!("{owner:?}"),
            "details":details, "ack_means":"delivery only", "effect_proved":false,
            "logging_input_events":0, "logging_additional_captures":0
        })
    ))
}


impl QmpQemuSocketApp {
    /// Record an initiated UI step with the current context and supplied frozen identity.
    ///
    /// `details` describes intent or a local reducer result, never transport completion.
    pub(super) fn trace_freecell_ui(&mut self, stage: &str, details: Value) {
        self.trace_freecell_ui_at(self.started_at.elapsed(), stage, details);
    }


    /// Preserve the original UI/worker timestamp when recording an existing log stage.
    pub(super) fn trace_freecell_ui_at(&mut self, elapsed: Duration, stage: &str, details: Value) {
        if let Some(record) = ui_record(
            &self.current_preview_context(),
            self.active_run,
            stage,
            details,
        ) {
            self.push_session_telemetry_at(elapsed, record);
        }
    }
}


#[cfg(test)]
mod tests {
    //! Trace selection and frozen operation identity are testable without an application.

    use super::*;


    /// UI intents stay isolated from Computer Vision and every other game selection.
    #[test]
    fn freecell_313_ui_telemetry_preserves_identity_only_in_selected_scope() {
        for game in GameMode::AVAILABLE {
            for strategy in SolvingStrategy::AVAILABLE {
                let context = ControllerContext::new(
                    PathBuf::from("/tmp/passive-ui-telemetry.sock"),
                    game,
                    strategy,
                    13,
                );
                let record = ui_record(
                    &context,
                    Some(ActiveRun::FreeCellCalculation),
                    "pause-request",
                    json!({"request_id":19,"source_generation":7,"pause_epoch":23}),
                );
                if game == GameMode::FreeCell && strategy == SolvingStrategy::ShortestPath {
                    let value: Value =
                        serde_json::from_str(record.unwrap().strip_prefix("FREE_CELL ").unwrap())
                            .unwrap();
                    assert_eq!(value["stage"], "pause-request");
                    assert_eq!(value["details"]["request_id"], 19);
                    assert_eq!(value["details"]["source_generation"], 7);
                    assert_eq!(value["details"]["pause_epoch"], 23);
                    assert_eq!(value["effect_proved"], false);
                    assert_eq!(value["logging_additional_captures"], 0);
                    assert_eq!(value["logging_input_events"], 0);
                    assert!(
                        value["context"]
                            .as_str()
                            .unwrap()
                            .contains("generation: 13")
                    );
                } else {
                    assert!(record.is_none());
                }
            }
        }
    }
}
