# Verification — application 3.1.11 / xSAR 0.14.3

Status: host qualification completed; operator Windows acceptance remains separate. Console remains 0.8.5.

All 51 fresh paired host gates passed: 1,882 application, 353 library and 80 console tests, zero failed/ignored/filtered.

Scope: four one-second pauses in the existing five-input ABS SINGLE STEP diagnostic, related UI/telemetry and local-peer timing/STOP regressions. Normal route drag, REL, recognition, route caching, AUTO sequencing, public signatures, schemas and dependencies are unchanged.

The previous 3.1.10 scoped sentinel passed before mutation using independent closeout digest `28ebeb2138bc7661c3adb344b54a8b564a61bcc218c530f0b1be66f44bb9ffbd`. All 1,037 previous compiler inputs matched. Starting-state and exact preimages are retained under `Evidence/FreeCell/Validation/freecell-pacing-311-20261111/`.

## Fresh checks

The first focused library run passed 14 tests; the first native-worker run passed both tests. They verify exact five-input JSON, source ACK gating, four actual gaps of at least one second, held replies after UP, source-wait STOP and release-only recovery during held waits. The full 51-gate schedule then passed on its first attempts: scoped formatting, paired locked offline all-target checks, strict private rustdoc, Clippy, full tests including ignored, release builds, dependency-free defaults, all sixteen isolated library feature checks/tests and actual console integration.

The final 1,040-input seal is `17ad2764ee87d46b7f7909c2ce8b21859f2eb9aa6c17443e1b3f96501d10ffcc`. Inputs, selected toolchain and inherited configuration were checked before and after every gate. Exact commands, counts, logs, flags and revisions are in qualification-summary.json, qualified-final-inputs.json and toolchain-qualified-final-inputs.json. Standalone all-features and application-consumer xSAR artifacts are identified separately from Cargo JSON.

No focused or qualification check failed. A final-document drafting tool call was rejected before command execution; orchestration-note.json records it separately. Independent source/document review passed. Final targeted scans retain redacted identity/path findings and working-text fingerprints. All four HEADs, index bytes and staged diffs, unrelated server changes and prior release bodies were preserved.

## Delivery and evidence limits

Qualified mutable executable: `/home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket`, version 3.1.11, 16829984 bytes, mode 0755, SHA-256 `8454ac2c02eb8033354a2846b3aec692914bdca6bbe42ee353d057599b6d77fa`. This is mutable Cargo output; no protected install or agent launch occurred. A rebuilt executable does not replace a running earlier process. Close the older GUI and launch this qualified build for observation.

The prior sealed 3.1.10 capsule is unchanged. New closeout.json and verify-pacing.py provide the next scoped handoff. The new sentinel is not claimed passed using a self-derived expected hash: obtain its expected digest from the independent delivery record. Default coverage excludes mutable Cargo artifacts and volatile local tooling; add both artifact/toolchain flags for a current cached-build claim.

Local peers and saved/native-shaped pixels qualify wire order, timing, sequencing and guards, not Windows pickup or drop. No agent guest input/capture, application relaunch, driver change, install or promotion occurred. The earlier fast failure remains preserved; neither its cause nor necessity of the repeated destination is established. See [contract](FreeCell-Pacing-3.1.11.md) and [verification](Verification-3.1.11.md).
