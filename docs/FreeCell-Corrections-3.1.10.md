# Free Cell recovery, recognition and double ABS — 3.1.10

Status: host-qualified; fresh Windows acceptance pending. Paired xSAR is 0.14.2.

## SINGLE STEP

With A* Free Cell and ABS selected, the input sequence is:

1. Move to the checked source card location and receive its acknowledgement.
2. Hold LEFT down.
3. Move to the checked ABS destination.
4. Send the identical ABS destination again.
5. Release LEFT.

There are five input packets, distinct request IDs and exactly two identical
destination arguments. No intentional waits separate these input phases. Fresh
routing checks and protocol responses still take time. The four held-phase
responses are collected after UP; every possible post-down failure retains the
original cause and uses the existing captured-pointer old/fresh release-only
recovery. No down or movement is retried. Restore the original pointer only after
confirmed release while authority remains valid. Release can affect the guest;
it is not rollback.

Delivery acknowledgements are separate from game effect. The application still
settles and acquires fresh frames. Observe recorded AUTO follow-ups requires two
successive exact final combined witnesses within six observations, credits only
the proved records and stops SINGLE STEP. Execute each recorded transfer keeps
its immediate-endpoint rule. Neither policy changes Solitaire settings.

Normal remaining-route playback retains its existing drag. REL SINGLE STEP
retains ten tiny direction-only updates with two-second phase waits; it is a
movement probe with no route credit, not this ABS experiment.

## Cached UNDO ALL recovery

After ordinary playback failure, keep the immutable original deal and verified
route while discarding live position history. SINGLE STEP and SOLVE remaining
route remain available when idle. Preserve the original FAILURE DETAILS and
packet. The next explicit press assumes manual UNDO ALL and starts from move 1,
unbound; it must freshly recognise and compare all 52 physical cards before input.
A different board refuses with zero input while retaining the same cached route.
There is no automatic retry or continuation from an uncertain cursor.

A transfer-policy change reuses that report and resets playback to the original
board at the next explicit request. A successful step with unchanged policy
continues from its verified cursor. Source/context changes, STOP, disconnect,
shutdown and replacement invalidate the cache. Exact request/run/source ownership
and monotonic run IDs prevent stale results from reviving it. Successful terminal,
full-board, partial exhaustion and REL-probe outcomes retire as before. An ordinary
REL after-frame failure can retain only the original cache for a new UNDO ALL
request; the delivered probe's live permission is already gone.

## Native recognition corrections

The normal 6D frame is retained byte-for-byte and supplies one labelled rank
prototype. Both initial and evolving-position banks use it; its suit template is
the original 6D suit bitmap. Rank 48/20, suit 16/12, coherent ink, local contrast,
variant offsets, distinct-label competition and complete inventory checks remain.
The earlier 9D scores 34/50 still refuse. This is measured calibration, not general
OCR or universal native-board coverage.

The real gold 5H right rail contains nineteen red-219 pixels. Only that role's red
minimum changes to 219. All required top/rail pixels remain positive evidence;
interrupted edges and red 218 still refuse. HALOs indicate challenge goals only,
never route instructions or input authority.

## Evidence and limits

Host tests cover exact raw wire/observer packets, source-ACK and release-before-
reply ordering, STOP/write/command failures, cached retry/mismatched board/policy
fences, the two authentic complete inventories and damaged/ambiguous negatives.
During closeout Charlie confirmed playing through the calculated A* winning
route successfully. This is recorded as operator route validation. The new
no-delay gesture awaits its own trial.

Local peers and synthetic composed position frames do not establish fresh guest
timing or Windows held-button state. The operator's earlier 35 moves support the
3.1.9 policy only. This no-delay gesture still needs its own live trial.

See [goal](FREE_CELL_goal_20261111A.md) and [verification](Verification-3.1.10.md).

## Code Analysis

Public APIs and schemas are unchanged. Coordinates and image access remain
checked and bounded; independent replay and physical identity proofs constrain
input. Cache retention never retains uncertain live permission. New invocations
must authenticate the original board, and hard invalidation cannot be reversed
by late completion. No new unsafe code, dependencies or background activity is
introduced. Transport uncertainty permits release only, never input replay.
