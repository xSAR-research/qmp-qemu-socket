# Verification — 3.1.7 / xSAR 0.14.0

Application **3.1.7** with xSAR **0.14.0** is host-qualified for explicit ABS/REL Free Cell A* SINGLE STEP diagnostics. Each request freezes one mode; both use at least two seconds between source positioning, LEFT down, held movement and LEFT up. No standalone source or destination click follows. Full-route playback retains its original gesture.

Exact outgoing JSON appears unescaped on its own line in the existing visible LOG and private session, after a QMP_TX identity header. Separate QMP_WIRE metadata retains microsecond/source clocks, owner, primary/recovery connection, QMP ID and write outcomes. Neither flushing nor acknowledgement proves guest effect.

All **51 fresh required gates passed**, bound to **1,032 compiler inputs** at SHA256 `a28d0d145b00d0807146f28fbfcb85f88dd4230e28d1a8d5366f4a7ba6f8f5c5` and the recorded toolchain/configuration. Full tests: **1,850 application, 345 library and 80 console**, zero failed, ignored or filtered. The matrix includes dependency-free defaults, sixteen isolated features, strict private rustdoc, Clippy and release builds for both owners and the real console consumer.

REL first queries the pointer inventory, positions the source absolutely, selects/verifies a unique relative pointer and rechecks it before pressing. Signed native deltas are approximate relative device units. Original-pointer restoration requires acknowledged UP and continuing authority. The explicitly approved failure exception selects the captured relative pointer before old/fresh release-UP; no DOWN or motion is replayed. STOP/error may leave REL selected. Concurrent routing changes remain outside a pinned-device guarantee.

Qualified mutable Cargo executable: `/home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket`, **3.1.7**, 16806144 bytes, mode 0755, SHA256 `3c1ecef2d0eec235d868f4284daa7e33dffa0e855270b63d663474526db11567`. No protected copy, install, launch, guest input or Git promotion. Four histories/index trees/staged changes and independent server work are preserved. No fresh Windows/Solitaire drag acceptance or QEMU root cause is claimed.

## Acceptance

| Requirement | Evidence |
|---|---|
| Frozen ABS/REL selection | Production selector/queue tests and native REL dispatch |
| Two-second phases; zero extra clicks | Exact peer wire/timestamp tests for ABS and REL |
| Readable exact JSON in both sinks | Byte equality, current/stale reducers and independent append-failure cases |
| REL routing/restoration/recovery | Unique inventory, effective activation, fresh selection, UP/STOP and failure tests |
| Ownership/effect safety | Fresh physical source/target, VM/pointer/STOP fences; two exact endpoints within six observations |
| Compatibility | One additive neutral REL API; existing diagnostic ABS pacing intentionally changes 1s to 2s; normal drag unchanged |

## Qualification records

Exact commands, environments, timings, source/toolchain seals and preserved first attempts are retained in `Evidence/FreeCell/Validation/qmp-rel-317-20261109`. Toolchain: rustc 1.102.0-nightly (f5cedb83a 2026-11-07); cargo 1.102.0-nightly (c07ac2c4d 2026-11-06). First application compilation found an obsolete test-only egui harness API; the next focused run found UI re-enable hit-test sequencing and a local mock writing after old-connection invalidation. Those fixtures were corrected without weakening production authority or expected wire traffic. Additional actual gate failures, if any, remain in their original logs and receipts.

These are host, retained-pixel and local Unix-QMP/TCP peer checks. They do not demonstrate a successful real guest drag. Re-enable the guest HID/tablet path for initial ABS source positioning; REL also needs the relative guest device. A stopped REL trial can require operator pointer restoration before further ABS control. The executable must be relaunched explicitly; rebuilding does not replace a running process.

## Code Analysis

No dependencies, default features, requirement strings, schemas, unsafe code or ordinary drag policy were added/changed. The public REL API is additive, with an approved pre-1.0 minor transition and real consumer lock alignment. Coordinates/deltas are checked before input. Routing failures before DOWN send no press or held motion. Possible post-DOWN uncertainty preserves the original cause, invalidates the old client and uses only selected-pointer safety releases. Source/STOP/context ownership and semantic endpoint proofs remain independent of transport delivery. Relative counts and concurrent pointer routing are not calibrated/pinned guarantees. Logging remains best effort on output failure or abrupt termination; existing aborting profiles cannot recover process-aborting failures.
