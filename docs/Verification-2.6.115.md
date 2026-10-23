# Verification — application 2.6.115, host verified

Status: **REQUIRED HOST CHECKS PASSED; protected artifact verified.** The sealed
audit-only preservation resumption passed. Root's final documentation, link and
preservation audit passed; fresh live acceptance has not been performed.

## Scope and completion boundary

This app-only revision combines normal first-board session binding/restart
groundwork with three measured corrections exposed during operation of 2.6.114.
Native Continuous remains disabled because actual normal `2 of 3` and `3 of 3`
references are missing. The whole goal remains active and incomplete; M10 is
Awaiting verification, M11 Planned, M12 In progress and M13 Planned. Challenge
objectives/redeal control, alternate-theme support and intermittent idle CPU
remain unfinished. xSAR 0.6.102, console 0.8.4 and protected 107–114 remain frozen.

No host test result establishes new guest acceptance, complete challenge
automation or a successful native repeated three-board loop. Existing bounded
New Game/Play control policy can be reused from its qualified entry roles;
separate whole-menu PNG fingerprints are not an added implementation prerequisite.

## Three distinct exposed failures

| Failure | Correction contract | Retained evidence |
| --- | --- | --- |
| Stock MATCH effect refused with apex K♣ still present | Execution-only whole coupled REDEAL/LEFT-X variant requires replayed DL=None/DR=None, positive ordinary RIGHT-empty, unchanged other scene anchors and full remaining/absent tableau proof. It adds no REDEAL input or completion inference. | [native-v15 stock frame/deal/provenance](../tests/fixtures/pyramid-recon/native-v15/README.md); original packet `/home/charlie/tmp/pyramid-error-1792433618738-65018-2`, 41/43 actions verified, 66 gestures ACKed. The final King was not dispatched in that packet. |
| Complete Undo All modal remains after the single OK ACK | One mandatory cooperative second after successful Confirm precedes the first restoration capture. The measured alternate OK is used only after `ledger.confirm_acknowledged`, with all six other modal components. Six observations and at most five further known-stale waits remain; two consecutive original-board proofs are required. STOP/context revoke waiting and later I/O; no second confirmation or unknown retry. | Three original post-confirm frames from `.pyramid-recon-65018-{3,4,5}`; start is `-5/baseline.png`, restored is `-6/latest.png`, copied in [native-v15](../tests/fixtures/pyramid-recon/native-v15/README.md). These separate runs do not establish same-run live restoration. |
| Gold objective-highlighted 7♣ rank at fixed `3|3` refuses | Append one exact full 27×24 rank sample per bank, rank-seven variants 1→2; preserve all suits, caps, gap, masks, ROI, paper/colour and ±1 alignment. | [native-v16 original/provenance](../tests/fixtures/pyramid-recon/native-v16/README.md), `Sevens cleared 0/9`; original SHA-256 `95d9e45b4c588e87970bc9b4cd909cccc7f63a668652c561ad5f38d0f431f7aa`. |

Native-v16 is the current bundled default: 902,743 bytes, SHA-256
`732af82cb2befbb3a420f5ae04ec11dd0b59bd5957a005df17218674bc01244e`.
It inherits the complete native-v15 profile, 893,279 bytes, SHA-256
`c107cec6ab419cdb345447c4dec216b3a3d3f9981c881f2ab38e22b7bf143b49`.
Both complete rollback contracts and frozen asset inventories are documented in
their fixture records. Schema 1 and absent-field schema 2 retain their semantics;
explicit malformed/null/unknown optional fields refuse. Literal File profiles
do not silently migrate or fall back to bundled bytes.

The independent exact-integer Python comparison recognises old/new 28/29→29/29
visible current cards and all 423 retained historical labels unchanged; 21
historical rank-seven witnesses improve. No non-seven winner/competitor changes
in that finite comparison. This is calibration/exposed-positive evidence, not
a production Rust test count or prospective/general-accuracy guarantee.

## Normal-series binding and restart contract

Paired full starting-board identity, normal `Boards: 1 of 3` and `New` footer
facts are bound to the run/context and checked through acquisition/restoration.
An optional execution frame guard checks the series on existing exact-position
gameplay observations, including matched empty-goal frames. Header/footer drift
refuses further input, including after an acknowledged selection. Unmatched
specific Level Up/new-board final observations retain their own classifier.

Typed restart admission requires actual verified final-board BoardClear or
specific paired GameComplete under the caller's bound normal third-board context.
A planned final gesture, ACK or DeliveredUnconfirmed cannot mint that authority.
Cold positively supported Pyramid Level Up uses the existing independent wrapper;
it need not invent the old game's unseen target. A witnessed expected New Game
uses the bounded control policy and non-gameplay veto. Bare menu/stock-overlap
ink does not establish completion. Menus remain distinct from reward/score-skip
evidence; legacy Computer Vision/manual/Single board policy is preserved.
After restart, a fresh actual normal first-board pair is still required before
Recon. Synthetic control-stage fixtures establish transport/order only.

## Verification ledger — required host checks passed

The earlier provisional-115 **1,179-test pass at 18:12:54–18:16:37 UTC on
19 October 2026 preceded all three measured corrections**. Earlier 63 focused
normal/session/execution-guard tests, check, private docs and Clippy likewise
describe the preceding revision. They are retained evidence and do not qualify
native-v16/current adapter code. The current run is recorded separately below.

Retain the preceding focused failures and their bounded corrections:

| Retained focused failure | Correction; no weakened production guard |
| --- | --- |
| E0599, stale `BundledNativeV14` test selector | Use the current `BundledNativeV16` selector. |
| Local stock-tail QMP handshake `WouldBlock` | Encode the labelled synthetic PNG before starting the bounded peer's accept/handshake period. |
| E0308, event receiver type mismatch | Receive the contextual worker-event envelope. |
| E0425, missing event-envelope type | Use `Option<mpsc::Receiver<crate::worker::ContextualWorkerEvent>>` and match `event.event`. |

The stock-tail regression also exposed a fixture setup timing failure during its
bounded local QMP handshake. Its labelled synthetic PNG is now constructed before
starting the waiting peer, rather than consuming the peer's handshake budget.
This changes test setup only; production timeouts, evidence and input policy are
not relaxed. The original failed attempt remains retained; the corrected stock
regression passes within the current suite rather than borrowing the earlier pass.

The post-OK timing correction adds exactly one `self.settle()` after successful
Confirm and before the first restoration capture. It reuses the existing
`OBSERVATION_DELAY = 1s` and cooperative authority/deadline checks. Core and native
adapter regressions assert that the first wait begins with zero restoration captures;
actual STOP/context revocation after the adapter announces `Verifying` must
prevent every restoration capture, further input and publication. The six-observation cap,
two consecutive original-board proofs and 300-second operation deadline remain.
Configured post-OK waits are one mandatory second plus at most five known-stale
seconds; capture/recognition time is additional. These regressions pass in the
current suite. Release, protected-artifact and sealed preservation checks have also passed.

| Current-source requirement | Result |
| --- | --- |
| Focused adapter/executor/core regressions | Passed: 45 tests (stock tail 1, post-confirmation 9, core 26, paired 9); these overlap the full suite |
| Measured vision regressions | Earlier focused native-v15 13 and native-v16 4 passed; also covered by the full suite, not added to its total |
| Locked all-target/all-feature Cargo check | Passed, 7.831 seconds |
| Full locked tests including ignored AF_UNIX cases | **1,199 passed, zero failed/ignored**; 246.965 seconds command / 246.80 seconds test report |
| Strict private rustdoc | Passed with private items and warnings/missing docs denied, 1.317 seconds |
| Warnings-denied Clippy | Passed, 8.282 seconds |
| Scoped formatting | Eight new modules, 0.064 seconds; 14 changed parents in isolated shadows, 1.468 seconds; inherited blank-line exceptions retained |
| Native release build | Passed, 14.594 seconds; owner selected from Cargo JSON |
| Native release and protected headless help | Both passed, exit 0, 0.002 seconds each |
| Protected artifact identity/parity/mode | Passed; exact SHA-256/native parity, 12,883,872 bytes, mode 0555 |
| Sealed frozen-input/index/sibling/artifact preservation | Passed through the audit-only resumption below |
| Final current-documentation/link/preservation audit | Passed; 11 documents, 306 local links, 51 paths |
| Fresh operator acceptance of the three corrected cases | Not performed |
| Complete native Continuous loop | Incomplete; UI disabled |

The exact current toolchain is rustc `1.101.0-nightly (b6eeb93c9 2026-10-18)`,
Cargo `1.101.0-nightly (3960fa69b 2026-10-12)`, native target
`x86_64-unknown-linux-gnu`. Commands, logs, exercised source snapshots, formatting
records and the command-completion seal are retained under
`/home/charlie/tmp/pyramid-continuous-session-validation-wb_57628/runs/0001`.
The current required commands ran from 19:11:24 UTC on 19 October 2026;
`verification.json` records `passed-required-checks`, completed at 19:16:45 UTC.

The initial final-preservation check failed only with
`Reviewed operational bytes changed: .trigger-tree/history.jsonl`.
The original failed `runs/0001/verification.json`, logs, source snapshots and
`commands-complete.json` seal remain retained. Root refreshed only the reviewed
operational telemetry pin, then used `--resume-audit runs/0001`.
`runs/0001/audit-resumptions/0001` passed without repeating Cargo, formatting,
release or help commands. The work-root `verification.json` records this successful
resumption; it does not erase the original failure. Root's final documentation
and preservation audit is a separate remaining check after these edits.

Final documentation/link/preservation audit passed at 19:22:21 UTC on
19 October 2026: **11 documents, 306 local links and 51 review paths**.
The compiler inputs, all three HEAD/index identities, frozen siblings, prior
fixtures and protected 107–115 artifacts remained unchanged. A later docs-only
recheck records these final status edits without repeating Cargo workloads.

## Explicit approved review paths

The final audit resolves the 51 application paths declared in
`/home/charlie/tmp/pyramid-continuous-session-validation-wb_57628/review-paths.json`
to the exact absolute paths below. This is the approved review scope, not a
claim of live acceptance. The xSAR and console arrays are empty: no sibling
changes are approved. The sealed source/input/index/artifact preservation audit
passed; root will separately audit the final documents against this same scope.

```text
/home/charlie/repo/RUST/qmp-qemu-socket/CHANGELOG.md
/home/charlie/repo/RUST/qmp-qemu-socket/Cargo.lock
/home/charlie/repo/RUST/qmp-qemu-socket/Cargo.toml
/home/charlie/repo/RUST/qmp-qemu-socket/README.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Codex-Current-Status.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Live-Test-2.6.115.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Pyramid-Evidence-Index.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Pyramid-Implementation-Schedule.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Verification-2.6.115.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/pyramid-recon-profile.md
/home/charlie/repo/RUST/qmp-qemu-socket/src/app.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/app/pyramid_workflow_tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_progress.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_progress/normal.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_progress/normal/tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_recon.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_recon/paired_tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_recon/vision.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_recon/vision/native_v15_tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_recon/vision/native_v16_tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/worker/computed_route.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/worker/computed_route/series_guard_tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/worker/computed_route/wire_tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/worker/computed_route/wire_tests/stock_empty_tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/worker/post_game.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/worker/post_game/pyramid_independent_tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/worker/pyramid_reconnaissance.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/worker/pyramid_reconnaissance/post_confirmation_tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/worker/pyramid_reconnaissance/starting_guard_tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/worker/pyramid_session.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/worker/pyramid_session/native_tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/worker/pyramid_session/tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v15/README.md
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v15/post-confirm-3.png
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v15/post-confirm-4.png
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v15/post-confirm-5.png
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v15/post-confirm-restored.png
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v15/post-confirm-start.png
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v15/profile.json
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v15/provenance.json
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v15/stock-empty-deal.json
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v15/stock-empty.png
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v16/README.md
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v16/gold-seven.png
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v16/profile.json
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v16/provenance.json
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-standard-progress/README.md
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-standard-progress/footer-new.rgb
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-standard-progress/header-one.rgb
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-standard-progress/normal-one-variant.png
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-standard-progress/provenance.json
```

## Protected artifact — host verified and ready for operator trial

Protected path:
`/home/charlie/tmp/pyramid-continuous-session-validation-wb_57628/release/qmp-qemu-socket-2.6.115`.

- Size: **12,883,872 bytes**; regular protected executable, mode **0555**.
- SHA-256: `7961510c1e3e49339e7728907c537384c976530a89512dbefc3aee35d0b9600e`.
- Observed: **2026-10-19 19:16:08.262994 UTC**.
- Exact parity with the Cargo-JSON-selected native release at
  `/home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket`;
  native and protected headless `--help` both returned zero.

Root's read-only runtime preflight at **2026-10-19 19:16:44 UTC** observed PID
**88003** still running protected **2.6.111** with `--image`, SHA-256
`32638935cae71f240a6c6e733168dbcd1217bad916af0bd3fa10c26aac97b55b`.
It did not observe a replacement with 114 or 115. The image directory was empty
at that instant only; this is not permission to discard later evidence.

No installation, process replacement, guest operation, commit, push or promotion
has been performed. The future operator command and remaining final-documentation
qualification condition are in [Live-Test-2.6.115.md](Live-Test-2.6.115.md).
Historical [114 verification](Verification-2.6.114.md) remains unchanged.

## Prepared promotion text — review only

Proposed title:

```text
fix(pyramid): harden reconnaissance and native session guards
```

Proposed body, for review:

```text
Add coupled depleted-stock execution evidence, one measured gold rank-seven
variant per reader bank, and phase-bound recognition of the complete post-OK
modal. Settle one cooperative second after the sole confirmation ACK before
restoration captures, retaining bounded observations, STOP/context and two
consecutive original-board proofs.

Bind normal first-board identity/count through acquisition and matched execution
frames. Reuse bounded independent restart controls only from supported terminal
or actual bound final-board evidence. Keep native Continuous disabled while
normal second/third ordinal evidence and live loop acceptance remain missing.

Current run 0001: 1,199 tests passed, zero failed/ignored; locked check, strict
private docs, Clippy, scoped formatting, native release and both headless help
checks passed. Protected 115 matches the native release; sealed preservation
passed by audit-only resumption after a reviewed telemetry-pin refusal, without
repeating Cargo. Final documentation audit and fresh operator acceptance remain
separate. xSAR, console and old protected artifacts remain frozen.
```

The explicit 51-path review list above is the complete approved file scope.
This prepared text performs no commit or push. Charlie commits/pushes unless he
delegates them; installation, promotion and registry publication remain separate.

## Code Analysis

The two optional schema-2 roles are explicit, bounded and fail closed. The
execution role requires a coupled whole-state proof rather than independent OR
patterns; the modal role requires already acknowledged confirmation and conveys
observation only. Both preserve checked geometry, full evidence and unknown/
absence separation. The rank append retains all frozen matching policy and must
pass damaged, blank, colour and competing-label regressions as well as positives.
STOP/context and uncertain-input handling still revoke further input. Host checks,
retained original evidence and human acceptance remain separate requirements.
