# Verification — application 2.6.116, host verified

Status: **REQUIRED HOST CHECKS PASSED; protected 116 artifact verified.**
`runs/0002` passed all **1,203 tests, zero failed/ignored and zero filtered out**,
plus required check, strict private docs, Clippy, scoped format, release, native/
protected help and exercised-input preservation. Root's final documentation/link/
preservation audit passed (seven documents, 243 local links, 12 approved review paths). Fresh live acceptance,
installation and controller replacement have not been performed.

## Problem and resulting behaviour

After Single board published its reconnaissance output, changing to Manual correctly
revoked the old acquisition receipt and disabled Plan route/Solve board. Manual
Recon then reused the old generated output path, so the worker correctly refused
to overwrite that existing export. The new request must obtain a fresh automatic
name before it reaches the unchanged publication boundary.

Both Manual Recon dispatch and automatic Start now use
`prepare_recon_request(form: &mut ReconnaissanceForm, context: &ControllerContext) -> Result<ReconRequest, String>`.
It prepares the next acquisition, refreshes fields still using automatic names,
arms consent for the current context, then consumes that consent once in the
request. Fresh request details are recorded after preparation. Naming or validation
failure returns no request; literal/mixed overrides retain their exact values.
An existing explicit destination continues to refuse rather than being overwritten.
Mode/method changes continue to invalidate receipts and plans: a fresh successful
Recon and a new Plan route are required before Solve board.

This routine app-only correction advances the tested revision 2.6.115 → 2.6.116. Native-v16, all existing
fixtures and card/controller core remain unchanged. xSAR 0.6.102, console 0.8.4,
all three HEAD/index identities and protected executables 107–115 remain frozen.
Continuous remains disabled; normal later-ordinal evidence, challenge objectives,
alternate-theme support, idle CPU and the wider Pyramid goal remain incomplete.
No additional milestone or goal completion is claimed.

## Focused evidence and retained first failure

The initial focused compile failed with **E0616**, private `request_details` access
at `src/app.rs:1719:31`. The original error remains in
`/home/charlie/tmp/pyramid-mode-switch-validation-mak21b_6/focused-first-failure.log`.
Root added `PyramidWorkflow::record_request_details` as a scoped advisory setter,
keeping the field private, and retained the same shared request-preparation path.
The rerun reports **25 passed, zero failed/ignored**, 1,178 filtered out, in
`/home/charlie/tmp/pyramid-mode-switch-validation-mak21b_6/focused-rerun.log`.
These are focused results, not a complete current-source test pass.

The four new headless boundary regressions cover automatic success/error/STOP
completion followed by Manual; method change and retry; exact literal/mixed fields
with an existing file preserved; and naming failure without request, consent or
output. No guest operation or new recognition calibration is involved.

## First full attempt and unchanged-fixture retry

The original full attempt, `runs/0001`, reported **1,202 passed, one failed,
zero ignored**: 247.677 seconds for the command, 247.45 seconds in the test report.
The failed existing frozen test was
`worker::pyramid_reconnaissance::starting_guard_tests::guarded_adapter_checks_restored_header_before_publication`.
Its local server panicked with `fixture QMP accept timed out` at
`src/worker/pyramid_reconnaissance/starting_guard_tests.rs:233:17`, followed by
`local guard QMP server: Any { .. }` at the same file's `431:30`.
The original failed full-run logs and result remain retained under
`/home/charlie/tmp/pyramid-mode-switch-validation-mak21b_6/runs/0001`.

The exact unchanged focused guard test then passed **one passed, zero failed,
zero ignored**, with an 11.13-second test report, retained in
`/home/charlie/tmp/pyramid-mode-switch-validation-mak21b_6/guard-focused.log`.
Its source starts the five-second local accept server before the client computes
the original position through production recognition. That ordering and the
focused pass suggest a load-sensitive fixture setup; a per-stage timing trace
was not measured, so CPU contention is an inference rather than a demonstrated
complete failure timeline.

Root retained the original private verifier as
`/home/charlie/tmp/pyramid-mode-switch-validation-mak21b_6/verify-before-test-thread-bound.py`
and changed only the subsequent private test command to add `--test-threads=8`
under the observed 16-CPU affinity. All 1,203 tests and `--include-ignored` remain
in scope. No source, assertions, fixture bytes or production timeout was changed,
and no test was excluded. The full rerun is `runs/0002`, with wrapper output in
`/home/charlie/tmp/pyramid-mode-switch-validation-mak21b_6/full-rerun.log`.
The complete rerun passed all **1,203 tests, zero failed/ignored/filtered**, with
311.970 seconds command time and a 311.77-second test report. The independently
retained focused pass is not added to that total. The source revision and fixture
bytes are unchanged between these attempts; both original failures remain retained.

## Required full qualification ledger — passed

| Requirement | Current result |
| --- | --- |
| Focused UI tests, including four new boundary regressions | 25 passed, zero failed/ignored; initial E0616 retained, not added to the full total |
| Locked/offline native all-target/all-feature Cargo check | Passed, 0.164 seconds |
| Full locked/offline all-feature tests including ignored cases | **1,203 passed, zero failed/ignored/filtered**, 311.970 seconds command / 311.77 seconds test report; `--include-ignored --test-threads=8` |
| Strict private rustdoc | Passed with private items and warnings/missing docs denied, 1.317 seconds |
| Warnings-denied all-target/all-feature Clippy | Passed, 8.483 seconds |
| Scoped formatting | Three changed parent modules checked in isolated shadows, 0.468 seconds; inherited blank-line policy retained |
| Native Cargo-JSON-selected release | Passed, 14.448 seconds |
| Native release headless help | Passed, exit 0, 0.002 seconds |
| Protected headless help | Passed, exit 0, 0.004 seconds |
| Protected 116 identity/parity/mode | Passed: 12,882,768 bytes, 0555, exact native parity and SHA-256 below |
| Closed scope, compiler/fixture/core inputs, siblings, HEAD/index and protected 107–115 preservation | Passed; root's post-qualification source audit also matches the sealed exercised inputs |
| Final documentation/local-link/preservation audit | **Passed: seven documents, 243 local links, 12 review paths** |
| Fresh operator acceptance of mode-switch correction | Not performed |
| Complete native Continuous loop | Incomplete; UI disabled |

Final root audit records `passed-final-documentation-and-preservation-audit` in
`/home/charlie/tmp/pyramid-mode-switch-validation-mak21b_6/final-audit.json`.
Its initial refusal on changed trigger-tree telemetry is retained in
`final-audit-first-refusal.log`: exactly two appended `read`/`Bash` records for
the new verification and live-test documents were checked against the prior
pinned history prefix, together with the existing session metadata. The second
audit-only refusal required the canonical telemetry classification string and
is retained in `final-audit-classification-refusal.txt`. Only the two exact
telemetry hashes/classifications were refreshed; those files were left intact.
The successful audit preserves every compiled/exercised input, all original
fixtures, all three HEADs/indexes, both siblings and protected releases 107–115.
Further audit-only telemetry refreshes after the final status reads are recorded
in `telemetry-review-rounds.jsonl`; the corresponding refusal is retained in
`final-audit-after-status-refusal.log`. No Cargo workload was repeated after the
passing required checks.

The exact toolchain is rustc `1.101.0-nightly (b6eeb93c9 2026-10-18)`,
Cargo `1.101.0-nightly (3960fa69b 2026-10-12)`, native target
`x86_64-unknown-linux-gnu`. Toolchain/Cargo observations took 0.016 seconds each.
`verification.json` records `passed-required-checks`, starting at
**2026-10-19 20:03:44.815667 UTC** and finishing at **20:09:26.826866 UTC**.
The command seal, exact argv/environment, exercised inputs, logs and results
remain under `/home/charlie/tmp/pyramid-mode-switch-validation-mak21b_6/runs/0002`.
No successful Cargo workload is repeated for this documentation finalization.

Work root: `/home/charlie/tmp/pyramid-mode-switch-validation-mak21b_6`.
Captured flat three-repository baseline SHA-256:
`9b46f263e188c480ba03412f42d5f59499a33302cd4af733c3a9af52cf04d88d`.
All three HEAD/index identities, frozen sibling files, all existing fixtures,
unrelated source and protected 107–115 passed preservation checks. The earlier
qualified [2.6.115 record](Verification-2.6.115.md) remains historical evidence.

## Explicit approved review paths

The root-owned `review-paths.json` declares exactly these 12 literal application
paths. Both sibling arrays are empty. This list declares scope and grants no
permission to mutate frozen files outside it.

```text
/home/charlie/repo/RUST/qmp-qemu-socket/CHANGELOG.md
/home/charlie/repo/RUST/qmp-qemu-socket/Cargo.lock
/home/charlie/repo/RUST/qmp-qemu-socket/Cargo.toml
/home/charlie/repo/RUST/qmp-qemu-socket/README.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Codex-Current-Status.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Live-Test-2.6.116.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Pyramid-Evidence-Index.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Pyramid-Implementation-Schedule.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Verification-2.6.116.md
/home/charlie/repo/RUST/qmp-qemu-socket/src/app.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/app/pyramid_workflow.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/app/pyramid_workflow_tests.rs
```

## Protected artifact — verified; final documentation audit passed

Protected destination:
`/home/charlie/tmp/pyramid-mode-switch-validation-mak21b_6/release/qmp-qemu-socket-2.6.116`.

- Size: **12,882,768 bytes**; regular protected executable, mode **0555**.
- SHA-256: `2b3efc9c64ab9edd39868969b63d684d3808a4fe9ed6d1ec94c166f370ab8fb0`.
- Observed: **2026-10-19 20:09:25.238715 UTC**.
- Exact parity with the Cargo-JSON-selected native release at
  `/home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket`.
- Native and protected headless help returned zero, 0.002/0.004 seconds respectively.

Root's read-only preflight at **2026-10-19 19:57:17 UTC** found protected **2.6.115**,
PID **92970**, matching SHA-256
`7961510c1e3e49339e7728907c537384c976530a89512dbefc3aee35d0b9600e`.
This is a dated existing-process observation; verification did not launch or
replace a GUI controller. See the conditional [operator brief](Live-Test-2.6.116.md).

## Proposed promotion text — review only

Proposed conventional title:

```text
fix(pyramid): refresh automatic recon names after mode changes
```

Proposed body, for review after root records the final documentation audit:

```text
Share request preparation between Manual Recon and automatic Start. Refresh
still-automatic acquisition names before arming and consuming current-context
consent, then record the prepared request details. Preserve literal overrides,
mode/method receipt invalidation and the worker's no-overwrite publication rule.

Cover completion-to-Manual, method-change/retry, literal/mixed overrides and naming
failure with headless boundary regressions. Retain the initial E0616 compile
failure, first full-run local-fixture accept failure, and successful focused
retries. The unchanged full rerun with eight test threads passed all 1,203 tests
including ignored cases. Locked check, strict private docs, Clippy, scoped format,
release, both headless-help checks and protected/native artifact preservation
passed. Final documentation audit and fresh operator acceptance remain separate.
```

This is prepared review text only. No commit, push, installation or promotion has
been performed. Historical 115 evidence and protected executables are preserved.

## Code Analysis

The correction belongs to the UI request boundary. It grants no new capture,
input, restoration or overwrite authority; worker validation/publication remain
independent. Naming is fallible and must finish before consent/request creation.
Caller-owned strings and a checked Result carry errors; no unsafe code or new
dependency is introduced. Exact overrides can still fail on an existing path,
which is the preserved protection rather than an automatic overwrite fallback.
Focused tests cover UI boundary cases, and the full 1,203-test run and required
host checks passed against sealed inputs. Root's final document audit and observed
guest acceptance remain distinct requirements.
