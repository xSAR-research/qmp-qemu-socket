# Verification — app 2.6.112 RECON reset timing

Status: **required host checks, protected artifact and final
documentation/preservation audit passed; live acceptance pending**. App
2.6.112 earns one correction build and no additional minor. Read-only runtime
preflight observed an existing 2.6.111 controller; later operator replacement
must be deliberate after the final audit.
No guest/GUI launch, input, installation, process replacement or promotion is
performed by this correction.

## Approved correction and retained contract

Only stock-only Pyramid RECON reset observation pacing changes. The normal path
is one guarded Undo All → two fresh matching full confirmation-modal captures →
one guarded OK → two fresh consecutive observations of the original tableau,
initial LEFT and positively empty RIGHT → complete JSON publication. Four
configured one-second waits are removed from that path. This is a configured
pause reduction, not a measured end-to-end performance result; capture, input,
recognition overhead and guest reliability remain unmeasured.

After Undo All, the first modal search captures immediately. A recognised
unchanged exhausted board permits a one-second wait only when another search
observation remains within the existing budget. Once the modal is found, the
second fresh matching modal observation is immediate. A changed modal or
unexpected scene withholds OK. OK is sent once through the existing guard.

After OK, restoration captures immediately. A recognised unchanged exhausted
board or the same confirmation modal resets consecutive restoration proof and
waits one second only if another attempt remains. Two fresh consecutive exact
original-board/pile observations remain mandatory; the second positive capture
has no unconditional wait. Exhaustion of either six-observation budget fails
closed. Input is never repeated to compensate for a missing effect.

The 180-capture and 24-D bounds, 300-second cooperative deadline, active/context
checks, VM/pointer probes and separately acknowledged input remain. STOP or
context loss revokes further input and publication; uncertain non-idempotent
input is never replayed. The initial pair of board observations retains its
one-second wait, each acknowledged D retains at least 500 ms, and stock
advancement/exhaustion, complete unique inventory, source recognition,
restoration and atomic no-overwrite export semantics remain. No other timing,
recognition profile, search rule, UI control or game changes are included.

Bundled native-v13 and its schema-2 matcher remain frozen. xSAR 0.6.102, console
0.8.4 and protected stable 107–111 remain frozen. The required full test scope
passed, earning one app correction build and no additional minor; unchanged
rebuilds and documentation finalisation earn no further increment.

## Host evidence — passed required checks

Work root: `/home/charlie/tmp/pyramid-recon-reset-validation-fcyu19f8`.
`baseline.json`, the captured `before` source and `review-paths.json` define the
retained task baseline and closed correction scope. The verifier preserves each
attempt's commands, logs, source snapshots and first-failure evidence. The
top-level `verification.json` records `passed-required-checks` after audit-only
attempt `runs/0004`; required command results are retained from `runs/0003`.

| Check | Result | Recorded evidence |
| --- | --- | --- |
| Focused RECON core regressions | Passed: 25/0/0, 1,068 filtered; compile 11.65 s, tests 0.53 s | `focused-reset.log`; six new reset tests included |
| Repository-selected nightly/compiler/Cargo/native host | Passed | Rust 1.101.0-nightly `b6eeb93c9` (2026-10-18), Cargo 1.101.0-nightly `3960fa69b` (2026-10-12), native `x86_64-unknown-linux-gnu` |
| Changed ranges in two modules, isolated rustfmt shadow | Passed, 0.065 s | `runs/0003/format-results.json`, `runs/0003/qmp-qemu-socket-parent-format-shadow.log`; required blank-line spacing retained |
| Locked/offline all-target/all-feature check | Passed, 1.718 s | `runs/0003/app-check.log` |
| Locked/offline all-feature tests including ignored | Passed: 1,093/0/0; 175.94 s harness, 179.257 s command | `runs/0003/app-tests.log` |
| Strict private rustdoc, no dependencies | Passed, 1.066 s | `runs/0003/app-rustdoc.log` |
| All-target/all-feature Clippy, warnings denied | Passed, 6.932 s | `runs/0003/app-clippy.log` |
| All-feature native release, Cargo JSON selection | Passed, 13.165 s | `runs/0003/app-release.log` |
| Native executable headless help | Passed, exit 0, 0.002 s | `runs/0003/app-help.log` |
| Protected executable parity/mode/hash/headless help | Passed, help exit 0, 0.004 s | `verification.json`, `runs/0004/stable-app-help.log` |
| Read-only process/image preflight | Observed existing protected 111; images 0 files/0 bytes at 2026-10-19T15:12:33.580190+00:00 only | `runtime-preflight.json`; details below |
| Final documentation/link/preservation audit | Passed: 6 documents, 134 local links, 10 review paths | `final-audit.json`; all three HEADs/indices, frozen siblings, reader fixtures and stable artifacts preserved |

The required Cargo operations use the repository-selected nightly and explicit
native target `x86_64-unknown-linux-gnu`:

| Operation | Exact argv / additional environment |
| --- | --- |
| Check | `cargo +nightly check --locked --offline --target x86_64-unknown-linux-gnu --all-targets --all-features` |
| Tests | `cargo +nightly test --locked --offline --target x86_64-unknown-linux-gnu --all-features -- --include-ignored` |
| Private docs | `cargo +nightly doc --locked --offline --target x86_64-unknown-linux-gnu --all-features --no-deps --document-private-items`; `RUSTDOCFLAGS=-D warnings -D missing_docs` |
| Clippy | `cargo +nightly clippy --locked --offline --target x86_64-unknown-linux-gnu --all-targets --all-features -- -D warnings` |
| Release | `cargo +nightly build --locked --offline --target x86_64-unknown-linux-gnu --all-features --release --message-format=json` |
| Help | Native and protected executable `--help`, with `DISPLAY` and `WAYLAND_DISPLAY` empty |

Focused tests cover immediate modal/restoration proof without waits, recognised
stale-scene retry waits, exact six-observation exhaustion, no OK replay, missing
export on failure and STOP before the second restored capture. Scripted tests
do not prove native recognition or guest effects. Focused counts overlap the
full suite and are not added to its total. No unchanged xSAR feature matrix,
console suite or private consumer smoke is rerun for this app-only timing change.

### Retained attempt history and audit-only resumption

Attempt `runs/0001` stopped at preflight before any Cargo command with the exact
error `Reviewed operational bytes changed: .tree-ring/memory.sqlite-shm`.
SQLite automatically removed its transient WAL/SHM read-coordination sidecars
after a read; the WAL was empty. The main database retained its hash, passed
its health check and held zero memories. The private verifier now classifies only those exact automatic
removals, separately from the correction files. The failed record is retained;
this was a preservation-classification stop, not a compiler or test result.

Attempt `runs/0002` passed the locked check, then full tests reported **1,092
passed, 1 failed, 0 ignored**, exit 101, 175.66 s harness/175.830 s command wall.
The sole failure was
`pyramid_recon::paired_tests::paired_scan_observes_twenty_frames_with_exact_draw_and_restoration_pacing`,
with ``assertion `left == right` failed`` at
`src/pyramid_recon/paired_tests.rs:221:5`. The actual trace captured immediately
after Undo All and OK; the expected trace still contained the four removed
`Wait(1s)` events. Only those four obsolete expected waits were removed from
the test. The exact ordering, 20 captures, 24 Draw operations, 26 probes and two
modal/two restored captures remain asserted. Production code was unchanged by
that test correction. The review list therefore adds the paired-test file as
its tenth path. Both failed attempts and the original test output remain intact.
Neither the failed run nor the focused-only pass earned the build increment.

Attempt `runs/0003` then passed all required commands, including **1,093 passed,
0 failed, 0 ignored**. Its final preservation audit stopped on one newly added,
genuine automatic lifecycle receipt, with the exact error:

```text
Undeclared file delta: qmp-qemu-socket:.tree-ring/activation/receipts/codex/9c5eb7aa01cbe5c53691e3ada8fda18455b20b577f9fd631c9ad18da634aad1a/receipt-7a69b860-bc79-444e-8685-3e5f756d087a.json: added
```

The receipt recorded a zero-result recall; it was retained unchanged, not
deleted to satisfy the audit. Attempt `runs/0004` narrowly resumed the audit
after validating that exact receipt, compiler/source data and retained command
logs against their hashes. It created the protected candidate and passed
headless help; **no Cargo or formatting command was repeated**.
`audit-resumption.json` records
`passed-command-evidence-reused-for-audit-only-finalization` and the retained
`runs/0003/verification.json` hash
`560480d3c72d1aacfe5020825c552fab6497d99d138be70de7051af337a3a2bd`.

The completed required-check audit preserves all three Git HEADs and indices,
all xSAR/console files, unrelated source/evidence and protected stable 107–111.
Six exact runtime paths are classified separately in `runtime-receipts.json`:
four genuine zero-match recall receipts and two automatic trigger-tree telemetry
files. These operational files are excluded from the ten promotion paths and
left intact. Final documentation/preservation audit passed: 6 documents, 134 local links and 10 review paths. Recorded at 2026-10-19T15:15:59.335274+00:00 in `final-audit.json`; the final repeat verifies this result-driven documentation update.

## Protected artifact and runtime boundary

Verified protected executable:
`/home/charlie/tmp/pyramid-recon-reset-validation-fcyu19f8/release/qmp-qemu-socket-2.6.112`.
It is byte-identical to the Cargo-selected native release at
`/home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket`,
12,733,200 bytes, mode 0555, SHA-256
`932d60177031f002cbc23c21c56335c5924522c8a71487e1ef818966d20c0041`.
Protected headless help passed with exit 0. Artifact verification is recorded at
2026-10-19T15:11:21.921656+00:00; it does not replace or qualify a running process.

Read-only preflight at **2026-10-19T15:12:33.580190+00:00** observed PID **45759**
running protected 2.6.111 with `--image`, executable
`/home/charlie/tmp/pyramid-diamond-v13-validation-ugnvhgv9/release/qmp-qemu-socket-2.6.111`,
SHA-256 `32638935cae71f240a6c6e733168dbcd1217bad916af0bd3fa10c26aac97b55b`.
`/home/charlie/tmp/images` held zero files/bytes at that instant only. The earlier
15:09:15.012164 UTC no-controller/empty-image snapshot is historical and does
not establish later state. No process was stopped or replaced by this work.

The later operator command requires a passed final documentation/preservation
audit, preserved evidence and deliberate closure of the existing controller:

1. Launch the verified protected candidate with GAME WIN image retention:

   ```zsh
   /home/charlie/tmp/pyramid-recon-reset-validation-fcyu19f8/release/qmp-qemu-socket-2.6.112 --image
   ```

Startup clears `/home/charlie/tmp/images`. Preserve existing evidence first;
an old empty-directory observation does not establish current contents. The
agent performs no GUI launch, QMP connection, input or process control here.
The operator must verify the actual running executable/hash after launch.

## Goal, live acceptance and remaining work

Charlie reports Single board works. That report is separate from local tests
and a fresh recorded 2.6.112 operator trial. The larger goal remains **ACTIVE
and incomplete**. Native Continuous remains disabled/unavailable; M10 is
Awaiting verification and M11 is Planned. Native 0/3, 1/3 and 2/3 source images
are now available but uncalibrated: they alone establish neither the game mode
nor challenge objective, terminal/restart authority or redeal semantics. The
master schedule now defines challenge target-value counting, including DL/DR
removals and zero DRAW/recycle credit; no challenge implementation is included.

Charlie also reports high idle CPU after route completion. Its cause remains
under read-only investigation and requires runtime profiling; this RECON
correction includes no CPU fix.

[M12](Pyramid-Implementation-Schedule.md) is the immediate measured pacing and
optional bounded, collapsed timing-slider milestone. It requires observed
end-to-end timings, repeatable margins and live acceptance; this correction
alone does not complete it. M13 is the longer-term professional, generic xSAR
naming review and requires a separate agreed API/consumer migration.

[Live test](Live-Test-2.6.112.md) defines the later operator trial. It must use
fresh complete same-session acquisition, proved restoration, exact published
bytes and the current prepared plan. Retain first failures and distinguish
delivery acknowledgement, observed effect, human report and controller proof.
No interrupted route, uncertain operation or old export is replay authority.

## Review scope and proposed promotion

The closed correction list contains these **ten absolute paths**, matching
`/home/charlie/tmp/pyramid-recon-reset-validation-fcyu19f8/review-paths.json`.
It defines the correction delta against the retained task baseline; accumulated
earlier dirty work and historical 2.6.111/earlier records are preserved. Both
sibling review lists are empty. Operational telemetry is classified separately
by exact path/hash in the preservation audit, and is outside promotion files.

```text
/home/charlie/repo/RUST/qmp-qemu-socket/Cargo.toml
/home/charlie/repo/RUST/qmp-qemu-socket/Cargo.lock
/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_recon.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_recon/paired_tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/README.md
/home/charlie/repo/RUST/qmp-qemu-socket/CHANGELOG.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Codex-Current-Status.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Pyramid-Implementation-Schedule.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Verification-2.6.112.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Live-Test-2.6.112.md
```

Proposed Conventional Commit title:

```text
fix(pyramid): remove unconditional RECON reset waits
```

Proposed body, subject to final recorded verification:

```text
Capture Undo All confirmation and original-board restoration immediately while
retaining two fresh matching proofs, guarded input and bounded stale-scene retries.

Preserve STOP/context, input uncertainty, inventory and atomic export gates.
Record app 2.6.112 host evidence and the operator brief; keep xSAR, console and
protected stable 107–111 frozen. Track the active pacing and future naming work.
```

Charlie owns commit/push unless separately delegated. No staging, commit, push,
publication or registry operation is performed by this correction.

## Code Analysis

This pacing change preserves the existing checked ownership and captured-scene
types; it introduces no unsafe Rust or new allocation/storage format. The main
regression risk is sampling animation earlier. Two fresh matching full-modal
captures and two consecutive original-board/pile observations still supply
authority; recognised stale scenes get bounded delayed retries, while changed
or unexpected scenes fail closed. Six-observation, 180-capture, 24-D and
300-second bounds remain. STOP/context loss, uncertainty and failed restoration
withhold subsequent input/publication. Host regressions can verify that policy;
only a fresh operator trial can establish native timing reliability.
