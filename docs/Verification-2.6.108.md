# Verification — app 2.6.108 / xSAR 0.6.102

Status: **host verified; required checks passed in attempt 0002**. Updated
19 October 2026. This record distinguishes source implementation, host proof, synthetic
controller tests and Beast acceptance. The larger native Continuous goal is
not complete. M01–M09 accepted; M10 Awaiting verification; M11 Planned.

## Approved scope and contracts

App 2.6.108 and xSAR 0.6.102 earn exactly one agreed minor and one build each
from 2.5.107/0.5.101 after their complete required verification. Console
0.8.4 remains byte-for-byte frozen. App consumes the xSAR library directly,
without a console dependency. Dependency versions, features, MSRV and edition
are unchanged; only owning package versions should change in lockfiles.

- Manual/Fast startup; Manual preserves separate Recon/Plan/Solve. Single board
  explicitly authorises one worker-owned acquire→plan→execute run, then stops.
  Mode/method selection is input-free and revokes old manual preparation.
- Paired D acquisition waits at least 500 ms after each ACK and recognises both
  endpoint cards; successful deterministic acquisition uses 20 captures/24 D.
  Complete unique inventory, positive exhaustion/restoration and private atomic
  no-overwrite publication remain mandatory. STOP may prevent restoration.
- After final delivered route ACK, retire the card sequencer and observe until
  positive clear/new-board/terminal proof or ACK plus two seconds. Started
  captures may finish later. DeliveredUnconfirmed never permits another card
  input, replay, cleanup or automatic continuation.
- Fast uses weight-two A*; Fewest actions uses exact unit-cost A*. Every solved
  route is independently replayed. No universal speedup is promised. Existing
  solve APIs remain exact; options/progress are additive. Latest-only app stats
  match context/run/board and report actual elapsed/expanded/generated values.
- Native-v10 is one exact highlighted-clubs append in both banks; v9 and caps
  24/8, alignment ±1, masks, colour, scenes, modals and authority remain intact.

**Blocking native boundary:** the Continuous Start button is disabled with an
operator-visible explanation. No positive 0/3, 1/3 or 2/3 progress or standard-
three target evidence exists. Full occupancy and terminal/New Game/Play facts
are insufficient. Native Continuous refuses both board and terminal entry
before restart/input. Synthetic known-count/target driver and Solver-free
restart tests do not prove native support. No Daily Challenge automation or
completed larger goal is claimed. STOP/context/errors revoke all later input;
one-shot receipts/plans are consumed, without uncertain input retry or resume.

## Exact calibration evidence

Source `/home/charlie/tmp/.pyramid-recon-721903-0` retains four captures/two D
acknowledgements, stock count two, restoration false, no uncertainty and no
publication. RIGHT 2♣ old rank XOR 9 passes 24; old suit XOR 9 fails 8. Exact new
clubs sample `[1027,708,20,16]` has full 320-pixel mask/tolerance zero, raw RGB SHA-256
`b04d2a7ad13c0f3a04af1d4020df66c5f97dad7a480321ebf7dc10666c4468b4`.
Profile 808,237 bytes, SHA-256
`778b74051a4f12b383c8e79ac951ef332dabf188ce67583e1358cc2b957f5d51`.
[Native-v10 provenance](../tests/fixtures/pyramid-recon/native-v10/README.md)
retains original PNG/JSON hashes and full-profile rollback to immutable v9.
No complete stock/restored receipt follows from those images.

## Required host evidence — passed

Work root: `/home/charlie/tmp/pyramid-improvements-2k8a917g`. `verification.json` is
`passed-required-checks`; completed attempt `runs/0002` started at
2026-10-19T04:24:35.713242+00:00 and finished at 2026-10-19T04:27:53.113423+00:00.
`commands.json` retains exact argv, working directories, flags, timings and log
paths. The baseline already contains earlier dirty M10 work. No HEAD diff is
used as this goal's scope: literal paths and baseline file/hash maps are binding.

Actual toolchain: rustc **1.101.0-nightly (8006091eb 2026-10-17)**, full commit
`8006091eb71ae7d9e9bccb66c6122c9d68af4466`, LLVM 23.1.4; Cargo
**1.101.0-nightly (3960fa69b 2026-10-12)**. Native target/host is
`x86_64-unknown-linux-gnu`. Cargo used explicit `+nightly --locked --offline`
and the native target. Check/Clippy selected all targets/features; full tests
included ignored tests; both private rustdoc runs used
`RUSTDOCFLAGS="-D warnings -D missing_docs"`, all features and no dependencies.
App release uses package defaults; xSAR release selects all features. Artifacts
were selected from Cargo JSON, without guessing a target path.

| Check | xSAR | Application |
| --- | --- | --- |
| Locked all-target/all-feature check | Passed, 0.033 s | Passed, 1.717 s |
| Full tests, including ignored | **266 passed, 0 failed, 0 ignored**, 7.128 s | **1,043 passed, 0 failed, 0 ignored**, 149.370 s; test report 149.18 s |
| Strict private rustdoc | Passed, 0.032 s | Passed, 1.267 s |
| Clippy, warnings denied | Passed, 0.114 s | Passed, 6.326 s |
| Additional neutral/search production safety-lint consumer | Passed, 0.064 s | Existing source-level contracts covered by full checks |
| Release build/Cargo JSON artifact | Passed, 0.114 s | Passed, 14.558 s |
| Native executable headless help | Library | Passed, exit 0,0.004 s |
| New-module scoped formatter | 1 module,0.016 s | 6 modules,0.032 s |
| Isolated existing-parent formatter shadows | 3 modules,0.114 s | 14 modules,1.617 s |

Formatting passed with matching nonempty lines in every inherited module;
standing blank spacing was preserved. App blank-only exceptions are
`src/app.rs`, `src/pyramid_recon.rs`, `src/pyramid_recon/vision.rs`,
`src/worker.rs`, `src/worker/post_game.rs` and
`src/worker/pyramid_reconnaissance.rs`. xSAR has no blank-only exceptions.
No blanket formatting or sibling traversal was performed.

| xSAR configuration | Passed | Failed | Ignored |
| --- | ---: | ---: | ---: |
| Default | 14 | 0 | 0 |
| search | 37 | 0 | 0 |
| execution | 25 | 0 | 0 |
| search + execution | 49 | 0 | 0 |
| pyramid_rules | 57 | 0 | 0 |
| pyramid_search | 80 | 0 | 0 |

These configurations overlap the 266-test full suite; never add them as unique
cases. Default/search/execution/combined dependency trees remain std-only.
Old exact solve callers, additive options/progress consumers and representative
image/PNG/QMP/TCP features passed their all-feature/matrix scopes.

Private frozen-console-copy check passed 4.778 s and tests **80 passed, 0 failed,
0 ignored**,6.784 s. Only the private copy's xSAR owner-version lock entry was
adapted. The original console was neither built nor edited; all its files remain
unchanged. No new original-console release/acceptance is claimed.

Required preservation passed: all three HEADs/indices, closed compiler/fixture
inputs, every original-console file, unrelated source/calibration/evidence and
the separate delivered 2.5.107 stable artifact. Only automatic
`.trigger-tree/history.jsonl` and
`.trigger-tree/sessions/b1065bcab784f174a496f2aa5538322e.json` telemetry changes
were classified and left untouched. **Root's final documentation/link/scope
and preservation audit passed at 2026-10-19T04:32:24.201057+00:00 after ownership
returned: 11 documents, 162 local links and all 44 explicit review paths.**
`final-audit.json` retains the exact compiler/artifact, frozen-console, unrelated
evidence and all-three-HEAD/index checks.

## Focused evidence and retained first failures

- Native-v10 production-parser/recognition filter: **9 passed, zero failed or
  ignored**, 1,033 filtered; test time 8.63 seconds. The preceding compile failed
  E0433 at `src/worker/computed_route/tests.rs:690:26`: the new Fast-consumer
  regression used unqualified `Ordering`. The test now uses
  `std::sync::atomic::Ordering::Acquire`. Original source/log/result remain in
  work-root `first-native10-focused-failure/`; no production predicate changed.
- Independent Pyramid restart filter: **10 passed, zero failed or ignored**,
  1,032 filtered; test time 13.22 seconds. Its first run was **9 passed, one
  failed, zero ignored**, 12.93 seconds. The erasure test painted the exact black
  caption-scan region black and thereby created the required black run instead
  of removing it. White now erases that caption; the production predicate is
  unchanged. Original evidence remains in `first-restart-focused-failure/`.
- Worker fixtures now retain an owned frame and assert concrete latest pixels.
  Four current-default test accessors select v10; historical v9 fixtures remain
  literal v9. These test changes are part of the closed compiler-input snapshot.

The earlier restart filter predates the final decoded-frame-retention regression.
The completed full suite contains 11 restart cases, the 9 native-v10 cases and
the real Fast worker-consumer integration. The new analysis-error-focused test
passed **1/0/0**, 1,042 filtered, 8.06 seconds. Focused filters overlap the final
1,043 application tests; do not sum them as additional unique tests.

Attempt `runs/0001` was deliberately interrupted with SIGINT during app tests
at 2026-10-19T04:23:27.315728+00:00 after review found that fallible analysis
could discard already decoded failure pixels. This was **not a test failure**.
`verification-interruption.json`, attempt logs/source snapshots and the original
first-failure pointer remain retained. The correction clones raw
`capture_screen` pixels before fallible `publish_and_analyse`; the owned latest
frame therefore survives an analysis error without another capture. The normal
Computer Vision path using no retained-frame sink is unchanged. The complete
required scope, including private console smoke, then passed in attempt 0002.
No previous failed/interrupted evidence was overwritten.

## Artifact and runtime — verified host candidate

Cargo-selected native app executable:
`/home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket`.
SHA-256: `badd90d6f75d78600066b749d1582a4ec681ab3f89bae8e774d0f23a7f35e573`.
The separate stable copy is `/home/charlie/tmp/pyramid-improvements-2k8a917g/release/qmp-qemu-socket-2.6.108`,
**12,680,152 bytes, mode 0555**, identical SHA-256 and headless help exit 0,
recorded in `stable-artifact.json` at 2026-10-19T04:28:28.011462+00:00.
The xSAR all-feature release rlib SHA-256 is
`4cbe918e0e080f355504bc3b40ca837fc480da80ae9caf47000465f010f83a42`;
its Cargo JSON/rmeta hashes are retained in the verifier.

The delivered stable 2.5.107 remains
`/home/charlie/tmp/pyramid-v9-validation-8s961bw8/release/qmp-qemu-socket-2.5.107`,
SHA-256 `b2460b0dc15327346349425cb45a9e9f285fc840a75593d42c6949f12b1cf6cb`.
Read-only `runtime-preflight.json` observed **no application-controller process**
at 2026-10-19T04:28:28.047232+00:00; `/home/charlie/tmp/images` contained
zero files/bytes then. This is an instant-only observation, not a later running
identity or promise that images remain empty. No QMP connection, GUI launch,
installation, guest input, process replacement, commit, push or publication was
performed. The [live brief](Live-Test-2.6.108.md) provides the exact stable launch
with startup-image-clearing warning and fresh supported Manual/Single board
acceptance steps. Beast acceptance and native Continuous remain pending.

## Explicit review paths

Current named correction scope contains 36 application paths and
8 xSAR paths (44 total), no console paths. The verifier closed this list against
the baseline; the final documentation audit follows ownership return. It is not the complete promotion
set for accumulated dirty M10 work; earlier accepted changes remain separate.

```text
/home/charlie/repo/RUST/qmp-qemu-socket/Cargo.toml
/home/charlie/repo/RUST/qmp-qemu-socket/Cargo.lock
/home/charlie/repo/RUST/qmp-qemu-socket/src/main.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/app.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/app/pyramid_workflow.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/app/pyramid_workflow_tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_recon.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_recon/paired_tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_recon/vision.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_recon/vision/daily_challenge_tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_recon/vision/native_v9_tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_recon/vision/native_v10_tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_workflow_model.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/worker.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/worker/computed_route.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/worker/computed_route/tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/worker/computed_route/wire_tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/worker/post_game.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/worker/post_game/pyramid_independent_tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/worker/pyramid_reconnaissance.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/worker/pyramid_session.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/worker/pyramid_session/tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v10/profile.json
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v10/baseline.png
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v10/latest.png
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v10/INCOMPLETE.json
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v10/provenance.json
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v10/README.md
/home/charlie/repo/RUST/qmp-qemu-socket/README.md
/home/charlie/repo/RUST/qmp-qemu-socket/CHANGELOG.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Codex-Current-Status.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Pyramid-Implementation-Schedule.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Pyramid-Evidence-Index.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/pyramid-recon-profile.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Verification-2.6.108.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Live-Test-2.6.108.md
/home/charlie/repo/RUST/xsar/Cargo.toml
/home/charlie/repo/RUST/xsar/Cargo.lock
/home/charlie/repo/RUST/xsar/src/search.rs
/home/charlie/repo/RUST/xsar/src/games/pyramid/search.rs
/home/charlie/repo/RUST/xsar/src/games/pyramid/planning.rs
/home/charlie/repo/RUST/xsar/tests/search_options.rs
/home/charlie/repo/RUST/xsar/README.md
/home/charlie/repo/RUST/xsar/CHANGELOG.md
```

## Proposed commits — review only

App title: `feat(pyramid): add guarded single-board workflows and route methods`

App body: Add Manual/Single board orchestration, paired 500 ms reconnaissance,
final acknowledgement-relative observation, identity-bound planning statistics
and exact native-v10 clubs recognition. Keep native Continuous unavailable until
positive progress/standard-target evidence exists. Preserve STOP, receipts,
independent replay, input boundaries and the delivered 2.5.107 artifact. Record
actual host checks separately from Beast acceptance.

xSAR title: `feat(search): add weighted options and bounded progress reporting`

xSAR body: Add explicit positive heuristic-weight options and synchronous initial,
throttled periodic and final statistics to generic and Pyramid search. Preserve
old exact solve entry points, unit action costs, cancellation, fallible growth,
feature/dependency contracts and independent replay responsibility.

No staging, commit, push or registry publication is delegated.

## Code Analysis

Guest authority stays in the serial application worker. Unknown native progress
and target facts cannot authorise continuation or terminal restart. Acknowledged
input is not effect proof; DeliveredUnconfirmed, STOP and context drift stop
further input, including restoration. A native terminal alone proves no target.

Search remains bounded by 2M histories/expansions and 120 seconds in the app,
with checked growth and no imposed byte cutoff. Shared defaults retain 512 MiB.
Weighted ordering can change route length; exact callers preserve their contract.
Callbacks are synchronous and cooperative, so a slow callback/I/O can delay STOP.
Latest-only statistics carry no input authority. Evidence naming/publication is
checked and no-overwrite; filesystem errors may leave partial private diagnostic
packets with explicit persistence warnings. No extra capture is requested to
save evidence. Original failure files remain immutable.
