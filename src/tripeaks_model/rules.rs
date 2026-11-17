#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

//! Immutable TriPeaks covering geometry, legal-move generation, and Beast-verified scoring rules.

use crate::{
    cards::{TABLEAU_ROW_LENGTHS, TABLEAU_ROW_STARTS},
    recon::{Card, CardSlot, Objective, SlotId},
};

use super::{
    DEAL_SLOT_COUNT, FULL_TABLEAU_MASK, INITIAL_WASTE_SLOT, ModelError, STOCK_CARD_COUNT,
    STOCK_START_SLOT, TABLEAU_SLOT_COUNT, TriPeaksMove, validate_slot_table,
};

/// Verified Row-1 Peak clear bonus (`+500` points awarded alongside the card's streak value).
pub const PEAK_CLEAR_BONUS: u32 = 500;

/// Compute the 28-slot child-covering bitmask table at compile time without slot compaction.
const fn build_covering_child_masks() -> [u32; TABLEAU_SLOT_COUNT] {
    let mut masks = [0_u32; TABLEAU_SLOT_COUNT];
    // Row 1 (`slots 0..3`): covered by pairs on Row 2 (`slots 3..9`).
    let mut peak = 0;
    while peak < 3 {
        let left = 3 + 2 * peak;
        let right = left + 1;
        masks[peak] = (1_u32 << left) | (1_u32 << right);
        peak += 1;
    }
    // Row 2 (`slots 3..9`): two slots per peak share adjacent Row 3 children (`slots 9..18`).
    let mut r2 = 0;
    while r2 < 6 {
        let p = r2 / 2;
        let side = r2 % 2;
        let left = 9 + 3 * p + side;
        let right = left + 1;
        masks[3 + r2] = (1_u32 << left) | (1_u32 << right);
        r2 += 1;
    }
    // Row 3 (`slots 9..18`): each covered by two adjacent Row 4 cards (`slots 18..28`).
    let mut r3 = 0;
    while r3 < 9 {
        let left = 18 + r3;
        let right = left + 1;
        masks[9 + r3] = (1_u32 << left) | (1_u32 << right);
        r3 += 1;
    }
    masks
}

/// Bitmask of child slots covering each tableau index `0..28` (`0` for Row 4).
pub const COVERING_CHILD_MASKS: [u32; TABLEAU_SLOT_COUNT] = build_covering_child_masks();

/// Map 1-based `(row, column)` tableau coordinates to immutable `SlotId(0..28)`.
pub const fn tableau_slot_id(row: u8, column: u8) -> Option<SlotId> {
    if row < 1 || row > 4 || column < 1 {
        return None;
    }
    let row_idx = (row - 1) as usize;
    let len = TABLEAU_ROW_LENGTHS[row_idx];
    if column > len {
        return None;
    }
    let slot = TABLEAU_ROW_STARTS[row_idx] + (column - 1) as usize;
    Some(SlotId(slot as u8))
}

/// Map `SlotId(0..28)` back to 1-based `(row, column)`; returns `None` for Stock/Waste `>= 28`.
pub const fn slot_to_tableau_coord(slot: SlotId) -> Option<(u8, u8)> {
    let idx = slot.0 as usize;
    if idx >= TABLEAU_SLOT_COUNT {
        return None;
    }
    let mut row = 0_usize;
    while row < 4 {
        let start = TABLEAU_ROW_STARTS[row];
        let end = start + TABLEAU_ROW_LENGTHS[row] as usize;
        if idx < end {
            return Some(((row + 1) as u8, (idx - start + 1) as u8));
        }
        row += 1;
    }
    None
}

/// Return the immutable `SlotId(29..52)` for a `0..23` Stock cursor.
pub const fn stock_slot_id(stock_cursor: u8) -> Option<SlotId> {
    if stock_cursor >= STOCK_CARD_COUNT {
        return None;
    }
    Some(SlotId(STOCK_START_SLOT + stock_cursor))
}

/// Return the two covering child `SlotId`s for `slot` (`Row 1..=3`), or `None` for `Row 4`/non-tableau.
pub const fn covering_children(slot: SlotId) -> Option<(SlotId, SlotId)> {
    let idx = slot.0 as usize;
    if idx >= 18 {
        return None;
    }
    let mask = COVERING_CHILD_MASKS[idx];
    let left = mask.trailing_zeros() as u8;
    let remaining = mask & !(1_u32 << left);
    let right = remaining.trailing_zeros() as u8;
    Some((SlotId(left), SlotId(right)))
}

/// Return true when `slot_idx` (`0..28`) has no occupied covering children in `tableau_mask`.
pub const fn is_slot_uncovered(tableau_mask: u32, slot_idx: u8) -> bool {
    if (slot_idx as usize) >= TABLEAU_SLOT_COUNT {
        return false;
    }
    (tableau_mask & COVERING_CHILD_MASKS[slot_idx as usize]) == 0
}

/// Return the zero-based rank index (`0 = Ace .. 12 = King`) for canonical card `0..52`.
pub const fn card_rank_index(card: Card) -> u8 {
    card.0 % 13
}

/// Return true when two zero-based ranks (`0..=12`) are `±1` adjacent with `King <-> Ace` wrap.
pub const fn are_ranks_adjacent(left_rank: u8, right_rank: u8) -> bool {
    if left_rank >= 13 || right_rank >= 13 {
        return false;
    }
    let delta = left_rank.abs_diff(right_rank);
    delta == 1 || delta == 12
}

/// Compute guaranteed points gained by a single tableau tap at `new_streak >= 1`.
///
/// Matches live Beast measurements: `100 * (2 * new_streak - 1)` per consecutive tap
/// plus `+500` when clearing a `Row 1` Peak (`slot 0..3`). Does not speculatively credit
/// uncalibrated end-of-board bonuses so `ReachScoreDelta` routes never fall short.
pub const fn tap_score_delta(new_streak: u8, is_peak: bool) -> u32 {
    if new_streak == 0 {
        return 0;
    }
    let base = 100_u32 * (2_u32 * (new_streak as u32) - 1);
    if is_peak {
        base + PEAK_CLEAR_BONUS
    } else {
        base
    }
}

/// Failure executing a candidate [`TriPeaksMove`] against a [`TriPeaksPosition`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum RuleViolation {
    /// Structural error on the underlying 52-slot deal table.
    #[error(transparent)]
    Model(#[from] ModelError),
    /// Move targets a slot already cleared from the tableau.
    #[error("tableau slot {slot:?} is already cleared")]
    SlotAlreadyCleared {
        /// Cleared slot targeted by move.
        slot: SlotId,
    },
    /// Move targets a tableau slot still covered by occupied Row `r + 1` children.
    #[error("tableau slot {slot:?} is still covered")]
    SlotCovered {
        /// Covered slot targeted by move.
        slot: SlotId,
    },
    /// Target tableau or next Stock slot is still `CardSlot::Unknown` (blocks play).
    #[error("slot {slot:?} is Unknown and blocks play")]
    SlotUnknown {
        /// Unknown slot blocking execution.
        slot: SlotId,
    },
    /// Target tableau or Stock slot is `CardSlot::Unreachable` (blocks play).
    #[error("slot {slot:?} is Unreachable and blocks play")]
    SlotUnreachable {
        /// Unreachable slot blocking execution.
        slot: SlotId,
    },
    /// Exposed `Known` card rank is not `±1 mod 13` adjacent to active Waste rank.
    #[error("card {card:?} is not +/-1 adjacent to active Waste {waste_card:?}")]
    RankNotAdjacent {
        /// Candidate tableau card.
        card: Card,
        /// Active Waste card.
        waste_card: Card,
    },
    /// `Draw` attempted when all `23` Stock cards have already been drawn.
    #[error("Stock exhausted (cursor == {STOCK_CARD_COUNT})")]
    StockExhausted,
}

/// Summary produced alongside the updated position when [`apply_move`] succeeds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StepOutcome {
    /// Points awarded by this single move (`0` for `Draw`).
    pub score_gained: u32,
    /// Tableau slot cleared by `TapTableau`, or `None` on `Draw`.
    pub cleared_slot: Option<SlotId>,
    /// Card placed onto the Waste pile by this move.
    pub new_waste_card: Card,
    /// Parent slots in `0..18` that transitioned from covered to uncovered (`Known` or `Unknown`).
    pub newly_uncovered_slots: Vec<SlotId>,
}

/// Immutable-deal-backed mutable gameplay state during TriPeaks simulation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TriPeaksPosition {
    /// Occupancy bitmask over `SlotId(0..28)` (`1` = still on board, `0` = cleared).
    pub tableau_mask: u32,
    /// Index (`0..=23`) of the next Stock card to draw (`SlotId(29 + stock_cursor)`).
    pub stock_cursor: u8,
    /// Currently visible top card of the Waste pile.
    pub waste_card: Card,
    /// Current consecutive tableau-clear streak (`Draw` resets to `0`).
    pub streak: u8,
    /// Cumulative verified point gain from `T = 0`.
    pub score_delta: u32,
    /// Count of cleared tableau cards per rank index (`0..=12`, Ace through King).
    pub cleared_by_rank: [u8; 13],
}

impl TriPeaksPosition {
    /// Construct the initial `T = 0` position from a validated `52`-slot deal.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError`] if `slots` fails validation or `SlotId(28)` is not `Known`.
    pub fn from_root(slots: &[CardSlot; DEAL_SLOT_COUNT]) -> Result<Self, ModelError> {
        validate_slot_table(slots)?;
        let CardSlot::Known(waste_card) = slots[usize::from(INITIAL_WASTE_SLOT.0)] else {
            return Err(ModelError::UnknownInitialWaste);
        };
        Ok(Self {
            tableau_mask: FULL_TABLEAU_MASK,
            stock_cursor: 0,
            waste_card,
            streak: 0,
            score_delta: 0,
            cleared_by_rank: [0; 13],
        })
    }

    /// Return all legal [`TriPeaksMove`]s from this state (`Unknown`/`Unreachable` slots excluded).
    pub fn legal_moves(&self, slots: &[CardSlot; DEAL_SLOT_COUNT]) -> Vec<TriPeaksMove> {
        legal_moves(self, slots)
    }

    /// Apply one legal [`TriPeaksMove`], returning the next position and step outcome.
    ///
    /// # Errors
    ///
    /// Returns [`RuleViolation`] when the move violates occupancy, cover, `Known`, or rank rules.
    pub fn apply_move(
        &self,
        slots: &[CardSlot; DEAL_SLOT_COUNT],
        mv: TriPeaksMove,
    ) -> Result<(Self, StepOutcome), RuleViolation> {
        apply_move(self, slots, mv)
    }

    /// Evaluate whether `objective` is satisfied at this board position.
    pub fn satisfies_objective(&self, objective: Objective) -> bool {
        match objective {
            Objective::ClearBoard => self.tableau_mask == 0,
            Objective::ClearRankQuota { rank, remaining } => {
                if remaining == 0 {
                    return true;
                }
                usize::from(rank) < 13 && self.cleared_by_rank[usize::from(rank)] >= remaining
            }
            Objective::ReachScoreDelta { points_needed } => self.score_delta >= points_needed,
        }
    }
}

/// Generate all deterministic legal [`TriPeaksMove`]s for `position` over `slots`.
///
/// Tableau taps appear in ascending `SlotId(0..28)` order, followed by [`TriPeaksMove::Draw`]
/// when a remaining Stock slot is `CardSlot::Known`. `Unknown` and `Unreachable` entries
/// never generate moves.
pub fn legal_moves(
    position: &TriPeaksPosition,
    slots: &[CardSlot; DEAL_SLOT_COUNT],
) -> Vec<TriPeaksMove> {
    let mut moves = Vec::with_capacity(11);
    let waste_rank = card_rank_index(position.waste_card);
    for (idx, entry) in slots.iter().take(TABLEAU_SLOT_COUNT).enumerate() {
        let bit = 1_u32 << idx;
        if (position.tableau_mask & bit) == 0
            || !is_slot_uncovered(position.tableau_mask, idx as u8)
        {
            continue;
        }
        if let CardSlot::Known(card) = *entry
            && are_ranks_adjacent(card_rank_index(card), waste_rank)
            && let Some((row, column)) = slot_to_tableau_coord(SlotId(idx as u8))
        {
            moves.push(TriPeaksMove::TapTableau { row, column });
        }
    }
    if let Some(stock_slot) = stock_slot_id(position.stock_cursor)
        && matches!(slots[usize::from(stock_slot.0)], CardSlot::Known(_))
    {
        moves.push(TriPeaksMove::Draw);
    }
    moves
}

/// Apply one [`TriPeaksMove`] to `position` against `slots`, returning the updated position and delta.
///
/// Clearing a child detects any newly uncovered occupied parents across `SlotId(0..18)`,
/// including parents whose identity is still `CardSlot::Unknown`.
///
/// # Errors
///
/// Returns [`RuleViolation`] if `mv` is illegal in `position` or on `slots`.
pub fn apply_move(
    position: &TriPeaksPosition,
    slots: &[CardSlot; DEAL_SLOT_COUNT],
    mv: TriPeaksMove,
) -> Result<(TriPeaksPosition, StepOutcome), RuleViolation> {
    validate_slot_table(slots)?;
    match mv {
        TriPeaksMove::TapTableau { row, column } => {
            let slot = tableau_slot_id(row, column)
                .ok_or(ModelError::InvalidTableauCoord { row, column })?;
            let idx = usize::from(slot.0);
            let bit = 1_u32 << idx;
            if (position.tableau_mask & bit) == 0 {
                return Err(RuleViolation::SlotAlreadyCleared { slot });
            }
            if !is_slot_uncovered(position.tableau_mask, slot.0) {
                return Err(RuleViolation::SlotCovered { slot });
            }
            let card = match slots[idx] {
                CardSlot::Known(card) if card.0 < 52 => card,
                CardSlot::Known(card) => {
                    return Err(ModelError::InvalidCardId {
                        slot,
                        card_id: card.0,
                    }
                    .into());
                }
                CardSlot::Unknown => return Err(RuleViolation::SlotUnknown { slot }),
                CardSlot::Unreachable => return Err(RuleViolation::SlotUnreachable { slot }),
            };
            if !are_ranks_adjacent(card_rank_index(card), card_rank_index(position.waste_card)) {
                return Err(RuleViolation::RankNotAdjacent {
                    card,
                    waste_card: position.waste_card,
                });
            }

            let prev_mask = position.tableau_mask;
            let next_mask = prev_mask & !bit;
            let next_streak = position.streak.saturating_add(1);
            let score_gained = tap_score_delta(next_streak, row == 1);
            let mut cleared_by_rank = position.cleared_by_rank;
            let rank_idx = usize::from(card_rank_index(card));
            cleared_by_rank[rank_idx] = cleared_by_rank[rank_idx].saturating_add(1);

            let mut newly_uncovered_slots = Vec::new();
            for parent in 0..18_u8 {
                let parent_bit = 1_u32 << parent;
                if (next_mask & parent_bit) != 0
                    && !is_slot_uncovered(prev_mask, parent)
                    && is_slot_uncovered(next_mask, parent)
                {
                    newly_uncovered_slots.push(SlotId(parent));
                }
            }

            let next_pos = TriPeaksPosition {
                tableau_mask: next_mask,
                stock_cursor: position.stock_cursor,
                waste_card: card,
                streak: next_streak,
                score_delta: position.score_delta.saturating_add(score_gained),
                cleared_by_rank,
            };
            let outcome = StepOutcome {
                score_gained,
                cleared_slot: Some(slot),
                new_waste_card: card,
                newly_uncovered_slots,
            };
            Ok((next_pos, outcome))
        }
        TriPeaksMove::Draw => {
            let Some(slot) = stock_slot_id(position.stock_cursor) else {
                return Err(RuleViolation::StockExhausted);
            };
            let card = match slots[usize::from(slot.0)] {
                CardSlot::Known(card) if card.0 < 52 => card,
                CardSlot::Known(card) => {
                    return Err(ModelError::InvalidCardId {
                        slot,
                        card_id: card.0,
                    }
                    .into());
                }
                CardSlot::Unknown => return Err(RuleViolation::SlotUnknown { slot }),
                CardSlot::Unreachable => return Err(RuleViolation::SlotUnreachable { slot }),
            };
            let next_pos = TriPeaksPosition {
                tableau_mask: position.tableau_mask,
                stock_cursor: position.stock_cursor.saturating_add(1),
                waste_card: card,
                streak: 0,
                score_delta: position.score_delta,
                cleared_by_rank: position.cleared_by_rank,
            };
            let outcome = StepOutcome {
                score_gained: 0,
                cleared_slot: None,
                new_waste_card: card,
                newly_uncovered_slots: Vec::new(),
            };
            Ok((next_pos, outcome))
        }
    }
}
