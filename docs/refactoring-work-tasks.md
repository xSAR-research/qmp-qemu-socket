# Game workflow refactoring work tasks

Draft for Charlie to edit — 31 October 2026.

Status: proposed work, not approval to implement. Documentation changes do not
change the application version. Assign a release version only after an agreed
code milestone passes its required checks and Charlie approves the increment.

This file is the editable task artefact. Do not copy its current contents into
Tree Ring, XERJ, context-mode or other long-term memory. Read the file again when
work resumes; only an explicitly approved settled revision may become a durable
plan. The separate scoring note may be indexed as requested.

## Objective and scope

Improve program flow and remove duplication while preserving the successful
behaviour of each game. Prefer small private modules that share mechanical work.
Retain game-owned recognition, recovery, completion and input permissions.

Charlie reports successful testing except where major theme changes affect the
rank/suit reader. That report is the behavioural context, not fresh validation
performed by this documentation task. Expanding theme support is separate work.

Owner: `qmp-qemu-socket`, with one implementation owner per milestone. No sibling
repository, dependency, public API, schema, threading or transport changes are
included. Reconsider any such change with Charlie before implementing it.

## Milestone index

| ID | Work package | Depends on | Status | Approval / release |
| --- | --- | --- | --- | --- |
| RF00 | Preserve the baseline and define comparison evidence | — | Proposed | To agree |
| RF01 | Share identical Free Cell/Spider image primitives | RF00 | Proposed | To agree |
| RF02 | Share bounded terminal-control observation | RF00 | Proposed | To agree |
| RF03 | Share Free Cell/Spider terminal progression | RF02 | Proposed | To agree |
| RF04 | Centralise common UI STOP revocation | RF00 | Proposed | To agree |
| RF05 | Separate preview mailbox from shared worker controls | RF00 | Proposed | To agree |
| RF06 | Clarify UI event handling and plan/execute entrypoints | RF04, RF05 | Proposed | To agree |
| RF07 | Assess remaining native I/O duplication | RF03, RF05 | Optional review | To agree |

IDs remain stable if Charlie changes the order. RF01 is a small independent
extraction; RF02–RF03 address the strongest cross-game controller duplication.
RF04 can proceed independently once RF00 is agreed. Complete and review one
bounded package before starting another.

## RF00 — preserve the baseline and comparison contract

- [ ] Read the active handoff, instructions and current manifests; run the required
  sentinel once and resolve its exact discrepancy before implementation.
- [ ] Preserve existing Cargo feature/lock edits, calibration, fixtures, Git index,
  unrelated changes and diagnostic evidence. The post-174 X11/Wayland manifest
  changes are outside this refactor and have not been newly qualified here.
- [ ] Select existing regressions and reference command/event traces for finite
  and continuous runs, failure paths, stale contexts and STOP.
- [ ] Record per-game input order, capture/observation budgets, intentional waits,
  action counts, terminal sequencing and latest-frame retention.
- [ ] Agree the first implementation package and its acceptance checks.

Done when: the preserved baseline and comparison evidence are explicit, and
Charlie has approved a bounded implementation scope. Reuse valid unchanged
qualification receipts; do not rebuild an unchanged baseline for ceremony.

## RF01 — share identical image primitives

Current owners: `src/freecell.rs`, `src/spider.rs`. The assessment found identical
frame validation, gold/felt/paper predicates and fraction scanning.

- [ ] Move only the identical checked helpers into a small private module, using
  existing `image_evidence` primitives where their contracts already match.
- [ ] Keep native geometry, thresholds, source classification and error semantics
  unchanged; callers retain game policy.
- [ ] Keep Free Cell's requirement for both clipped paper margins and Spider's
  either-margin rule. Preserve their distinct widths and edge acceptance.
- [ ] Run relevant existing recognition, malformed-frame and source-negative
  regressions; compare accepted/rejected fixture results with the baseline.

Done when: both callers use the same helpers with identical results, without a
generic card reader or broadened scene/input authority.

## RF02 — share bounded terminal-control observation

Current owners: `src/worker/freecell_execution.rs` and
`src/worker/spider_execution.rs`, particularly `await_post_score_control` and
`await_expected_control`. Reuse `src/worker/control_search.rs` recognition.

- [ ] Extract common observation/search mechanics behind a private typed interface;
  retain per-game profiles, re-observation delays and diagnostic context.
- [ ] Preserve three normal searches and at most one eligible final OK-only
  confirmation. Never add a fourth New Game search.
- [ ] Preserve current glyph/settled-geometry requirements, gameplay rejection and
  foreground OK-panel exclusion of underlying New Game artwork.
- [ ] Preserve ambiguity refusal when both controls are ready, cancellation,
  failed-capture handling and retention of the latest failed frame.
- [ ] Exercise existing absent, ambiguous, moving/partial glyph, eligible-final-OK,
  stale-geometry, STOP and no-replay cases in both controllers.

Done when: control searches, waits, captures, diagnostics and zero-input refusal
paths match the agreed traces for both games.

## RF03 — share terminal progression

Current owners: `restart_game` in the same Free Cell/Spider worker modules.

- [ ] Share Score → optional Level Up OK → New Game → Play orchestration using
  the RF02 observation owner.
- [ ] Keep one once-only guarded click per stage; each active input retains fresh
  supported evidence, STOP → VM/pointer probe → STOP ordering and checked mapping.
- [ ] Keep game-specific waits explicit, including Spider's configured Play
  game-start delay. Terminal/Solver clicks remain outside game-action counts.
- [ ] Require a fresh supported game/Solver board before resuming ordinary play.
- [ ] Check finite/continuous completion, optional-OK absence, STOP between stages,
  uncertain acknowledgement and failures without retrying prior input.

Done when: terminal command/event sequences and budgets match the baseline;
ordinary recovery and completion decisions remain in each game controller.

## RF04 — centralise common UI STOP revocation

Current owners: ordinary STOP in `src/app.rs` and Pyramid STOP in
`src/app/pyramid_workflow.rs`.

- [ ] Introduce one private operation for latching STOP, retiring observed state,
  clearing RECON consent, revoking acquisition and requesting disconnect.
- [ ] Return a local typed outcome so the ordinary and Pyramid views retain
  their distinct error reporting and active-run ownership handling.
- [ ] Preserve late-event filtering, retained diagnostics and cooperative worker
  shutdown. Do not add post-STOP restoration or cleanup input.
- [ ] Check both STOP buttons, disconnect failure, stale completion events and
  each relevant active-run kind using the existing lifecycle regressions.

Done when: the common revocation sequence has one owner and both views preserve
their existing visible outcomes and cancellation authority.

## RF05 — separate preview delivery from shared worker controls

Current owner: `LatestFrameSlot` in `src/worker.rs`, which currently combines the
preview mailbox with RECON, route, planning, pause/search and effect-setting state.

- [ ] Separate a private bounded frame mailbox from worker shared control/state.
- [ ] Preserve ownership, locking, latest-frame coalescing and contextual events;
  keep GUI polling non-blocking and do not introduce another queue or thread.
- [ ] Preserve direct pause/search/settings access, per-action setting snapshots
  and single-use route/acquisition authority.
- [ ] Check coalescing, stale-context rejection, worker completion, shutdown and
  settings/pause behaviour with existing integration tests.

Done when: preview handoff has a clear responsibility and unchanged delivery
behaviour, without making preview pixels sufficient authority for input.

## RF06 — clarify UI event and request flow

Current owners: `poll_worker` and `dispatch_computed_route(execute: bool)` in
`src/app.rs`; existing `complete_route`/`complete_session` reducers in
`src/app/pyramid_workflow.rs`.

- [ ] Move typed worker-event application into a private UI module, preserving
  stale-context rejection before applying any state-changing event.
- [ ] Reuse the existing completion reducers; preserve run-ID checks, detached
  state and separate RECON, route and session completion semantics.
- [ ] Give planning and once-only execution distinct private entrypoints rather
  than hiding their authority difference behind a boolean parameter.
- [ ] Centralise the repeated idle-preparation gate around the existing tested
  pure helper; preserve snapshot-review, pending-choice and STOP restrictions.
- [ ] Check stale logs versus stale state events, duplicate/late completions,
  failed dispatch, read-only planning and revoked execution/acquisition.

Done when: request/event flow is easier to follow with no changed `WorkerCommand`
contract, input permission or completion outcome.

## RF07 — assess further I/O sharing after the first extractions

- [ ] Recompare Free Cell/Spider capture retention, fresh probes, cancellation,
  waits and run-result publication after RF03/RF05.
- [ ] Propose a further extraction only where evidence and ordering contracts are
  actually identical. Retain decoded frames before analysis can fail.
- [ ] Assess selective Klondike terminal reuse separately; its completion evidence
  and capture return type differ. Do not force it into Free Cell/Spider policy.
- [ ] Leave TriPeaks/Pyramid effect, redeal, restoration and semantic completion
  with their existing owners unless a later proposal is explicitly approved.

Done when: a bounded follow-up proposal is accepted or explicitly deferred. This
milestone does not commit the project to a universal game-controller abstraction.

## Validation and acceptance for each implemented milestone

1. During edits, use narrow owning-package checks and focused regressions. Compare
   behaviour traces, not just line counts or tests mirroring the extracted code.
2. At the completed agreed code scope, run the required controller gates once:
   scoped formatting, locked check, strict private rustdoc, Clippy, full tests
   including ignored tests, and native release. Preserve required package, target,
   feature, toolchain and offline flags from current repository instructions.
3. Record actual results and preserved failures. Use unchanged xSAR receipts only
   when its exact dependency inputs still match; do not claim a fresh library run.
4. Arrange targeted live checks for the affected games and negative paths with
   Charlie's authority. Offline/source review is not live guest acceptance.
5. Obtain milestone acceptance and version approval; align only the approved
   manifest/lock pair, finish required alignment checks, then update canonical
   tracker/handoff evidence and provide the complete owned-path list.

Never change refresh allowances, observation budgets, waits, input order, action
counts, recognition signatures or completion meaning merely to simplify an
extraction. Any intended behaviour change requires its own explicit agreement.

## Edit and decision log

| Date | Milestone | Charlie's amendment / decision | Implementation approval |
| --- | --- | --- | --- |
| 2026-10-31 | All | Initial editable draft from the source assessment | None |

Keep stable task IDs and record deliberate scope changes here. This draft is
separate from the canonical release tracker; it records no completed refactor,
new qualification or new release.
