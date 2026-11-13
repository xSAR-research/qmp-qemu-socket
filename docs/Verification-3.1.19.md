# Verification — qmp-qemu-socket 3.1.19

Status: host-qualified.

## Scope and continuity

The pinned prior application 3.1.18 receipt records 1,905 tests; that result is historical. The starting snapshot preserves 1,064 baseline compiler inputs, 1,126 preimages and four Git states. Application 3.1.19 alone is changed; xSAR 0.16.2, console 0.8.5 and qemu-screen-grab-server retain their starting source/manifests/lockfiles.

Before mutation, the 3.1.18 Destination Sentinel passed exactly `HANDOFF VERIFIED (scoped source/evidence; no cached build claim)` against independently supplied closeout SHA-256 `6d5d9be0da6148844f6a3cf53592a01c232912dc91eaa202ae93bcd172b36cef`. The prior capsule remains immutable. The new 3.1.19 sentinel is unclaimed.

## Final host qualification

All six fresh application gates passed with 1,908 tests and zero failed, ignored, measured or filtered tests. xSAR 0.16.2 and console 0.8.5 retain historical qualification evidence; no fresh standalone sibling gates are claimed.

Final 1,068-input seal `ac3e54f1830314c0fe0b41ba26e411b210741a917defbf354afb13065f4b3745` and toolchain/configuration seal `577f3299ff52c92186ab0281bc2b63d9021d281d99a19939e26c819915767a52` bind all six fresh application commands. Gate receipts retain exact argv, environments, log hashes, durations and stable before/after source/toolchain observations.

Toolchain observations:

```text
rustc 1.102.0-nightly (ac8310ba6 2026-11-09)
binary: rustc
commit-hash: ac8310ba643a3796cbe77d2e06944d7c8513999f
commit-date: 2026-11-09
host: x86_64-unknown-linux-gnu
release: 1.102.0-nightly
LLVM version: 23.1.5
cargo 1.102.0-nightly (c07ac2c4d 2026-11-06)
release: 1.102.0-nightly
commit-hash: c07ac2c4d12237effc4b94f4543769b421790045
commit-date: 2026-11-06
host: x86_64-unknown-linux-gnu
libgit2: 1.9.6 (sys:0.21.0 vendored)
libcurl: 8.21.0-DEV (sys:0.4.90+curl-8.21.0 vendored ssl:OpenSSL/3.6.3)
ssl: OpenSSL 3.6.3 9 Jun 2026
os: Arch Linux [64-bit]
rustfmt 1.12.0-nightly (ac8310ba64 2026-11-09)
```

| Gate | Seconds | Command | Log SHA-256 |
| --- | ---: | --- | --- |
| format | 0.811 | `rustup run nightly rustfmt --edition 2024 --config unstable_features=true,skip_children=true,blank_lines_lower_bound=0,blank_lines_upper_bound=2,style_edition=2024 --check src/freecell_recon/position.rs src/freecell_recon/position/tests.rs src/worker/freecell_route.rs src/worker/freecell_route/tests.rs src/worker/freecell_route/tests/native.rs src/qmp.rs src/app/freecell_route.rs` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| check | 11.695 | `cargo +nightly check --locked --offline --package qmp-qemu-socket --target x86_64-unknown-linux-gnu --all-features --all-targets` | `de91a6657c2d2901c277344092bd4c0fdf18cdd9a3c1db968c8d4c7cf81b5561` |
| rustdoc | 3.125 | `cargo +nightly doc --locked --offline --package qmp-qemu-socket --target x86_64-unknown-linux-gnu --all-features --no-deps --document-private-items` | `96103cd32a534f8f61d53a3fb15551fd3c055e35cf3b2c2c6973ea8ccf695339` |
| clippy | 11.916 | `cargo +nightly clippy --locked --offline --package qmp-qemu-socket --target x86_64-unknown-linux-gnu --all-features --all-targets -- -D warnings` | `3885a30598253dcc88d1d7e0b26bfd9be9549ead3dc9cbfd972e5f870ca137ac` |
| tests | 98.939 | `cargo +nightly test --locked --offline --package qmp-qemu-socket --target x86_64-unknown-linux-gnu --all-features -- --include-ignored` | `8ed52393d4b84e70acd93419a358148284c8634277b1e43a42077974d346633a` |
| release | 57.123 | `cargo +nightly build --locked --offline --package qmp-qemu-socket --target x86_64-unknown-linux-gnu --all-features --release --message-format=json-render-diagnostics` | `74cc41914b32113d0f2e5041cc6e86073882fad58ae327708e246598e6f9c2a1` |


Final qualification verifies all four starting HEAD/index/staged-diff fingerprints and the three sibling tracked source/manifests/lockfiles/unstaged diffs. Historical sibling artifact references remain in retained-prior-artifacts.json without a current mutable-artifact or fresh-build claim.

## Focused checks and retained first failures

Focused native/local-peer tests exercise same-offset empty CELL/SUIT/PLAY destinations, unchanged occupied observations, the exact JC planned path (233,722)→(233,677)→(495,427)→(758,132), fresh parent-position/VM/pointer/ACK gates, exact single-attempt input, and direct child-position proof refusal against the authentic retained returned-card frame. The original Sixes deal and recorded single-card action prefix supply the parent position. Full-suite records govern final host qualification.

The following actual failed attempts remain retained; successful retries do not replace their first-failure evidence.

- `/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/FreeCell/Validation/freecell-drop-anchor-319-20261111/pickup-focused.json`: exit 101; retained log `/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/FreeCell/Validation/freecell-drop-anchor-319-20261111/pickup-focused.log`, SHA-256 `6bb3811e87a9d62aa3f7358850a5d37e2e5e1c0e66db2d4b73cf4c87d10cc426`. First diagnostics: error[E0277]: can't compare `[Option<u8>; 4]` with `&[Option<{integer}>; 4]`; error: could not compile `qmp-qemu-socket` (bin "qmp-qemu-socket" test) due to 1 previous error
- `/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/FreeCell/Validation/freecell-drop-anchor-319-20261111/pickup-focused-retry.json`: exit 101; retained log `/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/FreeCell/Validation/freecell-drop-anchor-319-20261111/pickup-focused-retry.log`, SHA-256 `43037d82c44759a5b62480d58e0ce5e88490217e02bed51403ac7a013a1653ee`. First diagnostics: test worker::freecell_route::tests::native::freecell_3119_native_returned_jc_to_cell4_keeps_pickup_offset_and_exact_path ... FAILED; thread 'worker::freecell_route::tests::native::freecell_3119_native_returned_jc_to_cell4_keeps_pickup_offset_and_exact_path' (1331936) panicked at src/worker/freecell_route/tests/native.rs:816:60:; thread '<unnamed>' (1331937) panicked at src/worker/tcp_capture_tests.rs:167:17:
- `/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/FreeCell/Validation/freecell-drop-anchor-319-20261111/pickup-focused-final.json`: exit 101; retained log `/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/FreeCell/Validation/freecell-drop-anchor-319-20261111/pickup-focused-final.log`, SHA-256 `26745a515829379aa4452e616f90adf62619e007de95f81dc2414ffb392bab36`. First diagnostics: error[E0308]: mismatched types; error: could not compile `qmp-qemu-socket` (bin "qmp-qemu-socket" test) due to 1 previous error

## Artifact

Executable `/home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket` is version 3.1.19, 16,842,312 bytes, mode 0755, SHA-256 `1754b4ec28d9bbc5836f6ec6da8c35382ee7489f15ef4d554d615fc98c152520`. It is the mutable Cargo output. Rebuilding may replace it and does not replace an already running process.

## Original failure and evidence limits

The original app 3.1.18 run 61/request 1/source-generation 1 packet `pyramid-error-1794397492303-1326849-3` records JC PLAY 1→CELL 4 at transfer 40/87. The planned pointer endpoints were (233,722) and slot centre (811,203). CELL 4 face bounds are `[743,880)×[112,294)`; the recognised source origin is (218,702), giving a (15,20) pickup offset. The old destination uses (68,91), a +53 px,+71 px anchor mismatch. Under a retained-offset model this predicts a card origin (796,183), while a matching destination cursor is (758,132). These are model predictions, not measurements of a held card at release. The operator reported a drop too far right followed by return. The retained first/latest PNGs show empty CELL 4 and JC returning/returned near PLAY 1; they do not preserve the held destination state. The session log explicitly selected HalfSecond 500 ms and its six input timestamps have adjacent gaps 501, 500, 500, 501, 500 ms. This was not a fast-timing run. One attempted and acknowledged gesture proves delivery only; failed child proof earns no cursor advancement and no replay authority. The separate 3.1.17 6S CELL 4→SUIT Spades failure remains a distinct historical incident. Original packet/PNGs and exact source/log hashes/line anchors are retained in [incident-assessment.json](../Evidence/FreeCell/Validation/freecell-drop-anchor-319-20261111/incident-assessment.json).

## Retained playback contracts

The first post-transfer wait remains min(configured settle, 200 ms); one eligible endpoint match uses min(configured settle, 100 ms) before confirmation. Unmatched frames and intermediate AUTO endpoints use the configured retry gap. Two successive fresh exact semantic endpoint matches are required within six captures; the sixth rejection ends immediately. At the default 750 ms configuration, all-unmatched captures follow 200, 950, 1,700, 2,450, 3,200 and 3,950 ms of intentional waiting, excluding acquisition/proof costs. This retains 3.1.18 cadence, including its 550 ms shorter all-unmatched window relative to the earlier sequence from 750 to 4,500 ms.

MULTI retains STOP AFTER STEP bound to the exact plan/run. The current transfer, two-frame proof, recorded AUTO follow-ups and animation waits finish before a proved cursor is retained for resumption. A request before the next transfer, including one received inside its PAUSE checkpoint, prevents that transfer. PAUSE remains independent and requires CONTINUE. Immediate STOP revokes further authority and permits only existing release-only recovery.

Playback retains the bounded latest-frame preview slot and releases completed same-context RECON preview ownership on successful dispatch. Every already acquired playback frame reaches that slot before semantic interpretation. Pending-RECON and stale-context fences remain. During a held gesture the preview retains the newest acquired frame until the next existing capture; there is no added in-flight capture or capture-only QMP connection.

Fresh Windows acceptance of 3.1.19 remains pending. Host regressions use retained/synthetic pixels and local TCP/Unix-socket peers. The retained JC frame demonstrates direct child-position refusal, and local peers verify the corrected outgoing packet sequence. This correction does not launch an app, take a new guest capture, send guest input, install, commit, push or publish. A rebuild does not replace an already running process; live acceptance must identify the executing host and capture-server builds.

See [goal](FREE_CELL_goal_20261111J.md), [contract](FreeCell-Drop-Anchor-3.1.19.md) and [verification](Verification-3.1.19.md).

## Code Analysis

Native anchor and path arithmetic remains checked and bounded before probes/input; calibration and image recognition remain authoritative. This application-private coordinate correction preserves sibling source/API/dependency versions and the single-card domain contract. Input authority remains exact-source/run/context owned. Rejected or stale pixels cannot earn cursor credit, and STOP or uncertainty cannot authorise replay. No unsafe Rust or new dependency is introduced. Transport acknowledgement, reconstructed geometry and local-peer tests leave the actual Windows held-card anchor and physical landing as live acceptance limits.
