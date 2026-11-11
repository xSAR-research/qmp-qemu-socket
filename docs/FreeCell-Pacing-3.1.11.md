# Free Cell five-packet pacing — 3.1.11 / xSAR 0.14.3

## Approved behaviour

Only A* Free Cell SINGLE STEP in ABS mode changes. It emits five separate `input-send-event` requests:

| Nominal time | Input |
| --- | --- |
| 0 s | Absolute source position; await its matching acknowledgement |
| 1 s | LEFT down after the fresh routing check |
| 2 s | Absolute destination |
| 3 s | The identical absolute destination |
| 4 s | LEFT up |

Each consecutive input pair has an intentional one-second pause. Protocol I/O, routing queries, guards and logging add time; these are minimum input-phase intervals, not exact wall-clock timestamps. Capabilities, pointer selection, probes, routing queries and restoration remain additional protocol requests. There is no additional click or pause after UP. The four held-phase replies are collected only after UP, preserving release before any held-reply wait.

The source pause follows its acknowledgement and precedes the final fresh pointer-routing check. The three subsequent pauses precede destination 1, destination 2 and UP. STOP/context authority is checked around every pause. Existing release-only recovery follows any possible post-DOWN failure; it may affect the guest and is not rollback. Ordinary pointer restoration requires acknowledged UP and continuing authority. Sleep and I/O make cancellation cooperative, not instantaneous.

The public API is unchanged. A private zero-delay seam makes negative local-peer tests bounded; real public API and native-worker positive tests exercise all four one-second waits. Normal `drag`, normal remaining-route playback, REL's two-second movement probe, `QmpConfig`, observer transport and coordinate mapping are unchanged.

## Evidence and retained safeguards

QMP acknowledgement proves delivery only. The application still binds all 52 physical cards to the original deal, freshly proves the parent, probes VM/pointer capability, checks STOP/context/source ownership, and requires two successive fresh exact final combined endpoints within six observations. Recorded AUTO follow-ups remain policy-bound. HALOs mark challenge goals only.

An ordinary playback failure retains the immutable verified route for a new explicit manual UNDO ALL restart. It never resumes uncertain live history: the next request starts at cursor 0, unbound, and must freshly match the original 52 physical cards. Source/context/STOP and terminal invalidation remain unchanged.

## Operator trial

Close the earlier application and launch the qualified 3.1.11 executable. Restore the original board with manual UNDO ALL before retrying its cached plan, or RECON and CALCULATE a fresh supported initial board. Select ABS and Observe recorded AUTO follow-ups. Press SINGLE STEP once and watch the four-second sequence: source positioning, pickup after DOWN, first destination movement, repeated destination and release. Assess the resulting card position and verified cursor. Keep the failure packet on refusal; never automatically replay uncertain input.

This release retains both destination packets so they can be distinguished visually. It does not establish that the second packet is necessary, that one second is a reliable minimum, or why the fast trial failed. Fresh Windows acceptance remains an operator test.
