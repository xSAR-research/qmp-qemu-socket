# Game workflow refactoring work tasks

Editable planning draft — updated 1 November 2026.

This revision merges the original assessment with Gemini's seven recommendations
and Charlie's direction to prioritise the A* process before Free Cell v3.0.0.
Approval covers this planning-document update, not implementation of the refactors.
No code, dependencies, version, public API or execution permissions change here.

Do not copy this editable document's contents into Tree Ring, XERJ, context-mode
or other long-term memory. Read the current file when work resumes. Only a
separately approved settled revision may become a durable plan.

## Objective and decision rule

Make the A* route lifecycle easy to follow and test: acquisition and RECON,
validated knowledge, search, independent replay, execution authority, observed
progress, and completion. Keep original deal, accumulated knowledge, planned route
and live position distinct. The next feature milestone is Free Cell v3.0.0;
Klondike follows, with Spider and TriPeaks requiring further investigation of
route navigation, information acquisition and backtracking.

Protect the working Computer Vision / guest Solver continuous-game workflow.
Charlie reports that it works, apart from major-theme rank/suit-reader changes.
That report is behavioural context, not new automated or live acceptance here.
Broader theme support is a separate capability change, not a refactor objective.

Rank by demonstrable benefit to the next A* milestone, size of the affected
behavioural surface, and strength of existing regression evidence. Line-count or
field-count reduction alone is insufficient. Prefer small private helpers and
explicit phases; preserve game-owned rules, recognition and completion evidence.
One implementation owner handles one bounded package at a time.

## Two solution methods, separate regression tracks

| Track | Current purpose | Main preservation contract |
| --- | --- | --- |
| A — A* Route Solution | Current Pyramid acquisition, planning, replay and guarded execution; foundation for future Free Cell | A valid route or search result alone never grants input authority; bind current acquisition, replay and source evidence |
| C — Computer Vision / Solver | Preserve the working continuous-game controllers across the five current game types | Fresh supported Solver/HALO evidence and the existing game-specific recovery/completion policy authorise each operation |
| X — Shared boundaries | UI, method selection, STOP, capture and worker events used by either track | Context/run identity, cancellation, capture freshness, bounded delivery and no uncertain replay survive either method |

These are solution-method tracks, not route objectives. Maximum/Shortest and
Exact/weighted search choices remain separate dimensions inside the A* track.
A* still uses computer vision to recognise the board; track C means the existing
guest Solver/HALO method, not all image recognition in the application.

The live focus for C is continuous play. Existing finite-step/Step Once entry
points, where reachable, remain compatibility checks; this plan does not remove
them. Exercise A* session/continuation modes where currently supported. Never
assume a new game already supports A* because its Solver controller exists.

## Ranked milestone index

Existing RF00–RF07 IDs are retained. Rank is separate from ID so Charlie can
reorder the plan without breaking references. New IDs split Gemini's suggestions
into independently reviewable changes. ROI is qualitative, not a measured speedup.

| Rank | ID / bounded package | Track | Expected ROI / regression exposure | Dependency / disposition |
| --- | --- | --- | --- | --- |
| 0 | RF00 — baseline and method-specific evidence | A, C, X | Essential / no runtime change | Required before implementation |
| 1 | RF09 — one final-clear verification chain | A | High / small extraction | RF00; recommended |
| 2 | RF08 — explicit source-transition versus board-position observation | A | High / medium, authority-sensitive | RF00, preferably RF09; recommended |
| 3 | RF06-A — explicit A* request, phase and completion flow | A, X | High / medium | RF00, RF08; recommended |
| 4 | RF04 — one UI STOP revocation owner | A, C, X | High / medium | RF00; recommended if trace equivalence is demonstrable |
| 5 | RF07-A — identical guard primitive only | A, C, X | Moderate / low if literally identical | Conditional: do only if needed by the above or Free Cell |
| — | **ROI / regression-risk stop line** | — | **Finish A* clarity, then move to Free Cell feature design** | **Everything below is deferred by default** |
| D1 | RF12 — exact underwater reference comparison helper/table | A recognition | Moderate / calibrated recognition surface | Separate from RF09; on-demand only |
| D2 | RF10 — extract TriPeaks execution and recovery | C | High local readability / medium | Valuable, but not a prerequisite for Free Cell A* |
| D3 | RF13 — explicit post-game stage methods/context | C, X | Moderate / medium-high | Requires affected-method ownership audit first |
| D4 | RF11 — snapshot/parameters/preview state groups | X | Moderate / broad UI lifetime surface | One group at a time if feature work needs it |
| D5 | RF01 — identical Free Cell/Spider image primitives | C | Modest / recognition surface | No urgency while current Solver flow works |
| D6 | RF02 — terminal-control observation mechanics | C | Moderate / bounded retry/evidence policy | Only with explicit per-game contracts |
| D7 | RF03 — Free Cell/Spider terminal progression | C | Moderate / live terminal sequencing | RF02 and its live acceptance |
| D8 | RF05 — preview mailbox/control-state split | X | Moderate / broad locking/lifecycle surface | Defer until ownership blocks a real feature |
| D9 | RF06-B — move remaining UI event application | X | Modest after RF06-A / broad | Defer bulk event-handler reshuffle |
| D10 | RF14 — inline test moves/parameter organisation/capture plumbing | Supporting | Local readability / avoidable diff churn | Only alongside an affected approved module |
| D11 | RF07-B — further cross-game I/O/controller sharing | C, X | Uncertain / high if policies diverge | Assess later; no universal executor commitment |

### The stop line

Stop after the recommended A* packages when a reviewer can trace acquisition to
completion, distinguish every authority boundary, and verify the regression
matrix below. Reassess the conditional guard extraction against actual Free Cell
requirements. Do not delay Free Cell v3.0.0 for wholesale cleanup of working
Solver controllers, theme-proof banks, application fields or test placement.

A package below the line moves above it only when it removes a concrete blocker,
fixes a demonstrated defect, or enables a specific approved Free Cell acceptance
check. Record the reason, affected tracks, expected benefit and additional tests.
If equivalence needs new retry rules, thresholds, waits or completion meaning,
stop treating the change as a behaviour-preserving refactor and obtain separate
agreement. Split or defer it rather than weakening a regression.

## RF00 — preserve the baseline and define evidence

- [x] Read current instructions, handoff, manifests and exact dependency revisions.
  Reconcile the known post-push HEAD discrepancy before code implementation;
  the old dirty-tree receipt is not proof of the new committed baseline.
- [x] Preserve the existing post-174 Cargo feature/lock changes, evidence, user
  edits and Git index. Record which checks cover those exact inputs; do not call
  those dependency changes newly qualified because they were committed.
- [x] Build the A/C/X regression inventory below from actual existing test owners.
  Link every milestone to its affected scenarios before editing code.
- [x] Select comparable command/event traces, fixtures and supported live cases.
  Record attempts, ACKs, logical actions, captures, waits, stage changes and outcomes.
- [x] Agree one implementation package, its exact preserved behaviour and release
  acceptance. Reuse valid unchanged evidence; no ceremonial baseline rebuild.

Done when: method-specific baselines, proof gaps and package acceptance are clear.

## RF09 — one authoritative final-clear chain

Gemini 3b. Owner: src/worker/computed_route.rs, in classify_observation and
observe_final. The five existing alternatives have the same precedence; their
callers still have different surrounding guards and error handling.

- [x] Extract one private helper for the existing five alternatives, preserving
  order, short-circuiting, inputs and final error outcome.
- [x] Keep caller-owned challenge binding, frame guards, source proofs, position
  checks and terminal meaning in their existing places.
- [x] Check every supported clear alternative and failures where the tableau is
  empty but required header, pile, scene or completion evidence is absent.

Done when: both callers use the same proof chain without broadening completion.

## RF08 — make A* observation authority explicit

Gemini 4. Owners: src/worker/computed_route.rs and
src/pyramid_recon/vision/source_transition.rs; existing removal-history owner is
src/pyramid_recon/vision/removal.rs.

- [ ] Map actual source_transition creation, lifetime and reachable None/Some
  branches. Audit stage/commit reachability separately; do not label it dead code.
- [ ] Separate source-disappearance observation from board-position observation
  with explicit private helpers and documented inputs/results.
- [ ] Keep common fatal frame checks, first-failure retention, bound challenge
  progress and source-proof publication in one visible, ordered common path.
- [ ] Preserve expected original source identity before once-only input and two
  successive fresh post-action checks; different card or None is accepted only
  for the already-issued action, not as proof of the next action's source.
- [ ] Preserve next-source readiness, same-frame pairs, resets, independent live
  position knowledge and separate semantic completion.

Done when: source, position and completion proofs can be followed independently,
with unchanged captures, input permissions and refusal outcomes.

## RF06-A — clarify the A* lifecycle and request/event flow

Gemini 2 (event handling) and the original RF06. Owners: src/app.rs,
src/app/pyramid_workflow.rs, src/worker/computed_route.rs,
src/worker/pyramid_reconnaissance.rs and src/worker/pyramid_session.rs.

- [ ] Document current ownership and transitions: acquire/RECON, validate/restore,
  plan, independently replay, prepare permit, execute/observe, retire/continue.
- [ ] Give planning and execution distinct private entrypoints instead of the
  current boolean dispatch parameter; planning remains input-free.
- [ ] Reuse typed commands and complete_route/complete_session reducers. Preserve
  acquisition identity, run IDs, generation/context binding and permit consumption.
- [ ] Isolate only A*-relevant event application needed for understandable phase
  transitions; preserve stale-log versus stale-state handling and pending gates.
- [ ] Keep STOP, pause, search controls, failure reporting and terminal outcomes
  observable at the current boundaries. No shared mutable global session state.

Done when: the current Pyramid A* lifecycle is clear enough to design the next
game adapter, without claiming that Pyramid's board or RECON types fit Free Cell.

## RF04 — centralise common UI STOP revocation

Original RF04. Owners: src/app.rs and src/app/pyramid_workflow.rs.

- [ ] Share STOP latching, observed-state retirement, RECON-consent/acquisition
  revocation and disconnect dispatch behind one private owner.
- [ ] Preserve caller-specific error reporting, active-run ownership, late-event
  filtering and cooperative shutdown.
- [ ] Check both method tracks, both STOP views, disconnect failures, late/duplicate
  completion and method/game/context changes. No post-STOP restoration or cleanup.

Done when: both views preserve the same outcomes and input revocation as before.

## RF07-A — conditional identical guard primitive

Narrow part of Gemini 5 and original RF07. STOP -> fresh VM/pointer probe -> STOP
is identical in the Free Cell/Spider helpers; that does not make their executors
interchangeable.

- [ ] Extract only if a recommended package or agreed Free Cell adapter needs it;
  retain the existing probe contract and precise failure accounting.
- [ ] Compare successful and failed probes, STOP before/after probe, uncertain
  delivery and zero further input. No trait or generic controller is required.

Done when: the bounded primitive is shared without moving game policy into it;
otherwise record this package as deliberately deferred.

## Deferred work packages and retained IDs

These tasks remain available for Charlie to promote with a concrete reason.
Their presence is not implementation approval or a requirement before v3.0.0.

### RF12 — exact underwater footprint comparisons (Gemini 3a)

- [ ] Consolidate repeated loops only if this recognition owner needs maintenance.
  There are six concrete reference entries across four helpers; retain fixed
  geometry, full 4,896-pixel/14,688-channel equality and checked bounds.
- [ ] Preserve whole-reference matching, wrong-slot outcomes, diagnostic errors,
  strong-white Present veto and ordering ahead of aged-material fallback. Do not
  build a per-pixel union of unrelated exact references.

### RF10 — TriPeaks execution and recovery extraction (Gemini 1)

- [ ] Move TriPeaks-only action verification/recovery from worker.rs into its own
  private execution module after confirming the boundary.
- [ ] Preserve TriPeaks HALO recovery, geometry, terminal checks and all shared
  pre-input work. Leave other modes and capture plumbing outside this package.

### RF13 — post-game stages and cohesive context (Gemini 6)

- [ ] Split the large post-game flow into explicit stages with typed local context;
  identify which solution-method callers actually use it first.
- [ ] Preserve stage-local budgets, score-skip ordering, reward waits, guards,
  evidence retention and failure outcomes. A context struct must clarify ownership,
  not merely silence too_many_arguments. Lint-only detector cleanup is incidental.

### RF11 — owned UI state groups (Gemini 2)

- [ ] Separate SnapshotDialog, ParametersForm and PreviewPanel one at a time when
  their ownership obstructs work. Preserve texture/frame lifetimes and busy-state
  controls; do not bundle all field moves with RF06-A.

### RF01 — identical image primitives (original RF01)

- [ ] Share identical checked frame/colour/fraction helpers only; retain all
  recognition thresholds, widths, edge acceptance and error contracts.
- [ ] Preserve Free Cell's both-margin and Spider's either-margin paper rules.

### RF02 — bounded terminal-control observation (original RF02 / Gemini 5)

- [ ] Reuse existing ControlLookup; share mechanics only after documenting per-game
  delays, evidence requirements and diagnostic retention.
- [ ] Preserve three normal searches and at most one eligible fourth OK-only
  observation; no fourth New Game search, foreground-underlay confusion or input
  when both controls are ready.

### RF03 — terminal progression (original RF03 / Gemini 5)

- [ ] Share Free Cell/Spider orchestration only after RF02 acceptance. Preserve
  Score -> optional Level Up OK -> New Game -> Play, once-only stage input,
  Spider's Play wait and fresh supported gameplay before resuming.

### RF05 — mailbox versus control-state ownership (original RF05)

- [ ] Separate bounded preview delivery only if needed; preserve current locking,
  coalescing, pause/search/settings access and action snapshots. No new queue/thread.

### RF06-B — remaining UI event extraction (original RF06 / Gemini 2)

- [ ] Move remaining event handlers only after RF06-A proves useful; preserve
  context rejection before mutation and dedicated completion semantics.

### RF14 — supporting organisation (Gemini 7 and optional part of Gemini 1)

- [ ] Move inline tests only with the affected approved module; preserve test
  discovery, names, privacy and fixture/include paths. No broad test-file sweep.
- [ ] Separate filesystem paths, calibration and runtime settings in parameters.rs
  only when an affected feature needs it; preserve names/defaults and callers.
- [ ] Extract capture I/O separately if worker ownership requires it; preserve
  backend selection, fallback, frame retention and cleanup/error semantics.

### RF07-B — remaining I/O sharing (original RF07)

- [ ] Recompare capture, retention, waits and publication after specific extractions.
  Keep decoded failure frames, game recovery and ordered completion game-owned.
- [ ] Assess Klondike reuse separately. Do not create a universal game executor,
  universal RECON model or cross-game completion predicate from superficial overlap.

## Regression strategy

### A — A* route regression matrix

Use current Pyramid A* as the reference implementation. Tests for future Free
Cell are new feature acceptance, not evidence that current refactoring preserved
an already-working Free Cell route solver.

| Area | Positive evidence | Required negative / boundary cases |
| --- | --- | --- |
| Acquisition and knowledge | Supported same-session RECON, exact identities/order, restoration and authenticated publication | Incomplete/duplicate inventory, unknown reader, wrong context/session, failed restoration, partial publication; no route authority |
| Read-only preparation | Valid current acquisition can plan without guest input | Explicit RECON consent remains separate; prepare/plan never activates Solver or sends gameplay input |
| Search and objective | Existing Shortest/Maximum and supported exact/weighted options retain route outcome/cost and stop reason | No route, incomplete route, budget stop, cancellation, fallback and operator-choice persistence remain distinct |
| Independent replay | Every prepared action and endpoint agrees with original deal/rules | Illegal source/action, mismatched objective or endpoint, stale plan and wrong deal refused before input |
| INITIAL source readiness | Expected original sources identified in the same supported frame | Cached first check plus existing bounded fresh retries, partial/flying cards, stale serial, STOP and structural error; no click before ready |
| Once-only execution | Ordered operations with fresh evidence and STOP -> probe -> STOP | STOP between pair clicks, uncertain ACK/release, wrong pointer/VM state; actual counts retained, no replay or false logical commit |
| POST observation | Two fresh successive source-disappearance checks for an issued action | Original reappears, structural/scene failure, reused capture, pair mismatch; next-source proof remains independent |
| Progress and completion | Bound progress, each supported final-clear alternative, partial endpoint and continuation | Removed source or empty-looking tableau alone is not BoardClear; wrong header/piles/dialog, regressed/unreadable counter and challenge mismatch refuse advancement |
| Session lifecycle | Current completion, pause/resume where allowed, continuation and diagnostics | Late/stale events, duplicate completion, method/game/socket/generation change, STOP during search/RECON/restoration/continuation; no stale-plan resume |

For ordinary code extraction, compare semantic outcomes and operation traces;
require exact route identity only where deterministic ordering is part of the
existing contract. Wall-clock search duration is not an equivalence assertion.

### C — Computer Vision / guest Solver continuous-game matrix

| Game / path | Preserve and exercise |
| --- | --- |
| Free Cell | Fresh source HALO; no ordinary card-pixel effect gate; shared activation/refresh reserve; bounded active-no-HALO handling; existing completion and continuous restart |
| Spider | Fresh source or Draw authority; no active-Solver refresh; pointer park after source click but not Draw; distinct deal/Play waits; no collapsed-suit/destination authority |
| Klondike | Its own supported-source and gold/no-black or terminal completion evidence, consecutive confirmation and ordered terminal flow |
| TriPeaks | Existing before/after effect evidence, HALO and geometry recovery, stock/redeal and completion policy |
| Pyramid Solver | Existing guided source/effect, pile/redeal/terminal and challenge policy; do not substitute A* acquisition/route permissions |
| Shared terminal controls | Absent/ambiguous controls, partial/moving glyph, stale OK geometry, eligible final OK-only observation, foreground panel versus underlay, STOP at every stage and latest failed frame |

Cover representative continuous transitions plus reachable finite/Step Once
compatibility controls. Missing HALO is never by itself a win. Terminal/Solver
setup input must retain its existing separation from counted game actions.

### X — cross-method and shared-boundary matrix

- [ ] Switch solution method, game, socket and generation: old frames, acquisition,
  prepared routes and queued results cannot acquire new input permission.
- [ ] STOP before input, after probe and between gestures: cancellation remains
  latched; no cleanup/restoration, accidental refresh or uncertain replay.
- [ ] Failed capture/analysis, bad dimensions/stride and stale frames retain correct
  diagnostics and do not authorise input from preview-only pixels.
- [ ] UI busy states, snapshot review, worker disconnect/failure, late logs versus
  late state events, and duplicate completions preserve their current outcomes.
- [ ] Preserve current bounded frame coalescing, per-action settings, progress
  publication and input/capture ownership without adding threads or globals.

### Evidence, traces and meaningful checks

Each implemented package records the before/after result for its affected A/C/X
rows, linking existing test names to requirements. Use instrumented test doubles
and current adapter integration tests for command order and zero-input negatives;
do not add production captures merely to build a recorder. Compare input attempts,
ACKs, logical commits, capture freshness/counts, intentional waits and their
ordering, state transitions, retained diagnostic frame and final outcome.

Keep current budgets and waits exact when their trace is contractual. When
measuring live pacing, distinguish actual waits from elapsed processing time;
use repeatable evidence rather than inventing a new delay. Unsupported scenes or
themes must still refuse input. Major-theme reader support is separate work.

Existing A* test owners include worker/computed_route/tests.rs, wire_tests.rs,
series_guard_tests/, pyramid_recon/vision/source_transition.rs and worker
pyramid_reconnaissance/starting_guard_tests.rs. Useful existing scenarios include:

- context_changed_and_back_or_idle_stop_cannot_reauthenticate_acquisition;
- partial_route_objective_must_match_independently_replayed_removals;
- incomplete_route_is_refused_before_any_capture_or_input;
- stop_after_first_pair_ack_refuses_second_click_and_logical_commit;
- uncertain_pair_delivery_retains_actual_counts_without_commit_or_replay;
- native_delivery_probes_after_fresh_capture_before_one_click_or_draw_key;
- native_delivery_stop_before_probe_ack_sends_no_pointer_key_or_button_input;
- readiness_173_both_sources_must_match_the_same_supported_frame;
- source_172_green_original_disappears_twice_without_replacement_matching;
- source_172_resets_on_original_reappearance_or_structural_error.

Resolve full current paths and test filters in RF00; an example name is not a
claim that a test was run by this document update. Retain existing CV controller,
terminal, recognition and UI lifecycle test owners rather than testing only a
new helper in isolation.

### Per-package qualification and live acceptance gate

1. Before editing, identify affected method rows and the unchanged method controls.
   Agree exact scope and reconcile the current handoff/baseline discrepancy.
2. During editing, run narrow owning-package checks and meaningful existing/focused
   regressions. Refuse changes to game policy merely to simplify the extraction.
3. Once the agreed package is complete, run required controller Tier 2 once:
   scoped format, locked check, strict private rustdoc, Clippy, full tests including
   ignored tests and native release, with the required toolchain/package/target/
   feature flags. Check current instructions for exact commands. No unchanged
   baseline rebuilds or repeated full suites without a new reason.
4. For an A-only extraction, run the A and X regression rows plus existing C
   compatibility tests; arrange a supported Pyramid A* live scenario covering
   RECON/restoration, replay, ordinary/pair/final source behaviour and STOP at
   relevant stages. If shared UI/guards are touched, also run representative CV
   continuous play. Cover affected objectives/session modes without claiming one
   successful board covers the entire matrix.
5. For a C/shared-controller extraction, run affected C and X rows plus A
   acquisition/plan/execution compatibility tests. Live-test each changed game's
   relevant continuous restart/recovery/terminal path and a representative A*
   scenario if a shared boundary changed. Authorise guest actions separately.
6. If xSAR changes become necessary, agree that scope separately and run its full
   required feature matrix plus real consumer checks; otherwise reuse valid exact
   dependency receipts and label them cached, not freshly executed.
7. Record actual pass/fail/pending results, fixed input/source hashes, full owned
   paths and retained failures. Offline regressions are not live guest acceptance.
   A milestone needing live evidence remains pending until that evidence exists.
8. Obtain acceptance and explicit version approval, align only approved manifests/
   locks and required checks, then update canonical tracker/handoff. This planning
   document earns no version increment. No automatic commit/push is authorised.

## Free Cell v3.0.0 readiness and later games

After the recommended refactors, conduct feature design for Free Cell A*: define
its own board/free-cell/foundation state, legal moves, multi-card capacity,
complete acquisition, search objective, replay and checked execution/effect/
completion proofs. Reuse generic xSAR search/card/geometry primitives where their
contracts already fit. Do not transplant Pyramid's 28-slot tableau or stock model,
claim a generic adapter exists, or implement Free Cell by changing only an enum.

Klondike then needs its own stock/waste/foundation and hidden-information contract.
Spider and TriPeaks may require iterative observation, knowledge updates, route
navigation and backtracking beyond today's fixed complete-deal route. Keep these
as game-specific feature design questions; do not build a speculative universal
framework before their requirements are known.

Search backtracking explores alternative model states. Physical guest UNDO or
rollback sends input and needs an explicit supported recovery contract, retained
knowledge, fresh evidence and STOP authority. Neither RECON knowledge nor a failed
search authorises physical rollback. Keep those two meanings separate.

## Gemini coverage and original-milestone cross-reference

| Gemini item | Integrated package(s) | Decision |
| --- | --- | --- |
| 1 — TriPeaks execution, optional capture plumbing | RF10, RF14 | Deferred unless A* feature work needs the boundary |
| 2 — UI state groups and poll_worker | RF06-A/RF06-B, RF11 | A* lifecycle first; broad ownership reshuffle deferred |
| 3 — exact footprints and final-clear chain | RF12, RF09 | Two packages; final-clear extraction first |
| 4 — temporal/source observation branches | RF08 | Recommended split; reachability audit before deletion |
| 5 — guard/controller boilerplate | RF07-A/RF07-B, RF02, RF03 | Identical guard only on demand; working Solver flows protected |
| 6 — post-game stage/context decomposition | RF13 | Deferred pending concrete caller/feature need |
| 7 — test modules and parameters layout | RF14 | Local supporting work, not a blanket cleanup milestone |

Original RF00–RF07 remain traceable above. Suffixes separate recommended and
broader parts of an existing ID; new RF08–RF14 carry the additional findings.

## Edit and decision log

| Date | Package | Amendment / decision | Implementation approval |
| --- | --- | --- | --- |
| 2026-10-31 | RF00–RF07 | Original editable source-assessment draft | None |
| 2026-11-01 | All | Merge Gemini 1–7; rank A* preparation first; add A/C/X regression strategy and explicit ROI stop line | Document revision only |

No refactor, new qualification, milestone acceptance or release is recorded by
this update. Charlie may amend this draft; its current contents remain excluded
from long-term memory and search indexes.
