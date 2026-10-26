# Live test — 2.9.135 partial-route continuation and MOVE pacing

Date: 24 October 2026. Project: native GUI controller; source correction on `main`.
Host qualification passed **1364 tests**, with zero failures or ignored tests;
see [Verification-2.9.135.md](Verification-2.9.135.md).
This brief creates no live acceptance and does not resume the consumed 2.9.133 run.

## What to test

- On a fresh supported Score challenge, begin with **Shortest route** in the
  target-aware **Continuous** mode. If exhaustive search finds no winning route,
  confirm the same authenticated board switches to **Maximum card matches**.
- At the verified last match of a partial route, confirm the controller continues
  through the remaining stock. Each DRAW/MOVE and RECYCLE must have an actual
  observed effect. Once all legal passes are exhausted and the one-deal REDEAL
  control is positively enabled, expect one REDEAL and fresh next-board Recon.
- On the next board, confirm the operating-state/log/UI shows **Shortest route**
  before a new search. A separately established no-win result may fall back again.
  Kings challenges deliberately retain their Maximum default.
- Check repeated computed-route MOVE effects with the configured **250 ms** wait,
  including residual stock draining. Retain actual configured/elapsed wait rows
  and the first refusal if any capture arrives before a stable effect. A passing
  host timer check cannot establish reliable guest timing.
- Computer Vision is a separate control check: its DRAW/MOVE still waits for the
  Solver's next HALO using the unchanged **1,000 ms** setting.

If testing an unsolvable Boards challenge, the existing policy may drain stock
directly rather than optimise partial matches. Confirm remaining DRAWs, both
available RECYCLEs and one enabled REDEAL; do not treat this as a Score fallback
test or as proof of full M11/M14 acceptance.

## Configuration

No runtime configuration field, file format, dependency or calibration changed.
The compiled MOVE delay is separate from the Computer Vision setting. Planned
RECYCLE stays **1,000 ms**; residual stock-drain RECYCLE and Recon D stay **500 ms**;
ordinary card and final-observation policies are unchanged.

Use the supported native 1920×1080, 100% display/text scale and CLASSIC BLUE scene.
Retain the selected socket, Pyramid mode, A*/computed-route strategy and actual
challenge-target binding. The old consumed receipt or a restoration flag grants
no new input authority. Unsupported artwork or REDEAL variants must refuse.

## Candidate and launch

Local PATH installation is skipped: this GUI trial uses a separately protected
versioned candidate. Protected 2.9.133/2.9.134 fallbacks must remain intact.
The private `redeal-move-20261024` evidence bundle records the release artifact,
protected copy, SHA-256, version and exact source-input hashes. Its protected
`native/qmp-qemu-socket-2.9.135` SHA-256 is
`427518c718af25a10bc4dcc4ec22857184c6afe246119ca5d3e9f25b36fcfa20`.

Before the trial, establish that the launched process resolves to that protected
2.9.135 executable, with its actual PID and startup log. A rebuild or an updated
manifest does not replace an already-running controller. Use one controller with
input authority; select the fresh supported challenge before START.

CLI table: not applicable to this GUI test. Derive any optional launch flags from
the qualified candidate's actual `--help`; this brief requires no optional flag.
The actual help reports that application startup clears the existing GAME WIN
image directory. Preserve any retained images before a fresh launch; this trial
does not require `--image` or extra screenshot recording.

## Procedure and evidence

1. Establish candidate provenance and a fresh supported Score scene with an unmet
   target. Record selected mode and stop condition alongside observed challenge.
2. Start one normal authorised Continuous run using Shortest route. A scene that
   solves fully does not exercise this partial-route regression.
3. Observe fallback, exact partial endpoint, stock effects, remaining RECYCLE
   accounting, enabled REDEAL, fresh acquisition/restoration and the next-board
   Shortest method. Keep the correlated existing session log and diagnostics.
4. Use the existing STOP when sufficient evidence is available. STOP may occur
   during an in-flight operation; no instantaneous pause between phases is promised.
5. On the first uncertainty/refusal/context loss, stop and preserve its packet.
   Do not repeat the last match, retry REDEAL, add cleanup clicks or resume an old
   consumed route. Record actual wait rows and the point where continuation stopped.
6. Charlie records the observed outcome and acceptance. If the scene never enters
   the partial fallback, or reaches its target before REDEAL is needed, that case
   does not establish this goal's live sequence.

No extra ordinary-gameplay screenshot recording is requested. Existing logs,
retained entry/error evidence and normal diagnostics suffice. The separate
below-target full-clear case, rewards/target stopping, Fours recognition and full
milestone acceptance retain their existing open status.
