#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

//! `xsar::search` A*, info-gain probe planner, and best-partial fallback for TriPeaks deals.

use std::{sync::atomic::AtomicBool, time::Duration};

use xsar::search::{
    BestEffortOutcome, BestEffortProblem, SearchLimits, SearchMethod, SearchOptions, SearchOutcome,
    SearchProblem, SearchProgress, SearchStats, SearchStop, solve_best_effort_with_control,
    solve_with_control,
};

use crate::recon::{CardSlot, Objective};

use super::{
    DEAL_SLOT_COUNT, FULL_TABLEAU_MASK, INITIAL_WASTE_SLOT, PEAK_MASK, STOCK_CARD_COUNT,
    STOCK_START_SLOT, TABLEAU_SLOT_COUNT, TriPeaksMove,
    replay::verify_move_sequence,
    replay::verify_objective_route,
    rules::{
        PEAK_CLEAR_BONUS, are_ranks_adjacent, card_rank_index, is_slot_uncovered,
        slot_to_tableau_coord,
    },
    validate_slot_table,
};

/// Bounded synchronous search limits used by the zero-argument [`crate::recon::ReconDomain`] helpers.
pub const DEFAULT_SEARCH_LIMITS: SearchLimits = SearchLimits {
    max_states: 500_000,
    max_expansions: 500_000,
    max_memory_bytes: 256 * 1024 * 1024,
    max_duration: Duration::from_secs(10),
};

/// Maximum legal transitions from any reachable TriPeaks state (`28` tableau + `1` draw).
const MAX_SUCCESSORS: usize = TABLEAU_SLOT_COUNT + 1;

/// Compact copyable A* search key canonicalising streak/score outside `ReachScoreDelta`.
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
        Some(Self {
            initial_waste_rank: card_rank_index(waste_card),
            known_tableau_mask,
            known_tableau_ranks,
            known_stock_ranks,
        })
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
            Objective::ClearRankQuota { rank, .. } if rank < 13 => {
                let mut mask = 0_u32;
                for idx in 0..TABLEAU_SLOT_COUNT {
                    if (model.known_tableau_mask & (1_u32 << idx)) != 0
                        && model.known_tableau_ranks[idx] == rank
                    {
                        mask |= 1_u32 << idx;
                    }
                }
                mask
            }
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

    /// Soundly prove whether any occupied slot in `ClearBoard` has zero adjacent rank anywhere remaining.
    fn clear_board_starved(&self, state: SearchState) -> bool {
        let mut stock_rank_bits = 0_u16;
        for maybe_rank in &self.model.known_stock_ranks[usize::from(state.stock_cursor)..] {
            let Some(rank) = maybe_rank else {
                break;
            };
            stock_rank_bits |= 1_u16 << *rank;
        }
        let waste_and_stock = (1_u16 << state.waste_rank) | stock_rank_bits;

        let mut remaining = state.tableau_mask;
        while remaining != 0 {
            let idx = remaining.trailing_zeros() as usize;
            let bit = 1_u32 << idx;
            remaining &= !bit;
            let rank = self.model.known_tableau_ranks[idx];
            let adj_mask = (1_u16 << ((rank + 1) % 13)) | (1_u16 << ((rank + 12) % 13));
            if (waste_and_stock & adj_mask) != 0 {
                continue;
            }
            let mut other = state.tableau_mask & !bit;
            let mut has_tableau_partner = false;
            while other != 0 {
                let oidx = other.trailing_zeros() as usize;
                other &= !(1_u32 << oidx);
                let orank = self.model.known_tableau_ranks[oidx];
                if (adj_mask & (1_u16 << orank)) != 0 {
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
        let mut candidates = state.tableau_mask & self.model.known_tableau_mask;
        while candidates != 0 {
            let idx = candidates.trailing_zeros() as usize;
            let bit = 1_u32 << idx;
            candidates &= !bit;
            if !is_slot_uncovered(state.tableau_mask, idx as u8) {
                continue;
            }
            let rank = self.model.known_tableau_ranks[idx];
            if !are_ranks_adjacent(rank, state.waste_rank) {
                continue;
            }
            if let Some((row, column)) = slot_to_tableau_coord(crate::recon::SlotId(idx as u8)) {
                let (next_streak, next_score) = if track_score {
                    let s = state.streak.saturating_add(1);
                    let delta = super::rules::tap_score_delta(s, row == 1);
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
        if let Some(Some(stock_rank)) = self
            .model
            .known_stock_ranks
            .get(usize::from(state.stock_cursor))
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
                    || self.clear_board_starved(state)
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

/// Bitmask of initially covered `CardSlot::Unknown` tableau parents (`0..18`).
fn covered_unknown_parent_mask(slots: &[CardSlot; DEAL_SLOT_COUNT]) -> u32 {
    let mut mask = 0_u32;
    for (idx, entry) in slots.iter().take(18).enumerate() {
        if matches!(entry, CardSlot::Unknown) && !is_slot_uncovered(FULL_TABLEAU_MASK, idx as u8) {
            mask |= 1_u32 << idx;
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
    /// Bitmask of `Unknown` slots covered at `T = 0`.
    covered_unknown_mask: u32,
}

impl SearchProblem for InfoGainProbeProblem {
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
        count * 1_000 + r1 * 4 + r2 * 2 + r3
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

/// Plan the highest-information-gain legal move sequence exposing `>= 1` `Unknown` parent.
pub fn plan_info_gain_probe_with_control(
    slots: &[CardSlot],
    limits: SearchLimits,
    cancel: &AtomicBool,
    progress: Option<&mut dyn FnMut(SearchProgress)>,
    checkpoint: Option<&dyn Fn() -> Duration>,
) -> Option<Vec<TriPeaksMove>> {
    let fixed = validate_slot_table(slots).ok()?;
    let covered_unknown_mask = covered_unknown_parent_mask(fixed);
    if covered_unknown_mask == 0 {
        return None;
    }
    let model = KnownDealModel::from_slots(fixed)?;
    let problem = InfoGainProbeProblem {
        model,
        covered_unknown_mask,
    };
    let options = SearchOptions {
        method: SearchMethod::Dijkstra,
        heuristic_weight: SearchOptions::default().heuristic_weight,
    };
    let actions = match solve_best_effort_with_control(
        &problem, limits, cancel, options, progress, checkpoint,
    ) {
        BestEffortOutcome::Solved { actions, .. } => actions,
        BestEffortOutcome::Partial { actions, score, .. } if score > 0 => actions,
        BestEffortOutcome::Partial { .. } | BestEffortOutcome::Stopped { .. } => return None,
    };
    if actions.is_empty() {
        return None;
    }
    let report = verify_move_sequence(fixed, &actions).ok()?;
    (!report.newly_uncovered_unknown_slots.is_empty()).then_some(actions)
}

/// Convenience wrapper for [`plan_info_gain_probe_with_control`] using [`DEFAULT_SEARCH_LIMITS`].
pub fn plan_info_gain_probe(slots: &[CardSlot]) -> Option<Vec<TriPeaksMove>> {
    let cancel = AtomicBool::new(false);
    plan_info_gain_probe_with_control(slots, DEFAULT_SEARCH_LIMITS, &cancel, None, None)
}

/// Compute the maximum-tableau-removal partial route over `Known` cards using `xsar::search`.
pub fn best_partial_route_with_control(
    slots: &[CardSlot],
    limits: SearchLimits,
    cancel: &AtomicBool,
    progress: Option<&mut dyn FnMut(SearchProgress)>,
    checkpoint: Option<&dyn Fn() -> Duration>,
) -> Vec<TriPeaksMove> {
    let Some(model) = KnownDealModel::from_slots(slots) else {
        return Vec::new();
    };
    let problem = BestPartialProblem { model };
    let options = SearchOptions {
        method: SearchMethod::Dijkstra,
        heuristic_weight: SearchOptions::default().heuristic_weight,
    };
    let actions = match solve_best_effort_with_control(
        &problem, limits, cancel, options, progress, checkpoint,
    ) {
        BestEffortOutcome::Solved { actions, .. } | BestEffortOutcome::Partial { actions, .. } => {
            actions
        }
        BestEffortOutcome::Stopped { .. } => return Vec::new(),
    };
    if verify_move_sequence(slots, &actions).is_ok() {
        actions
    } else {
        Vec::new()
    }
}

/// Convenience wrapper for [`best_partial_route_with_control`] using [`DEFAULT_SEARCH_LIMITS`].
pub fn best_partial_route(slots: &[CardSlot]) -> Vec<TriPeaksMove> {
    let cancel = AtomicBool::new(false);
    best_partial_route_with_control(slots, DEFAULT_SEARCH_LIMITS, &cancel, None, None)
}
