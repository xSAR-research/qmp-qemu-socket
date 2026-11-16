//! Comprehensive tests for TriPeaks rules, geometry, A* vs Dijkstra optimality, RECON loop, and JSON persistence.

use std::{
    collections::VecDeque,
    fs,
    io::Cursor,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use xsar::{
    games::cards::{Card as XsarCard, Rank, Suit},
    search::{SearchMethod, SearchOptions, SearchOutcome, SearchStop},
};

use crate::{
    geometry::PixelRect,
    parameters::TABLEAU_CARD_REGIONS,
    recon::{
        Card, CardSlot, Objective, ReconDomain, ReconError, ReconOutcome, SlotId,
        run_epistemic_recon,
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

    fn plan_info_gain_probe(&self, slots: &[CardSlot]) -> Option<Vec<Self::Move>> {
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

    fn best_partial_route(&self, slots: &[CardSlot]) -> Vec<Self::Move> {
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
    } = out_unsolv
    else {
        panic!("expected UnsolvableBestPartial, got {out_unsolv:?}");
    };
    assert!(unreachable_slots >= 3);
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

        fn plan_info_gain_probe(&self, slots: &[CardSlot]) -> Option<Vec<Self::Move>> {
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

        fn best_partial_route(&self, slots: &[CardSlot]) -> Vec<Self::Move> {
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
    assert!(plan_info_gain_probe(&truth).is_none()); // all 52 already Known -> 0 unknowns left
}
