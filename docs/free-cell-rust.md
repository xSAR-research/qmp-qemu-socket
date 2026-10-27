Here is the clean, modular Rust reference implementation showing how Block 2 (8-Column Hashing + $A^*$), Block 3 (Best-Effort Suit Maximizer), and Block 4 (Execution Coordinate Tracking) connect together:

```rust
//! Reference implementation for `src/games/freecell/planning.rs` and `execution.rs` bridge.
//! Zero-unsafe, bounded-memory A* with 8-column order-independent hashing and Best-Effort fallback.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::time::{Duration, Instant};

use crate::games::freecell::rules::{
    Card, CascadeStack, FreeCellBoard, FreeCellMove, Zone,
};

/// Compact, order-independent primary key for the A* transposition table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TranspositionKey {
    pub sorted_col_hashes: [u64; 8],
    pub sorted_freecells: [u8; 4],
    pub foundations: [u8; 4],
}

/// Tracks individual column hashes so each move only re-hashes 1 or 2 modified columns.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColumnHashState {
    pub col_hashes: [u64; 8],
}

impl ColumnHashState {
    pub fn new(board: &FreeCellBoard) -> Self {
        let mut col_hashes = [0u64; 8];
        for i in 0..8 {
            col_hashes[i] = Self::hash_column(&board.cascades[i]);
        }
        Self { col_hashes }
    }

    #[inline]
    pub fn update_col(&mut self, col_idx: usize, stack: &CascadeStack) {
        if col_idx < 8 {
            self.col_hashes[col_idx] = Self::hash_column(stack);
        }
    }

    #[inline]
    pub fn to_key(&self, board: &FreeCellBoard) -> TranspositionKey {
        let mut sorted_col_hashes = self.col_hashes;
        sorted_col_hashes.sort_unstable();

        let mut sorted_freecells = [
            Self::pack_opt_card(board.freecells[0]),
            Self::pack_opt_card(board.freecells[1]),
            Self::pack_opt_card(board.freecells[2]),
            Self::pack_opt_card(board.freecells[3]),
        ];
        sorted_freecells.sort_unstable();

        TranspositionKey {
            sorted_col_hashes,
            sorted_freecells,
            foundations: board.foundations,
        }
    }

    #[inline]
    fn hash_column(stack: &CascadeStack) -> u64 {
        if stack.is_empty() {
            return 0;
        }
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for slot in stack.as_slice() {
            if let Some(c) = slot {
                let b = ((c.suit as u8) << 4) | (c.rank & 0x0F);
                h ^= u64::from(b) + 1;
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        }
        h = (h ^ (h >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        h = (h ^ (h >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        h ^ (h >> 31)
    }

    #[inline]
    const fn pack_opt_card(opt: Option<Card>) -> u8 {
        match opt {
            None => 0,
            Some(c) => ((c.suit as u8) << 4) | (c.rank & 0x0F),
        }
    }
}

/// Search limits to guarantee responsive execution on `beast`.
#[derive(Clone, Copy, Debug)]
pub struct FreeCellSearchLimits {
    pub max_nodes: usize,
    pub max_duration: Duration,
}

impl Default for FreeCellSearchLimits {
    fn default() -> Self {
        Self {
            max_nodes: 500_000,
            max_duration: Duration::from_secs(5),
        }
    }
}

/// Unified output for Block 2 (Complete Win) and Block 3 (Best-Effort Suit Maximizer).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FreeCellPlan {
    pub is_complete_win: bool,
    pub cards_banked: u8,
    pub foundations_final: [u8; 4],
    pub moves: Vec<FreeCellMove>,
    pub nodes_expanded: usize,
}

struct ArenaNode {
    board: FreeCellBoard,
    col_hash_state: ColumnHashState,
    parent_idx: Option<u32>,
    deliberate_move: Option<FreeCellMove>,
    auto_moves: Vec<FreeCellMove>,
    g_cost: u32,
    buried_penalty: u32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct OpenEntry {
    f_cost: u32,
    banked: u8,
    g_cost: u32,
    node_idx: u32,
}

impl Ord for OpenEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .f_cost
            .cmp(&self.f_cost)
            .then_with(|| self.banked.cmp(&other.banked))
            .then_with(|| other.g_cost.cmp(&self.g_cost))
    }
}

impl PartialOrd for OpenEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Runs A* with 8-column order-independent hashing and automatic Best-Effort fallback.
pub fn plan_freecell_game(
    initial_board: &FreeCellBoard,
    limits: FreeCellSearchLimits,
) -> FreeCellPlan {
    let start_time = Instant::now();
    let mut root_board = initial_board.clone();
    let mut root_autos = Vec::new();
    root_board.run_horne_autoplay(&mut root_autos);

    let root_hashes = ColumnHashState::new(&root_board);
    let root_key = root_hashes.to_key(&root_board);
    let root_buried = root_board.buried_target_depth();
    let root_h = (52 - root_board.foundation_count()) as u32 + root_buried;

    let mut arena = Vec::with_capacity(65_536);
    arena.push(ArenaNode {
        board: root_board.clone(),
        col_hash_state: root_hashes,
        parent_idx: None,
        deliberate_move: None,
        auto_moves: root_autos,
        g_cost: 0,
        buried_penalty: root_buried,
    });

    let mut open = BinaryHeap::new();
    open.push(OpenEntry {
        f_cost: root_h,
        banked: root_board.foundation_count(),
        g_cost: 0,
        node_idx: 0,
    });

    let mut best_g_map: HashMap<TranspositionKey, u32> = HashMap::with_capacity(65_536);
    best_g_map.insert(root_key, 0);

    // Block 3 Best-Effort High-Water Mark Tracker
    let mut best_idx: u32 = 0;
    let mut best_banked: u8 = root_board.foundation_count();
    let mut best_g: u32 = 0;
    let mut best_buried: u32 = root_buried;
    let mut nodes_expanded: usize = 0;

    while let Some(entry) = open.pop() {
        let idx = entry.node_idx as usize;
        let current_board = arena[idx].board.clone();
        let current_hashes = arena[idx].col_hash_state;
        let current_g = arena[idx].g_cost;
        let current_buried = arena[idx].buried_penalty;
        let current_banked = current_board.foundation_count();

        nodes_expanded += 1;

        // Update Block 3 Best-Effort candidate:
        // 1. Most cards in Suit piles -> 2. Fewest moves -> 3. Shallowest next target
        let is_better_partial = (current_banked > best_banked)
            || (current_banked == best_banked && current_g < best_g)
            || (current_banked == best_banked
                && current_g == best_g
                && current_buried < best_buried);

        if is_better_partial {
            best_banked = current_banked;
            best_g = current_g;
            best_buried = current_buried;
            best_idx = entry.node_idx;
        }

        // Block 2 Complete Win Check
        if current_banked == 52 {
            return FreeCellPlan {
                is_complete_win: true,
                cards_banked: 52,
                foundations_final: [13, 13, 13, 13],
                moves: reconstruct_moves(&arena, entry.node_idx),
                nodes_expanded,
            };
        }

        // Guardrails against 300s stalls or unbounded memory growth
        if arena.len() >= limits.max_nodes || start_time.elapsed() >= limits.max_duration {
            break;
        }

        for mv in current_board.generate_legal_moves() {
            let mut next_board = current_board.clone();
            let mut next_hashes = current_hashes;

            // Apply deliberate move and incrementally update ONLY affected column hashes
            next_board.apply_move_and_rehash(mv, &mut next_hashes);

            // Run Horne's Safe Autoplay closure and update any columns that auto-popped
            let mut auto_log = Vec::new();
            next_board.run_horne_autoplay_and_rehash(&mut auto_log, &mut next_hashes);

            let key = next_hashes.to_key(&next_board);
            let tentative_g = current_g + 1;

            let should_insert = match best_g_map.get(&key) {
                Some(&prev_g) => tentative_g < prev_g,
                None => true,
            };

            if should_insert {
                best_g_map.insert(key, tentative_g);
                let next_banked = next_board.foundation_count();
                let next_buried = next_board.buried_target_depth();
                let h_cost = (52 - next_banked) as u32 + next_buried;
                let f_cost = tentative_g + h_cost;

                let child_idx = arena.len() as u32;
                arena.push(ArenaNode {
                    board: next_board,
                    col_hash_state: next_hashes,
                    parent_idx: Some(entry.node_idx),
                    deliberate_move: Some(mv),
                    auto_moves: auto_log,
                    g_cost: tentative_g,
                    buried_penalty: next_buried,
                });

                open.push(OpenEntry {
                    f_cost,
                    banked: next_banked,
                    g_cost: tentative_g,
                    node_idx: child_idx,
                });
            }
        }
    }

    // Block 3 Fallback: Return path that removes maximum cards to Suit piles
    let final_foundations = arena[best_idx as usize].board.foundations;
    FreeCellPlan {
        is_complete_win: false,
        cards_banked: best_banked,
        foundations_final: final_foundations,
        moves: reconstruct_moves(&arena, best_idx),
        nodes_expanded,
    }
}

fn reconstruct_moves(arena: &[ArenaNode], target_idx: u32) -> Vec<FreeCellMove> {
    let mut chain = Vec::new();
    let mut cursor = Some(target_idx);
    while let Some(idx) = cursor {
        chain.push(idx as usize);
        cursor = arena[idx as usize].parent_idx;
    }
    chain.reverse();

    let mut out = Vec::new();
    for idx in chain {
        if let Some(mv) = arena[idx].deliberate_move {
            out.push(mv);
        }
        out.extend_from_slice(&arena[idx].auto_moves);
    }
    out
}
```
