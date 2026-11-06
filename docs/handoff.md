# Active handoff — qmp-qemu-socket 3.0.2

Approved goal m13299 completed: Klondike SUIT scene-gate correction, cross-game scene audit and explicit Single step extended session-file telemetry. xSAR remains 0.10.0; Free Cell A* remains deferred to 3.1.0. Guest confirmation of this correction is pending.

Baseline_HEAD_SHA: 30dc3835eca3adcc8d7f997df398fc569f5c918c
Baseline_PARENT_SHA: 0d2eb0fb6352473fbb31b28e39b74991076d4daa

## Run

From `/home/charlie/repo/RUST/qmp-qemu-socket`:

```sh
cargo run --release
```

GAME WIN storage is on by default; `cargo run --release -- --no-image` opts out only from that recording. Failure packets remain enabled; no secondary executable was created. Normal binary SHA256 `b8d8b3260749bbeb97a20065f4acfa4cfbe53bf1d83c0919b11247047c8b417b`, 16233120 bytes. Rebuilding this mutable target invalidates that artifact identity.

## Destination sentinel

Use the independent SHA256 supplied in the final delivery/checkpoint:

```sh
python3 /home/charlie/repo/RUST/qmp-qemu-socket/Evidence/Klondike/Validation/klondike-scene-302-kszlr13h/verify-destination.py --state-sha256 <trusted-delivery-state-sha256>
```

The [destination receipt](/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/Klondike/Validation/klondike-scene-302-kszlr13h/destination-state.json) pins 996 compiler inputs, all current documentation/evidence, exact four-repository HEAD/index/patch/status and observed server files. A mismatch is `HANDOFF REGRESSION`, not a cached pass. Preserve historical sentinels and exact discrepancies; automatic post-checkpoint metadata must be measured narrowly. Do not reset, overwrite evidence or broadly ignore memory paths. The server has independent staged/unstaged work; preserve it unchanged from the pinned observed snapshot.

## Evidence and next action

1,758 full source tests, six focused regressions, all six gates and aligned check/release/help passed. [Verification](Verification-3.0.2.md), [telemetry contract](klondike-failure-telemetry.md), [publication commands](/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/Klondike/Validation/klondike-scene-302-kszlr13h/COMMIT-PUSH.md). Current xSAR path-dependency revision c4a9b839ece3668b8d46ca00676ae05c34a97943 and dirty state are recorded, not a Git pin. Original incident PNGs, failed suite/preflight and prior 3.0.1 release evidence remain preserved. No guest input, GUI startup, extra capture, staging, commit or push was performed for release verification.

Charlie can relaunch the controller, reproduce using Single step and inspect `SINGLE_STEP` records in the normal complete session log. Multiple Steps with one operation remains untraced. The agent must not perform that live test without authority. Microsoft Free Cell foundation-return legality is unverified and no policy change was made. Git promotion remains Charlie-owned; use only the prepared explicit application paths and review each sibling independently.
