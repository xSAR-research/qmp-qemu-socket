# Verification — 2.9.181 reusable game operations

Added a private reusable game-operation layer for fresh native capture, guarded mouse/key input, pure card and option recognition, and bounded semantic Undo protocols. Pyramid Recon now uses its shared Undo All protocol with the existing capture, wait, confirmation, STOP and publication sequence preserved.

All six application gates passed, including 1725 tests with zero failed, ignored, measured or filtered, followed by the approved 2.9.181 locked check, release and protected help. The 56 unchanged xSAR inputs retain cached qualification. Fresh live guest acceptance remains pending.

Charlie approved milestone acceptance and version 2.9.181 in message m10449. Source qualification preceded the version-only alignment: 961 inputs were sealed; only the application's manifest and lockfile changed afterwards, leaving 959 identical. Twelve new tests cover the operation contracts, partial card reading and a local simulated QMP peer. Existing Recon restoration tests retain the production consumer's capture/confirmation/settle/restoration ordering.

The protected executable is [qmp-qemu-socket-2.9.181](/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/Pyramid/Validation/game-operations-181-jdwow79z/native/qmp-qemu-socket-2.9.181), SHA-256 `193d42c2cae352f62d37dff4470370751e1ccf161bbcd227f9baa0d10d5b3233`, 12587280 bytes, mode 0700. Run with `--image` for the existing GAME WIN image policy. The earlier 2.9.180 operations candidate remains a historical preacceptance record, separate from the delivered 2.9.180 executable.

## Capability and evidence limits

The native partial-card adapter supports Pyramid. The generic single-Undo protocol requires a measured enabled-control detector and semantic predecessor adapter before native use. Option matching counts unique maximal exact-black horizontal runs; every input still requires a same-frame game/action guard. Acknowledgement records delivery separately from semantic completion. No Free Cell A* solver or exploration orchestrator was added. Local peer and offline fixtures do not establish live guest acceptance.

## Code Analysis

Input consumes validated frame authority, checks context/STOP and same-frame policy, probes current VM/pointer capability, checks STOP again, and attempts one operation. Any delivery error permanently fences replay in that Session; acknowledgement remains recorded if cancellation arrives afterwards. Inclusive coordinates, frame layout and serial arithmetic are checked. Recognition borrows frames and performs no input. No new unsafe code, dependencies, threads, sibling changes or capture protocol changes were introduced. See [the complete contracts](game-operations.md).

## Reproducible records

- [Source qualification](/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/Pyramid/Validation/game-operations-181-jdwow79z/qualification-app.json) and [source seal](/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/Pyramid/Validation/game-operations-181-jdwow79z/qualification-inputs.json).
- [Approved alignment](/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/Pyramid/Validation/game-operations-181-jdwow79z/aligned-qualification.json) and [approval](/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/Pyramid/Validation/game-operations-181-jdwow79z/version-approval.json).
- [Implementation and full source-path list](/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/Pyramid/Validation/game-operations-181-jdwow79z/IMPLEMENTATION.md); [completion audit](/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/Pyramid/Validation/game-operations-181-jdwow79z/completion-audit.json).
- [Lifecycle reconciliation](/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/Pyramid/Validation/game-operations-181-jdwow79z/lifecycle-final-audit.json); [destination state](/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/Pyramid/Validation/game-operations-181-jdwow79z/destination-state.json).

The original 180 sentinel regression is retained. The known post-candidate Tree Ring capture changed the main store; its current bytes and exact receipt additions are pinned separately. Prior database/sidecar payloads were unavailable, so logical equivalence and old sidecar emptiness are not claimed. No guest action, installation, staging, commit or push occurred.

Suggested commit: `feat(operations): add guarded reusable game operations`.
