# v2.2.0: explicit card values and schema-2 deal export

Date: 11 October 2026, Australia/Brisbane. Approved Gate 2 implementation.

## Scope and source boundary

Application baseline is local HEAD `57215a90fd2836cb13a0d1b14400c5d5d417e9f9`
with the accepted uncommitted 2.1.0 reconnaissance implementation. xSAR baseline
is HEAD `987856a0b5cb95aec2788551e050e18f7469b1ef` plus the reviewed uncommitted
0.3.0 foundation/publisher. This slice changes package versions to application
2.2.0 and xSAR 0.4.0; both manifests and root lock entries move together.
No dependency versions, editions, MSRV, native calibration or CV policies change.

Charlie subsequently promoted the complete xSAR 0.4.0 implementation at
`ab226a65d3bbc2b4914a5cecfb2d3247364c7536`. The current application manifest
and lockfile pin that Git revision. Normal builds require no local source patch.

xSAR owns `Rank::value()` and `Rank::from_value(u8)`, strict versioned reading,
canonical writing and no-overwrite publication. The application retains capture,
recognition, authority, restoration and UI ownership. It consumes the shared
writer rather than duplicating a value table or JSON serializer. Suits remain
part of every identity for recognition, uniqueness and future game planning.

## Wire contract

New output uses root `schema_version: 2`. Each of the 28 tableau cards and
24 first-draw stock cards has exactly this shape:

```json
{ "card": "K of spades", "value": 13, "suit": "spades" }
```

Values are integers 1..13: Ace=1, 2..10 unchanged, Jack=11, Queen=12, King=13.
Numeric value and canonical suit identify the card; its readable label must
exactly agree. Internally `Card` remains a typed `Rank` plus `Suit`, without
independently mutable numeric or label copies. These rank numbers are neither
a general game score nor proof that a card is exposed or legally playable.

The reader explicitly accepts schema 1's exact rank/suit records and schema 2's
exact card/value/suit records. No mixed shapes, missing fields, nulls, unknown or
duplicate keys, inconsistent identities, float/string values or unsupported
versions are accepted. Root and card object-key order is not significant.
The writer always emits schema 2; older schema-1-only consumers must be upgraded.
Collection order, metadata, complete/restoration checks and standard-deck
uniqueness remain unchanged. The input/output cap remains 65,536 bytes (64 KiB),
with a one-byte overflow probe on stream input and fixed 28/24 arrays.

The native calibration file is a different contract: its schema remains 1,
profile ID remains `pyramid-native-1920-classic-blue-v4`, and SHA-256 remains
`796f3debfa098715fac781f4aa69de1dc4eaefb91af5ca996db7dc8e12474577`.

## Accepted live source and conversion

Charlie reported successful acquisition, worker-verified original restoration
and export by 2.1.0 to `/home/charlie/tmp/pyramid-recon-20261011-04.json`.
That original is 3253 bytes, SHA-256
`21a61672fa37840fc0b4c14dbe54aa437ecf6fa8d250dc8c8569985fe7c6d180`.
It records 31 capture attempts and two-frame restoration. Its 28 tableau and
24 stock identities form the standard deck and exactly match the preceding
preserved scan, including first LEFT King of diamonds and final Queen of diamonds.

The accepted original is never rewritten. A private lean integration harness
reads it through xSAR and publishes a separate schema-2 document, then re-reads
and compares the complete typed deal. Conversion preserves the original deal
ID and acquisition producer/source/restoration; it must not relabel acquisition
as a 2.2.0 scan. No capture, QMP connection, input, restoration or route execution
is performed by conversion. Its result is representation validation, not a fresh
live run. Future application exports report their actual compiled producer version.

## Validation and review

Evidence root: `/tmp/pyramid-schema2.4FXYLJL6/`. Source backups, first failures,
logs, exact file hashes and promotion path lists are retained there.

The separate result is `/home/charlie/tmp/pyramid-recon-20261011-04-schema2.json`,
4738 bytes, private mode 0600, SHA-256
`fc9a46bb1cc728fac80017a59d85a3a8947467c1cbc2a4b20107abcac6de8f52`.
Actual xSAR `read_from` / `write_new` / `read_from` verified whole-deal equality.
An independent raw JSON audit verified all 52 exact records, values, suits,
tableau/stock order, metadata and the unchanged original source checksum.
The original acquisition producer remains 2.1.0 intentionally. No rescan is
necessary to convert this accepted deal.

Validation used rustc 1.101.0-nightly (32dba69d6, 2026-10-09) and
Cargo 1.101.0-nightly (29c5daa1a, 2026-10-07), with native release target
`x86_64-unknown-linux-gnu`. The following earlier paired-development checks used
`--locked --offline` with an external source override.

| Check | Observed result | Evidence filename |
| --- | --- | --- |
| Actual xSAR all-feature check | Passed | `xsar-actual-check.log` |
| Actual xSAR full tests, including ignored | 147 tests plus 2 doctests passed; 0 failed, 0 ignored | `xsar-actual-all-tests.log` |
| xSAR default/cards/pyramid/pyramid_json/image_matching/png/qmp independent checks | All seven passed | `xsar-feature-matrix.log` |
| xSAR lean default/cards library tests | 14 / 16 passed | `xsar-default-tests.log`, `xsar-cards-tests.log` |
| Actual xSAR Clippy, strict private rustdoc, release | Passed | `xsar-actual-clippy.log`, `xsar-actual-doc.log`, `xsar-actual-release.log` |
| Actual application check | Passed | `app-check.log` |
| Application focused reconnaissance tests | 80 passed, 0 failed, 0 ignored | `app-schema2-recon-tests-corrected.log` |
| Application full tests, including ignored | 697 passed, 0 failed, 0 ignored | `app-full-tests.log` |
| Application Clippy, strict private rustdoc, native release | Passed | `app-clippy.log`, `app-doc.log`, `app-release.log` |
| Lean actual-source converter | Built, converted and round-tripped; no QMP/PNG/egui dependencies | `converter-actual-build.log`, `converter-lean-tree.log`, `converted-live-deal.log` |
| Existing-destination refusal | Failed safely; original destination bytes unchanged | `converter-collision.log`, `final-source-audit.log` |
| Guarded sibling delivery rehearsal | Apply, idempotence, unknown-content refusal and injected-failure rollback passed | `delivery-rehearsal.log` |

Independent subagents supplied model/codec/publication tests, application
integration tests and a read-only codec review. No blocking codec issue was
reported. A new test initially expected `NotPublished` for destination collision;
the established publisher deliberately returns `Uncertain`. Only the incorrect
test expectation was corrected. The failure remains in
`app-schema2-first-failure.log`; no production failure policy was weakened.

Pattern-based source/staged-diff/recent-history scans found no high-signal
credential candidates. Author names and intentional local provenance paths remain
human-review findings; binary image OCR and exhaustive history scanning were not
performed. Reports are `app-secrets-report.md` and `xsar-secrets-report.md`.

The earlier paired native release executable was
`target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket`, SHA-256
`79a93ee3bcdd50cf6f1159836dd717c83eef0d676d1162c832cdd4caf7e3a0aa`.
The subsequent Git-backed build is recorded below. Neither build establishes
the identity of an already-running process. The deliberate human procedure is
in the [live-test brief](pyramid-recon-live-test.md). No launch, installation,
guest input or fresh 2.2.0 live acceptance was performed.

The earlier filesystem permission boundary required a private staged crate;
reviewed files were delivered to the sibling checkout with guarded byte/mode and
Git-index checks. Earlier application validation used external Cargo patches:
`local-xsar.toml` selected the actual sibling and `local-xsar-staged.toml` selected
the development copy. These historical configs stay outside promotion files.

## Git-backed consumer validation

Evidence root: `/tmp/pyramid-git-pin.KWcKZAfa/`. The complete promoted crate
matches all 12 reviewed delivery hashes, and Cargo's own Git checkout matches
every tracked file in `ab226a65d3bbc2b4914a5cecfb2d3247364c7536`. The sibling
xSAR checkout is clean. Cargo metadata selects version 0.4.0 with the existing
qmp/png/image_matching/pyramid_json features and their cards/pyramid dependencies.

Plain `cargo check` fetched the promoted source and regenerated the affected
lock entry. The only lock change from the paired 0.4.0 state is xSAR's Git source;
all package versions and transitive dependencies are unchanged. The initial
package-specific `cargo update` could not identify the old path-only lock entry;
its failure is retained in `lock-update.log`. Normal resolution succeeded in
`check-and-lock.log`, followed by locked checks without any external config.

| Check | Observed result | Evidence filename |
| --- | --- | --- |
| Plain check and narrow lock resolution | Passed | `check-and-lock.log` |
| Locked check | Passed | `app-check-locked.log` |
| Full tests, including ignored | 697 passed; 0 failed, 0 ignored | `app-full-tests.log` |
| Clippy, all targets, warnings denied | Passed | `app-clippy.log` |
| Strict private rustdoc | Passed | `app-doc.log` |
| Native release build | Passed | `app-release.log`, `release.sha256` |
| Source identity and lock/dependency audit | Passed | `cargo-metadata.json`, `source-audit.log` |

The original accepted JSON, separate schema-2 conversion and native profile
checksums remain unchanged. The application HEAD/index are preserved, with
no staged changes. This source validation does not establish a new live scan.

The current Git-backed native release executable has SHA-256
`37314eeb509667231dbf48ec36324bc4829b91595778ba0cd029611c0f83953c` at
`target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket`. Use the new scripts in
the Git-backed evidence directory for any deliberate launch. Both were syntax-
checked; neither was executed. Earlier launch scripts pin the preceding build.

## Promotion order

1. Complete: Charlie promoted xSAR at
   `ab226a65d3bbc2b4914a5cecfb2d3247364c7536`. No registry publication is implied.
2. Complete: application manifest/lockfile select that exact Git source and
   locked consumer checks pass without a local source patch.
3. Obtain consumer acceptance, then Charlie promotes the application using
   explicit staging paths, including the new reconnaissance modules and fixtures.

No automatic commit/push or remote synchronisation occurs. Explicit path lists
exclude local AGENTS/HANDOFF files, workspace configuration, temporary overrides,
executables and private live evidence. The successful native original remains
outside promotion files. Console intake and route search are subsequent work;
numeric values alone do not implement Pyramid move legality or find a win.

The current `app-promotion-paths.txt` and `app-promotion.sha256` in the Git-backed
evidence root enumerate all 52 changed/untracked application source paths,
including earlier reconnaissance work. Review them before explicit staging.
Earlier crate/application inventories remain historical evidence in the paired
validation directory. No staging or commit command has been executed by the agent.
The syntax-checked `stage-application.zsh` in the current evidence directory
verifies the expected application HEAD, empty index and all reviewed file hashes,
then stages exactly that list when Charlie deliberately runs it. A dry run of
the same path list succeeded. Local AGENTS/HANDOFF/workspace files are excluded.

Recorded xSAR commit text for the foundation/schema-2 promotion:

```text
Add card models and schema-2 Pyramid deal JSON

Add optional typed card and complete Pyramid deal models with deck validation.
Provide bounded versioned JSON reading and atomic no-overwrite publication.
Write canonical card labels, values 1-13 and suits in schema 2 while retaining
strict schema-1 reading and preserving deal order and acquisition provenance.
Add model, malformed-input, compatibility and publication coverage.
Advance xSAR to 0.4.0 and retain dependency-free default features.
```

Suggested application commit text after consumer acceptance:

```text
Add guarded Pyramid reconnaissance with schema-2 deal export

Recognise the native tableau and ordered DRAW pile with bounded observations.
Guard D, Undo All and its verified confirmation; require original restoration
before publishing a complete validated deal through the shared xSAR codec.
Export readable card labels, values 1-13 and suits while preserving provenance.
Retain failure evidence, no-overwrite publication and STOP/context revocation.
Pin xSAR 0.4.0 at ab226a65d3bbc2b4914a5cecfb2d3247364c7536.
Advance to 2.2.0 with integration tests, native fixtures and acceptance records.
Preserve the five existing CV controllers and leave the Spider XP quirk deferred.
```

## Code Analysis

Untrusted inputs remain bounded JSON/profile bytes, capture data and filesystem
paths. The codec uses safe Rust and fixed arrays, retains strict duplicate and
unknown-field handling, and derives display/value fields from typed identities.
Optional private wire fields support two explicit layouts, not permissive missing
field defaults. Canonical consistency and complete-deck checks prevent contradictory
planning identities. Parser error displays retain bounded locations, not hostile
document strings; retained caller I/O causes can still carry arbitrary text.

Existing atomic no-overwrite publication and its uncertain/published failure
states are unchanged. Filesystem operations and caller-supplied readers can block;
the document cap is not an exact heap or elapsed-time bound. Restoration metadata
is a producer declaration, not authenticated evidence or input authority. Five-game
behaviour, STOP/context revocation and the deferred Spider XP quirk remain untouched.
