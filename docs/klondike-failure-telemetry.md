# Klondike exact failure telemetry

## 3.0.2 — explicit Single step extended log

Removes the invalid Klondike SUIT/foundation scene requirement. Four felt gutters and at least five of seven PLAY top-edge probes remain mandatory; source HALO, Solver banner, fresh VM/pointer probes, STOP and no-replay rules retain their separate authority. Both byte-original incident frames now reach SUIT 3 at (1296,199). The other four GAME TYPE scene checks have no equivalent SUIT gate.

Extended `SINGLE_STEP` JSON telemetry is written only to the complete session log for the explicit **Single step** button. Multiple Steps, including a budget of one, remain untraced. It records existing capture identities/probe counts/classification and attempted/acknowledged transport operations, without extra captures, retries or visible-panel noise. Existing failure packets remain independent and enabled.

The SUIT-strip diagnostic counts remain advisory (`foundations_required=false`). The normal session log contains prefixed `SINGLE_STEP` JSON lines only during the explicit Single step request, including begin/end, settings/context, capture attempts/results, exact response identities and selected-game classification, plus QMP attempts/results. ACK means delivery only. An error is recorded without replay. Scope exit disables telemetry, including early validation refusal. Inactive requests do not build diagnostic JSON. Successful extended lines bypass the visible panel; a session-file write error is surfaced visibly.

The original 3.0.1 first/last failure-packet retention below remains active regardless of this switch. No added capture is performed for either telemetry mechanism. Capture-completion timestamps are sampled before telemetry computation. [3.0.2 verification](Verification-3.0.2.md).

Version 3.0.1 instruments the existing controller; it does not relax recognition or claim the rare refusal is fixed. All four SUIT source columns remain implemented. One source click is sent; the game Solver owns the destination.

## Reproduce the retained game

1. Preserve the game/deal. Restart Codex when ready; do not reset the guest merely for the editor restart. After this build is delivered, exit/relaunch the controller from the repo base with `cargo run --release` so the running process is 3.0.1.
2. Select Klondike and Computer Vision. Repeat the same single-step sequence or continuous run on the retained deal. Do not manually move or pause Solver until the application stops and its packet path is saved.
3. Copy the session log and the `Klondike exact failure packet:` path shown in the log. Send that path and the unchanged packet before manual testing. Do not replay uncertain input.

## What the packet contains

The private `klondike-error-*` directory is written under the session-log directory (normally `/home/charlie/tmp`), outside the startup-reset images directory. It contains `state.json`, `first-rejected.png` when a classified scene rejection occurred, and `last-observed.png` when final pixels are available. The PNGs use the existing decoded full-resolution frames; no additional acquisition occurs. The first rejection remains immutable across later observations.

State records the original failure, bounded run/settings/context, last attempted input and its acknowledgement separately from effect, capture ordinal/timing/layout, and the exact TCP protocol/session/request/frame/display-generation/reuse identity when supplied by that response. Legacy QMP has null TCP identity. Raw diagnostic frames have null scene classification. Ordinal exhaustion refuses stale metadata pairing. Input ACK never proves a move completed.

Each saved frame is independently inspected with the same predicates and geometry as the unchanged authority: all four felt gutters at 950 per mille, seven PLAY top strips and four SUIT top strips at 700 per mille, requiring five PLAY and three SUIT matches. All fifteen counts/minima/bounds are reported even when the classifier short-circuited. The recorded authoritative classification and diagnostic agreement remain distinct. Persistence warnings or omitted oversized/invalid images never replace the original failure; partial private artifacts are retained without overwrite.

The original and repeated 3.0.0 incident logs/newer read-only captures are preserved in `Evidence/Klondike/Validation/klondike-300-suit-mvohidm_`. Those later captures passed scene arithmetic and are not the original failed app frames. The new packet is intended to close that evidence gap. Fresh live reproduction remains pending.
