# Pyramid foundation — Gate 2 implementation and evidence

Date: 11 October 2026, Australia/Brisbane.
Scope: the approved model/JSON foundation, not live reconnaissance or export.

## Delivered state

The RGB prerequisite is promoted: application 2.0.11 at
`8446fbb41bc3d99e4c959fb01ccb4762e4ad7901`, consuming xSAR 0.2.1 at
`987856a0b5cb95aec2788551e050e18f7469b1ef`. The actual xSAR checkout now contains
the reviewed **0.3.0** foundation as uncommitted source. Both manifest and root
lock entry were updated together. Edition 2021 and MSRV 1.101.0 are unchanged.
The only lock changes are the crate version, its optional direct serde entry
and activation of already-locked serde_derive. No package was upgraded or added
to the resolved package set.

Implementation adds three independent optional features:

1. `cards`: closed standard rank/suit/card identities and canonical names.
2. `pyramid`: immutable complete original deals, fixed 28/24 arrays, all-52
   uniqueness, checked row/column lookup and bounded provenance metadata.
3. `pyramid_json`: schema-1 slice/stream readers and a bounded pretty byte writer.

Default and model-only builds remain dependency-free. JSON-only needs serde and
serde_json, not QMP, PNG, image or GUI dependencies. Existing telemetry, geometry,
image comparison and QMP APIs are preserved.

The codec requires object-shaped root/provenance/cards; ordinary positional
Serde struct arrays are explicitly refused. Unknown and duplicate fields,
invalid rank/suit strings, nulls, missing/excess cards, duplicate identities,
partial status, unsupported schema/game and trailing documents are rejected.
Optional evidence may be absent but cannot be null. Fixed wire types must pass
the public checked model constructors before becoming completed input.

The document cap is 65,536 bytes, including whitespace/newline. Streams consume
at most 65,537 bytes to detect oversize. Metadata has separate byte/character
bounds; parser scratch can grow up to document-driven bounds before those field
checks. Recursion protection remains enabled. The writer uses a capped buffer,
stable field/card order, two-space indentation and a trailing newline.

`to_vec` returns bytes; it is not an atomic file writer. There is no screenshot
reader, stock scan, Undo All/OK automation, restoration verification, filesystem
publisher or solver in this slice. Fixture declarations cannot claim verified
live restoration. A screen-recognition restoration declaration is validated for
consistency but not independently authenticated.

## Fixture and consumer evidence

The actual crate contains `tests/fixtures/pyramid-synthetic-v1.json`:

```text
SHA-256 2ae7f44d0aa41e7a381ac7dd9b88f4895c8e0770bf63b55713bd17ca28d653b3
```

It explicitly declares `synthetic_fixture` and `not_applicable` restoration.
The ordering is clubs, diamonds, hearts, spades, each Ace through King; first
28 cards are the tableau and remaining 24 are the first-draw stock order.
Independent tests enumerate every expected row and stock ordinal and require
the writer to reproduce the independently authored canonical fixture bytes.
This file proves interchange behaviour, not a recognised deal from the guest.

An independent external Cargo consumer builds and runs against the actual local
xSAR checkout with only `pyramid_json`. Metadata confirms exactly cards/pyramid/
pyramid_json enabled and no image/PNG/QMP/GUI features. It loads the fixture,
validates 28+24, verifies fixture provenance and writer/stream-reader round trip.
This is a public-API harness, not the `basic-route-planning` console milestone.

## Validation

Compiler: rustc 1.101.0-nightly (`32dba69d6`, 2026-10-09), LLVM 23.1.3.
Cargo: 1.101.0-nightly (`29c5daa1a`, 2026-10-07). The installed nightly was
explicitly selected; no global default or toolchain file changed.

The full matrix passed on the reviewed tree and again on the actual xSAR
checkout. Every matrix/quality command used `--locked --offline`. All test
commands included `--include-ignored`; no test was ignored or failed.

| Feature selection | Library tests | JSON contract tests | Model contract tests | Doc tests |
| --- | ---: | ---: | ---: | ---: |
| Default | 14 | 0 | 0 | 0 |
| No default | 14 | 0 | 0 | 0 |
| cards | 15 | 0 | 0 | 0 |
| pyramid | 15 | 0 | 24 | 0 |
| pyramid_json | 16 | 35 | 24 | 1 |
| image_matching | 29 | 0 | 0 | 1 |
| qmp | 23 | 0 | 0 | 0 |
| png | 31 | 0 | 0 | 1 |
| qmp + image_matching | 38 | 0 | 0 | 1 |
| pyramid_json + image_matching | 31 | 35 | 24 | 2 |
| All features | 42 | 35 | 24 | 2 |

All-feature check, strict private rustdoc (`RUSTDOCFLAGS=-D warnings`),
all-target Clippy (`-D warnings`) and release build passed. Dependency-tree
checks prove default/cards/pyramid remain external-dependency-free. Scoped
formatting checks passed for all six new Rust implementation/test files, with
edition 2021 and the existing two-blank-line convention. Existing unrelated
source was not reformatted. Tracked changes passed `git diff --check`.

An isolated copy of the promoted application consumed reviewed xSAR 0.3.0
through a temporary override with unchanged qmp/png/image_matching features.
Metadata verifies that source selection; no unrelated application lock entry
changed. Its check, **612 tests including all host tests**, strict private docs,
all-target Clippy and release build passed. The actual application manifest,
lockfile, controllers and running process were not changed to use this build.
No live guest action or new five-game live acceptance is claimed.

Three parent-owned sub-agents used inherited model/reasoning settings for model
tests, JSON contract tests/fixture, and independent codec safety review. The
review found no actionable defect; its I/O-cause presentation qualification is
documented. The parent integrated and independently ran the full checks above.
The 24 model tests include 2,652 duplicate replacements with exact locations;
the 35 codec tests include finite short/interrupted/failing/non-EOF readers,
all strict object levels, exact byte boundaries and bounded error displays.

## Retained evidence and delivery

Private evidence: `/tmp/xsar-pyramid-foundation.KMdM3nLI`.

- `before/`, `baseline-files.json`: original bytes and modes.
- `reviewed/`, `applied-files.json`: reviewed source and delivered hashes/modes.
- `validation-reviewed/`, `validation-actual/`: full command logs/results.
- `compatibility-validation/`: existing application metadata and checks.
- `consumer-actual-metadata.json`, `consumer-actual-run.log`: lean actual-source
  public consumer evidence.
- `secrets-scan.md`: working tree, empty staged index and five-commit pattern
  scan. No credential-pattern hits; expected identity references are listed.

The first JSON-only check encountered the sandbox's read-only Cargo cache when
unpacking already-cached serde_derive. Explicitly approved cache access resolved
it without changing package versions or validation flags. Initial application
metadata warned that the temporary patch was unused; a targeted offline xSAR
lock update selected it, then metadata asserted the intended source before any
compatibility result was accepted. Both first-attempt records remain available.

A guarded preview checked the recognised crate HEAD, all original tracked
hashes/modes and every new-file collision before applying the 12 reviewed paths.
Delivered files match their reviewed hashes/modes. Both Git indexes and the
application workspace file retain recorded hashes. There were no agent-performed
deletions, staging, commits, pushes, registry publication or installed binaries.
Files under `/tmp` are temporary; source and fixture live in the actual crate.

## Review and promotion boundary

Gate 2 foundation implementation is complete. Human review and crate promotion
remain separate; coordinate the next console-intake mini-goal for real repository
consumer acceptance. Do not mark console intake or live acquired-deal export
complete from the temporary harness. Later screen recognition/reconnaissance
and atomic export retain their own gates and native evidence requirements.

Proposed xSAR commit message:

```text
Add checked Pyramid deal models and bounded JSON interchange

Advance xSAR to 0.3.0 with independent cards, pyramid and pyramid_json
features. Validate complete original 28-card tableau and 24-card stock
arrangements, standard-deck uniqueness and bounded provenance.

Add strict schema-1 JSON readers and a deterministic byte writer with
a 65,536-byte document cap. Reject malformed, ambiguous and incomplete
input without granting recognition, restoration or input authority.

Add a labelled synthetic fixture and public model/codec regressions.
Preserve dependency-free defaults, existing consumers and feature boundaries.
```

Complete crate staging list, relative to the xSAR repository:

```text
.gitignore
Cargo.toml
Cargo.lock
src/lib.rs
src/games.rs
src/games/cards.rs
src/games/pyramid.rs
src/pyramid_json.rs
tests/pyramid_models.rs
tests/pyramid_contract.rs
tests/fixtures/pyramid-synthetic-v1.json
README.md
CHANGELOG.md
```

The twelve foundation paths remain unchanged from validation. On 11 October,
Charlie's promotion-readiness request also adds `.gitignore` exclusions for
local agent/handoff notes; the complete promotion list therefore has thirteen
paths. Existing workspace exclusions and all prior ignore rules are preserved.

The application has separate documentation/maintenance changes only:
`.gitignore`, `docs/pyramid-foundation-gate1.md`, this report, and
[the console-intake Gate 1 proposal](console-intake-gate1.md). The ignore rules
cover AGENTS.md, root AGENTS-original.md, root HANDOFF-xsar-next.md and
`*.code-workspace`. At the observed local check, AGENTS-original.md,
HANDOFF-xsar-next.md and qmp-qemu-socket.code-workspace were still present and
tracked. Ignore rules do not remove tracked files or rewrite history. Their
removal/untracking was not performed by this task; Charlie must explicitly
stage removal from tracking while retaining local copies if accepting that
maintenance change. Keep these changes separate from the crate's explicit
staging list and preserve any later user deletions.

## Code Analysis

Untrusted boundaries are JSON bytes, caller readers and metadata declarations.
Closed identities, private immutable model fields, fixed arrays and checked
constructors prevent a partial/duplicate deal from becoming valid input. No
unsafe code or new live input/filesystem publication authority is introduced.

Slice input is capped before parsing; stream consumption is capped at one byte
beyond the document limit. Identity validation uses fixed 52-slot storage.
Parser/metadata allocations remain document-driven and output payload is capped;
these are not exact process-memory or blocking-I/O time limits. Codec-generated
errors avoid echoing hostile strings, while caller-supplied I/O causes remain
available and may expose sensitive/large text through Debug or error-chain output.

Unknown/ambiguous cards must still stop future acquisition; metadata cannot prove
physical restoration. STOP, current-context authority and no-uncertain-replay
rules remain entirely with the unchanged application. Spider XP is deferred.
