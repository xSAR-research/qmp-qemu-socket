# Spider v1.4.0, candidate 4

## Result and exact base

This candidate uses the pushed v1.4.0 candidate 3 base,
`4b73986c4e1e540c947455ff31ff6ee194d197e0`. The package version remains
1.4.0 and the runtime label is candidate 4. Candidate 3's Free Cell visible
side-margin clipped-source check and Spider's higher local LEVEL UP OK check
are retained. The original and 19-pixel-higher OK positions, optional LEVEL UP
branch and deterministic one-shot terminal flow remain supported.

The new Spider correction moves the pointer away after an acknowledged source
click and removes active-Solver refresh input. Source recognition thresholds,
click points and all timing defaults remain unchanged. Other modes retain
their accepted execution policies.

## Cursor-overlap evidence and limit

The reported source was a Queen outline at rows 145..349, clicked at
(1045, 309). After the move, the next Jack source outline occupied rows
123..327; the pointer therefore remained over its required lower crossbar.
Charlie reproduced the failure four times and confirmed the pointer position.
The reported runtime used a configured 2000 ms re-observation interval.

The exact rejected runtime PNG is unavailable. On the supplied frames,
placing the supplied cursor at the confirmed click point obscures 12–16 gold
pixels where the lower-crossbar test requires 95 of 103 samples. The simulated
frame reproduces `NoHighlight`; moving the cursor away restores recognition.
This supports cursor obstruction as the cause, but it is a simulation from
supplied evidence rather than byte-identical reproduction of the missing
runtime frame. It establishes no card-transfer effect or card identity rule.

The park point is derived from the supplied full 33-by-46 cursor: its bounding
box lies on clear felt at (20, 500) across all nineteen supplied gameplay
fixtures, outside scene and source probes. Placing that cursor at (82, 362)
inside the existing felt probe leaves only 88.89% felt against its unchanged
90% requirement, so that point cannot safely preserve scene recognition.

## Source action and observation policy

For a fresh canonical PLAY source, the worker sends the existing source click.
Only after the button release is acknowledged does it send movement-only
parking at the measured clear-felt point (20, 500), outside scene and source
probes. The existing editable card/run settle follows the park,
then a fresh full-frame capture supplies the next recommendation.

The shared QmpClient helper checks guest pixel coordinates with the existing
pixel-to-absolute-axis conversion. It sends one acknowledged QMP command
containing two absolute-axis events, with no button/key event and no invented
delay. Spider owns the policy to call it. Auxiliary movement is counted
separately from the source click: one source still consumes one logical action
slot. DRAW sends D and uses its existing independent wait without parking.
Solver and terminal actions do not park.

Active Spider Solver is disabled, so no refresh click is sent. A missing HALO
receives bounded input-free recaptures and independent completion checks.
Supported inactive Solver retains one start wait and fresh capture before
one activation per unresolved context. After Play, the existing game-start
wait runs once before observing the new board. Activation does not reset the
delayed-observation allowance. Missing HALO does not authorise
DRAW, and missing stock does not establish a win. Unknown scenes, STOP and
uncertain source, park, Solver or terminal delivery retain bounded stopping
and no input replay.

Initial, read-only and manual Capture PNG/Recapture/Save paths send zero input.
Pointer parking belongs only to active Spider source-click execution; it is
not a capture prerequisite. Full-frame capture, temporary-file handling and
exact original-byte PNG saving remain unchanged.

## Preserved scope

Card/run, DRAW, re-observation and game-start defaults remain
1250/2000/1000/3000 ms. All four settings remain independently editable from
0 to 5000 ms and frozen per run. The reported 2000 ms observation setting does
not replace the 1000 ms default. Finite budgets, continuous 0 and the bounded
observation allowance stay unchanged.

No gold threshold is loosened, no missing outline is inferred, and no card,
destination or changed-pixel effect matching is added. Y947 remains the
exclusive source read/click boundary. Candidate 3 terminal corrections and
all Free Cell, Klondike, Pyramid and TriPeaks policies are retained. Dependencies,
edition 2024, minimum Rust 1.101.0, floating nightly, formatting configuration
and requested source spacing remain. No formatter or warning suppression is
introduced.

## Validation and Beast acceptance

Rust/Cargo are unavailable in this workspace. Native compilation, tests,
Clippy and live QMP have not been run here; their results remain pending.
The cursor simulation and source review do not substitute for these checks.
The delivery review records only checks actually executed for this candidate.

Run the supplied Beast checks, stopping at the first failure. Confirm the
installed source and launched startup label are v1.4.0, candidate 4. Exercise
the overlapping Queen-to-Jack situation and inspect the click-release/park/
settle/capture order, one gameplay slot and separate auxiliary command/event
counts. Check DRAW has no park, active no-HALO Solver has zero refresh clicks,
and inactive Solver gets one start wait/fresh capture/activation. Verify STOP
and uncertain park delivery do not replay input; preserve a fresh frame and
complete log for any miss.

Verify initial/read-only/manual captures send zero input and saved PNG bytes
remain exact. Retain FC20/FC21 margin checks, both Spider OK positions,
GAME WIN with and without LEVEL UP, restart and earlier-mode behaviour. This
delivery remains at Gate 2; Charlie verifies before committing and pushing.

## Code Analysis

- **Threats:** pointer obstruction denied a fresh source without authorising
  unsafe input. The park is attached only to an acknowledged Spider source
  click and targets measured clear felt outside scene/source probes. It
  supplies no new card, target or completion authority; active Solver refresh
  is removed.
- **Memory safety:** the helper reuses checked integer coordinate conversion
  and existing QMP JSON delivery. No unsafe code, image allocation, frame
  ownership or decoding change is required. Native verification is pending.
- **Bounds:** the park point and supplied cursor bounding box lie within the
  native 1920-by-1080 display, outside scene/source probes. Source geometry,
  Y947, gold thresholds, editable timings and action/observation budgets
  remain unchanged. Auxiliary movement is separately counted and never adds a
  gameplay slot.
- **Failures:** release must be acknowledged before parking. Invalid
  coordinates, STOP, unsupported scenes or uncertain source/park/Solver/
  terminal delivery stop without replay. Existing cancellable settling and
  bounded input-free recaptures remain; no additional wait is introduced.
