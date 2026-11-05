# Gameplay functions and exploration workflow — editable proposal

Status: design for Charlie's review. The function catalogue below is a work list,
not an implemented API or authority to operate the guest. Edit the names, order and
contracts before selecting an implementation package. Keep this draft's contents
out of long-term memory and search indexes.

The intended result is an application-owned set of small operations that can be
composed into a deterministic solving workflow. Reuse existing capture, geometry,
QMP and worker services. Each function has one owner and a visible result; callers
should not repeat its checks or hide extra captures, clicks or waits inside it.
The proposed composition module is not part of the foundation-HALO correction.

## Existing foundations

| Capability | Current owner | Reuse boundary |
| --- | --- | --- |
| Decoded frame and TCP capture | [capture facade](../src/capture.rs), [worker TCP capture](../src/worker/tcp_capture.rs), xSAR `tcp_capture` | Native 1920×1080 RGBA, checked layout; persistent connection and frame identity |
| QMP movement, click and D | [QMP facade](../src/qmp.rs), xSAR neutral transport | Checked coordinates; separately acknowledged input stages; no recognition policy |
| Worker I/O, cancellation and latest preview | [worker](../src/worker.rs) | Keep I/O off the egui thread and retain the bounded latest-frame mailbox |
| Solver source targets | [Free Cell](../src/freecell.rs), [Klondike](../src/klondike.rs), [Spider](../src/spider.rs) | Per-game source/scene/selection rules; a destination guide is not a source |
| Rank/suit recognition | [Pyramid vision](../src/pyramid_recon/vision.rs) | Existing calibrated Pyramid reader; other games require explicit capabilities and evidence |
| Option panels | [bounded control search](../src/worker/control_search.rs) | Existing local control recognition and confirmation; retry/click policy remains game-owned |
| Undo All and restoration | [Pyramid RECON](../src/worker/pyramid_reconnaissance.rs) | A supported bounded acquisition protocol; not a generic undo operation |
| Timing policy | [parameters](../src/parameters.rs) | Existing named delays/settings, copied once at the relevant operation boundary |
| Game-independent search and card models | xSAR | Reuse checked card/geometry/search types; a game's rules/state remain separate |

“Shared” below means shared mechanics. A game still supplies its supported scene,
source, expected result, completion rules and allowed recovery operations. Existing
continuous CV/Solver play does not need card-identity or changed-pixel effect gates.
A new card-model solver must establish its own acquisition, replay and restoration
contracts before executing a calculated route.

## Common inputs and results

These are conceptual contracts, not new Rust declarations. Prefer the smallest
private types consistent with the existing owners when implementing a selected task.

- `ObservedFrame`: existing `CapturedFrame`, controller context, capture serial,
  transport/session/request/frame/display-generation identity where available, and
  acquisition timings. Local and transport serials have different namespaces.
- `NativePoint` / `NativeRect`: checked screen coordinates. Conversion to QMP is
  owned by existing xSAR geometry: integer `pixel * 32767 / extent`.
- `CardLocation`: game-specific, checked original location. Pyramid uses fixed
  one-based `row|column`, DL and DR. Free Cell needs its own eight tableau columns,
  cells and foundations, with stack depth where needed. Do not reuse Pyramid's
  triangular 28-slot model for another game or renumber removed cards.
- `Recognition<T>`: `Known(T)`, positively established `Empty`, or `Unknown(reason)`.
  Invalid storage, unsupported scene and transport failures remain typed errors;
  a failed glyph match never means an empty pile.
- `InputPermit`: one-use association of current context, observed source/control,
  exact operation and selected game. This is authority checked by the input owner,
  not a reusable coordinate or screenshot.
- `DeliveryReceipt`: which movement/down/up or key stages were attempted and
  acknowledged. It proves transport delivery, not acceptance or effect in the game.
- `OperationResult`: delivered and observed/confirmed, pending within a fixed
  observation budget, stopped, refused before input, or uncertain delivery. An
  uncertain click/key is never automatically replayed.

All functions return explicit errors with operation, context and last usable
capture information. Expected “not present yet” is a normal recognition/observation
result. Unsupported or malformed evidence must not silently become success.

## Function catalogue

### GF01 — capture_standard

**Input:** selected capture session/backend, controller context, cancellation.
**Output:** one `ObservedFrame`, or capture/layout/transport/stopped error.

Reuse the retained TCP capture client and existing fallback selection. Validate
1920×1080, RGBA format, stride and backing bytes without resizing or colour
normalisation. Preserve transport metadata. Publish the same acquired pixels for
preview/diagnostics rather than requesting another screenshot. No game input,
card reading or routine gameplay PNG recording is part of this function.

**Acceptance:** one request produces one decoded frame; bad layout, stale context,
disconnect and cancellation are distinct; current v3 transport behaviour is retained.

### GF02 — validate_frame_context

**Input:** an observed frame and the expected context/native-layout contract.
**Output:** a borrowed checked view or a typed refusal. Pure; no capture or wait.

Centralise storage, dimensions and context validation so downstream routines can
use checked views. A preview is displayable without necessarily granting input
permission. Do not assume transport identity alone authenticates a restored deal.

**Acceptance:** reject wrong game/context/generation and invalid row storage; do not
modify pixels or elevate an advisory preview into execution authority.

### GF03 — read_card

**Input:** checked frame/view, game-specific location, supported reader profile.
**Output:** `Recognition<Card>` with rank/suit and diagnostic evidence when known.

Read only the requested card-owned region. No socket, screenshot or input. Reuse
Pyramid's calibrated complete-corner reader through its owning module. Declare
other game readers unavailable until their geometry, overlap and font evidence
are qualified. Do not identify hidden cards by elimination or return a guessed
card merely because the model needs it.

**Acceptance:** exact saved positives plus clipped/moving/mixed/unknown negatives;
location errors distinct from unreadable glyphs; all current theme limits retained.

### GF04 — find_solver_source

**Input:** checked frame and selected game's Solver policy.
**Output:** positively recognised typed source, supported no-source observation,
unsupported scene, or structural error. Pure; no input or recovery.

Reuse the current HALO detectors and their priorities. For the authorised
foundation correction, a complete solid foundation source can select one click;
dashed destination outlines remain non-actionable. A missing HALO is not a win
and does not itself authorise Solver refresh, Draw, Undo or a new game.

**Acceptance:** native source fixtures, all four foundation locations with honestly
labelled synthetic coverage, scene/activation prerequisites and existing priorities.

### GF05 — find_option

**Input:** checked frame, a bounded option identifier/rectangle and the current
expected panel/stage. **Output:** absent, awaiting confirmation, one ready local
control/point, ambiguous, or unsupported/error. Pure; no click or extra capture.

Reuse existing bounded control search and complete black-ink/glyph/geometry checks.
A black line alone is insufficient to identify a button. Keep foreground modal
precedence, optional OK versus New Game ambiguity, consecutive-geometry proof and
finite search budgets. Different games may expect different panels and waits.

**Acceptance:** current terminal controls, interrupted/fading captions, underlay
buttons, two ready alternatives and final-observation boundaries.

### GF06 — find_toolbar_action

**Input:** checked frame, game, requested toolbar action (`UndoLast` or `UndoAll`),
and qualified toolbar profile. **Output:** ready icon/point, disabled, absent,
unsupported, or error. Pure; no guessing from a shared colour or location.

Enabled and disabled native icon appearances need independent evidence. Require
the supported scene and no blocking modal. Do not assume “Undo” is lit because
some prior operation was sent. Existing Pyramid Undo All support is a starting
owner to inspect, not proof of support for every game or Undo Last.

**Acceptance:** per-game lit/disabled/missing/overlaid icons and bounds; unsupported
actions refuse before input. Obtaining missing positive evidence is an explicit task.

### GF07 — mouse_left_click

**Input:** current QMP session, native point, one-use permit, snapshotted timing
and cancellation. **Output:** delivery receipt or refused/stopped/uncertain result.

One existing worker input owner checks STOP/context and supported source/control,
probes running VM/absolute tablet, checks STOP again, then uses checked mapping.
Movement, button-down and release acknowledgements remain distinguishable. Use
existing release-only recovery where applicable. This routine performs no
retry, settling, capture or game-state update.

**Acceptance:** one typed click, exact coordinates, STOP at probe boundary,
uncertain-stage accounting and no replay. Do not add a second parallel QMP client.

### GF08 — send_key_stroke

**Input:** existing QMP session, an explicitly supported typed key/QEMU qcode,
one-use permit, key hold and cancellation. **Output:** delivery receipt/error.

Reuse neutral xSAR `press_key`; the application currently exposes Draw as D.
An arbitrary integer keycode is not the contract. Approve/qualify additional
shortcuts separately; do not infer Ctrl-Z or an Undo key from a lit toolbar icon.
No automatic retry or post-key state assumption.

**Acceptance:** D down/up ordering and all existing STOP/uncertain-delivery tests;
unsupported key/game pairs reject without transport input.

### GF09 — observe_operation

**Input:** before-state binding, delivery receipt, game's expected transition,
frozen timing/budget and the same capture service. **Output:** confirmed result,
pending, stopped or refusal with the latest evidence retained.

This is the only composition step that schedules post-input waits and fresh
observations. It calls GF01 and pure readers, without resending the operation.
Keep the game's existing proof: Solver HALO continuation for ordinary CV play;
source disappearance/expected route proof where the card-model controller uses it;
separate completion/terminal evidence. Do not put a universal pixel-change test
into a generic click helper.

**Acceptance:** exact existing per-game cadence, repeated sources, animation gaps,
final-clear/terminal distinction, exhausted budgets and STOP. No hidden recaptures
inside GF03/GF05/GF06.

### GF10 — undo_last

**Input:** current game state/operation journal, explicit rollback capability,
expected parent checkpoint and cancellation. **Output:** restored parent state,
unsupported/unavailable, stopped or an unresolved rollback result.

Composition: fresh supported observation → GF06 positive lit Undo Last → GF07
one click → GF09 bounded restoration observation. Only confirmed restoration
moves the logical position to its parent. Automatic foundation transfers may be
part of one game's undo semantics; determine that contract from evidence. A
transport acknowledgement or enabled icon cannot establish the parent position.

**Acceptance:** native one-step restores, disabled icon, automatic transfers,
wrong parent, partial/uncertain delivery and STOP; no unrequested cleanup clicks.

### GF11 — undo_all

**Input:** explicit game-specific restoration capability, acquired initial-state
identity, journal and cancellation. **Output:** authenticated initial restoration
or stopped/refused/uncertain result.

Compose positive Undo All recognition and one click, any separately proven
confirmation dialog, then complete restoration checks. Keep Pyramid RECON's exact
inventory/order/restoration publication protocol in its existing owner until a
replacement is separately qualified. Never call this routine implicitly after
STOP, an ambiguous click or an unsupported scene.

**Acceptance:** exact original board/deal binding, enabled/disabled/dialog cases,
no input after STOP and no export/planning authority from an unverified restoration.

### GF12 — record_observation

**Input:** immutable observation identity, recognised cards/locations and the
verified logical state transition. **Output:** checked update to a run-owned
knowledge ledger plus a journal entry, or a conflict requiring resolution.

Separate immutable original deal, accumulated known identities and current
occupancy. Keep newly seen identities after an authenticated undo, but attach
them to the same deal/original slot and provenance. Do not promote transient
animation labels, impossible duplicates or a guessed hidden card. Run journals
are private application evidence, not automatic Tree Ring memories.

**Acceptance:** hidden→known→undo retains knowledge; wrong deal/restart/context
revokes the ledger; conflicts and duplicate identities cannot silently overwrite it.

### GF13 — choose_branch

**Input:** game-owned legal state, acquired knowledge, branch history and objective.
**Output:** a legal next logical action, exploration target, solved/dead-end or
budget/STOP result. Pure search; no QMP, screenshot or physical undo.

Reuse xSAR search where applicable with each game's own state/rules. For hidden
cards, distinguish a possible reveal from a verified route. Keep exact/weighted
A* semantics, replay validation, resource accounting and objective choices clear.
Search backtracking is an in-memory operation, not permission to undo the guest.

**Acceptance:** legal replay, deterministic tie choices, duplicate/canonical state
handling, resource stops and game-specific goals; no guessed complete deal.

### GF14 — restore_branch_checkpoint

**Input:** journalled confirmed current/parent states and the game's proven undo
capability. **Output:** authenticated checkpoint or non-resumable refusal.

Select the exact rollback sequence, use GF10/GF11 only where supported, and verify
each restoration before reusing a branch. If automatic transfers, stock order or
recycle limits cannot be restored, declare that branch physically irreversible.
Never compensate for uncertain input by trying another input.

**Acceptance:** reversible and irreversible branches, no-op undo, stale context,
wrong stock order and STOP; retained knowledge stays separate from live position.

### GF15 — run_exploration

**Input:** explicit run consent, game capabilities, checked acquisition, objective,
limits and timing snapshot. **Output:** completion, bounded stop, or precise failure
with latest evidence and a non-resumable input permit.

State sequence: acquire → recognise → record → choose → authorise → deliver once →
observe → commit. At a supported dead end: choose a checkpoint → restore → verify →
choose an untried branch. Every transition names its current state, observation,
logical action, delivery outcome and next permitted operation. Unknowns stop or
consume only the specified input-free observation budget. UI remains responsive;
logs describe current versus historical failures and actual attempted/acknowledged
inputs. Completion is independently observed, not inferred from an empty plan.

**Acceptance:** representative full workflows plus replay/STOP/failure fixtures;
read-only planning never starts execution; stale UI events cannot regain authority.

### GF16 — click_option (bounded composition)

**Input:** current context/capture service, expected option stage and bounded
rectangle/profile, timing and cancellation. **Output:** not-ready/refused, stopped,
uncertain delivery, or a receipt with the observed next-stage result.

This is the convenient single call corresponding to the example `click_option`.
It obtains or accepts one explicitly fresh checked frame, calls GF05, authorises
only the one ready expected control, calls GF07 once, then GF09 within that stage's
existing wait/observation budget. Its internal acquisitions are counted in the
returned trace. It neither searches the whole screen nor retries an uncertain
click; an ambiguous OK/New Game pair returns without input. A caller can use the
pure GF05 separately when it only needs recognition.

**Acceptance:** ready/not-ready/ambiguous/disabled/foreground-modal cases, one click,
exact permitted captures/waits, new-stage confirmation and STOP between probe and
input. Existing terminal stage owners remain authoritative until this composition
is separately selected and qualified.

## Timing and error ownership

Use the existing `AnimationSettleDelays`/`StepRunSettings` owners in
[parameters](../src/parameters.rs). Give any genuinely new delay a named duration,
units, applicable operation, measured provenance and one owner. Freeze settings
per operation/batch. Recognition and control-location functions contain no sleeps.
The observer owns bounded scheduling; transport owns key/button hold durations.
Log actual waits/acquisition/recognition time rather than claiming a configured
delay is measured performance. A minimum reliable delay requires repeated live
measurement and margin; no arbitrary lower timing is proposed here.

Expected error groups: capture/storage/layout, unsupported capability/scene,
unknown/ambiguous recognition, disabled control, stale context/permit, cancelled,
transport refused before delivery, uncertain partial delivery, observation budget,
restoration mismatch and publication uncertainty. Preserve each at its owner;
attach context at composition boundaries instead of string-matching generic errors.

## Proposed implementation packages — select after editing

| ID | Package and result | Prerequisites / review boundary |
| --- | --- | --- |
| GP00 | Agree capabilities, types, owners and trace-based acceptance for GF01–GF16 | Review this editable proposal; select one game/path first |
| GP01 | Thin checked capture/view and once-only click/D wrappers (GF01/02/07/08) | Reuse existing services; preserve traces and STOP; no new transport |
| GP02 | Pure location/card/control/toolbar adapters (GF03–GF06) | Qualify each game's actual evidence, especially missing Undo/foundation sources |
| GP03 | Observation scheduler and journal/knowledge owner (GF09/12) | Preserve per-game expected-state/cadence contracts; private run state |
| GP04 | One proven Undo Last / Undo All restoration path (GF10/11) | Native enabled/disabled/confirmation evidence and authenticated restore |
| GP05 | Offline branch selection and checkpoint model (GF13/14) | Game-specific legal replay and explicit irreversibility |
| GP06 | One complete exploration workflow (GF15) | Prior packages, UI consent/retirement and full negative regression matrix |

Each selected code package gets focused owning tests while editing, then the
required full application gates and any affected crate integration once complete.
Only changed, qualified revisions earn a release increment. Host mocks/fixtures,
recorded guest evidence and fresh live acceptance remain separate report items.
The catalogue does not commit to finishing every package before Free Cell 3.0.0:
its complete visible initial layout and A* solver may need fewer exploration/undo
capabilities than hidden-card games. Agree that scope before implementation.

## Review and edit log

| Date | Decision / change | Selected implementation scope |
| --- | --- | --- |
| 2026-11-04 | Initial catalogue prepared from current source and Charlie's examples | Foundation-HALO correction only; proposed composition functions await review |
