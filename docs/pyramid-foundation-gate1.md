# Outcome 2 — Pyramid foundation Gate 1

Date: 10 October 2026; baseline updated 11 October 2026, Australia/Brisbane.
Status: Charlie approved Foundation Gate 2 in this conversation. The model,
codec, feature, dependency, version and acceptance scope below is approved;
the RGB prerequisite is now promoted and foundation implementation is authorised.
Gate 2 implementation and validation are recorded in
[the foundation delivery report](pyramid-foundation-gate2.md); human review and
promotion are not implied by those automated results.

## 1. Baseline and scope

| Repository | Current HEAD | Local package |
| --- | --- | --- |
| qmp-qemu-socket | 8446fbb41bc3d99e4c959fb01ccb4762e4ad7901 | 2.0.11 promoted |
| xsar | 987856a0b5cb95aec2788551e050e18f7469b1ef | 0.2.1 promoted foundation base |
| basic-route-planning | 9f78868bc0afaf9d3fe3d2a53a7d14af96e6dee0 | 0.1.0 scaffold |

Use the existing sibling checkouts under `/home/charlie/repo/RUST/`. Charlie
promoted the RGB crate first, then the application, and explicitly authorised
foundation code. Preserve both indexes, unrelated workspace changes and any
user-managed local instruction deletions. No remote refresh, reset, branch
change or instruction-file replacement is required.

The application manifest and lock now consume the exact promoted xSAR Git
revision above with qmp/png/image_matching and defaults disabled. Do not replace
that release pin with a development path. A private temporary consumer override
may verify compatibility with the unpromoted foundation. The console-intake
implementation remains separately gated; no console changes belong here.

The next bounded implementation is pure card/Pyramid models and a versioned,
bounded JSON codec in xSAR. Proposed release: **xSAR 0.3.0**, retaining existing
APIs, default features, edition 2021 and MSRV 1.101.0. Update its manifest and
root lock entry together only after approval. No application or console version
change belongs to this foundation slice.

Implementation tools on 11 October: rustc 1.101.0-nightly (`32dba69d6`, 2026-10-09) and Cargo
1.101.0-nightly (`29c5daa1a`, 2026-10-07). Use explicit installed nightly for
xSAR validation; do not alter host defaults or add a toolchain file to the
console. Its eventual MSRV/toolchain policy belongs to console-intake Gate 1.

## 2. Ownership and exclusions

- xSAR owns card identities, complete original-deal validation and the JSON
  representation. These types contain no QMP client, image, calibration or GUI.
- The application retains recognition, partial observations, scene/context
  authority, input, restoration verification, preview and output-path choice.
- The console later owns CLI arguments and presentation, consuming xSAR rather
  than duplicating card/deal validation or JSON interpretation.
- Keep original deal arrangement separate from accumulated evidence and live
  cursor/board state. This slice implements only the complete original-deal
  value and bounded provenance, not a speculative acquisition state machine.
- Do not modify CV controllers, introduce card-effect gates, address Spider XP,
  implement card recognition, send D/Undo All/OK, add a solver, or automate input.

## 3. Modules, features and dependencies

| Proposed feature | Public modules | Dependencies |
| --- | --- | --- |
| `cards` | `games::cards` | None beyond std |
| `pyramid` | `games::cards`, `games::pyramid` | Enables `cards`; none beyond std |
| `pyramid_json` | The two model modules and `pyramid_json` | Enables `pyramid`, optional serde and existing optional serde_json |

The existing empty default, telemetry, geometry, qmp, image_matching and png
boundaries remain intact. Do not make model/JSON consumers enable QMP, PNG,
image_matching, image, egui or eframe. No `no_std` claim is proposed.

Propose optional direct `serde` 1.0.229, with default features disabled and only
`std` and `derive` enabled, and reuse the existing optional `serde_json` 1.0.151.
Those package versions and derive support already occur in the inspected crate
lockfile. Re-resolve only the intentional feature/dependency change and verify
the graph; their presence in the lockfile is not permission for broad upgrades.
No new filesystem, UUID, hashing, clock or search dependency is proposed.

Serde implementations belong to private wire types, not unchecked public deal
fields. Deriving deserialisation must not bypass the complete-deal constructor.
Use explicit fixed structs and unknown-field rejection rather than unbounded
`serde_json::Value` trees or arbitrary metadata maps. Serde provides the
necessary [container attributes](https://serde.rs/container-attrs.html).

## 4. Minimal public model contract

| Proposed API | Contract |
| --- | --- |
| `games::cards::Rank` | Exactly Ace, Two through Ten, Jack, Queen and King; no joker/unknown variant |
| `games::cards::Suit` | Exactly Clubs, Diamonds, Hearts and Spades |
| `games::cards::Card` | Rank and suit identity, with equality/hash support; no pixels or game rules |
| `games::pyramid::DealMetadata` | Checked deal ID, provenance, restoration declaration and bounded source information |
| `games::pyramid::CompleteDeal::try_new(tableau, stock, metadata)` | Accepts `[Card; 28]`, `[Card; 24]` and validated metadata; rejects duplicate identities or invalid metadata combinations |
| `CompleteDeal::tableau()` / `stock()` / `metadata()` | Immutable access; no mutable arrays or public fields that invalidate a completed value |
| `CompleteDeal::card_at(row, column)` | Checked zero-based row/column lookup; rejects row > 6 or column > row |
| `games::pyramid::DealError` | Typed identity/location/metadata failures; useful bounded diagnostics |

Tableau array index is `row * (row + 1) / 2 + column`, for rows 0..6 and
columns 0..row. Array order is top-to-bottom, left-to-right. Stock array indices
0..23 are first-draw order, including the initial visible LEFT card at ordinal
0. Array order is authoritative; do not duplicate row/column/ordinal fields in
each serialised card. All 52 standard identities must occur exactly once.

The app's existing 31-entry CV target array includes three controls and scans
bottom-to-top. Future acquisition must explicitly translate that representation;
it is not the exported tableau array. No such adapter is implemented here.

`CompleteDeal` cannot contain unknown cards or a partial diagnostic inventory.
Its validation proves structure and internal consistency, not that a screen
reader was correct or that a physical game was restored.

## 5. JSON schema version 1

The root is one object with the following fields. All listed fields are required
unless explicitly optional. Duplicate object keys and unknown fields are errors
at every level. No implicit defaults or automatic schema migration.

| Field | Exact proposed meaning |
| --- | --- |
| `schema_version` | Integer 1 only |
| `game` | String `pyramid` only |
| `deal_id` | Caller-supplied, non-empty ASCII token, maximum 64 bytes; characters A-Z, a-z, 0-9, hyphen and underscore |
| `complete` | Boolean true only; partial diagnostics use a different future format |
| `restoration` | String `verified` or `not_applicable`, subject to provenance rules below |
| `provenance` | Fixed object with `kind`, `producer`, `source`, and optional `evidence_id` |
| `tableau` | Exactly 28 card objects in canonical tableau order |
| `stock` | Exactly 24 card objects in first-draw order |

Each card is exactly `{"rank":"A","suit":"spades"}` in shape. Valid rank
strings are `A`, `2` through `10`, `J`, `Q`, `K`; valid suit strings are `clubs`,
`diamonds`, `hearts`, `spades`. Reject numeric ranks, alternate spellings, nulls
and extra card fields. Semantic equality does not depend on JSON object-key order.

Provenance kinds are `synthetic_fixture`, `manual_fixture`, `screen_recognition`.
`producer` is 1..64 UTF-8 bytes; `source` is 1..256 UTF-8 bytes. Both reject
control characters. Optional `evidence_id` is a 1..128-byte ASCII token with the
same character set as `deal_id`. It is an opaque reference, never an opened path.
Deal IDs are caller-assigned labels, not hashes or proof of global uniqueness.

For `screen_recognition`, require restoration `verified`. Both fixture kinds
require `not_applicable`: never label a synthetic fixture as a restored live
board. Console consumers must retain/display that distinction. Accepting a
restoration declaration from JSON does not independently verify its truth or
grant permission to execute a move.

Do not include optional capture dimensions, coordinates, capture hashes or
per-card confidence in schema 1. Retain native diagnostic evidence separately,
referenced by `evidence_id` when available. Such evidence is optional under the
handoff contract. Adding it to this strict wire format later requires an explicit
schema-version decision, not silently accepted unknown fields.

## 6. Codec API and resource bounds

Proposed `pyramid_json` public functions:

- `from_slice(bytes: &[u8]) -> Result<CompleteDeal, JsonError>`.
- `read_from(reader: impl std::io::Read) -> Result<CompleteDeal, JsonError>`.
- `to_vec(deal: &CompleteDeal) -> Result<Vec<u8>, JsonError>`.

The writer returns one fully validated UTF-8 JSON document, two-space indented,
with a trailing newline and stable field/card ordering. Reader/writer round
trips preserve semantic identity and metadata; arbitrary input whitespace is
not preserved. The reader refuses trailing non-whitespace or a second document.

Proposed limits and implementation requirements:

1. Maximum encoded document size is **65,536 bytes**, including whitespace and
   newline. Slice input is checked before parsing. Stream input reads at most
   65,537 bytes to distinguish oversize from exact-limit input; do not trust a
   file's metadata length or use an unrestricted `read_to_end`.
2. Parse directly into fixed-size 28/24 card arrays and fixed metadata fields.
   Do not allocate collections based on untrusted length hints. Reject wrong
   array lengths rather than truncate, pad or infer missing cards.
3. Check metadata lengths before copying them into the owned model. JSON string
   decoding may use scratch space up to the already enforced document cap;
   do not claim that a field limit alone prevents parser scratch allocation.
4. Keep serde_json's recursion protection enabled. Do not enable unbounded-depth
   or arbitrary-precision features. The fixed schema permits no free-form
   nested metadata; test deeply nested invalid values as refusal cases.
5. Serialise through a bounded output buffer and refuse output above 65,536
   bytes. Do not partially publish a destination from `to_vec`.
6. Use fixed storage for 52-identity uniqueness checks. Checked arithmetic and
   fixed array bounds cover slot/index operations. No unsafe code is proposed.
7. No wall-clock deadline is promised for a generic blocking `Read`. Callers
   control stream/file selection and cancellation. Input/output byte limits
   are not an I/O timeout, exact RSS cap or recovery guarantee after process OOM.

`JsonError` should distinguish I/O, oversized input/output, malformed JSON,
unsupported schema/game and invalid complete-deal data. Preserve useful location
or offending-slot context without echoing the entire document or evidence.
Multiple-invalid-field error precedence need not be standardised.

The official [serde_json deserialiser documentation](https://docs.rs/serde_json/1.0.151/serde_json/struct.Deserializer.html)
describes recursion protection and final-input checking; these do not replace
the explicit byte, collection and field bounds above.

## 7. Filesystem publication boundary

Foundation implements the bounded reader and JSON byte writer, **not a filesystem
publisher**. The first milestone can round-trip a committed known fixture without
creating or replacing a live acquired-deal file. Do not describe `to_vec` as an
atomic file-save operation.

For the later export mini-goal, the agreed policy is: validate/encode before
publication; use a private same-directory temporary file; publish the completed
file atomically; refuse an existing destination by default; leave partial/error
diagnostics distinctly labelled. An initial new-file-only API is preferred over
adding overwrite support before a real need. No direct `File::create` of the
final destination is acceptable.

The specialist xSAR JSON/file area should own that mechanism; application/CLI
code selects paths and presents errors. Its exact platform support, no-clobber
primitive, durability semantics, cleanup and published-but-sync-failed result
must be agreed in the export Gate 1. Do not assume a temporary-file helper alone
provides universal atomicity: for example, the documented caveats on
[persist_noclobber](https://docs.rs/tempfile/latest/tempfile/struct.NamedTempFile.html#method.persist_noclobber)
need consideration before choosing it. No filesystem dependency is approved now.

## 8. Gate 2 acceptance checks

1. **Known complete fixture:** labelled `synthetic_fixture`, restoration
   `not_applicable`; suits ordered clubs, diamonds, hearts, spades and ranks
   Ace through King within each suit. First 28 cards form the tableau; remaining
   24 form the stock. Assert every logical row and stock ordinal independently.
   This is codec evidence, never screen-recognition evidence.
2. **Models:** all 52 distinct identities; exactly 28+24 positions; duplicate
   identities within and across arrays rejected; checked row/column lookup;
   immutable completed invariants and valid metadata combinations.
3. **Codec:** fixture reader/writer/read round trip; stable writer output; useful
   invalid-rank/suit, duplicate-key, unknown-field, unsupported-version/game,
   missing/null/partial-status and trailing-document failures.
4. **Bounds:** exact and over-limit document sizes, excess/missing array entries,
   every string boundary, multibyte lengths, control characters, malformed UTF-8,
   truncated JSON, nested invalid values, short/erroring reads and output bounds.
   Refuse duplicate/missing cards rather than infer identities by elimination.
5. **Feature matrix:** default, no-default, cards, pyramid, pyramid_json, existing
   image/qmp/png combinations and all features. Confirm default/model-only trees
   have no external dependencies and JSON-only has no QMP/image/GUI dependencies.
6. **Quality:** locked check, tests, strict private rustdoc, all-target Clippy with
   warnings denied and release build. Preserve existing RGB and transport tests;
   run include-ignored host tests where applicable and report actual results.
   Scoped formatting is permitted; preserve the established source layout.
7. **Public consumer evidence:** an integration test must import only the public
   model/codec API. Separately compile a minimal external consumer with only
   `pyramid_json`. Neither is a substitute for actual console-intake acceptance.

The separate console-intake mini-goal will provide real repository-level consumer
evidence. Do not mark that milestone or end-to-end acquisition complete from the
foundation harness. Coordinate Gate 3 integration acceptance and crate-first
promotion with that actual consumer; no automated test authorises a live scan.

## 9. Expected implementation paths and next gate

Proposed xSAR paths: `Cargo.toml`, `Cargo.lock`, `src/lib.rs`, `src/games.rs`,
`src/games/cards.rs`, `src/games/pyramid.rs`, `src/pyramid_json.rs`,
`tests/pyramid_contract.rs`, `tests/fixtures/pyramid-synthetic-v1.json`,
`README.md`, `CHANGELOG.md`. Tests may remain beside implementation where that
matches existing style. No deletion or existing-public-API removal is proposed.

No application controller or console source changes are included. No card
templates, image reader, stock/redeal rules, search costs or reconnaissance
timing budgets are implicitly approved. Those need their own bounded Gate 1.

Charlie approved the 0.3.0 version, feature/API/schema, 65,536-byte cap,
fixture/restoration distinction, dependencies and stream/byte codec scope at
Gate 2. Routine implementation fixes and rechecks within that scope require no
repeated approval once the baseline prerequisite is satisfied or explicitly
amended. No staging, commit, push, publication or guest input is authorised.
