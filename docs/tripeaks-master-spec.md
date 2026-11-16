# TriPeaks Master Specification (RECON + Search Pipeline)

**Document status:** Active implementation specification (baseline crate `3.3.2`, `../xsar` `0.16.3`; Goal 6 will append final `3.4.0` qualification metrics and `## Code Analysis`).

## 1. Purpose and System Boundaries

This document specifies the end-to-end partial-observability reconnaissance (`RECON`), search, JSON deal persistence, operator UI, and guarded QMP execution pipeline for *Microsoft Solitaire & Casual Games* — TriPeaks on Beast (Arch Linux host, KDE Wayland) controlling a Windows 11 QEMU guest.

- **Native guest calibration:** `1920×1080`, `100%` Windows display and text scale, `CLASSIC BLUE` card back.
- **Crate boundary (`qmp-qemu-socket` vs `../xsar`):**
  - `qmp-qemu-socket` owns all TriPeaks domain models, 52-slot state representations, covering and `±1` matching rules, pixel/corner card readers, `src/recon.rs` probe orchestration, JSON deal schemas/codecs, `xsar::search` problem adapters, worker execution loops, `egui` operator controls, and guest input authority.
  - `../xsar` (`0.16.3`) remains strictly domain-neutral infrastructure and is left untouched.
- **Operator UI wording rule:** User-facing labels must use plain, direct operational terminology such as **`Known / Unknown / Unreachable`**, **`Known cards`**, and **`Probes used`** rather than academic jargon (avoid `epistemic` in user-facing UI controls and status badges).

---

## 2. TriPeaks Board Model and Geometry

### 2.1 Slot Addressing (`52` Total Slots)

TriPeaks deals a standard single 52-card deck:

- **Tableau (`28` cards, indices `0..28`):** Four rows across three peaks with row lengths `TABLEAU_ROW_LENGTHS = [3, 6, 9, 10]` and row start offsets `TABLEAU_ROW_STARTS = [0, 3, 9, 18]` (`src/cards.rs`).
  - External coordinates remain **1-based** `(row, column)` with `1 <= row <= 4` and `1 <= column <= row_length(row)`:
    - Row 1 (`3` cards): columns `1..=3`
    - Row 2 (`6` cards): columns `1..=6`
    - Row 3 (`9` cards): columns `1..=9`
    - Row 4 (`10` cards): columns `1..=10` (initially exposed face-up)
  - Slot positions are never renumbered after card removal.
  - Row 4 cards begin exposed (`CardRead::Card`). Rows 1–3 begin covered (`CardRead::FaceDown` until both child positions on the row below have been cleared, at which point the game flips the newly uncovered card face-up).
- **Initial Waste (`1` card, index `28`):** Face-up starter card at `WASTE_CARD_BOUNDS`.
- **Stock (`23` cards, indices `29..52`):** Face-down draw pile flipped sequentially (`1..=23`) onto the Waste pile.

### 2.2 Calibrated `1920×1080` Regions (`src/parameters.rs`)

Established pixel bounds in the `1920×1080` native frame:

- **Tableau card slots:** `TABLEAU_CARD_REGIONS` (`28` entries):
  - Row 1: `y = 111`, height `181`, width `136`
  - Row 2: `y = 221`, height `181`, width `136`
  - Row 3: `y = 332`, height `181`, width `136`
  - Row 4: `y = 443`, height `181`, widths `136`/`137`
- **Non-overlapping mid-card scan bands:** `TABLEAU_ROW_SCAN_PROFILES` (`card_y + 85` -> `y = 196, 306, 417, 528`), keeping face-up paper / face-down back inspection strictly inside the uncovered band between overlapping rows.
- **Stock lane & initial Stock card:**
  - `TARGET_STOCK`: `PixelRect::new(700, 665, 280, 217)` (wide capture zone accommodating left-to-right horizontal contraction by `-3 px/draw` from `x = 836..966` at `T = 0` down to `x = 770..900` at Draw `22`, followed by Draw `23` dark-green empty tray `R = 0..81, G = 57..178, B = 28..140, min(G-R) = 54, min(G-B) = 23` inside outline `x = 767..900, y = 682..860`)
  - `STOCK_CARD_BOUNDS`: `PixelRect::new(832, 682, 136, 182)` (initial `T = 0` Stock top position)
- **Waste card:**
  - `WASTE_CARD_BOUNDS`: `PixelRect::new(1017, 682, 136, 182)`
- **Shared Undo All control:**
  - `SHARED_UNDO_ALL_CONTROL`: `(1320, 978)`
- **Goal 2 Challenge/Normal HUD, Tray Caption & Modal Bounds (`src/tripeaks_progress.rs`):**
  - Challenge header ribbon: border rect `(814, 41, 292, 46)`, word strip `(820, 48, 282, 32)`, short counter `(1_036, 48, 65, 32)`, `Score` counter `(959, 48, 142, 32)`
  - Normal top-HUD `Boards: <X> of 3`: header `(1658, 46, 160, 32)` and `New` footer `(212, 998, 58, 31)` (via `pyramid_progress::observe_normal`)
  - Normal top-HUD `Score: <N>`: centred `"Score:"` + native `h = 17` digits in `x = 1435..1625, y = 46..78`
  - Tray deals-left caption (`0 deals left` / `1 deal left` / `2 deals left`): `(882, 872, 156, 26)`
  - `"No Moves Remaining"` modal: panel `x = 572..1347, y = 268..795`, title `(740, 299, 445, 48)`, active `"Undo last move"` button `PixelRect::new(976, 692, 305, 72)` (`UNDO_LAST_MOVE_CLICK_POINT = PixelPoint::new(1_128, 728)`)

### 2.3 Covering, Legal-Move & Scoring Rules

- A Tableau card at `(row, col)` (for `row in 1..=3`) becomes uncovered if and only if both covering child slots below it on `row + 1` are empty (`Empty`). Covered cards cannot be played or read while `FaceDown`.
- An uncovered Tableau card is legal to play if and only if its rank differs from the active Waste rank by `±1` with wrap-around (`King <-> Ace <-> 2`, i.e. circular modulo-13 distance `1`).
- Drawing from Stock (`D`) moves the next Stock card onto Waste whenever at least one Stock card remains; Undo All resets the board to its exact `t = 0` deal state (`Row 4` + `Waste 28` + `23` Stock cards) once any Draw-23 `"No Moves Remaining"` modal is backed out cleanly (see Section 5).
- **Empirically verified live TriPeaks Normal scoring (`Goal 2`, Beast):**
  - Consecutive tableau clear streaks `k = 1, 2, 3, ...` increment `Score` by `100 * (2 * k - 1)` (`+100, +300, +500, +700, +900, +1,100`, cumulative `100 * streak^2`).
  - `Draw` (`D`) adds `0` points and resets `streak = 0`.
  - Clearing a `Row 1` Peak adds a `+500` Peak-clear bonus on top of the card's streak value (`+100 + 500 = +600` at streak `1`).

---

## 3. Card Observation and Recognition Contract (`Goal 1`)

### 3.1 `CardRead` Semantics (`src/game_operations.rs`)

`CardRead` separates positive physical evidence cleanly and fails closed on contradiction:

- `CardRead::Card(Card)` — Uniquely recognised rank and suit via `game_controls::classify_corner` plus positive face-up paper evidence.
- `CardRead::FaceDown` — Positive `CLASSIC BLUE` card-back evidence (`crate::game::is_classic_blue_back`) in a covered Tableau slot (`Rows 1–3`) or non-empty Stock (`Draw 1`), mutually exclusive with `Card(Card)` and `Empty`.
- `CardRead::Empty` — Positive background felt / empty-slot proof, exclusive of face-up rank/suit or card-back evidence.
- `CardRead::Unreadable` — Supported location lacks decisive identity, face-down back, or empty proof (for instance dimmed overlays, partial animation, or damaged corners).
- `CardRead::Ambiguous` — Competing rank/suit evidence or contradictory simultaneous face-up / face-down / empty evidence.

### 3.2 `LocationRequest` and `game_controls::read_card`

- `LocationRequest::tripeaks_play(row, column)` validates `1 <= row <= 4` and `1 <= column <= TABLEAU_ROW_LENGTHS[row - 1]`. Out-of-range coordinates return `OperationError::Invalid`.
- `LocationRequest::tripeaks_draw(slot)` validates `row == 1` and `slot in 1..=2` (`1` = Stock across `TARGET_STOCK`, `2` = Waste at `WASTE_CARD_BOUNDS`).
- Unsupported `Cell` and `Suit` kinds for `GameMode::TriPeaks` return `OperationError::Unsupported`.
- `game_controls::read_card` for Stock shares parameterised `TriPeaksSurfaceStats` sampling to distinguish contracted Draw-22 last-card `CardRead::FaceDown`, Draw-23 dark-green empty tray (`is_tripeaks_stock_tray_green`) `CardRead::Empty`, and dimmed modal backdrops `CardRead::Unreadable`.
- All TriPeaks corner/back/felt tolerances must honour Beast's calibrated foreground and card-back profiles without weakening margins to force test passes.

### 3.3 Progress, Deals-Left Caption & Modal Observers (`src/tripeaks_progress.rs`, Goal 2)

- `observe_challenge_header` extracts linear white-blend ink colour-independently across arbitrary Challenge ribbons (`<= 210_000 ppm`, `>= 100_000 ppm` distinct-class margin), supporting all `13` plural rank words (`Aces`..`Kings`, with `12` native + calibrated-synthetic spaced/compact `Threes` assembled strictly from native banner glyphs pending an opportunistic live `Threes` capture), `Boards cleared X/Y`, `Score X/Y`, `Solve the deck`, and `Clear the <single card>` (`TriPeaksChallenge::UnsupportedSingleCard`), mapping supported objectives via `TriPeaksChallenge::to_recon_objective` (`ClearBoard`, `ClearRankQuota`, `ReachScoreDelta`).
- `observe_normal_boards` and `observe_normal_boards_header` reuse `pyramid_progress::observe_normal` / `observe_normal_header`.
- `observe_normal_score` detects the green top HUD `"Score:"` label (`normal-score-label.ink` / `normal-score-label-alt.ink`) and decodes the native `h = 17` grouped score digits (`normal-digits.ink`, `28` native glyphs across `0..=9` and `','`) via shared `digits::read_with_glyph_bank` + `decimal::integer`.
- `observe_deals_left_caption` separates native tray captions (`deals-{zero,one,two}.ink` on `y = 872..898`) from blank Normal trays and dimmed modals.
- `observe_no_moves_modal` positively identifies dimmed backdrops + `"No Moves Remaining"` modal title (`no-moves-title.ink`) + active blue `"Undo last move"` button, returning `Ok(Some(UNDO_LAST_MOVE_CLICK_POINT))` with zero input authority.

---

## 4. Partial-Observability RECON Architecture (`src/recon.rs`)

### 4.1 Canonical `Card` Encoding and Checked Conversions

- `recon::Card(pub u8)` uses canonical `0..52` deck IDs matching `xsar::games::freecell::rules::card_id`: `suit_index * 13 + rank_index`, where `Suit::ALL = [Clubs, Diamonds, Hearts, Spades]` (`0..=3`) and `Rank::ALL = [Ace..=King]` (`0..=12`).
- `TryFrom<xsar::games::cards::Card> for recon::Card` and `TryFrom<recon::Card> for xsar::games::cards::Card` enforce `0..52` bounds without `.unwrap()` or `.expect()`.

### 4.2 Card Knowledge State & Fail-Closed Diagnostics

Each of the 52 slots transitions monotonically from `CardSlot::Unknown` to `CardSlot::Known(Card)` (or `CardSlot::Unreachable` if proved legally unreachable):

- **Single-deck uniqueness (`DuplicateCard`):** If `is_single_deck == true` and a newly observed `Card` already binds a distinct `SlotId`, `bind_observation` immediately fails with `ReconError::DuplicateCard { first_slot, second_slot, card }` rather than allowing corrupt state or triggering complement deduction.
- **Determinism (`NonDeterministicSlot`):** Re-observing the same `SlotId` with a different `Card` fails immediately with `ReconError::NonDeterministicSlot`.
- **51-to-52 complement deduction:** Once `51` distinct single-deck cards are known and exactly `1` `SlotId` remains unknown, `bind_observation` deduces the final card deterministically without requiring an extra physical probe.
- **Typed operational errors:** `ReconError` separates transport faults (`QmpTransport`), operator cancellation (`Cancelled`), uncertain QMP input delivery (`UncertainDelivery`), scene verification failures (`SceneVerification`), CV failures (`VisionClassificationFailed`), Undo All verification mismatches (`RootRestoreMismatch`), deal anomalies (`NonDeterministicSlot`, `DuplicateCard`), and probe ceiling hits (`ProbeBudgetExhausted`).

### 4.3 `ReconDomain` and `run_epistemic_recon` Loop (`ReconOutcome`)

1. `ReconDomain::initialize_and_sweep_stock` captures the initial `t = 0` root snapshot (Row 4 `0..10` cards, Waste `28`, and `root_hash`), runs the safe initial Stock sweep (`1..=23`) with Undo All restoration (see Section 5), and returns `(slot_table, root_hash)`.
2. For up to `max_probes` cycles inside `run_epistemic_recon`:
   - Attempt pure search over known slots (`ReconDomain::solve_known_subgraph(&slots, objective)` where `Objective` is `ClearBoard`, `ClearRankQuota`, or `ReachScoreDelta`).
     - If solvable cleanly from the restored root state, return `ReconOutcome::SolvedFromRoot { route, probes_used }`.
   - Plan next information-gain probe (`ReconDomain::plan_info_gain_probe`) picking a reachable move sequence that exposes one or more `Unknown` Tableau parents in Rows 1–3; if none are reachable from known information, mark remaining `Unknown` slots as `CardSlot::Unreachable` and return `ReconOutcome::UnsolvableBestPartial { best_partial_route: domain.best_partial_route(&slots), unreachable_slots }`.
   - Execute the probe over QMP (`ReconDomain::execute_probe_and_observe`), recording post-step observations; if the live board achieves the objective during probing, return `Ok(None)` -> `ReconOutcome::CompletedDuringRecon`.
   - Bind observations via `bind_observation`, then restore root (`ReconDomain::undo_all_and_hash`) and verify `restored_hash == root_hash`; on mismatch, fail with `ReconError::RootRestoreMismatch`.
3. Do **not** blindly issue `undo_all_and_hash` when `Cancelled`, `UncertainDelivery`, or `SceneVerification` aborts execution; preserve existing targeted release-only recovery rules without sending unverified post-`STOP` clicks.

---

## 5. Beast Calibration & Draw-23 Stock Sweep Risk Flag (Goals 2 & 4)

> **GOAL 2 EMPIRICAL VERIFICATION & GOAL 4 CONTRACT (`initialize_and_sweep_stock`):**
> Live Windows 11 calibration on Beast (`tests/fixtures/tripeaks-calibration-20261116/`) established exact Draw-23 and modal behaviour:
>
> - **Draw-23 with active Row-4 matches (`stock-empty.png`):** Stock empties into the dark-green slot tray without triggering a modal; `CardRead::Empty` triggers and `SHARED_UNDO_ALL_CONTROL` `(1320, 978)` executes directly on TriPeaks without an `OK / Cancel` dialog, animating all cards back over `~1.0 s` to `T = 0` (`0` differing pixels across `y = 40..1025`).
> - **Draw-23 with zero playable tableau matches (`no-moves-remaining.png`):** After `~1.2–2.0 s`, Microsoft Solitaire dims the full board (including `SHARED_UNDO_ALL_CONTROL` `(1320, 978)` -> `(16, 14, 13)`) and displays `"No Moves Remaining"` (`x = 572..1347, y = 268..795`).
> - **Mandatory Goal 4 handling:** `initialize_and_sweep_stock` must either avoid triggering the terminal lock when Stock exhaustion is already proven with no Row-4 moves, OR detect `tripeaks_progress::observe_no_moves_modal` (`src/tripeaks_progress.rs`) and click `"Undo last move"` (`UNDO_LAST_MOVE_CLICK_POINT` `(1_128, 728)` using `POST_GAME_MOUSE_HOLD = 500 ms`, as short `50 ms` clicks fail to dismiss the XAML dialog) to un-dim the board before clicking `SHARED_UNDO_ALL_CONTROL` `(1320, 978)`.
> - Never issue blind clicks onto `SHARED_UNDO_ALL_CONTROL` `(1320, 978)` while a modal overlay owns the guest screen, and treat any unrecognised dimmed overlay as `ReconError::SceneVerification` / zero input authority.

Fixture and template provenance for Goal 2 calibration (`tests/fixtures/tripeaks-calibration-20261116/`):

- `5` native `1920×1080` captures (`stock-last-card`, `stock-empty`, `tableau-row2-uncovered`, `peak1-uncovered`, `no-moves-remaining`).
- `28` native/calibrated `.ink` + `.txt` templates indexed with SHA-256 digests in `manifest.json` (`12` native plural ranks, calibrated-synthetic spaced/compact `Threes` assembled from native glyphs, shared Challenge keywords, deals-left captions, `"No Moves Remaining"` title, Normal `"Score:"` dual subpixel labels, and `28` native `h = 17` Normal HUD digits/commas).

---

## 6. JSON Deal Codecs, Search Adapter, Operator UI & Execution (Goals 3–6)

### 6.1 State Separation & Atomic JSON Persistence (`src/tripeaks_model/json.rs`)

- Separate immutable **initial deal / card-slot map** (`[CardSlot; 52]`), **raw recognition observations**, **planned/recorded move lists** (`TriPeaksMove::TapTableau { row, column }` / `TriPeaksMove::Draw`), and **mutable simulation occupancy** (`TriPeaksPosition` bitmask). `Unknown` and `Unreachable` strictly block tapping/drawing while still tracking newly uncovered parents during replay.
- `schema_version: 1`, `game: "tripeaks"`, `provenance` (`"restored_at_root"` / `"completed_during_recon"`), `probes_used`, `objective` (`"clear_board"`, `"clear_rank_quota"`, `"reach_score_delta"`), `52`-element `slots` (`{"Known": "AS"}`, `"Unknown"`, `"Unreachable"`), and optional `planned_route` (`<= 51` moves).
- Enforce the `65,536`-byte input/output limit, duplicate JSON object key rejection (including `\uXXXX`-escaped keys), trailing token refusal, and full deal/route replay validation (`verify_move_sequence`) before publishing.
- Atomic export (`publish_to_path`) uses same-directory `0600` temporary file creation (`create_new`), `sync_all`, and `no-overwrite` `hard_link` with immediate `DestinationState` inspection (`Absent`, `MatchingBytesPresent`, `PreExistingDifferentBytes`, `InspectionFailed`) on any link failure.

### 6.2 Search & Domain Adapter (`src/tripeaks_model/search.rs`, `xsar::search`)

- Use `xsar::search` (A*/Dijkstra and `solve_best_effort_with_control`) strictly from `qmp-qemu-socket` without modifying `../xsar`.
- Forward move scoring conservatively credits only Beast-verified per-tap streak points (`100 * (2 * streak - 1)`) plus `+500` per cleared `Row 1` Peak (`slots 0..3`); the admissible heuristic and sound impossibility pruning for `ReachScoreDelta` upper-bound `max_gain` optimistically over remaining `Known` cards and Peaks (and reset streak across forced `Draw` transitions) so `A*` remains optimal without underestimating guest thresholds or missing reachable score routes.
- Outside `ReachScoreDelta`, `SearchState` canonicalizes `streak = 0` and `score_progress = 0` to collapse transposition entries, and uses `solve_best_effort_with_control` (`SearchMethod::Dijkstra`) for both `plan_info_gain_probe` (maximising newly uncovered `Unknown` parents with deeper row & shortest-route tie-breaks) and `best_partial_route`.

### 6.3 Guarded QMP Execution & `GAME WIN` Capture

- Every gesture/click sequence honours the crate's guarded input contract:
  1. Check `STOP` / worker context token and fresh TriPeaks scene verification;
  2. Probe QEMU VM running state and active absolute pointer routing;
  3. Re-check `STOP`;
  4. Map `1920×1080` coordinates via checked `pixel * 32767 / extent`;
  5. Verify physical board/Waste/Stock change in a fresh frame before advancing to the next move.
- Record `GAME WIN` sequence frames by default (`--no-image` disables win-sequence recording while retaining failure diagnostics).

### 6.4 Operator UI Controls (`egui`)

- Present TriPeaks controls cleanly inside the existing left/workspace layout using plain operator terminology:
  - Status & card knowledge summary using **`Known / Unknown / Unreachable`** slot counts (e.g. `Known: 34 / 52`, `Unknown: 18`, `Unreachable: 0`) rather than `"epistemic"` phrasing.
  - Explicit **`Probes used`**, Stock sweep toggle/status, solver status, Play / Step / Pause / `STOP` / `Undo All` controls, and deal JSON load/export controls.
- Keep all QMP I/O, frame capture, Stock sweeps, probing, and search off the `egui` thread on bounded worker threads.

---

## 7. Phased Delivery Roadmap & Verification Gates

- **Goal 1 (baseline `3.3.2`):** Write `docs/tripeaks-master-spec.md`; wire and harden `src/recon.rs` (checked `Card` <-> `xsar` `TryFrom`, duplicate-card guard, new typed `ReconError` variants, Rust 2024 `let`-chain cleanup, comprehensive unit tests); add `CardRead::FaceDown` in `src/game_operations.rs`; implement `LocationRequest::tripeaks_play` / `tripeaks_draw` and `(GameMode::TriPeaks, LocationKind::Play | Draw)` in `src/game_controls.rs` verified against `tests/fixtures/tripeaks-redeal-20261016/{draw-first.png, left-ace.png, left-eight.png}` and synthetic edge fixtures.
- **Goal 2 (Beast Calibration — baseline `3.3.2`):** Captured live Beast screenshots across Stock `0..23`, Draw 23 empty-stock with/without Row-4 matches, `"No Moves Remaining"` modal + `"Undo last move"` (`POST_GAME_MOUSE_HOLD`), confirmation-less `Undo All` `T=0` restoration, uncovered Row 1–3 cards, and live `Score` streak/draw/peak formula; updated Stock empty-tray detection in `src/game_controls.rs`; shared `digits::read_with_glyph_bank` / `decimal::integer` in `src/pyramid_progress/challenge/`; and added `src/tripeaks_progress.rs` Challenge/Normal/deals-left/modal observers with curated fixtures and templates in `tests/fixtures/tripeaks-calibration-20261116/`.
- **Goal 3 (Pure Model & `xsar::search` Adapter — baseline `3.3.2`):** Implemented 52-slot TriPeaks board/rules/scoring (`src/tripeaks_model/rules.rs`), independent replay verifier (`replay.rs`), admissible `xsar::search` A*/ `BestEffortProblem` adapters (`search.rs`), and strict `schema_version: 1` `65,536`-byte JSON codec with atomic `no-overwrite` `publish_to_path` (`json.rs`), backed by end-to-end `run_epistemic_recon` tests covering solvable 52-card boards, 51->52 complement deduction, early-exit rank/score routes, live `CompletedDuringRecon`, `DuplicateCard` refusal, `UnsolvableBestPartial`, and A* vs Dijkstra optimality.
- **Goal 4 (`LiveTriPeaksDomain` & Stock Sweep):** Implement `ReconDomain` for live Beast QMP + capture, including `initialize_and_sweep_stock` with Draw-23 modal guard before `SHARED_UNDO_ALL_CONTROL` `(1320, 978)`.
- **Goal 5 (`tripeaks_session` Worker & Plain Operator UI):** Wire worker session, plain `Known / Unknown / Unreachable` UI status panels, `GAME WIN` recording, and cancel/`STOP` safety tests.
- **Goal 6 (Live Beast Qualification & `3.4.0` Release):** Full Beast acceptance testing, approved `3.4.0` version bump + `CHANGELOG.md` + `docs/milestones.md`, and final update to this document appending **final `3.4.0` qualification metrics** and **`## Code Analysis`**.

### Mandatory Rust Verification Gates (All Goals)

Every Rust code milestone must pass:

1. `cargo fmt --package qmp-qemu-socket -- --check`
2. `cargo clippy --workspace --lib --bins -- -D warnings -D clippy::unwrap_used -D clippy::dbg_macro -D clippy::todo`
3. `cargo clippy --workspace --all-targets -- -D warnings`
4. `cargo test --workspace --offline --locked`
5. `cargo test --locked --offline --package qmp-qemu-socket -- --include-ignored`
6. `RUSTDOCFLAGS="-D warnings" cargo doc --locked --offline --package qmp-qemu-socket --no-deps --document-private-items`
7. `cargo build --locked --offline --release --package qmp-qemu-socket --target x86_64-unknown-linux-gnu`
