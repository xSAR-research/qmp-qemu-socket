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
  - `TARGET_STOCK`: `PixelRect::new(700, 665, 280, 217)` (wide capture zone accommodating left-to-right horizontal contraction as cards are drawn from the Stock fan)
  - `STOCK_CARD_BOUNDS`: `PixelRect::new(832, 682, 136, 182)` (initial `T = 0` Stock top position)
- **Waste card:**
  - `WASTE_CARD_BOUNDS`: `PixelRect::new(1017, 682, 136, 182)`
- **Shared Undo All control:**
  - `SHARED_UNDO_ALL_CONTROL`: `(1320, 978)`

### 2.3 Covering and Legal-Move Rules

- A Tableau card at `(row, col)` (for `row in 1..=3`) becomes uncovered if and only if both covering child slots below it on `row + 1` are empty (`Empty`). Covered cards cannot be played or read while `FaceDown`.
- An uncovered Tableau card is legal to play if and only if its rank differs from the active Waste rank by `±1` with wrap-around (`King <-> Ace <-> 2`, i.e. circular modulo-13 distance `1`).
- Drawing from Stock (`D`) moves the next Stock card onto Waste whenever at least one Stock card remains; Undo All resets the board to its exact `t = 0` deal state (`Row 4` + `Waste 28` + `23` Stock cards) when no terminal loss lock blocks recovery (see Section 5).

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
- All TriPeaks corner/back/felt tolerances must honour Beast's calibrated foreground and card-back profiles without weakening margins to force test passes.

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

> **CRITICAL FLAG FOR GOAL 2 (BEAST CALIBRATION) & GOAL 4 (`initialize_and_sweep_stock`):**
> During Goal 2 live Beast inspection and Goal 4 Stock sweep implementation, explicitly verify what *Microsoft Solitaire & Casual Games* TriPeaks renders on the **23rd Stock draw (`D`)** when **0 Stock deals remain** AND **zero Row-4 cards match `±1` of the 23rd Waste card**.
>
> In particular:
> - Check whether TriPeaks immediately triggers a **`"No More Moves"` / `"Out of Moves"` modal overlay** (or disables toolbar controls / darkens the board) on Draw 23.
> - Ensure `initialize_and_sweep_stock` handles that exact state cleanly: either **cleanly back out / dismiss / undo out of any `"No More Moves / Out of Moves"` modal before clicking `SHARED_UNDO_ALL_CONTROL` `(1320, 978)`**, or **avoid triggering the terminal lock when appropriate** (for example, stopping/reversing safely when Waste transition proves empty stock exhaustion or checking if a matching Row-4 card already proves Undo availability).
> - Never issue blind clicks onto `SHARED_UNDO_ALL_CONTROL` `(1320, 978)` while a modal overlay owns the guest screen, and treat any unrecognised overlay as `SceneVerification` / no input authority until calibrated and proven in tests.

Additional Goal 2 Beast calibration items to capture and verify against live screenshots:
- Classic vs Daily Challenge TriPeaks HUD geometry, `Stock` empty slot felt/icon pixels across `1..=23` horizontal stack steps, `No More Moves / Out of Moves` overlay bounds & dismiss/undo buttons, `GAME WIN` / `Level Up` banners, and card-back vs face-up RGB distributions on Rows 1–4.

---

## 6. JSON Deal Codecs, Search Adapter, Operator UI & Execution (Goals 3–6)

### 6.1 State Separation & Atomic JSON Persistence

- Separate immutable **initial deal / card-slot map**, **raw recognition observations**, **planned/recorded move lists** (`Play(row, col)` / `Draw`), and **mutable simulation occupancy**. `Unknown` always blocks dependent parent plays and is never treated as `Empty`.
- Enforce the `65,536`-byte maximum input file limit and validate the full schema before encoding or publishing.
- Atomic export uses same-directory temporary file creation (`create_new` / `no-overwrite`) and post-error destination inspection on any I/O uncertainty.

### 6.2 Search & Domain Adapter (`xsar::search`)

- Use `xsar::search` (A*/Dijkstra) strictly from `qmp-qemu-socket` without modifying `../xsar`.
- Support both full-information solving (once sufficient cards are known to prove a complete win path from root) and partial-information frontier reachability (`ReconDomain::plan_info_gain_probe` targeting maximal newly uncovered `Unknown` slots per probe).

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
- **Goal 2 (Beast Calibration):** Capture live Beast screenshots across Stock `0..23`, Draw 23 empty-stock with/without Row-4 matches, `No More Moves / Out of Moves` overlay behaviour, cleared slots, and terminal `GAME WIN` screens; lock coordinates and overlay guards.
- **Goal 3 (Pure Model & `xsar::search` Adapter):** Implement 52-slot TriPeaks board/rules/solver, JSON deal codecs (`65,536`-byte cap, atomic temp-file export), and deterministic unit tests.
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
