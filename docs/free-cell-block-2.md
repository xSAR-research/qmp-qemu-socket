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
  