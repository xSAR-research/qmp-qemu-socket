# Verification - qmp-qemu-socket 2.9.164

Charlie approved build/delivery in m00996 and confirmed the contract and
independent reviews in m01006. This is host qualification and retained-image
reproduction. Fresh live guest acceptance is pending.

## Delivered behaviour

| Positively recognised fresh RECON fact | Route advice |
| --- | --- |
| Score challenge | Maximum card matches (`Fast`) |
| Card challenge, including non-King cards | Maximum card matches (`Fast`) |
| Board challenge | Shortest route (`FewestActions`) |
| Normal game | No override; preserve selected objective |
| Unknown/conflicting facts | No override; no new input authority |

One optional policy belongs to `ObservedGame` and is used by both locked-receipt
and fresh restored-frame paths. Advice remains authenticated to the acquisition,
current context and export. Fresh RECON applies recognised challenge advice even
over a prior selection; the user can select again afterwards. Reused acquisition
respects that later explicit choice. The NoChallenge stop setting does not
replace the positively observed challenge kind.

The native session retains the selected method across normal board generations.
It distinguishes the effective method from the preference: automatic Maximum
fallback stays board-local, while the existing accepted operator-switch
handshake updates the retained preference. Startup defaults are unchanged.
No unconditional planner override undoes later idle selection.

## Exact Coral reproduction

The retained 163 Request 2 first MATCH is 5S at original 7|1 plus 8S at 7|3.
Expected original slots 21 and 23 are empty; every other identity, including
5H at 7|6, remains; DL is 8C and DR is empty. Before the addition, the actual
full-position regression failed `execution slot 21 lacks positive absence`.
It now passes with immutable original frame bytes.

The final crop differs from the old 159 appearance at exactly ten blue channels,
each minus one. Its blue sum is 490994, below the unchanged structural minimum
491004. A separate exact proof compares all 14,688 RGB bytes at fixed native
clearance (286,588,144,34), after strong-white presence and before generic aged
refusal. The existing structural Coral functions, references, sums, absolute/edge
bounds and material-role vetoes are unchanged, as is exact 7|6 rock. This admits
one additional complete measured pattern without lowering the sum minimum.

Original final PNG SHA-256: `176c24ba1aeec1bb0ca2e5e9492cb3c50f57079ca56508bbee978143401b8377`.
Exact RGB crop SHA-256: `11058b3d28f0635ce6c546e56ac23f35abdc7358e13c7889df1078d95cacee1e`.
Deal SHA-256: `7c5ede4e85d16492b0642e50e080817507656e905e9719cffee151ce71c18512`.
Fixture [provenance](../tests/fixtures/pyramid-recognition-164/README.md) retains
the original PNGs/deal and exact crop. Eleven new image regressions cover the
complete position, every RGB boundary, all ten individual restoration negatives,
material, actual surviving card, shifted/other-slot artwork, remaining identities,
piles, required scene evidence and malformed layout. Six new objective tests and
corrected superseded defaults cover the route-policy integration.

## Actual host qualification

Initial 163 handoff returned HANDOFF VERIFIED; its baseline was PASSED (Cached).
The changed source then passed the following new gates using nightly, owning
package `qmp-qemu-socket`, native `x86_64-unknown-linux-gnu`, locked/offline
dependencies and all features. No sibling source was edited.

| Gate | Result | Seconds |
| --- | --- | --- |
| scoped-format | PASS | 0.76 |
| check | PASS | 3.98 |
| rustdoc | PASS | 2.59 |
| clippy | PASS | 4.56 |
| full-tests | PASS | 631.62 |
| release | PASS | 17.29 |

The full suite uses `--include-ignored --test-threads=8`: 1,633 passed; zero
failed, ignored, measured or filtered. Check/Clippy include all targets; Clippy
uses `-D warnings`; rustdoc includes private items with
`RUSTDOCFLAGS='-D warnings -D missing_docs'`. Formatting checks only the twenty
owned Rust files and retains repository spacing. The 876 input seals include
all 52 unchanged xSAR inputs. xSAR's 296 tests and three Miri remain
PASSED (Cached), not newly run evidence.

Early focused compilation exposed a missing test import and missing idle
argument; the next focused objective run found a nonexistent fixture path.
Review found three superseded app expectations and an unconsumed test Result.
All were corrected. Final focused results: Coral 11/11; objective 5/5; board
generation 5/5; challenge-default 3/3. The first qualification attempt stopped at
a format-helper range conversion error; correcting its 0-based to 1-based range
mapping changed no compiler input. First failures and diagnostics are retained.

The first full-suite run then completed with 1,632 passes and one failed
assertion: the retained restored frame literally shows Boards cleared 0/1,
while the test incorrectly expected a normal-game None advisory. The test now
asserts the literal Boards 0/1 observation and Some(FewestActions); modal,
input, capture and publication checks remain. The corrected ignored test passes.
An intermediate test compile error used `?` on an Option constructor; the
literal fixture now uses an explicit expectation. Production code is unchanged
by these corrections. The complete first full-suite log and source remain in
`first-full-suite-attempt`; final host gates above cover the resealed revision.

After all six source gates passed, the already approved version changed only
own Cargo.toml/Cargo.lock from 2.9.163 to 2.9.164. Aligned locked check, native
release and protected `--help` passed. The other 874 inputs are byte-identical.
Cargo JSON selected exactly one owning native binary. The compiled release label
contains v2.9.164; help itself does not print a version.

Protected executable: `/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/Pyramid/Validation/pyramid-coral-objective-164-20261029-fvp9nijq/native/qmp-qemu-socket-2.9.164`.
SHA-256: `5bd44ed9e700746762e54220b717691b66b1676721d66da50c52ef9130a1b1f0`. Mode 0700. It is not installed.
Evidence root: `/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/Pyramid/Validation/pyramid-coral-objective-164-20261029-fvp9nijq`.
Primary receipts: qualification.json, qualification-before-version.json,
qualification-inputs.json, aligned-inputs.json and destination-state.json.

HEAD `dcb92241aac491d917431b2ddcb851284f71898a`; parent `58aa2882fb0c4ff3aa0d71ae49de0871545b86e3`. The index remains empty. All four checkout
histories, sibling states and unrelated changes match the preserved baseline.
Independent image review found no issues. Objective production review found no
functional/safety issues; identified test corrections passed focused and full
checks. No GUI launch, guest input, new capture, install, staging, commit or push
was performed. The unchanged 163 protected executable is also sealed.

## Fresh live acceptance

From a fresh supported run, verify Coral removal and subsequent guarded effects;
Score/Card and Board RECON selections; normal multi-board preference persistence;
later idle selection; temporary fallback versus accepted operator replacement;
and the declared completion/continuation path. Preserve current ~/tmp/images
before startup and leave active logs at their original paths. The failed Request
2 route is not resumable. Retained artwork consistent with removal does not
prove guest score credit. Rendering cause and future appearance coverage remain
unproven. The standard Continuous acceptance retraction remains in force.

King/projection/scoring algorithms, Single-board withholding, M12/M13 and Free
Cell implementation remain separate. Input permits, STOP/context/target checks,
capture, timings, replay and sibling dependencies are unchanged.

## Complete owned paths

These are the paths owned by 164 relative to the qualified 163 baseline; this
list does not authorise staging or promotion of inherited changes.

- `Cargo.toml`
- `Cargo.lock`
- `src/pyramid_recon/vision/native_colour.rs`
- `src/pyramid_recon/vision/foreground_card/tests.rs`
- `src/pyramid_workflow_model/observed.rs`
- `src/app/pyramid_workflow.rs`
- `src/app.rs`
- `src/worker/pyramid_session.rs`
- `src/worker/pyramid_reconnaissance.rs`
- `src/worker/computed_route.rs`
- `src/worker.rs`
- `src/pyramid_workflow_model.rs`
- `src/worker/computed_route/tests.rs`
- `src/worker/computed_route/wire_tests.rs`
- `src/worker/computed_route/series_guard_tests.rs`
- `src/worker/computed_route/wire_tests/stock_empty_tests.rs`
- `src/worker/computed_route/wire_tests/redeal_tests.rs`
- `src/worker/pyramid_reconnaissance/starting_guard_tests.rs`
- `src/worker/pyramid_reconnaissance/post_confirmation_tests.rs`
- `src/worker/pyramid_session/native_tests.rs`
- `src/pyramid_recon/vision/foreground_card/tests/effect_164.rs`
- `src/app/pyramid_workflow_tests.rs`
- `README.md`
- `CHANGELOG.md`
- `docs/milestones.md`
- `docs/handoff.md`
- `docs/Verification-2.9.164.md`
- `docs/Pyramid-Acceptance-Before-3.0.0.md`
- `docs/Pyramid-Short-Term-Goal.md`
- `doc/scoring-target.md`
- `tests/fixtures/pyramid-recognition-164/README.md`
- `tests/fixtures/pyramid-recognition-164/before-match.png`
- `tests/fixtures/pyramid-recognition-164/calibration.json`
- `tests/fixtures/pyramid-recognition-164/coral-effect.png`
- `tests/fixtures/pyramid-recognition-164/deal.json`
- `tests/fixtures/pyramid-recognition-164/underwater-coral-clearance-exact.rgb`
