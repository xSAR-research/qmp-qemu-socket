# Verification — qmp-qemu-socket 2.5.105

Date: 19 October 2026, Australia/Brisbane.

**Host verified.** The first complete required sequence passed all **965 tests,
zero failed/ignored**, locked check, strict private rustdoc, Clippy, scoped
formatting, native release and headless help. The changed revision earns exactly
one **2.5.105** correction build, with no new minor milestone. xSAR stays 0.5.101
and accepted console stays frozen 0.8.4. **M01–M09 accepted; M10 Awaiting
verification; M11 Planned.** Required preservation and the final audit of all
seven documents, 103 relative links and 19 review paths passed. Beast acceptance,
installation and promotion remain separate and have not occurred for 2.5.105.

## Authorised scope and baseline

The correction is limited to Pyramid + Shortest Path presentation, explicit
button consent, matching completion status and automatic private error evidence.
It does not increase search limits, preserve a consumed receipt for retry or
change Daily Challenge completion/progress predicates. No guest Solver activation,
automatic Solve after Plan, route resumption or next-board input is added.

The pre-correction snapshot is
`/home/charlie/tmp/pyramid-workflow-9tl0wwok/baseline.json`, created
2026-10-18T22:41:30.091984+00:00. It predates these UI source writes and already
contains accepted dirty work from earlier milestones. Compare against that
snapshot rather than attributing the whole working-tree diff to this correction.

| Repository | Baseline HEAD | Baseline index SHA-256 |
| --- | --- | --- |
| qmp-qemu-socket | `842646e24b179fb4c4ee2cd04f1dc28b8cfed30b` | `3e64bf120a38e502340e9d5aab8450a20b83240a73df98a0aa2eff8d2716c168` |
| xsar | `00986b8d621803fc98da85616444fc76c14638a9` | `8ae82c5fbcdc791ea66a47ee17924147ccf57ff8c08223d6a33b738126e7e73c` |
| basic-route-planning | `22f52d7ec5fc867b542dd7ca4c20fe2b9fbe2e84` | `347f0f62f307234d1e9f813ea63fc84b35bba25b8330ba78deb08bd33dac2215` |

No branch, staging, commit, push, publication, installation or process replacement
is part of this correction. Historical verification/live briefs, Game-Win-Handoff,
old fixtures, AGENTS and unrelated operational files remain outside the edit set.

## Three-step UI and authority contract

Normal Pyramid Shortest Path displays **1. Recon**, **2. Plan route** and
**3. Solve board**, concise phase/status, preview and STOP. It hides generic
capture/manual controls, settings/output panels and already-open technical
windows. Other games and Computer Vision retain their existing UI path.

- **Recon** explicitly authorises only the immediate bounded acquisition in the
  current game/strategy/socket/generation. It reads all 52 identities, proves
  original-board restoration and publishes the complete deal. Start on an
  untouched supported board; keep Solitaire foreground and Solver disengaged.
  An unavailable Solver remains allowed.
- **Plan route** requires a successful current-session restored receipt and exact
  selected JSON bytes. It performs search and independent replay without guest
  connection/input. It does not automatically start execution.
- **Solve board** explicitly arms only the current prepared plan immediately
  before dispatch. Existing worker-side context, receipt, origin, per-gesture
  evidence, STOP and transport checks remain mandatory. Dispatch consumes the
  plan; uncertain delivery or failure grants no resume.

Normal per-run consent checkboxes are removed. The button press populates the
existing one-shot consent field only for its immediate operation; generated names,
hidden defaults and diagnostic metadata supply no input authority. Bundled
native-v8 calibration, exact successful-export handover, next-acquisition rotation
and independent literal manual overrides keep their prior contracts.

Matching `ComputedRouteFinished` now reconciles stale Capturing/Verifying status
to Ready after unstopped success or Uncertain after failure/STOP. An already
Detached state and existing cooperative STOP clearing order are preserved.
Wrong-owner, stale-context and wrong-run completions cannot release another run.
This is a defensive lifecycle fix, not proof of the cause of the old live badge.

## Automatic failure packets

`pyramid_diagnostics::PyramidPhase` identifies Reconnaissance, Planning or
Execution. `PyramidDiagnostic` carries exact context, phase, optional run ID,
optional advisory local failure ID, original error, packet directory and visible
persistence warnings. Backend diagnostics are sent before their dedicated finish
event. UI-local validation/queue failures ask the worker to persist a packet
through `WorkerHandle::record_pyramid_failure`; this command performs no guest I/O.
Context/phase/run checks and a checked local correlation ID prevent an older
notice from reopening an error during a newer request.

The worker reserves `pyramid-error-{epoch_ms}-{pid}-{serial}` under the existing
session-log directory policy: nonempty `QMP_SESSION_LOG_DIR`, then existing
`$HOME/tmp`, then the runtime temporary directory. It requests 0700 directories
and exclusive 0600 files; umask may tighten permissions. It never overwrites an
existing artifact. Names use checked monotonic serials and at most 64 collision
attempts. No capture or input is requested merely to report an error.

| Artifact | Source and bound | Failure behaviour |
| --- | --- | --- |
| `error-state.json` | Context, phase, identities, error, counters/limits/fields, artifact outcomes; at most 256 KiB | Explicit omissions/fallbacks remain visible; metadata write failure remains in returned UI warnings. |
| `error.log` | Plaintext error report, not the complete application session log; at most 128 KiB | Preserve partial bytes and report the failure; independent artifacts still proceed. |
| `deal.json` | Exact source bytes already owned by the worker, at most 65,536 bytes | Absent when unavailable; oversized data refused. It cannot recreate a live receipt. |
| `first-unmatched.png` | Existing retained first mismatch, losslessly encoded original pixels; at most 32 MiB | Missing/oversized/encoding/write failures are explicit; no substitute capture. |
| `last-observed.png` | Existing latest original frame, same PNG bound | Identical encoded pixels may reference the first artifact instead of duplicating it; differing first/latest frames remain separate. |

Optional structured details are bounded at 64 KiB, depth 32 and 8,192 visited
nodes; persisted text/warnings have further byte/count caps. The returned UI
error preserves the original caller text even when its stored copy is bounded.
Packet metadata marks diagnostic-only, no input authority and no resume authority.
Recon packets identify the existing native diagnostic directory rather than
requesting another frame. Pre-capture/UI validation failures can have no source
document or PNG. Disk-full, permission, collision and partial-write failures
retain the original error and any already created files, without recursive retry.

The error view automatically shows the actual failure, dispatch field values,
session log and saved packet path, including warnings. Literal profile/output/
Deal-ID/route/socket overrides remain available there. Buffered socket edits
require explicit Apply so editing cannot hide the recovery fields after the first
character. Error-only Capture Frame/PNG requests new advisory pixels; it does not
replace the saved first mismatch. Persistence itself runs on the worker and may
block on external filesystem I/O; STOP remains cooperative.

This is an explicitly authorised **failure-only** PNG persistence exception.
Successful ordinary gameplay captures remain unsaved; the existing `--image`
GAME WIN sequence policy is unchanged. Failure packets use already captured
pixels and do not alter normal capture frequency or the guest.

## Retained live boundaries — unchanged in this slice

The retained 2.5.104 log is
`/home/charlie/tmp/qmp-qemu-socket-1792359297304.log`; read-only evidence is under
`/home/charlie/tmp/pyramid-completion-r8u_svib/`. Its independently replayed
41-action route reached **40 verified logical actions**, **65 acknowledged
gestures**, then `BudgetExceeded(EffectObservations)` with first unmatched
`execution board scene[2] mismatch at [931,695,60,50]`. Charlie reported a cleared
board and transition. That human observation is separate from the controller's
unverified final effect; no controller-confirmed tableau/challenge completion is
claimed. The route is consumed and cannot resume.

Both original acquisitions restored/published with 31 captures, 24 acknowledged
draws and acknowledged Undo All/confirmation. The later second-board baseline
shows `Boards cleared 1/2`, while the shared three-board probe finds 0/76 black
pixels and classifies GameComplete. `second-board-diagnosis.json` reproduces the
source predicate on retained frames, not the exact final-effect capture sequence.
The two-board Daily Challenge completion/progress rule is **not corrected here**.

The second restored deal hit MemoryLimit under unchanged limits: 2,000,000
states/expansions, 512 MiB estimated storage and 120 seconds. At least 1,024 bytes
are accounted per history, implying at most 524,288 histories before other
capacity accounting; this is not process RSS. The smaller effective ceiling is
intentional current behaviour, not a newly increased cap. Planning currently
takes its acquisition receipt and drops it on failure, leaving no prepared plan.
The intact export remains offline evidence; retry preservation is not implemented.

Execution bounds also remain unchanged: 102 actions, 126 gestures, six ordinary
phase observations, twenty final observations, a 1,200-second cooperative deadline
and capture allowance `max(2+6*(G+A)+20, 12*A+G+16)`. No broad completion
recognition, glyph/signature relaxation, cleanup click or automatic next-board
behaviour is added.

## Current host verification — passed

Work root: `/home/charlie/tmp/pyramid-workflow-9tl0wwok`.
`verification.json` reports `passed-required-checks`; the verifier exited zero.
`commands.json` retains exact argv, cwd, exit status, durations and log paths.
This first complete required sequence had no failed check/test. The initial
integration check's deprecation warning was corrected before the full scope;
prior 2.5.104 first failures remain unchanged historical evidence.

Actual repository-selected toolchain: rustc **1.101.0-nightly**
(`8006091eb`, 2026-10-17), full revision
`8006091eb71ae7d9e9bccb66c6122c9d68af4466`, LLVM 23.1.4;
Cargo **1.101.0-nightly** (`3960fa69b`, 2026-10-12), native target
`x86_64-unknown-linux-gnu`. Cargo validation used explicit `+nightly`, locked,
offline commands and that target. Check/Clippy covered all targets/features;
full tests/private docs covered all features. Release used default application
features and Cargo JSON artifact selection. No global toolchain default changed.

| Check | Actual retained result |
| --- | --- |
| Initial integration check | Passed in 1.73s with deprecated `fetch_update` warning; replaced with compiler-recommended `try_update` before subsequent test compilation and the clean full scope. |
| Focused UI tests | `ui-focused.log`: 13 passed, zero failed/ignored, 950 filtered; three-step consent/gates, error visibility, diagnostic identities, recovery editor and terminal completion identity. |
| Focused packet writer | `writer-focused.log`: eight passed, zero failed/ignored, 955 filtered; 0.01s. Exact artifacts/private modes, collision/overflow, omission/partial-write/bounded metadata and local ID cases. |
| Focused backend integration | `backend-focused.log`: two passed, zero failed/ignored, 963 filtered; compilation 4.19s/tests 0.00s. Local refusal identity/STOP without transport; original error/packet precedes matching calculation finish. |
| Locked all-target/all-feature check | Passed; 4.974s, `app-check.log`. |
| Complete locked tests including ignored tests | **965 passed, zero failed/ignored**, zero filtered; command 136.625s, test report 134.28s, `app-tests.log`. Focused filters overlap this total. |
| Strict private rustdoc | Passed; 0.966s, `app-rustdoc.log`; `RUSTDOCFLAGS='-D warnings -D missing_docs'`, private items included. |
| Clippy | Passed with warnings denied; 5.324s, `app-clippy.log`. |
| Scoped formatting | Six scoped modules passed 0.064s; four parent shadow ranges passed 0.165s, preserving inherited blank-only spacing. Details below; no whole-package formatting pass is claimed. |
| Native release | Passed; 11.975s, `app-release.log`; exact Cargo-selected artifact and SHA-256 below. |
| Native headless `--help` | Passed; 0.004s, `app-help.log`; text distinguishes unsaved successful ordinary captures from private Pyramid failure packets. |
| Compiler inputs and preservation | Passed against the pre-correction baseline: all three indices, every xSAR/console file, unrelated source/evidence and verified compiler inputs preserved; automatic telemetry exception below. |
| Read-only runtime/image observation | Completed; older PID 683122 remained at a different executable/hash; images zero files/bytes at observation only. No process replacement or new live identity. |

The six scoped formatter files were `src/app/pyramid_workflow.rs`,
`src/app/pyramid_workflow_tests.rs`, `src/pyramid_diagnostics.rs`,
`src/worker/computed_route.rs`, `src/worker/computed_route/wire_tests.rs` and
`src/worker/pyramid_diagnostic_tests.rs`. Four parent shadow-range checks passed;
only `src/app.rs`, `src/worker.rs` and
`src/worker/pyramid_reconnaissance.rs` retained required inherited blank-line
contexts. Nonempty changed lines match the formatter. Preserve the established
spacing rather than claiming or applying blanket repository formatting.

## Exact artifact, preservation and runtime observation

Compiler-selected native artifact:

```text
/home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket
```

SHA-256:

```text
77eb7763f01f7698399582466f879e726689059697c556abf7e1cbaa96600e7a
```

The required verifier preserved compiler inputs, all three Git indices, every
file in xSAR and the accepted console, and unrelated existing source/evidence.
Only automatic `.trigger-tree/history.jsonl` and
`.trigger-tree/sessions/b1065bcab784f174a496f2aa5538322e.json` changed; both
were classified as operational telemetry, recorded with before/after hashes in
`verification.json` and left untouched. No router or Tree Ring configuration
was modified by this correction. The final documentation/review-path audit
passed after ownership return at 2026-10-18T23:09:17.406049+00:00, checking all
seven documents, 103 relative links, 19 review paths and the unchanged verified
compiler inputs/artifact. `final-audit.json` records the latest audit result.

`runtime-preflight.json` observed at
**2026-10-18T23:05:38.821658+00:00** that **PID 683122** still ran:

```text
/home/charlie/repo/RUST/qmp-qemu-socket/target/release/qmp-qemu-socket
argv: target/release/qmp-qemu-socket --image
SHA-256: a47a389a08d069e13a8a520ee95321613ffba8944132a789b88c710a9800e400
```

That differs from the newly verified native artifact's path and hash. Building
neither replaced nor stopped it and does not establish the installed executable
or a newly running 2.5.105 identity. `/home/charlie/tmp/images` contained zero
files/zero bytes at that instant only. No GUI launch, installation, process
control, QMP connection or guest action was performed. Preserve the old log/
preview and recheck later image state before deliberate operator replacement;
startup clears the image directory.

[Live-Test-2.5.105.md](Live-Test-2.5.105.md) gives the exact later launch command
and explicit Recon → Plan route → Solve board brief. Required host success does
not establish guest completion, permit resumption or resolve the retained
Daily Challenge/MemoryLimit/receipt boundaries.

## Explicit current correction review paths

These 19 paths (12 source/manifests and seven documents) separate the UI/diagnostic correction from earlier accepted dirty
work in overlapping files. These paths describe only the correction delta, not
a complete promotion of the accumulated uncommitted M10 stack. HEAD remains
`842646e` from the earlier 2.2.100-era source. This is a review list, with no
staging or promotion authority.

```text
Cargo.toml
Cargo.lock
src/main.rs
src/app.rs
src/app/pyramid_workflow.rs
src/app/pyramid_workflow_tests.rs
src/pyramid_diagnostics.rs
src/worker.rs
src/worker/pyramid_reconnaissance.rs
src/worker/computed_route.rs
src/worker/computed_route/wire_tests.rs
src/worker/pyramid_diagnostic_tests.rs
README.md
CHANGELOG.md
docs/Codex-Current-Status.md
docs/Pyramid-Implementation-Schedule.md
docs/Pyramid-Evidence-Index.md
docs/Verification-2.5.105.md
docs/Live-Test-2.5.105.md
```

`src/app/pyramid_defaults.rs`, native-v8 calibration/fixtures and older records
are retained unchanged. Both sibling trees and all indices are outside the edit
scope. Record any automatic operational telemetry exceptions explicitly during
the final preservation audit rather than discarding those files.

Suggested Conventional Commit title if separately authorised:

```text
fix(pyramid): simplify the workflow and retain failure evidence
```

Suggested body, for review only:

```text
Present explicit Recon, Plan route and Solve board steps with automatic defaults,
single-use button consent, concise progress and error-only technical controls.
Retain worker-side receipt/context/STOP/evidence gates and no automatic execution.

Persist bounded private failure packets from existing source/frames without new
capture or input. Surface exact errors, fields, logs, packet paths and partial
writes; reject stale diagnostics and reconcile matching terminal run status.

Keep search limits, consumed planning receipts and Daily Challenge completion
rules unchanged. xSAR 0.5.101 and accepted console 0.8.4 remain frozen.
The first complete required sequence passed 965 tests, zero failed/ignored, locked
check, strict private docs, Clippy, scoped formatting, native release and help.
All indices/frozen siblings/verified inputs/unrelated evidence were preserved;
automatic telemetry was classified and left untouched. The older process was
not replaced. M10 live acceptance remains pending and M11 stays Planned.
```

## Code Analysis

- Input authority remains in the worker. UI buttons arm only their immediate
  context-bound request; packet IDs and saved evidence cannot create permits.
  STOP is cooperative, uncertain input is never replayed, and plans stay consumed.
- Safe Rust ownership retains existing source/frame bytes for diagnostics. Packet
  size/detail/depth/count caps bound serialisation and artifact output; no new
  capture or arbitrary source-path reread is introduced for saving a packet.
- Private exclusive directory/files prevent accidental overwrite; checked serials
  do not wrap. Existing parent filesystems can still fail or stall. Partial files
  remain inspectable with visible warnings rather than being reported complete.
- Context/phase/run/local-failure checks prevent stale advisory packets and finish
  events from replacing a newer request. Matching completion ends stale busy
  presentation while preserving Detached and cooperative STOP ordering.
- Search capacity, receipt consumption and two-board completion mismatches remain
  material limits. Better UI/error evidence does not establish route completion,
  permit a retry or resolve Daily Challenge compatibility. Host and live evidence
  require separate acceptance.
