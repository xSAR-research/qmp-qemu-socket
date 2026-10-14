# Pyramid and TriPeaks terminal transition correction

Candidate: qmp-qemu-socket 2.2.2-candidate.3.

## Observed failure

Charlie's session log qmp-qemu-socket-1791900631929.log records an initial
Pyramid OK-region match at (944,771), followed by a delivered click. The very
next New Game search succeeded at (863,862). However, the legacy shared
controller prioritised an earlier OK-region match at (944,810) and clicked it
instead. A later earlier-region match at (944,849) caused another click while
New Game was missing. That backward recovery reset the missing-control count.
The lookup finished with four searches, three recaptures and 7213.398 ms wall
time, despite finding New Game on its first search.

The two successive 39-pixel downward movements are consistent with captured
pointer pixels. This is an inference: the supplied log contains coordinates,
not the underlying Pyramid images. The control-priority defect is established
by the log and source regardless of which object supplied those black pixels.
An OK-region black run is not independent proof that Level Up appeared. A game
that does not award a level can proceed directly to New Game; difficulty alone
does not establish whether that happened in this run.

The failure message also promised a retained image, but the UI rejected the
latest publication as advisory after the worker became uncertain. Capturing
initially published diagnostic pixels; publishing the analysed post-game
observation then overwrote the diagnostic flag.

## Correction

- Search and accept the current expected menu control.
- After a menu click is delivered, retire that earlier stage as input authority.
  Black runs in its old region cannot cause another click or reset the current
  missing-control count.
- Prefer New Game at the optional Level Up step. Preserve separately recognised
  foreground TriPeaks Level Up context so underlying ink cannot bypass its modal.
- One missing-control observation permits one input-free fresh capture. A second
  consecutive miss stops. Independently recognised reward or gameplay recovery
  retains its existing separate policy.
- Publish unverified post-game observations as diagnostic frames, carrying their
  original mode/socket/generation and no actionable prediction. The normal UI
  context checks remain in force.

## TriPeaks Play evidence

The native screenshot supplied as
qmp-qemu-socket 261014 003540 TRIPEAKS-Failure-screen.png is preserved byte-for-byte
as tests/fixtures/tripeaks-play-20261014.png. Its SHA256 is
402d1499489505a82a70321af5ef988307a04bf65e34161cbea05918d06d9dde.

A direct Python pixel scan with the existing profile returns:

| Control | Match | Pixels visited | Rows visited |
| --- | --- | ---: | ---: |
| OK | (0,0) | 3775 | 151 |
| New Game | (0,0) | 3381 | 49 |
| Play | (680,905) | 710 | 36 |

This is native-image calibration evidence, not proof that the previous run
reached its Play-search stage. The screenshot shows Grandmaster selected; Easy
on the right belongs to the Daily Challenges panel. No search rectangle or
selected game difficulty is changed by this correction.

## Verification boundary

The attached Pyramid logs and supplied TriPeaks screenshot establish the input
evidence. Focused Rust regressions cover forward transitions, distractor pixels,
optional Level Up and diagnostic delivery. Refer to the delivery verification
record for checks actually executed; Rust compilation and tests are not claimed
as run in this workspace. Live acceptance remains with Charlie's local tests.
The existing xsar crate is reused unchanged.

## Code Analysis

The change removes backward menu-click authority from a purely positional
black-run match. It adds no new network or guest-input operation and does not
change the mathematical scanner, frame validation or configured bounds. QMP
uncertainty and cancellation still stop input. Diagnostic pixels preserve their
request context and have no actionable prediction; the UI's context rejection
policy is unchanged. Matching remains positional rather than word recognition,
so unrelated black pixels inside the currently expected region can still match.
