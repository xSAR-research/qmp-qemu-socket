# Free Cell Block 1: implemented read-only RECON

This implementation follows [the Block 1 plan](free-cell-block-1-plan.md). Charlie selected application v3.0.0 for RECON/export and deferred A* to v3.1.0. Source qualification, version alignment and fresh guest acceptance are separate evidence steps; this document does not assert that every step has completed.

## Operator workflow

From the repository root, use `cargo run --release`. Select the independent `A* Route Solution` strategy, then Free Cell. In this game the strategy exposes read-only preparation; it grants no search or input authority. Selection does not automatically acquire or export a deal. Use `RECON / Export 52` explicitly.

An optional export path is used literally, including unusual names. Leaving it blank selects a process/request-specific path under `/home/charlie/tmp`. Existing destinations are never overwritten. A refusal retains the acquired pixels for inspection; a new explicit request or context change retires that inspection association.

The selected capture backend must be TCP v3 or v3-persistent. QMP capture and TCP v2 are refused before acquisition. The workflow uses the existing client once: it does not retry, switch transport, enable Solver, press D, click, undo or search. STOP during this workflow does not acquire another frame.

## Ownership and data contract

The application owns measured geometry, recognition, the serial worker request and exact context/request identity. The optional xSAR `freecell` feature owns checked card storage and initial-deal validation. `freecell_json` owns bounded strict intake, encoding and publication. Dependency-free xSAR defaults remain available.

The checked cascade stores at most twenty cards. The initial deal must contain eight physical columns with lengths `[7,7,7,7,6,6,6,6]`, four empty CELLs and four empty foundations. Every one of the 52 card identities must occur exactly once. Duplicate errors retain both original locations. Completeness never supplies an unreadable identity.

Schema 1 uses `game: "freecell"`, physical left-to-right cascades in top-to-bottom order, four explicit null CELLs, and required foundation keys `clubs`, `diamonds`, `hearts`, `spades`, each zero. The foundation model uses suit identity independently of screen decoration order. Root, card, foundation and provenance values must be objects; duplicate, unknown or missing fields, positional arrays for objects and trailing content are rejected. Documents are bounded to 65,536 bytes; reading consumes at most one extra byte to detect excess. The bounded input also limits temporary wire-column allocations before checked model construction.

Provenance has bounded, nonempty deal/profile/source labels. A label is descriptive evidence and does not grant input authority. Export serialises completely before filesystem access, then uses the existing no-overwrite publication mechanism: private temporary file, file synchronisation, hard-link publication, one cleanup attempt and parent-directory synchronisation. Publication errors retain NotPublished, Uncertain or Published outcomes. A failed link attempt must not be retried automatically. Unix permissions and trusted stable-parent/relative-working-directory preconditions are documented by the public writer.

The shared private filesystem implementation is also used by the existing Pyramid writer. Its public errors, display text and publication outcomes are retained; Free Cell has its own encoding errors. No server protocol or ordinary CV/Solver policy changes are required.

## Native recognition profile

`native-freecell-v1` accepts packed 1920×1080 RGBA at the measured native scale. Existing Free Cell horizontal anchors remain authoritative. The original card-top rows are `[331,384,437,490,543,597,650]`; the last two intervals deliberately preserve native rounding rather than imposing an integer stride.

The complete scene requires the measured Solver-off header, all eight positive empty top-slot regions, the expected 52 top borders, complete final bottom edges and both outer side rails of every column. Positive felt below each expected stack excludes additional cards. A readable corner alone cannot establish complete cascade geometry.

Rank and suit use separate bounded regions relative to each card origin: rank `(2,7,26,23)`, suit `(5,30,18,16)`. The bank contains the 44 exposed strips from the manually labelled FC13 original. Each candidate alignment re-crops actual RGB pixels at offsets −1, 0 and +1; no out-of-region ink is replaced with artificial padding. Paper, independent print colour, absolute quality and distinct-label separation must pass. Rank and suit colours must agree.

Current evidence-derived Hamming limits are rank error at most 48 of 598 samples and gap at least 20; suit error at most 16 of 288 and gap at least 12. Paper P90 must be at least 245; dominant print evidence is at least 70 samples, with at most 10 opposite-colour samples. These are specific to the retained native evidence. They are not general font, theme or scale support.

Failures retain typed category, physical column/depth or top slot, profile, region, bounds and first cause. Unknown identity, ambiguity, colour disagreement and positive empty-slot evidence are distinct. Cancellation is checked between bounded scene, calibration, card, rank and suit units as well as before and after capture and immediately before publication.

## Evidence and acceptance limits

FC01 and FC13 were each manually read at every physical card position without elimination. Both have a complete unique deck and empty top slots. Production reader tests compare every recognised location against those independent transcriptions. FC13 trains exposed-strip templates; FC01 also informed the measured limits, so neither result is described as blind threshold acceptance.

Later original frames provide additional native full-face typography tests for Jacks, Sevens, Six and Fours. Native Twos in foundation faces test standalone typography; they do not establish physical PLAY-bottom Two coverage. That position remains unverified. Authentic dealing-animation and unsupported scale/artwork negatives, a blind new whole board and fresh read-only guest acceptance also remain unverified. Synthetic erased/clipped/mixed/duplicate/extra-row cases establish implementation refusal behaviour only.

The tests cover checked model bounds and mutation failure, exact deck/location preservation, strict codec objects and size limits, publication collision outcomes, colour and geometry refusals, cancellation between recognition units, stale UI completions and inspection retention across STOP/disconnect ordering. Local real-TCP tests exercise production acquisition, recognition and strict export in one request, selected-backend refusal before capture, incomplete-payload STOP with no export, and retained same-frame diagnostics. They do not constitute live guest acceptance.

Until the missing native evidence and required read-only guest acceptance are recorded, Block 1 is source-qualified for its demonstrated profile only; the plan's full implementation-acceptance criteria remain open. A version-labelled build must not be presented as complete native coverage.

## Code Analysis

External boundaries are the selected capture server, untrusted frame layout and pixels, bounded JSON, and the operator-selected publication path. Checked dimensions/stride/storage and crop arithmetic precede pixel access. Model constructors reject invalid capacity, shape, foundation progress and duplicate cards. Strict bounded parsing prevents unbounded document intake; no custom allocator or new unsafe code is required.

The existing serial worker owns capture and publication. Exact context plus checked request identity prevents stale completion from releasing a newer owner. Same-frame inspection does not confer search or input authority. Genuine transport, recognition and publication errors retain their cause even if STOP arrives later. No uncertain publication or cancelled acquisition is replayed.

No A*, best-effort planning, automatic transfer model, move execution or restart is implemented in this block. Those remain later work; A* is assigned to v3.1.0.
