# CODEX AGENT IMPLEMENTATION RUNBOOK: FreeCell Solver Pipeline

## Architectural Rules & Constraints
1. **Reuse Existing Infrastructure:** Inspect `src/games/cards.rs`, `src/capture.rs`, `src/geometry.rs`, `src/image_matching.rs`, `src/search.rs`, `src/search/best_effort.rs`, `src/execution.rs`, and `src/qmp.rs`. Do not duplicate `Card`, `Suit`, `Rank`, or QMP transport primitives.
2. **Zero Regressions:** All existing Pyramid tests (`tests/pyramid_*.rs`, `tests/search_*.rs`, `tests/tcp_*.rs`) must continue to pass without modification.
3. **Security-First & Bounded Memory:** Use fixed-capacity stack structures (`CascadeStack` capped at 20 cards) for board columns, enforce strict `max_nodes` / `max_duration` search caps, and validate 52-card bijection on every recon snapshot before running $A^*$.

---

## BLOCK 1: RECON — Single Screen Capture at T=0
Unlike Pyramid (which cycles the Draw pile and triggers Undo All), Microsoft FreeCell deals all 52 cards face-up at $T=0$.

### Step 1.1: Define FreeCell Screen Geometry (`src/geometry.rs`)
Add layout coordinate calculators for the Microsoft Solitaire & Casual Games FreeCell board:
* **8 Tableau Cascades (Columns `0..=7`):**
  * Columns `0..=3` contain **7 cards** each at $T=0$ (indices `0..=6` top-to-bottom).
  * Columns `4..=7` contain **6 cards** each at $T=0$ (indices `0..=5` top-to-bottom).
  * Compute the bounding box for each card rank/suit corner glyph along the vertical cascade stride $\Delta y$.
* **4 FreeCell Slots (`0..=3`):** Top-left buffer rectangles (empty at $T=0$).
* **4 Suit Foundation Piles (`0..=3`):** Top-right target rectangles (empty at $T=0$).

### Step 1.2: Single-Frame 52-Card Extraction (`src/capture.rs` & `src/image_matching.rs`)
1. Capture a single frame at $T=0$ via the existing capture / TCP v3 pipeline (`src/tcp/protocol/v3.rs`).
2. Run glyph/template matching (`src/image_matching/comparison.rs` & `black_run.rs`) across the 52 cascade slots ($4 \times 7 + 4 \times 6 = 52$).
3. **52-Card Bijection Gate:** Verify that a `u64` bitmask of the 52 recognized cards equals `(1u64 << 52) - 1` (every standard card present exactly once, zero duplicates, zero unknowns). Return a structured error if recognition is incomplete.

---

## BLOCK 2: A* Shortest Route with Column-Hash Caching, Horne Autoplay & Canonicalization
Create `src/games/freecell/{rules.rs, search.rs, replay.rs}`.

### Step 2.1: Physical State vs. Canonical Transposition Key
* **Live State (`FreeCellBoard`):** Must preserve the exact physical column indices (`cascades: [CascadeStack; 8]`) and physical FreeCell indices (`freecells: [Option<Card>; 4]`) so planned moves map to the true screen columns in Block 4.
* **Order-Independent Column Hashing (`IncrementalBoardHasher`):**
  * Compute an order-dependent `u64` hash for each of the 8 columns based on the exact bottom-to-top sequence of cards inside that column (`0` if empty).
  * When a move transfers a card between `src_col` and `dst_col`, **only recompute the hashes for those 2 columns**.
* **State Canonicalization (`TranspositionKey`):**
  * To collapse symmetrical states in the $A^*$ visited/transposition `HashMap`, build `TranspositionKey`:
    1. Copy `raw_col_hashes: [u64; 8]` and call `.sort_unstable()`. Because the 8 column hashes are sorted, a stack of cards at Column 0 produces the exact same primary key as the same stack at Column 1..7 ($8! = 40,320$ reduction).
    2. Copy the 4 FreeCell encoded bytes `[u8; 4]` and call `.sort_unstable()` ($4! = 24$ reduction).
    3. Include `foundations: [u8; 4]` (indexed by Suit `0..=3`).

### Step 2.2: Horne’s Safe Autoplay Closure
After every deliberate move in the search tree, run `apply_horne_autoplay_closure(&mut next_board, &mut auto_moves)` in a loop until no more safe moves exist:
* An exposed card $C$ (at the bottom of a Cascade or in a FreeCell) with $\text{rank}(C) == \text{foundations}[C.\text{suit}] + 1$ is automatically moved to its Suit Foundation if:
  * $\text{rank}(C) \le 2$ (Aces and Twos), **OR**
  * Both opposite-color Suit Foundations already have rank $\ge \text{rank}(C) - 1$.
* Tag these moves with `is_auto: true` and append them to the node's transition log.

### Step 2.3: Symmetry-Pruned Move Generator
From any state, generate deliberate legal moves in priority order:
1. **Explicit Foundation Moves (`is_auto: false`):** Any exposed Cascade bottom card or FreeCell card where $\text{rank}(C) == \text{foundations}[C.\text{suit}] + 1$ that was *not* already swept by Horne’s rule. *(Critical for uncovering buried cards and for Block 3).*
2. **Cascade $\to$ Non-Empty Cascade:** Exposed bottom card $C_{\text{src}}$ onto exposed bottom card $C_{\text{dst}}$ where $\text{color}(C_{\text{src}}) \neq \text{color}(C_{\text{dst}})$ and $\text{rank}(C_{\text{dst}}) == \text{rank}(C_{\text{src}}) + 1$.
3. **FreeCell $\to$ Non-Empty Cascade:** Same alternating-color descending rule.
4. **Cascade or FreeCell $\to$ Empty Cascade:**
   * Only target the **first** empty Cascade (`first_empty_col`).
   * **Never** move a single lone card (`cascade.len() == 1`) from one Cascade into an empty Cascade.
5. **Cascade $\to$ Empty FreeCell:**
   * Only target the **first** empty FreeCell (`first_empty_fc`).

### Step 2.4: Foundation-Depth Heuristic $f(n) = g(n) + h(n)$
* **$g(n)$:** Count of deliberate moves from $T=0$ (Horne autoplays have $0$ step cost).
* **$h(n)$:**
  $$h(n) = \left(52 - \sum_{s=0}^{3} \text{foundations}[s]\right) + \sum_{s=0}^{3} \text{buried\_depth}(s)$$
  where $\text{buried\_depth}(s)$ is the number of cards sitting on top of the next needed card ($\text{foundations}[s] + 1$) for Suit $s$ inside its Cascade.

---

## BLOCK 3: Fallback to "Maximum Card Removal" (`src/games/freecell/planning.rs` & `src/search/best_effort.rs`)
When a deal has no complete 52-card solution (e.g., Deal #11982) or reaches the search budget (`max_nodes` / timeout):

### Step 3.1: High-Water Mark Tracking Across All Visited States
Maintain a `BestPartialTracker` inside the $A^*$ loop that evaluates every expanded node:
1. **Primary Score:** Maximize `foundation_count()` ($\sum_{s=0}^{3} \text{foundations}[s]$ — total cards moved to the 4 Suit piles at the top of the screen).
2. **Secondary Tie-Breaker:** Minimize $g(n)$ (shortest move sequence to achieve that maximum Suit count).
3. **Tertiary Tie-Breaker:** Minimize $\sum_{s=0}^{3} \text{buried\_depth}(s)$ (closest proximity to unlocking the next Suit card).

### Step 3.2: Why Explicit Foundation Moves Matter in Best-Effort Mode
In deadlocked boards, a higher-rank card (e.g., $6\spadesuit$ when $5\heartsuit$ is permanently trapped) will **not** trigger Horne’s Safe Autoplay rule. Because Step 2.3 explicitly generates non-Horne Foundation moves (`Cascade -> Foundation` and `FreeCell -> Foundation`), the $A^*$ search will naturally explore banking $6\spadesuit, 7\spadesuit, \dots$ to maximize `foundation_count()`.

### Step 3.3: Replay Verification (`src/games/freecell/replay.rs`)
Before returning either `PlanResult::Complete(moves)` or `PlanResult::BestEffort { moves, cards_banked }`, pass the planned move list through `replay::verify_move_sequence(&initial_board, &moves)` to guarantee every step is 100% legal.

---

## BLOCK 4: Implement Game Play Mechanics (`src/execution.rs` & `src/qmp.rs`)
Map `Vec<FreeCellMove>` into the existing QMP pointer execution pipeline:

### Step 4.1: Coordinate Mapping for FreeCell Zones
Extend `src/execution.rs` to translate `Zone` variants into screen click/drag coordinates via `src/geometry.rs`:
* `Zone::Cascade(col)` $\to$ Current bottom-most card coordinate of column `col` (tracked dynamically as column lengths change during playback).
* `Zone::FreeCell(slot)` $\to$ Center coordinate of FreeCell `0..=3`.
* `Zone::Foundation(suit)` $\to$ Center coordinate of the target Suit Foundation pile `0..=3`.

### Step 4.2: Handling `is_auto` Moves vs. Microsoft In-Game Autoplay
In *Microsoft Solitaire & Casual Games*, the client UI automatically flies Horne-safe cards to the Foundations after a deliberate move completes.
* **Configurable Execution Policy (`FreeCellExecutionOptions`):**
  * `skip_horne_auto_moves: true` (Default when Microsoft UI Autoplay is active): The executor performs only deliberate moves (`!mv.is_auto`) and inserts an animation settle delay (`auto_settle_delay_ms`) whenever one or more `mv.is_auto` moves immediately follow a deliberate move, updating internal column heights to stay synchronized.
  * `skip_horne_auto_moves: false`: Explicitly clicks/drags every `is_auto` move if in-game autoplay is disabled or if a card requires an explicit click.

---

## BLOCK 5: Game Win Sequence & Unattended Loop Integration
Hook the completed FreeCell execution cycle into the existing unattended controller loop:
1. **Complete Win (`cards_banked == 52`):** After the final deliberate move triggers the cascade/victory animation, invoke the existing post-game win handler (wait for victory banner $\to$ trigger "New Game" / next deal sequence $\to$ return to Block 1 RECON at $T=0$).
2. **Best-Effort Termination (`cards_banked < 52`):** After executing all moves to bank the maximum possible cards into the Suit piles, log the final telemetry (`cards_banked`, `moves_executed`, `nodes_explored`) and invoke the existing reset / "New Game" handler for unattended continuous operation.