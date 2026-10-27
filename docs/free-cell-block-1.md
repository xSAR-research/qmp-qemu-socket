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

