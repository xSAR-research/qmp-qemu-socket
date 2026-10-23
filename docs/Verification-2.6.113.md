# Verification — app 2.6.113 native-v14 aligned King diamond sample

Status: **required host checks, protected artifact and audit-only preservation
passed; final root documentation/preservation audit passed; live acceptance
pending**. All Cargo/help commands exited zero in attempt 0001. Its final
preservation check stopped only on automatic trigger-tree telemetry. Audit-only
resumption validated the sealed original command evidence and frozen inputs,
refreshed the two already declared operational hashes and passed without any
workload/help repeat. App 2.6.113 earns one correction build/no additional minor.
The original failed audit record remains retained; no test failed. No guest/GUI,
input, installation, process replacement or promotion is authorised by this
record.

## Correction contract

The app-only native-v14 correction appends **one measured full 20×16 RGB
diamond sample to both reader banks, 12→13**, and selects bundled native-v14.
The source is fixed `5|3` K♦ from the original failed RECON capture. Its nominal
shape already fits at 69,137 ppm, but one strong-black face-art pixel makes
nominal colour nonexclusive; the best eligible old witness is 130,980 ppm,
above the unchanged 120,000 ppm cap. The measured clean crop uses native
`[896,354,20,16]`, the existing `(+1,-1)` alignment. This is a bounded calibration
addition for an exposed case, not a general artwork or colour-policy repair.

Schema 2 `paper_ink_l1_v2`, rank/suit caps 220,000/120,000 ppm, different-label
gap 80,000 ppm, full masks, nine ±1 shifts, paper support and exclusive colour
remain unchanged. Native-v13, its templates and explicit custom File behaviour
are retained. The host-verified 2.6.112 stock-only Undo All/OK pacing correction
is preserved; no wait, search, input-authority or other-game change is added.

The prior nominal witness is diamonds variant1, 1623/23475 (69,137 ppm), but
red90/black1. Actual pixel `(910,370)` is RGB `[100,100,96]` from King artwork.
All three nominal shifts and three downward shifts are colour-ineligible;
40 existing templates × six shifts explain 240 colour refusals. The best
eligible unchanged-v13 witness is variant10 at `(+1,-1)`, 3080/23515
(130,980 ppm), versus spades 401,501 ppm. Rank K passes 5310/44914
(118,225 ppm). Exact checked ratios, not floored diagnostic ppm, decide.

The exact new sample is native `[896,354,20,16]`, 960 untransformed RGB bytes,
full 320-pixel mask, tolerance zero. Its expected shift `(+1,-1)` is already
inside the existing window and establishes red89/black0 in this captured case.
Simply appending the nominal crop would not fix that colour refusal. No global
colour relaxation, cap increase, cropped-away row, identity inference or wider
alignment is introduced. All retained v13 entries remain literal; removing both
new entries and restoring the profile ID must reproduce the complete v13 JSON.

Native-v14 profile is **832,593 bytes**, SHA-256
`2e1c1cc7135dc8f5310396bba52473ba53c6ad4eb4b99b3b0b1631eac57ea9f9`.
The appended raw RGB SHA-256 is
`cb56876b7359a18432c4acd237931090aa341372298b009d8bceaac6d2cfdbed`.
Production regressions and the final preservation audit verified these measured
assets against the original source bytes and unchanged reader policy.

## Original evidence and boundaries

Original `/home/charlie/tmp/.pyramid-recon-45759-3/baseline.png` and `latest.png`
are byte-identical, 3,771,011 bytes, SHA-256
`7263c1ccc78dd7eed213217bbbd739c9128adb7375fb0465a2263300b856d857`.
INCOMPLETE.json SHA-256 is
`6e86160486800e88dbbb863cf77ec6a02d981c247d147f795465701aeea7ed05`.
The diagnostic records one capture, zero D/Undo All/confirmation attempts and
acknowledgements, no uncertain delivery, restoration false and publication not
attempted. It identifies bundled native-v13. No stock/deal/receipt is inferred.

The later operator screenshot
`/home/charlie/Pictures/Screenshots/qmp-qemu-socket 261020 012031 SUIT-Fail.png`
is pixel-identical corroboration; it is not substituted for original acquisition
PNG/report bytes in the fixture. The new capture is an exposed calibration and
regression case, not an untouched holdout or general accuracy measurement.
[Fixture provenance](../tests/fixtures/pyramid-recon/native-v14/README.md)
records exact copies, sample and profile hashes.

## Host checks and immutable attempt evidence

Work root: `/home/charlie/tmp/pyramid-diamond-v14-validation-_r1fz2yn`.
`baseline.json` and the closed `review-paths.json` define preservation/scope.
The root verifier owns commands, toolchain, first-failure retention, compiled
input hashes, native/protected artifact hashes and runtime inspection. No
verification result is inferred from the Python diagnostic mirror.

| Check | Current result |
| --- | --- |
| Exact profile delta, source bytes/crop and rollback | Passed production regressions and audit-only preservation |
| Focused production native-v14 recognition/refusal regressions | Passed: 7/0/0, 1,093 filtered; compile 13.83 s/tests 80.22 s; `focused-v14.log`, `focused-v14-result.json` |
| Repository-selected nightly and explicit native target | Rust 1.101.0-nightly `b6eeb93c9` (2026-10-18); Cargo 1.101.0-nightly `3960fa69b` (2026-10-12); `x86_64-unknown-linux-gnu` |
| Scoped rustfmt / inherited spacing inspection | Passed: new module 0.033 s; five-parent isolated shadow 0.816 s; inherited blank-only exceptions retained, nonempty lines match |
| Locked/offline check, all targets/features | Passed, 7.379 s; `runs/0001/app-check.log` |
| Full tests including ignored | Passed: 1,100/0/0; harness 188.38 s/command 188.557 s; `runs/0001/app-tests.log` |
| Strict private rustdoc | Passed, 1.117 s; `runs/0001/app-rustdoc.log` |
| All-target/all-feature Clippy, warnings denied | Passed, 6.979 s; `runs/0001/app-clippy.log` |
| Native release/Cargo artifact selection/headless help | Passed, release 13.497 s/help exit 0, 0.002 s; Cargo JSON selected native artifact |
| Protected copy parity/mode/hash/headless help | Passed, 12,736,432 bytes/mode 0555/native parity/help exit 0, 0.004 s; `stable-artifact.json` |
| Read-only process/image preflight | 2026-10-19T15:44:55.058178+00:00: PID 45759 protected 2.6.111 still running; images 0 files/bytes then only |
| Final documentation/link/preservation audit | Passed: nine documents, 221 local links, 21 review paths |

The required operations use the selected nightly and explicit native target:

| Operation | Exact command / environment |
| --- | --- |
| Check | `cargo +nightly check --locked --offline --target x86_64-unknown-linux-gnu --all-targets --all-features` |
| Full tests | `cargo +nightly test --locked --offline --target x86_64-unknown-linux-gnu --all-features -- --include-ignored` |
| Private docs | `cargo +nightly doc --locked --offline --target x86_64-unknown-linux-gnu --all-features --no-deps --document-private-items`; `RUSTDOCFLAGS=-D warnings -D missing_docs` |
| Clippy | `cargo +nightly clippy --locked --offline --target x86_64-unknown-linux-gnu --all-targets --all-features -- -D warnings` |
| Release | `cargo +nightly build --locked --offline --target x86_64-unknown-linux-gnu --all-features --release --message-format=json` |
| Help | Selected native/protected executable `--help`, headless environment |

The native-v14 filter passed seven tests, zero failed or ignored, with 1,093
filtered out; compilation took 13.83 s and tests 80.22 s. The root retains its
actual command and result in `focused-v14-result.json` and `focused-v14.log`.
All required Cargo/protected artifact and audit-only preservation checks passed.
Runtime preflight is recorded below; final root documentation audit passed.
Focused counts overlap the required full suite. Retained historical corpus
observations are finite exposed/development checks; repeated matching does not
turn them into independent frames or prospectively unseen boards. Unchanged
xSAR/console suites are retained evidence, not newly run for this app-only change.

### Attempt 0001 preservation stop

All required Cargo and executable-help commands passed. The final preservation
audit stopped with the exact error:

```text
Reviewed operational bytes changed: .trigger-tree/history.jsonl
```

The root retains `runs/0001/verification.json`, command logs and compiled input
hashes. Automatic read telemetry is not correction source. The root preserved the
original failed record, validated the immutable `commands-complete.json` seal,
original command logs and all frozen compiled inputs. Audit-only resumption at
`runs/0001/audit-resumptions/0001` refreshed only the two already declared
trigger-tree operational hashes and passed. `verification.json` now records
`passed-required-checks`; `audit-only-operational-refresh.json` and
`runtime-receipts-before-audit-0001.json` retain exact operational evidence.
No Cargo, formatting, test, build or help command was repeated; source, scope
and compiled inputs were unchanged. The separate final root documentation/preservation
audit passed for nine documents, 221 local links and 21 review paths. No full test failure occurred.

## Artifact and later operator trial

The verified protected path is
`/home/charlie/tmp/pyramid-diamond-v14-validation-_r1fz2yn/release/qmp-qemu-socket-2.6.113`.
It is byte-identical to the Cargo-selected native release,
`/home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket`,
12,736,432 bytes, mode 0555, SHA-256
`a8d8140677668f694d883f1fa3c9cadc4b6385a34cc1c244533dec790d6a4930`.
Native/protected headless help exited zero, respectively 0.002/0.004 s;
`stable-artifact.json` records observation 2026-10-19T15:44:24.894774+00:00.
The final root documentation/preservation audit passed for nine documents,
221 local links and 21 review paths.
Read-only preflight at **2026-10-19T15:44:55.058178+00:00** observed PID **45759**
still running protected **2.6.111** with `--image`, executable
`/home/charlie/tmp/pyramid-diamond-v13-validation-ugnvhgv9/release/qmp-qemu-socket-2.6.111`,
SHA-256 `32638935cae71f240a6c6e733168dbcd1217bad916af0bd3fa10c26aac97b55b`.
Images were zero files/bytes at that instant only. This does not qualify the
new artifact or establish later process/image state. No controller was stopped,
replaced or launched. Preserve current logs and GAME WIN images;
startup clears `/home/charlie/tmp/images`. Deliberately close an old controller
before a later operator launch; the agent neither stops nor replaces one here.

[Live test](Live-Test-2.6.113.md) requires a fresh untouched supported board,
complete same-session acquisition, proved original restoration and exact current
published bytes. A retained frame or old export never supplies live authority.

## Goal and remaining work

The larger goal remains **ACTIVE and incomplete**. Native Continuous is still
disabled pending implemented/validated progress, objective, terminal/redeal and
restart handling. Charlie reports Single board works; that report is separate
from a new 2.6.113 operator trial. M10 remains Awaiting verification, M11 Planned,
M12 In progress and M13 Planned. The intermittent idle-CPU report has not recurred
under SysProf and remains unresolved; this correction contains no CPU fix.
xSAR 0.6.102, console 0.8.4, indices and protected stable 107–112 remain frozen.
No guest/GUI/input, installation, process replacement or promotion is performed.

## Complete correction review scope and proposed promotion

The root's closed review-path list contains these **21 absolute paths**. It
defines the correction delta against the retained task baseline, preserving
accumulated earlier work and historical records. Both sibling review lists
remain empty; operational memory/telemetry is excluded.

```text
/home/charlie/repo/RUST/qmp-qemu-socket/Cargo.toml
/home/charlie/repo/RUST/qmp-qemu-socket/Cargo.lock
/home/charlie/repo/RUST/qmp-qemu-socket/src/app.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/app/pyramid_workflow_tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_recon.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_recon/vision.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/worker/pyramid_reconnaissance.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_recon/vision/native_v14_tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/README.md
/home/charlie/repo/RUST/qmp-qemu-socket/CHANGELOG.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Codex-Current-Status.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Pyramid-Implementation-Schedule.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Pyramid-Evidence-Index.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/pyramid-recon-profile.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Verification-2.6.113.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Live-Test-2.6.113.md
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v14/README.md
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v14/profile.json
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v14/baseline.png
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v14/INCOMPLETE.json
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v14/provenance.json
```
No staging, commit, push or publication occurs.

Proposed commit title: `fix(pyramid): recognise the retained aligned King diamond variant`

Proposed body:

- Append one untransformed full-ROI diamond RGB sample to both native-v14 banks.
- Preserve the graded policy, full masks, exclusive colour and existing alignment.
- Cover the original zero-input RECON refusal and retain preceding regressions.
- Preserve stock-only RECON reset pacing and unrelated authority/gameplay behaviour.

## Code Analysis

The change adds calibration evidence and updates the bundled default without
adding unsafe Rust, dependencies, new input authority or a matching-policy
fallback. Bounded masks, variant counts, parser size, scene/modal checks,
co-located paper/colour, shape/error separation and exact checked arithmetic
remain. A new measured appearance cannot prove all unseen face-art phases;
unsupported or ambiguous observations continue to refuse. Fresh guest effects
and restoration require later live acceptance.
