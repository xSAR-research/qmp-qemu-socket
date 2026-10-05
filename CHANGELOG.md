# Changelog

## v1.3.0 — candidate 4

- Enable evidenced toolbar-clipped Free Cell PLAY sources using only visible
  source geometry and card paper above Y=947; never infer the hidden lower edge
  or read toolbar pixels for source authority. Connected runs remain one click.
- Add an independent editable game-start wait, default 1000 ms as confirmed by
  Charlie, followed by fresh capture before inactive Solver activation.
- Restore one bounded Solver refresh for a supported no-HALO board after an
  input-free re-observation and independent GAME WIN check. Report observed
  Solver state; uncertain input is not replayed.
- Add original FC14/FC16/FC17 evidence and focused regression coverage. Preserve
  candidate 3 optional LEVEL UP, shared capture and accepted earlier modes.
- Pin the guarded installer to pushed candidate 3 at `676afb477495a166a67132a5735b2347b343585d`.

## v1.3.0 — candidate 3

- Make Free Cell LEVEL UP optional after score counting. Each fresh frame checks
  the local OK and New Game controls; New Game can proceed directly when no
  level was earned. Ambiguous or absent controls do not authorise input.
- Preserve one-shot terminal clicks, bounded input-free observations, STOP and
  uncertain-input non-replay; New Game → Play → fresh board remains ordered.
- Correct the test-only constant RGBA chunk loop to `as_chunks_mut::<4>()`,
  addressing the reported Clippy lint without suppressing warnings.
- Add byte-identical FC15 evidence and focused optional-LEVEL-UP regressions.
- Pin the guarded installer to pushed candidate 2 at `656881ee3f906347bd6f5665c733892621b1cc98`.
  Runtime gameplay detectors, other modes, timings and capture are unchanged.

## v1.3.0 — candidate 2

- Enable the approved Free Cell Solver-led slice from original FC01–FC13 PNGs:
  CELL sources first, then PLAY sources bottom-up, with one bottom-card click
  per solid highlighted block. Dashed landing guides are not actionable.
- Observe automatic SUIT transfers without card identity, changed-pixel proof
  or source-position comparison. Activate visibly inactive Solver once; an
  active Solver with no HALO receives bounded input-free captures.
- Add an editable delayed-observation allowance, initially 20, bounded 1–100.
  Preserve independent 750/1000 ms Free Cell timings and continuous budget 0.
- Recognise independent one-board GAME WIN entry from the score/skip panel or
  completed-game New Game panel. Continuous play follows expected local
  OK → New Game → Play controls, fresh board and Solver activation. Finite runs
  do not restart. Level, medal, fireworks and surrounding panel colours do not
  gate these expected controls.
- Keep source probes and clicks above Y=947. Optional FC14 is absent, so a run
  with no complete visible lower outline remains unsupported. No SUIT-source,
  Draw, Recycle or Solve input is enabled.
- Preserve existing modes, STOP, uncertain-input non-replay and exact-byte PNG
  saving. Pin the installer to `44727193d4e620a749925e9cc46c9d9d4df1700f`.
- Native Rust validation is unavailable in the delivery workspace; the review
  distinguishes pixel rehearsals and installer checks from unrun Rust tests.

## v1.3.0 — candidate 1

- Add a selectable read-only Free Cell calibration mode and independent profile.
  Agree CELL, PLAY and SUIT names for future calibration. No Free Cell HALO detector,
  gameplay input, Solver activation or completion/restart automation is enabled.
- Add independent future Free Cell settle/re-observation settings, defaulting
  to 750/1000 ms, and a zero continuous-action budget, with existing bounds.
  Editing these values does not authorise guest input.
- Preserve the accepted Klondike live Y<947 tableau boundary and Solver-led
  action cycle, Pyramid's highlighted MOVE/Recycle D shortcut, TriPeaks delayed
  no-HALO recovery, STOP and original-byte PNG saving.
- Consolidate current architecture and input contracts while retaining dated
  candidate evidence. Add the FC01–FC13 native screenshot intake checklist.
- Base the candidate on `146fe173f6e123a92e2eadf518523715d042cd66`. Charlie
  reported successful candidate-11 overlap recovery and a complete Klondike
  GAME WIN sequence before pushing that base on 4 October 2026.

## v1.2.6 — candidate 11

- End live Klondike tableau HALO detection before the measured guest toolbar
  boundary at Y=947. Undo All, Hint and other artwork below that row no longer
  participate in source recognition.
- Recognise a boundary-clipped source from its visible closed top, connected
  opposing rails and card paper. Keep one upper-card click for the whole block;
  record only its visible bounds rather than guessing its hidden lower edge.
- Add the supplied five-card source fixture K95 and live detector/controller
  checks. Retain historical pixel-effect diagnostics separately in tests; they
  do not gate gameplay or restore toolbar-dependent live detection.
- Preserve candidate 10's Pyramid/TriPeaks corrections, one-board restart
  sequence, timings, STOP and original-byte capture/save contract.
- Rebuild the guarded candidate package against pushed candidate 10 at
  `66c779920a1a57d183f07725943e881f689ed3c6`.

## v1.2.6 — candidate 10

- Pyramid's freshly highlighted MOVE/Recycle target sends D. Both operations
  share the confirmed shortcut; card and Left/Right targets retain their clicks.
- TriPeaks uses three delayed input-free no-HALO observations before one Solver
  recovery, then three further delayed observations before stopping. Its new
  Params interval defaults to 1000 ms and is frozen for each run.
- Once a one-board Klondike win is independently confirmed, expected OK,
  New Game and Play controls use local button readiness. Rank, medal, title,
  frame and fireworks signatures do not gate those ordered clicks.
- Handle the measured Undo All overlay with one bounded lower-border envelope
  and positive icon/source evidence, retaining ordinary border requirements
  outside that envelope. No card-pixel effect checks gate Klondike play.
- Add output-only text sizing while preserving Copy Output and existing
  2000-entry retention and three-game visible-log rollover.
- Retain candidate 9's Hint-shadow correction. Installation supports exact
  base or the complete pinned candidate 9 predecessor, with guarded rollback.

## v1.2.6 — candidate 9

- Recognise both measured Hint-shadow lower-edge positions in column four.
  Keep all 96 lower-edge samples positive, existing rail brackets, closed top,
  card interior, scene context and safe upper-card click.
- Add two complete uploaded PNG fixtures and source/controller regressions.
- Keep fresh-HALO execution, Solve, one-board completion/restart, timings and
  accepted TriPeaks/Pyramid behaviour. No live card-pixel effect check is added.
- Deliver complete affected files against pushed candidate 8 at `cafdd609`.

## v1.2.6 — candidate 8

- Measure the clear seven-pixel column-five/six scene gutter core for the three
  captured column-five source HALOs, preserving196 samples and the95% felt threshold.
  Source-outline, click, capture and HALO-driven execution policy are unchanged.
- Recognise the independently measured Master medal LEVEL UP layout using
  complete semantic/control/frame/completed-board evidence and the existing
  OK point. Preserve both earlier layouts and input timing; add diagnostics.
- Add seven original-byte native fixtures and adverse/controller regressions;
  restore originalK69 because its old sparse mask omitted part of the new probe.
- Deliver complete affected files against pushed candidate7 at exact449004d.
  No live card matching, formatter, warning suppression or other-mode change.

## v1.2.6 — candidate 7

- Derive Klondike source-height capacity from the existing rows 332..998 scan
  envelope instead of the earlier 604-pixel observed maximum. K80's complete
  ten-card source is 620 pixels tall and passes every other existing guard.
- Preserve scan coordinates, colour predicates, connected rails, closed edges,
  paper/scene checks and one upper-card click. No card-effect matcher or
  controller policy is added; Solve, one-board restart and other modes are unchanged.
- Add two byte-identical native PNG fixtures, detector boundary/adverse tests
  and finite-run controller coverage using fresh recommendations. Keep manual
  Undo evidence separate from unsaved automatic worker frames.
- Deliver complete affected files against committed candidate 6 at exact
  329d806; accept only that base or a complete installed candidate 7 state.

## v1.2.6 — candidate 6

- Follow fresh canonical Klondike Solver recommendations without source/recipient
  card matching. The preview is advisory; each acknowledged action consumes one
  budget slot, settles and captures anew. Keep previous effects explicitly unproven.
- Continue repeated fresh HALOs as new logical actions; finite Step Once stops
  after one action and fresh result. Missing HALOs use bounded read-only captures,
  independent completion and one scene-validated Solver refresh per context.
- Move the previous card-effect matcher to test-only diagnostics; retain its
  historical native/adverse tests without a live execution gate or warning allow.
- Recognise K75's thin column-4 HALO through the measured Hint toolbar shade.
  Require positive shadow-gold on the bounded rail band and two lower-edge
  samples, ordinary paired bracketing and the complete remaining outline.
- Recognise the fresh post-Play deal using positive card-paper perimeter
  geometry, preserving fresh-deal topology, inactive Solver and scene guards.
  The King-diamonds artwork no longer prevents the existing one Solver click.
- Add six original-byte PNG fixtures and native/adverse controller regressions.
  Keep the first no-HALO report separate from the earlier post-click source
  verification stall; its source thresholds no longer gate live Klondike play.
- Deliver against the same committed candidate-2 base at exact 7d869c2.
  Accept complete base, delivered candidate 5 or candidate 6 content states,
  preserving unrelated work and rollback to the actual backed-up before-state.


## v1.2.6 — candidate 5

- Add separate tableau-to-SUIT continuation when the old source becomes the
  next landing guide. Require independently located exposed header, opposed
  positive source strips and uniquely received old artwork in a bright SUIT face.
- Reject neutral guide whitening, retained old print, ambiguous receivers and
  incomplete geometry. Keep ordinary effect thresholds and clipped-corner refusal.
- Add four original-byte PNG fixtures and native/adverse controller regressions.
  Preserve the distinction between the first manual pair and its logged frames.
- Retain candidate 4's source, SUIT-return and Pro Level Up corrections; input,
  timing, Solve, one-board restart and accepted other-game policies are unchanged.
- Deliver against the same exact 7d869c2 base, accepting complete base or
  delivered candidate 4 contents and restoring the actual backed-up before-state.


## v1.2.6 — candidate 4

- Add separately logged SUIT-return continuation when nearly identical foundation
  faces hide most source changes inside the existing cursor exclusion. Require
  paired opposite-corner print, stable paper and independently located newly
  received tableau artwork. Complete action effect remains unverified.
- Recognise the separately evidenced Pro Level Up artwork with complete dialog
  and completed-board guards. Reuse the existing measured OK point and timing;
  the reported failure had sent no OK input.
- Recognise the newly evidenced column-3 and column-7 source borders through
  row 997; bound shadow-gold rail support to its measured column-7 band.
  Keep input/effect limits, complete outline guards and existing height limit.
- Add K65-K69 native regressions and adverse/controller cases; preserve ordinary
  material thresholds, input non-replay and the confirmed one-board restart flow.
- Deliver against committed candidate 2 at exact 7d869c2, accepting a complete
  base or installed candidate 3 r2. Back up and restore the actual before-state.


## v1.2.6 — candidate 3

- Packaging revision r2 targets committed candidate 2 at 7d869c2 after the
  previous f916-based package refused installation. Rebuild all base/content
  records and report expected/actual HEAD; Rust code and fixtures are unchanged.

- Recognise the measured long column-6 source with two positively shadowed gold
  edge samples beside Undo All, retaining the complete visible-edge requirement.
- Add bounded card-relative proof for clipped single-card stack reflow; retain
  material/printed/paper/cursor guards and log continuation separately from
  complete effect verification.
- Support the measured ordinary replacement face after a clipped-card transfer,
  using unique geometry and independent recipient print for continuation only.
- Add separate positive neutral-paper to chromatic-felt continuation for the
  queen and King transfers; combine disjoint measured shade ranges under the
  unchanged source/destination bounds, leaving full effect unverified.
- Reposition one lower scene probe inside the measured clear gutter; retain its
  196-pixel area, 95% felt requirement and all previous scene classifications.
- Add K54-K64 native regressions. Packaging r2 applies to committed candidate 2
  at exact base 7d869c2 and supports rollback to that base. Preserve confirmed
  GAME WIN flow.


## v1.2.6 candidate 2 — 3 October 2026

- Rebuild complete-file installer authority against f9167a0, whose package remains
  v1.2.5. Candidate 1 had refused installation after HEAD changed; later tests and
  the launched binary still used the old source.
- Include the prior measured New Game pointer-probe correction and bounded
  pending-Solve recapture policy, with a distinct candidate 2 runtime label.
- Preserve Charlie's committed README purpose edits. Add read-only installed
  candidate verification before build/test/run commands.
- Record the separate sparse hearts source failure and transient Undo evidence;
  no gameplay effect threshold is relaxed. Beast acceptance remains pending.

## 1.2.6 — 2026-10-03 (candidate 1; Beast verification pending)

- Move one Home button-body colour probe outside the measured pointer shadow
  after Level Up OK; preserve six required samples, tolerance and click points.
- Defer lower-priority tableau/SUIT input during a strongly recognised pending
  Solve face, using at most three editable delayed read-only captures.
  The original full Solve availability check remains the only click authority.
- Preserve fresh initial preview approval, action budgets, STOP and uncertain
  input handling; add native K53 and controlled adverse/controller regressions.
- Deliver complete files against pushed main
  8125c9df1dd87c7d8e9ed0e928fbb7fdd7bae82b with guarded preview/backups/rollback.

## 1.2.5 — 2026-10-03 (candidate 1; Beast verification pending)

- Recognise Solve from stable interior/lettering with empty-stock and scene guards,
  independently of its animated outer ring; retain full artwork diagnostics.
- Add source-supported continuation for the measured bottom 4-clubs transfer
  followed by a different tableau source, with stable paper/printed-detail guards.
  The prior complete effect remains unverified; generic thresholds are unchanged.
- Recognise the coherent two-pixel Level Up label shift and log terminal guards.
- Continue active Multi-Step 0 from a fresh target after one initial Solver
  activation; Single Step/finite recovery still returns the preview for review.
- Add K50–K52 native-coordinate regression evidence and adverse/controller cases.
- Deliver full affected files and a guarded installer against exact pushed main
  eb18478b59b29b00bd5c1e11f1a30348226a5f61. No predecessor candidate authority.

## 1.2.4 — 2026-10-03 (candidate 2; Beast verification pending)

- Recognise K44's measured horizontal tableau-source displacement using a second
  bounded scan with the existing closed-outline and safe-click requirements.
- Re-observe unsupported post-action scenes input-free within the existing shared
  three-capture recovery budget, preserving the pending action and STOP.
- Allow the measured contracted bottom-card source to support qualified fresh-HALO
  continuation through spatially aggregated bidirectional print evidence; keep
  complete-effect verification unchanged and the previous move unverified.
- Replace a variable Level Up text-shadow probe with stable artwork, permit the
  measured warm fireworks only at four context points, and require mixed OK
  lettering/background evidence.
- Record exact Solve stock/artwork/glyph counts and selected target on each run
  observation. K48 already passes the recogniser; no further colour relaxation
  or claim that the unseen live refusal is resolved.
- Add K44–K49 native-coordinate regressions and adverse/controller coverage.
- Deliver complete files against the same exact base or complete candidate 1,
  with guarded preview, backups, content verification and rollback.

## 1.2.4 — 2026-10-02 (candidate 1; Beast verification pending)

- Recognise the evidenced 604-pixel ten-card source and its closing edge beneath
  Undo All, keeping click/effect bounds above the toolbar.
- Accept the measured positive red warming of the Solve border while preserving
  inner artwork, glyph, scene, empty-stock and selection-priority checks.
- Use the existing post-game observation bound after acknowledged Solve instead
  of the short card-recovery budget; require two consecutive fresh win positives.
- Add native K41–K43 regression evidence and negative/controller coverage.
- Preserve timings, one-board Klondike completion, existing terminal controls,
  TriPeaks/Pyramid behaviour, STOP, uncertain-input handling and original PNGs.
- Deliver complete files against d9ad2cdf536165bb99ae4e02dabc49f7a9fa7227 with
  guarded preview, backups, content verification and rollback.

## 1.2.3 — 2026-10-01 (candidate 5; Beast verification pending)

- Continue from the recorded bottom-card replacement after the remaining tableau
  stack spreads, using two full visible proof patches above the toolbar.
- Preserve ordinary clipped-corner refusal, directional/paper thresholds and
  the 512-pixel source gate; keep this new route separate from complete-effect
  verification because the next HALO's shading is not receiver proof.
- Add the native before/result pair and adverse source, receiver, patch, shading,
  cursor, gold, toolbar, material and geometry checks.
- Preserve candidate 4 continuation, Solve, budgets, timings, STOP, shared modes
  and exact-byte PNG saving; deliver exact candidate 4 upgrade and rollback.

## 1.2.3 — 2026-10-01 (candidate 4; Beast verification pending)

- Continue from a supported fresh Klondike recommendation after an acknowledged
  card-source action when its existing source proof passes; keep the previous
  effect unverified if independent receiver evidence is insufficient.
- Permit an independently recognised next Solve control to become the next plan,
  retaining one-shot input, its separate settle and independent completion checks.
- Count each acknowledged action against finite limits and report verified and
  continued operations separately; unchanged cards and uncertain input stop.
- Recognise the measured nine-card run closing at rows 988–990, accounting only
  for the exact Undo All overlap while preserving complete-border and input bounds.
- Add the real black-pip RIGHT pair, Solve priority and deeper-run pixel fixtures
  with adverse controller/geometry checks and explicit runtime limits.
- Deliver complete guarded candidate 3 upgrade and rollback to its actual contents;
  preserve timings, continuous defaults, shared modes and original PNG saving.

## 1.2.3 — 2026-10-01 (candidate 3; Beast verification pending)

- Recognise the measured 3-hearts foundation recipient change under a persistent
  dark landing guide, preserving the ordinary source/destination thresholds.
- Extend fixed upper-card replacement-corner evidence to existing SUIT-source
  returns, adding guarded bidirectional printed-detail changes over stable bright
  paper with separate destination proof and explicit synthetic-test limits.
- Add the actual reconstructed RIGHT pair with native-pixel provenance and
  adverse guide, mask, source-only and destination-only checks.
- Retain candidate 2 completion, continuous defaults, editable timings, STOP,
  warning fixes and established TriPeaks/Pyramid behaviour.
- Deliver a guarded exact candidate 2 upgrade and rollback to its actual contents.
- Record FreeCell Gate 1 evidence; FreeCell implementation remains a later gate.

## 1.2.3 — 2026-10-01 (candidate 2; Beast verification pending)

- Recognise an evidenced tableau transfer with automatic white-face replacement
  in the same column, retaining both source identity and independent destination
  evidence without lowering the existing material-change thresholds.
- Return a changed initial preview for review with zero guest input; preserve
  exact fresh approval before gameplay or no-HALO Solver recovery.
- Default Klondike Actions per Multi-Step to 0, continuous until STOP or a guarded
  stop. Add an independently editable Solve animation interval.
- Detect one-board Klondike game completion from two fresh positive observations;
  require intact full-gold/zero-black progress or completed-game artwork.
- Enable the evidenced score-skip, Level Up OK, New Game, Draw 1 Play and fresh
  Solver sequence only for continuous runs, one guarded control input per stage.
- Preserve shared TriPeaks/Pyramid completion policies and input coordinates.
- Correct compiler/Clippy diagnostics without new warning suppression or a
  formatter, preserving Charlie's source spacing and nightly configuration.
- Supply complete-file installation against the same committed base, accepting
  the exact installed candidate 1 and backing up the actual starting contents.

## 1.2.3 — 2026-09-30 (candidate 1; Beast verification pending)

- Recognise tableau card removal exposing green felt dimmed beneath a dashed
  destination guide, retaining independent source and destination evidence.
- Log dimmed-felt removal counts separately from bright replacement pixels.
- Add original-pixel regression scenes for the repeated three-spades refusal
  and the later empty-column result, with explicit evidence limits.
- Preserve the accepted tall-run geometry, editable timings, continuous mode,
  bounded observations, STOP and one-shot Solve result review.
- Build guarded complete-file installation records for committed v1.2.2 at
  `bcce7515979e2c4520b02f39e9c4d4d1bb019d7f`.

## 1.2.2 — 2026-09-30 (candidate 2; Beast verification pending)

- Correct tall Klondike source bounds where a run overlaps the dimmed toolbar.
  Reject internal card edges while connected side rails continue below them.
- Keep the source click and effect evidence above toolbar controls.
- Add original-pixel regressions for the reconstructed Draw pair, the unchanged
  RIGHT failure scene and the tall highlighted run.
- Log existing pointer/hold/settle intervals and measured input/capture times
  to support diagnosis of intermittent no-op inputs without changing timings.
- Support guarded installation over the exact uncommitted candidate 1 contents,
  with backups and rollback to the actual pre-install state.

## 1.2.2 — 2026-09-30 (candidate 1; Beast verification pending)

- Restore Klondike Multi-Step `0 = continuous`, with bounded recovery per action,
  STOP checks and checked operation counters.
- Set the editable Klondike action-settle default to 750 ms; retain the independent
  1000 ms recapture default and update control wording.
- Correct a demonstrated RIGHT-pile effect-verification blind spot using two
  opposite card-corner comparisons plus independent destination evidence.
- Add source/destination proof diagnostics for unresolved effects.
- Recognise the separately evidenced Solve button; request auto-finish once,
  capture its result and stop for review without claiming completion or restart.
- Preserve Charlie's committed Rust minimum 1.101.0, nightly settings, source
  spacing, existing-game policies and original PNG saving contract.

## 1.2.1 — 2026-09-30 (candidate 1; Beast verification pending)

- Add the Solver-driven Klondike Draw 1 profile, dynamic source-block detection,
  three-position RIGHT fan, stock draw/recycle and SUIT-source priority.
- Add an independent bounded Klondike controller, configurable settle/recapture
  values, finite Multi-Step, and one immediate-capture Solver recovery per
  unresolved context. Unknown results never enter shared completion/restart.
- Preserve the established TriPeaks/Pyramid policies and exact-byte PNG saving.
- Add two blank lines before definitions and statement blocks throughout the
  existing and new Rust source without running a formatter.
- Keep nightly, rustfmt configuration and dependency versions unchanged.

## 1.0.1 — 2026-09-26 (candidate; Beast verification pending)

- Record all 28 Pyramid card envelopes and hit points, plus Left, Move/Recycle and Right, from Issue #1's 1920x1080 screenshots.
- Replace broad Pyramid preview envelopes with labelled target rectangles and hit-point markers (Issue #3). Draw overlays only on the main preview; preserve original saved PNG bytes. Keep Pyramid input disabled pending behaviour and detector validation.
- Name Solver, Undo All and Undo as shared toolbar controls and display them in both mode previews.
- Update architecture, socket-path and capture documentation for Issue #2; remove stale project-stage labels while preserving calibration provenance.
- Synchronise the package version in Cargo.toml and Cargo.lock without changing dependencies.

## 1.0.0 — 2026-09-26

The following behaviour was already present in the 1.0.0 base commit:

- Allow the Level Up screen three seconds to settle after the score panel click. If a fresh frame already shows New Game, require a second capture of that stage before acting on it.
- Capture a fresh QMP PNG when Capture PNG is clicked, preview that image, and save precisely those original bytes when Save PNG or Enter is used. Add field focus and a Cut/Copy/Paste context menu.
- Record the Challenge Complete Continue button geometry for future challenge support, without enabling automated input.
- Remove inherited application and artefact names; default to `qmp-qemu-socket.sock`. `QMP_SOCKET_PATH` supports an existing QEMU socket while the launch configuration is migrated.

- Establish `qmp-qemu-socket` with guarded TriPeaks actions and read-only Pyramid calibration.
- Use QMP to capture the VM display, verify each guarded action, and retain private session logs and original PNG evidence.
- Keep the application version in `Cargo.toml` as the source for the window title and Parameters display.
