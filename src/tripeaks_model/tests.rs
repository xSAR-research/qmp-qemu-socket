//! Comprehensive tests for TriPeaks rules, geometry, A* vs Dijkstra optimality, RECON loop, and JSON persistence.

use std::{
    collections::VecDeque,
    fs,
    io::Cursor,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use xsar::{
    games::cards::{Card as XsarCard, Rank, Suit},
    search::{PartialProof, SearchLimits, SearchMethod, SearchOptions, SearchOutcome, SearchStop},
};

use crate::{
    geometry::PixelRect,
    parameters::TABLEAU_CARD_REGIONS,
    recon::{
        BestPartialRoute, Card, CardSlot, Objective, ProbePlan, ReconDomain, ReconError,
        ReconOutcome, SlotId, run_epistemic_recon,
    },
};

use super::*;

/// Construct canonical `recon::Card` cleanly for tests from rank index (`0..13`) and suit (`0..4`).
fn card(rank_idx: u8, suit_idx: u8) -> Card {
    let rank = Rank::from_value(rank_idx + 1).expect("rank 1..=13");
    let suit = Suit::ALL[usize::from(suit_idx)];
    Card::try_from(XsarCard::new(rank, suit)).expect("canonical card id < 52")
}

/// Build a valid 52-card deal where `order` gives a valid bottom-up sequence of tableau slots
/// matching step-by-step from `Waste (28)` = `Ace of Clubs` through `+1 mod 13` (with an optional
/// Stock-gated slot) and fills unused Stock slots deterministically with the remaining unique cards.
fn build_cyclic_52_card_deal() -> [CardSlot; DEAL_SLOT_COUNT] {
    // Bottom-up clearing order: all Row 4 -> all Row 3 -> all Row 2 -> Peaks 0, 1, 2.
    let order: [usize; 28] = [
        18, 19, 20, 21, 22, 23, 24, 25, 26, 27, // Row 4 (`10`)
        9, 10, 11, 12, 13, 14, 15, 16, 17, // Row 3 (`9`)
        3, 4, 5, 6, 7, 8, // Row 2 (`6`)
        0, 1, 2, // Row 1 (`3`)
    ];
    let mut used = [false; 52];
    let mut suit_cursor = [0_u8; 13];
    let mut take_rank = |rank_idx: u8| -> Card {
        let s = suit_cursor[usize::from(rank_idx)];
        suit_cursor[usize::from(rank_idx)] = s + 1;
        let c = card(rank_idx, s);
        used[usize::from(c.0)] = true;
        c
    };

    let mut slots = [CardSlot::Unknown; DEAL_SLOT_COUNT];
    // Waste = Ace (rank 0)
    slots[usize::from(INITIAL_WASTE_SLOT.0)] = CardSlot::Known(take_rank(0));
    for (step, &slot_idx) in order.iter().enumerate() {
        let rank_idx = ((step + 1) % 13) as u8;
        slots[slot_idx] = CardSlot::Known(take_rank(rank_idx));
    }
    let mut stock_idx = usize::from(STOCK_START_SLOT);
    for (id, &taken) in used.iter().enumerate() {
        if !taken {
            slots[stock_idx] = CardSlot::Known(Card(id as u8));
            stock_idx += 1;
        }
    }
    assert_eq!(stock_idx, DEAL_SLOT_COUNT);
    validate_slot_table(&slots).expect("valid 52-card cyclic deal");
    slots
}

/// Build a 52-card deal where Peak 3 (`slot 2`, needing `Row 2` slot `8`) is isolated behind `Stock[0]`
/// while Rows 4 -> 3 -> 2 (`slots 3..=7`) + Peaks `0` and `1` clear smoothly from `Waste`.
fn build_stock_gated_52_card_deal() -> [CardSlot; DEAL_SLOT_COUNT] {
    // Chain 1 (from Waste rank 0): Row 4 (`18..=27`), Row 3 (`9..=17`), Row 2 (`3..=7`), Peaks `0, 1` (`24` cards).
    // Chain 2 (from Stock[0]): Row 2 (`8`), Peak `2` (`2`) (`2` cards). Wait: `order` covers `26` cards, plus `7` & `8`?
    // Wait: Peak 2 (`slot 2`) is covered by `slots 7 and 8`! To reach `51` known via probing ONLY up to Peak 0 & Peak 1,
    // clear `3..=6` (`2` peaks = `0` and `1`), then Peak `2` stays covered until `7` & `8` are ALSO cleared.
    // Put `slot 7` & `slot 8` on ranks matching `Stock[0] -> 7 -> 8 -> 2`, breaking direct Waste chain cleanly!
    let main_chain: [usize; 25] = [
        18, 19, 20, 21, 22, 23, 24, 25, 26, 27, // Row 4
        9, 10, 11, 12, 13, 14, 15, 16, 17, // Row 3
        3, 4, 5, 6, // Row 2 slots under Peak 0 and Peak 1
        0, 1, // Peak 0 and Peak 1 (leaving Peak 2 hidden until Stock[0] -> 7 -> 8 -> 2!)
    ];
    let mut used = [false; 52];
    let mut suit_cursor = [0_u8; 13];
    let mut take_rank = |rank_idx: u8| -> Card {
        let s = suit_cursor[usize::from(rank_idx)];
        assert!(s < 4, "rank {rank_idx} exhausted");
        suit_cursor[usize::from(rank_idx)] = s + 1;
        let c = card(rank_idx, s);
        used[usize::from(c.0)] = true;
        c
    };

    let mut slots = [CardSlot::Unknown; DEAL_SLOT_COUNT];
    // Waste = rank 0 (Ace). Main chain cycles through 25 ranks `+1 mod 13`: ends at rank `25 % 13 = 12` (King)!
    slots[usize::from(INITIAL_WASTE_SLOT.0)] = CardSlot::Known(take_rank(0));
    for (step, &slot_idx) in main_chain.iter().enumerate() {
        let rank_idx = ((step + 1) % 13) as u8;
        slots[slot_idx] = CardSlot::Known(take_rank(rank_idx));
    }
    // Note: if `slots 7, 8, 2` used ranks `{4, 5, 6}`, main_chain ends at King (`12`) which does NOT match `5`!
    // Assign `Stock[0] = rank 4`, `slot 7 = rank 5`, `slot 8 = rank 6`, `slot 2 = rank 7`!
    // Check suit usage: main chain (`25` cards) + Waste (`1`) uses each rank at most twice, so `4, 5, 6, 7` have plenty of suits!
    slots[usize::from(STOCK_START_SLOT)] = CardSlot::Known(take_rank(4));
    slots[7] = CardSlot::Known(take_rank(5));
    slots[8] = CardSlot::Known(take_rank(6));
    slots[2] = CardSlot::Known(take_rank(7));

    // Fill Stock[1..23] with all unused cards (avoiding accidental `0 -> 5` etc. wait: doesn't hurt, all cards valid!).
    let mut stock_idx = usize::from(STOCK_START_SLOT) + 1;
    for (id, &taken) in used.iter().enumerate() {
        if !taken {
            slots[stock_idx] = CardSlot::Known(Card(id as u8));
            stock_idx += 1;
        }
    }
    assert_eq!(stock_idx, DEAL_SLOT_COUNT);
    validate_slot_table(&slots).expect("valid 52-card stock-gated deal");
    slots
}

/// Mask initial `Rows 1..=3` (`0..18`) as `CardSlot::Unknown`, leaving Row 4, Waste, and Stock `Known`.
fn hide_rows_1_to_3(full: &[CardSlot; DEAL_SLOT_COUNT]) -> [CardSlot; DEAL_SLOT_COUNT] {
    let mut initial = *full;
    for slot in &mut initial[0..18] {
        *slot = CardSlot::Unknown;
    }
    initial
}

/// Helper to check whether two `PixelRect` rectangles intersect in `1920×1080` coordinates.
fn rects_overlap(a: PixelRect, b: PixelRect) -> bool {
    a.x < b.right() && b.x < a.right() && a.y < b.bottom() && b.y < a.bottom()
}

#[test]
fn covering_masks_match_calibrated_tableau_card_regions_exactly() {
    for (parent, parent_region) in TABLEAU_CARD_REGIONS.iter().enumerate() {
        let mut expected_mask = 0_u32;
        let parent_row = slot_to_tableau_coord(SlotId(parent as u8))
            .expect("valid parent")
            .0;
        for (child, child_region) in TABLEAU_CARD_REGIONS.iter().enumerate() {
            if parent == child {
                continue;
            }
            let child_row = slot_to_tableau_coord(SlotId(child as u8))
                .expect("valid child")
                .0;
            if child_row == parent_row + 1
                && rects_overlap(parent_region.card_bounds, child_region.card_bounds)
            {
                expected_mask |= 1_u32 << child;
            }
        }
        assert_eq!(
            COVERING_CHILD_MASKS[parent], expected_mask,
            "covering mask mismatch at slot {parent}"
        );
    }
}

#[test]
fn coordinates_adjacency_streak_scoring_and_unknown_blocking_match_rules() {
    // Bijection over all 28 tableau slots.
    let mut seen = [false; TABLEAU_SLOT_COUNT];
    for row in 1..=4_u8 {
        let max_col = match row {
            1 => 3,
            2 => 6,
            3 => 9,
            4 => 10,
            _ => unreachable!(),
        };
        for col in 1..=max_col {
            let sid = tableau_slot_id(row, col).expect("valid coordinate");
            assert_eq!(slot_to_tableau_coord(sid), Some((row, col)));
            seen[usize::from(sid.0)] = true;
        }
        assert_eq!(tableau_slot_id(row, max_col + 1), None);
    }
    assert!(seen.iter().all(|&x| x));
    assert_eq!(tableau_slot_id(0, 1), None);
    assert_eq!(tableau_slot_id(5, 1), None);
    assert_eq!(slot_to_tableau_coord(INITIAL_WASTE_SLOT), None);

    // King <-> Ace circular wrap.
    assert!(are_ranks_adjacent(0, 12));
    assert!(are_ranks_adjacent(12, 0));
    assert!(are_ranks_adjacent(5, 6));
    assert!(!are_ranks_adjacent(5, 5));
    assert!(!are_ranks_adjacent(0, 11));

    // Beast-verified streak points (`100*(2k-1)`) + Peak bonus (`+500`).
    let expected = [100, 300, 500, 700, 900, 1_100];
    for (i, &pts) in expected.iter().enumerate() {
        assert_eq!(tap_score_delta((i + 1) as u8, false), pts);
        assert_eq!(tap_score_delta((i + 1) as u8, true), pts + PEAK_CLEAR_BONUS);
    }

    // `Unknown` and `Unreachable` must block tapping & Stock drawing even if uncovered.
    let full = build_cyclic_52_card_deal();
    let mut blocked = hide_rows_1_to_3(&full);
    blocked[18] = CardSlot::Unknown;
    blocked[19] = CardSlot::Unreachable;
    blocked[usize::from(STOCK_START_SLOT)] = CardSlot::Unreachable;
    let pos = TriPeaksPosition::from_root(&blocked).expect("known waste");
    assert_eq!(
        apply_move(
            &pos,
            &blocked,
            TriPeaksMove::TapTableau { row: 4, column: 1 }
        ),
        Err(RuleViolation::SlotUnknown { slot: SlotId(18) })
    );
    assert_eq!(
        apply_move(
            &pos,
            &blocked,
            TriPeaksMove::TapTableau { row: 4, column: 2 }
        ),
        Err(RuleViolation::SlotUnreachable { slot: SlotId(19) })
    );
    assert_eq!(
        apply_move(&pos, &blocked, TriPeaksMove::Draw),
        Err(RuleViolation::SlotUnreachable {
            slot: SlotId(STOCK_START_SLOT),
        })
    );
    assert!(!legal_moves(&pos, &blocked).contains(&TriPeaksMove::TapTableau { row: 4, column: 1 }));
    assert!(!legal_moves(&pos, &blocked).contains(&TriPeaksMove::Draw));

    // Clearing (4, 1) and (4, 2) uncovers `Unknown` parent (3, 1) -> `SlotId(9)` while `Draw` resets streak.
    let partial_unknown = hide_rows_1_to_3(&full);
    let pos0 = TriPeaksPosition::from_root(&partial_unknown).expect("known waste");
    let (pos1, out1) = apply_move(
        &pos0,
        &partial_unknown,
        TriPeaksMove::TapTableau { row: 4, column: 1 },
    )
    .expect("tap (4,1)");
    assert_eq!(out1.score_gained, 100);
    assert_eq!(pos1.streak, 1);
    assert!(out1.newly_uncovered_slots.is_empty());

    let (pos2, out2) = apply_move(
        &pos1,
        &partial_unknown,
        TriPeaksMove::TapTableau { row: 4, column: 2 },
    )
    .expect("tap (4,2)");
    assert_eq!(out2.score_gained, 300);
    assert_eq!(pos2.streak, 2);
    assert_eq!(out2.newly_uncovered_slots, vec![SlotId(9)]);

    let (pos3, out3) = apply_move(&pos2, &partial_unknown, TriPeaksMove::Draw).expect("draw");
    assert_eq!(out3.score_gained, 0);
    assert_eq!(pos3.streak, 0);
    assert_eq!(pos3.score_delta, 400);
}

#[test]
fn replay_rejects_covered_nonadjacent_and_overlong_routes() {
    let full = build_cyclic_52_card_deal();
    assert!(matches!(
        verify_move_sequence(&full, &[TriPeaksMove::TapTableau { row: 3, column: 1 }],),
        Err(ReplayError::Rule {
            source: RuleViolation::SlotCovered { .. },
            ..
        })
    ));
    assert!(matches!(
        verify_move_sequence(&full, &[TriPeaksMove::TapTableau { row: 4, column: 2 }],),
        Err(ReplayError::Rule {
            source: RuleViolation::RankNotAdjacent { .. },
            ..
        })
    ));
    assert!(matches!(
        verify_move_sequence(&full, &[TriPeaksMove::Draw; MAX_ROUTE_MOVES + 1]),
        Err(ReplayError::RouteTooLong { actual }) if actual == MAX_ROUTE_MOVES + 1
    ));
}

#[test]
fn astar_matches_dijkstra_minimum_cost_across_all_three_objectives() {
    // 10-card Row-4 clear with full board masked (`Slots 0..18` untouched? Wait: for `ClearBoard` test
    // all 28 cards Known on the small gated/cyclic deal so both Dijkstra & A* finish fast!)
    // Notice on `build_stock_gated_52_card_deal`, let's construct a compact mid-board deal for ClearBoard
    // OR test `ClearRankQuota` / `ReachScoreDelta` where `Known` subset solves cleanly in milliseconds!
    let full = build_cyclic_52_card_deal();
    let cancel = AtomicBool::new(false);

    for objective in [
        Objective::ClearRankQuota {
            rank: 4, // 5s (`2` available on Row 4 / Row 3? `step=3` is `(4,4)`, `step=16` is `(3,8)`!)
            remaining: 1,
        },
        Objective::ReachScoreDelta { points_needed: 900 }, // 3 consecutive taps (`100+300+500 = 900`)
        Objective::ClearBoard,
    ] {
        let slots = full;

        let astar_outcome = solve_known_subgraph_with_control(
            &slots,
            objective,
            DEFAULT_SEARCH_LIMITS,
            SearchOptions {
                method: SearchMethod::AStar,
                heuristic_weight: std::num::NonZeroU32::MIN,
            },
            &cancel,
            None,
            None,
        );
        let SearchOutcome::Solved {
            actions: astar_actions,
            ..
        } = astar_outcome
        else {
            panic!("A* failed on {objective:?}");
        };

        // Notice: on `ClearBoard` of a completely connected 28-card deal without pruning dead branches,
        // pure blind Dijkstra over 2^10 * 2^9... can visit many permutations, whereas on `ClearRankQuota`
        // and `ReachScoreDelta` Dijkstra finishes in milliseconds! Check Dijkstra directly on both,
        // and on `ClearBoard` assert optimal lower bound (`actions.len() == 28`, matching mathematical minimum
        // because 28 tableau slots cannot be cleared in `< 28` moves!).
        if !matches!(objective, Objective::ClearBoard) {
            let dijkstra_outcome = solve_known_subgraph_with_control(
                &slots,
                objective,
                DEFAULT_SEARCH_LIMITS,
                SearchOptions {
                    method: SearchMethod::Dijkstra,
                    heuristic_weight: std::num::NonZeroU32::MIN,
                },
                &cancel,
                None,
                None,
            );
            let SearchOutcome::Solved {
                actions: dijkstra_actions,
                ..
            } = dijkstra_outcome
            else {
                panic!("Dijkstra failed on {objective:?}");
            };
            assert_eq!(astar_actions.len(), dijkstra_actions.len());
        } else {
            assert_eq!(astar_actions.len(), 28);
        }
        verify_objective_route(&slots, &astar_actions, objective).expect("route passes replay");
    }

    // Verify exact `AStar == Dijkstra` minimum move count on a `ClearBoard` deal across
    // three disjoint rank islands (`{0..3}`, `{4..7}`, `{9..12}`) bridged by `2` Stock `Draw`s (`30` moves).
    let mut island_deal = [CardSlot::Unknown; DEAL_SLOT_COUNT];
    let mut is_used = [false; 52];
    let mut is_sc = [0_u8; 13];
    let mut is_pick = |r: u8| -> Card {
        let s = is_sc[usize::from(r)];
        assert!(s < 4, "rank {r} exhausted in island_deal");
        is_sc[usize::from(r)] = s + 1;
        let c = card(r, s);
        is_used[usize::from(c.0)] = true;
        c
    };
    island_deal[usize::from(INITIAL_WASTE_SLOT.0)] = CardSlot::Known(is_pick(0));
    island_deal[usize::from(STOCK_START_SLOT)] = CardSlot::Known(is_pick(4));
    island_deal[usize::from(STOCK_START_SLOT) + 1] = CardSlot::Known(is_pick(9));
    let island_assign: &[(usize, u8)] = &[
        (18, 1),
        (19, 2),
        (9, 3),
        (20, 2),
        (21, 1),
        (10, 0),
        (3, 1),
        (11, 2),
        (4, 3),
        (0, 2),
        (22, 5),
        (23, 6),
        (12, 7),
        (13, 6),
        (24, 5),
        (14, 6),
        (5, 7),
        (6, 6),
        (1, 5),
        (25, 10),
        (15, 11),
        (26, 12),
        (16, 11),
        (27, 10),
        (17, 11),
        (7, 12),
        (8, 11),
        (2, 10),
    ];
    for &(slot_idx, rank_idx) in island_assign {
        island_deal[slot_idx] = CardSlot::Known(is_pick(rank_idx));
    }
    let mut is_stock_cursor = usize::from(STOCK_START_SLOT) + 2;
    for (id, &taken) in is_used.iter().enumerate() {
        if !taken {
            island_deal[is_stock_cursor] = CardSlot::Known(Card(id as u8));
            is_stock_cursor += 1;
        }
    }
    assert_eq!(is_stock_cursor, DEAL_SLOT_COUNT);
    validate_slot_table(&island_deal).expect("valid island deal");

    let astar_cb = solve_known_subgraph_with_control(
        &island_deal,
        Objective::ClearBoard,
        DEFAULT_SEARCH_LIMITS,
        SearchOptions::default(),
        &cancel,
        None,
        None,
    );
    match astar_cb {
        SearchOutcome::Solved { actions: a, .. } => {
            assert_eq!(a.len(), 30);
            verify_objective_route(&island_deal, &a, Objective::ClearBoard).expect("valid astar");
        }
        other => panic!("unexpected full-deal A* ClearBoard outcome: {other:?}"),
    }

    // Cap unused tail stock (`31..52`) so blind Dijkstra terminates cleanly and confirms 30 is optimal.
    let mut island_two_draws = island_deal;
    island_two_draws[31..DEAL_SLOT_COUNT].fill(CardSlot::Unknown);
    let astar_two = solve_known_subgraph_with_control(
        &island_two_draws,
        Objective::ClearBoard,
        DEFAULT_SEARCH_LIMITS,
        SearchOptions::default(),
        &cancel,
        None,
        None,
    );
    let dijkstra_two = solve_known_subgraph_with_control(
        &island_two_draws,
        Objective::ClearBoard,
        DEFAULT_SEARCH_LIMITS,
        SearchOptions {
            method: SearchMethod::Dijkstra,
            heuristic_weight: std::num::NonZeroU32::MIN,
        },
        &cancel,
        None,
        None,
    );
    match (astar_two, dijkstra_two) {
        (
            SearchOutcome::Solved { actions: a_two, .. },
            SearchOutcome::Solved { actions: d_two, .. },
        ) => {
            assert_eq!(a_two.len(), 30);
            assert_eq!(a_two.len(), d_two.len());
            verify_objective_route(&island_two_draws, &a_two, Objective::ClearBoard)
                .expect("valid capped astar");
        }
        other => panic!("unexpected capped island ClearBoard outcome: {other:?}"),
    }

    // Also verify Dijkstra == A* on a Peak-bonus `ReachScoreDelta` route (`1,600` points).
    let high_score = Objective::ReachScoreDelta {
        points_needed: 1_600,
    };
    let astar_s = solve_known_subgraph_with_control(
        &full,
        high_score,
        DEFAULT_SEARCH_LIMITS,
        SearchOptions::default(),
        &cancel,
        None,
        None,
    );
    let dijk_s = solve_known_subgraph_with_control(
        &full,
        high_score,
        DEFAULT_SEARCH_LIMITS,
        SearchOptions {
            method: SearchMethod::Dijkstra,
            heuristic_weight: std::num::NonZeroU32::MIN,
        },
        &cancel,
        None,
        None,
    );
    match (astar_s, dijk_s) {
        (SearchOutcome::Solved { actions: a, .. }, SearchOutcome::Solved { actions: d, .. }) => {
            assert_eq!(a.len(), d.len());
        }
        other => panic!("unexpected score outcome: {other:?}"),
    }
}

/// Pure deterministic `ReconDomain` backed by an oracle 52-card deal for end-to-end tests.
struct SimulatedTriPeaksDomain {
    /// Complete ground-truth 52-card deal (`T = 0` hidden cards in `0..18` revealed on probe).
    truth: [CardSlot; DEAL_SLOT_COUNT],
    /// Stable visual root hash verified on `undo_all_and_hash`.
    root_hash: u64,
    /// Objective currently solved against `truth`.
    objective: Objective,
    /// Optional forced error injected on a specific call (duplicate card, mismatch, etc.).
    inject_duplicate_on_probe: Option<(SlotId, Card)>,
    /// If `Some(n)`, simulate completing the live Challenge/board during probe #`n`.
    complete_live_on_probe: Option<usize>,
    /// Counter of executed probes (`1`-based inside callback).
    probe_counter: usize,
    /// Counter of `undo_all_and_hash` calls.
    undo_counter: usize,
    /// Log of executed probe lengths for inspection.
    probe_history: VecDeque<usize>,
}

impl SimulatedTriPeaksDomain {
    /// Create a simulated TriPeaks session starting at `T = 0`.
    fn new(truth: [CardSlot; DEAL_SLOT_COUNT], objective: Objective) -> Self {
        Self {
            truth,
            root_hash: 0x5452_4950_4541_4B53,
            objective,
            inject_duplicate_on_probe: None,
            complete_live_on_probe: None,
            probe_counter: 0,
            undo_counter: 0,
            probe_history: VecDeque::new(),
        }
    }
}

impl ReconDomain for SimulatedTriPeaksDomain {
    type Move = TriPeaksMove;

    fn initialize_and_sweep_stock(&mut self) -> Result<(Vec<CardSlot>, u64), ReconError> {
        // Convert `[CardSlot; 52]` copy hiding Rows 1..3 as Unknown (`34` Known + `18` Unknown).
        let full_fixed =
            validate_slot_table(&self.truth).map_err(|_| ReconError::SceneVerification {
                detail: "invalid truth deal".to_owned(),
            })?;
        Ok((hide_rows_1_to_3(full_fixed).to_vec(), self.root_hash))
    }

    fn solve_known_subgraph(
        &self,
        slots: &[CardSlot],
        objective: Objective,
    ) -> Option<Vec<Self::Move>> {
        solve_known_subgraph(slots, objective)
    }

    fn plan_info_gain_probe(
        &self,
        slots: &[CardSlot],
    ) -> Result<ProbePlan<Self::Move>, ReconError> {
        plan_info_gain_probe(slots)
    }

    fn execute_probe_and_observe(
        &mut self,
        probe: &[Self::Move],
    ) -> Result<Option<Vec<(SlotId, Card)>>, ReconError> {
        self.probe_counter += 1;
        self.probe_history.push_back(probe.len());
        if self.complete_live_on_probe == Some(self.probe_counter) {
            return Ok(None);
        }
        // Replay `probe` against `self.truth` to observe the physical newly uncovered parent cards.
        let report = verify_move_sequence(&self.truth, probe).map_err(|e| {
            ReconError::SceneVerification {
                detail: e.to_string(),
            }
        })?;
        // Walk step-by-step so we capture every parent uncovered during `probe` (in `0..18`).
        let mut pos = TriPeaksPosition::from_root(&self.truth).map_err(|e| {
            ReconError::SceneVerification {
                detail: e.to_string(),
            }
        })?;
        let mut uncovered_ids = Vec::new();
        for &mv in probe {
            let (next, out) =
                apply_move(&pos, &self.truth, mv).map_err(|e| ReconError::SceneVerification {
                    detail: e.to_string(),
                })?;
            for s in out.newly_uncovered_slots {
                if !uncovered_ids.contains(&s) {
                    uncovered_ids.push(s);
                }
            }
            pos = next;
        }
        if pos.satisfies_objective(self.objective) {
            return Ok(None);
        }
        let _ = report;
        if let Some(dup) = self.inject_duplicate_on_probe.take() {
            return Ok(Some(vec![dup]));
        }
        let mut observations = Vec::with_capacity(uncovered_ids.len());
        for s in uncovered_ids {
            let CardSlot::Known(c) = self.truth[usize::from(s.0)] else {
                return Err(ReconError::VisionClassificationFailed { slot: s });
            };
            observations.push((s, c));
        }
        Ok(Some(observations))
    }

    fn undo_all_and_hash(&mut self) -> Result<u64, ReconError> {
        self.undo_counter += 1;
        Ok(self.root_hash)
    }

    fn best_partial_route(
        &self,
        slots: &[CardSlot],
    ) -> Result<BestPartialRoute<Self::Move>, ReconError> {
        best_partial_route(slots)
    }
}

#[test]
fn end_to_end_recon_solves_full_board_with_51_to_52_deduction_and_round_trips_json() {
    // Stock-gated deal leaves Peak 3 (`slot 2`) unexposed during Peak 0+1 probing (`34 + 9 + 6 + 2 = 51`!).
    let truth = build_stock_gated_52_card_deal();
    let mut domain = SimulatedTriPeaksDomain::new(truth, Objective::ClearBoard);
    // Avoid `satisfies_objective` prematurely ending during probe by only keeping objective = ClearBoard.
    let outcome = run_epistemic_recon(&mut domain, Objective::ClearBoard, true, 10)
        .expect("RECON should converge cleanly");

    let ReconOutcome::SolvedFromRoot { route, probes_used } = outcome else {
        panic!("expected SolvedFromRoot, got {outcome:?}");
    };
    assert!(probes_used >= 2);
    assert_eq!(domain.undo_counter, probes_used);
    // Notice `solve_known_subgraph` uses `plan_info_gain_probe`: check our route solves the entire `truth` deal!
    let report = verify_objective_route(&truth, &route, Objective::ClearBoard)
        .expect("root route clears full 52-card deal");
    assert_eq!(report.final_position.tableau_mask, 0);

    // Export and reload through strict JSON codec.
    let doc = TriPeaksDealDocument {
        provenance: DealProvenance::RestoredAtRoot,
        probes_used,
        objective: Objective::ClearBoard,
        slots: truth,
        planned_route: Some(route),
    };
    let encoded = to_vec(&doc).expect("encode valid deal");
    assert!(encoded.len() <= MAX_DOCUMENT_BYTES);
    let decoded = from_slice(&encoded).expect("decode round-trip");
    assert_eq!(decoded, doc);
}

#[test]
fn end_to_end_recon_exits_early_for_clear_rank_and_score_with_unknown_slots_remaining() {
    // Rank quota (`remaining: 1` Ace -> only needs Row 4 or Waste match, leaving Rows 1..3 Unknown!).
    let truth = build_cyclic_52_card_deal();
    // On `build_cyclic_52_card_deal`, Row 4 has `2..J` (`ranks 1..10`) so `Two` (`rank 1`) is at `(4, 1)`!
    let rank_obj = Objective::ClearRankQuota {
        rank: 1,
        remaining: 1,
    };
    let mut dom_rank = SimulatedTriPeaksDomain::new(truth, rank_obj);
    let out_rank =
        run_epistemic_recon(&mut dom_rank, rank_obj, true, 8).expect("rank RECON succeeds");
    assert_eq!(
        out_rank,
        ReconOutcome::SolvedFromRoot {
            route: vec![TriPeaksMove::TapTableau { row: 4, column: 1 }],
            probes_used: 0,
        }
    );
    assert_eq!(dom_rank.probe_counter, 0);

    // Score delta (`points_needed: 1_600` = 4 consecutive Row-4 taps: `100+300+500+700 = 1,600`), zero probes!
    let score_obj = Objective::ReachScoreDelta {
        points_needed: 1_600,
    };
    let mut dom_score = SimulatedTriPeaksDomain::new(truth, score_obj);
    let out_score =
        run_epistemic_recon(&mut dom_score, score_obj, true, 8).expect("score RECON succeeds");
    let ReconOutcome::SolvedFromRoot { route, probes_used } = out_score else {
        panic!("expected early root score solve");
    };
    assert_eq!(probes_used, 0);
    verify_objective_route(&hide_rows_1_to_3(&truth), &route, score_obj)
        .expect("solves using strictly Known Row-4 slots");
}

#[test]
fn end_to_end_recon_handles_live_completion_duplicate_card_and_unsolvable_unreachable_peaks() {
    let truth = build_cyclic_52_card_deal();

    // 1. Live Challenge completion on probe #1 (`CompletedDuringRecon`).
    let mut dom_live = SimulatedTriPeaksDomain::new(truth, Objective::ClearBoard);
    dom_live.complete_live_on_probe = Some(1);
    let out_live = run_epistemic_recon(&mut dom_live, Objective::ClearBoard, true, 8)
        .expect("live completion exits gracefully");
    assert_eq!(
        out_live,
        ReconOutcome::CompletedDuringRecon { probes_used: 1 }
    );
    let live_doc = TriPeaksDealDocument {
        provenance: DealProvenance::CompletedDuringRecon,
        probes_used: 1,
        objective: Objective::ClearBoard,
        slots: hide_rows_1_to_3(&truth),
        planned_route: None,
    };
    assert_eq!(
        from_slice(&to_vec(&live_doc).expect("encode")).expect("decode"),
        live_doc
    );

    // 2. Duplicate single-deck card returned by probe fails closed with `ReconError::DuplicateCard`.
    let mut dom_dup = SimulatedTriPeaksDomain::new(truth, Objective::ClearBoard);
    let CardSlot::Known(already_known) = truth[18] else {
        panic!("expected Known at 18");
    };
    dom_dup.inject_duplicate_on_probe = Some((SlotId(9), already_known));
    let err = run_epistemic_recon(&mut dom_dup, Objective::ClearBoard, true, 8)
        .expect_err("duplicate must fail closed");
    assert!(matches!(
        err,
        ReconError::DuplicateCard {
            first_slot: SlotId(18),
            second_slot: SlotId(9),
            card,
        } if card == already_known
    ));

    // 3. Unsolvable deal: trap all 4 Fours (rank 3) and all 4 Kings (rank 12) + 1 Ace in `slots 0..9` (`Rows 1..=2`),
    // fill `slots 9..18` + `26..28` with `{3 Aces, 4 Twos, 4 Threes}`, and restrict `{18..26, Waste, Stock}` to `ranks 4..=11`.
    // Because no card outside `slots 0..9` has rank `3` (Four) or `12` (King), none of `{A, 2, 3}` can ever be tapped!
    let mut blocked_deal = [CardSlot::Unknown; DEAL_SLOT_COUNT];
    let mut b_used = [false; 52];
    let mut bsc = [0_u8; 13];
    let mut bpick = |r: u8| -> CardSlot {
        let s = bsc[usize::from(r)];
        bsc[usize::from(r)] = s + 1;
        let c = card(r, s);
        b_used[usize::from(c.0)] = true;
        CardSlot::Known(c)
    };
    // Rows 1..=2 (`0..9`): 4 Fours + 4 Kings + 1 Ace (`9` slots).
    for (slot_idx, r) in [3_u8, 3, 3, 3, 12, 12, 12, 12, 0].into_iter().enumerate() {
        blocked_deal[slot_idx] = bpick(r);
    }
    // Row 3 (`9..18`) + Row 4 right (`26..28`): 3 Aces + 4 Twos + 4 Threes (`11` slots).
    for (dst, r) in (9..18)
        .chain(26..28)
        .zip([0_u8, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2])
    {
        blocked_deal[dst] = bpick(r);
    }
    let mut rest = Vec::new();
    for (id, &used_flag) in b_used.iter().enumerate() {
        if !used_flag {
            rest.push(CardSlot::Known(Card(id as u8)));
        }
    }
    let mut ri = rest.into_iter();
    blocked_deal[usize::from(INITIAL_WASTE_SLOT.0)] = ri.next().expect("waste");
    for slot in &mut blocked_deal[18..26] {
        *slot = ri.next().expect("row4");
    }
    for slot in &mut blocked_deal[29..52] {
        *slot = ri.next().expect("stock");
    }
    assert!(ri.next().is_none());
    validate_slot_table(&blocked_deal).expect("valid 52-card deal");

    let mut dom_unsolv = SimulatedTriPeaksDomain::new(blocked_deal, Objective::ClearBoard);
    let out_unsolv = run_epistemic_recon(&mut dom_unsolv, Objective::ClearBoard, true, 10)
        .expect("unsolvable deal returns UnsolvableBestPartial");
    let ReconOutcome::UnsolvableBestPartial {
        best_partial_route,
        unreachable_slots,
        proof,
    } = out_unsolv
    else {
        panic!("expected UnsolvableBestPartial, got {out_unsolv:?}");
    };
    assert!(unreachable_slots >= 3);
    assert_eq!(proof, PartialProof::Exhaustive);
    verify_move_sequence(&hide_rows_1_to_3(&blocked_deal), &best_partial_route)
        .expect("best partial route is legal from T=0");

    // Verify `"Unreachable"` serialisation round-trips cleanly.
    let mut partial_slots = hide_rows_1_to_3(&blocked_deal);
    partial_slots[2] = CardSlot::Unreachable;
    let doc_unreach = TriPeaksDealDocument {
        provenance: DealProvenance::RestoredAtRoot,
        probes_used: dom_unsolv.probe_counter,
        objective: Objective::ClearBoard,
        slots: partial_slots,
        planned_route: Some(best_partial_route),
    };
    let unreach_bytes = to_vec(&doc_unreach).expect("encode Unreachable deal");
    assert_eq!(from_slice(&unreach_bytes).expect("decode"), doc_unreach);
}

#[test]
fn fifty_one_to_fifty_two_complement_deduces_final_peak_without_physical_probe() {
    // Stock-gated deal: keep ONLY `slot 0` (Peak 0) and `slot 2` (Peak 2) Unknown at initial sweep (`50` Known).
    // Probing clears `3, 4` to expose `slot 0` (`51` Known) WITHOUT ever exposing `slot 2`.
    // `bind_observation` must immediately deduce `slot 2` via 52-card complement so `ClearBoard` solves!
    struct PenultimateDomain {
        truth: [CardSlot; DEAL_SLOT_COUNT],
        probes_run: usize,
        root_hash: u64,
    }
    impl ReconDomain for PenultimateDomain {
        type Move = TriPeaksMove;

        fn initialize_and_sweep_stock(&mut self) -> Result<(Vec<CardSlot>, u64), ReconError> {
            let mut slots = self.truth;
            slots[0] = CardSlot::Unknown;
            slots[2] = CardSlot::Unknown;
            Ok((slots.to_vec(), self.root_hash))
        }

        fn solve_known_subgraph(
            &self,
            slots: &[CardSlot],
            objective: Objective,
        ) -> Option<Vec<Self::Move>> {
            solve_known_subgraph(slots, objective)
        }

        fn plan_info_gain_probe(
            &self,
            slots: &[CardSlot],
        ) -> Result<ProbePlan<Self::Move>, ReconError> {
            plan_info_gain_probe(slots)
        }

        fn execute_probe_and_observe(
            &mut self,
            probe: &[Self::Move],
        ) -> Result<Option<Vec<(SlotId, Card)>>, ReconError> {
            self.probes_run += 1;
            let report = verify_move_sequence(&self.truth, probe).map_err(|e| {
                ReconError::SceneVerification {
                    detail: e.to_string(),
                }
            })?;
            let _ = report;
            // Observe strictly `slot 0` (never exposing or reporting `slot 2`).
            let CardSlot::Known(peak0) = self.truth[0] else {
                return Err(ReconError::VisionClassificationFailed { slot: SlotId(0) });
            };
            Ok(Some(vec![(SlotId(0), peak0)]))
        }

        fn undo_all_and_hash(&mut self) -> Result<u64, ReconError> {
            Ok(self.root_hash)
        }

        fn best_partial_route(
            &self,
            slots: &[CardSlot],
        ) -> Result<BestPartialRoute<Self::Move>, ReconError> {
            best_partial_route(slots)
        }
    }

    let truth = build_stock_gated_52_card_deal();
    let mut dom = PenultimateDomain {
        truth,
        probes_run: 0,
        root_hash: 42,
    };
    let outcome =
        run_epistemic_recon(&mut dom, Objective::ClearBoard, true, 4).expect("complement RECON");
    let ReconOutcome::SolvedFromRoot { route, probes_used } = outcome else {
        panic!("expected SolvedFromRoot, got {outcome:?}");
    };
    assert_eq!(probes_used, 1);
    assert_eq!(dom.probes_run, 1);
    verify_objective_route(&truth, &route, Objective::ClearBoard).expect("valid full route");
}

#[test]
fn json_codec_enforces_size_keys_duplicates_and_atomic_no_overwrite_publication() {
    let full = build_cyclic_52_card_deal();
    let route = solve_known_subgraph(&full, Objective::ClearBoard).expect("route");

    // 1. All 3 Objective variants round-trip identically.
    for objective in [
        Objective::ClearBoard,
        Objective::ClearRankQuota {
            rank: 0,
            remaining: 4,
        },
        Objective::ReachScoreDelta {
            points_needed: 4_600,
        },
    ] {
        let doc = TriPeaksDealDocument {
            provenance: DealProvenance::RestoredAtRoot,
            probes_used: 3,
            objective,
            slots: full,
            planned_route: Some(route.clone()),
        };
        let bytes = to_vec(&doc).expect("valid encode");
        assert!(bytes.ends_with(b"\n"));
        assert_eq!(from_slice(&bytes).expect("roundtrip"), doc);
        assert_eq!(read_from(Cursor::new(&bytes)).expect("read_from"), doc);
    }

    // 2. Oversized `> 65,536` input refused.
    let oversized = vec![b' '; MAX_DOCUMENT_BYTES + 1];
    assert!(matches!(
        from_slice(&oversized),
        Err(JsonError::InputTooLarge)
    ));
    assert!(matches!(
        read_from(Cursor::new(&oversized)),
        Err(JsonError::InputTooLarge)
    ));

    // 3. Duplicate key (both plain & Unicode escape), trailing garbage, unknown key, and invalid slot refused.
    let sample = String::from_utf8(
        to_vec(&TriPeaksDealDocument {
            provenance: DealProvenance::RestoredAtRoot,
            probes_used: 0,
            objective: Objective::ClearBoard,
            slots: full,
            planned_route: None,
        })
        .expect("sample bytes"),
    )
    .expect("utf8");

    let dup_plain = sample.replacen(
        "\"probes_used\": 0,",
        "\"probes_used\": 0,\n  \"probes_used\": 1,",
        1,
    );
    assert!(matches!(
        from_slice(dup_plain.as_bytes()),
        Err(JsonError::DuplicateKey { key }) if key == "probes_used"
    ));

    let dup_escaped = sample.replacen(
        "\"probes_used\": 0,",
        "\"probes_used\": 0,\n  \"probes\\u005fused\": 1,",
        1,
    );
    assert!(matches!(
        from_slice(dup_escaped.as_bytes()),
        Err(JsonError::DuplicateKey { key }) if key == "probes_used"
    ));

    let trailing = format!("{sample} 42");
    assert!(matches!(
        from_slice(trailing.as_bytes()),
        Err(JsonError::Malformed { .. })
    ));

    let unknown_field = sample.replacen(
        "\"probes_used\": 0,",
        "\"extra\": true,\n  \"probes_used\": 0,",
        1,
    );
    assert!(matches!(
        from_slice(unknown_field.as_bytes()),
        Err(JsonError::UnexpectedField { field }) if field == "extra"
    ));

    // 4. Atomic no-overwrite same-directory publication + collision inspection.
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let test_dir: PathBuf = std::env::temp_dir().join(format!(
        "qmp-tripeaks-json-test-{}-{stamp}",
        std::process::id()
    ));
    fs::create_dir_all(&test_dir).expect("create test temp dir");

    let doc = TriPeaksDealDocument {
        provenance: DealProvenance::RestoredAtRoot,
        probes_used: 1,
        objective: Objective::ClearBoard,
        slots: full,
        planned_route: Some(route),
    };
    let dest = test_dir.join("deal.json");
    publish_to_path(&doc, &dest).expect("first publish succeeds");
    assert_eq!(
        fs::read(&dest).expect("read back"),
        to_vec(&doc).expect("bytes")
    );

    // Verify no leftover temp files remain in `test_dir`.
    let entries: Vec<_> = fs::read_dir(&test_dir)
        .expect("readdir")
        .map(|e| e.expect("entry").file_name())
        .collect();
    assert_eq!(entries.len(), 1);

    // Same content collision reports `MatchingBytesPresent` without overwriting.
    let err_same = publish_to_path(&doc, &dest).expect_err("no-overwrite refuses existing");
    assert!(matches!(
        err_same,
        PublishDealError::LinkFailed {
            destination_state: DestinationState::MatchingBytesPresent,
            ..
        }
    ));

    // Different existing bytes stay intact and report `PreExistingDifferentBytes`.
    let other_doc = TriPeaksDealDocument {
        provenance: DealProvenance::RestoredAtRoot,
        probes_used: 99,
        objective: Objective::ClearBoard,
        slots: full,
        planned_route: None,
    };
    let err_diff = publish_to_path(&other_doc, &dest).expect_err("collision refused");
    assert!(matches!(
        err_diff,
        PublishDealError::LinkFailed {
            destination_state: DestinationState::PreExistingDifferentBytes,
            ..
        }
    ));
    assert_eq!(
        fs::read(&dest).expect("original untouched"),
        to_vec(&doc).expect("orig")
    );

    // Bad destination path refused before touching filesystem.
    assert!(matches!(
        publish_to_path(&doc, Path::new("")),
        Err(PublishDealError::InvalidPath)
    ));

    fs::remove_dir_all(&test_dir).expect("cleanup temp dir");
}

#[test]
fn search_respects_cancellation_and_stop_limits() {
    let truth = build_cyclic_52_card_deal();
    let cancel = AtomicBool::new(true);
    let outcome = solve_known_subgraph_with_control(
        &truth,
        Objective::ClearBoard,
        DEFAULT_SEARCH_LIMITS,
        xsar::search::SearchOptions::default(),
        &cancel,
        None,
        None,
    );
    assert!(matches!(
        outcome,
        SearchOutcome::Stopped {
            reason: SearchStop::Cancelled,
            ..
        }
    ));
    cancel.store(false, Ordering::Relaxed);
    assert!(matches!(
        plan_info_gain_probe(&truth),
        Ok(ProbePlan::Exhausted)
    )); // all 52 already Known -> 0 unknowns left
}

/// Build a `29`-move `ClearBoard` deal where `Peak0+Peak1_prefix -> Draw -> Peak2+Peak1_suffix`
/// splits streaks `(14, 14)` (`40_700` pts) while `Draw -> Peak2 -> Peak0 -> Peak1` runs `28` unbroken (`79_900` pts).
fn build_two_streak_29_move_deal() -> [CardSlot; DEAL_SLOT_COUNT] {
    // Disjoint topological groups matching Peak 0 -> Peak 1 prefix -> Draw -> Peak 2 -> Peak 1 suffix:
    // - Peak 0 (`10`): `[18..=21, 9..=11, 3, 4, 0]` (`2 -> 1 -> 0 -> 12 -> 11 -> 10 -> 9 -> 8 -> 9 -> 8`)
    // - Peak 1 prefix (`4`): `[22, 23, 12, 13]` (`9 -> 10 -> 11 -> 12`)
    // - Peak 2 (`10`): `[24..=27, 15..=17, 7, 8, 2]` (`6 -> 7 -> 6 -> 5 -> 4 -> 5 -> 4 -> 3 -> 2 -> 1`)
    // - Peak 1 suffix (`4`): `[5, 14, 6, 1]` (`0 -> 1 -> 2 -> 1`, covered by `24` via `14` until Peak 2 clears)
    let peak0: [usize; 10] = [18, 19, 20, 21, 9, 10, 11, 3, 4, 0];
    let peak0_ranks: [u8; 10] = [2, 1, 0, 12, 11, 10, 9, 8, 9, 8];
    let peak1_p: [usize; 4] = [22, 23, 12, 13];
    let peak1_p_ranks: [u8; 4] = [9, 10, 11, 12];
    let peak2: [usize; 10] = [24, 25, 26, 27, 15, 16, 17, 7, 8, 2];
    let peak2_ranks: [u8; 10] = [6, 7, 6, 5, 4, 5, 4, 3, 2, 1];
    let peak1_s: [usize; 4] = [5, 14, 6, 1];
    let peak1_s_ranks: [u8; 4] = [0, 1, 2, 1];

    let mut used = [false; 52];
    let mut suit_cursor = [0_u8; 13];
    let mut take_rank = |rank_idx: u8| -> Card {
        let s = suit_cursor[usize::from(rank_idx)];
        assert!(s < 4, "rank {rank_idx} exhausted");
        suit_cursor[usize::from(rank_idx)] = s + 1;
        let c = card(rank_idx, s);
        used[usize::from(c.0)] = true;
        c
    };

    let mut slots = [CardSlot::Unknown; DEAL_SLOT_COUNT];
    // Waste = Four (`3`) -> matches ONLY `peak0[0] = Three (2)` on Row 4.
    slots[usize::from(INITIAL_WASTE_SLOT.0)] = CardSlot::Known(take_rank(3));
    // Stock[0] = Six (`5`) -> matches `peak2[0] = Seven (6)`.
    slots[usize::from(STOCK_START_SLOT)] = CardSlot::Known(take_rank(5));

    for (indices, ranks) in [
        (&peak0[..], &peak0_ranks[..]),
        (&peak1_p[..], &peak1_p_ranks[..]),
        (&peak2[..], &peak2_ranks[..]),
        (&peak1_s[..], &peak1_s_ranks[..]),
    ] {
        for (&slot_idx, &rank_idx) in indices.iter().zip(ranks.iter()) {
            slots[slot_idx] = CardSlot::Known(take_rank(rank_idx));
        }
    }

    let mut stock_idx = usize::from(STOCK_START_SLOT) + 1;
    for (id, &taken) in used.iter().enumerate() {
        if !taken {
            slots[stock_idx] = CardSlot::Known(Card(id as u8));
            stock_idx += 1;
        }
    }
    assert_eq!(stock_idx, DEAL_SLOT_COUNT);
    validate_slot_table(&slots).expect("valid 2-streak deal");
    slots
}

/// Build an unsolvable deal (`ClearBoard` impossible) with partial rank (`Four`) and score gains.
fn build_unsolvable_partial_deal() -> [CardSlot; DEAL_SLOT_COUNT] {
    // Allow clearing `Row 4[0..=2]` (`Three, Four, Five` -> hits rank `3 = Four` and streak 3!)
    // while trapping remaining tableau (`0` adjacency to Waste/Stock or rest of tableau).
    let mut used = [false; 52];
    let mut suit_cursor = [0_u8; 13];
    let mut take_rank = |rank_idx: u8| -> Card {
        let s = suit_cursor[usize::from(rank_idx)];
        assert!(s < 4, "rank {rank_idx} exhausted");
        suit_cursor[usize::from(rank_idx)] = s + 1;
        let c = card(rank_idx, s);
        used[usize::from(c.0)] = true;
        c
    };

    let mut fixed = [CardSlot::Unknown; DEAL_SLOT_COUNT];
    // Waste = `Two` (`1`). Row 4 slots `18, 19, 20` = `[Three (2), Four (3), Five (4)]`.
    fixed[usize::from(INITIAL_WASTE_SLOT.0)] = CardSlot::Known(take_rank(1));
    fixed[18] = CardSlot::Known(take_rank(2));
    fixed[19] = CardSlot::Known(take_rank(3));
    fixed[20] = CardSlot::Known(take_rank(4));
    // Trapped Row 4 (`slots 21..=27`) and covered `9, 10` keep distance >= 2 from Waste/Stock `{0..=6}`.
    for (slot_idx, rank_idx) in (21..28).zip([8_u8, 8, 8, 8, 10, 10, 10]) {
        fixed[slot_idx] = CardSlot::Known(take_rank(rank_idx));
    }
    fixed[9] = CardSlot::Known(take_rank(9));
    fixed[10] = CardSlot::Known(take_rank(9));
    // Fill `Waste+Stock` (`29..52`) from remaining ranks `{1,2,3,4,5,6}` (`20` cards) + `3` Aces (`0`):
    let mut stock_idx = usize::from(STOCK_START_SLOT);
    for r in [
        0_u8, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 6,
    ] {
        fixed[stock_idx] = CardSlot::Known(take_rank(r));
        stock_idx += 1;
    }
    assert_eq!(stock_idx, DEAL_SLOT_COUNT);
    let mut rem_iter = (0_u8..52).filter(|&id| !used[usize::from(id)]);
    for slot_idx in (0..9).chain(11..18) {
        let id = rem_iter.next().expect("remaining trapped tableau card");
        fixed[slot_idx] = CardSlot::Known(Card(id));
    }
    assert!(rem_iter.next().is_none());
    validate_slot_table(&fixed).expect("valid partial deal");
    fixed
}

#[test]
fn plan_deal_route_handles_multi_board_clear_escalation_and_partial_fallbacks() {
    // 1. Direct quota within one board -> SolvedObjective.
    let cyclic = build_cyclic_52_card_deal();
    let direct = plan_deal_route(
        &cyclic,
        Objective::ClearRankQuota {
            rank: 0,
            remaining: 2,
        },
    )
    .expect("direct plan");
    assert!(matches!(
        direct,
        TriPeaksPlanOutcome::SolvedObjective { .. }
    ));

    // 2. Multi-board ClearRankQuota exceeding 1 deck (remaining = 8) on solvable board -> EscalatedClearBoard (28/28 cleared).
    let escalated_rank = plan_deal_route(
        &cyclic,
        Objective::ClearRankQuota {
            rank: 3,
            remaining: 8,
        },
    )
    .expect("escalated rank plan");
    match escalated_rank {
        TriPeaksPlanOutcome::EscalatedClearBoard {
            actions, replay, ..
        } => {
            assert_eq!(replay.final_position.tableau_mask, 0);
            verify_objective_route(&cyclic, &actions, Objective::ClearBoard)
                .expect("escalated full board clear");
        }
        other => panic!("expected EscalatedClearBoard for rank quota > 1 deck, got {other:?}"),
    }

    // 3. Multi-board ReachScoreDelta exceeding 1 deck (90_000) chooses max-score ClearBoard route (79_900 > 40_700).
    let two_streak = build_two_streak_29_move_deal();
    let fast_clear =
        solve_known_subgraph(&two_streak, Objective::ClearBoard).expect("fast 29-move clear");
    let fast_replay = verify_objective_route(&two_streak, &fast_clear, Objective::ClearBoard)
        .expect("fast replay");
    assert_eq!(fast_replay.final_position.score_delta, 40_700);

    let escalated_score = plan_deal_route(
        &two_streak,
        Objective::ReachScoreDelta {
            points_needed: 90_000,
        },
    )
    .expect("escalated score plan");
    match escalated_score {
        TriPeaksPlanOutcome::EscalatedClearBoard {
            actions, replay, ..
        } => {
            assert_eq!(replay.final_position.tableau_mask, 0);
            assert_eq!(replay.final_position.score_delta, 79_900);
            verify_objective_route(&two_streak, &actions, Objective::ClearBoard)
                .expect("verified max-score clear");
        }
        other => panic!("expected EscalatedClearBoard for score target > 1 deck, got {other:?}"),
    }

    // 4. Unsolvable board across all 3 objective types:
    //    - ClearBoard: empty route (`0` board-clear progress), `max_tableau_cleared == 3`
    //    - ClearRankQuota(Four=3): banks tapping `[18, 19, 20]` (`1` Four cleared)
    //    - ClearRankQuota(Queen=11): `0` Queens reachable -> empty route
    //    - ReachScoreDelta(5_000): banks `100 + 300 + 500 = 900` score gain
    let partial = build_unsolvable_partial_deal();
    match plan_deal_route(&partial, Objective::ClearBoard).expect("clear partial") {
        TriPeaksPlanOutcome::UnsolvablePartial {
            actions,
            max_tableau_cleared,
            proof,
            unknown_slots,
            ..
        } => {
            assert!(actions.is_empty(), "ClearBoard partial stays at T=0");
            assert_eq!(max_tableau_cleared, 3);
            assert_eq!(proof, PartialProof::Exhaustive);
            assert_eq!(unknown_slots, 0);
        }
        other => panic!("expected UnsolvablePartial for ClearBoard dead end, got {other:?}"),
    }
    match plan_deal_route(
        &partial,
        Objective::ClearRankQuota {
            rank: 3,
            remaining: 2,
        },
    )
    .expect("rank partial")
    {
        TriPeaksPlanOutcome::UnsolvablePartial {
            actions,
            replay,
            max_tableau_cleared,
            proof,
            unknown_slots,
            ..
        } => {
            assert!(!actions.is_empty());
            assert_eq!(replay.final_position.cleared_by_rank[3], 1);
            assert_eq!(max_tableau_cleared, 3);
            assert_eq!(proof, PartialProof::Exhaustive);
            assert_eq!(unknown_slots, 0);
        }
        other => panic!("expected UnsolvablePartial with rank progress, got {other:?}"),
    }
    match plan_deal_route(
        &partial,
        Objective::ClearRankQuota {
            rank: 11,
            remaining: 2,
        },
    )
    .expect("zero rank partial")
    {
        TriPeaksPlanOutcome::UnsolvablePartial {
            actions,
            proof,
            unknown_slots,
            ..
        } => {
            assert!(
                actions.is_empty(),
                "zero target-rank gain must keep board at T=0"
            );
            assert_eq!(proof, PartialProof::Exhaustive);
            assert_eq!(unknown_slots, 0);
        }
        other => panic!("expected empty UnsolvablePartial for unreachable rank, got {other:?}"),
    }
    match plan_deal_route(
        &partial,
        Objective::ReachScoreDelta {
            points_needed: 5_000,
        },
    )
    .expect("score partial")
    {
        TriPeaksPlanOutcome::UnsolvablePartial {
            actions,
            replay,
            proof,
            unknown_slots,
            ..
        } => {
            assert_eq!(actions.len(), 3);
            assert_eq!(replay.final_position.score_delta, 900);
            assert_eq!(proof, PartialProof::Exhaustive);
            assert_eq!(unknown_slots, 0);
        }
        other => panic!("expected UnsolvablePartial with score progress, got {other:?}"),
    }
}

#[test]
fn probe_planning_distinguishes_bounded_from_exhausted_and_propagates_faults() {
    let masked = hide_rows_1_to_3(&build_cyclic_52_card_deal());
    let cancel = AtomicBool::new(false);

    // 1. Tight expansion bound stops before clearing any Row-4 pair to uncover Row 3 (`score == 0`).
    let tight_limits = SearchLimits {
        max_states: 1_000,
        max_expansions: 1,
        max_memory_bytes: DEFAULT_SEARCH_LIMITS.max_memory_bytes,
        max_duration: Duration::from_secs(5),
    };
    assert!(matches!(
        plan_info_gain_probe_with_control(&masked, tight_limits, &cancel, None, None),
        Ok(ProbePlan::Bounded(SearchStop::ExpansionLimit))
    ));

    // 2. Zero duration stops before expansion begins and returns Bounded(TimeLimit), not Exhausted.
    let zero_duration = SearchLimits {
        max_duration: Duration::ZERO,
        ..DEFAULT_SEARCH_LIMITS
    };
    assert!(matches!(
        plan_info_gain_probe_with_control(&masked, zero_duration, &cancel, None, None),
        Ok(ProbePlan::Bounded(SearchStop::TimeLimit))
    ));

    // 3. Cancellation and invalid slot tables/unknown waste propagate errors rather than Exhausted.
    cancel.store(true, Ordering::Release);
    assert!(matches!(
        plan_info_gain_probe_with_control(&masked, DEFAULT_SEARCH_LIMITS, &cancel, None, None),
        Err(ReconError::Cancelled)
    ));
    cancel.store(false, Ordering::Release);

    let mut no_waste = masked;
    no_waste[usize::from(INITIAL_WASTE_SLOT.0)] = CardSlot::Unknown;
    assert!(matches!(
        plan_info_gain_probe(&no_waste),
        Err(ReconError::SearchFailed {
            context: "probe planning",
            ..
        })
    ));
    let mut no_waste_all_known_parents = build_cyclic_52_card_deal();
    no_waste_all_known_parents[usize::from(INITIAL_WASTE_SLOT.0)] = CardSlot::Unknown;
    assert!(matches!(
        plan_info_gain_probe(&no_waste_all_known_parents),
        Err(ReconError::SearchFailed {
            context: "probe planning",
            ..
        })
    ));
    assert!(matches!(
        plan_info_gain_probe(&masked[..10]),
        Err(ReconError::SearchFailed {
            context: "probe planning",
            ..
        })
    ));

    // 4. Tight search limits through RECON produce BoundedBestPartial while keeping Unknown slots intact.
    /// Wrapper domain delegating search calls with tight bounds to simulate a bounded probe timeout.
    struct BoundedProbeDomain {
        /// Underlying deterministic TriPeaks test domain.
        inner: SimulatedTriPeaksDomain,
        /// Tight search budget causing `ProbePlan::Bounded`.
        limits: SearchLimits,
        /// Cooperative cancellation token.
        cancel: AtomicBool,
    }
    impl ReconDomain for BoundedProbeDomain {
        type Move = TriPeaksMove;

        fn initialize_and_sweep_stock(&mut self) -> Result<(Vec<CardSlot>, u64), ReconError> {
            self.inner.initialize_and_sweep_stock()
        }

        fn solve_known_subgraph(
            &self,
            slots: &[CardSlot],
            objective: Objective,
        ) -> Option<Vec<Self::Move>> {
            self.inner.solve_known_subgraph(slots, objective)
        }

        fn plan_info_gain_probe(
            &self,
            slots: &[CardSlot],
        ) -> Result<ProbePlan<Self::Move>, ReconError> {
            plan_info_gain_probe_with_control(slots, self.limits, &self.cancel, None, None)
        }

        fn execute_probe_and_observe(
            &mut self,
            probe: &[Self::Move],
        ) -> Result<Option<Vec<(SlotId, Card)>>, ReconError> {
            self.inner.execute_probe_and_observe(probe)
        }

        fn undo_all_and_hash(&mut self) -> Result<u64, ReconError> {
            self.inner.undo_all_and_hash()
        }

        fn best_partial_route(
            &self,
            slots: &[CardSlot],
        ) -> Result<BestPartialRoute<Self::Move>, ReconError> {
            best_partial_route_with_control(slots, self.limits, &self.cancel, None, None)
        }
    }

    let mut bounded_dom = BoundedProbeDomain {
        inner: SimulatedTriPeaksDomain::new(build_cyclic_52_card_deal(), Objective::ClearBoard),
        limits: tight_limits,
        cancel: AtomicBool::new(false),
    };
    let recon_out = run_epistemic_recon(&mut bounded_dom, Objective::ClearBoard, true, 5)
        .expect("bounded probe completes with BoundedBestPartial");
    assert!(matches!(
        recon_out,
        ReconOutcome::BoundedBestPartial {
            unknown_slots: 18,
            unreachable_slots: 0,
            probe_stop: SearchStop::ExpansionLimit,
            proof: PartialProof::Bounded(SearchStop::ExpansionLimit),
            ..
        }
    ));
}

#[test]
fn best_partial_and_plan_deal_route_preserve_partial_proof_unknowns_and_non_cancel_stops() {
    let masked = hide_rows_1_to_3(&build_cyclic_52_card_deal());
    let cancel = AtomicBool::new(false);
    let tight_limits = SearchLimits {
        max_states: 1_000,
        max_expansions: 1,
        max_memory_bytes: DEFAULT_SEARCH_LIMITS.max_memory_bytes,
        max_duration: Duration::from_secs(5),
    };

    let bounded_partial =
        best_partial_route_with_control(&masked, tight_limits, &cancel, None, None)
            .expect("bounded best partial returns prefix with Bounded proof");
    assert_eq!(
        bounded_partial.proof,
        PartialProof::Bounded(SearchStop::ExpansionLimit)
    );
    assert!(bounded_partial.has_unknown_slots);

    // Exhaustive `plan_deal_route` on an unsolvable deal WITH Unknown slots carries `unknown_slots > 0`.
    let partial_unknown = hide_rows_1_to_3(&build_unsolvable_partial_deal());
    match plan_deal_route(&partial_unknown, Objective::ClearBoard).expect("partial with unknowns") {
        TriPeaksPlanOutcome::UnsolvablePartial {
            proof,
            unknown_slots,
            ..
        } => {
            assert_eq!(proof, PartialProof::Exhaustive);
            assert_eq!(unknown_slots, 18);
        }
        other => panic!("expected UnsolvablePartial with unknown_slots=18, got {other:?}"),
    }

    // Tight limits on `plan_deal_route_with_control` surface `Stopped` (non-cancel) rather than false exhaustion.
    let zero_duration = SearchLimits {
        max_duration: Duration::ZERO,
        ..DEFAULT_SEARCH_LIMITS
    };
    let stopped = plan_deal_route_with_control(
        &partial_unknown,
        Objective::ClearBoard,
        zero_duration,
        &cancel,
        None,
        None,
    )
    .expect("zero-duration search returns Stopped outcome");
    assert!(matches!(
        stopped,
        TriPeaksPlanOutcome::Stopped {
            reason: SearchStop::TimeLimit,
            ..
        }
    ));
}

/// Decode a short rank+suit string (`"QS"`, `"10H"`, etc.) into canonical `CardSlot::Known`.
fn slot_from_code(code: &str) -> CardSlot {
    let (rank_str, suit_str) = code.split_at(code.len() - 1);
    let rank_idx: u8 = match rank_str {
        "A" => 0,
        "2" => 1,
        "3" => 2,
        "4" => 3,
        "5" => 4,
        "6" => 5,
        "7" => 6,
        "8" => 7,
        "9" => 8,
        "10" => 9,
        "J" => 10,
        "Q" => 11,
        "K" => 12,
        other => panic!("unexpected rank code {other}"),
    };
    let suit = match suit_str {
        "C" => Suit::Clubs,
        "D" => Suit::Diamonds,
        "H" => Suit::Hearts,
        "S" => Suit::Spades,
        other => panic!("unexpected suit code {other}"),
    };
    let rank = Rank::from_value(rank_idx + 1).expect("rank 1..=13");
    let canonical = Card::try_from(XsarCard::new(rank, suit)).expect("canonical card id < 52");
    CardSlot::Known(canonical)
}

#[test]
fn queens_challenge_671k_state_deal_escalates_to_full_clear_under_10gb_limits() {
    assert_eq!(
        DEFAULT_SEARCH_LIMITS.max_memory_bytes,
        10_usize * 1024 * 1024 * 1024
    );
    assert_eq!(DEFAULT_SEARCH_LIMITS.max_states, 20_000_000);
    assert_eq!(DEFAULT_SEARCH_LIMITS.max_expansions, 20_000_000);
    assert_eq!(DEFAULT_SEARCH_LIMITS.max_duration, Duration::from_secs(600));

    let codes = [
        "QS", "4S", "7S", "AH", "JH", "AC", "3D", "4H", "7D", "QC", "8C", "4D", "9H", "10S", "KC",
        "QH", "6S", "QD", "AD", "3C", "5C", "2C", "10D", "5S", "9D", "7C", "2S", "JS", "KH", "8S",
        "6C", "7H", "10C", "8H", "2H", "JC", "AS", "KD", "3H", "3S", "9S", "JD", "4C", "2D", "6H",
        "9C", "KS", "6D", "8D", "5H", "10H", "5D",
    ];
    let slots: [CardSlot; DEAL_SLOT_COUNT] = codes.map(slot_from_code);
    let objective = Objective::ClearRankQuota {
        rank: 11,
        remaining: 10,
    };

    let outcome = plan_deal_route(&slots, objective)
        .expect("671k-state Queens Challenge deal solves cleanly under 10 GiB policy");
    let TriPeaksPlanOutcome::EscalatedClearBoard {
        actions, replay, ..
    } = outcome
    else {
        panic!("expected EscalatedClearBoard on live 671k-state Queens deal, got {outcome:?}");
    };
    assert_eq!(replay.final_position.tableau_mask, 0);
    verify_objective_route(&slots, &actions, Objective::ClearBoard)
        .expect("escalated ClearBoard route replays cleanly");

    // Non-time resource bounds on Pass 2 (`ClearBoard` escalation) fall through to Pass 3 with `PartialProof::Bounded`.
    let cancel = AtomicBool::new(false);
    let tight_limits = SearchLimits {
        max_states: 1_000,
        max_expansions: 50,
        max_memory_bytes: DEFAULT_SEARCH_LIMITS.max_memory_bytes,
        max_duration: Duration::from_secs(5),
    };
    let fallthrough =
        plan_deal_route_with_control(&slots, objective, tight_limits, &cancel, None, None)
            .expect("Pass 2 ExpansionLimit falls through to Pass 3 bounded partial");
    assert!(matches!(
        fallthrough,
        TriPeaksPlanOutcome::UnsolvablePartial {
            proof: PartialProof::Bounded(SearchStop::ExpansionLimit),
            unknown_slots: 0,
            ..
        }
    ));
}

#[test]
fn unseen_rank_probability_breaks_probe_ties_and_continuation_chains_from_mid_board() {
    let mut slots = [CardSlot::Unknown; DEAL_SLOT_COUNT];
    // Fix all 4 Kings (12) and all 4 Twos (1) in `Known` -> `unseen_adjacent(Ace) == 0`.
    slots[0] = CardSlot::Known(card(12, 0));
    slots[1] = CardSlot::Known(card(12, 1));
    slots[2] = CardSlot::Known(card(12, 2));
    slots[3] = CardSlot::Known(card(12, 3));
    slots[4] = CardSlot::Known(card(1, 0));
    slots[5] = CardSlot::Known(card(1, 1));
    slots[6] = CardSlot::Known(card(1, 2));

    // Fork A (`18=2, 19=A` uncovering `slot 9`) ends at `Ace (0)` with zero unseen adjacent ranks.
    slots[18] = CardSlot::Known(card(1, 3));
    slots[19] = CardSlot::Known(card(0, 0));

    // Fork B (`25=4, 26=5` uncovering `slot 16`) ends at `Five (4)` with 6 unseen adjacent cards (`4`s and `6`s).
    slots[25] = CardSlot::Known(card(3, 0));
    slots[26] = CardSlot::Known(card(4, 0));
    slots[27] = CardSlot::Known(card(6, 0));
    slots[17] = CardSlot::Known(card(5, 0));

    // Non-bridging Row 4 spacers (`Nine` and `Jack`).
    slots[20] = CardSlot::Known(card(8, 0));
    slots[21] = CardSlot::Known(card(8, 1));
    slots[22] = CardSlot::Known(card(10, 0));
    slots[23] = CardSlot::Known(card(10, 1));
    slots[24] = CardSlot::Known(card(8, 2));

    // Initial Waste = Three (`rank 2`).
    slots[usize::from(INITIAL_WASTE_SLOT.0)] = CardSlot::Known(card(2, 0));

    let ProbePlan::Found(initial_probe) =
        plan_info_gain_probe(&slots).expect("tie-break selects highest unseen continuation probe")
    else {
        panic!("expected ProbePlan::Found for Fork A vs Fork B tie");
    };
    assert_eq!(
        initial_probe,
        vec![
            TriPeaksMove::TapTableau { row: 4, column: 8 },
            TriPeaksMove::TapTableau { row: 4, column: 9 },
        ]
    );

    // Simulate online flip: `slot 16` reveals `Six of Diamonds` (`rank 5`); mid-board continuation chains up to `slot 8` (`Unknown`)!
    slots[16] = CardSlot::Known(card(5, 1));
    let cancel = AtomicBool::new(false);
    let ProbePlan::Found(cont) =
        search::plan_info_gain_continuation(search::ProbeContinuationInput {
            slots: &slots,
            executed_prefix: &initial_probe,
            limits: DEFAULT_SEARCH_LIMITS,
            cancel: &cancel,
        })
        .expect("mid-board continuation succeeds cleanly")
    else {
        panic!("expected continuation probe after slot 16 flips Six");
    };
    assert_eq!(
        cont,
        vec![
            TriPeaksMove::TapTableau { row: 3, column: 8 },
            TriPeaksMove::TapTableau { row: 4, column: 10 },
            TriPeaksMove::TapTableau { row: 3, column: 9 },
        ]
    );
}


#[test]
fn tripeaks_350_plan_deal_route_controlled_yield_best_and_cancel_precedence() {
    use xsar::games::cards::Suit;
    let waste_card = Card::try_from(XsarCard::new(Rank::Ace, Suit::Clubs)).expect("card id");
    let tap_1 = Card::try_from(XsarCard::new(
        Rank::from_value(2).expect("rank 2"),
        Suit::Clubs,
    ))
    .expect("card id");
    let tap_2 = Card::try_from(XsarCard::new(
        Rank::from_value(3).expect("rank 3"),
        Suit::Diamonds,
    ))
    .expect("card id");
    let mut deck = Vec::with_capacity(52);
    for suit in [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades] {
        for r in 1_u8..=13 {
            let c = Card::try_from(XsarCard::new(
                Rank::from_value(r).expect("rank 1..=13"),
                suit,
            ))
            .expect("card id");
            if c != waste_card && c != tap_1 && c != tap_2 {
                deck.push(c);
            }
        }
    }
    let mut slots = [CardSlot::Unknown; DEAL_SLOT_COUNT];
    slots[usize::from(INITIAL_WASTE_SLOT.0)] = CardSlot::Known(waste_card);
    slots[18] = CardSlot::Known(tap_1);
    slots[19] = CardSlot::Known(tap_2);
    let mut deck_iter = deck.into_iter();
    for entry in &mut slots {
        if matches!(entry, CardSlot::Unknown) {
            *entry = CardSlot::Known(deck_iter.next().expect("full 52-card deck"));
        }
    }

    let cancel = AtomicBool::new(false);
    let yield_best = AtomicBool::new(false);
    let polls = std::cell::Cell::new(0usize);
    let checkpoint = || {
        let n = polls.get() + 1;
        polls.set(n);
        if n >= 300 {
            yield_best.store(true, std::sync::atomic::Ordering::Release);
        }
        Duration::ZERO
    };
    let outcome = super::plan_deal_route_controlled(
        &slots,
        Objective::ClearBoard,
        DEFAULT_SEARCH_LIMITS,
        xsar::search::BestEffortControl {
            cancel: &cancel,
            yield_best: Some(&yield_best),
            progress: None,
            checkpoint: Some(&checkpoint),
        },
    )
    .expect("controlled TriPeaks planning succeeds cleanly");
    if let TriPeaksPlanOutcome::UnsolvablePartial {
        actions,
        proof,
        max_tableau_cleared,
        ..
    } = outcome
    {
        assert_eq!(proof, PartialProof::Bounded(SearchStop::Interrupted));
        assert!(max_tableau_cleared > 0);
        assert!(!actions.is_empty());
        verify_move_sequence(&slots, &actions).expect("yielded TriPeaks prefix replays cleanly");
    } else if !matches!(outcome, TriPeaksPlanOutcome::SolvedObjective { .. }) {
        panic!("expected SolvedObjective or Interrupted Partial, got {outcome:?}");
    }

    // Hard cancel must supersede yield_best.
    cancel.store(true, std::sync::atomic::Ordering::Release);
    yield_best.store(true, std::sync::atomic::Ordering::Release);
    let cancelled = super::plan_deal_route_controlled(
        &slots,
        Objective::ClearBoard,
        DEFAULT_SEARCH_LIMITS,
        xsar::search::BestEffortControl {
            cancel: &cancel,
            yield_best: Some(&yield_best),
            progress: None,
            checkpoint: None,
        },
    )
    .expect("cancelled planning returns cleanly");
    assert!(matches!(
        cancelled,
        TriPeaksPlanOutcome::Stopped {
            reason: SearchStop::Cancelled,
            ..
        }
    ));
}

#[test]
fn tripeaks_350_multi_pass_score_monotonicity_upgrade_yield_and_prior_bound_preservation() {
    use xsar::games::cards::Suit;

    // 1. Build a 52-card deal where tableau + stock form consecutive adjacent ranks so ClearBoard
    // solves swiftly in Pass 2 and ReachScoreDelta enters `maybe_upgrade_clear_score`.
    let card_at = |rank_idx: u8, suit_idx: usize| {
        let suit = [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades][suit_idx];
        Card::try_from(XsarCard::new(
            Rank::from_value(rank_idx + 1).expect("rank 1..=13"),
            suit,
        ))
        .expect("card id")
    };
    let mut used = [false; 52];
    let mut take = |rank_idx: u8| -> Card {
        for s in 0..4 {
            let idx = usize::from(rank_idx) * 4 + s;
            if !used[idx] {
                used[idx] = true;
                return card_at(rank_idx, s);
            }
        }
        panic!("rank exhausted");
    };
    let mut clear_slots = [CardSlot::Unknown; DEAL_SLOT_COUNT];
    clear_slots[usize::from(INITIAL_WASTE_SLOT.0)] = CardSlot::Known(take(0)); // Ace
    // Clear bottom-up: row 3 (slot 27..=18) -> row 2 (17..=9) -> row 1 (8..=3) -> row 0 (2..=0).
    // Adjacent rank ramp keeps each rank count <= 3 while ensuring an instant 28-card ClearBoard.
    let ranks_28 = [
        1_u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 1, 2, 3, 4,
    ];
    for (step, slot_idx) in (0..28).rev().enumerate() {
        clear_slots[slot_idx] = CardSlot::Known(take(ranks_28[step]));
    }
    for entry in &mut clear_slots {
        if matches!(entry, CardSlot::Unknown) {
            let next_idx = used
                .iter()
                .position(|&u| !u)
                .expect("remaining unique card");
            used[next_idx] = true;
            *entry = CardSlot::Known(card_at((next_idx / 4) as u8, next_idx % 4));
        }
    }

    // Exercise `maybe_upgrade_clear_score` with `yield_best` triggered once score_progress > 0 inside
    // upgrade probes; best_score must remain strictly positive and return `EscalatedClearBoard`.
    let cancel = AtomicBool::new(false);
    let yield_best = AtomicBool::new(false);
    let upgrade_scores = std::cell::RefCell::new(Vec::new());
    let probes = std::cell::Cell::new(0usize);
    let mut progress = |snap: xsar::search::BestEffortProgress| {
        let p = probes.get() + 1;
        probes.set(p);
        upgrade_scores.borrow_mut().push(snap.best_score);
        if p > 3 && snap.best_score > 0 {
            // Delay slightly so Pass 2 completes ClearBoard and enters `maybe_upgrade_clear_score`.
            if snap.search.stats.expanded > 10 {
                yield_best.store(true, std::sync::atomic::Ordering::Release);
            }
        }
    };
    let escalated = super::plan_deal_route_controlled(
        &clear_slots,
        Objective::ReachScoreDelta {
            points_needed: 500_000,
        },
        DEFAULT_SEARCH_LIMITS,
        xsar::search::BestEffortControl {
            cancel: &cancel,
            yield_best: Some(&yield_best),
            progress: Some(&mut progress),
            checkpoint: None,
        },
    )
    .expect("ReachScoreDelta upgrade interruption succeeds cleanly");
    assert!(
        matches!(
            &escalated,
            TriPeaksPlanOutcome::EscalatedClearBoard { .. }
                | TriPeaksPlanOutcome::SolvedObjective { .. }
        ) || matches!(
            &escalated,
            TriPeaksPlanOutcome::UnsolvablePartial {
                proof: PartialProof::Bounded(SearchStop::Interrupted),
                max_tableau_cleared: 1..=28,
                ..
            }
        ),
        "unexpected outcome: {escalated:?}"
    );
    let recorded = upgrade_scores.borrow();
    if let Some(first_pos) = recorded.iter().position(|&s| s > 0) {
        assert!(
            recorded[first_pos..].iter().all(|&s| s > 0),
            "live best_score must never dip back to 0 during upgrade probes: {recorded:?}"
        );
    }

    // 2. Directly trigger yield_best on Pass 2 via ReachScoreDelta where ClearBoard completes and
    // enters `maybe_upgrade_clear_score` before yield_best is raised inside probe #1's checkpoint.
    let cancel2 = AtomicBool::new(false);
    let yield2 = AtomicBool::new(false);
    let mut saw_positive_after_clear = Vec::new();
    let finished_count = std::cell::Cell::new(0usize);
    let mut obs2 = |snap: xsar::search::BestEffortProgress| {
        if finished_count.get() >= 2 {
            saw_positive_after_clear.push(snap.best_score);
        }
        if snap.search.finished {
            finished_count.set(finished_count.get() + 1);
        }
    };
    let ckpt2 = || {
        // Trigger Play once Pass 1 & Pass 2 finished reports and we are inside `maybe_upgrade_clear_score`.
        if finished_count.get() >= 2 {
            yield2.store(true, std::sync::atomic::Ordering::Release);
        }
        Duration::ZERO
    };
    let escalated2 = super::plan_deal_route_controlled(
        &clear_slots,
        Objective::ReachScoreDelta {
            points_needed: 500_000,
        },
        DEFAULT_SEARCH_LIMITS,
        xsar::search::BestEffortControl {
            cancel: &cancel2,
            yield_best: Some(&yield2),
            progress: Some(&mut obs2),
            checkpoint: Some(&ckpt2),
        },
    )
    .expect("upgrade probe Play succeeds");
    assert!(matches!(
        escalated2,
        TriPeaksPlanOutcome::EscalatedClearBoard { .. }
    ));
    assert!(
        saw_positive_after_clear.iter().all(|&s| s > 0),
        "maybe_upgrade_clear_score must not reset best_score to 0: {saw_positive_after_clear:?}"
    );

    // 3. Prior StateLimit in Pass 1 + ClearBoard in Pass 2 must NOT mask Interrupted in Pass 3a.
    let mut state_bound_slots = [CardSlot::Unknown; DEAL_SLOT_COUNT];
    let mut deck: Vec<Card> = (0_u8..13)
        .flat_map(|r| (0..4).map(move |s| card_at(r, s)))
        .collect();
    let pop_rank = |pool: &mut Vec<Card>, r: u8| {
        let p = pool
            .iter()
            .position(|c| (0..4).any(|s| *c == card_at(r, s)))
            .expect("rank present in 52-card deck");
        pool.remove(p)
    };
    state_bound_slots[usize::from(INITIAL_WASTE_SLOT.0)] = CardSlot::Known(pop_rank(&mut deck, 4)); // Five
    // Top peak 0 = Ace (rank 0) covered by non-adjacent cards; row 3 has a tappable Four (`rank 3`)
    // so depth >= 1 immediately scores positive cards while StateLimit=100 stops Pass 1 & Pass 2!
    state_bound_slots[0] = CardSlot::Known(pop_rank(&mut deck, 0));
    state_bound_slots[18] = CardSlot::Known(pop_rank(&mut deck, 3)); // Four (tappable from Five!)
    state_bound_slots[19] = CardSlot::Known(pop_rank(&mut deck, 2)); // Three
    state_bound_slots[20] = CardSlot::Known(pop_rank(&mut deck, 1)); // Two
    for entry in &mut state_bound_slots {
        if matches!(entry, CardSlot::Unknown) {
            *entry = CardSlot::Known(deck.remove(0));
        }
    }
    let cancel3 = AtomicBool::new(false);
    let yield3 = AtomicBool::new(false);
    let ckpt3_count = std::cell::Cell::new(0usize);
    let pass_finished3 = std::cell::Cell::new(0usize);
    let mut obs3 = |snap: xsar::search::BestEffortProgress| {
        if snap.search.finished {
            pass_finished3.set(pass_finished3.get() + 1);
        }
    };
    let ckpt3 = || {
        // After Pass 1 (ClearRankQuota) + Pass 2 (ClearBoard) hit StateLimit (2 finished passes),
        // wait until Pass 3a (`BestPartialProblem`) expands root + Draw/Tap successors (~20 polls),
        // long before hitting `max_states = 100`, then raise `yield3`.
        if pass_finished3.get() >= 2 {
            let n = ckpt3_count.get() + 1;
            ckpt3_count.set(n);
            if n >= 20 {
                yield3.store(true, std::sync::atomic::Ordering::Release);
            }
        }
        Duration::ZERO
    };
    let tight_limits = SearchLimits {
        max_states: 100,
        max_expansions: 100,
        ..DEFAULT_SEARCH_LIMITS
    };
    let yielded3 = super::plan_deal_route_controlled(
        &state_bound_slots,
        Objective::ClearRankQuota {
            rank: 0,
            remaining: 1,
        },
        tight_limits,
        xsar::search::BestEffortControl {
            cancel: &cancel3,
            yield_best: Some(&yield3),
            progress: Some(&mut obs3),
            checkpoint: Some(&ckpt3),
        },
    )
    .expect("Pass 3a yield under prior StateLimit succeeds cleanly");
    assert!(
        matches!(
            yielded3,
            TriPeaksPlanOutcome::UnsolvablePartial {
                proof: PartialProof::Bounded(SearchStop::Interrupted),
                max_tableau_cleared,
                ..
            } if max_tableau_cleared > 0
        ),
        "prior StateLimit must not mask Pass 3a Interrupted: {yielded3:?}"
    );
}

#[test]
fn tripeaks_350_multi_pass_play_preserves_superior_prior_pass_candidate() {
    use xsar::games::cards::Suit;

    let card_at = |rank_idx: u8, suit_idx: usize| {
        let suit = [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades][suit_idx];
        Card::try_from(XsarCard::new(
            Rank::from_value(rank_idx + 1).expect("rank 1..=13"),
            suit,
        ))
        .expect("card id")
    };
    let mut deck: Vec<Card> = (0_u8..13)
        .flat_map(|r| (0..4).map(move |s| card_at(r, s)))
        .collect();
    let pop_rank = |pool: &mut Vec<Card>, r: u8| {
        let p = pool
            .iter()
            .position(|c| (0..4).any(|s| *c == card_at(r, s)))
            .expect("rank present in 52-card deck");
        pool.remove(p)
    };

    // Construct an unsolvable 52-card deal where:
    // - Peak 0 (`slot 0`) holds the 4th Queen (`rank 11`), permanently blocked by Sevens (`9..=12`)
    //   whose only partners (`6`, `8`) reside in `1..=8`.
    // - `slot 18` holds a non-target `Ten` (`rank 9`), tappable immediately from Waste `Jack` (`28`).
    // - `slots 19..=23` hold `Q, K, Q, J, Q`, allowing `3` Queens to be cleared cleanly in Pass 1
    //   (`ClearRankQuota { rank: 11, remaining: 4 }`), and also yielding `>= 900` streak points.
    // - `slots 24..=27` hold `5, 5, 5, 5`, blocked from `Stock` because `4`s live in `14..=17`.
    let mut slots = [CardSlot::Unknown; DEAL_SLOT_COUNT];
    slots[0] = CardSlot::Known(pop_rank(&mut deck, 11)); // Q (blocked)
    for slot in &mut slots[1..=4] {
        *slot = CardSlot::Known(pop_rank(&mut deck, 5)); // 6
    }
    for slot in &mut slots[5..=8] {
        *slot = CardSlot::Known(pop_rank(&mut deck, 7)); // 8
    }
    for slot in &mut slots[9..=12] {
        *slot = CardSlot::Known(pop_rank(&mut deck, 6)); // 7
    }
    slots[13] = CardSlot::Known(pop_rank(&mut deck, 0)); // A
    for slot in &mut slots[14..=17] {
        *slot = CardSlot::Known(pop_rank(&mut deck, 3)); // 4
    }
    slots[18] = CardSlot::Known(pop_rank(&mut deck, 9)); // 10 (first successor at root, 0 Queens)
    slots[19] = CardSlot::Known(pop_rank(&mut deck, 11)); // Q
    slots[20] = CardSlot::Known(pop_rank(&mut deck, 12)); // K
    slots[21] = CardSlot::Known(pop_rank(&mut deck, 11)); // Q
    slots[22] = CardSlot::Known(pop_rank(&mut deck, 10)); // J
    slots[23] = CardSlot::Known(pop_rank(&mut deck, 11)); // Q
    for slot in &mut slots[24..=27] {
        *slot = CardSlot::Known(pop_rank(&mut deck, 4)); // 5
    }
    slots[usize::from(INITIAL_WASTE_SLOT.0)] = CardSlot::Known(pop_rank(&mut deck, 10)); // Initial Waste J
    // Fill Stock (`29..=51`) deterministically with remaining 23 cards (`A x3, 2 x4, 3 x4, 9 x4, 10 x3, J x2, K x3`).
    assert_eq!(deck.len(), 23);
    for (dst, src) in slots[usize::from(STOCK_START_SLOT)..DEAL_SLOT_COUNT]
        .iter_mut()
        .zip(deck)
    {
        *dst = CardSlot::Known(src);
    }

    let rank_obj = Objective::ClearRankQuota {
        rank: 11,
        remaining: 4,
    };

    // Case 1: Interrupt Pass 2 (`ClearBoard`) right after root expansion scores `Tap 18 (10)` (0 Queens).
    // Must retain Pass 1's exhaustively discovered 3-Queen route instead of Pass 2's 1 non-target card.
    let run_interrupt_after_passes = |objective: Objective, passes_before_yield: usize| {
        let cancel = AtomicBool::new(false);
        let yield_best = AtomicBool::new(false);
        let finished = std::cell::Cell::new(0usize);
        let ckpt_polls = std::cell::Cell::new(0usize);
        let mut obs = |snap: xsar::search::BestEffortProgress| {
            if snap.search.finished {
                finished.set(finished.get() + 1);
            }
        };
        let ckpt = || {
            if finished.get() >= passes_before_yield {
                let n = ckpt_polls.get() + 1;
                ckpt_polls.set(n);
                // Allow root expansion (root + Tap 18 = 10) before raising yield_best.
                if n >= 8 {
                    yield_best.store(true, std::sync::atomic::Ordering::Release);
                }
            }
            Duration::ZERO
        };
        super::plan_deal_route_controlled(
            &slots,
            objective,
            DEFAULT_SEARCH_LIMITS,
            xsar::search::BestEffortControl {
                cancel: &cancel,
                yield_best: Some(&yield_best),
                progress: Some(&mut obs),
                checkpoint: Some(&ckpt),
            },
        )
        .expect("controlled planning succeeds")
    };

    let pass2_yield = run_interrupt_after_passes(rank_obj, 1);
    match &pass2_yield {
        TriPeaksPlanOutcome::UnsolvablePartial {
            actions,
            replay,
            proof,
            ..
        } => {
            assert_eq!(*proof, PartialProof::Bounded(SearchStop::Interrupted));
            assert_eq!(
                replay.final_position.cleared_by_rank[11], 3,
                "Pass 2 Play must not regress Pass 1's 3-Queen route to Pass 2's non-target card: {actions:?}"
            );
            verify_move_sequence(&slots, actions).expect("preserved route replays");
        }
        other => panic!("expected UnsolvablePartial in Case 1, got {other:?}"),
    }

    // Case 2: Interrupt Pass 3a (`BestPartialProblem`) after Pass 1 & Pass 2 exhaust.
    let pass3a_yield = run_interrupt_after_passes(rank_obj, 2);
    match &pass3a_yield {
        TriPeaksPlanOutcome::UnsolvablePartial {
            actions,
            replay,
            proof,
            ..
        } => {
            assert_eq!(*proof, PartialProof::Bounded(SearchStop::Interrupted));
            assert_eq!(
                replay.final_position.cleared_by_rank[11], 3,
                "Pass 3a Play must not regress Pass 1's 3-Queen route: {actions:?}"
            );
            verify_move_sequence(&slots, actions).expect("preserved route replays");
        }
        other => panic!("expected UnsolvablePartial in Case 2, got {other:?}"),
    }

    // Case 3: Interrupt Pass 3b (`MaxRankPartialProblem`) after Pass 1, Pass 2 & Pass 3a exhaust.
    let pass3b_yield = run_interrupt_after_passes(rank_obj, 3);
    match &pass3b_yield {
        TriPeaksPlanOutcome::UnsolvablePartial {
            actions,
            replay,
            proof,
            ..
        } => {
            assert_eq!(*proof, PartialProof::Bounded(SearchStop::Interrupted));
            assert_eq!(
                replay.final_position.cleared_by_rank[11], 3,
                "Pass 3b Play must not regress Pass 1's 3-Queen route: {actions:?}"
            );
            verify_move_sequence(&slots, actions).expect("preserved route replays");
        }
        other => panic!("expected UnsolvablePartial in Case 3, got {other:?}"),
    }

    // Case 4: `ReachScoreDelta` with optimistically reachable target (`10_000 <= 79_900` so Pass 1
    // explores instead of pruning at root) finds high-streak score (>= 2_500 pts); interrupting Pass 2
    // after 1 low-score card (100 pts, ClearBoard score 1_000) must keep Pass 1's higher score.
    let score_yield = run_interrupt_after_passes(
        Objective::ReachScoreDelta {
            points_needed: 10_000,
        },
        1,
    );
    match &score_yield {
        TriPeaksPlanOutcome::UnsolvablePartial {
            actions,
            replay,
            proof,
            ..
        } => {
            assert_eq!(*proof, PartialProof::Bounded(SearchStop::Interrupted));
            assert!(
                replay.final_position.score_delta >= 2_500,
                "Pass 2 Play must preserve Pass 1's high-streak score (got {}): {actions:?}",
                replay.final_position.score_delta
            );
            verify_move_sequence(&slots, actions).expect("preserved route replays");
        }
        other => panic!("expected UnsolvablePartial in Case 4, got {other:?}"),
    }

    // Case 5: `ClearRankQuota { remaining: 5 }` (only 4 Queens exist on tableau, so Pass 1 prunes at root),
    // Pass 2 (`ClearBoard`) exhausts with multiple cards (`>= 5`), then Pass 3a is interrupted at root+1.
    let pass2_clearboard_yield = run_interrupt_after_passes(
        Objective::ClearRankQuota {
            rank: 11,
            remaining: 5,
        },
        2,
    );
    match &pass2_clearboard_yield {
        TriPeaksPlanOutcome::UnsolvablePartial {
            actions,
            max_tableau_cleared,
            proof,
            ..
        } => {
            assert_eq!(*proof, PartialProof::Bounded(SearchStop::Interrupted));
            assert!(
                *max_tableau_cleared >= 5,
                "Pass 3a Play must preserve Pass 2's exhaustive multi-card partial route (got {max_tableau_cleared}): {actions:?}"
            );
            verify_move_sequence(&slots, actions).expect("preserved route replays");
        }
        other => panic!("expected UnsolvablePartial in Case 5, got {other:?}"),
    }
}
