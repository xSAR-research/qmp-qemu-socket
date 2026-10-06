# Spider v1.4.0, candidate 2

## Result and exact base

This narrow update starts from Charlie's pushed candidate 1 at exact commit
`5a0f730a4248d030ae98fb466a00b092b8083133`. A fresh main checkout matched the
announced commit, package/lockfile version 1.4.0 and runtime candidate 1, with
a clean initial tree. No subsequent remote poll is used. The runtime label is
now candidate 2; the package version stays 1.4.0.

Charlie reports the application worked beautifully, with one failing native
restart test and frequent extra observations/Solver refreshes during stack
animation. No gameplay defect was reported in this turn.

The only behaviour change is the requested initial **After Spider card / run
action** value: **1250 ms**, still editable from 0 to 5000 ms. The same existing
setting also supplies Solver activation/refresh settling. DRAW remains 2000 ms,
repeat observation 1000 ms, game start 3000 ms, continuous Multi-Step 0 and the
existing bounded observation budget. No source/action/terminal logic changes.

## Corrected restart test

`continuous_win_restart_handles_optional_level_up_and_spider_play` previously
counted every wait equal to the default game-start duration and expected one.
Both score-skip's `LEVEL_UP_APPEAR_DELAY` and Spider's default game-start wait
are 3000 ms. The correct controller sequence therefore produced two matches.
The test conflated durations with purposes; its failure was not evidence of a
second game-start delay.

The corrected test uses an editable 3500 ms start delay and checks the complete
ordered waits and terminal controls. It covers LEVEL UP present/absent crossed
with Solver inactive/already active after Play. It checks one activation only
when required, one source action, and no extra start wait. The fixture queue
ends deliberately to terminate continuous mode; this is not a guest failure.
The default-timing assertion is updated to 1250 ms.

## Preserved boundaries

Spider continues fresh HALO, one click/D, editable settle, fresh capture. There
is no card identity or changed-pixel effect matching. The solid source and
toolbar Y947 geometry, automatic packing and optional LEVEL UP flow are
unchanged. All earlier mode policies, QMP/capture/snapshot/logging, STOP and
uncertain-input non-replay remain. Original fixtures are already in the base
and are not redistributed or changed by this candidate.

Dependencies, edition 2024, Rust minimum 1.101.0, nightly, rustfmt configuration
and double blank lines remain. No formatter or warning suppression is added.
The Beast's root AGENTS.md remains untracked and protected. No companion
AGENTS.md was available in this intake; the handoff/current instructions apply.

## Validation and Beast acceptance

Executed source/installer checks and tool versions are recorded in the delivery
review and validation archive. Rust/Cargo/rustup are absent in this workspace;
native tests, compilation, Clippy and live QMP are not claimed. Charlie's
candidate 1 gameplay report is separate from candidate 2 validation.

Run the supplied full Beast sequence, stopping at the first failure. Confirm
v1.4.0, candidate 2 and the initial 1250 ms PARAM. Check the previously failing
restart test and full suite. Verify stack animations produce fewer early
no-HALO captures/Solver refreshes, without assuming they are eliminated. Test
GAME WIN with or without LEVEL UP and resumed play after the normal deal wait.
This delivery stops at Gate 2; Charlie commits/pushes only after verification.

## Code Analysis

- **Threats:** no new input or scene authority is introduced. Existing fresh
  Solver targets and expected terminal controls continue to authorise input.
  The installer guards the new exact base, content, modes and affected index.
- **Memory safety:** production changes are one bounded duration constant and
  release text. No allocation, unsafe code, frame storage or pixel access is
  changed. Native compilation remains unverified in this workspace.
- **Bounds:** the existing 0–5000 ms timing range and per-run snapshot remain;
  geometry, observation and finite-action bounds do not change. Tests exercise
  a distinct editable start value without changing the default.
- **Failures:** longer settling retains cancellable waits. Missing HALOs,
  unsupported scenes, transport failure and uncertain input retain existing
  bounded stop/recovery policies. The corrected test distinguishes two valid
  waits of equal duration rather than relaxing runtime validation.
