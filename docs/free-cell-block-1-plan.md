# Free Cell Block 1 implementation plan — T=0s RECON

Status: planning artefact for review. Implementation has not started.

Charlie has approved **v3.0.0 as the first Free Cell build**. This document does not change a manifest or create a qualifying build. Approval to implement Block 1 remains a separate gate.

## 1. Intended result

Add independent, read-only recognition of one untouched Free Cell deal. One explicit RECON request acquires **one fresh native TCP v3 frame**, recognises all 52 actual rank-and-suit identities, validates the initial arrangement, and produces a strict, bounded Free Cell deal document. No guest Solver activation or gameplay input is needed.

The accepted initial arrangement is eight physical PLAY columns in left-to-right order, with lengths **[7, 7, 7, 7, 6, 6, 6, 6]**, four positively empty CELL positions and four positively empty SUIT positions. Cards remain in top-to-bottom order within each column.

T=0 denotes the untouched initial position, not zero capture latency. The operator supplies a fully dealt, unplayed board; this request does not click New Game or wait through repeated captures to manufacture one.

Block 1 ends at a validated exported deal and an application result tied to the originating request. It does not calculate a route, apply Horne closure, select a partial route, execute a move or restart a game. These remain sequential later blocks.

## 2. Documents reviewed and their relationship

| Input | What it contributes | Planning interpretation |
| --- | --- | --- |
| [free-cell-target-blocks.md](free-cell-target-blocks.md) | Four delivery passes, reuse list and model-test expectation | Pass 1 is Block 1; Pass 2 is Block 2; Pass 3 is Block 3; Pass 4 groups Blocks 4 and 5. The master specification still defines five sequential blocks. |
| [free-cell-master-spec.md](free-cell-master-spec.md) | Whole workflow and contracts across all five blocks | Block 1 supplies the immutable, physical initial state consumed by later planning. Future canonicalisation must not reorder this deal. |
| [free-cell-block-1.md](free-cell-block-1.md) | Detailed T=0 requirements and bounds | This is the implementation scope: one frame, complete identity extraction, empty top areas, capacity limits and exact deck validation. |
| [free-cell-rust.md](free-cell-rust.md) | Reference pseudocode for Blocks 2–4 | Use it to identify future interface requirements and defects to resolve later. It is not a ready-to-compile Block 1 implementation. |

Reviewed application revision: **b67d93942c68c55bfebe90e0f4e1e03f57300740**, version **2.9.153**. Reviewed xSAR revision: **d20dca8ad2320dcbbf869e22b35a2583317d4569**, version **0.9.0**.

These revisions describe the inspected baseline, not a proposed Git dependency pin. The application currently uses a local xSAR path dependency. The worktree specifications are the authoritative planning inputs even though they are not part of those commits.

The older [Free Cell Gate 1 record](freecell-gate-1.md) describes Solver-led behaviour and existing evidence. It is historical context, not authorisation for the new independent workflow. The similarly named free-cell-block-1-recon.md is currently empty and supplies no additional contract.

## 3. What the current source actually provides

| Area and evidence | Existing behaviour | Planned reuse |
| --- | --- | --- |
| [Free Cell layout](../src/freecell.rs#L23) and [scene detector](../src/freecell.rs#L205) | Native CELL/PLAY x origins, card width and toolbar boundary; coarse scene detection deliberately does not recognise rank, suit or SUIT occupancy | Reuse measured layout anchors through a narrow shared interface. Add a separate complete-start recogniser; the coarse scene detector alone cannot prove T=0. |
| [Free Cell execution](../src/worker/freecell_execution.rs) and [terminal controls](../src/freecell_terminal.rs) | Existing guest Solver/HALO path and post-game flow | Preserve their behaviour and existing tests. These are later execution references, not a source of recognised card identities. |
| [Shared capture facade](../src/capture.rs#L6), [worker capture selection](../src/worker.rs#L4381), [TCP adapter](../src/worker/tcp_capture.rs#L267) | xSAR transport, retained client, explicit wire mode, cancellation, frame-layout checks and capture timing | Reuse this path for exactly one full-frame acquisition. Do not add a TCP client or PNG round trip. |
| [Checked geometry facade](../src/geometry.rs#L1) | xSAR points/rectangles and widened checked QMP mapping | Use checked crops and coordinate arithmetic. Block 1 does not need to send mapped coordinates. |
| [Pyramid observation](../src/pyramid_recon/vision.rs#L577) and [private CardReader](../src/pyramid_recon/vision.rs#L223) | Independent scene recognition, explicit rank/suit evidence, unique candidate acceptance and Unknown versus Empty | Reuse the recognition design and xSAR matching primitives. The private Pyramid reader and its profile are not a public Free Cell recogniser. |
| [Pyramid card matching](../src/pyramid_recon/vision.rs#L1185) and [graded joining](../src/pyramid_recon/vision.rs#L1383) | Card identity is accepted only from measured matching evidence; colour and ambiguity checks remain explicit | Use measured Free Cell corner regions and labelled templates. Share a small pure matching helper only where code and pixel semantics are demonstrably the same; retain Pyramid wrappers and thresholds. |
| [Pyramid worker adapter](../src/worker/pyramid_reconnaissance.rs#L267) | Request validation, context revocation, cancellation, diagnostics and guarded publication | Reuse the adapter pattern. Do not invoke its draw/Undo/confirmation scan: Free Cell exposes the full initial deal in one frame. |
| [Pyramid publication boundary](../src/worker/pyramid_reconnaissance.rs#L979) | No-overwrite publication with distinct absent/published/uncertain outcomes and no blind retry | Apply the same publication contract to a Free Cell codec, retaining all Pyramid publication regressions. |
| [Strategy contract](../src/strategy.rs#L89) and [RECON UI gate](../src/app.rs#L3466) | Independent preparation currently admits Pyramid only | Add an explicit Free Cell read-only branch. Preserve the existing Pyramid gate; do not make every game eligible or enable general input. |

xSAR already owns the neutral identities: [Rank](../../xsar/src/games/cards.rs#L11), [Suit](../../xsar/src/games/cards.rs#L134) and [Card](../../xsar/src/games/cards.rs#L172). Card fields are private; consumers use Card::rank(), Card::suit() and Rank::value(). The application src/cards.rs contains calibrated screen regions, not a second identity model.

xSAR's [generic controlled A*](../../xsar/src/search.rs#L845), [best-effort search](../../xsar/src/search/best_effort.rs#L355), execution and QMP modules remain future integration points. Block 1 must not create another search engine, transport or card enum merely to prepare a deal.

## 4. Ownership and proposed files

One implementation owner should make the approved changes. No parallel source edits are needed.

| Owner | Proposed change surface | Contract |
| --- | --- | --- |
| xSAR | New src/games/freecell.rs, or a small module directory if needed; src/games.rs export | Reusable checked stacks, stable physical locations and validated initial-deal model. Reuse games::cards identities. |
| xSAR | New src/freecell_json.rs and codec tests; src/lib.rs and Cargo feature declarations | Strict bounded read/write API for the Free Cell initial deal. No GUI, socket selection or input authority. |
| xSAR | tests/freecell_models.rs | Meaningful public-API model tests corresponding to the target document's requested test binary. |
| Application | New src/freecell_recon.rs with focused profile/vision/test modules | Game-specific native geometry, measured glyph recognition, positive empty evidence and typed recognition failures. |
| Application | New src/worker/freecell_reconnaissance.rs; narrow wiring in src/worker.rs | One capture, context/STOP checks, request/result identity, export ownership and bounded diagnostics. |
| Application | Narrow changes in src/app.rs, src/strategy.rs, src/main.rs and relevant parameter definitions | Free Cell RECON under A* Route Solution, worker dispatch and result presentation. Existing Computer Vision solving remains available. |
| Application | Focused native fixtures and fixture manifest under tests/fixtures/ | Original evidence, labels, measured profile provenance and independent held-outs. |
| Documentation | Verification/live-test record, milestone receipt, active handoff and release notes when implementation qualifies | Actual results, accepted scope, dependency revision and v3.0.0 decision. |

Propose feature-gated Free Cell support in xSAR, following its existing cards/Pyramid pattern: a model feature depends on cards, and a JSON feature depends on the model plus the existing optional serialisation dependencies. Keep default features dependency-free. Confirm the exact feature names and public API before editing the sibling crate.

Publication reuse may require extracting the existing private byte-publication mechanism from the Pyramid codec into a neutral internal helper. This must preserve the public Pyramid API, permission model, failure classifications and tests. It is a targeted shared implementation change, not a replacement file-handling framework.

Application-native calibration stays in the application. Where existing Free Cell layout constants are needed by both paths, expose one authoritative definition rather than copying numeric arrays. New rank/suit offsets and vertical spacing require their own measured evidence.

## 5. Model contract

### 5.1 Shared identities and bounded storage

- Use the existing xSAR Card, Rank and Suit types.
- Introduce a CascadeStack with private storage and a hard capacity of **20 cards**, as required by the specification. Construction and every push reject overflow; an empty pop or invalid index returns a checked failure.
- Storage can use a fixed array of optional shared Card values plus a checked length. Unused slots are empty; do not fabricate an Ace as an unknown card.
- Define a BoardState with the full shape needed by later blocks: eight cascades, four optional CELL cards and four foundation progress values in 0..=13. Block 1 constructs only the initial form.
- Provide a validated InitialDeal wrapper whose construction enforces all T=0 invariants. Keep the original deal immutable; later mutable planning positions and recorded routes are separate values.
- Define physical location types now, but defer legal move generation, Horne closure and route execution.

### 5.2 Ordering and indices

Internal PLAY indices are **0..7**, matching the new specification. Each cascade is top-to-bottom; index 0 is the first visible card and the final occupied index is the exposed bottom card. The first four cascades contain seven cards; the last four contain six.

Existing user-facing names remain **PLAY 1–8, CELL 1–4 and SUIT 1–4**. Convert at explicit display/API boundaries. Do not change Pyramid's one-based row|column conventions.

Use an explicit suit-to-foundation mapping, proposed as Clubs, Diamonds, Hearts, Spades. Physical SUIT slots remain separately described by the native profile; do not assume a visual left-to-right suit order without measuring it.

Sorting cascade hashes or CELL identities is later search-key behaviour. It must never change physical recognition order, exported card order or future execution locations.

### 5.3 Exact 52-card validation

Use an explicit suit index and Rank::value() to calculate:

    card_id = suit_index * 13 + (rank_value - 1)
    full_deck_mask = (1u64 << 52) - 1

All arithmetic and shift indices are checked. Before setting a bit, reject a bit already present and report both physical locations. A successful InitialDeal requires exactly 52 recognised cards, the exact initial column lengths, four empty cells, four zero foundations and seen_mask == full_deck_mask.

Validate the same invariants at both recognition completion and JSON intake. Deck completeness is a consistency check; it must never be used to infer an unreadable final card.

Unknown, ambiguous and Empty are distinct. An unreadable card is not an empty slot; a failed empty-slot template is not proof of occupancy or permission to proceed.

## 6. Native geometry and recognition

### 6.1 Supported first profile

The initial target is the existing native **1920×1080, 100% display/text scale, CLASSIC BLUE** environment. The existing Free Cell source records PLAY x origins [218, 410, 603, 795, 988, 1180, 1373, 1565], CELL x origins [166, 358, 551, 743], face width 137 and toolbar boundary y=947.

These are useful measured anchors, not complete T=0 calibration. Measure and record:

1. The eight first-card y origins and the exposed-strip vertical stride DeltaY.
2. Rank and suit rectangles relative to each card origin, for every initial strip and bottom face.
3. Four CELL empty regions and four SUIT empty regions, including positive decoration and exclusion of card material.
4. A complete supported Free Cell start-scene signature, including exclusion of dialogs, menus, Solver overlays and dealing animation.
5. Any bounded native alignment variants and the original evidence supporting them.

Use checked integer arithmetic for origin + depth * DeltaY. Every crop must remain inside the native frame and its intended visible strip; a short strip must not sample the card beneath it. Detect wrong column counts, extra cards below expected initial stacks, clipping and unsupported spacing rather than silently reading only the expected 52 locations.

The existing Free Cell coarse scene detector accepts partial slot-row evidence. Retain it for its existing callers; require the complete new start contract for independent RECON.

### 6.2 Identity extraction

Read rank and suit from each measured corner independently, using existing xSAR frame/crop/matching operations. Templates must have explicit labels and measured quality/ambiguity limits. Rank/suit ink colour must agree with the accepted suit.

The exact-black black_run helper may assist alignment only where the evidence supports exact black pixels. It cannot identify red suits or establish card identity by itself. Its no-hit (0,0) sentinel must not become a valid inferred origin in these non-origin regions.

The RGB comparison helper measures differences; it is not a complete card classifier. Wrap it in explicit labelled-candidate acceptance, retaining no-match and multiple-match failures.

Existing Pyramid glyph banks may be reused only after Free Cell evidence proves the same native typography, scale, crop and colour semantics. Do not assume the shared card artwork, resample images, relax limits or extend arbitrary-font support. Prefer small shared pure evaluation code; keep game-specific profiles and scene policy separate.

### 6.3 Calibration evidence

Start by inspecting the original FC01 and FC13 full-frame fixtures described by the historical intake. They are candidates for initial-board evidence, not an already labelled 52-card Free Cell dataset. Verify actual pixels and every card label before using them.

Retain independent whole-board held-outs across different deals. Record rank, suit, ink-colour, top-strip and bottom-face coverage. Obtain missing exemplars as a prerequisite to implementation acceptance; do not fill coverage gaps by elimination or synthesis.

Include negative cases for occupied CELL/SUIT areas, dealing animation, Solver overlays, unsupported artwork/scale, clipped glyphs and confusing rank/suit pairs. Synthetic corruption tests supplement original native frames and are labelled as synthetic.

### 6.4 Structured failures and resource bounds

Use typed failure categories carrying the relevant physical column/depth or CELL/SUIT slot, profile identity and matching region. Distinguish invalid frame layout, unsupported scene, invalid initial geometry, unknown/ambiguous identity, rank/suit colour disagreement, occupied/unknown top slot, duplicate identity, revoked context and publication outcome. A user-facing string is formatted from that structure; retain the first underlying failure.

Bound profile bytes, template counts, crop dimensions, alignment alternatives and diagnostic candidate lists before recognition or allocation. Reuse the existing bounded matching/profile conventions where applicable. Check cancellation between bounded units of recognition; do not add an unbounded scan or per-frame image archive.

## 7. Worker and UI flow

    Select Free Cell + A* Route Solution
      → validate idle request, destination and current context
      → require the existing TCP v3 capture route
      → acquire one fresh frame
      → check STOP/context
      → validate scene, geometry, empty areas and 52 identities
      → validate InitialDeal and encode bounded JSON
      → check STOP/context immediately before publication
      → publish once and return a context-bound result

A single frame means one capture acquisition per RECON request, not one image repeatedly refreshed until it looks acceptable. If that frame is incomplete, ambiguous or animated, refuse with its diagnostics. A later explicit request is a new observation; no guest input is replayed.

Use the existing backend selection and persistent v3 client. The current default is v3-persistent. If another wire/backend is selected, report that Block 1 requires the v3 route; do not silently mutate configuration, fall back to file-backed QMP capture or create a new transport.

Reuse immutable ControllerContext and generation-based revocation. Check STOP/context before acquisition, after acquisition, after bounded recognition and immediately before publication. Reject stale results in the UI, including a selection changed away and back. Cancellation revokes further work; it does not promise instantaneous interruption of an in-progress system call.

ControllerContext::validate currently rejects independent Free Cell. Extend only the supported preparation combination and its documentation. Keep ordinary ShortestPath input permission disabled; distinguish the new Free Cell RECON command from Pyramid reconnaissance and its separate input authority.

Reuse idle/busy/snapshot guards and the worker's bounded latest-frame ownership. The result should show the physical eight-column inventory, empty-area checks, profile identity, capture information and export outcome. Planning/execution controls remain unavailable for this new path until their blocks are approved.

No Solver click, source click, D key, Undo, confirmation, restoration or terminal transition belongs to this flow. Socket-fixture tests must prove the absence of input-send-event and other gameplay commands, rather than merely asserting an empty local list.

A published file is offline deal data. Neither a serialised flag nor its profile/source strings authenticate current guest state or grant future execution permission. Later execution must establish its own fresh, same-context authority.

## 8. JSON and publication contract

Propose a Free Cell schema version **1**, independent of Pyramid schema 2:

| Field | Contract |
| --- | --- |
| schema | Exactly 1. |
| game | Exactly freecell. |
| cascades | Eight arrays in physical order; initial lengths [7,7,7,7,6,6,6,6], top-to-bottom shared card identities. |
| cells | Exactly four explicit empty values for Block 1. |
| foundations | Explicit suit-keyed values, all zero for Block 1. |
| provenance | Bounded deal/profile/source metadata. Descriptive data only; no execution authority or absolute host configuration. |

Provide checked from_slice, read_from, to_vec and write_new interfaces following existing codec conventions. Use duplicate-aware deserialisation; a generic map that has already discarded duplicate keys cannot enforce this contract. Reject unknown/duplicate fields, malformed or incomplete identities, unsupported schema/game, invalid lengths, nonempty initial top areas, duplicates and trailing non-whitespace data.

Adopt a **65,536-byte maximum document size**, with bounded reading before deserialisation and bounded output. Document the new Free Cell constant independently of whether the Pyramid JSON feature is enabled.

Validate and serialise before touching the destination. Reuse same-directory private temporary-file creation, restrictive permissions, atomic no-overwrite publication and durability handling. Preserve filenames exactly, including spaces, Unicode and leading hyphens through Rust path APIs. Do not trim, reconstruct or pass them through a shell.

Expose separate NotPublished, Published and Uncertain outcomes, including cleanup/directory-sync errors after publication. Refuse an existing destination. After any uncertain or published error, retain the first failure and inspect the destination before a new user-requested operation; do not retry automatically.

Reusing publication does not authorise a new ordinary-gameplay recording path. Keep the existing --image GAME WIN boundary. Use the same in-memory frame for preview/recognition; no second capture for diagnostics. Additional persistent Free Cell image diagnostics need an explicitly agreed boundary.

## 9. Implementation sequence and reviewable outputs

| Step | Work | Observable completion |
| --- | --- | --- |
| B1.1 — agree contracts | Confirm sibling API/features, schema, supported native profile and the Free Cell-only read-only UI route | Approved ownership/API/file list, v3.0.0 target and measured evidence requirements; no permission for Blocks 2–5 implied. |
| B1.2 — model | Implement checked stack/location types and InitialDeal validation in xSAR | Public model tests prove bounds, exact inventory, initial shape and physical ordering. No search engine changes. |
| B1.3 — codec/publication | Implement bounded strict Free Cell JSON and share the proven neutral publisher where necessary | Round-trip and negative codec tests; no-overwrite, permissions and failure-stage tests; Pyramid publication regressions unchanged. |
| B1.4 — native evidence/profile | Inspect and label original start frames; measure all corners, spacing and empty areas; reserve held-outs | Calibration manifest with original evidence, profile bounds and actual coverage/gaps. |
| B1.5 — pure recognition | Implement complete-start and all-card recognition using the measured profile and shared primitives | Whole-frame positives, held-outs and refusal regressions. No card identity inferred from deck elimination. |
| B1.6 — worker/UI integration | Add typed read-only request/result, one-frame capture, STOP/context checks and export presentation | Actual consumer/socket tests prove one acquisition, current-context result and zero guest input. |
| B1.7 — qualification | Run the narrow iterative loop, then one complete required Tier 2 pass | Recorded actual results, preserved first failures and unchanged existing behaviour/tests. |
| B1.8 — version/acceptance | Present host evidence, obtain milestone acceptance, apply the approved major transition and align lock/current labels | Application v3.0.0 with a locked metadata check and version-correct native artefact; separate read-only guest acceptance and updated receipts. |

Keep source edits scoped to the owning package. Use one implementation owner; seek one independent review if warranted by the shared publisher or recognition changes. Do not start repeated review cycles without a concrete unresolved issue.

## 10. Acceptance and regression matrix

| Requirement | Evidence required for Block 1 |
| --- | --- |
| Shared identities and capacity 20 | Use actual xSAR Card values; test empty/one/twenty/twenty-one boundaries, checked underflow and invalid indices. |
| Exact initial layout | Tests for [7,7,7,7,6,6,6,6], wrong lengths/order, extra/missing cards, occupied CELL and nonzero foundation cases. |
| Exact 52-card bijection | All 52 unique identities accepted; duplicate first/last identities, unknown/ambiguous cards, invalid rank/suit and malformed counts rejected. |
| Physical ordering | Round-trip and recogniser tests compare every card and original column/depth; no canonical sorting leaks into the deal. |
| Checked geometry | Native edge and arithmetic-overflow tests; stride/pixel-layout failures, bad crop, unsupported scale, clipped/extra rows and no-hit alignment refuse. |
| Recognition quality | Independently labelled native frames and held-outs; bounded threshold/ambiguity/colour tests; coverage gaps explicit. |
| Positive emptiness | All four CELL and all four SUIT regions positively pass; hidden/unknown or occupied areas refuse. |
| One v3 frame | Actual worker/server fixture counts one acquisition and checks expected wire mode. No implicit retry/fallback or extra diagnostic capture. |
| No input authority | Actual socket trace contains no gameplay/input commands; no Solver, Undo, D, click or terminal dispatch. |
| STOP and stale contexts | Before/after capture, before result/publication and change-away-and-back cases; no stale success enables later controls. |
| Bounded codec | Public model/codec tests, 65,536-byte boundary, over-limit stream, duplicate/unknown fields, bad schema/game and trailing content. |
| Publication integrity | Existing destination, invalid path, exact unusual filenames, restrictive permissions, partial failure/cleanup and durability outcome tests. |
| Consumer integration | Application uses the new sibling API and feature set; existing Pyramid, search, best-effort, TCP v3 and HALO game regressions pass unchanged. |
| Live read-only result | Fresh Windows 11 guest start at the supported native settings; operator compares the exported 52 cards/physical order and eight empty top areas; retained evidence proves one capture and zero gameplay input. |

New tests should prove these behaviours or credible failure modes. Do not change existing Pyramid/search/TCP tests or recognition thresholds to make the new path pass.

Live acceptance is separate from offline fixtures. It requires an agreed running build and operator-controlled untouched Free Cell board. This planning request does not authorise GUI launch, START or guest input.

## 11. Verification and version policy

The baseline host receipts remain retained evidence: application **1,522 tests**, xSAR **296 tests and three Miri checks**. They are not new Block 1 results and do not qualify modified source.

During implementation, use the repository-selected nightly toolchain, edition 2024, MSRV 1.101.0 and existing formatting. Run only owning-package locked checks and focused tests while editing. The target document's model command becomes an actual owning-package test, for example cargo +nightly test --locked --package xsar --test freecell_models with the agreed feature selection and host target.

At Block 1 completion, run the full required Tier 2 scope once per changed owning package: scoped formatting, locked check, tests including Beast's --include-ignored scope, strict private rustdoc, Clippy and native release build. For xSAR, include default/minimal-feature and Free Cell feature checks, existing feature combinations and actual consumer integration. Follow current repository commands rather than copying a stale toolchain or workspace-wide formatter invocation.

The existing three Miri results can only remain cached for unchanged applicable code. Decide any new Miri coverage from the concrete storage/ownership changes; do not label the new model Miri-verified on the strength of an old receipt.

Charlie has already approved **application 2.9.153 → 3.0.0**, with both minor and patch reset to zero. Do not ask again for that numeric classification. Apply it after the qualifying Tier 2 pass and milestone acceptance, following the repository gate; then synchronise the application manifest, own lock entry, release history and current documentation/labels. RELEASE_LABEL already derives from CARGO_PKG_VERSION.

Metadata-only alignment then needs a fast locked package check. Produce a native executable whose embedded label is 3.0.0 before claiming a runnable v3.0.0 build; a final aligned build is distinct from rerunning the unchanged full suite.

The sibling xSAR version is independent. Agree its additive API/feature release classification and number before changing it; the application-major approval does not silently authorise a library increment.

Documentation-only planning earns no increment. Failed or unchanged repeated checks earn none. Later blocks need their own acceptance and version decisions.

Record actual HEAD, parent, porcelain state, dependency revision/cleanliness and results in docs/milestones.md and the active handoff. Preserve unrelated files and the index. Publication, installation and registry release remain separate decisions.

## 12. Later-block findings to retain, not implement now

| Finding in the reference pseudocode/specification | Consequence for later planning |
| --- | --- |
| Card fields are used directly and as numeric bits | Adapt to the shared private-field accessors and explicit rank/suit encoding. |
| Sorted u64 column hashes are treated as complete equality | A hash collision must not merge distinct states. Define exact canonical state equality, while preserving physical execution indices. |
| The proposed heuristic adds unbanked cards and buried depth while autoplay costs zero | Shortest-route admissibility/consistency is not established. Shared blockers can also be counted repeatedly. Prove an admissible lower bound or use a clearly labelled non-optimal policy before making shortest-route claims. |
| Search limits are 500,000 nodes / five seconds and capacity reservations are 65,536 | These are reference defaults/allocation hints, not measured acceptance budgets or hard memory limits. Enforce limits before allocating every child and bound retained action/arena storage. |
| Arena indices use unchecked len as u32 and reconstruction indexes directly | Use checked conversion and checked parent traversal with structural bounds. |
| Heap Ord omits fields considered by derived Eq; stale entries are not visibly discarded against best g | Define consistent queue ordering/equality and explicit stale-entry/reopening behaviour. |
| “Maximum partial solution” is requested under finite budgets | Report best observed under the declared objective and limits unless exhaustive evidence proves a global maximum. Preserve the required replay validation. |
| Horne closure and auto-move skipping assume guest behaviour | Retain an explicit auto-versus-deliberate trace and validate it against the actual guest policy before execution. |

These findings do not expand Block 1 into search or execution. Its stable output contract prevents those later choices from corrupting original card order or authorising input.

## 13. Block 1 completion decision

Block 1 is complete only when the approved model/codec/recogniser/worker path exists, all acceptance rows have evidence, Tier 2 passes, the application is aligned to the approved v3.0.0, and the required read-only guest acceptance is recorded.

For the present task, the deliverable is this reviewed implementation plan. The next implementation decision is approval of Block 1's file/API/feature scope, with native calibration evidence prerequisites and the independent xSAR version decision explicit.

## 14. Retained contracts for Blocks 2–5

This continuity supplement was saved with the complete Block 1 plan while the separate Pyramid 2.9.155 correction was authorised. Free Cell implementation remains unapproved. The first qualifying Free Cell application build remains the approved **3.0.0**; xSAR's version is independent.

The additional [Block 2](free-cell-block-2.md), [Block 3](free-cell-block-3.md), [Block 4](free-cell-block-4.md) and [Block 5](free-cell-block-5.md) specifications refine the future contracts below. The combined reference [free-cell-block-2-3-4.md](free-cell-block-2-3-4.md) is byte-identical to [free-cell-rust.md](free-cell-rust.md), SHA-256 `bf4f32a0d6d34fa5ab0832696a0b82c90676b780a0618d12b373bd0bd1a472c8`; these are two copies of one reference, not independent validation.

| Future concern | Contract and unresolved implementation decision |
| --- | --- |
| Initial Horne closure | Apply closure to a copy of the original physical board before expanding the root. Record the root's automatic moves and include them in replay and execution reconciliation. Automatic moves have zero deliberate cost; an auto-only prefix must not disappear when a route is reconstructed. |
| Deliberate moves | Include legal foundation moves that fail Horne's safe-autoplay predicate. The generator's specified order is explicit foundation, cascade to non-empty cascade, cell to non-empty cascade, cascade/cell to the first empty cascade, then cascade to the first empty cell. Never relocate a lone cascade card to an empty cascade merely to permute physical columns. |
| Physical identity and symmetry | Keep physical column/cell positions in the live state and route. Canonical transposition keys may sort equivalent columns/cells only for search comparison. Equality must compare actual canonical state, not only a 64-bit hash. |
| Incremental hashes | Refresh every column changed by automatic closure as well as the deliberate source/destination. Closure can pop several other columns; the reference's two-column update alone leaves stale keys. |
| Complete and partial replay | Before returning either outcome, replay from the original physical board, including root and interleaved automatic moves. Verify legality, final foundations, deliberate move count and reported cards banked. The Block 3 specification requires this even though the reference's complete and partial return paths omit it. |
| Best partial ordering | Compare foundation count descending, deliberate cost ascending, then buried depth ascending. Budget-limited output is the best observed candidate under this ordering, not a proven global maximum. |
| Search stopping reasons | Distinguish full clear, exhausted graph, node budget, elapsed budget, cancellation and invalid input. Decide whether the tracker covers expanded nodes or every generated candidate; the current reference tracks expanded nodes and can stop with unevaluated children queued. Do not silently call both policies equivalent. |
| Checked cost and capacity | Use checked additions for deliberate `g` and priority `f`, checked arena/index conversions, explicit allocation errors and budgets checked during child generation. A capacity hint is not a hard resource bound. Retain the separate concerns about stale heap entries, collision equality and `Ord`/`Eq` consistency in section 12. |
| Search optimality | Do not claim that the proposed foundation-plus-buried-depth heuristic is admissible with zero-cost Horne moves. Shared blockers and automatic banking require proof or a clearly described alternative before calling the result the shortest deliberate route. |
| Guest autoplay policy | Block 4 specifies `skip_horne_auto_moves=true` by default, with `auto_settle_delay_ms` after consecutive autos; `false` explicitly executes autos. Confirm the actual guest configuration and current result before updating physical heights or issuing further input. A delay or QMP acknowledgement alone does not prove that cards reached foundations. Handle the root auto prefix under the same policy. |
| Dynamic geometry | Cascade points follow actual physical column heights as cards move. Preserve source identity, current supported scene, STOP/context and fresh VM/pointer probes. Use the existing checked coordinate mapping and acknowledged typed-operation pipeline. An uncertain move cannot be replayed. |
| Complete outcome | Planned `cards_banked == 52` is an offline result. Require independent supported victory evidence before using the existing post-game handler, then require a fresh untouched deal before the next RECON. |
| Partial outcome | Log `cards_banked`, `moves_executed` and `nodes_explored`. A partial endpoint does not itself authorise reset/New Game; that needs the separately approved supported-scene handler. STOP/context revocation prohibits subsequent transitions, including cleanup or reset. |

The package paths named in the specifications describe ownership intent, not permission to duplicate the existing application execution, capture or QMP layers. Reuse the current xSAR neutral models/search and the application's guarded worker/input boundaries. No Free Cell model, codec, search, execution, version or guest change was made while preserving this plan.

## Code Analysis

The main threats are false card recognition, misordered physical locations, stale/cancelled results and uncertain file publication. A 52-card mask detects duplicates and omissions but cannot prove that 52 mislabelled cards are correct; independent glyph evidence and held-out frames are essential.

Use safe Rust, checked crops/arithmetic/conversions and a fixed stack capacity. Do not expose partially initialised deals, use unchecked indexing or interpret unknown pixels as empty. Bound profile/template work, document input and diagnostics. Keep recognition and capture on the worker so the GUI stays responsive.

Block 1 has no gameplay input capability. A malformed frame, unsupported scene, ambiguous card, invalid deck or revoked context produces a structured refusal. Publication errors preserve whether a destination may already exist, preventing automatic retries from overwriting evidence or duplicating an uncertain operation.
