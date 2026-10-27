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
