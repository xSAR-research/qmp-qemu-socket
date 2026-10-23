# Verification — qmp-qemu-socket 2.5.106

Date: 19 October 2026, Australia/Brisbane.

**Host verified; Beast live acceptance pending.** The complete required application
sequence passed **968 tests, zero failed/ignored**. App 2.5.105 → 2.5.106 earns
one correction build, with no additional minor milestone. xSAR 0.5.101 and
accepted console 0.8.4 remain fully frozen. M01–M09 are accepted;
**M10 Awaiting verification and M11 Planned**. The final documentation/path audit
passed after ownership handover: seven documents, 109 relative links and all
13 correction paths; compiler inputs, artifact and preservation checks passed.

## Correction and contract

`src/pyramid_search_policy.rs` supplies a single application-owned policy to the
worker and UI. `max_memory_bytes = usize::MAX` removes the imposed byte cutoff
without changing shared defaults or reserving memory. Diagnostics encode this as
`max_memory_bytes: null` and `memory_policy: "allocator_checked"`; they do not
present a machine-word maximum as available RAM. Existing 2,000,000 retained
histories, 2,000,000 expansions, 120 seconds and cooperative cancellation remain.
Fallible collection reservations still report `AllocationFailure`. This removes
a conservative accounting cutoff, not physical memory pressure.

The UI's three explicit steps, automatic native-v8/output/Deal ID, private
failure packets and worker input/evidence/STOP authority are unchanged. A failed
Plan still consumes/drops its acquisition receipt. No Daily Challenge completion
or progress predicate, route retry/resume, algorithm, calibration or input budget
change is included. The console retains its existing 512 MiB default allowance;
all older finite-limit descriptions remain accurate in their dated scope.

[Upstream at pinned revision c6dde73](https://github.com/mchung94/pyramid-solver/blob/c6dde73fd27ec817e2f1329f2a761b384cd8c663/README.md#the-pyramid-stock-pile-and-waste-pile) counts 1,430 legal occupancy
patterns for the 28-card tableau. Complete search also tracks stock/waste
occupancy and position plus recycle state. That count is not a cap on full
states, search histories or possible routes.

## Baseline and retained evidence

Work root: `/home/charlie/tmp/pyramid-memory-policy-91h5iyi5`.
`baseline.json` records all three index digests and file maps before this
correction. The application HEAD remains `842646e24b179fb4c4ee2cd04f1dc28b8cfed30b`
and already carries earlier uncommitted M10 work. xSAR HEAD is
`00986b8d621803fc98da85616444fc76c14638a9`; console HEAD is
`22f52d7ec5fc867b542dd7ca4c20fe2b9fbe2e84`. These identify local source boundaries,
not remote synchronisation or clean promotion candidates.

`verification.json`, `commands.json`, `foreground.log`, `toolchain.log`, `cargo.log`
and individual check logs retain actual evidence. `focused-policy.log` reports
three passed/zero failed/ignored, 965 filtered, 0.74 seconds after 10.26-second
test compilation. These overlap the full suite. The real long-path regression
retains 524,290 histories and reconstructs 524,289 actions, exceeding the former
524,288 ceiling; the other cases preserve an independent StateLimit and honest
finite/unset diagnostic semantics.

## Exact reported-deal comparison

Source: `/home/charlie/tmp/pyramid-error-1792370373968-701820-0/deal.json`.
SHA-256: `d30235d4d1ae9ead8dfbbe32da6d3762f5c69aed8d6d728a3a8f330ea9bd4c6f`.

The isolated private Cargo probe includes the exact application policy source
and invokes the frozen real xSAR Pyramid A* plus independent replay with
`require_clear=true`. `offline-deal-comparison.json` records:

| Measurement | Previous 512 MiB allowance | New application policy |
| --- | --- | --- |
| Outcome | Stopped: MemoryLimit | Solved, 42 actions |
| Retained histories | 524,288 | 1,979,388 |
| Expanded | 355,904 | 1,640,677 |
| Generated | 839,928 | 3,768,945 |
| Peak frontier | 167,354 | 337,289 |
| Solver elapsed seconds | 0.567107662 | 2.779285536 |
| Independent clear replay | No route available | Passed |
| Guest input | None | None |

The unchanged two-million-history/expansion bounds suffice for this deal.
These are offline solver timings, not GUI/live timings or measured RSS.
Initial direct-rustc rlib/rmeta linking attempts failed before execution; preserve
`probe-build-first-rlib-failure.log` and `probe-build-first-rmeta-failure.log`.
The successful separate Cargo manifest/lock/target are private under the work
root; no application or sibling dependency override was added.

## Actual host validation

Selected toolchain: rustc 1.101.0-nightly (`8006091eb`, 2026-10-17), Cargo
1.101.0-nightly (`3960fa69b`, 2026-10-12), native `x86_64-unknown-linux-gnu`.
All Cargo checks use explicit `+nightly`, `--locked --offline` and native target.
Strict private rustdoc uses `RUSTDOCFLAGS="-D warnings -D missing_docs"`.

| Check | Actual result | Command seconds |
| --- | --- | --- |
| Scoped rustfmt: policy, workflow, computed-route modules | Passed | 0.032 |
| Parent changed-range shadow: main.rs | Passed; no inherited blank-line exception | 0.032 |
| Locked all-target/all-feature Cargo check | Passed | 5.677 |
| All-feature tests, including ignored | 968 passed/0 failed/0 ignored; test report 140.43 seconds | 140.629 |
| Strict private rustdoc | Passed | 1.116 |
| All-target/all-feature Clippy, warnings denied | Passed | 6.026 |
| Native release, default features, Cargo JSON artifact selection | Passed | 13.377 |
| Exact native artifact headless `--help` | Passed | 0.001 |

Relevant Cargo argument vectors are recorded exactly in `commands.json`:

```text
cargo +nightly check --locked --offline --target x86_64-unknown-linux-gnu --all-targets --all-features
cargo +nightly test --locked --offline --target x86_64-unknown-linux-gnu --all-features -- --include-ignored
cargo +nightly doc --locked --offline --target x86_64-unknown-linux-gnu --all-features --no-deps --document-private-items
cargo +nightly clippy --locked --offline --target x86_64-unknown-linux-gnu --all-targets --all-features -- -D warnings
cargo +nightly build --locked --offline --target x86_64-unknown-linux-gnu --release --message-format=json
```

The full required sequence passed without a new check/test failure. No blanket
package/sibling formatter was run for this slice. New compiler inputs and existing
unrelated evidence remained frozen during verification. The earlier xSAR 256-test
and matrix results are retained history, not newly run shared suites here.

## Artifact and runtime observation

Compiler-selected native artifact:

```text
/home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket
SHA-256: 8340bd4ee6e0480e762f2cd6b870ac2be56da62d4bd7184ef8977116abb35ba7
```

`runtime-preflight.json` observed 2026-10-19T01:38:52.828320+00:00:
PID 701820 still used `/home/charlie/repo/RUST/qmp-qemu-socket/target/release/qmp-qemu-socket`
with `--image`, SHA-256
`0b9fd4044f6054a769f1647190b80a35953ca18f9bf517a4fcaa2d85fb7fb035`.
`/home/charlie/tmp/images` had zero files/zero bytes at that instant. This is an
observation, not newly installed/running 2.5.106 identity. No process was stopped
or replaced; no GUI, capture, QMP, installation or promotion occurred.
[The live brief](Live-Test-2.5.106.md) provides the later exact-artifact launch and
requires rechecking/preserving evidence because startup clears the image directory.

## Preservation and explicit review paths

The verifier confirmed all three Git indices unchanged, every xSAR/console file
frozen, and unrelated application source/evidence preserved. Only automatic
`.trigger-tree/history.jsonl` and
`.trigger-tree/sessions/b1065bcab784f174a496f2aa5538322e.json` telemetry changed;
they were classified and left untouched. The final audit passed after the
seven-document handover, confirming 109 relative links, all 13 review paths,
exact verified compiler inputs/artifact and the same preservation boundaries.

This **13-path correction delta** is not the complete promotion set for the
accumulated uncommitted M10 work. No staging, commit or push is authorised.

```text
/home/charlie/repo/RUST/qmp-qemu-socket/Cargo.toml
/home/charlie/repo/RUST/qmp-qemu-socket/Cargo.lock
/home/charlie/repo/RUST/qmp-qemu-socket/src/main.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_search_policy.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/worker/computed_route.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/app/pyramid_workflow.rs
/home/charlie/repo/RUST/qmp-qemu-socket/README.md
/home/charlie/repo/RUST/qmp-qemu-socket/CHANGELOG.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Codex-Current-Status.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Pyramid-Implementation-Schedule.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Pyramid-Evidence-Index.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Verification-2.5.106.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Live-Test-2.5.106.md

```

Proposed Conventional Commit title, for review only:

```text
fix(pyramid): remove the application search memory cutoff
```

Proposed body:

```text
Use one application search policy with no imposed byte allowance and honest
allocator-checked diagnostics. Preserve work/deadline/cancellation limits,
checked allocations, receipt consumption and all guest input guards.

Exercise real growth beyond the old memory-derived ceiling and compare the
reported deal: old policy stops at 524288 histories; new policy finds 42 actions
at 1979388 histories and independent clear replay passes. Required host checks
pass 968 tests with no failures or ignored tests. Beast acceptance is pending.
```

## Code Analysis

The policy owns no guest I/O or card state and introduces no unsafe Rust.
Removing the conservative byte allowance permits more heap growth; record,
expansion, deadline and cancellation bounds remain independent. Checked
reservations report allocation refusal but do not guarantee operating-system
memory availability or successful process survival under external pressure.
No RSS guarantee is made. Diagnostic null accurately distinguishes no application
budget from a finite supplied allowance. Real growth/reconstruction and work-limit
regressions cover the policy boundary without cloning the search implementation.

Exact-deal replay verifies logical clearance only. Receipt consumption, strict
vision/input/context/STOP checks and no-resume semantics remain authoritative;
the existing Daily Challenge completion mismatch can still stop execution. Retain
first-failure pixels/packets and never replay uncertain input. M10 acceptance
requires a fresh successful restored acquisition and actual observed live effects.
