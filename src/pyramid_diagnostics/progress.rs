//! Immutable failure-stage facts, independent of retained-frame provenance.

use serde_json::{Value, json};


/// Exact controller boundary at which work stopped.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FailureStage {
    /// Expected original sources have not yet authorised input.
    #[default]
    InitialSource,
    /// A once-only typed operation is being delivered.
    InputDelivery,
    /// Two fresh source-disappearance confirmations are pending.
    PostConfirmation,
    /// Independent terminal or board-clear evidence is pending.
    FinalConfirmation,
    /// Bounded stock acquisition or original-board restoration.
    Reconnaissance,
}


impl FailureStage {
    /// Stable user-visible boundary; never derived from a frame phase.
    pub const fn label(self) -> &'static str {
        match self {
            Self::InitialSource => "INITIAL source verification",
            Self::InputDelivery => "INPUT delivery",
            Self::PostConfirmation => "POST-action confirmation",
            Self::FinalConfirmation => "FINAL semantic confirmation",
            Self::Reconnaissance => "RECON acquisition/restoration",
        }
    }
}


/// Worker-owned facts frozen when failure is published, even if persistence fails.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FailureProgress {
    /// Current controller boundary, independent of the latest image's phase.
    pub stage: FailureStage,
    /// One-based logical action, when an action exists.
    pub action: Option<usize>,
    /// Number of independently replayed logical actions, when available.
    pub total: Option<usize>,
    /// Current action or acquisition substage in readable text.
    pub operation: Option<String>,
    /// Initial source checks, including a reused boundary frame.
    pub initial_checks: usize,
    /// Initial finite check allowance; zero means unavailable.
    pub initial_limit: usize,
    /// Actual captures taken for initial checks, excluding reuse.
    pub fresh_retries: usize,
    /// Successive fresh disappearance confirmations, bounded to two.
    pub post_confirmations: usize,
    /// Previous completed action confirmations, when retained by the execution owner.
    pub previous_post_confirmations: Option<usize>,
    /// Last actual native capture serial.
    pub capture: usize,
    /// Verified logical-action prefix, separate from transport acknowledgements.
    pub verified_actions: usize,
    /// Actual total typed gestures attempted.
    pub attempted: usize,
    /// Actual total typed gestures acknowledged.
    pub acknowledged: usize,
    /// Total attempts immediately before the current action began.
    pub action_attempted_start: usize,
    /// Total acknowledgements immediately before the current action began.
    pub action_acknowledged_start: usize,
    /// Expected and observed identities at the same exact locations.
    pub sources: Vec<String>,
    /// Current refusal or pending reason; never replaced by an earlier mismatch.
    pub reason: Option<String>,
    /// Earlier first-unmatched evidence, retained separately without assuming recovery.
    pub earlier_observation: Option<String>,
}


impl FailureProgress {
    /// Inspection-only JSON snapshot; no field grants input or resume authority.
    pub fn as_json(&self) -> Value {
        json!({"stage":self.stage.label(),"action":self.action,"total_actions":self.total,
            "operation":self.operation,"initial_checks":self.initial_checks,
            "initial_limit":self.initial_limit,"fresh_initial_captures":self.fresh_retries,
            "post_confirmations":self.post_confirmations,"previous_post_confirmations":self.previous_post_confirmations,"capture":self.capture,
            "verified_actions":self.verified_actions,"gestures_attempted":self.attempted,
            "gestures_acknowledged":self.acknowledged,
            "current_action_attempted":self.attempted.saturating_sub(self.action_attempted_start),
            "current_action_acknowledged":self.acknowledged.saturating_sub(self.action_acknowledged_start),
            "sources":self.sources,"final_reason":self.reason,"earlier_observation":self.earlier_observation})
    }


    /// Selectable failure-screen text, explicitly separating current and prior facts.
    pub fn display_text(&self) -> String {
        let action = self.action.map_or_else(
            || "unavailable".to_owned(),
            |n| {
                self.total
                    .map_or_else(|| n.to_string(), |total| format!("{n} of {total}"))
            },
        );
        let current_attempted = self.attempted.saturating_sub(self.action_attempted_start);
        let current_acknowledged = self
            .acknowledged
            .saturating_sub(self.action_acknowledged_start);
        let current_input = if current_attempted == 0 {
            "none sent"
        } else if current_attempted == current_acknowledged {
            "acknowledged; effect is separate"
        } else {
            "delivery incomplete or uncertain; no replay"
        };
        let checks = if self.initial_limit == 0 {
            "unavailable".to_owned()
        } else {
            format!(
                "{}/{}; fresh captures: {}",
                self.initial_checks, self.initial_limit, self.fresh_retries
            )
        };
        format!(
            "Current stage: {}\nAction: {}\nOperation: {}\nSuccessfully completed: {} logical actions\nCumulative gestures: {} attempted / {} acknowledged\nCurrent action input: {} ({} attempted / {} acknowledged)\nINITIAL checks: {}\nCurrent action POST confirmations: {}/2\nPrevious action POST confirmations: {}\nLatest capture: {}\nSources:\n{}\nFinal refusal: {}\n\nEarlier first-unmatched observation (historical): {}",
            self.stage.label(),
            action,
            self.operation.as_deref().unwrap_or("unavailable"),
            self.verified_actions,
            self.attempted,
            self.acknowledged,
            current_input,
            current_attempted,
            current_acknowledged,
            checks,
            self.post_confirmations,
            self.previous_post_confirmations
                .map_or_else(|| "unavailable".to_owned(), |n| format!("{n}/2")),
            self.capture,
            if self.sources.is_empty() {
                "unavailable".to_owned()
            } else {
                self.sources.join("\n")
            },
            self.reason.as_deref().unwrap_or("see original error below"),
            self.earlier_observation
                .as_deref()
                .unwrap_or("none recorded")
        )
    }
}


#[cfg(test)]
mod tests {
    use super::*;


    /// Prior acknowledgements cannot be mistaken for input in the refused current action.
    #[test]
    fn readiness_173_failure_snapshot_separates_stage_prior_work_and_current_input() {
        let mut progress = FailureProgress {
            action: Some(5),
            total: Some(67),
            initial_checks: 6,
            initial_limit: 6,
            fresh_retries: 5,
            verified_actions: 4,
            attempted: 5,
            acknowledged: 5,
            action_attempted_start: 5,
            action_acknowledged_start: 5,
            reason: Some("9S at 7|7 unreadable".to_owned()),
            earlier_observation: Some("source board scene[0] mismatch".to_owned()),
            ..Default::default()
        };
        let snapshot = progress.clone();
        progress.stage = FailureStage::PostConfirmation;
        let text = snapshot.display_text();
        assert!(text.contains("INITIAL source verification"));
        assert!(text.contains("none sent (0 attempted / 0 acknowledged)"));
        assert!(text.contains("4 logical actions"));
        assert!(text.contains("6/6; fresh captures: 5"));
        assert_eq!(snapshot.as_json()["current_action_attempted"], 0);
        assert_eq!(snapshot.as_json()["verified_actions"], 4);
        progress.stage = FailureStage::InputDelivery;
        progress.attempted = 6;
        assert!(
            progress
                .display_text()
                .contains("delivery incomplete or uncertain; no replay")
        );
        assert_eq!(progress.as_json()["current_action_acknowledged"], 0);
    }
}
