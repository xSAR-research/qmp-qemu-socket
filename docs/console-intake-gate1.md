# Outcome 2 — console intake Gate 1

Date: 11 October 2026, Australia/Brisbane.
Status: Charlie authorised this Gate 1 review. Gate 2 implementation is not
approved by that request. No console source, dependency or version was changed.

## 1. Baseline and promotion boundary

| Repository | Observed HEAD | Package and local state |
| --- | --- | --- |
| qmp-qemu-socket | 8446fbb41bc3d99e4c959fb01ccb4762e4ad7901 | 2.0.11; documentation and ignore maintenance only |
| xsar | 987856a0b5cb95aec2788551e050e18f7469b1ef | 0.2.1 promoted; reviewed 0.3.0 foundation and ignore rules uncommitted |
| basic-route-planning | 9f78868bc0afaf9d3fe3d2a53a7d14af96e6dee0 | 0.1.0 scaffold; untracked local workspace file |

Use the existing sibling checkouts. The console has only Cargo.toml, Cargo.lock,
src/main.rs, .gitignore and LICENSE tracked. Main prints Hello, world!. There is
no dependency, README, test suite, Cargo configuration, repository instruction
file or toolchain/formatter file. Preserve basic-route-planning.code-workspace;
do not stage it. No remote refresh or branch change is required.

The foundation implementation and its actual-source validation are recorded in
[the Gate 2 report](pyramid-foundation-gate2.md). The temporary public consumer
and application compatibility checks do not constitute acceptance of this real
console milestone. Retain the real-consumer gate before foundation promotion:

1. Implement and validate the console against the reviewed local foundation in
   an isolated temporary paired workspace; no development host path is promoted.
2. Charlie verifies the console's complete known-fixture output.
3. Charlie promotes xSAR first and supplies the exact full commit revision.
4. Pin the actual console manifest to that Git revision with defaults disabled
   and only pyramid_json enabled; regenerate and revalidate its lockfile without
   unrelated upgrades. Obtain final consumer acceptance before its promotion.

The current xSAR HEAD does not contain the new APIs. Do not pin the console to
that older revision and claim it consumes 0.3.0. Do not commit a fictitious SHA,
unpublished registry version or temporary path. The existing QMP application
retains its accepted xSAR 0.2.1 Git pin; this slice does not require updating it.
Application documentation/ignore maintenance can be promoted separately now.

## 2. Proposed version, ownership and dependencies

Propose basic-route-planning **0.2.0**, retaining edition **2024**, and declare
Rust **1.101.0**, matching the existing xSAR manifest requirement. Update the
console manifest and root lock entry together at Gate 2. No additional xSAR or
QMP application version change belongs to this mini-goal.

Use the already installed nightly explicitly for validation, targeting the
native x86_64-unknown-linux-gnu host. Observed foundation tools were rustc
1.101.0-nightly (32dba69d6, 2026-10-09) and Cargo 1.101.0-nightly (29c5daa1a,
2026-10-07); record the actual versions again when implementation is validated.
Do not add a floating toolchain file or alter global defaults. A nightly pass
does not establish testing on stable Rust 1.101.0.

The console owns argument handling, read-only file selection, presentation and
exit status. xSAR owns all JSON and complete-deal validation. Its existing
public API is sufficient:

- pyramid_json::read_from and SCHEMA_VERSION;
- games::pyramid::CompleteDeal::{card_at, stock, metadata};
- DealMetadata accessors, ProvenanceKind and Restoration;
- games::cards::Card's ASCII Display implementation.

The sole direct dependency is xSAR, default-features=false,
features=["pyramid_json"]. No direct serde, argument-parser, image, PNG, QMP,
GUI, async or platform library is required. This is a single small executable,
not a new public console library or a duplicate JSON parser.

## 3. Proposed CLI and output contract

Invocation: `basic-route-planning <deal.json>`.

1. Accept exactly one non-empty literal file-path operand using args_os. Keep
   the original OsString/PathBuf for filesystem operations; UTF-8 is not required.
   Check for excess arguments without collecting the whole iterator.
2. There are no options or implicit input sources in this first slice. A sole
   `-`, `--help` or other leading-hyphen operand is a literal filename, not stdin
   or a flag. Missing, empty or extra operands print the short usage diagnostic.
3. Accept regular files, including symlinks resolving to regular files. Check
   metadata before open to reject ordinary directory/FIFO/device mistakes, and
   check the opened handle too. This is not a race-proof filesystem boundary.
4. Open read-only and call the bounded xSAR stream reader. Fully validate before
   writing a report to stdout. Do not use an unbounded read_to_string or trust
   a metadata length as the input bound.
5. Print schema/game, deal ID, complete status, provenance, restoration,
   producer/source and optional evidence ID. Print seven rows labelled 0..6,
   top-to-bottom and left-to-right, followed by DRAW ordinals 0..23. Ordinal 0
   is the initially visible LEFT card before the first stock advance.
6. Use Card Display identities such as `Q of hearts`, with explicit separators
   between tableau cards. Display each stock ordinal on its own line. ASCII
   output avoids dependence on suit-glyph/font support.
7. Label synthetic input as a synthetic fixture, not screen-recognition
   evidence; label manual input as a manually recorded fixture. Screen input is
   producer-declared recognition. A verified restoration value is a producer
   declaration, not independent proof by this reader. Fixture restoration is
   not applicable, never claimed as a successful live Undo All operation.
8. Escape metadata to printable ASCII, including Unicode directional controls.
   Diagnostic path previews must be escaped and capped at 256 displayed bytes,
   with a truncation indicator; never alter the actual path used for input.

Exit status is 0 only after successful validation, complete output and explicit
flush; 2 for argument-contract errors; 1 for file, validation or output failure.
Use fallible Write operations. BrokenPipe exits 1 quietly; other failures receive
short stderr diagnostics. Failure to write stderr must not panic or replace the
original unsuccessful exit. A partial report is possible after an output-device
failure; it must never receive a successful exit status.

Use xSAR's bounded Display diagnostics. In particular, do not print arbitrary
error chains, raw JSON or unbounded I/O error strings. File/output errors can
report a stable error category and optional OS code. Unknown/duplicate fields
currently produce a malformed-schema line/column diagnostic; adding detailed
field-name errors to xSAR is outside this scope.

## 4. Bounds and excluded authority

The codec caps documents at 65,536 bytes, consuming at most 65,537 bytes to detect
excess. The output contains exactly 52 card identities and bounded metadata;
stream it without collecting an unbounded report. No input-dependent card lists,
search frontier or screenshot data is allocated by the console. These are not
exact process-memory limits or blocking-I/O deadlines. Metadata/open/read/write
can block; the regular-file checks do not remove races or network-filesystem
latency. No cancellation or asynchronous I/O subsystem is proposed here.

Do not add search, move rules, live QMP input, capture, card recognition,
reconnaissance, Undo All, filesystem JSON publication or application adapters.
Preserve all five controllers and defer Spider XP. The existing ShortestPath
preparation remains read-only. Unknown/partial data is rejected, not repaired or
filled by elimination.

## 5. Planned files and acceptance

Planned console paths at Gate 2:

```text
.gitignore
Cargo.toml
Cargo.lock
src/main.rs
README.md
fixtures/pyramid-synthetic-v1.json
fixtures/pyramid-synthetic-v1.txt
```

The ignore change will preserve /target and exclude local agent/handoff notes
and *.code-workspace. Do not install or replace instruction files. The JSON is
the labelled upstream fixture, retained with attribution and original checksum:
2ae7f44d0aa41e7a381ac7dd9b88f4895c8e0770bf63b55713bd17ca28d653b3.
Review the expected text independently rather than generating the only oracle
from the implementation under test.

There is no existing console test suite. Do not introduce a new in-repository
test framework or test modules in this slice. Use an external temporary
acceptance harness against the real executable and relevant helper behaviour,
retain its code/results as evidence, and retain the known input/output fixtures
in the repository. Run cargo test and report its actual counts honestly; empty
scaffold tests are not evidence that executable acceptance was exercised.

Required acceptance:

1. Exact output comparison covers all seven rows, all 28 tableau identities and
   all 24 stock ordinals, not just counts. This fixture starts DRAW with 3 of
   hearts and ends with K of spades. It must clearly identify its synthetic origin.
2. Verify manual/recognition provenance labels, compatible restoration values,
   absent evidence ID, metadata escaping and bounded diagnostic presentation.
3. Verify absent/extra/empty operands, relative/absolute/spaced paths,
   leading-hyphen filenames and Unix non-UTF-8 paths.
4. Verify missing/unreadable files, directory and controlled special-file
   refusal, with non-zero status and no report on stdout before validation.
5. Exercise representative malformed JSON, unknown/duplicate fields,
   unsupported schema/game, invalid UTF-8, duplicate identity, missing/excess
   cards, partial status, incompatible restoration and trailing data. Padding a
   valid fixture to the exact byte cap must pass; one byte more must refuse.
6. Exercise short/interrupted writes, write-zero, broken-pipe and flush failure
   using deterministic external checks; diagnostics must not panic.
7. Run locked check, cargo test, strict private rustdoc, all-target Clippy with
   warnings denied, and release build. Record compiler/Cargo and source selection.
   Scope any authorised formatting check to changed files and preserve layout.
8. Verify Cargo metadata/tree enables only cards, pyramid and pyramid_json for
   xSAR and no image/PNG/QMP/GUI dependencies. Recheck after the final Git pin.
9. Charlie runs the verified actual console executable on the retained fixture
   and accepts the ordering, labels and error behaviour before promotion.

## 6. Approval request and next stages

Gate 2 approval would authorise the version/dependency, seven-file scope, literal
path-only CLI, regular-file policy, exit/error contract and acceptance checks
above. It does not authorise a commit/push, global toolchain changes or live input.
Sibling-checkout writes still use the normal scoped filesystem approval where
required by the execution environment.

After console intake is accepted, the next separately gated stages are native
rank-and-suit recognition, guarded stock reconnaissance with positive restoration,
then atomic no-clobber publication and acquired-deal console acceptance. This
known-fixture reader alone is not completion of live board/DRAW acquisition.

## Code Analysis

This proposal adds no runtime code. The planned trust boundaries are CLI path
input, filesystem bytes, producer metadata and terminal/output I/O. Existing
xSAR checked types remain the sole source of valid complete-deal input. No
unsafe code or live input authority is proposed. Resource bounds cover codec
bytes, fixed card counts and metadata, not filesystem latency or exact RSS.
Failure before validation emits no deal report; output failure may leave partial
text and must remain unsuccessful. Restoration metadata remains a declaration,
not authenticated evidence of the guest's actual state.
