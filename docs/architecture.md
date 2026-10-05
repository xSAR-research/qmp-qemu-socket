# Architecture and control flow

## Current boundary

Version 1.3.0 candidate 1 starts from accepted and pushed v1.2.6 candidate 11
at `146fe173f6e123a92e2eadf518523715d042cd66`. Charlie reported a successful
replay of the Undo All overlap case and a complete Klondike GAME WIN sequence
before promotion. Free Cell is now selectable for read-only calibration;
it cannot send gameplay, Solver or terminal input until its independent native
detector and scene policy are evidenced. The later candidate-history sections
retain their original decisions, including superseded card-effect policies.


## Component boundaries

| **Module** | **Responsibility** |
| --- | --- |
| `app.rs` | egui controls, latest-frame preview, concise status and bounded visible output |
| `session_log.rs` | private session-file creation, append and complete-history reads |
| `game.rs` | small game/profile boundary and typed actions |
| `tripeaks.rs` | validated TriPeaks profile assembly |
| `klondike.rs` | live Klondike dynamic sources, stock/recycle, RIGHT fan, Solve and independent win evidence; retired effect diagnostics are test-only |
| `klondike_terminal.rs` | Klondike completed-game scenes, terminal controls and fresh Draw 1 deal recognition |
| `freecell.rs` | independent read-only Free Cell profile, native frame validation and calibration-only status |
| `pyramid.rs` | Pyramid targets, fixed halo probes, priority and action-effect evidence |
| `parameters.rs` | fixed geometry, colour values, delays, limits and release label |
| `capture.rs` | decoded immutable frame representation |
| `snapshot.rs` | collision-safe PNG naming and saving the original captured bytes |
| `cards.rs` | calibrated card regions and test-only geometry validation |
| `detector.rs` | bounded pixel and scene discriminators |
| `tracker.rs` | profile-driven HALO and row observation |
| `stepper.rs` | one typed action plan and result validation |
| `geometry.rs` | guest-pixel and QMP coordinate conversion |
| `qmp.rs` | allow-listed QMP client and non-idempotent input delivery |
| `worker.rs` | worker commands/events, capture, guarded execution, timing and cancellation |
| `worker/post_game.rs` | shared score, Level Up, New Game, Play and Solver progression |
| `worker/pyramid_execution.rs` | Pyramid effect, fresh-pair continuation and redeal checks |
| `worker/klondike_execution.rs` | Klondike execution, bounded recovery, independent completion and continuous terminal progression |
| `worker/tests.rs` | worker regression cases, including transition and cancellation guards |

---

`GameMode` selects a static `GameProfile`. QMP, full-frame capture, cancellation,
logging and exact-byte PNG saving are shared facilities. TriPeaks and Pyramid
share their guarded gameplay and post-game controller.
Their target detection and effect rules remain separate: TriPeaks requires one
unique highlight; Pyramid chooses the first eligible target in its ordered
31-slot profile. Typed action identities must belong to the selected mode.
Shared Solver, Undo All and Undo toolbar geometry has explicit `SHARED_*`
names. Displaying an overlay alone never grants input authority.

## Normal guarded run

```mermaid
flowchart TD
    A["Fresh capture"] --> B{"Mode permits input?"}
    B -->|"Free Cell calibration"| C["Publish read-only preview"]
    B -->|"Active mode"| D["Apply mode target policy"]
    D --> E["Probe QMP and send once"]
    E --> F["Settle and capture result"]
    F --> G["Apply mode result policy"]
    G -->|"Fresh action eligible"| D
    G -->|"Unsupported or uncertain"| H["Stop with latest frame"]
```

TriPeaks and Pyramid freshly reproduce the approved initial prediction and keep
their accepted result-verification policies. An effect-verified result or
Pyramid's qualified repeated Left/Right HALO pair becomes the next planning
frame. Klondike's preview is advisory: each fresh supported scene and canonical
Solver recommendation authorises one operation, then editable settle and fresh
capture. Card-rank, source/recipient matching and changed-pixel proof do not gate
Klondike play. Acknowledged delivery is logged with its effect explicitly
unproven; independent positive evidence is still required for game completion.
Every active input checks STOP and freshly probes VM/current absolute tablet.

For N ordinary actions this yields one initial capture plus N result captures.
Bounded recovery and board/game transitions may require additional sparse
observations.

## Input ownership

Only the worker module owns a connected `QmpClient` in the application control flow.
Its post-game and Pyramid execution submodules borrow that connection.
Ordinary gameplay operations require a validated `StepPlan`; bounded board and
post-game transitions have separate worker paths with stage checks and retry
limits. Read-only capture, detector, UI and profile code cannot send input.

Each enabled action has:

- stable semantic identity;
- exactly one `InputOperation`;
- an animation class and settle delay;
- a mode-owned result policy; effect regions and thresholds apply only where
  that mode uses effect verification;
- a repeat-target policy.

A QMP error after delivery begins makes the result uncertain. The operation is
not automatically retried.

## Capture and connection lifetime

The worker owns a QMP connection for one read-only capture or snapshot request,
or for the duration of an active guarded run. A run reuses its connection
across actions, result captures and post-game transitions. It drops the client
when the request returns. STOP is cooperative: a pending request does not
interrupt an in-flight socket operation, and release handling can complete
before the worker exits.

Every capture currently follows the same file-backed path: reserve a private
temporary PNG beside the socket, ask QEMU to write it, read its bytes, decode
RGBA, and attempt file removal during normal cleanup. Detectors consume the
decoded frame. Neither the QMP socket nor `CapturedFrame` is a native-framebuffer
transport. The unused `FrameSource` abstraction and file-reading wrapper were
removed for 1.1.0; no replacement capture backend was introduced.

Manual Capture PNG additionally keeps the fresh original PNG bytes and the
decoded frame in the dialog until Save, Recapture or Cancel. Save writes the
stored bytes without another capture. Preview scaling never changes the saved
PNG. File-free automatic capture remains an open requirement; it has not been
implemented or benchmarked.

## Mode availability

| **Capability** | **TriPeaks** | **Pyramid** | **Klondike** | **Free Cell** |
| --- | --- | --- | --- | --- |
| Capture Frame / Capture PNG | Read-only | Read-only | Read-only | Read-only calibration |
| HALO selection | Unique target | First eligible of 31 fixed probes | Mode-owned ordered source classes | Not calibrated |
| Single Step / Multiple Steps | Guarded | Guarded | Fresh Solver-led actions | Input disabled |
| Ordinary input | D or tableau click | D for highlighted MOVE/Recycle; card/pile click | D for Draw 1; recycle/source click | None |
| Result policy | Existing effect checks | Effect checks or qualified fresh pair | Fresh valid recommendation; effect unproven | Calibration-only status |
| Editable settle/reobserve | Draw/tableau and late-HALO intervals | 1000/2000/1000 ms defaults | 750/750/1000 ms defaults | Future 750/1000 ms settings |
| Board/game transitions | Shared three-board policy | Shared three-board policy | Independent one-board policy | No automation enabled |

Pyramid's semantic order is `Move`, `Left`, `Right`, then 28 card identities
from row 7 left-to-right upward to the row-1 apex. It does not use TriPeaks row
exposure or unique-target policy. A highlighted pair needs one click; no
second click is queued.

Pyramid keeps `clicked_target_slots[31]` for the current board. Only a clicked
`Card` receives a consumed mark, and only after its face is positively observed
as removed. A highlight disappearing or arbitrary changed pixels are
insufficient. The unclicked partner receives no consumed mark. `Move`, `Left`
and `Right` are always eligible for later scans, including consecutive actions,
when their effect has been verified. A settled fresh Left/Right pair can also
authorise continuation when its two replacement cards look identical; that
route neither retires a card nor advances completion counters.

Unsettled effects receive a bounded number of fresh observations, spaced by
the snapshotted Pyramid repeat delay (default 1000 ms), without resending the gameplay click. A halo-free ordinary result
is recaptured without Solver input. Only positively confirmed redeal
evidence followed by three settled halo-free new-board observations permits one Solver
re-activation on the new board. Fresh-pair continuation requires recognised
gameplay, known faces, a unique eligible Left/Right pair, remaining tableau and
no pending final-card/redeal phase. A halo alone never proves board/game
completion; an unresolved scene stops the run.

Redeal needs two consecutive gameplay captures containing restored cards and
`AnotherBoard` progress. Contradictory progress readings are logged with probe
counts and retried within the existing observation bound; no guest input is
sent while final-apex completion is pending. An early action effect does not
suppress later redeal recognition.

Confirmed redeal/new-game, an explicit progress reset, and changes of mode or
socket clear the Pyramid board state. Every new context requires a fresh scene
before guest input. See `pyramid-execution.md` for geometry and the remaining
live acceptance checks.

After a stopped run, read-only Capture Frame can encounter a new board while
the old board's consumed marks remain. Only when the fresh frame has a halo and
present card at a previously verified removed slot, and old-state analysis
found no eligible target, the worker clears the per-board record, resets the
advisory board count to unknown, and reuses that exact frame for planning. This
does not substitute for the automatic redeal checks described above.

The expanded detailed-output area has a fixed logical height. The preview
reserves that fixed space only while the log is expanded, so additional window
height increases the image area instead of stretching the log.

## Klondike boundary

This section records the earlier Klondike boundary and effect design. It is
historical; the current target cycle above and candidate-10/11 summaries below
supersede its effect, initial-preview and toolbar-exception requirements.

Klondike uses an independent controller and no three-board progress probe.
Its dynamic target identity includes the highlighted source geometry. It follows
DRAW/RECYCLE, RIGHT HALO, RIGHT Solve, tableau bottom-up and SUIT priority, using a key for ordinary draw
and a single source click for transfers/recycle. Solid-source validation rejects
the dark dashed landing guide. Source blocks can change height after every move.
The outline search includes the evidenced overlap with the toolbar. Interior
crossbars cannot close a source while its exterior rails continue below.
Input and material-effect bounds remain above the toolbar even when the visual
source outline extends into it. The measured deeper run closes at rows988–990;
the lower-edge check excludes only Undo All's observed x1308..1336,y988..991
overlap, requiring all remaining edge pixels and the existing complete-outline
guards. Face evidence is sampled above row 947.

Tableau effect verification accepts white card paper changing to the measured
green felt dimmed beneath a destination guide. That removal evidence is counted
separately from bright replacement pixels and still requires material change at
an independent destination. Neutral grey, opaque black and a fresh source HALO
alone do not establish removal. Same-column automatic reveal also receives a
bounded source-face replacement comparison, still requiring independent
destination change. Upper SUIT returns use the corresponding fixed card-corner
replacement proof or bidirectional printed-detail changes over stable bright
paper, still with independent material source and destination bounds. A dimmed
occupied foundation can prove a small red printed-content transition while its
guide and surrounding neutral paper remain stable; this is an alternative to
the unchanged ordinary destination threshold. These checks compare pixels without
recognising card ranks or suits.

Klondike supports finite gameplay limits and zero for continuous operation.
Every unresolved action has a bounded recovery allowance. One recognised no-HALO context can request
one Solver activation and an immediate capture, followed by bounded delayed
observations. A recovery never proves the preceding gameplay effect. Unknown
scenes and uncertain input stop. An unresolved ordinary card effect may continue
only from an independently supported fresh recommendation: source-removal or
replacement proof is required for a HALO target; a recognised next Solve control
has its own authority. The route is disabled after Solver refresh and excludes
Draw/recycle. Acknowledged actions consume finite budget slots, with strict
verification and continuation counters kept separate. It does not infer previous
effect or completion. Other unresolved results stop. The independently recognised
Solve button in RIGHT's place permits one auto-finish request followed by its
separate animation delay and up to 20 delayed read-only completion observations
at the editable Klondike re-observation interval. This reuses the existing
post-game bound; ordinary card recovery retains three delayed observations. No Solve
or uncertain gameplay input is retried. An initial changed preview returns a
fresh approved prediction for review without guest input; an exact approved
no-HALO match permits one independently guarded Solver activation. An unbounded
request continues from its fresh recovered target; finite requests publish it for
review. Uncertain input is never replayed.

One Klondike board is one game. Two consecutive fresh frames must show either
an intact active Solver banner with all 76 calibrated right-interior pixels gold
and zero black, or positively recognised completed-game artwork. Overlay or
ordinary background black absence is insufficient. Finite runs stop before
terminal input. Continuous runs alone may advance through the mode-owned
score-skip, Level Up OK, New Game, Draw 1 Play and fresh-board Solver sequence.
Each recognised control is clicked once, settled and freshly observed; missing
or unchanged stages receive bounded input-free recaptures. Unexpected recognised
stages stop. Only a fresh actionable new Solver board publishes
`KlondikeGameCompleted` and resumes gameplay. Shared three-board policies remain
separate. See `klondike-v1.2.3-candidate-5.md` for evidence and limitations.

A complete 180..190-pixel single-card outline with its normal lower proof corner
crossing the toolbar can use a
separate source proof with two full 28-by-44 visible patches. The normal upper
corner stays at source-top+5; the lower-right patch is anchored at proof-bottom−44.
Both stay above row 947 and require 50% white paper and at least 48 changes in each
print direction. The route additionally requires 512 material source changes.
It supplies source replacement for acknowledged, settled, supported fresh-HALO
continuation only; it does not enter complete-effect verification. The ordinary
clipped-corner refusal and existing complete-effect policy remain unchanged.
Stack expansion is handled by the observed pixel transition; no rank or suit is
decoded. In K39/K40 the generic destination count 1298 comes from the next HALO's
shading; the actual receiving foundation changes only 318 pixels. Neither is
promoted to complete-effect proof by this new route.

## State and bounds

- Frame size is locked to 1920x1080.
- Startup requests one read-only capture; previews are bound to the requesting
  game mode and QMP socket, then mapped to host physical pixels.
- Unknown visual progress remains `?/3` rather than being promoted from the
  session counter. Verified board completions publish the advisory counter
  immediately; full three-segment visual calibration remains pending.
- The main preview mailbox retains only the newest pending full-resolution
  frame. On an unverified post-action stop, the latest result frame is shown
  for inspection without an approved prediction; a fresh Capture Frame is
  required before another guarded action. A manual snapshot separately
  retains its original PNG and decoded frame.
- The visible log is bounded; the complete private session log remains on
  disk.
- TriPeaks/Pyramid Multi-Step defaults to continuous (`0`) and is STOP-cancellable.
- Klondike Multi-Step defaults to 0; 1–10000 bounds actions and 0 runs continuously.
- Free Cell future Multi-Step defaults to 0, with a finite range of 1–10000;
  calibration mode still rejects all guest input.
- Klondike Solve sends one request, then bounded read-only completion observations;
  terminal progression requires continuous authority and independent win proof.
- Transition retries and click attempts are bounded.
- Advisory session counters never override visual transition evidence.
- STOP and application exit detach locally; neither shuts down QEMU or Windows.

## Identity and host paths

`Cargo.toml` is the application name/version source. `main.rs` launches
`QmpQemuSocketApp`; parameters, logs, screenshots and temporary capture names
use the `qmp-qemu-socket` identity. References to Microsoft Solitaire and its
Solver control identify the guest application and its UI.

`QMP_SOCKET_PATH` overrides the socket path. Otherwise `XDG_RUNTIME_DIR` takes
precedence; the fallbacks are `QMP_RUNTIME_DIR`, an existing
`$HOME/tmp/qemu-runtime`, then the system temporary directory's `qemu-runtime`
subdirectory. The filename is `qmp-qemu-socket.sock`. These directories must
already be usable; the application does not create the QEMU listener.

Use an absolute socket path. The temporary PNG path is derived from its parent
directory and must also be absolute. Consequently the normal configuration does
not depend on the working directory of QEMU or the application. Both processes
must see and have access to the same host paths; a remote or isolated QEMU
filesystem is not supported by this capture design.

## Coordinate domains

| **Domain** | **Origin and units** | **Use** |
| --- | --- | --- |
| egui | top-left, logical points | Widget layout only |
| guest frame | top-left, physical pixels | ROIs, detection, tracking and preview overlays |
| QMP absolute | top-left, integer `0..=32767` per axis | Guest pointer input |

Host desktop coordinates never enter the QMP transform.

Candidate2 keeps unsupported post-action recapture inside Klondike's existing
shared result-observation budget. It retains the original action frame and plan,
skips effect/input analysis on unsupported scenes, and evaluates only fresh
supported results. Source-supported continuation does not upgrade the preceding
move to verified. Solve diagnostic counts have no input or completion authority.

## v1.2.5 correction boundary

The Solve recogniser retains empty-stock, stable interior, complete glyph and
scene/priority guards. Animated outer-ring template agreement is diagnostic only.
A bottom single-card source with uneven print changes can support continuation
only through the bounded stable-paper and bidirectional-print checks; strict
complete-effect verification is unchanged. This does not decode ranks or suits.
Level Up label variants match an entire coherent signature at the evidenced
nominal or two-pixel horizontal offset, never a mixture of individual probes.
Terminal evidence logs identify named guard failures on the same fresh frame.
All input, effect-proof and completion authorities remain separate.

## v1.2.6 observation and terminal guards

A strong but incomplete Solve interior can suspend lower-priority tableau/SUIT
planning for one bounded context of at most three input-free captures. Existing
DRAW/RIGHT priority, strict Solve availability, previous effect proof and action
budgets remain separate. A first approved actionable preview changed by waiting
requires review before input. Continuous recovered/new-game targets retain their
existing fresh-target authority. No-HALO disappearance cannot trigger speculative
Solver input in this pending context.

K53 moves one Home body colour probe beyond the measured post-OK cursor edge.
The six-of-six body policy, modal/background/lettering checks and actual New Game
click point remain unchanged. No cursor position alone establishes a scene.


Version 1.2.6 candidate 2 is based on actual package 1.2.5 source at f9167a0.
The previous candidate was not installed because its exact HEAD guard refused
both preview and apply. The measured New Game and pending-Solve changes are
reapplied as the candidate patch; f9167a0 is the base authority. A read-only
installer verification requires complete candidate source before Cargo or launch
commands. Sparse hearts source evidence remains unresolved and does not change
Klondike effect/continuation authority.

## Klondike clipped-card reflow correction

Candidate 3 keeps the narrow Undo All edge context and independently compares
qualified clipped single-card faces relative to their fresh source outlines.
An ordinary replacement additionally needs a unique calibrated face seam and
independent recipient print. The geometry descriptor authorises no input.
Aligned print/paper/material evidence feeds `source_replaced` only; ordinary
complete effect, completion and shared controller authority remain unchanged.
See [candidate 3](klondike-v1.2.6-candidate-3.md).

The recorded deeper destination guide has a separate positive-felt continuation
rule for complete single cards. It requires formerly neutral paper, opposed
source halves and independent non-source changes; full effect remains unverified.
Disjoint deep and existing neutral dimmed-felt ranges are counted once per pixel.
The lower scene probe retains 196 samples and its 95% felt threshold in the
measured clear gap, preserving the independent lower-board context.


## Klondike SUIT-return continuation

A fixed SUIT card can be replaced by similar artwork after a valid return to the
tableau. A separate mode-owned witness combines opposing signed corner changes,
stable paper and newly received print matching the old source. A uniquely found
new neutral card seam selects receiving geometry before artwork is compared.
This witness supplies `source_replaced` only; the previous full effect remains
unverified. The shared controller then uses the fresh canonical target under its
existing action budget, STOP and non-replay rules. See
[candidate 4](klondike-v1.2.6-candidate-4.md).


The Pro Level Up artwork is a separate complete terminal-layout signature. It
returns the existing typed LevelUp stage only with positive completed-board and
button evidence; the existing OK point remains inside the measured button. No
shared three-board policy, additional input retry or timing inference is introduced.


K68/K69 extend outline recognition through exclusive row 998. Only K68's
column-7 rails may use the existing shadow-gold predicate inside the measured
toolbar band, supported by two ordinary paired rail rows on each side. Complete
top/lower edges, terminated rails, above-toolbar paper and the 604-pixel height
limit remain. K69 needs the scan extension alone. Input/effect bounds are not
extended into the toolbar.


## Tableau source covered by its next landing guide

Candidate 5 adds a local Klondike continuation witness for a tableau-to-SUIT
transfer followed by a guide over the old source. The unique source header and
unique changed fixed receiver are selected from geometry/paper independently
of source-artwork matching. Positive source-side transitions and newly received
old source ink are both mandatory. Visible source ink stops above row 947;
no clipped corner is complete. A canonical different fresh target supplies
context only. The existing controller reports one continued action, keeps full
effect/completion separate, and preserves all probe, STOP and uncertainty gates.


## Shaded source borders and fresh-deal recognition

K75 supplies a separate measured column-4 toolbar shade. Existing positive
shadow-gold may bridge only its bounded exterior-rail band, with ordinary paired
bracketing, and its two measured lower-edge samples. No missing, black or neutral
sample is omitted. Complete lower/top edges, terminated rails and above-toolbar
card paper remain mandatory; effect pixels remain above row 947.

After an acknowledged Play, the terminal classifier requires a positively
recognised initial Klondike deal with Solver inactive before returning the
existing SolverReady stage. Card-paper perimeter geometry replaces the old
interior-area test, which rejected the illustrated King-diamonds in K77. The
existing terminal controller sends one Solver click and captures fresh evidence;
an arbitrary no-HALO board does not become SolverReady.


## Historical candidate 6 Solver-led policy

Charlie explicitly selected the fresh-HALO cycle over the earlier source/effect
witnesses described above. The initial preview is advisory. The worker freshly
classifies the selected Klondike scene and canonical recommendation, probes the
VM/tablet and STOP, sends one logical action, waits the editable settle and captures
again. Every acknowledged gameplay action consumes one budget slot; a finite run
stops after its last fresh result. Repeated fresh HALOs remain eligible as new
logical actions without proving a previous transfer.

No live card-pixel comparison or recipient/rank matching gates this cycle. The
previous effect analyser and its regressions are test-only diagnostic code. Worker
reports record Solver-directed actions and keep effects unproven; the UI does not
present the unmeasured pixel field as a zero-pixel effect. Missing HALOs receive
bounded input-free observations, independent completion checks, and at most one
Solver refresh on a positively recognised gameplay scene. Solve/win/restart retain
their independent evidence and existing input/uncertainty/cancellation gates.

## Candidate 10 runtime policy retained by candidate 11

This section supersedes historical card-effect and terminal-artwork descriptions
above for current Klondike execution. Klondike play follows fresh canonical
Solver targets; card differences remain diagnostic-only. Independent two-frame
win evidence establishes the terminal context. The controller then checks only
the expected local OK, New Game or Play button, preserving existing measured
clicks, holds, settles, VM/tablet probes, STOP and uncertain-delivery refusal.
Printed-caption contrast supplies readiness without exact glyph or medal matching.
The ordered stage itself cannot establish a win. The next deal still requires
positive mode-owned board evidence before Solver or gameplay input.

Pyramid Move/Recycle uses the confirmed D shortcut only when its fixed four-pixel
HALO probe passes. The other 30 targets retain clicks. TriPeaks's ordinary no-HALO
recovery owns three delayed observations before one Solver refresh and three
afterward; its interval is session-editable and immutable per run. This does not
change existing effect verification or independently authorised redeal handling.

The Undo All overlay is represented by one measured bounded envelope, supported
by red icon evidence and unoccluded connected source borders. It cannot substitute
for source paper, closed top, exterior rail termination or ordinary evidence
outside the envelope. No new card-effect proof or blanket toolbar exclusion exists.

Detailed output retains 2000 String entries, evicting old entries on insertion,
and rolls the visible tail over every three completed games. Session-file history
is complete. Output text sizing changes rendering only; copied text is unchanged.

## Candidate 11 tableau boundary

This section supersedes the historical tableau toolbar/icon exceptions above.
The live tableau detector excludes all rows at or below the measured game
toolbar boundary, 947. Ordinary closed source outlines above that boundary keep
their existing interpretation. A source clipped at the boundary requires its
visible closed top, connected opposing gold rails and card paper above the
boundary. It reports the observed exclusive bottom, 947, and retains the upper
source click at top plus 40; no hidden geometry is inferred.

The frame remains full size. Upper piles, Solver and terminal controls keep
separate recognition areas. Current source selection and worker regressions use
the live detector. Retired card-effect tests use a named test-only historical
classifier for their original full-source measurements, without contributing
input authority to the application. Candidate 10's known-win terminal sequence,
Pyramid D shortcut and TriPeaks delayed recovery remain unchanged. See
[candidate 11](klondike-v1.2.6-candidate-11.md).


## Free Cell read-only foundation

The agreed board names are CELL 1–4 for the upper-left temporary slots, PLAY
1–8 for the tableau columns and SUIT 1–4 for the upper-right foundations.
Numbering is left to right. These names document the next calibration slice;
there are no executable Free Cell source actions or click constructors yet.

Free Cell owns no live scene discriminator or HALO selector. Valid native
1920-by-1080 frames publish calibration-only status; malformed storage or other
dimensions are rejected. The independent profile has no actionable targets.
The worker refuses Free Cell execution before guest input; UI capture remains
read-only. Shared capture retains the full frame and original manual PNG bytes.

The intended next slice is one fresh supported CELL or PLAY HALO, one source
click, editable settle including automatic SUIT transfers, then fresh capture.
It does not require rank recognition or card-pixel move proof. The future PLAY
detector must keep toolbar pixels outside its source region; upper slots and
terminal controls need their own evidence. There is no Draw, Recycle or Solve
operation to inherit. Independent scene/target and one-board completion evidence
must precede input or deterministic terminal progression. The current candidate
adds only read-only foundations and future Params values. See
[Free Cell candidate notes](freecell-v1.3.0-candidate-1.md).
