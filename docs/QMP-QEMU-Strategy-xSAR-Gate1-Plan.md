# QMP-QEMU strategy split and xSAR reuse — Gate 1 plan

Prepared 7 October 2026, Australia/Brisbane. Status: proposal for Charlie's review; no application or crate source edited.

## Intended outcome

At startup, choose who determines the next move:

- **Shortest path route calculation**: recognise a Pyramid deal, calculate an independent minimum-action route, then execute the chosen route through the existing guarded QMP facilities.
- **Computer Vision solving**: retain the current guest Solver/HALO controllers for TriPeaks, Pyramid, Klondike, Free Cell and Spider.

Both ultimately consume captured images. The distinction is independent planning versus following the guest's recommendation.

Frame the broader goal as **route planning relevant to drone navigation**, using the card games as controlled test environments. The reusable process is **observe → plan → advance → detect an impasse → backtrack → replan**. Backtracking changes the live position/state while retaining knowledge of what blocked the earlier route. In the card tests, Undo supplies rollback; a later navigation adapter would return to a previous position. These are different action adapters over the same planning concepts, not a claim that this application already controls drones.

With a fully known model, calculate a shortest route under the chosen cost. With a partially known model, explore, remember revealed information and revise the route when a new constraint appears. Replanning invalidates the affected route; it does not erase the accumulated model.

Separate **Solving strategy** from **Game Type**. A strategy is not another GameMode variant.

## Intake and source discrepancy

GitHub plugin intake pinned the application's main to:

`f076eed1235439b2d2f5c4cd487aaf8fceee84ca`

Package and label: **v1.4.0, candidate 4**.

**HANDOFF REGRESSION — incomplete pushed candidate.** All 11 modified candidate 4 files match their delivered contents, but these three additions are absent from the pinned remote tree:

1. `docs/spider-v1.4.0-candidate-4.md`
2. `tests/fixtures/spider-SP21.png`
3. `tests/fixtures/spider-SP22.png`

The Spider source references both images with include_bytes!, so a fresh test build would lack its inputs. The local delivered copies still exist and match the candidate package. This finding does not establish why the files were omitted. Restore them before the new implementation base is established; do not reset or discard local work.

No fresh clone, refetch, code edits, builds, GitHub writes or public comments were performed for this plan. The current local candidate 4 worktree was inspected read-only and its affected blobs compared with the pinned remote tree.

Applicable companion instructions were recovered and read from `/Rust/AGENTS.md`, uploaded 29 September 2026. Its historical v1.2.0 facts are superseded by Charlie's later requests and current source. Keep the Beast's root AGENTS.md untracked. In particular, Charlie's subsequent instruction not to run rustfmt/cargo fmt supersedes the earlier formatting-check list.

Relevant issue records read: #1 and its completion comment (geometry; stock reconnaissance/Undo All remain future work), #10 and its completion comment (cleanup completed; independent solving excluded), plus the relevant Pyramid/controller issue search results. No issue was reopened or edited. The old #10 body envisaged v2.0.0 for independent solving; that is a numbering precedent, not a current approved release.

## Startup and first candidate behaviour

Permanent explanatory copy:

| Choice | Description |
| --- | --- |
| Shortest path route calculation | Observe the environment, build a route model and calculate a path using Dijkstra or A*. Replan when new information reveals an impasse. |
| Computer Vision solving | Follow the game's built-in Solver highlights using screen captures and QMP input. |

For the initial structural candidate, add a prominent note under the first option:

**Preparation only: capture and inspect Pyramid. Card recognition and route calculation follow in later candidates.**

Both buttons are clickable. The first opens a distinct, read-only **Pyramid preparation** workspace with fresh capture, exact-byte Capture PNG/Save and existing slot inspection. It must not activate Solver, show a HALO action as an independent route, enable gameplay/terminal inputs, or imply that a route was calculated.

The CV option opens the present application flow and starts its existing read-only capture only after selection. Retain all five current game controllers and their existing parameters. Later strategy changes are available while idle, using the current busy/STOP/snapshot gates.

The new independent workspace initially supports Pyramid only. Unsupported game/strategy combinations are explicit at the worker boundary as well as in the UI.

## State and parameter separation

Use a small strategy enum and explicit outer dispatch; do not introduce a large plugin/trait framework before the second planner exists.

| Ownership | Contents |
| --- | --- |
| Shared application settings | QMP socket, capture/save facilities, logging, cancellation and input transport configuration. |
| Computer Vision settings | Current game-specific HALO, Solver activation, settle, reobserve and terminal settings. |
| Independent Pyramid settings | Later: validated rules, Dijkstra/A* selection, route cost and search budgets. Observation/execution timings remain distinct from search limits. |

Preserve current timing snapshots for active runs. Strategy changes invalidate prediction authority, current board bookkeeping, card/deck caches, route caches and advisory progress, while retaining the session log.

Tag requests and their relevant worker results with strategy, game, socket and a generation identifier. Late results from an earlier selection must not become current after switching away and back. Raw preparation captures are not fabricated GuidedAction or NoHighlight results.

## xSAR crate boundary

Pinned xsar repository:

`cbee27824f21d6d04d52adb49a7a0266cfc7ed19`

Its manifest declares version **0.1.0**, edition 2021 and AGPL-3.0-or-later. The official Cargo registry index contains only 0.1.0, not yanked, with no dependencies/features. Published package bytes were not independently inspected.

The repository currently implements telemetry types and formatting, with four tests. QMP, capture and image matching are **new capabilities**, not existing xsar features.

Proposed additions are optional and additive. Preserve the current root telemetry API and dependency-free default. Keep the existing drone Position unrelated to pixel coordinates.

| Move into xsar | Keep in qmp-qemu-socket |
| --- | --- |
| Checked pixel/rectangle geometry and native-pixel to QMP mapping. | Game board layouts, calibrated coordinates and target priorities. |
| Optional Unix QMP client: negotiation, numeric IDs, event filtering, probes, absolute movement, mouse/key delivery, screendump and release-only recovery. | Permission to send an action, D meaning Draw/Recycle, editable game timings and current game controllers. |
| Validated RGBA frame/layout and optional PNG decoding. | Capture lifecycle orchestration, temporary path selection, UI preview, original-byte ownership and save workflow. |
| Generic predicate/palette scans, horizontal runs, blocks and pixel counts. | Gold/white/felt predicates, source/destination distinction, minimum HALO evidence, toolbar bounds and terminal policy. |

Suggested namespaces: `xsar::geometry`, `xsar::qmp` and `xsar::image_matching`, with optional PNG support. Final public API names belong to implementation review.

The first extraction must be functional: application adapters import the crate implementation. Do not create duplicate utility copies that remain unused. Keep game-facing wrapper signatures where practical to avoid broad caller changes.

The existing detector supplies colour-run/block/count operations. This does not establish general template matching, scale/rotation recognition or card-rank recognition.

Public boundaries require checked frame dimensions/stride/buffer arithmetic, explicit ROI errors and deterministic scan ordering. A matcher returns an invalid-input error separately from no match. QMP input preserves release-before-down-acknowledgement ordering and never replays an uncertain press.

Do not promise globally bounded QMP commands solely from current per-I/O timeouts. Public transport work must document that limitation or implement explicit packet-byte and overall command bounds, with focused mock-socket tests.

Keep graph-search/Pyramid rules in application modules initially. Extract a generic search library only after its reusable interface is demonstrated.

## Two-repository integration

Proposed numbering for agreement:

- Application: **v2.0.0, candidate 1** as groundwork towards the independent-solving milestone.
- xsar: **v0.2.0, candidate 1** for the new optional capabilities.

These are proposals, not versions already written.

Validate both repositories as a local pair. Do not commit a Beast-specific path dependency or depend on an unpublished registry version. Supply a guarded local validation override with the coordinated candidates. After Charlie promotes the validated xsar commit, pin the application dependency to that exact Git revision and revalidate the lockfile/integration. A crates.io publication remains Charlie's separate decision; no automatic push or publication is authorised.

Confirm the Beast xsar checkout location before constructing the two-repository installer. The existing application path remains unchanged.

## What can be reused from the Lisp solver

Pinned upstream:

`mchung94/pyramid-solver@c6dde73fd27ec817e2f1329f2a761b384cd8c663`

The actual solver is A* with unit-cost logical moves. Its goal is to clear the 28 tableau cards, not empty every pile or maximise score. Input is a complete known 52-card deal: 28 tableau cards top-to-bottom row-major, then 24 stock cards in first-draw order.

Useful adaptation targets:

- Pyramid exposure rules and legal remaining shapes.
- Stock cursor/waste model and explicit redeal budget.
- King/pair/Draw/Recycle transitions.
- An admissible lower-bound heuristic and conservative impossible-state pruning.
- Upstream transition/solution fixtures, with independent replay.

The heuristic counts remaining tableau kings plus, for each complementary rank pair, the larger remaining rank count. Dijkstra uses the same legal transitions with the heuristic set to zero; A* is the normal execution engine. Compare minimum route costs on manageable fixtures rather than requiring full Dijkstra on every large deal.

Use readable Rust state/action types and a normal priority heap before porting Lisp-specific caches or packed-state optimisations. Upstream's fixed priority bound assumes its exact deck/rules/unit costs.

Retain Mitchell Chung's MIT copyright and permission notice for substantial translated code and fixtures. Correct two source caveats during adaptation: require nonempty waste before recycle, and independently verify that every replayed pair sums to 13.

Proposed route objective: **fewest logical moves**. One king removal, pair removal, Draw or Recycle costs one. A two-click pair input remains one logical move. Fewest QMP packets, fastest elapsed time and highest score are different objectives and must not be presented as this result.

Upstream models both stock top and waste top as playable and permits two redeals. Put those in an explicit RulesProfile; confirm the current guest variant before using them for live planning.

## Recognition and observation boundary

Charlie confirmed the intended acquisition sequence on 7 October 2026: read the play-board cards, step through the Draw pile to determine every card and its order, then use **Undo All**, which returns to the start without penalty. Treat this as the agreed workflow; it was not exercised in this planning workspace.

No working card-rank reader exists in the current application. The removed provisional reader never recognised a rank. Recognition is new work.

The opening Pyramid screen exposes the tableau, but not all 24 stock cards. An unknown stock order cannot support a proven globally shortest route for the actual deal.

Recommended development order:

1. Pure known-deck search using reviewed fixtures or a strict manual 52-card input.
2. Recognition that shows the read cards for correction, including unknown/ambiguous values.
3. Stock reconnaissance with Solver off, followed by Charlie's confirmed no-penalty Undo All restoration.
4. Live independent execution once its card-input contract is confirmed.

The stock/Undo All intent in issue #1 now has Charlie's explicit workflow confirmation. Retain an immutable recognised-deal record across Undo All; reset the live search/execution state to the original board after observing restoration. Preserve Kings in the acquired deal as ordinary recognised inputs; the search decides when to discard them.

The acquisition controller records each card before advancing, uses the existing evidenced D operation and editable observation timing, and ends after a complete validated 28+24-card record. It must identify ambiguous observations rather than append a duplicate or fabricate a card. Undo All and any configured confirmation belong to acquisition, before route execution.

Charlie also confirmed on 7 October 2026 that with **Solver off**, a pair requires clicking **both cards** and a King requires **one click**. A typed pair removal therefore maps to two ordered click gestures, while retaining cost one in the logical route. Preserve STOP between gestures and stop on uncertain delivery rather than replaying the pair or either press.

Before live execution, obtain only remaining missing evidence:

- Any input detail not covered by Charlie's confirmed two-card/one-King rule; do not re-ask the number of clicks for those operations.
- Only the still-missing Undo All confirmation-screen input, if the configured guest presents one; do not re-ask whether Undo All restores the start without penalty.
- Current Pyramid redeal limit and stock-top availability.

Reuse existing geometry/evidence. Do not request a repeat of previously supplied HALO captures. A strict full-deck input allows the algorithm stage to proceed while these observations are collected.

There is no card-pixel changed-count proof in this proposal. Deterministic logical transitions and confirmed input semantics drive the route. Fresh supported scenes, STOP, context invalidation and uncertain-delivery handling remain execution requirements.

## Later games, retained knowledge and reversible exploration

Charlie wants complete-deck reconnaissance first, then probabilistic exploration using single Undo, multiple Undo and Undo All. This extends the shared knowledge/state boundary; it does not add hidden-card planning to the first Pyramid candidate.

Recommended progression is Pyramid, Free Cell, then Klondike, followed by Spider and TriPeaks exploration. This orders the work by observability rather than claiming every deal is winnable.

| Game | Acquisition approach |
| --- | --- |
| Pyramid | Read tableau and ordered stock, then confirmed no-penalty Undo All. |
| Free Cell | Recognise the initially face-up deal, obtaining complete visual coverage. |
| Klondike | Record visible cards and stock; reversible legal moves reveal covered tableau cards before full knowledge is possible. |
| Spider and TriPeaks | Build knowledge through reveals, branch exploration and rollback; use deck counts for deductions and probabilities for unresolved positions. |

Use three small concepts: captured observation, accumulated deal knowledge and live board state. Knowledge is attached to original deal positions/card occurrences, not just current pixel coordinates. Undo rolls back placements, face-up status and pile counters, while preserving reliable identities already learned for the same deal. Seeing a card again after Undo does not count it again. New Game clears the deal knowledge.

Deck counts describe exact remaining multiplicities. For a four-copy rank, three distinct identified occurrences mean exactly one remains unidentified. Under a uniform unknown-position model, its chance of occupying a particular one of U unknown positions is 1/U, not automatically 25%. Actual position probabilities may differ when observations impose constraints.

Use a deck-composition profile. Standard Spider uses two decks (104 cards, eight of each rank); suit multiplicities vary by variant. Confirm the guest's variant before applying that profile. A rank/suit label is not a unique physical card occurrence in a repeated deck.

Record an action/state journal for reversible exploration. A newly revealed identity remains knowledge after rollback, while the live face-up state changes. A failed explored branch is not proof that the whole deal is unsolvable. Keep reconnaissance and Undo costs separate from the final solving-route cost after restoration.

Fully known search can establish a shortest route under its rules/cost model. Partially observed exploration returns a policy or conditional plan, and does not claim global optimality for an unknown deal. This later planner remains outside the first implementation slice.

## Candidate sequence and acceptance

**First candidate: mode separation and actual crate reuse.**

- Startup choices and truthful preparation-only independent workspace.
- Strategy-specific parameter views and authority/context separation.
- Reusable geometry/QMP/frame primitives and small generic matching operations in xsar.
- All five CV game controllers retain accepted behaviour.
- Exact-byte PNG save and temporary-file capture contract retained.

**Second slice: independent Pyramid calculation from known input.**

- Strict 52-card deal validation; explicit rules and logical-action costs.
- Pure legal transitions, Dijkstra reference, A* route calculation.
- Search cancellation and memory/state/time limits.
- Results distinguish optimal solution, proven no solution, budget exhausted, cancelled and missing observation.
- Replayed solution must be legal and clear the tableau. Budget exhaustion is not proof of no solution.

**Third slice: recognition, stock acquisition and independent execution.**

- Show recognised card identities and unresolved values.
- Establish the complete ordered deck/current state.
- Map typed route actions to evidenced guest inputs with fresh observation.
- Keep built-in Solver entirely out of this strategy.
- Reuse established Pyramid completion/restart only where its independent-mode preconditions are satisfied.

Do not combine every UI/worker file into a broad cleanup. Modules may be extracted as each boundary becomes real.

## Validation and delivery

Performed for this plan: read-only source inspection, pinned GitHub tree/blob comparisons, xsar source/registry-index inspection, Lisp source/tests/license review, applicable companion instructions and selected issue/comment review. No runtime inputs or Rust tests executed. Rust/Cargo are unavailable in this workspace.

Implementation checks will be reported as actually run, never inferred:

- Crate feature combinations: existing defaults, QMP, image matching/PNG, and combined features.
- Application locked check/tests/docs/Clippy/release build on available tools; Charlie's Beast remains live acceptance.
- Targeted strategy switching, late-event invalidation, STOP and no independent-path Solver input.
- Existing game fixture and input/release tests across all five CV modes.
- Exact saved PNG bytes and transport command-order/count preservation.
- Focused legal-transition/replay and Dijkstra/A* agreement tests when search is added.
- Guarded installer success, idempotence, unknown-content refusal and rollback for both repositories.

Preserve floating nightly, current Rust 2024/minimum 1.101.0 for the application, existing formatting files and Charlie's double blank lines. Do not run rustfmt or cargo fmt. Choose and actually test xsar's support requirements separately; its current manifest declares no MSRV.

Deliver complete affected files, exact-base guarded installers, manifest, SHA256SUMS, review and validation records. Include all numbered Beast steps together and a fenced version/candidate commit message. Charlie verifies before promotion.

## Code Analysis

This is a design review, not a claim of implemented fixes.

- **Threat surfaces:** QMP packets and captured PNGs enter reusable code; validate packet/image sizes and malformed layouts. Restrict actions through the application worker, not through colour predicates.
- **Memory safety:** use safe Rust and checked indexing/arithmetic; a public frame cannot claim dimensions/stride inconsistent with its buffer.
- **Bounds:** half-open ROIs and widened integer QMP conversion retain current semantics. Bound independent search explicitly and preserve responsive cancellation.
- **Failures:** stale strategy results cannot authorise input; uncertain presses cannot be retried. Unsupported scenes retain the latest frame/logs. Missing cards return an observation requirement. Unknown/budget-limited search must not claim optimality or impossibility.
- **Behaviour preservation:** extraction must retain Spider pointer parking, disabled active-Solver handling, Free Cell toolbar-boundary fixes, editable timings and established terminal flows. It introduces no new card-effect thresholds.

## Primary references

- [Current application commit](https://github.com/xSAR-research/qmp-qemu-socket/commit/f076eed1235439b2d2f5c4cd487aaf8fceee84ca)
- [xsar repository manifest](https://github.com/xSAR-research/xsar/blob/cbee27824f21d6d04d52adb49a7a0266cfc7ed19/Cargo.toml)
- [xsar public API](https://github.com/xSAR-research/xsar/blob/cbee27824f21d6d04d52adb49a7a0266cfc7ed19/src/lib.rs)
- [Official Cargo index record](https://github.com/rust-lang/crates.io-index/blob/d30f5a63be3c02750c27ddb6f593077f37558e71/xs/ar/xsar)
- [Lisp solver](https://github.com/mchung94/pyramid-solver/blob/c6dde73fd27ec817e2f1329f2a761b384cd8c663/src/pyramid-solver.lisp)
- [Lisp tests](https://github.com/mchung94/pyramid-solver/blob/c6dde73fd27ec817e2f1329f2a761b384cd8c663/t/pyramid-solver-tests.lisp)
- [MIT notice](https://github.com/mchung94/pyramid-solver/blob/c6dde73fd27ec817e2f1329f2a761b384cd8c663/LICENSE)
- [Geometry/reconnaissance issue #1](https://github.com/xSAR-research/qmp-qemu-socket/issues/1)
- [Completed cleanup issue #10](https://github.com/xSAR-research/qmp-qemu-socket/issues/10)
- [Spider deck definition, research paper](https://arxiv.org/pdf/1110.1052)
