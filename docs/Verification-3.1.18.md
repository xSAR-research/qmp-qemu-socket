# Verification — qmp-qemu-socket 3.1.18

Status: host-qualified.

## Scope and continuity

Application-only correction uses xSAR 0.16.2 and console 0.8.5. Before mutation, the 3.1.17 sentinel passed exactly `HANDOFF VERIFIED (scoped source/evidence; no cached build claim)` against independently delivered closeout SHA-256 `72acf70193db34347c9c171561b978b23954a686038c02195d1f2d294c4d312e`. Starting state contains 1,061 baseline compiler inputs, 1,112 preimages and four preserved Git states. The prior capsule stays immutable.

## Final host qualification

All six fresh application gates passed with 1,905 tests and zero failed, ignored, measured or filtered tests. xSAR 0.16.2 and console 0.8.5 retain historical qualification evidence.

Final 1,064-input seal `6ff31fac24eff444a230a5b479f895510825f4f4fe282c72ac61ba00dee8688c` and toolchain/configuration seal `577f3299ff52c92186ab0281bc2b63d9021d281d99a19939e26c819915767a52` bind all six fresh application commands. All selected receipt/log hashes, exact commands, environments and before/after source/toolchain checks passed.

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

All four repositories retain their starting HEAD, index bytes and staged diffs. All three siblings retain their exact starting tracked source/manifests/lockfiles and unstaged diffs. Sibling artifact references in retained-prior-artifacts.json are historical evidence. This correction runs no fresh standalone sibling gates.

| Gate | Seconds | Command | Log SHA-256 |
| --- | ---: | --- | --- |
| check | 13.348 | `cargo +nightly check --locked --offline --package qmp-qemu-socket --target x86_64-unknown-linux-gnu --all-features --all-targets` | `d7cc93eb3dd01d28bb5432afc4508cdf71d58c705a8552dd9fe9da514dea0aa3` |
| clippy | 13.005 | `cargo +nightly clippy --locked --offline --package qmp-qemu-socket --target x86_64-unknown-linux-gnu --all-features --all-targets -- -D warnings` | `9d84eabac1dbd9dd5f660f5dd1743b4e8e06582873b2a9488598ac995008373f` |
| format | 0.96 | `rustup run nightly rustfmt --edition 2024 --config unstable_features=true,skip_children=true,blank_lines_lower_bound=0,blank_lines_upper_bound=2,style_edition=2024 --check src/freecell_route.rs src/freecell_route/tests.rs src/app.rs src/app/freecell_recon.rs src/qmp.rs src/app/freecell_route.rs src/worker.rs src/worker/freecell_route.rs src/worker/freecell_route/tests.rs src/worker/freecell_route/tests/native.rs src/worker/freecell_telemetry.rs src/worker/qmp_wire.rs` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| release | 61.166 | `cargo +nightly build --locked --offline --package qmp-qemu-socket --target x86_64-unknown-linux-gnu --all-features --release --message-format=json-render-diagnostics` | `537de9664090839bab6ec3904e475135f6179f613cf4700b133964d900f8c460` |
| rustdoc | 2.982 | `cargo +nightly doc --locked --offline --package qmp-qemu-socket --target x86_64-unknown-linux-gnu --all-features --no-deps --document-private-items` | `f7db5c689c370b41b5e37ec53839ca87e26774ff6681233dc335dc03afb21511` |
| tests | 99.826 | `cargo +nightly test --locked --offline --package qmp-qemu-socket --target x86_64-unknown-linux-gnu --all-features -- --include-ignored` | `7d0e41027fb764b1c7375bf079cb660557d5b64266dcb8cd56c40d976dfd55e7` |


## Focused checks and review

The first focused run passed ten tests. Review then identified a stop-after-step request received inside the pre-transfer PAUSE checkpoint could otherwise start another transfer after CONTINUE. The added recheck and regression passed in the final eleven-test focused run. Its input/toolchain stability and full command are in responsiveness-final-focused.json. Existing missing-AUTO tests now assert full configured retry cadence, and review corrected documentation to state the 550 ms shorter all-unmatched window explicitly. Both focused logs are retained. Full-suite results above govern final qualification.

Tests exercise real headless controls, exact-owner direct stop dispatch/idempotence/reset, the PAUSE-boundary request, completion/resume, configured/confirmation/post-transfer wait selection, six fresh unsuccessful captures with five retries, native 100/200 ms six-input gestures and exact acquired preview pixels/context/coalescing. Local peers supply retained/synthetic native frames. Wait-only tests assert acquisition counts stay constant.

## Artifact

Executable `/home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket` is version 3.1.18, 16841896 bytes, mode 0755, SHA-256 `c5cb75c9e9fa9128bb3d21ff9d65f2d87d46d270e1fafc5c7d97722285182fb7`. This is the mutable Cargo output. Rebuilding can replace it and does not replace an already running process.

## Saved failure

The retained 3.1.17 packet pyramid-error-1794394350164-1307502-10 shows 6S still in CELL 4 and 5S on the Spades foundation. Its first/latest PNG bytes match (SHA-256 `fb0c593a843e343bd6ba55790986b2224e88fe7aac04dcc100b17c0206a2a73a`). CELL 4 has zero qualifying outline pixels on each of its four required strokes. Gold tint also yields zero bright-paper pixels, so paper absence alone cannot establish an empty cell. The empty-slot refusal correctly prevents cursor 30→31 credit. Five attempted and acknowledged gestures establish delivery counts. The saved frames leave the input-side cause and in-flight Windows button state unresolved. The capsule retains the original PNG, error state/log and incident-assessment.json.

## Behaviour and practical limits

The first post-transfer wait is min(configured settle, 200 ms). After one eligible endpoint match, the confirmation wait is min(configured settle, 100 ms). Unmatched frames and intermediate AUTO endpoints use the configured retry gap. Two successive fresh exact endpoint matches are required within six captures. The sixth rejection ends immediately. At the default 750 ms configuration, an all-unmatched run reaches its six effect captures after 200, 950, 1700, 2450, 3200 and 3950 ms of intentional waiting. The prior sequence reached them after 750 through 4500 ms. The total observation window is therefore 550 ms shorter before capture/processing time, and the former final 750 ms rejection wait is removed.

Successful playback dispatch releases completed RECON preview ownership for the same context. Each acquired playback frame already enters the bounded latest-frame slot before semantic interpretation, so the preview can show the newest acquired readiness, effect and failure pixels. Pending RECON and stale-context fences remain required. Frames coalesce in one slot. During a held gesture the preview displays the latest acquisition until the next existing capture, without inserting an in-flight capture or capture-only QMP connection.

Fresh Windows acceptance of 3.1.18 remains pending. Host tests use retained/synthetic pixels and local TCP/Unix-socket peers. Relevant source/destination/AUTO identity proofs, fresh original-52 binding, source ACK, fresh VM/pointer preflight, queued timing and source/run/context/STOP fences govern input. QMP delivery and physical effect have separate evidence. Uncertain non-idempotent gestures are never replayed. No application launch, new guest capture/input, installation, commit, push or registry publication occurred in this correction.

See [goal](FREE_CELL_goal_20261111I.md), [contract](FreeCell-Responsiveness-3.1.18.md) and [verification](Verification-3.1.18.md).

## Code Analysis

Arithmetic remains checked and all native path points are validated before probes/input. The private timing enum uses the existing xSAR Duration API, preserving sibling source/API/dependency versions. STOP AFTER STEP adds exact-run state under the existing route mutex and bypasses the serial worker command queue. A fresh request resets the flag, stale requests fail, and normal proof/cursor ownership remains authoritative. Short waits change cadence rather than the two-frame matching condition. Cancellation is checked between bounded native sleep slices and at semantic/input boundaries. Best-effort preview/log delivery, finite synchronous gesture cancellation latency and concurrent QEMU routing changes retain their practical limits.
