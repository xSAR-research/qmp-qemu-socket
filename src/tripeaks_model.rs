#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

//! Pure 52-slot TriPeaks rules, replay verification, `xsar::search` adapters, and JSON persistence.
//!
//! This crate module keeps all TriPeaks game logic, streak/peak scoring, route verification,
//! and atomic `65,536`-byte deal export inside `qmp-qemu-socket` while leaving `xsar` untouched
//! and domain-neutral.

pub mod json;
pub mod replay;
pub mod rules;
pub mod search;

#[cfg(test)]
mod tests;

use crate::recon::{Card, CardSlot, SlotId};

pub use json::{
    DealProvenance, DestinationState, JsonError, MAX_DOCUMENT_BYTES, PublishDealError,
    SCHEMA_VERSION, TriPeaksDealDocument, from_slice, publish_to_path, read_from, to_vec,
};
pub use replay::{ReplayError, ReplayReport, verify_move_sequence, verify_objective_route};
pub use rules::{
    COVERING_CHILD_MASKS, PEAK_CLEAR_BONUS, RuleViolation, StepOutcome, TriPeaksPosition,
    apply_move, are_ranks_adjacent, card_rank_index, covering_children, is_slot_uncovered,
    legal_moves, slot_to_tableau_coord, stock_slot_id, tableau_slot_id, tap_score_delta,
};
pub use search::{
    DEFAULT_SEARCH_LIMITS, SearchState, best_partial_route, best_partial_route_with_control,
    plan_info_gain_probe, plan_info_gain_probe_with_control, solve_known_subgraph,
    solve_known_subgraph_with_control,
};

/// Total number of slots in a standard TriPeaks single-deck deal (`28` tableau + `1` waste + `23` stock).
pub const DEAL_SLOT_COUNT: usize = 52;

/// Number of initial tableau slots (`Rows 1..=4`: `3 + 6 + 9 + 10`).
pub const TABLEAU_SLOT_COUNT: usize = 28;

/// Immutable slot index of the initial `T = 0` face-up Waste card (`SlotId(28)`).
pub const INITIAL_WASTE_SLOT: SlotId = SlotId(28);

/// First slot index of the ordered `23`-card Stock draw pile (`SlotId(29..52)`).
pub const STOCK_START_SLOT: u8 = 29;

/// Number of Stock cards in an untouched `T = 0` TriPeaks deal.
pub const STOCK_CARD_COUNT: u8 = 23;

/// Maximum legal moves in any TriPeaks deal (`28` tableau taps + `23` Stock draws).
pub const MAX_ROUTE_MOVES: usize = 51;

/// Bitmask covering all `28` tableau positions (`bits 0..28`).
pub const FULL_TABLEAU_MASK: u32 = (1_u32 << TABLEAU_SLOT_COUNT) - 1;

/// Bitmask covering the three `Row 1` Peak positions (`bits 0..3`).
pub const PEAK_MASK: u32 = 0b111;

/// Physical TriPeaks action expressed in 1-based external coordinates.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TriPeaksMove {
    /// Tap an uncovered `Known` tableau slot (`row` in `1..=4`, `column` in `1..=row_length`).
    TapTableau {
        /// One-based tableau row (`1..=4`).
        row: u8,
        /// One-based tableau column (`1..=3`, `1..=6`, `1..=9`, or `1..=10`).
        column: u8,
    },
    /// Press `D` / Draw to flip the next `Known` Stock card onto Waste.
    Draw,
}

/// Structural validation failure for a `52`-slot TriPeaks deal table.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ModelError {
    /// Deal slot slice does not contain `52` entries.
    #[error("expected {DEAL_SLOT_COUNT} TriPeaks deal slots, found {actual}")]
    InvalidSlotTableLength {
        /// Actual slice length supplied.
        actual: usize,
    },
    /// A `CardSlot::Known` entry held a card identifier outside `0..52`.
    #[error("invalid canonical card id {card_id} at {slot:?} (expected 0..52)")]
    InvalidCardId {
        /// Slot holding the out-of-range card identifier.
        slot: SlotId,
        /// Invalid card ID.
        card_id: u8,
    },
    /// Single-deck deal contains the same `Known` card at two distinct slots.
    #[error("duplicate card {card:?} at {first_slot:?} and {second_slot:?}")]
    DuplicateKnownCard {
        /// Earlier slot bound to `card`.
        first_slot: SlotId,
        /// Later conflicting slot bound to `card`.
        second_slot: SlotId,
        /// Duplicated canonical card.
        card: Card,
    },
    /// One-based `(row, column)` coordinate falls outside the TriPeaks tableau.
    #[error("invalid TriPeaks tableau coordinate ({row}, {column})")]
    InvalidTableauCoord {
        /// Out-of-range row.
        row: u8,
        /// Out-of-range column.
        column: u8,
    },
    /// Initial `SlotId(28)` Waste card is not `CardSlot::Known`.
    #[error("initial Waste slot 28 must be Known before simulation")]
    UnknownInitialWaste,
}

/// Validate that `slots` has length `52`, all `Known` IDs are `< 52`, and no `Known` card repeats.
///
/// # Errors
///
/// Returns [`ModelError`] on bad slice length, out-of-range card ID, or duplicate single-deck card.
pub fn validate_slot_table(slots: &[CardSlot]) -> Result<&[CardSlot; DEAL_SLOT_COUNT], ModelError> {
    let fixed: &[CardSlot; DEAL_SLOT_COUNT] =
        slots
            .try_into()
            .map_err(|_| ModelError::InvalidSlotTableLength {
                actual: slots.len(),
            })?;
    let mut seen_slot: [Option<u8>; DEAL_SLOT_COUNT] = [None; DEAL_SLOT_COUNT];
    for (idx, entry) in fixed.iter().enumerate() {
        let slot_u8 = u8::try_from(idx).map_err(|_| ModelError::InvalidSlotTableLength {
            actual: slots.len(),
        })?;
        if let CardSlot::Known(card) = *entry {
            if card.0 >= 52 {
                return Err(ModelError::InvalidCardId {
                    slot: SlotId(slot_u8),
                    card_id: card.0,
                });
            }
            let card_idx = usize::from(card.0);
            if let Some(first) = seen_slot[card_idx] {
                return Err(ModelError::DuplicateKnownCard {
                    first_slot: SlotId(first),
                    second_slot: SlotId(slot_u8),
                    card,
                });
            }
            seen_slot[card_idx] = Some(slot_u8);
        }
    }
    Ok(fixed)
}
