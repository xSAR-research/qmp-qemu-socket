# Klondike v1.2.2, candidate 1

Date: 30 September 2026, Australia/Brisbane.
Base: v1.2.1, commit `1cc57c21e696759b5d61bbcee24b96a733e9c430`.
This document supersedes the execution limits in the historical
`klondike-candidate-1.md`; the original input and capture contracts still apply.

## Changes

- Klondike Multi-Step accepts **0 = continuous** or 1 through 10000 actions.
  The initial UI value remains 10. Continuous operation stops on STOP,
  uncertainty, unsupported scenes or the one-shot Solve endpoint.
- The editable **After Klondike card / Draw / Recycle / Solve action** delay
  defaults to **750 ms**, following Charlie's Beast observation. The separate
  re-observation delay remains **1000 ms** and editable. A run snapshots both
  settings; subsequent Params edits affect the next run.
- RIGHT-card replacement has an additional conservative source check and logs
  the actual source, corner and destination evidence on every effect check.
- The evidenced upper **Solve** control is recognised separately from the
  lower **Solver** hint control. Solve is clicked once, settled and captured;
  execution then stops for review, including in continuous mode.
- A diagnostic preview remains non-actionable. While a run is active, the UI
  describes the in-progress observation without repeatedly requesting a manual
  capture. A stopped uncertain result still requires a fresh explicit capture.
- A newly decoded Klondike frame is retained even if its analysis fails or STOP
  arrives immediately after capture. Acquisition/decode failures retain the last
  usable frame because no newer decoded pixels are available.

## What the supplied failure log establishes

The reported final stop followed one RIGHT click at `(680, 199)`. Three delayed
observations were performed. Their next target was correctly classified as
Recycle, but the preceding RIGHT effect remained unverified. The controller
therefore stopped before sending a recycle input. No recycle input appears in
the supplied session log. A later manual capture again recognised Recycle.

The failure is consequently a preceding-action verification stop, not evidence
that the recycle click itself failed. The before frame for that RIGHT action
was not supplied, so its precise live cause remains unproven.

Controlled tests using real card-face pixels from the supplied result images
reproduce a weakness in the previous source gate. A mostly white card can
replace another mostly white card while revealing too few newly white/felt
pixels outside the inset and cursor mask. An A-spades/J-clubs replacement and
a 2-diamonds/2-clubs replacement each produce only 71 positive source pixels,
below the existing threshold of 512. These are labelled synthetic cases;
they are not reconstructed live before/after pairs.

## RIGHT replacement evidence

The existing positive removal/reveal check remains in force. For RIGHT only,
replacement can alternatively be established by at least 48 material paper/ink
changes in **each** of two opposite corner patches. Both patches must retain
at least 50% white paper before and after; the commanded cursor area is excluded.
Either accepted source check still requires at least 512 changed content pixels
in a separate tableau or foundation destination region.

A fresh HALO alone never proves the preceding action. Source-only change,
unchanged cards, one changed corner, cursor-only changes and dark guide changes
remain refusal cases. The added check compares pixels; it does not decode ranks
or suits, establish legal moves symbolically, or run an independent solver.

## Solve control and result boundary

Detection priority is DRAW/RECYCLE, recognised Solve or RIGHT, bottom-up tableau,
then SUIT. Solve recognition uses 899 original RGB samples on a four-pixel
lattice over the supplied control, measured at `(562, 143)` with size
`124 x 113`. At least 98% of samples must match within 24 RGB levels both across
the control and within its check-mark/text area. Empty stock and a recognised
gameplay scene are separate requirements. The centre click is `(624, 199)`.
This button can be recognised with the lower Solver hint banner off.

After fresh validation and the normal pre-input STOP/tablet checks, one click
requests Solve. The worker waits the editable Klondike settle interval, captures
one decoded diagnostic result without requiring an already-known terminal
layout, and stops with **Solve requested — inspect result**. The request does
not increment a verified-completion counter or authorise any further input.
STOP or capture failure can prevent that result capture; already captured
pixels are retained even if STOP becomes set immediately afterwards.

No post-Solve animation duration, win screen, no-solution flow or restart
sequence has been established. Colour/theme, hover or layout changes may make
the strict template refuse the button. Such refusal must be diagnosed from a
fresh original PNG rather than widening detection speculatively.

## Continuous execution and recovery

Zero removes only the total gameplay-action limit. Each unresolved context
still allows at most one authorised Solver refresh and three delayed,
input-free observations. The first capture after Solver activation remains
immediate, with no additional post-click sleep. Finite runs also retain the
total Solver-refresh bound of their action limit plus one. Continuous counters
use checked arithmetic and stop before overflow.

All gameplay inputs require fresh evidence and a fresh VM/tablet probe. An
uncertain input is never replayed. An unverified RIGHT effect cannot be bypassed
by a later Draw, Recycle or Solve prediction. STOP remains cooperative and
cannot instantly interrupt an in-flight QMP request.

## Scope and verification

TriPeaks/Pyramid policy, shared post-game handling, QMP transport, calibration,
and exact-byte manual PNG saving are unchanged. Automatic capture remains QMP
temporary PNG, read, RGBA8 decode and normal cleanup; it is not file-free.
Nightly and rustfmt configuration are unchanged, and Charlie's committed
`rust-version = "1.101.0"` is preserved. No formatter is run. The separate
candidate review records actual compiler versions and validation results.

K13-K15 retain the relevant original pixels with provenance in the fixture
manifest. Tests exercise their targets, synthetic RIGHT replacement/refusal
cases and the real controller through deterministic I/O. They do not establish
live QEMU input effects. Beast checks must cover continuous STOP, RIGHT-to-Draw
and RIGHT-to-Recycle transitions, an actual recycle, the one-shot Solve result,
and existing-mode smoke regressions. Retain the first failure log and original
PNG; completion/restart support requires evidence from the resulting scene.
