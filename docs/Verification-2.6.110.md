# Verification — app 2.6.110 native-v12 reader robustness

Status: **host verified; required app checks passed in complete attempt 0002**.
Prepared 19 October 2026. The authorised reader correction is complete. The larger
objective is blocked on missing native standard-three-board progress/target evidence.
Exactly one app correction build/no minor is earned after full pass;
xSAR 0.6.102 and console 0.8.4 all files, stable 107/108/109 stay frozen.
No unchanged shared suite rerun, guest effect or completed larger goal is claimed.

## Approved reader contract

Native-v12 selects strict schema 2 `paper_ink_l1_v2` with unchanged native-v11
RGB banks. Each actual/template ROI independently requires nonzero raw-paper
support of at least area/3 (all RGB channels >=180). Its paper reference `p` is
the lower median of those pixels' minRGB values, index `(count-1)/2`, 180..255;
no default reference is allowed. Every ROI pixel contributes
`floor(255*max(p-minRGB,0)/p)` ink; pixels brighter than paper contribute zero.
Error remains `sum(abs(actual-template))/sum(max(actual,template))`. Inclusive
rank 220,000/suit 120,000 ppm maxima and different-label gap 80,000 ppm/scale 1,000,000
are unchanged. Checked exact ratios decide; floored score ppm is diagnostic
only. Both absolute quality and separation are required; nearest label alone
never suffices. Exclusive strong colour and paper are co-located with that
label's winning one of nine ±1 shifts; rank/suit colours must agree/match suit.
No RGB sample, deck inference or input authority is added.

Schema 1 keeps its exact previous XOR 24/8 matching and strict keys. Literal
custom File profiles receive no automatic migration or fallback. Native
1920×1080 geometry, scene/dialog, positive-empty, complete-inventory,
restoration, receipt/context/STOP and input-effect contracts remain. Manual/Fast
and Single board keep their existing workflow/timing/search budgets. **Native
Continuous remains disabled/unavailable** without positive standard-three target
and progress evidence; Daily Challenge automation is not added. The resumed
larger objective is blocked on native standard-three-board evidence; M10 Awaiting verification/M11 Planned.

Profile `profile.json`: 817,977 bytes/SHA-256
`f8f1bc8b8a7e28d0ad3433494c47096f5ca5e0b546275912d845adf6afdd0813`. The policy record SHA-256 is
`5343651cea940c25add3ee79653b2c9c9da257b30d9874143f6b400cf0d81236`. The native-v11 parent profile remains
817,701 bytes/SHA-256
`7269769d0c1373edd431e7606f6cdad3ed1239dc5d70839257b5ca89ed408124`.
Only schema/ID/explicit matching policy select the new semantics; RGB rank/suit
banks are not extended. Legacy schema 1 profiles remain separately parsed.
Policy parser accepts only the frozen integer values; arbitrary relaxation,
mixed legacy/new fields, missing/extra/wrong-typed keys must refuse.

## Calibration, evaluation and original failures

The initial raw-ink policy used 104 historical+29 ten-failure examples and failed
its first Rust filter: 8 passed/4 failed/0 ignored. The independently labelled
fifth K♦ also failed its initial prospective test at 123,012 ppm versus 120,000.
That exact failed policy and outcome are preserved; the fifth is now
`exposed_regression_fifth`, with no renewed prospective/untouched claim.
The revised paper-relative policy was frozen at 2026-10-19T13:02:56.014505+00:00
using 250 development+29 ten-regression observations; thresholds/gap and RGB
banks remain unchanged. Corpus totals stay 423 observations/40 sources/14 positions:
250 development, 29 ten-regression, 115 prior-exposed validation and 29 exposed
fifth observations. Capture/deal lineage stays whole; daily/v8 share one lineage.
The 115 validation observations retain prior calibration/regression exposure;
they are not untouched evidence. Focused graded/native/corpus results passed as recorded below; full checks passed as recorded below. Counts
are observations, not unique frames, decks or a general accuracy guarantee.

Both runtime reader banks must evaluate all 423 observations; counts and results
must remain separated by corpus role. No pass for 846 bank/observation pairs is
inferred from their existence. The selected position retains 26 cards/two empty
original tableau slots plus visible pile tops; locations are never compacted.
Full/partial positions, positive empty evidence and original PNG hashes are
checked independently of card-level recognition. Current focused/full Rust results passed; the first policy's prospective failure is preserved separately.

The ten-failure and fifth-failure diagnostics remain incomplete, with zero
input attempts and no published complete deal/restored receipt. Fifth source
SHA-256 `81519aaa7ae9f8d47136af0868834368ecf40f49ff1fd546e929ff37dcd2b2c3`
contains independently labelled K♦ at zero-based slot 9/fixed 4|4. Schema1/v11
refuses it: nominal suit colour includes 3 strong black pixels; at the upward
one-pixel shift colour is clean but best shape XOR 10 exceeds unchanged cap 8.
That diagnosis does not justify weakening colour or reclassifying the original
failure as success. Its initial raw-policy prospective refusal is retained; current evaluation is exposed regression, with no renewed prospective claim.

See [corpus/provenance](../tests/fixtures/pyramid-recon/native-v12/README.md)
and [profile contract](pyramid-recon-profile.md). Saved visible labels establish
no unseen stock, complete acquisition, restoration, current authority or guest
acceptance. No new guest capture/input is requested by this host correction.

## Retained first failures and revised candidate

The first focused compile failed E0603 because PixelRect was imported through
its private re-export; the test uses the owning parent geometry path. Original
source/log evidence remains in work-root `first-focused-failure/`.

The initial raw-ink policy (`initial-policy.json`, 1,194 bytes/SHA-256
`d6c8a647946ae2898cb27068a88c5314d5f2550f80048fa7976b47312871b3e1`)
was rejected, never released. Its native filter was **8 passed/4 failed/0 ignored**.
Genuine development 2♥ failed with rank 268,860 ppm/suit 144,337 ppm. Initial
prospective fifth K♦ at fixed 4|4 accepted rank K but refused diamonds: exact
error 3010/24469 (diagnostic 123,012 ppm), above 120,000 suit cap at shift (0,-1).
Corpus assertions stop at first mismatch; no completed 846-decision accuracy
result exists for that policy. `initial-policy-results.json` SHA-256
`4a59e01e330cf31d60721468a5d2c192ae5c658e26d62a00b0f22cff0a4dc5ed`
and work-root `first-policy-failure/` preserve the original result/log identity.

The revision changes paper-relative strength, not 220000/120000/80000 bounds,
RGB banks or colour/scene authority. Current policy `paper_ink_l1_v2` is 1,730
bytes/SHA-256 `5343651cea940c25add3ee79653b2c9c9da257b30d9874143f6b400cf0d81236`;
profile 817,977 bytes/SHA-256
`f8f1bc8b8a7e28d0ad3433494c47096f5ca5e0b546275912d845adf6afdd0813`;
corpus 311,406 bytes/SHA-256
`51db58fbb292d262d43a81ce1c2491b1c99bae7e1c5effe4c26767f696c61eaa`.
Independent Python recognition reports all 423 observations correct; this is
comparison evidence, **not** a Rust/full/live pass. Current graded-unit filter
**14 passed**; the 12 native tests also passed as recorded below. The complete
required host scope subsequently passed as recorded below. The fifth is exposed regression for this revision; no renewed
prospective success is claimed.

Focused production tests now pass: **14 graded unit tests, 0 failed/ignored**
(compile 8.26 s,test 0.00 s), and **12 native-v12 tests, 0 failed/ignored**,
1,066 filtered (compile 0.09 s,test 60.10 s). They verify **846 bank/observation
decisions** across 423 directly labelled cards, 14 full/partial positions and 40
scene checks (36 boards/4 modals). These are finite exposed/development/prior-
exposed validation regressions, not a new prospective result. Initial raw-policy
8/4 failure and first prospective K♦ refusal remain retained. **Full 1,078/0/0 app tests and required host checks passed in attempt 0002;
protected stable 110/native parity/help passed. Live acceptance is still pending.**

## Required host evidence — passed

Work root: `/home/charlie/tmp/pyramid-reader-v12-validation-8jiy32rw`. `verification.json` reports
`passed-required-checks`, attempt `runs/0002`, from 2026-10-19T13:12:28.724293+00:00
to 2026-10-19T13:16:06.116947+00:00. Baseline hashes and the 29 literal paths define this
correction; accumulated older dirty M10 changes are not adopted into scope.
Exact argv/cwd/environment/timings/logs remain in `runs/0002/commands.json`.

Actual compiler is **rustc 1.101.0-nightly (b6eeb93c9 2026-10-18)**, commit
`b6eeb93c992027a03e1fd120b6652546396b142c`, LLVM 23.1.4; Cargo
**1.101.0-nightly (3960fa69b 2026-10-12)**; native host/target
`x86_64-unknown-linux-gnu`. The actual compiler is newer than the previous 109
record; no older compiler hash is reused as this revision's evidence.
All Cargo commands used explicit `+nightly --locked --offline` and native target.
Check/Clippy selected all targets/features; full tests all features and ignored
cases; private docs all features/no dependencies with
`RUSTDOCFLAGS="-D warnings -D missing_docs"`. Release used package defaults
and Cargo JSON-selected executable, followed by native and stable headless help.

| Check | Actual result |
| --- | --- |
| All-target/all-feature locked check | Passed, 1.769 s |
| Full app tests including ignored | **1,078 passed, 0 failed, 0 ignored**, 166.532 s; test report 163.59 s |
| Strict private rustdoc | Passed, 1.016 s |
| Clippy, warnings denied | Passed, 6.627 s |
| New-module scoped formatter | 3 modules passed, 0.032 s |
| Existing-parent formatter shadows | 5 modules passed, 0.766 s; identical nonempty lines |
| Default-feature release/Cargo JSON | Passed, 37.486 s |
| Native executable headless help | Passed, exit 0,0.002 s |
| Protected stable executable help | Passed, exit 0,0.004 s |
| All three HEADs/indices/frozen xSAR and console files | Passed |
| Stable 107/108/109 and unrelated source/calibration/evidence | Preserved |
| Executable/profile/corpus/scoring inputs | Verified exact through required checks; separate provenance-pointer amendment below |
| Final documentation/link/scope audit | Passed: 9 documents, 177 local links, 29 review paths |
| Fresh Beast/operator acceptance | Pending; no agent GUI/guest/input/install/promotion |

Formatter-shadow blank-only exceptions preserve standing spacing in
`src/app.rs`, `src/pyramid_recon.rs`, `src/pyramid_recon/vision.rs` and
`src/worker/pyramid_reconnaissance.rs`; there was no blanket formatting.
Unchanged xSAR 0.6.102/console 0.8.4 suites were not rerun or reattributed.
The verifier records six exact automatic operational-telemetry exceptions:
four local activation receipts and the two `.trigger-tree` history/session
paths. These were classified and left untouched; no broad exemption is implied.

The earlier full attempt 0001 failed **1,076 passed/2 failed/0 ignored**, report
164.49 s, retained in `runs/0001` and `first-failure.json`. Both failures were
legacy native-v8 exact-error expectations at tableau slot 17: schema 2 details
had leaked into schema 1 messages. The corrected code retains legacy
`execution tableau identity mismatch at slot 17` exactly and adds observed/
policy details only for graded schema 2. This preserves the historical API/error
contract and recognition authority; no signature was weakened or assertion
skipped. The complete required scope then passed in attempt 0002. First E0603,
raw-policy 8/4 and initial prospective fifth failures above remain retained.

## Protected artifact and runtime — host verified

Native Cargo-selected executable: `/home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket`.
SHA-256 `dcdbda38b1d16e9bd99b2608f01a8a19b3c5f66fc9fde7f6cbece7a2f832058d`.
Protected separate candidate: `/home/charlie/tmp/pyramid-reader-v12-validation-8jiy32rw/release/qmp-qemu-socket-2.6.110`,
**12,722,096 bytes, mode 0555**, identical SHA-256/headless help 0,
created at 2026-10-19T13:16:05.745874+00:00. Existing differing destinations are
refused without overwrite. Stable 107/108/109 remain exact; a successful build
or copy does not replace a running process.

Read-only preflight at 2026-10-19T13:16:29.450099+00:00 observed **no application-controller
process** and `/home/charlie/tmp/images` had zero files/bytes then. These are
instant-only observations, not a promise of later emptiness or running identity.
No QMP connection, GUI launch, process replacement, guest input, installation,
staging/commit/push or publication was performed. The [live brief](Live-Test-2.6.110.md)
provides the exact protected launch, startup-image-clearing warning and fresh
operator acceptance. Native Continuous remains disabled/unavailable and the
larger objective is blocked on native standard-three-board evidence; M10 live Awaiting verification,
M11 Planned.

## Separately validated provenance-pointer amendment

After the required pass, root identified a non-executable sidecar pointer:
`provenance.json.initial_policy.file` said `frozen-policy.json` while its stored
SHA-256 names the preserved initial raw policy. The correct file is
`initial-policy.json`. Root owns the one-field metadata correction and separate
validation; this documentation does not edit JSON. Tested sidecar SHA-256 is
`0df224e4a3a1235f33010d79172e4dbe4ebf2275bca7daf96c1b049f6570f886`.
Amended sidecar SHA-256 is `318c8c4605795308edfa86a18070dc643ccd745c5a65127de8d53d992a560f2b`.
`post-verification-metadata-amendment.json` records validated metadata-only
amendment at 2026-10-19T13:19:46.988683+00:00: exactly `initial_policy.file` changes
`frozen-policy.json`→`initial-policy.json`; all other JSON values are identical.
Before/after copies are retained under `post-verification-metadata/`; both
initial/final policy references now resolve to their correct stored hashes.

The original successful verification/snapshots remain unchanged. No executable,
Rust source, profile, corpus, RGB templates, policy, tested PNG, original report,
threshold or scoring result changed; no extra Rust rebuild is asserted or needed
for this documentation pointer. Final audit explicitly reconciled this one
sidecar exception, validated both policy references, and checked every
compiled/exercised reader input and artifact exactly. Do not interpret the full pass as proving
that every fixture-metadata byte remained unchanged after the pass. Final
root audit passed: 9 documents, 177 local links and 29 review paths. Its exact
source/data comparison permits only this documented metadata pointer.

Proposed review-only Conventional Commit title/body:

```text
fix(pyramid): add a versioned graded glyph reader

Select fixed schema-2 paper_ink_l1_v2 matching with exact quality and label-gap bounds.
Keep native-v11 RGB banks and legacy schema-1 XOR/custom-file semantics intact.
Require co-located paper and exclusive colour at each winning shift.
Retain original failures and label lineage/development/exposed roles and the original prospective failure.
Record actual host/corpus-evaluation and stable-artifact evidence after passing.
Preserve siblings/delivered binaries and all live input/receipt/STOP authority.
Native Continuous and the larger objective remain unfinished.
```

## Code Analysis

All ROI pixels, including negative space, participate; zero mass, unknown shape,
insufficient paper/colour, too-small distinct-label gap and contradictions refuse.
ROI area is bounded 4096 and arithmetic uses checked u64 products. Exact cross-
products, not rounded ppm, enforce inclusive acceptance and separation. One
label's best eligible witness does not borrow paper/colour from another shift.
Finite measured corpora do not prove universal false-acceptance performance;
unsupported artwork/scene must still refuse. Diagnostics explain measured best/
competitor scores and shifts without creating identity/input authority. Reader
changes introduce no QMP action, deck inference, cleanup input or retry/resume.

## Closed correction review paths

These 29 literal paths are the current correction delta, not the complete dirty
M10 promotion set. No staging/commit/push authority is implied.

```text
/home/charlie/repo/RUST/qmp-qemu-socket/CHANGELOG.md
/home/charlie/repo/RUST/qmp-qemu-socket/Cargo.lock
/home/charlie/repo/RUST/qmp-qemu-socket/Cargo.toml
/home/charlie/repo/RUST/qmp-qemu-socket/README.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Codex-Current-Status.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Live-Test-2.6.110.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Pyramid-Evidence-Index.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Pyramid-Implementation-Schedule.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/Verification-2.6.110.md
/home/charlie/repo/RUST/qmp-qemu-socket/docs/pyramid-recon-profile.md
/home/charlie/repo/RUST/qmp-qemu-socket/src/app.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/app/pyramid_workflow_tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_recon.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_recon/vision.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_recon/vision/graded_glyph.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_recon/vision/graded_glyph_tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_recon/vision/native_v12_tests.rs
/home/charlie/repo/RUST/qmp-qemu-socket/src/worker/pyramid_reconnaissance.rs
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v12/README.md
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v12/corpus.json
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v12/fifth-failure-INCOMPLETE.json
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v12/fifth-failure-baseline.png
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v12/frozen-policy.json
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v12/initial-policy-results.json
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v12/initial-policy.json
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v12/profile.json
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v12/provenance.json
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v12/ten-failure-INCOMPLETE.json
/home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recon/native-v12/ten-failure-baseline.png
```

## First full verification failure retained

Attempt `runs/0001` completed 1,076 passed, 2 failed and 0 ignored in the full app suite. Both failures were exact legacy schema-1 execution-error assertions in `native_v8_tests`: richer unconditional mismatch diagnostics changed the prior message. All recognition/corpus tests passed in that run. The correction restores the exact legacy mismatch string and confines observed-identity/graded-score detail to schema 2. No matching thresholds, RGB templates or tests changed. `first-full-failure/` retains the original verification/source record; `runs/0001/` retains the complete logs and input hashes. The complete required rerun in `runs/0002` passed all checks: 1,078 app tests, zero failed/ignored.
