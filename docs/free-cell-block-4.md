# Block 4 — guarded Free Cell route playback (3.1.2)

The application owns this executor: `src/freecell_route.rs` sequences recorded moves, `src/worker/freecell_route.rs` owns live authority/I/O and `src/app/freecell_route.rs` presents explicit controls. `src/freecell_recon/position.rs` verifies physical positions and measured coordinates. Existing Computer Vision source-HALO gameplay remains a separate method.

## Use and automatic transfers

Select A* Route Solution → Free Cell, choose a strict saved initial deal and CALCULATE. Put the guest on that exact original deal with Solver off. Select **Game autoplay ON: observe automatic transfers** or **Game autoplay OFF: execute every transfer** to match the game. Single Step and SOLVE remaining route are explicit input requests. The first invocation freezes the policy; changing it requires CALCULATE again.

`ObserveGame` sends no automatic-record clicks: it observes the complete recorded automatic closure twice, rejecting irreversible prefix regression. `ExecuteRecorded` performs every original tagged edge through the same guarded transfer protocol, including automatic records. Neither mode falls back to the other. Single Step includes leading/trailing automatic transfers and at most one deliberate move, so it can perform multiple physical transfers when autoplay is off. Full playback repeats until the complete route is observed.

## Binding, input and completion

The immutable original deal and independently replayed route remain separate from the mutable physical position/cursor. Initial binding recognises all 52 identities. Subsequent hidden identities may be carried only through this same authenticated run's checked legal history; every complete visible glyph, rank colour and structural boundary must agree. Physical columns/cells/foundations are compared explicitly; canonical search equality cannot authenticate a live arrangement.

Each recorded transfer obtains a fresh native TCPv3 frame and verifies the supported physical parent, measured source/destination, STOP/context/candidate ownership and running-VM/absolute-pointer probe. Both endpoints are frozen for one guarded source-to-destination drag. Source movement must be acknowledged before down; down, held destination movement and up are transmitted before their independent replies are collected. Phase guards can interrupt delivery; post-down failures permit release-only recovery, never transfer replay or a rollback claim. Two consecutive fresh observations of the expected semantic endpoint complete a transfer. Acknowledgements alone never advance the cursor. Readiness/effect observation is bounded to six input-free checks.

PAUSE/CONTINUE stop at safe boundaries and refresh evidence before subsequent input. STOP, context/source invalidation, unsupported evidence, unexpected effects and delivery uncertainty revoke the candidate permanently; CALCULATE is required again. No uncertain drag is replayed. Existing release-only transport recovery remains unchanged. Already acquired first-rejection/latest pixels are retained privately without a diagnostic recapture.

## Supported evidence and remaining acceptance

Native profile: 1920×1080, 100% display/text scale, CLASSIC BLUE, Solver off. Cascade seams are measured from pixels, anchored at y331; screen foundation order H,C,D,S maps explicitly to model C,D,H,S. Occupied top slots require coherent whole-face/identity evidence at the measured nominal or one-pixel-left origin. Positive empty outlines/felt, complete final faces and continuous exposed rails remain required. Toolbar-clipped ten-card columns can refuse; arbitrary legal positions and artworks are not universally supported.

Host qualification and local simulated-peer tests prove implementation behaviour. No original clean Solver-off midgame sequence or full guest transfer/autoplay acceptance is claimed. Native drag effects, autoplay timing, repeated Single Step and complete midgame playback need fresh operator-checked validation. See [3.1.2 verification](Verification-3.1.2.md); [3.1.1 verification](Verification-3.1.1.md) retains the earlier protocol evidence. Block 3 partial optimisation and later win/restart loops remain separate scope.

## Retained original design proposal

The following proposal is historical; the implemented contract above supersedes its hypothetical APIs, coordinate ownership and delay-only synchronisation.

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
