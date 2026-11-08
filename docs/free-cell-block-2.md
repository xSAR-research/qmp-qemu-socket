# Block 2 — Free Cell complete offline routes (3.1.0)

This is the approved implemented contract. It supersedes the earlier shortest-route proposal. xSAR owns `games::freecell::{rules,search,replay}` behind `freecell_rules` and `freecell_search`; the application owns offline file selection, job identity, worker control and presentation.

## State and legal moves

The immutable initial deal retains all 52 card identities and physical top-to-bottom column order. Search positions retain eight physical cascades, four cells and fixed Clubs/Diamonds/Hearts/Spades foundation counts. Move records retain actual physical source, destination, card and automatic/deliberate tag.

Only exposed single cards move: PLAY to PLAY, PLAY to empty CELL, CELL to PLAY (including empty PLAY), or exposed PLAY/CELL to its own next-rank foundation. Cascade placement is descending alternating colour; any card may occupy an empty cascade. Foundations are irreversible. There is no multi-card primitive or CELL permutation move. Empty destinations use the first physical empty slot; lone-column relabelling is omitted.

## Exact symmetry and automatic transfers

Equality compares sorted complete ordered column sequences including lengths, sorted cells including empty slots, and fixed suit foundation counts. Incrementally cached column hashes accelerate hashing; a hash collision never establishes equality. Only touched column hashes are refreshed. Immutable search-arena parents preserve physical route coordinates across canonical deduplication and reopening.

Horne autoplay runs at the root and between deliberate moves. A card must be the next rank of its suit, with rank at most two OR both opposite-colour foundations at least rank minus one. Deterministic scan order is PLAY 0..7 then CELL 0..3. Every automatic transfer is recorded separately at zero deliberate cost; deliberate transfers cost one. Unsafe-but-legal explicit foundation moves remain available.

## Search, replay and control

The guided A* ordering uses deliberate cost plus remaining-card/burial guidance. The guidance is not an admissibility proof; only a complete legal route is promised. There are no configured resource cutoffs. Checked arithmetic, fallible allocation, cancellation and actual OS resources still apply. Existing bounded exact and best-effort APIs retain their policies.

Independent replay starts from cloned physical `BoardState`/`Card` data and does not call the packed rules mutation or canonical equality. It checks exposure, every transfer, auto order/tags, capacity, rank/colour/suit and all 52 identities after every move, ending with four complete foundations and empty cells/cascades. Only replay-verified `Solved` exposes a route; exhausted, cancelled and failed outcomes expose no prefix. Replay honours the same pause/cancel checkpoint. Search-finished progress precedes replay and is advisory.

The dedicated serial worker reads the bounded strict document once, freezes the deal, coalesces measured progress and retires its exact context/request/source generation before one completion. PAUSE preserves the frontier; CONTINUE resumes it; CANCEL wakes a paused job. The UI uses authoritative final statistics and virtualises the route, streaming its log without a duplicate line vector. Calculation alone grants no guest-input authority. App 3.1.2 provides separate explicit-target guarded playback of the retained complete route; see [Block 4](free-cell-block-4.md).

See [3.1.0 verification](Verification-3.1.0.md) for independent physical-reference tests, the exported-deal result and all qualification receipts. Read-only native RECON acceptance remains a separate evidence boundary.
