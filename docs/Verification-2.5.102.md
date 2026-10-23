# qmp-qemu-socket 2.5.102 verification — host verified

Updated: 19 October 2026. **M10 is Awaiting verification; M11 remains Planned.**
M01–M09 retain Charlie's acceptance. The approved host implementation of
Rust-calculated Pyramid routes with Solver off is complete. Application 2.5.102
and xSAR 0.5.101 passed all required host checks, each applying one agreed minor
increase and exactly one earned build increment. Beast live verification is
pending. No installation, automated live success or promotion is claimed.

## Source boundary and implementation

The inspected application baseline was host-verified **2.4.101**, with HEAD
`842646e24b179fb4c4ee2cd04f1dc28b8cfed30b` and accepted dirty native-v7/docs work.
Its index SHA-256 was
`3e64bf120a38e502340e9d5aab8450a20b83240a73df98a0aa2eff8d2716c168`.
The complete per-file hash/status/index snapshot is
`/home/charlie/tmp/pyramid-m10-k4g2jqq2/baseline.json`. HEAD alone does not identify
that accepted source content.

The supporting xSAR baseline was clean 0.4.100 at
`00986b8d621803fc98da85616444fc76c14638a9`. The application directly consumes
`../xsar` with the new `pyramid_search`/`execution` features and its existing
transport/image/JSON features. Generic bounded unit-cost search and finite
observed sequencing are reusable; Pyramid rules/replay remain in the pure
game adapter. The application owns calibration, observations, input authority
and private receipts. No application-to-console dependency is added.

**basic-route-planning 0.8.4 stays frozen byte-for-byte**, preserving its accepted
dirty baseline at HEAD `22f52d7ec5fc867b542dd7ca4c20fe2b9fbe2e84`. Its baseline
index SHA-256 is
`347f0f62f307234d1e9f813ea63fc84b35bba25b8330ba78deb08bd33dac2215`.
No M05–M09 version credit is repeated. The verified application M10 version is
2.4.101 → 2.5.102; the separately agreed xSAR support milestone is
0.4.100 → 0.5.101. Both changes apply one minor increase and one build increment
after the required full checks passed; the build counters never reset.

Ordinary Shortest Path capture remains read-only. Successful complete
reconnaissance/restoration/publication and diagnostics mint a private same-session
receipt retaining the complete deal, actual calibration, authority epoch and
published bytes. **Calculate route** loads that exact bounded document, performs
input-free A* and independent replay, then retains one plan. A moved or
extensionless file is allowed; altered bytes or arbitrary restoration metadata
cannot authenticate the current guest. Calculation makes no QMP connection.

The file reader checks regular-file metadata before opening, refusing known
FIFOs before an open can wait for a writer, then rechecks the opened handle
before reading at most 65,537 bytes to enforce the 65,536-byte document ceiling.
The known-FIFO regression passed in the full 907-test run; it is not counted
in the earlier focused results below.

**Execute route** separately requires current no-interference/Solver-off
confirmation and consumes the plan before connecting. It checks the original
position twice, verifies the expected position and current VM/absolute-pointer
authority before each gesture, and observes the whole logical action's predicted
effect. MATCH has ordered selections with STOP/context checks between them.
Movement/down/up acknowledgement is delivery evidence, not action completion.
The run ends after one positively verified tableau clear, without terminal or
next-board input. Failed or uncertain delivery/evidence, STOP or context changes
leave no retry/resume plan or cleanup click. Reacquisition is required.

The UI keeps a dedicated monotonic run token through generic worker states and
STOP until the matching finish event. Stale results cannot release another run
or install an old plan. Context/path/reconnaissance/STOP changes revoke the plan
and confirmation; execution dispatch consumes both. Authority epoch exhaustion
is sticky, so it cannot wrap back into a usable receipt.

## Fixed execution bounds and accounting correction

For the actual route's A logical actions and G native gestures, the adapter
checks A ≤ 102 and G ≤ 126. The accepted v7 regression has A=49 and G=73.
Defaults remain immutable in the UI: 1,000 ms settle after MOVE/RECYCLE,
2,000 ms after removals, 1,000 ms reobservation spacing, six observations per
ordinary precondition/effect phase, twenty final effect observations, and one
cooperative 1,200-second execution deadline including initial observations,
transport and waits.

The total capture allowance is now:

```text
max(2 + 6*(G + A) + 20, 12*A + G + 16)
```

The second term accounts for two initial observations, G fresh gesture guards,
6*A preconditions, 6*(A−1) ordinary effects and twenty final effects. It corrects
the original total's shortfall for some action/gesture combinations while
preserving the approved allowance when larger. Checked arithmetic rejects invalid
counts/overflow. For the accepted A=49/G=73 route the maximum stays 754 captures;
the phase-derived term is 677. This changes total accounting only, retaining the
same finite gesture limits, phase limits and deadline.

## Retained focused results

Host evidence is in `/home/charlie/tmp/pyramid-m10-k4g2jqq2/`:

| Completed scope | Actual result | Retained log |
| --- | --- | --- |
| xSAR generic search/execution/Pyramid planning integration | 37 passed, 0 failed/ignored: 14 search, 11 execution, 12 planning | `xsar-focused.log` |
| Application computed-route controller and synthetic QMP/TCP wire tests | 15 passed, 0 failed/ignored, including three wire tests | `app-focused-second.log` |
| UI ownership subset of the earlier mixed controller/UI run | Four UI tests passed within the preceding 12-test run; not rerun by the later controller filter | `app-focused-first.log` |
| Native route-position observer regressions | 9 passed, 0 failed/ignored | `vision-focused.log` |
| Application all-feature binary check | Passed | `app-second-check.log` |

The application controller fixture completed the accepted 49 logical actions
with exactly 73 acknowledged native gestures and independently supplied effects.
It also refused wrong origin, STOP between pair selections, acknowledged input
without effect, missing acknowledgement, fresh position/capability failure and
uncertain delivery without replay or logical commit. UI tests cover monotonic
tokens, dedicated ownership through all worker states, context-bound single-use
consent and no plan after failed/stopped completion.

The later 15-test controller run additionally covers finite capture accounting,
receipt absence, changed-and-back context/idle STOP, exact moved-document
acceptance and single-use execution. Its three synthetic wire cases verify
fresh-capture/probe/input ordering, STOP before probe acknowledgement with no
pointer/key/button input, and a missing down acknowledgement with one press and
release-only recovery. The separate earlier `wire-first.log` passed the same
three wire tests; it is preceding evidence and is not an additional test count.

The native observer tests retain the exact v7 origin, positive lower-felt removal
evidence and fixed original slots. Known wrong remaining identities, unknown
removed cards, incorrect DL/DR or empty-pile evidence, changed scenes/Solver
signatures, invalid layout and v6 unknown glyphs are refused. These controlled
fixtures do not establish live rendering behaviour.

The [xSAR verification record](../../xsar/docs/Verification-0.5.101.md) retains the
licensed 15/44 cases and separate accepted 48/49-action replay/optimal-cost
regressions. First failed checks remain in the work directory; later successes
do not erase their evidence.

## Completed full host verification

The final record is `/home/charlie/tmp/pyramid-m10-k4g2jqq2/verification.json`;
`commands.json` records exact argv, working directories, results and log names.
Actual toolchain: `rustc 1.101.0-nightly (8006091eb 2026-10-17)`, compiler revision
`8006091eb71ae7d9e9bccb66c6122c9d68af4466`, LLVM 23.1.4, and
`cargo 1.101.0-nightly (3960fa69b 2026-10-12)`.
Host/target: `x86_64-unknown-linux-gnu`; edition 2024/MSRV 1.101.0 unchanged.

Cargo commands below used `cargo +nightly` with `--locked --offline --target
x86_64-unknown-linux-gnu`; strict documentation additionally used
`RUSTDOCFLAGS='-D warnings -D missing_docs'`.

| Required verification | Actual result and retained log |
| --- | --- |
| New-module scoped formatting | Passed `rustfmt +nightly --edition 2024`, owning settings and `skip_children=true`; `scoped-format.log`. |
| Edited parent ranges | Only two inherited required first-`if` blank-line differences remain, identical to HEAD; `format-parent-check.json`. Full package formatting retains pre-existing baseline differences. |
| `check --all-targets --all-features` | Passed; `app-check.log`. |
| `test --all-features -- --include-ignored` | **907 passed, 0 failed, 0 ignored**; `app-tests.log`, including UI, controller/wire, vision and known-FIFO regression. |
| Strict private `doc --all-features --no-deps --document-private-items` | Passed; `app-rustdoc.log`. |
| `clippy --all-targets --all-features -- -D warnings` | Passed; `app-clippy.log`. |
| Native `build --release --message-format=json` | Passed, executable selected from Cargo artifact output; `app-release.log`. |
| Selected native executable `--help` | Passed headlessly; `app-help.log`. |
| xSAR full suite/features/check/docs/Clippy/safety/release | Passed: **256 tests, 0 failed/ignored** plus overlapping independent feature matrix; [crate record](../../xsar/docs/Verification-0.5.101.md). |
| Compiler/source/evidence/console/index preservation | Passed `finalise.py` hash comparison; telemetry-only exception below. |
| Installation/live M10 acceptance | Not performed. Host implementation complete; Beast live verification pending, M11 Planned. |

The xSAR feature configurations passed default **14**, search **28**, execution
**25**, search+execution **40**, pyramid_rules **57**, pyramid_search **71** tests,
all zero failed/ignored. These are overlapping configurations, not extra unique
tests to sum. Pure Pyramid configurations omit JSON; neutral feature dependency
trees contain only xSAR. The combined public non-card consumer verifies two
settings actions/three gestures without card/image/transport dependencies.

The native executable selected by Cargo is:

```text
/home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket
SHA-256 5bd527f9398df87ebd1f9b10a03d71f5388b6f94135e72960aa3a300c136dda3
```

This is a built candidate artifact, not an installed executable identity. The
read-only `runtime-preflight.json` checkpoint found no running
qmp-qemu-socket process and `/home/charlie/tmp/images` present with zero files,
zero bytes and zero symlinks. No GUI was launched or process stopped. That
observation does not establish a later process or an installed binary. The
[operator test brief](Live-Test-2.5.102.md) describes the separately controlled
fresh-session acquisition, calculation, confirmation and execution trial.

## First failures, formatting and preservation

First strict docs found 23 pre-existing missing variant/field contracts in xSAR
`src/lib.rs`; documentation was added without telemetry behaviour changes.
An optional crate-wide safety probe found four existing geometry/complete-deal
invariant accesses. They were retained; indexing/unwrap/expect/panic are denied
mechanically in the new production modules. The final module-scoped safety and
required Clippy checks passed. The application's three new style and seven field
documentation errors were fixed before final full checks. The first failures
remain in the private work directory rather than being replaced by later success.

New modules passed the scoped formatter. The edited-parent range check reports
only two inherited first-`if` blank-line differences in
`ensure_strategy_input_authorised` and `run_preparation_capture`. Their spacing
matches HEAD/current and is preserved under the standing instructions. Package
formatting retains pre-existing differences; no full package-format pass or
blanket/sibling reformat is claimed.

All required commands passed before `verify.py`'s post-check helper asserted on
automatic documentation-read telemetry. `finalise.py` verified exact compiler
inputs unchanged since the checks, all three Git indices unchanged, every
frozen console file unchanged and unrelated source/evidence unchanged. Only app
`.trigger-tree/history.jsonl` and
`.trigger-tree/sessions/b1065bcab784f174a496f2aa5538322e.json` changed automatically;
the before/after hashes and classification are recorded in `verification.json`,
and both files were left intact. There was no source/evidence drift, so the
helper-only correction did not require repeating passed checks.

The historical [2.4.101 result](Verification-2.4.101.md) passed 876 tests for its
different calibration revision. The accepted console's 80-test verification and
Charlie's fresh-v7 manual 49-action success are retained historical evidence,
not current full M10 checks or automated guest acceptance.

## Possible live blocker and evidence boundary

**Intermediate selected-card rendering is unverified.** Selected-card tint is a
possible live blocker: after the first selection of a pair, a fresh
second-gesture guard may refuse the tinted card. No live execution trial has
established whether that unsupported intermediate rendering occurs or passes.
Recognition signatures, masks, caps and positive identity/absence requirements
have not been weakened to permit that state. Host synthetic sequencing success
does not establish that live intermediate rendering is supported. Retain the
first mismatching frame in memory and its diagnostic log; no ordinary gameplay
PNG capture or input replay is introduced.

M11 must separately establish the installed executable and current process,
fresh supported acquired/restored deal, actual input/effect results and Charlie's
acceptance. Report one tableau independently of a three-board GAME WIN. Any
required recognition change needs its own concrete evidence and approved scope.

## Explicit review paths and accepted baseline work

These paths already differed before M10 and remain accepted prior work:

```text
Cargo.toml
Cargo.lock
README.md
CHANGELOG.md
docs/pyramid-recon-profile.md
docs/Codex-Current-Status.md
docs/Game-Win-Handoff.md
docs/Pyramid-Evidence-Index.md
docs/Pyramid-Implementation-Schedule.md
docs/Verification-2.4.101.md
src/pyramid_recon/vision.rs
src/pyramid_recon/vision/native_v7_tests.rs
tests/fixtures/pyramid-recon/native-v7/
```

The M10 review must compare the saved baseline bytes, including overlapping
manifest/README/tracker/vision paths, rather than treating their complete Git
diff as new M10 work. The explicit M10 application review paths are:

```text
Cargo.toml
Cargo.lock
README.md
CHANGELOG.md
docs/Verification-2.5.102.md
docs/Live-Test-2.5.102.md
docs/Codex-Current-Status.md
docs/Pyramid-Evidence-Index.md
docs/Pyramid-Implementation-Schedule.md
src/app.rs
src/pyramid.rs
src/strategy.rs
src/worker.rs
src/worker/pyramid_reconnaissance.rs
src/worker/computed_route.rs
src/worker/computed_route/tests.rs
src/worker/computed_route/wire_tests.rs
src/pyramid_recon/vision.rs
src/pyramid_recon/vision/route_tests.rs
```

The complete supporting crate path list, including all new generic/Pyramid
modules, public consumer tests, copied MIT fixtures and licence files, is in
[Verification-0.5.101.md](../../xsar/docs/Verification-0.5.101.md).
These are review lists, not authority to stage, commit, push, install or publish.
Local AGENTS.md, temporary logs/work directories, original console Evidence and
unrelated accepted work are outside promotion. If promotion is separately
authorised, promote the crate first, agree the consumer dependency/pin and
revalidate; the current path dependency is not a release revision pin.

## Proposed promotion description

Proposed Conventional Commit title, for review only:

```text
feat(pyramid): execute guarded Rust-calculated routes
```

Proposed body:

```text
Calculate a route directly through xSAR from a same-session restored deal and
exact JSON bytes, then consume it through separately confirmed finite execution.
Guard each gesture and commit each action only after observing its full effect;
stop after one tableau and refuse interrupted or uncertain runs without resume.

Apply the M10 minor milestone and one verified build increment: 2.5.102.
Validation: 907 all-feature tests including ignored tests, locked check, strict
private docs, Clippy, native release/help and full xSAR feature/consumer checks.
Beast live verification remains pending; console 0.8.4 stays byte-unchanged.
```

This proposed M10 description does not automatically include accepted pre-M10
native-v7 work; review the saved baseline and explicit overlapping paths above.
No staging, commit, push, installation or publication was performed.

## Code Analysis

Exact bounded file bytes plus a private same-session origin receipt prevent
arbitrary valid JSON from granting live input authority. Neither the restoration
field nor retained historical diagnostics authenticate a current board. Expected
positions retain original fixed slots and all identities in one complete deal.
Unknown/ambiguous observations refuse; no identity is inferred by elimination.

Regular-file preflight rejects a known FIFO before blocking open and opened
handle metadata is rechecked. This is a trusted-local-path guard, not protection
against concurrent hostile path replacement: replacement between metadata and
open can still block, and external/network filesystem metadata/open/read calls
can stall. The read-size ceiling bounds bytes rather than I/O time. Filesystem
operations and STOP remain cooperative; no hard external-filesystem deadline
or immediate interruption is promised.

The pull-driven generic sequencer consumes each operation once; the application
reserves and guards concrete input, tracks attempted/acknowledged counts and
commits only complete observed effects. Pair interruption is terminal even if
the first click was acknowledged. STOP/context checks revoke subsequent input,
including cleanup; existing release-only QMP recovery is retained. Cooperative
deadlines cannot promise instantaneous cancellation of an in-flight operation.

Checked action/gesture/capture arithmetic, finite observations and deadlines
bound the execution. Search storage is estimated rather than an RSS limit, and
independent replay validates a maximum 4,096-action trace before the stricter
live 102-action/126-gesture bounds. No unsafe memory operation or unbounded
automatic execution is introduced. Intermediate selected-card rendering remains
unverified and may be refused; any actual refusal needs retained evidence and
human review rather than assuming a proved live rendering failure.
