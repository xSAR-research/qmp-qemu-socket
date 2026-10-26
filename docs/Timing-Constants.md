# Timing constants and tuning reference

## Current correction — 2.9.149 / xSAR 0.9.0

The bounded Wild West aged-paper correction is host qualified: 1483 application
tests, 753 sealed inputs and all six required gates passed. Fresh Beast behaviour
and Charlie's acceptance remain pending.

The original bright-paper path is retained. When it is insufficient, measured aged
material and a complete Western glyph family support the two retained entry frames.
Stock keeps the inclusive 45% presence threshold over its full 139×187 footprint.
Unreadable occupied stock remains Unknown. Aged material above the existing 5%
allowance also vetoes ordinary/final-band absence; it grants no presence or input
authority by itself.

The two actual frames provide 58 visible identities, with 25 exact rank-by-colour
exemplars and 33 whole-card held-outs. No black Ten was available; black Nine, red
Jack and red Queen have calibration-only same-rank/colour evidence. All 423 Classic
and 87 cream/Retro observations remain covered. Quality/ambiguity limits, complete
scene/modal proof, STOP/context, fresh probes and no uncertain replay remain intact.
The separate Boards/Perfect final-proof case remains deferred.

See [verification](Verification-2.9.149.md) and the
[fresh live-test brief](Live-Test-2.9.149.md). Earlier release sections are
retained evidence and do not establish 149 guest acceptance.

## Retained correction — 2.9.148 / xSAR 0.9.0; host qualified

The game and solution heading starts at the top left, with wrapping options below
it and the fixed activity badge alongside on the right. Action controls, status
and preview retain the full panel width. The displayed execution label is
**SOLVING**; face size, centred bold text, border, colours and halo timing remain.

This presentation revision preserves 147 gameplay and search ownership. Recognition
resolution for the saved wood/aged-paper entry failure is held for further testing;
its exact-reader diagnosis is recorded in the verification brief. No recognition,
worker, dependency or timing policy changes are included.

Host result: 1478 full tests, zero failures/ignored/filtered, 747 sealed inputs and all six required gates passed. Fresh Beast/Charlie acceptance remains pending.
The 19 matrix cases and 14 milestone states remain unchanged. See
[148 verification](Verification-2.9.148.md) and [148 manual checks](Live-Test-2.9.148.md).

## Retained 2.9.147 candidate: direct completed-RECON handoff

Source inspection found no artificial wait after completed Recon for the RECON
button or START's acquisition → planning → execution paths. No timing edit is
required. Once the restored deal has been validated/published and its receipt
authenticated, manual RECON returns directly and automatic START proceeds directly
to planning, then execution according to the selected lifecycle.

The existing 1000 ms UndoAll/OK settle is **inside Recon**, after the acknowledged
confirmation and before the first of two positive original-board restoration
observations. Preserve it, the two observations, conditional stale-scene retry
waits, DRAW settling, cancellation slices, effect deadlines and capture budgets.
Do not move or remove a restoration proof to improve the apparent completion time.

Reward/counting continuation and the explicit Shortest-to-Maximum switch introduce
no new fixed wait. The switch retains PAUSE until CONTINUE and uses the same
authenticated RECON; Maximum keeps its uncapped weighted search with no completion-
time guarantee. Receipt/publication, search, fresh probes and capture latency are
real work, separate from intentional waits. The source audit is not a new timing
measurement or guest acceptance. 147 passed all six host gates (1476 full tests, 747 sealed inputs); see
[verification](Verification-2.9.147.md) and [manual checks](Live-Test-2.9.147.md).

## Retained 2.9.146 candidate: display-only halo timing

Full 146 host qualification passed (1449 tests, 743 sealed inputs); see [verification](Verification-2.9.146.md).
Gameplay waits, cancellation slices, effect deadlines and capture budgets retain
their existing values. The activity badge adds display-only timing in
`src/app/activity_status/badge.rs`; it neither sleeps nor starts a worker.

| Display value | Value | Scope |
| --- | --- | --- |
| Active halo period | 1.6 seconds | RECON, ROUTE and SOLVING only |
| Active repaint request | 25 ms | egui repaint request; not an input or capture cadence |
| Halo level | 0.15 to 0.85 | Surrounding glow only; face colour stays constant |
| Static states | WAITING and ERROR | No halo animation or active repaint request |
| Fixed face and reserved halo | 240×80 and 10 points | Logical points, independent of label length |

Independent inert render review passed 20 cases; this is not guest timing evidence.
The following qualified145 and older timing records remain historical evidence.

Retained 145 is host qualified (1436 full tests): [stock-paper correction](Verification-2.9.145.md). No timing, deadline or capture-budget values change. The separate final-confirmation handoff remains open.

Retained 144 is host qualified (1434 full tests): [temporal RIGHT correction](Verification-2.9.144.md). No timing, deadline or capture-budget values change. The separate run 8 final-confirmation timeout remains open.

Retained 140 is host qualified (1422 full tests): [session-guard qualification](Verification-2.9.140.md).
No timing, deadline or capture-budget values change in this correction.

Retained 139 is host qualified (1420 full tests): [qualification and pause accounting](Verification-2.9.139.md).
Intentional acknowledged suspension is excluded from A* active deadlines/statistics;
ordinary effect windows and MOVE/CV timings are unchanged. The older catalogue follows.

Retained 138 correction: [2.9.138 empty-circle verification](Verification-2.9.138.md).
Its stock-position change adds no timing changes. The combined 2.9.137 timing and
UI qualification below is retained evidence; 2.9.138 independently passed 1404 tests
and all six host gates with 698 seals. Fresh acceptance remains pending.


## Retained combined 2.9.137 roles

The catalogue below retains the original 2.9.134 timing refactor. Its no-new-dependency
statement applies to that delivery; the current app adds Unix signal-hook for shutdown.
Current corrections are authoritative where their roles differ:

| Owner constant | Value | Current purpose |
| --- | --- | --- |
| `COMPUTED_ROUTE_MOVE_SETTLE_DELAY` | 250 ms | Computed planned and residual DRAW/MOVE only |
| `COMPUTED_ROUTE_REOBSERVE_DELAY` | 50 ms | Read-only ordinary computed effect retry |
| `COMPUTED_ROUTE_EFFECT_WINDOW` | 2 s | Last whole-action ACK deadline; includes settle |
| `PYRAMID_MOVE_SETTLE_DELAY` | 1000 ms | Planned non-final computed RECYCLE |
| `PYRAMID_MOVE_SETTLE_DELAY_MS` | 1000 ms | Unchanged Computer Vision HALO setting |
| `PYRAMID_CARD_SETTLE_DELAY_MS` | 2000 ms | Unchanged CV/manual card setting; the old Duration alias was removed |

Computed MATCH/KING observe immediately; no two-second fixed card sleep remains.
Residual RECYCLE and Recon retain their separate 500 ms policy. Final and REDEAL
policies stay independent. See [combined evidence](Verification-2.9.137.md).

## Retained timing catalogue

The 2.9.134 refactor preserved every previous timing value. The 2.9.135 correction separates computed MOVE 250 ms from Computer Vision/RECYCLE. The 2.9.136 correction removes fixed computed MATCH/KING settling and uses immediate observation with bounded 50 ms retries; the live 136 log provides cadence samples, while full acceptance remains open. The 2.9.137 correction adds a separate two-second ordinary-effect window from the last whole-action ACK, including settle, without imposing a fixed two-second wait. Values are configured policies, not measured minimums. Tests, transport acknowledgements and a completed wait do not prove a guest effect.

## Source files

| Repository | Constants source | Ownership |
| --- | --- | --- |
| qmp-qemu-socket | [`src/parameters.rs`](../src/parameters.rs) | UI refresh, game animation, observation, input policy, execution bounds and logging |
| xsar | `src/parameters.rs` | Reusable QMP defaults, TCP deadlines/cancellation polling and search reporting/default budget |
| basic-route-planning | `src/parameters.rs` | Offline console search default |
| qemu-screen-grab-server | `src/parameters.rs` | Capture-service deadlines, reporting and shutdown polling |

xSAR owns the five shared QMP defaults. The application aliases these defaults, while existing `QmpConfig` overrides remain supported. Equal values with different purposes stay separate: a network deadline is not a game-animation wait, and a reporting interval never sleeps. Each repository keeps its relative sibling dependency; no new dependencies are introduced.

## First places to inspect for Pyramid pacing

| Setting | Configured value | Actual use |
| --- | --- | --- |
| `PYRAMID_CARD_SETTLE_DELAY_MS` | 2,000 ms | Unchanged Computer Vision card-animation default; computed MATCH/KING uses no fixed settle |
| `COMPUTED_ROUTE_REOBSERVE_DELAY` | 50 ms | Ordinary computed source/effect retry only when the captured position is not ready; no input is repeated |
| `COMPUTED_ROUTE_EFFECT_WINDOW` | 2 s | Ordinary-effect capture-start/retry window from the whole action's last ACK, including settle; immediate first read, finite derived 41-read cap, no input replay |
| `COMPUTED_ROUTE_MOVE_SETTLE_DELAY` | 250 ms | Planned and residual stock-drain MOVE only; fresh live reliability pending |
| `PYRAMID_MOVE_SETTLE_DELAY` | 1 s | Non-final planned RECYCLE; Computer Vision separately retains the 1,000 ms `PYRAMID_MOVE_SETTLE_DELAY_MS` default |
| `PYRAMID_DRAW_SETTLE_DELAY` | 500 ms | Each Recon D release and each stock-drain RECYCLE; stock-drain MOVE uses the separate 250 ms policy |
| `PYRAMID_REOBSERVE_DELAY` | 1 s | Recon observation/restoration and session/stock-boundary retry policies; ordinary computed retries use their separate 50 ms value |
| `PYRAMID_FINAL_OBSERVATION_DELAY` | 200 ms | Final-action settle and repeated final observations |
| `PYRAMID_FINAL_CONFIRMATION_WINDOW` | 2 s | ACK-relative final effect confirmation budget; expiry withholds success |
| `PYRAMID_CLEAR_HANDOFF_DELAY` | 3 s | NoChallenge full-clear handoff to fresh Recon after final ACK |
| `PYRAMID_REDEAL_SETTLE_DELAY` | 1.5 s | Guarded partial-board REDEAL before fresh Recon |

These waits do not all occur after every move. Retry intervals apply when another observation is needed; input holds, transport, capture/PNG work and synchronous processing contribute additional time. The first action acquires one source frame; each verified effect supplies the next action's source. Ordinary source checks retain six reads, effects have the ACK-based window, and final confirmation retains its independent 200 ms policy and final budget. The normal game's `BOARD_REDEAL_SETTLE_DELAY` remains 4 s because it has a different workflow from partial-board REDEAL.

Animation settings already exposed by the UI still use their existing immutable execution snapshot and clamps. The computed A* executor uses the compiled Pyramid defaults rather than mutable UI animation settings. Changing a Rust constant requires rebuilding and restarting the affected executable; editing source does not change an already running process. Changing a default does not overwrite an existing operator setting.

## qmp-qemu-socket

Definitions are in `qmp-qemu-socket/src/parameters.rs`. Values below include aliases and millisecond defaults so each timing declaration can be found by name.

| Constant | Value | Role | Source expression / shared default |
| --- | --- | --- | --- |
| `CANCELLABLE_WAIT_SLICE` | 25 ms | Maximum interval between cancellation checks during intentional waits. | `Duration::from_millis(25)` |
| `MINIMUM_ANIMATION_SETTLE_DELAY_MS` | 0 ms | Smallest editable animation-settle interval in milliseconds. | `0` |
| `MAXIMUM_ANIMATION_SETTLE_DELAY_MS` | 5 s | Largest editable animation-settle interval in milliseconds. | `5_000` |
| `DRAW_ANIMATION_SETTLE_DELAY_MS` | 500 ms | Default TriPeaks draw-to-capture settling interval in milliseconds. | `500` |
| `TABLEAU_ANIMATION_SETTLE_DELAY_MS` | 750 ms | Default TriPeaks card-to-capture settling interval in milliseconds. | `750` |
| `BOARD_REDEAL_SETTLE_DELAY_MS` | 4 s | Default shared board-redeal settling interval in milliseconds. | `4_000` |
| `PYRAMID_MOVE_SETTLE_DELAY_MS` | 1 s | Default Pyramid Move/Recycle-to-capture interval in milliseconds. | `1_000` |
| `PYRAMID_CARD_SETTLE_DELAY_MS` | 2 s | Default Pyramid card or pile click-to-capture interval in milliseconds. | `2_000` |
| `PYRAMID_REOBSERVE_DELAY_MS` | 1 s | Default Pyramid no-halo recapture interval in milliseconds. | `1_000` |
| `TRIPEAKS_REOBSERVE_DELAY_MS` | 1 s | Default TriPeaks input-free interval while waiting for a late Solver halo. | `1_000` |
| `KLONDIKE_SETTLE_DELAY_MS` | 750 ms | Klondike action settling interval selected from Charlie's Beast gameplay. | `750` |
| `KLONDIKE_SOLVE_SETTLE_DELAY_MS` | 750 ms | Separate editable Solve animation interval, retaining the existing delay until Charlie measures a more suitable value on the Beast. | `KLONDIKE_SETTLE_DELAY_MS` |
| `KLONDIKE_REOBSERVE_DELAY_MS` | 1 s | Initial Klondike input-free result recapture interval in milliseconds. | `1_000` |
| `FREECELL_SETTLE_DELAY_MS` | 750 ms | Initial editable Free Cell action settle; Beast gameplay may refine it. | `750` |
| `FREECELL_REOBSERVE_DELAY_MS` | 1 s | Initial editable Free Cell input-free observation interval. | `1_000` |
| `FREECELL_GAME_START_DELAY_MS` | 3 s | Editable delay measured by Charlie for starting and dealing a Free Cell game. | `3_000` |
| `SPIDER_SETTLE_DELAY_MS` | 1.25 s | Editable Spider source-action settle tuned by Charlie after Beast gameplay. | `1_250` |
| `SPIDER_DRAW_SETTLE_DELAY_MS` | 2 s | Spider stock-deal delay from Charlie's observed one-to-two-second deal. | `2_000` |
| `SPIDER_REOBSERVE_DELAY_MS` | 1 s | Initial editable Spider input-free observation interval. | `1_000` |
| `SPIDER_GAME_START_DELAY_MS` | 3 s | Initial Spider deal delay, borrowing the accepted Free Cell start default. | `3_000` |
| `DRAW_ANIMATION_SETTLE_DELAY` | 500 ms | Default TriPeaks draw settling interval as a typed duration. | `Duration::from_millis(DRAW_ANIMATION_SETTLE_DELAY_MS)` |
| `TABLEAU_ANIMATION_SETTLE_DELAY` | 750 ms | Default TriPeaks tableau settling interval as a typed duration. | `Duration::from_millis(TABLEAU_ANIMATION_SETTLE_DELAY_MS)` |
| `BOARD_REDEAL_SETTLE_DELAY` | 4 s | Default shared board-redeal settling interval as a typed duration. | `Duration::from_millis(BOARD_REDEAL_SETTLE_DELAY_MS)` |
| `NO_HIGHLIGHT_REOBSERVE_DELAY` | 150 ms | Short TriPeaks planning/effect recapture delay; not Solver-recovery timing. | `Duration::from_millis(150)` |
| `POST_GAME_STAGE_DELAY` | 1 s | Settling interval after ordinary post-game control clicks. | `Duration::from_secs(1)` |
| `LEVEL_UP_APPEAR_DELAY` | 1 s | Delay after score-skip clicks before looking for Level Up. | `Duration::from_millis(1_000)` |
| `LEVEL_UP_CONFIRMATION_DELAY` | 200 ms | Fresh geometry confirmation after a settled-size OK with a current black glyph. This replaces only that lookup's next retry wait, including its final reserve. | `Duration::from_millis(200)` |
| `BOARD_TRANSITION_REOBSERVE_DELAY` | 2 s | Delay between observations of an unresolved board transition. | `Duration::from_secs(2)` |
| `POINTER_SETTLE_DELAY` | 100 ms | Pause after absolute pointer movement before a gameplay mouse press. | `xsar::parameters::POINTER_SETTLE_DELAY` |
| `MOUSE_HOLD` | 50 ms | Ordinary gameplay mouse-button hold duration. | `xsar::parameters::MOUSE_HOLD` |
| `POST_GAME_MOUSE_HOLD` | 500 ms | Longer button hold used for calibrated post-game controls. | `Duration::from_millis(500)` |
| `SOLVER_MOUSE_HOLD` | 500 ms | Button hold used when enabling Solver highlighting. | `Duration::from_millis(500)` |
| `KEY_HOLD` | 20 ms | Draw-key hold duration between QMP key-down and key-up. | `xsar::parameters::KEY_HOLD` |
| `QMP_IO_TIMEOUT` | 750 ms | Read/write timeout for ordinary QMP protocol traffic. | `xsar::parameters::QMP_IO_TIMEOUT` |
| `QMP_SCREENDUMP_TIMEOUT` | 5 s | Extended QMP timeout for PNG screen dump generation. | `xsar::parameters::QMP_SCREENDUMP_TIMEOUT` |
| `UI_REPAINT_INTERVAL` | 100 ms | Idle UI refresh interval for consuming worker progress and preview updates.  Preserve the existing responsiveness/idle-work trade-off; this is not a guest wait. | `Duration::from_millis(100)` |
| `PYRAMID_RECON_DEADLINE` | 300 s | Cooperative full-Recon deadline, including restoration and observation waits.  This is a safety ceiling, not a delay added to each operation; blocking I/O retains its separate transport timeout and may finish after the deadline. | `Duration::from_secs(300)` |
| `PYRAMID_DRAW_SETTLE_DELAY` | 500 ms | Fixed settle after a Pyramid stock D release, shared by Recon and stock drain.  Preserve the paired-Recon pacing and drain policy; acknowledgement alone never proves the new stock position. Ordinary planned MOVE actions use their own wait. | `Duration::from_millis(500)` |
| `PYRAMID_MOVE_SETTLE_DELAY` | 1 s | Compiled ordinary route MOVE/RECYCLE settle, sharing the Pyramid UI default.  This uses the default value, not the operator's mutable settings snapshot. | `Duration::from_millis(PYRAMID_MOVE_SETTLE_DELAY_MS)` |
| `PYRAMID_CARD_SETTLE_DELAY` | 2 s | Compiled route MATCH/KING settle, sharing the Pyramid card-animation default.  Final actions use the shorter bounded final-observation interval instead. | `Duration::from_millis(PYRAMID_CARD_SETTLE_DELAY_MS)` |
| `PYRAMID_REOBSERVE_DELAY` | 1 s | Interval between Pyramid position proofs, stale observations and transition probes.  Shared by Recon, ordinary route verification and session entry/transition. It also preserves the post-confirmation settle; final-action checks differ. | `Duration::from_millis(PYRAMID_REOBSERVE_DELAY_MS)` |
| `PYRAMID_FINAL_CONFIRMATION_WINDOW` | 2 s | Observation-only final confirmation budget measured from the last complete ACK.  Expiry withholds success; it never authorises retransmission of an input. | `Duration::from_secs(2)` |
| `PYRAMID_FINAL_OBSERVATION_DELAY` | 200 ms | Final-action settle and subsequent observation interval within the final budget.  Preserve short cancellable observations instead of ordinary card-animation waits. | `Duration::from_millis(200)` |
| `PYRAMID_EXECUTION_DEADLINE` | 1200 s | Cooperative route deadline covering initial proof, gestures, waits and transport.  This execution safety ceiling is independent of operator-controlled search. | `Duration::from_secs(1200)` |
| `PYRAMID_CLEAR_HANDOFF_DELAY` | 3 s | NoChallenge pause after the final clear-action ACK before fresh Recon.  This is the existing three-second policy, not proof that dealing has finished; fresh starting-board recognition must independently establish readiness. | `Duration::from_secs(3)` |
| `PYRAMID_REDEAL_SETTLE_DELAY` | 1.5 s | Pause after a guarded partial-board REDEAL ACK before fresh Recon.  Preserve the existing 1.5-second policy, separately from the normal board's four-second redeal default; readiness is still checked from fresh pixels. | `Duration::from_millis(1500)` |
| `SEARCH_LOG_INTERVAL` | 1 s | Minimum spacing between intermediate search log rows; final statistics always log.  This bounds reporting overhead and never sleeps or limits the search itself. | `Duration::from_secs(1)` |
| `TCP_CAPTURE_SUMMARY_INTERVAL` | 60 s | Aggregate TCP capture telemetry interval; frame requests retain their own cadence. | `Duration::from_secs(60)` |

## xsar

Definitions are in `xsar/src/parameters.rs`. Values below include aliases and millisecond defaults so each timing declaration can be found by name.

| Constant | Value | Role | Source expression / shared default |
| --- | --- | --- | --- |
| `QMP_IO_TIMEOUT` | 750 ms | Ordinary QMP read/write budget; callers may choose a different I/O timeout. | `Duration::from_millis(750)` |
| `QMP_SCREENDUMP_TIMEOUT` | 5 s | Longer QMP budget for guest PNG generation, independent of ordinary replies. | `Duration::from_secs(5)` |
| `POINTER_SETTLE_DELAY` | 100 ms | Pause after absolute pointer movement before pressing a button. | `Duration::from_millis(100)` |
| `MOUSE_HOLD` | 50 ms | Ordinary button-down duration; application-specific controls may need longer. | `Duration::from_millis(50)` |
| `KEY_HOLD` | 20 ms | Key-down duration before release; delivery still needs separate acknowledgements. | `Duration::from_millis(20)` |
| `TCP_CONNECT_TIMEOUT` | 5 s | Bound one TCP connection attempt; this does not authorise automatic retries. | `Duration::from_secs(5)` |
| `TCP_SOCKET_TIMEOUT` | 10 s | Independent absolute budget for writing a request or reading each response stage. | `Duration::from_secs(10)` |
| `TCP_CANCEL_POLL` | 100 ms | Maximum blocking socket slice between cancellation and deadline checks. | `Duration::from_millis(100)` |
| `DEFAULT_SEARCH_DURATION` | 120 s | Default search time allowance; each caller retains its own explicit limits. | `Duration::from_secs(120)` |
| `SEARCH_PROGRESS_INTERVAL` | 200 ms | Minimum interval between non-final observer callbacks to bound reporting overhead. | `Duration::from_millis(200)` |

## basic-route-planning

Definitions are in `basic-route-planning/src/parameters.rs`. Values below include aliases and millisecond defaults so each timing declaration can be found by name.

| Constant | Value | Role | Source expression / shared default |
| --- | --- | --- | --- |
| `DEFAULT_SEARCH_DURATION` | 120 s | Default offline search budget; an explicit CLI time limit overrides this value.  Preserve the established two-minute allowance, independently of app policy. | `std::time::Duration::from_secs(120)` |

## qemu-screen-grab-server

Definitions are in `qemu-screen-grab-server/src/parameters.rs`. Values below include aliases and millisecond defaults so each timing declaration can be found by name.

| Constant | Value | Role | Source expression / shared default |
| --- | --- | --- | --- |
| `CLIENT_IDLE_TIMEOUT` | 120 s | Idle first-byte budget so persistent clients cannot retain a connection forever. | `Duration::from_secs(120)` |
| `REQUEST_TIMEOUT` | 10 s | Absolute budget for the remaining bytes of one request after its first byte. | `Duration::from_secs(10)` |
| `CAPTURE_QUEUE_TIMEOUT` | 2 s | Maximum queue residence before capture starts, bounding obsolete waiting work. | `Duration::from_secs(2)` |
| `RESPONSE_TIMEOUT` | 10 s | Capture-result wait budget and then the shared header/pixels/flush send budget. | `Duration::from_secs(10)` |
| `STATISTICS_INTERVAL` | 60 s | Interval for aggregate service statistics, independent of request pacing. | `Duration::from_secs(60)` |
| `SHUTDOWN_POLL_INTERVAL` | 20 ms | Capture-owner and listener polling interval for cooperative shutdown checks. | `Duration::from_millis(20)` |
| `RESPONSE_POLL_INTERVAL` | 10 ms | Result-wait polling interval for abandoned requests and shutdown detection. | `Duration::from_millis(10)` |

## Boundaries when tuning

- Settle delays and input holds preserve existing pacing; a shorter wait still requires the same fresh, exact effect proof. Never compensate for a missed animation with uncertain input replay.
- `CANCELLABLE_WAIT_SLICE`, socket cancellation polls and server shutdown polls control responsiveness. Increasing them can delay STOP or shutdown; they are not animation tuning controls.
- QMP/TCP/service timeouts and Recon/execution deadlines bound operations. Increasing them can prolong failure detection. They are separate from the application's operator-controlled search, which still has no application state/expansion/time/byte cutoff.
- xSAR and console search defaults remain 120 s for callers using those defaults. Explicit caller limits and `--max-seconds` are preserved. Search callback/log intervals reduce reporting overhead and never pause search.
- Preserve fresh frame, scene/modal/card/pile, context, epoch and STOP guards. A 3 s handoff or 1.5 s REDEAL wait does not establish readiness; fresh Recon still must prove it.
- Test-only durations remain independent fixtures. Measured elapsed-time conversions, unit conversions, `Duration::ZERO`, `Duration::MAX` and caller-provided durations are not hidden pacing literals.
- Linux runtime tests for the capture server do not validate Windows GDI capture or guest animation timing. Live tuning needs fresh native evidence, several reliable repetitions and a measured margin.

## Verification

See [Verification-2.9.145.md](Verification-2.9.145.md) for the current stock-paper correction; it changes no timing values. Retain [Verification-2.9.137.md](Verification-2.9.137.md) as historical timing evidence.
The original refactor's [Verification-2.9.134.md](Verification-2.9.134.md) records its actual owning-package tests, feature checks, preserved source/fixture hashes and host/live boundaries. No guest actions, installation or process replacement are part of this refactor.
