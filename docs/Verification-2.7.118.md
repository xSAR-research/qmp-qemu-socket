# App 2.7.118 / xSAR 0.7.103 — Continuous and removal fallback

## Qualification boundary

**Host verified: app 2.7.118 / xSAR 0.7.103.** The corrected app passed all 1,218
tests; xSAR passed 278, six feature configurations and the private unchanged-source
console passed 80. Required checks, strict private documentation, Clippy, scoped
formatting, release builds and native/protected help passed. Versions are now
qualified. Final documentation/preservation audit passed; fresh live acceptance
is separate. No guest trial, installation, running-process replacement or Git
promotion was performed or authorised by this record.

The current implementation enables Continuous on measured normal `Boards: 1 of 3`,
`2 of 3` and `3 of 3` scenes, retaining same-frame identity, exact starting-board
witness, independent replay, whole-action ACK/effect checks, guarded final outcome
and Solver-free restart. Mid-game entry reads the actual ordinal. A challenge
denominator of three supplies no normal-series identity. Supported end scenes use
the existing bounded restart controller; no separate menu screenshot prerequisite
has been added.

The default **Least moves (weighted A*)**, weight two, seeks short winning routes.
It does not establish a minimum action count. The exact option uses weight one;
only a successfully popped exact goal establishes minimum unit action cost. A goal
retained at a resource cutoff is labelled as having no minimum proof. The normal
UI shows method, elapsed time, examined positions and candidate moves; the completed
report includes stored histories. The exact statistics, including peak frontier,
also remain in diagnostic evidence.

## Removal fallback and execution authority

The additive xSAR API leaves `solve` and `solve_with_options` unchanged. Full-goal
search retains sound impossibility pruning. If it cannot return a full route,
remaining aggregate record, expansion and time budgets permit an unpruned Dijkstra
phase. A primary exhaustive Dijkstra traversal already supplies the exhaustive
fallback proof. The two phases share the original deadline and cumulative counters;
they do not receive two search budgets.

Fallback maximises **tableau cards removed**, then minimises logical action count
at that removal count. Stock-only removals earn no score. An exhausted unpruned
graph proves the maximum and shortest equal-score route. A resource cutoff reports
only the best discovered prefix; neither maximum removal nor shortest cost is
claimed. Safe cutoffs retain a route reconstructed before the cutoff. Cancellation,
allocation failure and invariant failure expose no executable actions, including
STOP raised by the final progress callback.

App policy retains the existing 2,000,000-history, 2,000,000-expansion and
120-second bounds with allocator-checked storage and no fixed byte ceiling.
Shared defaults remain 512 MiB estimated storage. Candidate output capacities count
against that conservative estimate; it is not a process RSS guarantee. The existing
102-action, 126-gesture and 1,200-second execution bounds remain unchanged.

Every selected route is independently replayed against the exact authenticated
same-session deal. A partial route must have a positive, non-clear tableau score,
matching removed and remaining counts, and end in a real tableau removal. Its last
whole action is acknowledged before the existing two-second observation window.
Only ordinary exact-position verification can confirm that non-clear endpoint;
Perfect, terminal and next-board alternatives cannot confirm it. The result reports
removed/remaining cards and stops both automatic modes, including on board three.
It grants no board-clear, series-transition or restart authority. Unknown effect,
STOP and context change never cause replay or cleanup input.

This is a general maximum-removal fallback. Rank-specific challenge targets,
challenge score/redeal rules, alternate themes and the intermittent idle CPU report
remain separate open work.

## Measured native references and preservation

Three unchanged operator Capture PNG originals are retained under
`tests/fixtures/pyramid-standard-series/`. Only full header patches for ordinals two
and three are added. The complete existing header region is `[1658,46,160,32]`,
tolerance 12 per channel; the New footer is `[212,998,58,31]`, tolerance 10.
No digit substitution, OCR, changed tolerance or local ordinal counter is used.

| Original | Bytes | SHA-256 |
| --- | ---: | --- |
| `normal-1.png` | 3762879 | `c33474448890492aa940bde05e0cedcee5d308b03a62312bce0d997a1e923124` |
| `normal-2.png` | 3778708 | `db17b6ced8d5059de485a7ba24245c6147e6db5ee80c97f8a90343948d1906f5` |
| `normal-3.png` | 3768291 | `7471256cbb6daaf4b8132346bd33d0d88ccc0e8be79d1743c6a2802a181180a6` |

Derived `header-2.rgb` and `header-3.rgb` are 15,360 bytes each, SHA-256
`159ecff92bcf9b29f8f2bc8b26b1ccccf3de1db334c9818e988ea53c9208df59`
and `f94b88f6663abb1db4fec19f556d6c169bab97a655ca4314cadc78dd13692d93`.
Original external filenames, all older fixture bytes and the native-v17 card
recognition/final-clear contract remain frozen.

The immutable baseline is
`/home/charlie/tmp/pyramid-continuous-fallback-validation-Vs54oXto/baseline.json`,
SHA-256 `8e516bbbccb0a9f6b9e328324819a9847bf6bd9b128b0e66b58a34b029cf85ea`.
It records all three checkout file maps, HEADs and index identities before this
feature. Console 0.8.4 remains byte-for-byte frozen. Its historical xSAR lock entry
is not rewritten; any compatibility smoke uses a private copy and records its
local lock adjustment. xSAR owns the additive reusable search API; the app owns
native calibration, game policy, execution and STOP.

## Actual verification evidence

Focused logs are retained in
`/home/charlie/tmp/pyramid-continuous-fallback-validation-Vs54oXto/`.
Earlier shared engine checks passed 8 tests and public Pyramid checks passed 3.
Earlier app filters passed 20 partial-related tests, the original-series observer,
native mid-game entry, all three literal ordinals and the Continuous UI gate.
The passing full scope includes the subsequent final-callback cancellation and
replayed-removal mismatch regressions.

Original compile failures are retained: `compile-focused.log` contains E0425 for
`TABLEAU_LEN`, E0433 for a test-only `SearchOutcome` import, and E0063 for missing
test `Prepared.objective` fields. Explicit imports and Clear-objective constructors
corrected them; the no-run rerun passed. `shared-focused-first-failure.txt` retains
E0277 from an invalid test `SearchMethod::default`; an explicit AStar method fixed
the test. No production recognition or resource policy was relaxed.

The first full app run is retained at
`/home/charlie/tmp/pyramid-continuous-fallback-validation-Vs54oXto/runs/0001/app-tests.log`:
**1,215 passed, three failed, zero ignored/filtered**, 324.893 seconds command /
310.73 seconds test report. These assertions still described superseded behaviour:

| Test | Original failure and correction |
| --- | --- |
| `automatic_start_requires_an_explicit_automatic_mode_and_idle_controller` | `src/app/pyramid_workflow_tests.rs:575:9`: left `true`, right `false`. Both explicit automatic modes now start while idle; Manual, busy and outstanding-token refusals remain asserted. |
| `incomplete_route_is_refused_before_any_capture_or_input` | `src/worker/computed_route/tests.rs:1179:5`: `error.contains("remaining tableau cards")`. Require the new `match the replayed objective` refusal text and retain no-capture/no-input assertions. |
| `owned_fast_calculation_retains_authority_and_final_weighted_search_statistics` | `src/worker/computed_route/tests.rs:798:5`: `report.contains("method=Fast; independent replay passed")`. Require the new visible `Least moves (weighted A*)` label; authority, weighted proof and exact final-statistics assertions remain. |

Only those test expectations changed after the first full run. No production
source, policy, tolerance or resource bound changed. The corrected app's full
required scope passed as `runs/0002`; the original failure remains intact.
The shared source, fixture and artifact hashes are unchanged, so 18 successful
shared/matrix/neutral-tree/private-console commands are retained through
`shared-successes.json`, SHA-256
`3ff71cc1e76641b744c2ce4cc4c4c18192308ef8f7dbb77120688e8592f9489b`,
without repeating those workloads. Shared full tests passed **278/0/0**, the six
overlapping feature configurations passed **14/46/25/58/57/89**, and the private
console passed **80/0/0**, all with zero filtered. Matrix counts are not additional
unique tests. Strict shared check/private rustdoc/Clippy/release and all four
dependency-free neutral feature trees passed.

| Required evidence | Result |
| --- | --- |
| App locked all-target/all-feature check | Passed; 2.068 s |
| App full tests, include ignored, eight threads | Passed; 1,218/0/0/0 filtered; 319.022 s command / 315.93 s report |
| App strict private rustdoc / warnings-denied Clippy / scoped format | Passed; 1.266 / 7.277 / 0.465 s |
| xSAR locked all-feature check / full tests / docs / Clippy / release | Passed; 278 tests; retained unchanged |
| xSAR six feature configurations and neutral dependency checks | Passed; counts 14/46/25/58/57/89; four neutral trees |
| Private unchanged-source console compatibility smoke | Passed; 80 tests; check and help; original console frozen |
| App native release / native and protected help / artifact identity | Passed; 14.375 / 0.004 / 0.001 s; exact SHA below |
| Final documentation and preservation audit | Passed; 12 documents, 309 local links, 40 review paths; exercised inputs and all preservation contracts verified |
| Fresh Beast gameplay and human acceptance | Not performed |

All Cargo invocations used nightly, the explicit owning package, `--locked`,
`--offline` and native `--target x86_64-unknown-linux-gnu`. Full tests used
`--all-features -- --include-ignored --test-threads=8`; strict docs included private
items with `RUSTDOCFLAGS='-D warnings -D missing_docs'`; all-target/all-feature
Clippy used `-D warnings`. No dependency, feature or toolchain change was made.
Scoped formatting covered 19 approved Rust files without `--all` and preserved
inherited blank-line spacing. No unchanged successful Cargo workload was repeated.

Toolchain: rustc **1.101.0-nightly (b6eeb93c9 2026-10-18)**, full commit
`b6eeb93c992027a03e1fd120b6652546396b142c`, Cargo **1.101.0-nightly
(3960fa69b 2026-10-12)**, LLVM 23.1.4. The corrected app attempt began
`2026-10-20T15:01:31.355707+00:00` and passed its final required preservation check
at `2026-10-20T15:07:19.012354+00:00`; retained shared work began at
`2026-10-20T14:53:48.961977+00:00`. Current commands/logs/source snapshots and hashes
are sealed in `runs/0002/commands-complete.json` and `verification.json` under the
private workroot.

Shared actual command durations: check 4.827 s, full tests 14.760 s, private docs
0.315 s, Clippy 1.167 s, release 9.978 s; feature rows default/search/execution/
combined/Pyramid-rules/Pyramid-search 0.417/0.667/0.517/0.769/0.817/1.171 s;
each neutral dependency tree 0.032 s. Private console check/tests/help took
4.071/6.283/0.032 s. These are retained original results, not rerun claims.

## Protected delivery and final audit

Protected executable:
`/home/charlie/tmp/pyramid-continuous-fallback-validation-Vs54oXto/release/qmp-qemu-socket-2.7.118`.
It is **13,046,672 bytes**, mode **0555**, SHA-256
**`73ed229ea6a00a36c117a33778c40a888b83b76cd0a2930f7f2a9ca8cbbd8e90`**,
byte-identical to the actual native release
`/home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket`.
Both help checks passed with DISPLAY/WAYLAND_DISPLAY empty. The xSAR native release
`/home/charlie/repo/RUST/xsar/target/x86_64-unknown-linux-gnu/release/libxsar.rlib`
has SHA-256 `34ace04f3e70f7fc54ca9db23fd5681cf8c9b77739ce26354f6c86cce4ccb496`.
Cargo JSON identified each actual fresh owner artifact; no binary path was guessed.

Required preservation passed: all three original HEADs/indices, all original
322 app and nine shared fixture files, unchanged console, unrelated source and
11 protected executables 107–117. Exactly two automatic Trigger Tree telemetry
files were individually reviewed and hash pinned, with no directory-wide ignore.
The final documentation/link audit passed at `2026-10-20T15:17:20.126642+00:00`:
12 documents, 309 local links and all 40 review paths. `final-audit-initial-success.json`
and `final-audit-reviewed-telemetry.log` retain that first success; the final record
is `final-audit.json`. Compiled/exercised inputs, all HEADs/indices, unrelated files,
old fixtures and native/protected artifact identities remain unchanged.

Its first attempt correctly refused `Reviewed telemetry bytes differ:
.trigger-tree/history.jsonl`. The earlier reviewed 1,852 rows remained an exact
SHA-matching prefix; three automatic documentation read/scan/read rows brought
the history to 1,855 rows. The paired session retained the same four-key schema
and matching recent events. `final-telemetry-review.json` preserves the exact
append, before/after hashes and byte counts; `runtime-receipts-before-final-audit.json`
preserves the earlier pins. Only the two individual external telemetry pins were
refreshed for the read-only audit. No Cargo check, test or build was repeated.

Runtime-only observation at `2026-10-20T14:54:50.396629+00:00` found PID 108737 still
running protected 116, SHA-256
`2b3efc9c64ab9edd39868969b63d684d3808a4fe9ed6d1ec94c166f370ab8fb0`.
There were 407 image files / 1,017,964,056 bytes under `/home/charlie/tmp/images`.
This dated observation neither replaces that controller nor supplies live 118
acceptance. Preserve evidence before any operator startup, which clears images.

## Exact review scope

Exactly **40 paths: 31 app, nine xSAR and zero console**. All paths below are
literal absolute paths. Automatic telemetry, if changed externally, must be
individually classified and hash pinned; no directory-wide exception is allowed.

```text
/home/charlie/repo/RUST/qmp-qemu-socket/Cargo.toml
/home/charlie/repo/RUST/qmp-qemu-socket/Cargo.lock
/home/charlie/repo/RUST/qmp-qemu-socket/README.md
/home/charlie/repo/RUST/qmp-qemu-socket/CHANGELOG.md
/home/charlie/repo/RUST/qmp-qemu-socket/src/app/pyramid_workflow.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/app/pyramid_workflow_tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_progress/normal.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_progress/normal/tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_workflow_model.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/worker/computed_route.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/worker/computed_route/partial.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/worker/computed_route/series_guard_tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/worker/computed_route/tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/worker/computed_route/wire_tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/worker/computed_route/wire_tests/stock_empty_tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/worker/pyramid_session.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/worker/pyramid_session/native_tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/worker/pyramid_session/tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-standard-series/README.md
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-standard-series/header-2.rgb
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-standard-series/header-3.rgb
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-standard-series/normal-1.png
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-standard-series/normal-2.png
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-standard-series/normal-3.png
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Codex-Current-Status.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Pyramid-Implementation-Schedule.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Pyramid-Evidence-Index.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/pyramid-recon-profile.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Game-Win-Handoff.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Verification-2.7.118.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Live-Test-2.7.118.md
/home/charlie/repo/RUST/xsar/Cargo.toml
/home/charlie/repo/RUST/xsar/Cargo.lock
/home/charlie/repo/RUST/xsar/README.md
/home/charlie/repo/RUST/xsar/CHANGELOG.md
/home/charlie/repo/RUST/xsar/src/search.rs
/home/charlie/repo/RUST/xsar/src/search/best_effort.rs
/home/charlie/repo/RUST/xsar/src/search/best_effort/tests.rs
/home/charlie/repo/RUST/xsar/src/games/pyramid/search.rs
/home/charlie/repo/RUST/xsar/tests/pyramid_best_effort.rs
```

## Review-only paired commit proposal

Promote xSAR first only when Charlie authorises promotion and agrees the app's
dependency/pin. Neither repository has been staged, committed or pushed here.

xSAR title: `feat(search): add bounded scored fallback routes (0.7.103)`

Preserve existing complete-route APIs and add two-phase scored search with one
aggregate budget, honest exhaustive/bounded proof, checked retained prefixes and
cancellation at every result boundary. Add a pure Pyramid tableau-removal adapter
and independent replay regressions.

App title: `feat(pyramid): enable continuous games and removal fallback (2.7.118)`

Recognise all three measured normal ordinals, enable guarded Continuous and
Solver-free restart, show weighted/exact proof and statistics, and execute a
replayed beneficial partial route with exact endpoint proof before releasing
control. Preserve card recognition, STOP, uncertain-input and restart boundaries.

See the [live brief](Live-Test-2.7.118.md),
[current status](Codex-Current-Status.md) and
[master schedule](Pyramid-Implementation-Schedule.md) for acceptance boundaries.
