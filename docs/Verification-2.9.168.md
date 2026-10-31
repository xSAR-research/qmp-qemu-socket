# Verification — qmp-qemu-socket 2.9.168

2.9.168 is host qualified with the modified xSAR 0.9.1 path dependency.
All 1,669 controller tests and 303 full xSAR tests pass. Controller format,
locked check, strict private rustdoc, Clippy and release pass. xSAR's default
14 tests, seven individual feature checks, all-feature checks, strict docs,
Clippy, release and two new focused Miri checks pass. The qualified xSAR working tree retains 0.9.1; its added comparator remains an unreleased path-dependency change.

## Exact comparison contract

A pixel counts only when the absolute R, G and B differences at the same
coordinate are ALL strictly greater than the selected tolerance (default 13); alpha is ignored. An occurrence
requires strictly more than the selected eligible-pixel coverage (default 10.0 percent). For the existing
31 by 49 corner this means at least 152 of 1,519 pixels. There is no mean gate.
LE packing remains `u32::from_le_bytes([r,g,b,0])`. The generic comparator uses
safe `abs_diff` and the minimum of the three deltas, with overflow-free integer
percentage arithmetic and bounded batches. Application coverage uses exact
tenths-of-a-percent allowance arithmetic; count aggregation always scans both regions fully. Early positive counts are prefix
counts and lower bounds; refusal counts cover the complete eligible region.

## Live PARAMS settings

PARAMS remains accessible during active Pyramid runs, including ShortestPath.
RGB tolerance accepts 0..255; changed coverage accepts 0.0..100.0 percent in
0.1 percent steps. Both strict comparisons remain in force. One validated pair
is replaced under a short mutex, without changing input authority or STOP.
Each computed action and continuation D freezes a pair before its first gesture;
guided actions freeze before input; Recon freezes at the first D of its batch
and retains it across the second D and stale observations. REDEAL refreshes at
its explicit boundary before its exhausted-source observation/validation, then keeps the same
pair through classification, appearance and click checks. Long-lived REDEAL
pixels remain independent of policy. In-flight edits apply to the next boundary.
Committed removal history is never reinterpreted. Refusal diagnostics describe
joint-change and complementary pixel percentages; the complement is not an
exact-pixel similarity score. Prefix counts are labelled as lower bounds.

## Action coverage

Computed tableau removals keep the occupied pre-action corner, recognised-card
veto, two fresh quiet after-observations and original-slot history. Computed
stock-only KING/MATCH, MOVE/RECYCLE and stock continuation also require RGB
occurrence on the pile corners whose expected identities change. Full scene,
remaining-card and exact pile identities remain mandatory. Ordinary position
and frame-guard checks precede removal-history commitment; the later bound
challenge-counter check still precedes progression and new source authority.

Native Recon keeps one original stock baseline across paired D and stale
observations, with no intermediate capture. Recognised stock advancement must
also satisfy the joint RGB rule. UndoAll/Confirm and pause revoke these pixel
baselines; their specific dialog/restored-arrangement proofs remain authoritative.
Guided/Solver pile comparisons preserve the inset and union of cursor exclusions,
the unique highlighted partner rules and semantic terminal exceptions. REDEAL
appearance uses the shared predicate, while one acknowledged click still hands
off to fresh Recon. Challenge completion, terminal/final-apex and restoration
proofs remain explicit semantic exceptions. Colour change alone grants no input.

## Evidence and limits

The retained standard-green and both Coral cases pass their complete expected
positions offline at the approved default13/10 policy. In the latest saved
pairs, Coral's two corners measure 353 and 362 of 1,519 jointly changed pixels;
green measures 367. The older Coral partner measures 318. Explicit frozen26
controls retain the original 147/1,519 and 127/1,519 refusals; no glyph/scene
recognition or quiet-settling threshold was relaxed. The synthetic stock-only
MATCH control also preserves its exact frozen26 refusal (115/1,519), and KING
passes despite in-flight edits. Native tests confirm action-pair snapshots,
next-action refresh, first/second-D batch retention and next-batch refresh.

Stored first/final images are not the immediate runtime pre-action source.
Two repeated retained-image serials test settling mechanics, not a fresh live
settling interval. The native paired-D adapter test is synthetic and does not
prove complete guest Recon. Fresh Coral/green and Xbox, Wild West, Classic Blue,
Cream/Retro acceptance remains pending. Normal Continuous retraction and the
acknowledged-D/no-advancement investigation remain open; no timing/focus fix is
claimed. No guest input, new capture, install, staging, commit, push or publication
was performed. Preserve GAME WIN images before controller startup and leave
any active original session log in place.

## Build configuration

Both repositories now have `.cargo/config.toml`: x86_64 Linux uses
`target-cpu=native`; AArch64 Linux uses `cortex-a76` and the requested
`+neon,+dotprod,+fp16,+crc,+aes,+lse,+rcpc`. These host gates used the normal
project configuration. AArch64 feature probes are compiler evidence only;
no Pi execution or cross-build is claimed. Native host binaries require the
compiling host's ISA. Global Cargo config and the Windows server were unchanged.
The guest-side ROI/comparison proposal remains only a Possible futures item.
Charlie's updated controller Cargo.toml profiles are preserved byte-for-byte
except the approved package version. Release uses opt-level3, fat LTO, one
codegen unit, panic=abort and symbol stripping; development uses opt-level1
with dependency overrides at3. Target-cpu=native allows code for this Ryzen
host, and fat LTO permits cross-crate optimisation; this is not a measured
speedup or guarantee of particular SIMD instructions. Cargo tests use unwind
semantics even though a release panic now aborts the process without unwinding.
The interrupted older-profile run is retained as an interruption, followed by
complete controller qualification against the exact user-provided manifest.
See https://doc.rust-lang.org/cargo/reference/profiles.html .

Protected executable: /home/charlie/repo/RUST/qmp-qemu-socket/Evidence/Pyramid/Validation/pyramid-temporal-168-4ysoo5xU/native/qmp-qemu-socket-2.9.168

SHA256 `8f20ec38b2fc59af2b7e99d0a838ce5320bf9c1acdc801a7120a41d3d7bb8013`, mode 0700.

## Host qualification

All six controller and sixteen library gates pass. Full commands, exits, durations
and log hashes are in qualification-app.json and qualification-xsar.json.
aligned-qualification.json records final locked checks/releases and artifact provenance.
Failed attempts and focused-test corrections remain in the private case. The six
legacy guided-test failures exposed synthetic probe-only cards and superseded
128-pixel thresholds; only test evidence was corrected, with production unchanged.

## Code Analysis

The library uses safe byte extraction, abs_diff and checked lengths. Exact percentage
arithmetic avoids multiplication overflow. Samples use checked native geometry,
row stride and exclusion unions; alpha cannot prove an occurrence. Prefix counts
are labelled and full-count diagnostics scan the whole region. Colour change
does not replace semantic identities, acquisition/context, STOP, input permits,
release-only recovery or no-replay rules. The exact policy intentionally refuses
some legitimate small glyph changes; future coverage and live behaviour remain open.

## Lifecycle metadata preservation

lifecycle-memory-audit.json records one automatic checkpoint addition to the
project-local Tree Ring store: mem_20261029_142348_5c49750ebe53. All 163
prior memory rows and all other plain store tables are unchanged. The exact
before/after store hashes and checkpoint operation are pinned; this is not
a blanket metadata exclusion. The original 167 sentinel regression remains
historical; the 168 checkpoint binds the reconciled state and current artefact.

## Preserved server-local configuration

An independently added qemu-screen-grab-server/.cargo/config.toml is recorded
in external-server-config.json with its exact hash and single untracked-status
addition. It was left untouched and was not used by these controller builds.
Server HEAD, parent, index and tracked diff remain unchanged.

## Complete owned paths

- /home/charlie/repo/RUST/qmp-qemu-socket/.cargo/config.toml
- /home/charlie/repo/RUST/qmp-qemu-socket/CHANGELOG.md
- /home/charlie/repo/RUST/qmp-qemu-socket/Cargo.lock
- /home/charlie/repo/RUST/qmp-qemu-socket/Cargo.toml
- /home/charlie/repo/RUST/qmp-qemu-socket/README.md
- /home/charlie/repo/RUST/qmp-qemu-socket/docs/Pyramid-Acceptance-Before-3.0.0.md
- /home/charlie/repo/RUST/qmp-qemu-socket/docs/Pyramid-Short-Term-Goal.md
- /home/charlie/repo/RUST/qmp-qemu-socket/docs/Verification-2.9.168.md
- /home/charlie/repo/RUST/qmp-qemu-socket/docs/development.md
- /home/charlie/repo/RUST/qmp-qemu-socket/docs/handoff.md
- /home/charlie/repo/RUST/qmp-qemu-socket/docs/milestones.md
- /home/charlie/repo/RUST/qmp-qemu-socket/docs/pyramid-execution.md
- /home/charlie/repo/RUST/qmp-qemu-socket/src/app.rs
- /home/charlie/repo/RUST/qmp-qemu-socket/src/main.rs
- /home/charlie/repo/RUST/qmp-qemu-socket/src/parameters.rs
- /home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid.rs
- /home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_effect.rs
- /home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_recon/vision.rs
- /home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_recon/vision/removal.rs
- /home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_recon/vision/removal/joint_tests.rs
- /home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_recon/vision/removal/tests.rs
- /home/charlie/repo/RUST/qmp-qemu-socket/src/worker.rs
- /home/charlie/repo/RUST/qmp-qemu-socket/src/worker/computed_route.rs
- /home/charlie/repo/RUST/qmp-qemu-socket/src/worker/computed_route/continuation.rs
- /home/charlie/repo/RUST/qmp-qemu-socket/src/worker/computed_route/redeal_appearance.rs
- /home/charlie/repo/RUST/qmp-qemu-socket/src/worker/computed_route/series_guard_tests.rs
- /home/charlie/repo/RUST/qmp-qemu-socket/src/worker/computed_route/series_guard_tests/cadence_tests.rs
- /home/charlie/repo/RUST/qmp-qemu-socket/src/worker/computed_route/wire_tests.rs
- /home/charlie/repo/RUST/qmp-qemu-socket/src/worker/computed_route/wire_tests/redeal_tests.rs
- /home/charlie/repo/RUST/qmp-qemu-socket/src/worker/computed_route/wire_tests/stock_empty_tests.rs
- /home/charlie/repo/RUST/qmp-qemu-socket/src/worker/pyramid_execution.rs
- /home/charlie/repo/RUST/qmp-qemu-socket/src/worker/pyramid_reconnaissance.rs
- /home/charlie/repo/RUST/qmp-qemu-socket/src/worker/pyramid_reconnaissance/post_confirmation_tests.rs
- /home/charlie/repo/RUST/qmp-qemu-socket/src/worker/pyramid_reconnaissance/starting_guard_tests.rs
- /home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recognition-168/README.md
- /home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recognition-168/coral/deal.json
- /home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recognition-168/coral/error-state.json
- /home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recognition-168/coral/first-unmatched.png
- /home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recognition-168/coral/last-observed.png
- /home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recognition-168/green/deal.json
- /home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recognition-168/green/error-state.json
- /home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recognition-168/green/first-unmatched.png
- /home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recognition-168/green/last-observed.png
- /home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recognition-168/provenance.json
- /home/charlie/repo/RUST/xsar/.cargo/config.toml
- /home/charlie/repo/RUST/xsar/CHANGELOG.md
- /home/charlie/repo/RUST/xsar/README.md
- /home/charlie/repo/RUST/xsar/docs/milestones.md
- /home/charlie/repo/RUST/xsar/src/image_matching.rs
- /home/charlie/repo/RUST/xsar/src/image_matching/packed_rgb.rs

## Final lifecycle receipt audit

lifecycle-closeout-audit.json pins four additional successful empty startup recalls
from this project/store/workflow and two preserved SQLite support files. The WAL
is empty and the main store retains the previously audited exact checkpoint hash.
These six paths are individually sealed; no files were removed or blanket-ignored.
telemetry-closeout-audit.json separately records one AGENTS.md read added to the
bounded64-event trigger-tree history, with files/scans unchanged.
