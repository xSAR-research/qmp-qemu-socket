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
  