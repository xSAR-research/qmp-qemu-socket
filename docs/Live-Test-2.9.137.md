# Live test — combined 2.9.137

Date: 24 October 2026. Combined host qualification passed: 1401 tests, zero
failed/ignored, all strict gates and native release; fresh acceptance remains pending;
see [verification](Verification-2.9.137.md) and the
[active goal](Pyramid-Short-Term-Goal.md). Preserve older protected artifacts and
first-failure packets. Version text alone cannot distinguish the older timing-only
137 from the new combined binary: verify path, SHA-256, actual PID and startup log.

## Fresh product and game start

1. STOP and close the old controller when ready. Launch the newly protected combined
   candidate only after host qualification, with one input-owning controller.
2. Select the supported 1920×1080, 100% Classic white-face scene. Use fresh Recon
   and a fresh route; do not resume an old diagnostic or reconstructed test receipt.
3. For the retained Boards endgame or a matched equivalent, verify exact LEFT empty,
   RIGHT usable, the remaining stock/tableau pair, then the last King. Expect the
   correct terminal/next-board policy, no premature REDEAL and no repeated input.
4. Where a slow animation occurs, immediate observation may retry read-only within
   two seconds of the last whole-action ACK; settled actions must proceed promptly.
   MOVE remains 250 ms; Computer Vision's HALO-dependent timing remains unchanged.

## Preview, details and log

- At normal and constrained window sizes, check all four capture corners and aspect.
  Overlays and pointer reports must still map to native card coordinates. HiDPI
  and other actual desktop scaling remain separate from the isolated X11 check.
- RECON, Start continuous, View Log and STOP occupy one row; EXIT is at the right.
- Use a local invalid path if a guest-free failure is needed. A new accepted failure
  opens once. Resize/scroll/select/copy the full frozen error, failed configuration,
  step/run/version/profile, packet result and session-log path. Close/reopen after
  STOP; later parameter edits must not replace that snapshot. A second identical
  failure is new; a late saved-packet callback updates without surprise reopening.
- View Log opens a separate resizable live window. Observe logging while work
  continues; select/copy without STOP. Copy Output includes the full disk session
  log, while the visible tail remains bounded. Clear Output keeps its idle-only gate.

## Colour roles and refusal bounds

Test the genuine Xbox/Classic artwork with fresh supported evidence when available.
The retained intact image and synthetic recolours are offline proof only; they do
not establish every empty-slot, modal, endgame or live state on that artwork.
Expect highlighted/partial/occluded or ambiguous paper to remain Unknown, never
Empty by failed recognition. Enabled REDEAL still requires complete widget/count,
exact position, legal stock accounting and fresh probe/STOP checks. Unknown or
disabled controls must not trigger a click. Custom JSON and CV semantics remain.

## Exit and evidence

Check EXIT and the native X at idle, then during cooperative work when appropriate.
SIGINT/SIGTERM must latch shutdown and finish worker teardown before exit, including
when the window is hidden. Existing QMP release-only recovery may finish; no cleanup
click is authorised. Blocking I/O and cooperative cancellation add delay, so instant
exit is not promised. SIGKILL cannot be caught and cannot guarantee graceful cleanup.

Retain the correlated actual process, existing logs and error packet; no extra
ordinary-gameplay screenshot recording is requested. Stop at the first uncertain
delivery/effect, unsupported scene or context loss. Record observed outcomes and
Charlie's acceptance; host tests and isolated GUI checks do not supply that acceptance.
