# QMP drag diagnostics — 3.1.6

Charlie approved this diagnostic build on 9 November 2026: application 3.1.6 with additive xSAR 0.13.0 wire observation. Patch changes need no separate version approval; major and minor changes do. Align the two consumer lockfiles without changing other packages or dependency requirements.

## Required behaviour

1. Record every outgoing QMP JSON request through the application's existing private session log. Include the exact JSON, numeric request ID, connection identity, emission time and separate attempted, flushed or failed write outcomes. Cover initial capability negotiation, probes, input, capture requests and release-only recovery on old and fresh connections. Logging must not add commands, retries or input authority.
2. An explicit A* Free Cell SINGLE STEP uses the frozen route coordinates. For the reported first move these are PLAY 8's 6♦ and PLAY 3's 7♠; those identities and coordinates must not be hard-coded.
3. Move to the source, pause one second, press LEFT, pause one second, move to the destination while held, pause one second, then release LEFT. Preserve release-before-reply collection and release-only recovery after uncertain button-down.
4. After the release is confirmed, wait three seconds, then perform one separately labelled LEFT click at the destination. This is an explicitly requested diagnostic action. It is not a substitute for destination-controlled route execution and may itself affect the game.
5. The pause and final click retain STOP, context and exact run/source authority. Before that separate click, acquire one fresh frame for completion and physical-position checks. Positive completion retires authority; unsupported or uncertain positions refuse the click. This capture is an input guard, not an extra recording capture. A stopped operation with a confirmed released button sends no new recovery traffic.
6. Normal remaining-route playback and other game controllers keep their existing input sequence and pacing. All their outgoing QMP commands are still logged. The existing two fresh exact physical endpoint proofs remain necessary before the route cursor advances; an acknowledgement proves delivery only.
7. Preserve the current source, prior evidence and incidents, all four Git histories/indices, and independent capture-server changes. No installation, application launch, guest input or Git promotion is authorised by the build request.

## Qualification

Use local Unix-QMP and TCP peers to check exact JSON against received bytes, handshake and recovery coverage, IDs and connection attribution, stage timing, cancellation and failure boundaries, Single Step selection, terminal/unsupported-frame refusal and unchanged normal wire behaviour. Include observer failure isolation and final session-telemetry drain. Retain first failures.

Complete the required locked owning-package checks, full tests including ignored cases, strict private rustdoc, Clippy and release builds; validate xSAR defaults and isolated features and the real console consumer. Bind receipts to the exact inputs, toolchain and executable. Host peers do not establish Windows/Solitaire drag acceptance or prove a QEMU defect.

## Diagnostic evidence

The saved 3.1.5 failure showed no net game-area change despite acknowledged input. Charlie visually reported a source click animation with no card following the pointer. A manual source click moved the same card to a legal cascade, but source clicks can choose a cascade when the route requires CELL. Explicit target control remains necessary. Loss of held state during movement is a hypothesis; the responsible component is unproved.

## Code Analysis

The observer is passive and does not own files or game policy. The application owns the existing private session logger. Observer failure must not replace the original transport error or bypass release-only recovery. JSON write, flush, QMP acknowledgement and proved game effect are distinct. Slow pacing adds cancellation boundaries but does not promise instant cancellation. The diagnostic click is separately identified, never retried, and cannot waive physical endpoint verification.
