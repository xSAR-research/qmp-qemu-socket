#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

//! Deterministic offline replay verification for TriPeaks move sequences.

use crate::recon::{CardSlot, Objective, SlotId};

use super::{
    DEAL_SLOT_COUNT, MAX_ROUTE_MOVES, ModelError, TriPeaksMove,
    rules::{RuleViolation, TriPeaksPosition, apply_move},
    validate_slot_table,
};

/// Errors produced when replaying or certifying a planned [`TriPeaksMove`] sequence.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ReplayError {
    /// Deal slot table failed structural validation.
    #[error(transparent)]
    Model(#[from] ModelError),
    /// Candidate move sequence exceeds `MAX_ROUTE_MOVES` (`51`).
    #[error("TriPeaks route length {actual} exceeds maximum {MAX_ROUTE_MOVES}")]
    RouteTooLong {
        /// Supplied move sequence length.
        actual: usize,
    },
    /// A move violated TriPeaks occupancy, covering, `Known`, or rank adjacency rules.
    #[error("replay failed at step {step} ({mv:?}): {source}")]
    Rule {
        /// Zero-based index of the failing move.
        step: usize,
        /// Failing move.
        mv: TriPeaksMove,
        /// Underlying rule violation.
        source: RuleViolation,
    },
    /// Complete legal sequence ended without achieving the requested [`Objective`].
    #[error("route is legal over {steps_executed} moves but did not achieve {objective:?}")]
    ObjectiveNotSatisfied {
        /// Expected objective.
        objective: Objective,
        /// Legal moves replayed before checking goal satisfaction.
        steps_executed: usize,
    },
}

/// Summary returned after independently verifying every step of a [`TriPeaksMove`] list.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayReport {
    /// Terminal board position after replaying every move.
    pub final_position: TriPeaksPosition,
    /// Number of verified moves executed.
    pub steps_executed: usize,
    /// Ordered, deduplicated `CardSlot::Unknown` tableau slots newly uncovered during replay.
    pub newly_uncovered_unknown_slots: Vec<SlotId>,
}

/// Verify that `moves` executes legally from `T = 0` against `slots` without mutation.
///
/// Tracks newly uncovered `CardSlot::Unknown` tableau parents in discovery order for probe
/// validation without treating `Unknown` slots as playable or cleared.
///
/// # Errors
///
/// Returns [`ReplayError`] if `slots` is invalid, `moves` exceeds [`MAX_ROUTE_MOVES`], or
/// any move breaks TriPeaks rules.
pub fn verify_move_sequence(
    slots: &[CardSlot],
    moves: &[TriPeaksMove],
) -> Result<ReplayReport, ReplayError> {
    let fixed: &[CardSlot; DEAL_SLOT_COUNT] = validate_slot_table(slots)?;
    if moves.len() > MAX_ROUTE_MOVES {
        return Err(ReplayError::RouteTooLong {
            actual: moves.len(),
        });
    }
    let mut position = TriPeaksPosition::from_root(fixed)?;
    let mut newly_uncovered_unknown_slots = Vec::new();

    for (step, &mv) in moves.iter().enumerate() {
        let (next, outcome) = apply_move(&position, fixed, mv)
            .map_err(|source| ReplayError::Rule { step, mv, source })?;
        for slot in outcome.newly_uncovered_slots {
            if matches!(fixed[usize::from(slot.0)], CardSlot::Unknown)
                && !newly_uncovered_unknown_slots.contains(&slot)
            {
                newly_uncovered_unknown_slots.push(slot);
            }
        }
        position = next;
    }

    Ok(ReplayReport {
        final_position: position,
        steps_executed: moves.len(),
        newly_uncovered_unknown_slots,
    })
}

/// Verify that `moves` is legal from `T = 0` against `slots` and satisfies `objective`.
///
/// # Errors
///
/// Returns [`ReplayError`] if step replay fails or the terminal state does not satisfy `objective`.
pub fn verify_objective_route(
    slots: &[CardSlot],
    moves: &[TriPeaksMove],
    objective: Objective,
) -> Result<ReplayReport, ReplayError> {
    let report = verify_move_sequence(slots, moves)?;
    if !report.final_position.satisfies_objective(objective) {
        return Err(ReplayError::ObjectiveNotSatisfied {
            objective,
            steps_executed: report.steps_executed,
        });
    }
    Ok(report)
}
