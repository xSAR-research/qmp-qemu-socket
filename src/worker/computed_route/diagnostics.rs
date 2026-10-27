//! Bounded read-only descriptions of the current frame and unverified route action.
//!
//! These values grant no input, replay or resume authority. Card identities always
//! come from the original deal and shared rules; observations retain their own frame.

use super::{
    CompleteDeal, FrameGuardPhase, Location, Prepared, RecordedAction, SearchState, expected,
    operations, rules,
};
use serde_json::{Value, json};
use xsar::games::cards::Card;


/// Metadata belonging to exactly the latest successfully captured native frame.
#[derive(Debug)]
pub(super) struct Observation {
    /// One-based capture attempt that actually produced the retained frame.
    pub(super) capture: usize,
    /// Source or effect classification requested for these same pixels.
    pub(super) phase: FrameGuardPhase,
    /// Shared rule state whose complete visible position was requested.
    pub(super) state: SearchState,
    /// Whether final-route alternatives were allowed for this observation.
    pub(super) final_effect: bool,
    /// Actual acknowledged whole gestures before this frame was obtained.
    pub(super) acknowledged: usize,
    /// This frame's refusal or pending reason; cleared by each new captured frame.
    pub(super) mismatch: Option<String>,
}


impl Observation {
    /// Describe these pixels and their expected state without recapture or transport.
    /// Invalid rule locations return a diagnostic error, never execution authority.
    pub(super) fn value(&self, deal: &CompleteDeal) -> Result<Value, String> {
        Ok(json!({
            "capture": self.capture,
            "phase": match self.phase {
                FrameGuardPhase::Source => "source",
                FrameGuardPhase::Effect => "effect",
            },
            "final_effect": self.final_effect,
            "acknowledged_gestures": self.acknowledged,
            "expected_state_internal": format!("{:?}", self.state),
            "expected_position": position(deal, self.state)?,
            "unmatched_observation": self.mismatch,
        }))
    }
}


/// Render an original card identity while keeping positive empty separate.
fn card_label(card: Option<Card>) -> Option<String> {
    card.map(|card| format!("{} of {}", card.rank().as_str(), card.suit().as_str()))
}


/// Preserve public one-based row|column evidence at the internal location boundary.
fn location_name(location: Location) -> Result<String, String> {
    match location {
        Location::DL => Ok("DL".to_owned()),
        Location::DR => Ok("DR".to_owned()),
        Location::Tableau(slot) => {
            let (row, column) = xsar::games::pyramid::planning::tableau_coordinates(slot.index())
                .ok_or("Invalid diagnostic tableau location")?;
            Ok(format!("{row}|{column}"))
        }
    }
}


/// Describe fixed original tableau slots and visible piles using the existing rules.
fn position(deal: &CompleteDeal, state: SearchState) -> Result<Value, String> {
    let (tableau, left, right) = expected(deal, state)?;
    let mut slots = Vec::with_capacity(tableau.len());
    for (index, card) in tableau.into_iter().enumerate() {
        let slot =
            super::TableauLocation::from_index(index).ok_or("Invalid diagnostic tableau index")?;
        slots.push(json!({
            "location": location_name(Location::Tableau(slot))?,
            "card": card_label(card),
        }));
    }
    Ok(json!({"tableau": slots, "DL": card_label(left), "DR": card_label(right)}))
}


/// Describe a deliberate source card from the original deal, never from elimination.
fn source(deal: &CompleteDeal, state: SearchState, location: Location) -> Result<Value, String> {
    let card = rules::card_index(state, location).and_then(|index| rules::card_at(deal, index));
    Ok(json!({"location": location_name(location)?, "card": card_label(card)}))
}


/// Describe the next unverified prepared action and its legal before/after positions.
/// A completed trace has no pending action. Invalid prefixes return diagnostic
/// errors for a separate field and must never replace the original execution error.
pub(super) fn pending_action(plan: &Prepared, verified: usize) -> Result<Option<Value>, String> {
    if verified > plan.trace.len() || plan.trace.len() > super::MAX_ROUTE_ACTIONS {
        return Err("Diagnostic verified prefix is outside the bounded prepared trace".to_owned());
    }
    let Some(step) = plan.trace.get(verified) else {
        return Ok(None);
    };
    let deal = &plan.acquisition.deal;
    let mut before = SearchState::initial();
    for prior in plan.trace.iter().take(verified) {
        before = rules::apply(deal, before, prior.action).map_err(|error| error.to_string())?;
    }
    let after = rules::apply(deal, before, step.action).map_err(|error| error.to_string())?;
    let (action, cards) = match step.action {
        RecordedAction::Match(first, second) => (
            "MATCH",
            vec![source(deal, before, first)?, source(deal, before, second)?],
        ),
        RecordedAction::King(location) => ("KING", vec![source(deal, before, location)?]),
        RecordedAction::Move => ("MOVE", Vec::new()),
        RecordedAction::Recycle => ("RECYCLE", Vec::new()),
    };
    let gestures: Vec<_> = operations(step.action)?
        .iter()
        .map(|operation| format!("{operation:?}"))
        .collect();
    Ok(Some(json!({
        "action_number": verified.checked_add(1).ok_or("Diagnostic action number overflow")?,
        "action": action,
        "cards": cards,
        "before": position(deal, before)?,
        "after": position(deal, after)?,
        "gestures": gestures,
        "authority": "diagnostic_only_no_resume_or_replay",
    })))
}
