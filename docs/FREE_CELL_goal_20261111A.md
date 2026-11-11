# Free Cell corrections — 3.1.10 / xSAR 0.14.2

Status: host qualification complete; fresh no-delay gesture acceptance pending. Charlie approved the
application patch and necessary library patch on 11 November 2026. The operator
reports about 35 successful 3.1.9 game moves with AUTO follow-ups enabled. That is
bounded operator evidence for the earlier build, not acceptance of this gesture.

During closeout Charlie confirmed playing through the calculated A* winning
route successfully. This is recorded as operator route validation. The new
no-delay gesture awaits its own trial.

## Approved scope

1. For A* Free Cell SINGLE STEP with ABS selected, send source movement, LEFT down,
   ABS destination, the identical ABS destination and LEFT up. Acknowledge source
   positioning first. Add no intentional gesture waits or standalone clicks.
2. Keep the original RECON and independently verified route after an ordinary
   playback failure. Keep idle playback buttons available. The next explicit
   request assumes manual UNDO ALL, starts at move 1 unbound and freshly compares
   all 52 physical cards with the cached original before any input. Changing the
   transfer policy reuses the same solution; it does not rerun A*.
3. Add the retained native normal 6D rank prototype to both recognition banks.
   Keep rank/suit thresholds and distinct-label competition unchanged; do not
   infer unread cards by elimination or accept the earlier ambiguous 9D.
4. Correct the measured gold 5H right-rail red minimum from 220 to 219 while
   retaining complete geometry. Challenge HALOs mark goal cards only. They never
   select the next move; input comes from the recorded A* route.

## Authority and bounds

Existing fresh parent, VM/absolute-pointer, STOP/context/source and exact effect
checks remain. Two successive fresh final combined recorded endpoints within six
observations are still required. The default policy observes recorded AUTO
follow-ups; it does not change Solitaire settings. Missing or uncertain effects
never trigger automatic replay or guessed cursor credit.

Live permission and cached solution are separate. Failure retires the uncertain
physical history; a new button press starts a different invocation. STOP, source
or context change, replacement, disconnect and shutdown still invalidate the
cache. Positive completion and successful REL probes retain their retirement
rules. A late completion cannot restore invalidated input authority.

Normal remaining-route drag, other games, REL pacing, public signatures,
dependencies, schemas, edition/MSRV and target defaults are unchanged. The console
consumer permits only xSAR lockfile alignment and remains version 0.8.5.

## Qualification and delivery

Run focused regressions and the paired locked host gates: scoped format, check,
private rustdoc, Clippy, full tests including ignored cases and release builds;
also library dependency-free defaults, isolated optional features and the actual
console consumer. Preserve first failures and bind all results to exact inputs,
configuration and toolchain. Compare all four Git histories/indices with the
starting snapshot. No guest input, process replacement, installation or promotion
is part of this build.

Deliver the actual qualified executable and its digest. Fresh Windows acceptance
of the double-destination gesture remains an operator trial. See the
[contract](FreeCell-Corrections-3.1.10.md) and [verification](Verification-3.1.10.md).
