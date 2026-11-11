# Free Cell correction release — goal recovery

Recorded: 8 November 2026.

Goal ID: `01a1823f-b491-7941-8c49-0f520978aff9`.

## Goal

Build and verify the approved new application release qmp-qemu-socket v3.1.3 and associated xSAR crate v0.11.1 addressing the following 7 points:

1. **FAILURE DETAILS:** Automatically open a populated window when an error is encountered. Preserve the original worker failure and request identity; include useful stage, recognition, capture and evidence details. Stale or duplicate results must not replace a current failure. Persistence errors must not erase the primary error.
2. **Challenge text:** RECON must not fail solely because challenge details occupy the area previously compared with blank reference pixels. Replace that assumption with supported scene evidence, retaining the independent layout, geometry, identity, ambiguity, Solver and modal checks. Investigate how challenge information is retained with the acquired deal and subsequent planning/playback; do not silently equate a challenge objective with clearing the full board.
3. **Default game:** For current testing, default GAME TYPE to FREE CELL when entering the A* Solution section. Preserve the ordered five-game selector and explicit unavailable capabilities for unimplemented games.
4. **Alignment:** Clean up alignment of existing on-screen controls and output. The broader UI overhaul and button-styling proposal remain separate.
5. **Fresh RECON preview:** Each explicit RECON request must obtain a fresh screenshot before recognition and publish those same pixels to the application image control when acquisition completes. Recognition and any failure report must refer to that request's frame. Do not take another capture to manufacture diagnostic evidence.
6. **Free Cell telemetry scope:** Enable enhanced telemetry for all operations in **A* Solution → FREE CELL** during this development phase. Do not extend this new tracing to other games or Computer Vision workflows.
7. **Telemetry detail:** Record initiated steps, image requests and responses, RECON regions and pass/fail stages, QMP key codes and pointer/button operations, saved evidence filenames, and other relevant application stages. Keep delivery acknowledgements distinct from proved game effects. Logging must not add captures, inputs, retries or authority.

The recovered original seven-item request is available at `/tmp/freecell-seven-request-recovered.txt`. That temporary file is a recovery reference; the requirements above are retained here so continuation does not depend on it.

## Evidence and safety boundaries

- Preserve unrelated edits, calibration, incident evidence, all four repository histories and Git indices, and the capture server's independent staged and unstaged work.
- Keep this correction application-only unless further cross-repository changes are expressly approved. The recorded dependency baseline is xSAR `0.11.1`.
- Do not reset, stash, switch branches, stage, commit, tag, push, publish, install, or send input to the running guest as part of this scope.
- Preserve STOP/context revocation, exact request/result fencing, offline CALCULATE, strict recognition and no-overwrite publication. Never replay an uncertain non-idempotent transfer; preserve release-only recovery.
- Failure evidence must work independently of GAME WIN recording, including `--no-image`. Launching the application resets `/home/charlie/tmp/images`; preserve incident evidence before any separately authorised launch.
- Original failed RECON frames were not recovered. The later operator screenshot demonstrates a challenge-banner mismatch, not the exact cause in those missing frames. Its highlighted card border is a separate geometry issue and must remain fail-closed unless separately supported.

## Recorded release and preservation blocker

The current handoff records application `3.1.2` / xSAR `0.11.1`, with historical host test totals of 323 library, 1,789 application and 80 console tests. Those records do not qualify the requested correction or establish fresh guest acceptance. No new implementation, qualification or version transition was completed during this recovery turn.

The recorded destination receipt is:

`Evidence/FreeCell/Validation/freecell-route-312-9fxt_n42/destination-state.json`

Its independently recorded SHA256 is:

`ce18e413b6fc1a94d69c2286ab5379699aea36f0ccb2c812fe168f7a4b44b497`

The preservation verifier previously returned:

```text
HANDOFF REGRESSION: Repository state changed: qemu-screen-grab-server
```

The recorded server unstaged-patch digest changed from `093e405e67cf02f70ab334bae480005138b6f052f3f18b04f935e188ad47641e` to observed `311639238a6ed6e118a68cae88259af4fdedb7c316869bd62e31d4fca7ea21f3`. Historical patch bytes are unavailable, so individual hunks cannot be attributed to changes after the receipt. Application guidance/draft files and Tree Ring lifecycle files also differ from the old snapshot. Preserve all of these; do not edit old receipts or remove files to force agreement.

The latest inspected, unapproved reconciliation proposal is `/tmp/freecell-preservation-review-v2-53fey37r/preservation-proposal.json`, SHA256 `6fb065bb7a671c1601492d6a95795f2ac9a0a227eb13b5ab7a98ad7d7a4b9c20`. Its snapshot is historical and must be rechecked before adoption. It proposes a separate rooted reconciliation retaining the original receipt. No reconciliation was performed in this turn. The proposed application version `3.1.3` has not been assigned; the exact transition remains pending explicit approval.

## Verification and completion criteria

After the preservation and version gates are resolved, implement and check each of the seven requirements against the actual checkout. Use retained native evidence, explicitly labelled synthetic derivatives, host tests and local mock peers. Run the applicable locked Tier 2 gates, including tests with ignored tests enabled, strict private rustdoc, Clippy and a release build. Preserve first failed attempts and bind passing results to the exact source and final executable.

Report requirement-by-requirement evidence, the actual artifact and version, remaining limitations, and the distinction between host/local-peer qualification and fresh guest/operator acceptance. Do not declare the goal complete while required work or verification remains outstanding.

