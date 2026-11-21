#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

//! `xsar::search` A*, info-gain probe planner, and best-partial fallback for TriPeaks deals.

use std::{sync::atomic::AtomicBool, time::Duration};

use xsar::search::{
    BestEffortControl, BestEffortOutcome, BestEffortProblem, BestEffortProgress, PartialProof,
    SearchLimits, SearchMethod, SearchOptions, SearchOutcome, SearchProblem, SearchProgress,
    SearchStats, SearchStop, solve_best_effort_controlled, solve_best_effort_with_control,
    solve_controlled, solve_with_control,
};

use crate::recon::{BestPartialRoute, CardSlot, Objective, ProbePlan, ReconError};

use super::{
    DEAL_SLOT_COUNT, FULL_TABLEAU_MASK, INITIAL_WASTE_SLOT, ModelError, PEAK_MASK,
    STOCK_CARD_COUNT, STOCK_START_SLOT, TABLEAU_SLOT_COUNT, TriPeaksMove,
    replay::{ReplayError, ReplayReport, verify_move_sequence, verify_objective_route},
    rules::{
        PEAK_CLEAR_BONUS, are_ranks_adjacent, card_rank_index, is_slot_uncovered,
        slot_to_tableau_coord, tap_score_delta,
    },
    validate_slot_table,
};

/// Default Beast workstation search allowances (`10 GiB` nominal storage, `20M` retained history/expansions, `600 s`).
pub const DEFAULT_SEARCH_LIMITS: SearchLimits = SearchLimits {
    max_states: 20_000_000,
    max_expansions: 20_000_000,
    max_memory_bytes: 10_usize * 1024 * 1024 * 1024,
    max_duration: Duration::from_secs(600),
};

/// Maximum legal transitions from any reachable TriPeaks state (`28` tableau + `1` draw).
const MAX_SUCCESSORS: usize = TABLEAU_SLOT_COUNT + 1;

/// Compact copyable A* search key normalising streak/score outside `ReachScoreDelta`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SearchState {
    /// Occupied tableau bitmask (`bits 0..28`).
    pub tableau_mask: u32,
    /// Cumulative verified point gain when tracking `ReachScoreDelta`, otherwise `0`.
    pub score_progress: u32,
    /// Next Stock draw index (`0..=23`).
    pub stock_cursor: u8,
    /// Zero-based active Waste rank (`0..=12`).
    pub waste_rank: u8,
    /// Active streak count when tracking `ReachScoreDelta`, otherwise `0`.
    pub streak: u8,
}

/// Compact copyable search key used for reachability probes and best-partial fallback.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct ReachState {
    /// Occupied tableau bitmask (`bits 0..28`).
    tableau_mask: u32,
    /// Next Stock draw index (`0..=23`).
    stock_cursor: u8,
    /// Zero-based active Waste rank (`0..=12`).
    waste_rank: u8,
}

/// Fixed-capacity stack iterator yielding successors without per-expansion heap allocation.
struct SuccessorIter<S: Copy> {
    /// Pre-populated successor slots up to `len`.
    items: [Option<(TriPeaksMove, S)>; MAX_SUCCESSORS],
    /// Number of valid items in `items`.
    len: usize,
    /// Next slot index to yield.
    index: usize,
}

impl<S: Copy> SuccessorIter<S> {
    /// Construct an empty fixed-capacity successor buffer.
    const fn empty() -> Self {
        Self {
            items: [None; MAX_SUCCESSORS],
            len: 0,
            index: 0,
        }
    }

    /// Push one legal successor transition within [`MAX_SUCCESSORS`].
    fn push(&mut self, mv: TriPeaksMove, state: S) {
        if let Some(slot) = self.items.get_mut(self.len) {
            *slot = Some((mv, state));
            self.len += 1;
        }
    }
}

impl<S: Copy> Iterator for SuccessorIter<S> {
    type Item = (TriPeaksMove, S);

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.items.get(self.index).copied().flatten()?;
        self.index += 1;
        Some(item)
    }
}

/// Precomputed immutable card ranks and masks extracted from a validated 52-slot deal.
#[derive(Clone, Copy)]
struct KnownDealModel {
    /// Initial Waste rank (`0..=12`).
    initial_waste_rank: u8,
    /// Bitmask of `CardSlot::Known` tableau positions in `0..28`.
    known_tableau_mask: u32,
    /// Rank indices (`0..=12`) for `Known` tableau slots (`0` if non-`Known`).
    known_tableau_ranks: [u8; TABLEAU_SLOT_COUNT],
    /// Optional `Known` rank index for each ordered Stock card `0..23`.
    known_stock_ranks: [Option<u8>; STOCK_CARD_COUNT as usize],
    /// Remaining unseen cards per rank (`0..=4`) derived from the `52`-card single-deck complement.
    unseen_rank_counts: [u8; 13],
}

impl KnownDealModel {
    /// Compile masks and ranks from a validated 52-slot deal, requiring `Known` initial Waste.
    fn from_slots(slots: &[CardSlot]) -> Option<Self> {
        let fixed = validate_slot_table(slots).ok()?;
        let CardSlot::Known(waste_card) = fixed[usize::from(INITIAL_WASTE_SLOT.0)] else {
            return None;
        };
        let mut known_tableau_mask = 0_u32;
        let mut known_tableau_ranks = [0_u8; TABLEAU_SLOT_COUNT];
        for (idx, entry) in fixed[..TABLEAU_SLOT_COUNT].iter().enumerate() {
            if let CardSlot::Known(card) = *entry {
                known_tableau_mask |= 1_u32 << idx;
                known_tableau_ranks[idx] = card_rank_index(card);
            }
        }
        let mut known_stock_ranks = [None; STOCK_CARD_COUNT as usize];
        for (idx, entry) in fixed[usize::from(STOCK_START_SLOT)..DEAL_SLOT_COUNT]
            .iter()
            .enumerate()
        {
            if let CardSlot::Known(card) = *entry {
                known_stock_ranks[idx] = Some(card_rank_index(card));
            }
        }
        let mut unseen_rank_counts = [4_u8; 13];
        let mut seen_cards = 0_u64;
        for entry in fixed {
            if let CardSlot::Known(card) = *entry
                && card.0 < 52
            {
                let bit = 1_u64 << card.0;
                if (seen_cards & bit) == 0 {
                    seen_cards |= bit;
                    let rank_idx = usize::from(card_rank_index(card));
                    unseen_rank_counts[rank_idx] = unseen_rank_counts[rank_idx].saturating_sub(1);
                }
            }
        }
        Some(Self {
            initial_waste_rank: card_rank_index(waste_card),
            known_tableau_mask,
            known_tableau_ranks,
            known_stock_ranks,
            unseen_rank_counts,
        })
    }

    /// Return the exact count (`0..=8`) of unseen cards adjacent (`±1`) to `rank`.
    fn unseen_adjacent_count(&self, rank: u8) -> u8 {
        if rank >= 13 {
            return 0;
        }
        let lower = usize::from((rank + 12) % 13);
        let upper = usize::from((rank + 1) % 13);
        self.unseen_rank_counts[lower].saturating_add(self.unseen_rank_counts[upper])
    }

    /// Return true when at least one currently uncovered `Known` card matches `waste_rank`.
    fn has_adjacent_uncovered_tap(&self, tableau_mask: u32, waste_rank: u8) -> bool {
        let mut candidates = tableau_mask & self.known_tableau_mask;
        while candidates != 0 {
            let idx = candidates.trailing_zeros() as usize;
            candidates &= !(1_u32 << idx);
            if is_slot_uncovered(tableau_mask, idx as u8)
                && are_ranks_adjacent(self.known_tableau_ranks[idx], waste_rank)
            {
                return true;
            }
        }
        false
    }

    /// Generate all legal `(TriPeaksMove, ReachState)` transitions from `state`.
    fn push_reach_successors(&self, state: ReachState, out: &mut SuccessorIter<ReachState>) {
        for idx in 0..TABLEAU_SLOT_COUNT {
            let bit = 1_u32 << idx;
            if (state.tableau_mask & bit) == 0
                || (self.known_tableau_mask & bit) == 0
                || !is_slot_uncovered(state.tableau_mask, idx as u8)
            {
                continue;
            }
            let rank = self.known_tableau_ranks[idx];
            if are_ranks_adjacent(rank, state.waste_rank)
                && let Some((row, column)) = slot_to_tableau_coord(crate::recon::SlotId(idx as u8))
            {
                out.push(
                    TriPeaksMove::TapTableau { row, column },
                    ReachState {
                        tableau_mask: state.tableau_mask & !bit,
                        stock_cursor: state.stock_cursor,
                        waste_rank: rank,
                    },
                );
            }
        }
        if let Some(Some(next_stock_rank)) =
            self.known_stock_ranks.get(usize::from(state.stock_cursor))
        {
            out.push(
                TriPeaksMove::Draw,
                ReachState {
                    tableau_mask: state.tableau_mask,
                    stock_cursor: state.stock_cursor.saturating_add(1),
                    waste_rank: *next_stock_rank,
                },
            );
        }
    }

    /// Generate all legal `(TriPeaksMove, SearchState)` transitions from `state`.
    fn push_search_successors(
        &self,
        state: SearchState,
        track_score: bool,
        out: &mut SuccessorIter<SearchState>,
    ) {
        let mut candidates = state.tableau_mask & self.known_tableau_mask;
        while candidates != 0 {
            let idx = candidates.trailing_zeros() as usize;
            let bit = 1_u32 << idx;
            candidates &= !bit;
            if !is_slot_uncovered(state.tableau_mask, idx as u8) {
                continue;
            }
            let rank = self.known_tableau_ranks[idx];
            if !are_ranks_adjacent(rank, state.waste_rank) {
                continue;
            }
            if let Some((row, column)) = slot_to_tableau_coord(crate::recon::SlotId(idx as u8)) {
                let (next_streak, next_score) = if track_score {
                    let s = state.streak.saturating_add(1);
                    let delta = tap_score_delta(s, row == 1);
                    (s, state.score_progress.saturating_add(delta))
                } else {
                    (0, 0)
                };
                out.push(
                    TriPeaksMove::TapTableau { row, column },
                    SearchState {
                        tableau_mask: state.tableau_mask & !bit,
                        score_progress: next_score,
                        stock_cursor: state.stock_cursor,
                        waste_rank: rank,
                        streak: next_streak,
                    },
                );
            }
        }
        if let Some(Some(stock_rank)) = self.known_stock_ranks.get(usize::from(state.stock_cursor))
        {
            out.push(
                TriPeaksMove::Draw,
                SearchState {
                    tableau_mask: state.tableau_mask,
                    score_progress: state.score_progress,
                    stock_cursor: state.stock_cursor.saturating_add(1),
                    waste_rank: *stock_rank,
                    streak: 0,
                },
            );
        }
    }

    /// Return the bitmask of `Known` tableau slots matching `rank` (`0..13`).
    fn target_rank_mask(&self, rank: u8) -> u32 {
        if rank >= 13 {
            return 0;
        }
        let mut mask = 0_u32;
        for idx in 0..TABLEAU_SLOT_COUNT {
            if (self.known_tableau_mask & (1_u32 << idx)) != 0
                && self.known_tableau_ranks[idx] == rank
            {
                mask |= 1_u32 << idx;
            }
        }
        mask
    }

    /// Soundly prove whether any occupied slot in `tableau_mask` lacks an adjacent partner anywhere remaining.
    fn clear_board_starved(&self, tableau_mask: u32, stock_cursor: u8, waste_rank: u8) -> bool {
        let mut stock_rank_bits = 0_u16;
        for maybe_rank in &self.known_stock_ranks[usize::from(stock_cursor)..] {
            let Some(rank) = maybe_rank else {
                break;
            };
            stock_rank_bits |= 1_u16 << *rank;
        }
        let waste_and_stock = (1_u16 << waste_rank) | stock_rank_bits;

        let mut remaining = tableau_mask;
        while remaining != 0 {
            let idx = remaining.trailing_zeros() as usize;
            let bit = 1_u32 << idx;
            remaining &= !bit;
            let rank = self.known_tableau_ranks[idx];
            let adj_mask = (1_u16 << ((rank + 1) % 13)) | (1_u16 << ((rank + 12) % 13));
            if (waste_and_stock & adj_mask) != 0 {
                continue;
            }
            let mut other = tableau_mask & !bit;
            let mut has_tableau_partner = false;
            while other != 0 {
                let other_idx = other.trailing_zeros() as usize;
                other &= !(1_u32 << other_idx);
                let other_rank = self.known_tableau_ranks[other_idx];
                if (adj_mask & (1_u16 << other_rank)) != 0 {
                    has_tableau_partner = true;
                    break;
                }
            }
            if !has_tableau_partner {
                return true;
            }
        }
        false
    }
}

/// Objective-guided `xsar::search::SearchProblem` over `Known` TriPeaks transitions.
struct TriPeaksObjectiveProblem {
    /// Compiled deal ranks and masks.
    model: KnownDealModel,
    /// Target solving objective.
    objective: Objective,
    /// Bitmask of `Known` tableau slots matching `ClearRankQuota.rank` (`0` otherwise).
    target_rank_mask: u32,
}

impl TriPeaksObjectiveProblem {
    /// Construct an objective problem; builds `target_rank_mask` when `ClearRankQuota` applies.
    fn new(model: KnownDealModel, objective: Objective) -> Self {
        let target_rank_mask = match objective {
            Objective::ClearRankQuota { rank, .. } => model.target_rank_mask(rank),
            _ => 0,
        };
        Self {
            model,
            objective,
            target_rank_mask,
        }
    }

    /// Whether remaining state satisfies `self.objective`.
    fn goal_reached(&self, state: SearchState) -> bool {
        match self.objective {
            Objective::ClearBoard => state.tableau_mask == 0,
            Objective::ClearRankQuota { rank, remaining } => {
                if remaining == 0 {
                    return true;
                }
                if rank >= 13 {
                    return false;
                }
                let cleared = ((!state.tableau_mask) & self.target_rank_mask).count_ones();
                cleared >= u32::from(remaining)
            }
            Objective::ReachScoreDelta { points_needed } => state.score_progress >= points_needed,
        }
    }
}

/// Optimistic point upper bound for `taps` uninterrupted tableau removals starting at `streak`.
const fn max_score_gain_in_taps(start_streak: u32, taps: u32, rem_peaks: u32) -> u32 {
    let peaks_hit = if taps < rem_peaks { taps } else { rem_peaks };
    100 * taps * (2 * start_streak + taps) + PEAK_CLEAR_BONUS * peaks_hit
}

/// Smallest `m in 1..=rem_cards` whose optimistic streak/peak gain meets `deficit > 0`.
fn min_taps_for_deficit(
    start_streak: u32,
    rem_cards: u32,
    rem_peaks: u32,
    deficit: u32,
) -> Option<u32> {
    if deficit == 0 {
        return Some(0);
    }
    (1..=rem_cards).find(|&m| max_score_gain_in_taps(start_streak, m, rem_peaks) >= deficit)
}

impl SearchProblem for TriPeaksObjectiveProblem {
    type State = SearchState;
    type Action = TriPeaksMove;
    type Successors = SuccessorIter<SearchState>;

    fn initial(&self) -> Self::State {
        SearchState {
            tableau_mask: FULL_TABLEAU_MASK,
            score_progress: 0,
            stock_cursor: 0,
            waste_rank: self.model.initial_waste_rank,
            streak: 0,
        }
    }

    fn is_goal(&self, state: Self::State) -> bool {
        self.goal_reached(state)
    }

    fn successors(&self, state: Self::State) -> Self::Successors {
        let mut out = SuccessorIter::empty();
        let track_score = matches!(self.objective, Objective::ReachScoreDelta { .. });
        self.model
            .push_search_successors(state, track_score, &mut out);
        out
    }

    fn heuristic(&self, state: Self::State) -> u32 {
        match self.objective {
            Objective::ClearBoard => {
                let rem = state.tableau_mask.count_ones();
                if rem == 0 {
                    0
                } else if self
                    .model
                    .has_adjacent_uncovered_tap(state.tableau_mask, state.waste_rank)
                {
                    rem
                } else {
                    rem.saturating_add(1)
                }
            }
            Objective::ClearRankQuota { rank, remaining } => {
                let cleared = ((!state.tableau_mask) & self.target_rank_mask).count_ones();
                let still_needed = u32::from(remaining).saturating_sub(cleared);
                if still_needed == 0 {
                    return 0;
                }
                if rank >= 13 {
                    return still_needed;
                }
                let mut target_rem = state.tableau_mask & self.target_rank_mask;
                let direct_target_tap = loop {
                    if target_rem == 0 {
                        break false;
                    }
                    let idx = target_rem.trailing_zeros() as usize;
                    target_rem &= !(1_u32 << idx);
                    if is_slot_uncovered(state.tableau_mask, idx as u8)
                        && are_ranks_adjacent(rank, state.waste_rank)
                    {
                        break true;
                    }
                };
                if direct_target_tap {
                    still_needed
                } else {
                    still_needed.saturating_add(1)
                }
            }
            Objective::ReachScoreDelta { points_needed } => {
                let deficit = points_needed.saturating_sub(state.score_progress);
                if deficit == 0 {
                    return 0;
                }
                let rem_cards = (state.tableau_mask & self.model.known_tableau_mask).count_ones();
                let rem_peaks =
                    (state.tableau_mask & self.model.known_tableau_mask & PEAK_MASK).count_ones();
                let immediate_tap = self
                    .model
                    .has_adjacent_uncovered_tap(state.tableau_mask, state.waste_rank);
                let start_streak = if immediate_tap {
                    u32::from(state.streak)
                } else {
                    0
                };
                let min_taps = min_taps_for_deficit(start_streak, rem_cards, rem_peaks, deficit)
                    .unwrap_or(rem_cards);
                min_taps.saturating_add(u32::from(!immediate_tap))
            }
        }
    }

    fn impossible(&self, state: Self::State) -> bool {
        match self.objective {
            Objective::ClearBoard => {
                (state.tableau_mask & !self.model.known_tableau_mask) != 0
                    || self.model.clear_board_starved(
                        state.tableau_mask,
                        state.stock_cursor,
                        state.waste_rank,
                    )
            }
            Objective::ClearRankQuota { rank, remaining } => {
                let cleared = ((!state.tableau_mask) & self.target_rank_mask).count_ones();
                let still_needed = u32::from(remaining).saturating_sub(cleared);
                if still_needed == 0 {
                    return false;
                }
                if rank >= 13 {
                    return true;
                }
                (state.tableau_mask & self.target_rank_mask).count_ones() < still_needed
            }
            Objective::ReachScoreDelta { points_needed } => {
                let deficit = points_needed.saturating_sub(state.score_progress);
                if deficit == 0 {
                    return false;
                }
                let rem_cards = (state.tableau_mask & self.model.known_tableau_mask).count_ones();
                let rem_peaks =
                    (state.tableau_mask & self.model.known_tableau_mask & PEAK_MASK).count_ones();
                let immediate_tap = self
                    .model
                    .has_adjacent_uncovered_tap(state.tableau_mask, state.waste_rank);
                let start_streak = if immediate_tap {
                    u32::from(state.streak)
                } else {
                    0
                };
                max_score_gain_in_taps(start_streak, rem_cards, rem_peaks) < deficit
            }
        }
    }
}

impl BestEffortProblem for TriPeaksObjectiveProblem {
    fn score(&self, state: Self::State) -> u32 {
        match self.objective {
            Objective::ClearBoard => {
                let cleared = (FULL_TABLEAU_MASK & !state.tableau_mask).count_ones();
                let peaks_cleared = (PEAK_MASK & !state.tableau_mask).count_ones();
                cleared * 1_000 + peaks_cleared * 100
            }
            Objective::ClearRankQuota { .. } => {
                let rank_cleared = ((!state.tableau_mask) & self.target_rank_mask).count_ones();
                if rank_cleared == 0 {
                    return 0;
                }
                let cleared = (FULL_TABLEAU_MASK & !state.tableau_mask).count_ones();
                let peaks_cleared = (PEAK_MASK & !state.tableau_mask).count_ones();
                rank_cleared * 100_000 + cleared * 1_000 + peaks_cleared * 100
            }
            Objective::ReachScoreDelta { .. } => state.score_progress,
        }
    }
}

/// Bitmask of currently covered `CardSlot::Unknown` tableau parents (`0..18`) under `tableau_mask`.
fn covered_unknown_mask_from(
    slots: &[CardSlot; DEAL_SLOT_COUNT],
    initial_tableau_mask: u32,
) -> u32 {
    let mut mask = 0_u32;
    for (idx, entry) in slots.iter().take(18).enumerate() {
        let bit = 1_u32 << idx;
        if (initial_tableau_mask & bit) != 0
            && matches!(entry, CardSlot::Unknown)
            && !is_slot_uncovered(initial_tableau_mask, idx as u8)
        {
            mask |= bit;
        }
    }
    mask
}

/// Bitmask of currently uncovered `Unknown` target parents in `covered_unknown_mask`.
fn exposed_unknown_mask(tableau_mask: u32, covered_unknown_mask: u32) -> u32 {
    let mut exposed = 0_u32;
    let mut pending = covered_unknown_mask & tableau_mask;
    while pending != 0 {
        let idx = pending.trailing_zeros() as usize;
        let bit = 1_u32 << idx;
        pending &= !bit;
        if is_slot_uncovered(tableau_mask, idx as u8) {
            exposed |= bit;
        }
    }
    exposed
}

/// `BestEffortProblem` searching for the move sequence exposing maximum `Unknown` slots.
struct InfoGainProbeProblem {
    /// Compiled deal ranks and masks.
    model: KnownDealModel,
    /// Starting `ReachState` (`T = 0` root or live mid-probe board state).
    initial_state: ReachState,
    /// Bitmask of `Unknown` slots covered at `initial_state`.
    covered_unknown_mask: u32,
}

impl SearchProblem for InfoGainProbeProblem {
    type State = ReachState;
    type Action = TriPeaksMove;
    type Successors = SuccessorIter<ReachState>;

    fn initial(&self) -> Self::State {
        self.initial_state
    }

    fn is_goal(&self, state: Self::State) -> bool {
        self.covered_unknown_mask != 0
            && exposed_unknown_mask(state.tableau_mask, self.covered_unknown_mask)
                == self.covered_unknown_mask
    }

    fn successors(&self, state: Self::State) -> Self::Successors {
        let mut out = SuccessorIter::empty();
        self.model.push_reach_successors(state, &mut out);
        out
    }
}

impl BestEffortProblem for InfoGainProbeProblem {
    fn score(&self, state: Self::State) -> u32 {
        let exposed = exposed_unknown_mask(state.tableau_mask, self.covered_unknown_mask);
        let count = exposed.count_ones();
        if count == 0 {
            return 0;
        }
        let r1 = (exposed & PEAK_MASK).count_ones();
        let r2 = ((exposed >> 3) & 0b11_1111).count_ones();
        let r3 = ((exposed >> 9) & 0x1FF).count_ones();
        let row_bonus = r1 * 4 + r2 * 2 + r3;
        let adj_unseen = u32::from(self.model.unseen_adjacent_count(state.waste_rank));
        let rem_draws = u32::from(STOCK_CARD_COUNT.saturating_sub(state.stock_cursor));
        let cleared = (self.initial_state.tableau_mask & !state.tableau_mask).count_ones();
        count * 100_000 + row_bonus * 1_000 + adj_unseen * 100 + rem_draws * 3 + cleared
    }
}

/// `BestEffortProblem` finding the maximum-cleared-card fallback route for unsolvable deals.
struct BestPartialProblem {
    /// Compiled deal ranks and masks.
    model: KnownDealModel,
}

impl SearchProblem for BestPartialProblem {
    type State = ReachState;
    type Action = TriPeaksMove;
    type Successors = SuccessorIter<ReachState>;

    fn initial(&self) -> Self::State {
        ReachState {
            tableau_mask: FULL_TABLEAU_MASK,
            stock_cursor: 0,
            waste_rank: self.model.initial_waste_rank,
        }
    }

    fn is_goal(&self, state: Self::State) -> bool {
        state.tableau_mask == 0
    }

    fn successors(&self, state: Self::State) -> Self::Successors {
        let mut out = SuccessorIter::empty();
        self.model.push_reach_successors(state, &mut out);
        out
    }
}

impl BestEffortProblem for BestPartialProblem {
    fn score(&self, state: Self::State) -> u32 {
        let cleared = (FULL_TABLEAU_MASK & !state.tableau_mask).count_ones();
        let peaks_cleared = (PEAK_MASK & !state.tableau_mask).count_ones();
        cleared * 1_000 + peaks_cleared * 100
    }
}

/// Solve `slots` for `objective` over `Known` cards using `xsar::search::solve_with_control`.
///
/// Any `Solved` move list is independently verified via [`verify_objective_route`] before return.
pub fn solve_known_subgraph_with_control(
    slots: &[CardSlot],
    objective: Objective,
    limits: SearchLimits,
    options: SearchOptions,
    cancel: &AtomicBool,
    progress: Option<&mut dyn FnMut(SearchProgress)>,
    checkpoint: Option<&dyn Fn() -> Duration>,
) -> SearchOutcome<TriPeaksMove> {
    let Some(model) = KnownDealModel::from_slots(slots) else {
        return SearchOutcome::Unsolvable {
            stats: SearchStats::default(),
        };
    };
    let problem = TriPeaksObjectiveProblem::new(model, objective);
    let outcome = solve_with_control(&problem, limits, cancel, options, progress, checkpoint);
    match outcome {
        SearchOutcome::Solved { actions, stats } => {
            if verify_objective_route(slots, &actions, objective).is_ok() {
                SearchOutcome::Solved { actions, stats }
            } else {
                SearchOutcome::Stopped {
                    reason: SearchStop::InternalInvariant,
                    stats,
                }
            }
        }
        other => other,
    }
}

/// Solve `slots` for `objective` over `Known` cards with default limits and weight-1 A*.
pub fn solve_known_subgraph(slots: &[CardSlot], objective: Objective) -> Option<Vec<TriPeaksMove>> {
    let cancel = AtomicBool::new(false);
    match solve_known_subgraph_with_control(
        slots,
        objective,
        DEFAULT_SEARCH_LIMITS,
        SearchOptions::default(),
        &cancel,
        None,
        None,
    ) {
        SearchOutcome::Solved { actions, .. } => Some(actions.into_vec()),
        SearchOutcome::Unsolvable { .. } | SearchOutcome::Stopped { .. } => None,
    }
}

/// Borrowed context for continuing info-gain probing from a live mid-board prefix.
pub(crate) struct ProbeContinuationInput<'a> {
    /// Current `52`-slot deal table combining known cards and unrevealed slots.
    pub slots: &'a [CardSlot],
    /// Legal physical moves already executed from `T = 0` to reach the live board state.
    pub executed_prefix: &'a [TriPeaksMove],
    /// Search limits governing probe planning.
    pub limits: SearchLimits,
    /// Cooperative cancellation flag checked during graph search.
    pub cancel: &'a AtomicBool,
}

/// Shared base planner backing both `T = 0` probe planning and mid-board continuation.
fn plan_info_gain_from_prefix(
    input: ProbeContinuationInput<'_>,
    progress: Option<&mut dyn FnMut(SearchProgress)>,
    checkpoint: Option<&dyn Fn() -> Duration>,
) -> Result<ProbePlan<TriPeaksMove>, ReconError> {
    let fixed = validate_slot_table(input.slots).map_err(|err| ReconError::SearchFailed {
        context: "probe planning",
        detail: err.to_string(),
    })?;
    let model = KnownDealModel::from_slots(fixed).ok_or_else(|| ReconError::SearchFailed {
        context: "probe planning",
        detail: ModelError::UnknownInitialWaste.to_string(),
    })?;
    let prefix_report = verify_move_sequence(fixed, input.executed_prefix).map_err(|err| {
        ReconError::SearchFailed {
            context: "probe planning",
            detail: err.to_string(),
        }
    })?;
    let initial_state = ReachState {
        tableau_mask: prefix_report.final_position.tableau_mask,
        stock_cursor: prefix_report.final_position.stock_cursor,
        waste_rank: card_rank_index(prefix_report.final_position.waste_card),
    };
    let covered_unknown_mask = covered_unknown_mask_from(fixed, initial_state.tableau_mask);
    if covered_unknown_mask == 0 {
        return Ok(ProbePlan::Exhausted);
    }
    let problem = InfoGainProbeProblem {
        model,
        initial_state,
        covered_unknown_mask,
    };
    let options = SearchOptions {
        method: SearchMethod::Dijkstra,
        heuristic_weight: SearchOptions::default().heuristic_weight,
    };
    let actions = match solve_best_effort_with_control(
        &problem,
        input.limits,
        input.cancel,
        options,
        progress,
        checkpoint,
    ) {
        BestEffortOutcome::Solved { actions, .. } => actions,
        BestEffortOutcome::Partial { actions, score, .. } if score > 0 => actions,
        BestEffortOutcome::Partial {
            proof: PartialProof::Exhaustive,
            ..
        } => return Ok(ProbePlan::Exhausted),
        BestEffortOutcome::Partial {
            proof: PartialProof::Bounded(stop),
            ..
        } => return Ok(ProbePlan::Bounded(stop)),
        BestEffortOutcome::Stopped {
            reason: SearchStop::Cancelled,
            ..
        } => return Err(ReconError::Cancelled),
        BestEffortOutcome::Stopped {
            reason: reason @ (SearchStop::AllocationFailure | SearchStop::InternalInvariant),
            ..
        } => {
            return Err(ReconError::SearchFailed {
                context: "probe planning",
                detail: reason.to_string(),
            });
        }
        BestEffortOutcome::Stopped {
            reason:
                reason @ (SearchStop::StateLimit
                | SearchStop::ExpansionLimit
                | SearchStop::MemoryLimit
                | SearchStop::TimeLimit
                | SearchStop::Interrupted),
            ..
        } => return Ok(ProbePlan::Bounded(reason)),
    };
    if actions.is_empty() {
        return Err(ReconError::SearchFailed {
            context: "probe planning",
            detail: "info-gain solver returned an empty probe with positive score".to_owned(),
        });
    }
    let mut combined =
        Vec::with_capacity(input.executed_prefix.len().saturating_add(actions.len()));
    combined.extend_from_slice(input.executed_prefix);
    combined.extend_from_slice(&actions);
    let report =
        verify_move_sequence(fixed, &combined).map_err(|err| ReconError::SearchFailed {
            context: "probe planning",
            detail: err.to_string(),
        })?;
    if report.newly_uncovered_unknown_slots.len()
        <= prefix_report.newly_uncovered_unknown_slots.len()
    {
        return Err(ReconError::SearchFailed {
            context: "probe planning",
            detail: "planned probe did not uncover any Unknown slot under replay".to_owned(),
        });
    }
    Ok(ProbePlan::Found(actions))
}

/// Plan an information-gain legal move sequence exposing `>= 1` [`CardSlot::Unknown`] parent,
/// distinguishing exhaustive impossibility from bounded interruption and fatal faults.
///
/// # Errors
///
/// Returns [`ReconError::Cancelled`] on operator cancellation or [`ReconError::SearchFailed`]
/// if `slots` fail validation, lack a `Known` initial Waste, or hit allocation/invariant faults.
pub fn plan_info_gain_probe_with_control(
    slots: &[CardSlot],
    limits: SearchLimits,
    cancel: &AtomicBool,
    progress: Option<&mut dyn FnMut(SearchProgress)>,
    checkpoint: Option<&dyn Fn() -> Duration>,
) -> Result<ProbePlan<TriPeaksMove>, ReconError> {
    plan_info_gain_from_prefix(
        ProbeContinuationInput {
            slots,
            executed_prefix: &[],
            limits,
            cancel,
        },
        progress,
        checkpoint,
    )
}

/// Plan an online continuation probe from the live state reached after `executed_prefix`.
///
/// # Errors
///
/// Propagates [`ReconError`] identically to [`plan_info_gain_probe_with_control`].
pub(crate) fn plan_info_gain_continuation(
    input: ProbeContinuationInput<'_>,
) -> Result<ProbePlan<TriPeaksMove>, ReconError> {
    plan_info_gain_from_prefix(input, None, None)
}

/// Convenience wrapper for [`plan_info_gain_probe_with_control`] using [`DEFAULT_SEARCH_LIMITS`].
///
/// # Errors
///
/// Propagates [`ReconError`] from [`plan_info_gain_probe_with_control`].
pub fn plan_info_gain_probe(slots: &[CardSlot]) -> Result<ProbePlan<TriPeaksMove>, ReconError> {
    let cancel = AtomicBool::new(false);
    plan_info_gain_probe_with_control(slots, DEFAULT_SEARCH_LIMITS, &cancel, None, None)
}

/// Compute the maximum-tableau-removal partial route over `Known` cards using `xsar::search`.
///
/// # Errors
///
/// Returns [`ReconError::Cancelled`] on operator cancellation or [`ReconError::SearchFailed`]
/// if `slots` fail validation, lack a `Known` initial Waste, or hit allocation/invariant faults.
pub fn best_partial_route_with_control(
    slots: &[CardSlot],
    limits: SearchLimits,
    cancel: &AtomicBool,
    progress: Option<&mut dyn FnMut(SearchProgress)>,
    checkpoint: Option<&dyn Fn() -> Duration>,
) -> Result<BestPartialRoute<TriPeaksMove>, ReconError> {
    let fixed = validate_slot_table(slots).map_err(|err| ReconError::SearchFailed {
        context: "partial fallback",
        detail: err.to_string(),
    })?;
    let has_unknown_slots = fixed.iter().any(|slot| matches!(slot, CardSlot::Unknown));
    let model = KnownDealModel::from_slots(fixed).ok_or_else(|| ReconError::SearchFailed {
        context: "partial fallback",
        detail: ModelError::UnknownInitialWaste.to_string(),
    })?;
    let problem = BestPartialProblem { model };
    let options = SearchOptions {
        method: SearchMethod::Dijkstra,
        heuristic_weight: SearchOptions::default().heuristic_weight,
    };
    let (actions, proof) = match solve_best_effort_with_control(
        &problem, limits, cancel, options, progress, checkpoint,
    ) {
        BestEffortOutcome::Solved { actions, .. } => (actions, PartialProof::Exhaustive),
        BestEffortOutcome::Partial { actions, proof, .. } => (actions, proof),
        BestEffortOutcome::Stopped {
            reason: SearchStop::Cancelled,
            ..
        } => return Err(ReconError::Cancelled),
        BestEffortOutcome::Stopped {
            reason: reason @ (SearchStop::AllocationFailure | SearchStop::InternalInvariant),
            ..
        } => {
            return Err(ReconError::SearchFailed {
                context: "partial fallback",
                detail: reason.to_string(),
            });
        }
        BestEffortOutcome::Stopped {
            reason:
                reason @ (SearchStop::StateLimit
                | SearchStop::ExpansionLimit
                | SearchStop::MemoryLimit
                | SearchStop::TimeLimit
                | SearchStop::Interrupted),
            ..
        } => (Vec::new(), PartialProof::Bounded(reason)),
    };
    verify_move_sequence(fixed, &actions).map_err(|err| ReconError::SearchFailed {
        context: "partial fallback",
        detail: err.to_string(),
    })?;
    Ok(BestPartialRoute {
        moves: actions,
        proof,
        has_unknown_slots,
    })
}

/// Convenience wrapper for [`best_partial_route_with_control`] using [`DEFAULT_SEARCH_LIMITS`].
///
/// # Errors
///
/// Propagates [`ReconError`] from [`best_partial_route_with_control`].
pub fn best_partial_route(
    slots: &[CardSlot],
) -> Result<BestPartialRoute<TriPeaksMove>, ReconError> {
    let cancel = AtomicBool::new(false);
    best_partial_route_with_control(slots, DEFAULT_SEARCH_LIMITS, &cancel, None, None)
}

/// Complete, escalated, partial, or stopped planning result produced by [`plan_deal_route_with_control`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TriPeaksPlanOutcome {
    /// Route satisfies the caller's original objective directly on this deal.
    SolvedObjective {
        /// Replay-verified TriPeaks move list starting from `T = 0`.
        actions: Vec<TriPeaksMove>,
        /// Verified final state and cleared rank/score tallies.
        replay: ReplayReport,
        /// Aggregate bounded search counters across passes.
        stats: SearchStats,
    },
    /// Multi-board Challenge escalation: direct quota/score exceeded a single deck, so a full
    /// `ClearBoard` clear was planned to earn a free next deal (maximising score on `ReachScoreDelta`).
    EscalatedClearBoard {
        /// Replay-verified `28`-card full-board clear starting from `T = 0`.
        actions: Vec<TriPeaksMove>,
        /// Verified final state (guaranteed `tableau_mask == 0`).
        replay: ReplayReport,
        /// Aggregate bounded search counters across passes.
        stats: SearchStats,
    },
    /// Direct objective and `ClearBoard` escalation could not be solved on the known cards;
    /// carries the best partial progress, proof status (`Exhaustive` vs `Bounded`), and remaining `Unknown` slot count.
    UnsolvablePartial {
        /// Replay-verified best objective-progress route (empty when `0` progress is reachable).
        actions: Vec<TriPeaksMove>,
        /// Verified final state of `actions`.
        replay: ReplayReport,
        /// Best number of tableau cards cleared on the known deal (`0..=28`; proven max iff `proof == Exhaustive && unknown_slots == 0`).
        max_tableau_cleared: u8,
        /// Whether partial-progress search exhausted the known subgraph or stopped at a bound.
        proof: PartialProof,
        /// Number of slots in the evaluated table still marked [`CardSlot::Unknown`].
        unknown_slots: usize,
        /// Aggregate bounded search counters across passes.
        stats: SearchStats,
    },
    /// Search stopped cleanly under operator cancellation or bounded resource limits.
    Stopped {
        /// Bounded search stop cause (`Cancelled`, `ExpansionLimit`, etc.).
        reason: SearchStop,
        /// Aggregate search counters completed prior to stopping.
        stats: SearchStats,
    },
}

/// A* `SearchProblem` seeking a full `28`-card clear (`tableau_mask == 0`) with `score >= min_score`.
struct ClearBoardMinScoreProblem {
    /// Compiled deal ranks and masks.
    model: KnownDealModel,
    /// Minimum required verified score delta along the full-board clear.
    min_score: u32,
}

impl ClearBoardMinScoreProblem {
    /// Compute remaining optimistic taps from `state` (`(rem_cards, rem_peaks, start_streak, immediate_tap)`).
    fn optimistic_bounds(&self, state: SearchState) -> (u32, u32, u32, bool) {
        let rem_cards = state.tableau_mask.count_ones();
        let rem_peaks = (state.tableau_mask & PEAK_MASK).count_ones();
        let immediate_tap = self
            .model
            .has_adjacent_uncovered_tap(state.tableau_mask, state.waste_rank);
        let start_streak = if immediate_tap {
            u32::from(state.streak)
        } else {
            0
        };
        (rem_cards, rem_peaks, start_streak, immediate_tap)
    }
}

impl SearchProblem for ClearBoardMinScoreProblem {
    type State = SearchState;
    type Action = TriPeaksMove;
    type Successors = SuccessorIter<SearchState>;

    fn initial(&self) -> Self::State {
        SearchState {
            tableau_mask: FULL_TABLEAU_MASK,
            score_progress: 0,
            stock_cursor: 0,
            waste_rank: self.model.initial_waste_rank,
            streak: 0,
        }
    }

    fn is_goal(&self, state: Self::State) -> bool {
        state.tableau_mask == 0 && state.score_progress >= self.min_score
    }

    fn successors(&self, state: Self::State) -> Self::Successors {
        let mut out = SuccessorIter::empty();
        if state.tableau_mask != 0 {
            self.model.push_search_successors(state, true, &mut out);
        }
        out
    }

    fn heuristic(&self, state: Self::State) -> u32 {
        let (rem_cards, rem_peaks, start_streak, immediate_tap) = self.optimistic_bounds(state);
        if rem_cards == 0 {
            return 0;
        }
        let deficit = self.min_score.saturating_sub(state.score_progress);
        let min_score_taps =
            min_taps_for_deficit(start_streak, rem_cards, rem_peaks, deficit).unwrap_or(rem_cards);
        rem_cards
            .max(min_score_taps)
            .saturating_add(u32::from(!immediate_tap))
    }

    fn impossible(&self, state: Self::State) -> bool {
        if (state.tableau_mask & !self.model.known_tableau_mask) != 0
            || self.model.clear_board_starved(
                state.tableau_mask,
                state.stock_cursor,
                state.waste_rank,
            )
        {
            return true;
        }
        let deficit = self.min_score.saturating_sub(state.score_progress);
        if deficit == 0 {
            return false;
        }
        let (rem_cards, rem_peaks, start_streak, _) = self.optimistic_bounds(state);
        max_score_gain_in_taps(start_streak, rem_cards, rem_peaks) < deficit
    }
}

impl BestEffortProblem for ClearBoardMinScoreProblem {
    fn score(&self, state: Self::State) -> u32 {
        state.score_progress
    }
}

/// `BestEffortProblem` maximising target-rank removals (with total cards/peaks as tie-breakers).
struct MaxRankPartialProblem {
    /// Compiled deal ranks and masks.
    model: KnownDealModel,
    /// Bitmask of `Known` tableau slots matching the Challenge's target rank.
    target_rank_mask: u32,
}

impl SearchProblem for MaxRankPartialProblem {
    type State = ReachState;
    type Action = TriPeaksMove;
    type Successors = SuccessorIter<ReachState>;

    fn initial(&self) -> Self::State {
        ReachState {
            tableau_mask: FULL_TABLEAU_MASK,
            stock_cursor: 0,
            waste_rank: self.model.initial_waste_rank,
        }
    }

    fn is_goal(&self, state: Self::State) -> bool {
        (state.tableau_mask & self.target_rank_mask) == 0 && self.target_rank_mask != 0
    }

    fn successors(&self, state: Self::State) -> Self::Successors {
        let mut out = SuccessorIter::empty();
        self.model.push_reach_successors(state, &mut out);
        out
    }
}

impl BestEffortProblem for MaxRankPartialProblem {
    fn score(&self, state: Self::State) -> u32 {
        let rank_cleared = ((!state.tableau_mask) & self.target_rank_mask).count_ones();
        if rank_cleared == 0 {
            return 0;
        }
        let cleared = (FULL_TABLEAU_MASK & !state.tableau_mask).count_ones();
        let peaks_cleared = (PEAK_MASK & !state.tableau_mask).count_ones();
        rank_cleared * 100_000 + cleared * 1_000 + peaks_cleared * 100
    }
}

/// `BestEffortProblem` maximising verified single-board score on an unsolvable deal.
struct MaxScorePartialProblem {
    /// Compiled deal ranks and masks.
    model: KnownDealModel,
}

impl SearchProblem for MaxScorePartialProblem {
    type State = SearchState;
    type Action = TriPeaksMove;
    type Successors = SuccessorIter<SearchState>;

    fn initial(&self) -> Self::State {
        SearchState {
            tableau_mask: FULL_TABLEAU_MASK,
            score_progress: 0,
            stock_cursor: 0,
            waste_rank: self.model.initial_waste_rank,
            streak: 0,
        }
    }

    fn is_goal(&self, _state: Self::State) -> bool {
        false
    }

    fn successors(&self, state: Self::State) -> Self::Successors {
        let mut out = SuccessorIter::empty();
        self.model.push_search_successors(state, true, &mut out);
        out
    }
}

impl BestEffortProblem for MaxScorePartialProblem {
    fn score(&self, state: Self::State) -> u32 {
        state.score_progress
    }
}

/// Observer channel supporting legacy progress and controlled best-score progress.
enum PlanProgress<'b> {
    /// Legacy progress callback without live best score.
    Search(&'b mut dyn FnMut(SearchProgress)),
    /// Controlled best-effort callback carrying live best score.
    BestEffort(&'b mut dyn FnMut(BestEffortProgress)),
}

/// Single-phase objective search outcome supporting cooperative `Play` best-prefix yield.
enum ObjectivePassResult {
    /// Verified route satisfying the target objective.
    Solved(Vec<TriPeaksMove>),
    /// Operator soft-yield interrupted before completion with a candidate prefix.
    PartialYield {
        /// Ordered moves from root (`empty` when no positive score existed).
        actions: Vec<TriPeaksMove>,
        /// Problem score reached at `actions`.
        score: u32,
    },
    /// Frontier depleted without solving the target objective.
    Exhausted,
    /// Search stopped due to resource limit, cancellation, or invariant error.
    Stopped(SearchStop),
}

/// Comparable progress key ranking verified partial/complete candidates for a target [`Objective`].
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct CandidateRank {
    /// Target objective satisfied directly (`SolvedObjective`).
    solved_objective: bool,
    /// Full `28`-card board cleared (`EscalatedClearBoard` for rank/score challenges).
    cleared_board: bool,
    /// Primary partial objective metric (`rank_cleared` for `ClearRankQuota`,
    /// `score_delta` for `ReachScoreDelta`, `0` for `ClearBoard`).
    primary_metric: u32,
    /// Total tableau cards cleared (`0..=28`).
    cards_cleared: u32,
    /// Row-1 peak cards cleared (`0..=3`).
    peaks_cleared: u32,
    /// Verified point delta along the route.
    score_delta: u32,
    /// Shorter routes win all progress ties.
    reverse_steps: std::cmp::Reverse<usize>,
}

/// Independently replay `actions` and compute its [`CandidateRank`] for `objective`.
///
/// Returns `None` if `actions` is empty, fails replay, or achieves zero positive progress.
fn evaluate_candidate(
    fixed: &[CardSlot; DEAL_SLOT_COUNT],
    objective: Objective,
    actions: &[TriPeaksMove],
) -> Option<(ReplayReport, CandidateRank)> {
    if actions.is_empty() {
        return None;
    }
    let replay = verify_move_sequence(fixed, actions).ok()?;
    let solved_objective = verify_objective_route(fixed, actions, objective).is_ok();
    let cleared_board = replay.final_position.tableau_mask == 0;
    let cards_cleared = (FULL_TABLEAU_MASK & !replay.final_position.tableau_mask).count_ones();
    let peaks_cleared = (PEAK_MASK & !replay.final_position.tableau_mask).count_ones();
    let score_delta = replay.final_position.score_delta;
    if !solved_objective && !cleared_board && cards_cleared == 0 && score_delta == 0 {
        return None;
    }
    let primary_metric = match objective {
        Objective::ClearBoard => 0,
        Objective::ClearRankQuota { rank, .. } => u32::from(
            replay
                .final_position
                .cleared_by_rank
                .get(usize::from(rank))
                .copied()
                .unwrap_or(0),
        ),
        Objective::ReachScoreDelta { .. } => score_delta,
    };
    let rank = CandidateRank {
        solved_objective,
        cleared_board,
        primary_metric,
        cards_cleared,
        peaks_cleared,
        score_delta,
        reverse_steps: std::cmp::Reverse(actions.len()),
    };
    Some((replay, rank))
}

/// Shared borrowed control context passed across planning passes.
struct PlanSearchEnv<'a, 'b> {
    /// Fixed-size validated `52`-slot deal table for candidate replay scoring.
    fixed: &'a [CardSlot; DEAL_SLOT_COUNT],
    /// Compiled known-card deal model.
    model: KnownDealModel,
    /// Target caller objective used to rank candidates across passes.
    objective: Objective,
    /// Resource bounds per search pass.
    limits: SearchLimits,
    /// Shared cancellation flag.
    cancel: &'a AtomicBool,
    /// Optional soft-stop flag halting early to return the best scored route so far.
    yield_best: Option<&'a AtomicBool>,
    /// Optional cooperative search progress observer.
    progress: Option<PlanProgress<'b>>,
    /// Optional cooperative Pause checkpoint callback.
    checkpoint: Option<&'a dyn Fn() -> Duration>,
    /// Cumulative search work across sub-passes.
    acc_stats: SearchStats,
    /// Monotonic best score reported while a valid candidate route is available.
    reported_best_score: u32,
    /// Highest-ranking verified move sequence preserved across pass transitions.
    best_candidate: Option<(Vec<TriPeaksMove>, CandidateRank)>,
}

impl PlanSearchEnv<'_, '_> {
    /// Accumulate counters from one completed sub-search.
    fn record_stats(&mut self, next: SearchStats) {
        self.acc_stats.expanded = self.acc_stats.expanded.saturating_add(next.expanded);
        self.acc_stats.generated = self.acc_stats.generated.saturating_add(next.generated);
        self.acc_stats.states = self.acc_stats.states.max(next.states);
        self.acc_stats.peak_frontier = self.acc_stats.peak_frontier.max(next.peak_frontier);
        self.acc_stats.elapsed = self.acc_stats.elapsed.saturating_add(next.elapsed);
    }

    /// Retain the highest-ranking non-empty verified candidate for `self.objective` across passes.
    fn record_best_candidate(&mut self, actions: Vec<TriPeaksMove>, score: u32) {
        if score == 0 || actions.is_empty() {
            return;
        }
        let Some((_, rank)) = evaluate_candidate(self.fixed, self.objective, &actions) else {
            return;
        };
        if self
            .best_candidate
            .as_ref()
            .is_none_or(|(_, existing)| rank > *existing)
        {
            self.reported_best_score = self.reported_best_score.max(score);
            self.best_candidate = Some((actions, rank));
        }
    }

    /// Ensure an uncommitted prior-pass score does not keep `Play` enabled if no candidate route was preserved.
    fn sync_score_for_next_pass(&mut self) {
        if self.best_candidate.is_none() && self.reported_best_score > 0 {
            self.reported_best_score = 0;
            if let Some(PlanProgress::BestEffort(cb)) = self.progress.as_mut() {
                cb(BestEffortProgress {
                    search: SearchProgress {
                        stats: self.acc_stats,
                        finished: false,
                    },
                    best_score: 0,
                });
            }
        }
    }

    /// Forward progress monotonically so sub-pass boundaries never drop a valid live best score.
    fn forward_progress(
        cb: &mut dyn FnMut(BestEffortProgress),
        reported_best_score: &mut u32,
        p: BestEffortProgress,
    ) {
        *reported_best_score = (*reported_best_score).max(p.best_score);
        cb(BestEffortProgress {
            search: p.search,
            best_score: *reported_best_score,
        });
    }

    /// Run one `BestEffortProblem` pass cleanly, preserving partial proof and propagating `Stopped`.
    fn run_best_effort<P: BestEffortProblem<Action = TriPeaksMove>>(
        &mut self,
        problem: &P,
        options: SearchOptions,
    ) -> Result<(Vec<TriPeaksMove>, PartialProof), SearchStop> {
        let cancel = self.cancel;
        let yield_best = self.yield_best;
        let checkpoint = self.checkpoint;
        let limits = self.limits;
        let reported_best = &mut self.reported_best_score;
        let outcome = match self.progress.as_mut() {
            Some(PlanProgress::BestEffort(cb)) => {
                let mut forward = |p: BestEffortProgress| {
                    Self::forward_progress(*cb, reported_best, p);
                };
                solve_best_effort_controlled(
                    problem,
                    limits,
                    options,
                    BestEffortControl {
                        cancel,
                        yield_best,
                        progress: Some(&mut forward),
                        checkpoint,
                    },
                )
            }
            Some(PlanProgress::Search(cb)) => {
                let mut forward = |p: BestEffortProgress| cb(p.search);
                solve_best_effort_controlled(
                    problem,
                    limits,
                    options,
                    BestEffortControl {
                        cancel,
                        yield_best,
                        progress: Some(&mut forward),
                        checkpoint,
                    },
                )
            }
            None => solve_best_effort_controlled(
                problem,
                limits,
                options,
                BestEffortControl {
                    cancel,
                    yield_best,
                    progress: None,
                    checkpoint,
                },
            ),
        };
        match outcome {
            BestEffortOutcome::Solved { actions, stats, .. } => {
                self.record_stats(stats);
                self.record_best_candidate(actions.clone(), 1);
                Ok((actions, PartialProof::Exhaustive))
            }
            BestEffortOutcome::Partial {
                actions,
                score,
                proof,
                stats,
            } => {
                self.record_stats(stats);
                self.record_best_candidate(actions.clone(), score);
                Ok((actions, proof))
            }
            BestEffortOutcome::Stopped { reason, stats } => {
                self.record_stats(stats);
                Err(reason)
            }
        }
    }

    /// Run one bounded score-upgrade probe supporting cooperative `Play` without zeroing live best score.
    fn run_clear_score_probe(
        &mut self,
        problem: &ClearBoardMinScoreProblem,
        limits: SearchLimits,
    ) -> Result<Option<Vec<TriPeaksMove>>, SearchStop> {
        let cancel = self.cancel;
        let yield_best = self.yield_best;
        let checkpoint = self.checkpoint;
        let reported_best = &mut self.reported_best_score;
        let outcome = match self.progress.as_mut() {
            Some(PlanProgress::BestEffort(cb)) => {
                let mut forward = |p: BestEffortProgress| {
                    Self::forward_progress(*cb, reported_best, p);
                };
                solve_controlled(
                    problem,
                    limits,
                    SearchOptions::default(),
                    BestEffortControl {
                        cancel,
                        yield_best,
                        progress: Some(&mut forward),
                        checkpoint,
                    },
                )
            }
            Some(PlanProgress::Search(cb)) => {
                let mut forward = |p: BestEffortProgress| cb(p.search);
                solve_controlled(
                    problem,
                    limits,
                    SearchOptions::default(),
                    BestEffortControl {
                        cancel,
                        yield_best,
                        progress: Some(&mut forward),
                        checkpoint,
                    },
                )
            }
            None => solve_controlled(
                problem,
                limits,
                SearchOptions::default(),
                BestEffortControl {
                    cancel,
                    yield_best,
                    progress: None,
                    checkpoint,
                },
            ),
        };
        match outcome {
            BestEffortOutcome::Solved { actions, stats, .. } => {
                self.record_stats(stats);
                Ok(Some(actions))
            }
            BestEffortOutcome::Partial {
                proof: PartialProof::Bounded(SearchStop::Interrupted),
                stats,
                ..
            } => {
                self.record_stats(stats);
                Err(SearchStop::Interrupted)
            }
            BestEffortOutcome::Partial { stats, .. } => {
                self.record_stats(stats);
                Ok(None)
            }
            BestEffortOutcome::Stopped {
                reason:
                    reason @ (SearchStop::Cancelled
                    | SearchStop::AllocationFailure
                    | SearchStop::InternalInvariant),
                stats,
            } => {
                self.record_stats(stats);
                Err(reason)
            }
            BestEffortOutcome::Stopped { stats, .. } => {
                self.record_stats(stats);
                Ok(None)
            }
        }
    }

    /// Run one single-phase objective A* pass supporting cooperative `Play` best-prefix yield.
    fn run_objective(&mut self, objective: Objective) -> ObjectivePassResult {
        let problem = TriPeaksObjectiveProblem::new(self.model, objective);
        let cancel = self.cancel;
        let yield_best = self.yield_best;
        let checkpoint = self.checkpoint;
        let limits = self.limits;
        let reported_best = &mut self.reported_best_score;
        let outcome = match self.progress.as_mut() {
            Some(PlanProgress::BestEffort(cb)) => {
                let mut forward = |p: BestEffortProgress| {
                    Self::forward_progress(*cb, reported_best, p);
                };
                solve_controlled(
                    &problem,
                    limits,
                    SearchOptions::default(),
                    BestEffortControl {
                        cancel,
                        yield_best,
                        progress: Some(&mut forward),
                        checkpoint,
                    },
                )
            }
            Some(PlanProgress::Search(cb)) => {
                let mut forward = |p: BestEffortProgress| cb(p.search);
                solve_controlled(
                    &problem,
                    limits,
                    SearchOptions::default(),
                    BestEffortControl {
                        cancel,
                        yield_best,
                        progress: Some(&mut forward),
                        checkpoint,
                    },
                )
            }
            None => solve_controlled(
                &problem,
                limits,
                SearchOptions::default(),
                BestEffortControl {
                    cancel,
                    yield_best,
                    progress: None,
                    checkpoint,
                },
            ),
        };
        match outcome {
            BestEffortOutcome::Solved { actions, stats, .. } => {
                self.record_stats(stats);
                if verify_objective_route(self.fixed, &actions, objective).is_ok() {
                    ObjectivePassResult::Solved(actions)
                } else {
                    ObjectivePassResult::Stopped(SearchStop::InternalInvariant)
                }
            }
            BestEffortOutcome::Partial {
                actions,
                score,
                proof: PartialProof::Exhaustive,
                stats,
            } => {
                self.record_stats(stats);
                self.record_best_candidate(actions, score);
                ObjectivePassResult::Exhausted
            }
            BestEffortOutcome::Partial {
                actions,
                score,
                proof: PartialProof::Bounded(SearchStop::Interrupted),
                stats,
            } => {
                self.record_stats(stats);
                ObjectivePassResult::PartialYield { actions, score }
            }
            BestEffortOutcome::Partial { stats, .. } => {
                self.record_stats(stats);
                ObjectivePassResult::Stopped(SearchStop::InternalInvariant)
            }
            BestEffortOutcome::Stopped { reason, stats } => {
                self.record_stats(stats);
                ObjectivePassResult::Stopped(reason)
            }
        }
    }
}

/// Upgrade an escalated full-board clear for `ReachScoreDelta` via binary search over `100`-pt
/// thresholds up to the `28`-card theoretical ceiling, preserving `fast_actions` if capped or played.
fn maybe_upgrade_clear_score(
    env: &mut PlanSearchEnv<'_, '_>,
    fast_actions: Vec<TriPeaksMove>,
    fast_replay: ReplayReport,
) -> Result<(Vec<TriPeaksMove>, ReplayReport), SearchStop> {
    let mut best_actions = fast_actions;
    let mut best_replay = fast_replay;
    env.reported_best_score = env
        .reported_best_score
        .max(best_replay.final_position.score_delta.max(1));
    let max_theoretical = max_score_gain_in_taps(0, TABLEAU_SLOT_COUNT as u32, 3);
    let mut lo_hundreds = best_replay.final_position.score_delta / 100 + 1;
    let mut hi_hundreds = max_theoretical / 100;
    if lo_hundreds > hi_hundreds {
        return Ok((best_actions, best_replay));
    }

    // Bound each score-maximising binary-search probe proportionally to workstation budgets.
    let probe_limits = SearchLimits {
        max_states: env.limits.max_states.min(2_000_000),
        max_expansions: env.limits.max_expansions.min(2_000_000),
        max_memory_bytes: env.limits.max_memory_bytes,
        max_duration: env.limits.max_duration.min(Duration::from_secs(5)),
    };

    while lo_hundreds <= hi_hundreds {
        if env.cancel.load(std::sync::atomic::Ordering::Acquire) {
            return Err(SearchStop::Cancelled);
        }
        if env
            .yield_best
            .is_some_and(|flag| flag.load(std::sync::atomic::Ordering::Acquire))
        {
            break;
        }
        let mid_hundreds = lo_hundreds + (hi_hundreds - lo_hundreds) / 2;
        let problem = ClearBoardMinScoreProblem {
            model: env.model,
            min_score: mid_hundreds * 100,
        };
        match env.run_clear_score_probe(&problem, probe_limits) {
            Ok(Some(candidate)) => {
                if let Ok(candidate_replay) =
                    verify_objective_route(env.fixed, &candidate, Objective::ClearBoard)
                {
                    let candidate_score = candidate_replay.final_position.score_delta;
                    if candidate_score > best_replay.final_position.score_delta
                        || (candidate_score == best_replay.final_position.score_delta
                            && candidate_replay.steps_executed < best_replay.steps_executed)
                    {
                        best_actions = candidate;
                        best_replay = candidate_replay;
                        env.reported_best_score = env
                            .reported_best_score
                            .max(best_replay.final_position.score_delta.max(1));
                    }
                    lo_hundreds = (candidate_score / 100)
                        .saturating_add(1)
                        .max(mid_hundreds + 1);
                } else {
                    hi_hundreds = mid_hundreds - 1;
                }
            }
            Ok(None) => {
                hi_hundreds = mid_hundreds - 1;
            }
            Err(SearchStop::Interrupted) => break,
            Err(reason) => return Err(reason),
        }
    }

    Ok((best_actions, best_replay))
}

/// Plan the optimal direct objective route, multi-board `ClearBoard` escalation, or verified
/// best-partial fallback route over `slots`, verifying every returned move list via replay.
///
/// # Errors
///
/// Returns [`ReplayError`] if `slots` fails validation or lacks a `Known` initial Waste slot.
pub fn plan_deal_route_with_control(
    slots: &[CardSlot],
    objective: Objective,
    limits: SearchLimits,
    cancel: &AtomicBool,
    progress: Option<&mut dyn FnMut(SearchProgress)>,
    checkpoint: Option<&dyn Fn() -> Duration>,
) -> Result<TriPeaksPlanOutcome, ReplayError> {
    plan_deal_route_inner(
        slots,
        objective,
        limits,
        cancel,
        None,
        progress.map(PlanProgress::Search),
        checkpoint,
    )
}


/// Plan a TriPeaks route with live best-score tracking, cooperative soft-yield (`Play`) and pause hooks.
///
/// Matches [`plan_deal_route_with_control`] when `control.yield_best` stays false. When
/// `control.yield_best` becomes true without cancellation, the active pass halts promptly and
/// returns the best independently verified partial route discovered so far with
/// [`PartialProof::Bounded`] of [`SearchStop::Interrupted`].
///
/// # Errors
///
/// Returns [`ReplayError`] if `slots` fails validation or lacks a `Known` initial Waste slot.
pub fn plan_deal_route_controlled(
    slots: &[CardSlot],
    objective: Objective,
    limits: SearchLimits,
    control: BestEffortControl<'_>,
) -> Result<TriPeaksPlanOutcome, ReplayError> {
    plan_deal_route_inner(
        slots,
        objective,
        limits,
        control.cancel,
        control.yield_best,
        control.progress.map(PlanProgress::BestEffort),
        control.checkpoint,
    )
}


/// Select the higher-ranked independently verified candidate between a current pass candidate
/// and any prior verified fallback candidate for `objective`.
fn select_best_candidate(
    fixed: &[CardSlot; DEAL_SLOT_COUNT],
    objective: Objective,
    actions: Vec<TriPeaksMove>,
    score: u32,
    fallback_candidate: Option<(Vec<TriPeaksMove>, CandidateRank)>,
) -> Vec<TriPeaksMove> {
    let current = if score > 0 {
        evaluate_candidate(fixed, objective, &actions).map(|(_, rank)| (actions, rank))
    } else {
        None
    };
    match (current, fallback_candidate) {
        (Some((curr_actions, curr_rank)), Some((prev_actions, prev_rank))) => {
            if prev_rank > curr_rank {
                prev_actions
            } else {
                curr_actions
            }
        }
        (Some((curr_actions, _)), None) => curr_actions,
        (None, Some((prev_actions, _))) => prev_actions,
        (None, None) => Vec::new(),
    }
}

/// Convert an interrupted pass candidate into an independently replay-verified outcome,
/// promoting full objective or ClearBoard achievements cleanly before falling back to partial.
fn partial_yield_outcome(
    fixed: &[CardSlot; DEAL_SLOT_COUNT],
    objective: Objective,
    actions: Vec<TriPeaksMove>,
    score: u32,
    fallback_candidate: Option<(Vec<TriPeaksMove>, CandidateRank)>,
    unknown_slots: usize,
    stats: SearchStats,
) -> Result<TriPeaksPlanOutcome, ReplayError> {
    let candidate = select_best_candidate(fixed, objective, actions, score, fallback_candidate);
    if !candidate.is_empty() {
        if let Ok(replay) = verify_objective_route(fixed, &candidate, objective) {
            return Ok(TriPeaksPlanOutcome::SolvedObjective {
                actions: candidate,
                replay,
                stats,
            });
        }
        if matches!(
            objective,
            Objective::ClearRankQuota { .. } | Objective::ReachScoreDelta { .. }
        ) && let Ok(replay) = verify_objective_route(fixed, &candidate, Objective::ClearBoard)
        {
            return Ok(TriPeaksPlanOutcome::EscalatedClearBoard {
                actions: candidate,
                replay,
                stats,
            });
        }
    }
    let replay = verify_move_sequence(fixed, &candidate)?;
    let max_tableau_cleared =
        u8::try_from((FULL_TABLEAU_MASK & !replay.final_position.tableau_mask).count_ones())
            .unwrap_or(28);
    Ok(TriPeaksPlanOutcome::UnsolvablePartial {
        actions: candidate,
        replay,
        max_tableau_cleared,
        proof: PartialProof::Bounded(SearchStop::Interrupted),
        unknown_slots,
        stats,
    })
}


/// Unified implementation of [`plan_deal_route_with_control`] and [`plan_deal_route_controlled`].
fn plan_deal_route_inner(
    slots: &[CardSlot],
    objective: Objective,
    limits: SearchLimits,
    cancel: &AtomicBool,
    yield_best: Option<&AtomicBool>,
    progress: Option<PlanProgress<'_>>,
    checkpoint: Option<&dyn Fn() -> Duration>,
) -> Result<TriPeaksPlanOutcome, ReplayError> {
    let fixed = validate_slot_table(slots)?;
    let model = KnownDealModel::from_slots(fixed).ok_or(super::ModelError::UnknownInitialWaste)?;
    let mut env = PlanSearchEnv {
        fixed,
        model,
        objective,
        limits,
        cancel,
        yield_best,
        progress,
        checkpoint,
        acc_stats: SearchStats::default(),
        reported_best_score: 0,
        best_candidate: None,
    };
    let unknown_slots = fixed
        .iter()
        .filter(|slot| matches!(slot, CardSlot::Unknown))
        .count();

    let mut prior_bound: Option<SearchStop> = None;

    // 1. Try the direct caller objective first; non-time resource bounds fall through to escalation/partial.
    match env.run_objective(objective) {
        ObjectivePassResult::Solved(route) => {
            let replay = verify_objective_route(fixed, &route, objective)?;
            return Ok(TriPeaksPlanOutcome::SolvedObjective {
                actions: route,
                replay,
                stats: env.acc_stats,
            });
        }
        ObjectivePassResult::PartialYield { actions, score } => {
            let fallback = env.best_candidate.take();
            return partial_yield_outcome(
                fixed,
                objective,
                actions,
                score,
                fallback,
                unknown_slots,
                env.acc_stats,
            );
        }
        ObjectivePassResult::Stopped(
            reason
            @ (SearchStop::StateLimit | SearchStop::ExpansionLimit | SearchStop::MemoryLimit),
        ) => {
            prior_bound = Some(reason);
        }
        ObjectivePassResult::Stopped(reason) => {
            return Ok(TriPeaksPlanOutcome::Stopped {
                reason,
                stats: env.acc_stats,
            });
        }
        ObjectivePassResult::Exhausted => {}
    }

    // 2. Multi-board ClearBoard escalation rule for rank/score challenges exceeding one board.
    if matches!(
        objective,
        Objective::ClearRankQuota { .. } | Objective::ReachScoreDelta { .. }
    ) {
        env.sync_score_for_next_pass();
        match env.run_objective(Objective::ClearBoard) {
            ObjectivePassResult::Solved(fast_route) => {
                let fast_replay =
                    verify_objective_route(fixed, &fast_route, Objective::ClearBoard)?;
                let (actions, replay) = if matches!(objective, Objective::ReachScoreDelta { .. }) {
                    match maybe_upgrade_clear_score(&mut env, fast_route, fast_replay) {
                        Ok(pair) => pair,
                        Err(reason) => {
                            return Ok(TriPeaksPlanOutcome::Stopped {
                                reason,
                                stats: env.acc_stats,
                            });
                        }
                    }
                } else {
                    (fast_route, fast_replay)
                };
                return Ok(TriPeaksPlanOutcome::EscalatedClearBoard {
                    actions,
                    replay,
                    stats: env.acc_stats,
                });
            }
            ObjectivePassResult::PartialYield { actions, score } => {
                let fallback = env.best_candidate.take();
                return partial_yield_outcome(
                    fixed,
                    objective,
                    actions,
                    score,
                    fallback,
                    unknown_slots,
                    env.acc_stats,
                );
            }
            ObjectivePassResult::Stopped(
                reason @ (SearchStop::StateLimit
                | SearchStop::ExpansionLimit
                | SearchStop::MemoryLimit),
            ) => {
                if prior_bound.is_none() {
                    prior_bound = Some(reason);
                }
            }
            ObjectivePassResult::Stopped(reason) => {
                return Ok(TriPeaksPlanOutcome::Stopped {
                    reason,
                    stats: env.acc_stats,
                });
            }
            ObjectivePassResult::Exhausted => {}
        }
    }

    // 3. Dead-end fallback: measure maximum clearable cards, and bank positive rank/score gain.
    env.sync_score_for_next_pass();
    let dijkstra = SearchOptions {
        method: SearchMethod::Dijkstra,
        heuristic_weight: SearchOptions::default().heuristic_weight,
    };
    let (best_cards_actions, best_cards_proof) = match env
        .run_best_effort(&BestPartialProblem { model }, dijkstra)
    {
        Ok((actions, pass3_proof)) => {
            let combined_proof = match (prior_bound, pass3_proof) {
                (_, PartialProof::Bounded(SearchStop::Interrupted)) => {
                    PartialProof::Bounded(SearchStop::Interrupted)
                }
                (Some(stop), _) => combine_partial_proofs(PartialProof::Bounded(stop), pass3_proof),
                (None, _) => pass3_proof,
            };
            (actions, combined_proof)
        }
        Err(reason) => {
            return Ok(TriPeaksPlanOutcome::Stopped {
                reason,
                stats: env.acc_stats,
            });
        }
    };
    let best_cards_replay = verify_move_sequence(fixed, &best_cards_actions)?;
    let max_tableau_cleared = u8::try_from(
        (FULL_TABLEAU_MASK & !best_cards_replay.final_position.tableau_mask).count_ones(),
    )
    .unwrap_or(28);

    if best_cards_proof == PartialProof::Bounded(SearchStop::Interrupted) {
        let fallback = env.best_candidate.take();
        let fallback_score = if max_tableau_cleared > 0 {
            u32::from(max_tableau_cleared)
        } else {
            0
        };
        return partial_yield_outcome(
            fixed,
            objective,
            best_cards_actions,
            fallback_score,
            fallback,
            unknown_slots,
            env.acc_stats,
        );
    }

    let (actions, replay, proof) = match objective {
        Objective::ClearBoard => (
            Vec::new(),
            verify_move_sequence(fixed, &[])?,
            best_cards_proof,
        ),
        Objective::ClearRankQuota { rank, .. } => {
            let target_rank_mask = model.target_rank_mask(rank);
            if target_rank_mask == 0 {
                (
                    Vec::new(),
                    verify_move_sequence(fixed, &[])?,
                    best_cards_proof,
                )
            } else {
                let problem = MaxRankPartialProblem {
                    model,
                    target_rank_mask,
                };
                match env.run_best_effort(&problem, dijkstra) {
                    Ok((candidate, rank_proof)) => {
                        let proof = combine_partial_proofs(best_cards_proof, rank_proof);
                        if rank_proof == PartialProof::Bounded(SearchStop::Interrupted) {
                            let fallback = env.best_candidate.take();
                            let score = u32::from(!candidate.is_empty());
                            return partial_yield_outcome(
                                fixed,
                                objective,
                                candidate,
                                score,
                                fallback,
                                unknown_slots,
                                env.acc_stats,
                            )
                            .map(|outcome| match outcome {
                                TriPeaksPlanOutcome::UnsolvablePartial {
                                    actions,
                                    replay,
                                    max_tableau_cleared: yielded_cleared,
                                    proof: _,
                                    unknown_slots,
                                    stats,
                                } => TriPeaksPlanOutcome::UnsolvablePartial {
                                    actions,
                                    replay,
                                    max_tableau_cleared: max_tableau_cleared.max(yielded_cleared),
                                    proof,
                                    unknown_slots,
                                    stats,
                                },
                                other => other,
                            });
                        }
                        let best = select_best_candidate(
                            fixed,
                            objective,
                            candidate,
                            1,
                            env.best_candidate.take(),
                        );
                        let best_replay = verify_move_sequence(fixed, &best)?;
                        let cleared = best_replay
                            .final_position
                            .cleared_by_rank
                            .get(usize::from(rank))
                            .copied()
                            .unwrap_or(0);
                        if cleared > 0 {
                            (best, best_replay, proof)
                        } else {
                            (Vec::new(), verify_move_sequence(fixed, &[])?, proof)
                        }
                    }
                    Err(reason) => {
                        return Ok(TriPeaksPlanOutcome::Stopped {
                            reason,
                            stats: env.acc_stats,
                        });
                    }
                }
            }
        }
        Objective::ReachScoreDelta { .. } => {
            let problem = MaxScorePartialProblem { model };
            match env.run_best_effort(&problem, dijkstra) {
                Ok((candidate, score_proof)) => {
                    let proof = combine_partial_proofs(best_cards_proof, score_proof);
                    if score_proof == PartialProof::Bounded(SearchStop::Interrupted) {
                        let fallback = env.best_candidate.take();
                        let score = u32::from(!candidate.is_empty());
                        return partial_yield_outcome(
                            fixed,
                            objective,
                            candidate,
                            score,
                            fallback,
                            unknown_slots,
                            env.acc_stats,
                        )
                        .map(|outcome| match outcome {
                            TriPeaksPlanOutcome::UnsolvablePartial {
                                actions,
                                replay,
                                max_tableau_cleared: yielded_cleared,
                                proof: _,
                                unknown_slots,
                                stats,
                            } => TriPeaksPlanOutcome::UnsolvablePartial {
                                actions,
                                replay,
                                max_tableau_cleared: max_tableau_cleared.max(yielded_cleared),
                                proof,
                                unknown_slots,
                                stats,
                            },
                            other => other,
                        });
                    }
                    let base = select_best_candidate(
                        fixed,
                        objective,
                        best_cards_actions,
                        1,
                        env.best_candidate.take(),
                    );
                    let base_rank =
                        evaluate_candidate(fixed, objective, &base).map(|(_, rank)| (base, rank));
                    let best = select_best_candidate(fixed, objective, candidate, 1, base_rank);
                    let best_replay = verify_move_sequence(fixed, &best)?;
                    if best_replay.final_position.score_delta > 0 {
                        (best, best_replay, proof)
                    } else {
                        (Vec::new(), verify_move_sequence(fixed, &[])?, proof)
                    }
                }
                Err(reason) => {
                    return Ok(TriPeaksPlanOutcome::Stopped {
                        reason,
                        stats: env.acc_stats,
                    });
                }
            }
        }
    };

    Ok(TriPeaksPlanOutcome::UnsolvablePartial {
        actions,
        replay,
        max_tableau_cleared,
        proof,
        unknown_slots,
        stats: env.acc_stats,
    })
}

/// Combine two partial-search proof flags (`Interrupted` and `Exhaustive` rules preserve `Play`).
const fn combine_partial_proofs(first: PartialProof, second: PartialProof) -> PartialProof {
    match (first, second) {
        (PartialProof::Exhaustive, PartialProof::Exhaustive) => PartialProof::Exhaustive,
        (PartialProof::Bounded(SearchStop::Interrupted), _)
        | (_, PartialProof::Bounded(SearchStop::Interrupted)) => {
            PartialProof::Bounded(SearchStop::Interrupted)
        }
        (PartialProof::Bounded(stop), _) | (_, PartialProof::Bounded(stop)) => {
            PartialProof::Bounded(stop)
        }
    }
}

/// Convenience wrapper for [`plan_deal_route_with_control`] using [`DEFAULT_SEARCH_LIMITS`].
///
/// # Errors
///
/// Returns [`ReplayError`] if `slots` fails validation or lacks a `Known` initial Waste slot.
pub fn plan_deal_route(
    slots: &[CardSlot],
    objective: Objective,
) -> Result<TriPeaksPlanOutcome, ReplayError> {
    let cancel = AtomicBool::new(false);
    plan_deal_route_with_control(slots, objective, DEFAULT_SEARCH_LIMITS, &cancel, None, None)
}
