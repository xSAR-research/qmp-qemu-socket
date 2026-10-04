# Development conventions for v1.2.6 candidate 6

Candidate 5 corrects the tableau-to-SUIT transfer whose previous source
becomes a landing guide, retaining candidate 4's corrections against committed
candidate 2 at `7d869c22a6ca98179cd3efc8f83552d2edd1ada7`. The installer
recognises one complete base or delivered candidate-4 state, and restores its
actual backed-up contents. TriPeaks and Pyramid retain their execution policies.
Independent card-rank recognition, search and file-free capture are outside
this candidate.

## Naming and documentation

- Use `snake_case` for modules, functions, methods, fields and local bindings.
- Use `UpperCamelCase` for types, traits and enum variants.
- Use `SCREAMING_SNAKE_CASE` for constants and statics.
- Prefer module-qualified names where two modules expose the same concept,
  such as `pyramid::TABLEAU_CARD_COUNT`, rather than a renamed import.
- Numeric `as` casts are conversions, not name aliases. Review their bounds
  and meaning; do not replace them as a cosmetic naming change.
- Place meaningful `///` documentation on functions, methods, constants,
  statics and type definitions. Explain the contract and non-obvious parameters,
  result, failure conditions or authority granted. Document important fields
  and enum variants. Place `//!` documentation at module boundaries.
- Ordinary local bindings and implementation explanations use `//`; attaching
  item documentation to a local `let` does not provide a function contract.
- Keep test helpers and regression cases documented too. A test-only helper
  belongs under `#[cfg(test)]`; do not silence dead-code warnings globally.

These conventions follow the [Rust API naming guidelines](https://rust-lang.github.io/api-guidelines/naming.html)
and [Rust Reference documentation-comment rules](https://doc.rust-lang.org/reference/comments.html).
The crate enables missing-documentation warnings, including Clippy's private
item check. These checks complement review of what each comment actually says.

## Responsibilities and behaviour preservation

`game.rs` defines typed profiles/actions, while each mode owns its detection
and effect policy. The worker owns the QMP connection and cancellation state.
Its `post_game` submodule shares the existing terminal-screen sequence; its
`pyramid_execution` submodule contains Pyramid-specific result observation.
Its `klondike_execution` submodule owns finite or continuous Klondike runs with
bounded per-action recovery and independently verified single-board completion.
`klondike_terminal.rs` owns the evidenced terminal controls. The separate Solve
button is clicked once before bounded completion observations; only continuous
runs may restart through fresh recognised stages. The shared terminal controller
retains the existing TriPeaks/Pyramid policy and coordinates.
The UI delegates persistent log I/O to `session_log.rs`.

When adding a mode, supply its profile/detection policy and preserve the shared
input and capture contracts. Do not scatter new mode checks through QMP or
session logging. A new mode's special rules should remain visible at the game
policy boundary.

The unused provisional `card_reader.rs`, inactive card-rank/history types and
their reader-only thresholds were removed. The provisional reader never
returned a recognised visible rank: it deliberately reported incomplete
calibration. The old implementation is recoverable from Git commit
`dccb7f9ae7ff11acae615f029f6e183b15484a63` if useful during future self-solving
work. Active card geometry and its validation coverage remain.

Preserve capture dimensions, calibrated probes, click coordinates, timing
defaults, QMP command policy and manual PNG byte identity during a structural
refactor. Keep fresh-HALO continuation distinct from proven action effect and
from board/game completion. A progress-bar reading alone is not a win.

## Source spacing

Charlie's 30 September 2026 instruction requires two blank lines before function,
struct, enum, trait, type, constant, static, impl, inline-module and macro
definitions, above attached documentation and attributes. Statement blocks
(for, while, loop, if, match and bare scope statements) receive the same spacing
before the enclosing statement. Documentation remains attached to its item.

Apply this convention to existing source as well as new code. Do not run
`rustfmt` or `cargo fmt` for this candidate. Preserve `rust-toolchain.toml` and
`rustfmt.toml`; changing those files or the global toolchain is not part of the
spacing change. A whitespace-only pass must preserve parsed Rust tokens and
comments and introduce no syntax errors.

## Local verification

Run from the repository root with its selected nightly. Record `rustc --version`
and `cargo --version`. Each check is independent; stop and retain diagnostics on
the first error.

```text
cargo check --locked
cargo test --locked
cargo doc --locked --no-deps --document-private-items
cargo clippy --locked --all-targets
cargo build --locked --release
```

No formatter check is included, by Charlie's explicit instruction. Build/test
results from another host do not prove Beast installation or live guest input.
The candidate delivery notes record the checks actually executed.

On the Beast, confirm the launched executable identifies v1.2.6, candidate 6. Verify
Capture Frame, exact-byte manual Capture PNG, single-step, finite and continuous
multi-step, STOP and mode/socket invalidation. Exercise Klondike Draw 1, source
transfers, RIGHT fan positions, recycle, Solve and late/no-HALO recovery. Preserve first-failure
frames and session logs. Recheck the accepted TriPeaks/Pyramid behaviour before
Charlie commits and pushes the candidate.

For the inherited tall-run behaviour, verify the complete source bounds and
its single click above the toolbar. The reconstructed Draw pair is not a
successful RIGHT transfer pair. Replay success supports the existing input
contract; an intermittent RIGHT no-op still needs live diagnosis. Logged pointer,
hold, action-settle and measured I/O times are separate quantities.

For v1.2.3, replay the three-spades foundation transfer and a last-card removal
leaving a dashed guide over an empty column. The source-removal check must
recognise dimmed green felt without accepting unchanged cards darkened by a
new recommendation. Verify Single Step, continuous mode and STOP, preserving
the first failure frame and full log. Card and recapture timings remain editable.

Candidate 2 also covers a same-column replacement face after automatic reveal,
the zero-input changed-preview endpoint, the default continuous limit and the
five supplied terminal scenes. Verify the new separate Solve delay, finite-run
stop before terminal input, and continuous score → Level Up → New Game → Play →
Solver progression. Unknown or unchanged stages must stop after bounded read-only
observations without another control click. All five input controls remain live
Beast checks; fixture recognition is not a runtime input test.

Remove compiler/Clippy diagnostics by preserving behaviour in the affected code.
Use compile-time assertions for fixed configuration invariants, named evidence
records for grouped recovery guards and boxed failure payloads for large error
variants. Do not introduce warning suppression or run a formatter. The candidate
review records actual compiler versions and warning counts.

Candidate 3 adds the measured 3-hearts recipient transition under a persistent
foundation guide and extends upper-card replacement evidence to SUIT returns,
including stable-paper changes in both printed-detail directions.
The generic material thresholds remain. Test the real RIGHT pair and adverse
guide/paper/source/destination/mask cases. SUIT return tests are explicitly
synthetic. Its named screenshot arrived after candidate 3 validation, but shows
the source recommendation only; an actual before/result pair remains a Beast
verification requirement. See `klondike-v1.2.3-candidate-3.md`.

FreeCell is evidence intake only in `freecell-gate-1.md`. Its implementation follows
Klondike acceptance/promotion and a new authorised cycle; do not introduce dormant
mode code or claim Klondike calibration establishes FreeCell input behaviour.

Candidate 4 separates source-supported fresh-HALO continuation from strict effect
verification. Test finite budgets and reported counts, repeated-source replacement,
unchanged/cursor-only refusals, independent Solve handoff, and prohibition after
uncertain input or Solver refresh. The real black-pip RIGHT pair changes only 251
recipient pixels; it continues without being called verified. Extend the tall-run
case to the measured closing edge partly covered by Undo All. Require the remaining
edge, paired rails, closed top and white face; test gaps outside the exact overlap,
clipped borders and toolbar input/effect bounds. Preserve K19 recognition.
See `klondike-v1.2.3-candidate-4.md` for provenance and live acceptance limits.

Candidate 5 covers a bottom single-card transfer whose source stays mostly white
as the remaining stack spreads. A normal lower corner crosses the toolbar, so the
ordinary corner function keeps its refusal. A separate pair of full visible
patches must retain both paper/ink directions, positive paper and material source.
This new source proof supports fresh-HALO continuation only; a complete effect
remains unverified. Test restoration of either patch, source-only replacement
without complete verification, receiver-only changes, one-way shading/whitening, masks, insufficient material,
run geometry and malformed frames. No toolbar pixels may supply effect proof.
The controller, action budgets, timing defaults and completion policy are unchanged.
See `klondike-v1.2.3-candidate-5.md` for native measurements and Beast checks.

Version 1.2.4 candidate 1 adds K41–K43. Verify the 604-pixel source, the
warmer Solve border and input-free completion across animation into the settled
Congratulations screen. The longer Solve wait uses the existing 20-round
post-game bound and current editable re-observation timing; this is not a fixed
20-second timeout. Card recovery and terminal-control advancement retain their
existing bounds. All new source keeps the requested spacing without a formatter.

Version 1.2.4 candidate 2 retains the same exact Git base and adds K44–K49.
Test the displaced native source, persistent/transient unsupported results, the
source-supported contracted-card continuation and both early/late Level Up
recognition. The provided earlier rollback screenshot is not the original
planning frame for the last bottom-card action; that regression combines native
result pixels, runtime counts and an explicitly synthetic predecessor.
Do not claim K48 establishes the missing live Solve failure: its settled artwork
already passes. Inspect new per-observation Solve diagnostics in the next run.
No input may be replayed after uncertain delivery or unsupported-scene recovery.

Version 1.2.5 candidate 1 adds K50–K52 and corrects continuous initial Solver
recovery. Preserve the finite-run review endpoint. Test stable Solve lettering
with a removed/changed outer ring, unknown scenes and intact stock refusals,
coherent Level Up label alignment, and source-supported bottom-card continuation.
Retain unchanged/cursor/guide/paper/mask refusals and strict-effect distinctions.
The unrecorded frames after the last Level Up OK click remain unverified.

Version 1.2.6 candidate 1 adds native K53 and fixes the Home body sample covered
by the last OK pointer. Preserve all six required body probes and independent
modal/label/completed-board guards. A pending Solve face grants observation
authority only; it never grants input at the lower 95% interior threshold.
Test full availability after bounded captures, disappearance, persistence,
initial changed-preview review, finite budgets, priority, STOP and input errors.
Early Solve pixels were not saved; controlled test derivatives are not original
worker captures. Verify the complete new-game/deal/Solver sequence on the Beast.


Candidate 2 rebuilds the installer records for f9167a0 after both candidate-1
preview/apply refused a changed HEAD. Preserve Charlie's README purpose edits.
Require `--verify-installed` before every Cargo or launch block; download hashes
do not prove installation. Its read-only check requires complete candidate bytes
and modes, the exact base HEAD and an unchanged affected index, and refuses the
complete old base as well as unknown/mixed content. It does not create evidence
directories or change repository files.

The latest hearts stop occurred on RIGHT 5-hearts to SUIT, exposing 8-hearts;
6-hearts is the next tableau recommendation. Logged source change is only 98
pixels with 7 reverse corner print pixels. The Undo reconstruction contains a
separate gold-outlined 5-hearts over the table, so it is not an accepted fresh
Waste planning frame. No source threshold is changed from this evidence. Obtain
a settled original pre-click PNG and one-action result/log before authorising a
new source-effect policy. See `klondike-v1.2.6-candidate-2.md`.

## v1.2.6 candidate 3 verification

Packaging revision r2 uses exact base 7d869c22a6ca98179cd3efc8f83552d2edd1ada7.
Candidate 2 is now the committed base; no predecessor-package authority is needed.
Manifest schema 4 retains whole-state checks and rollback to that base.
Installed verification requires candidate 3 before
Cargo or launch. Native K54-K64 regressions distinguish outline detection from
card-relative source continuation; complete effect stays unverified for that
continuation route. Keep the full unfiltered checks and Beast end-cycle test.

The ordinary replacement regression independently binds the five-pixel nominal
face offset with unchanged artwork. Test missing/duplicate seams, paper rails,
felt gutters, pure translated print and independent recipient requirements.

Test K59/K60 queen transfer and retained-card guide darkening, with cursor/gold,
source-only and recipient-only refusals. Keep the older dimmed-felt predicate and
strict complete effect verdict unchanged.

Test K61/K62 Draw-to-long-source classification, the previous 60 scene outcomes,
exact 187/186 felt acceptance, independent gutters and invalid outlines. K63/K64
require the disjoint positive-felt union: neither shade component alone reaches
512. Retained-card neutral guide darkening, material/cursor/gold and opposed-half
negatives must still refuse continuation. No stack input was sent in K61/K62.


## v1.2.6 candidate 4 verification

K65/K66 are the later Undo predecessor and stopped result for the SUIT 2 return.
Complete paired source corners and a uniquely located new recipient face support
continuation only. New receiving print must match old source artwork and differ
from the prior recipient; the geometry is selected before the print comparison.
Keep old source material thresholds and the 96-by-96 cursor exclusion intact.
Run native, adverse and production-controller regressions, including source-only,
recipient-only, shading, translated print and ambiguous receiving seams. Require
complete source verification before each Beast build or launch. Current check
results, actual compiler versions and remaining limits are recorded in the
candidate delivery review. Do not run a formatter or suppress warnings.

K67 supplies a separate Pro Level Up panel signature, retaining the completed
foundation/empty-tableau context and the existing OK coordinate/hold/settle. The
full native restart integration uses real completion and terminal classifiers;
no mocked stage approval hides a detector failure. Require only one click per
recognised terminal stage and retain read-only bounds for unknown transitions.


K68/K69 cover the new column-7/column-3 source borders at row 997. Preserve
closed edges, terminated rails, strong rail bracketing around the measured
column-7 shading band, paper above the toolbar and the existing height limit.
K69 is a detection-only sparse fixture, not material-effect evidence. The
original Strange-fail frame already selects its column-5 source; a matching log
and immediately preceding frame are required to diagnose the earlier action.


## Candidate 5 continuation regressions

K70-K73 are complete original-byte manual uploads. Test newly received SUIT ink
and the independently exposed source header together, then remove each separately.
Test neutral paper whitening, retained old source print, masks, ambiguous or
missing geometry and source rows at the toolbar boundary. Native controller
checks distinguish continuation from verified effect/completion, preserve finite
budgets and continuous STOP, and stop before reading evidence after uncertain
input. K70/K71 does not reproduce the logged automatic highlight or effect counts;
K72/K73 reproduces the unavailable source proof in native image analysis.


## Candidate 6 detection and restart regressions

K74/K75 show the same unmoved two-spades source in two HALO intensities. K75
must reproduce the full outline through the measured Hint shadow using only
positive gold samples; remove each rail bracket, edge sample and card/scene
guard separately. The phase pair must not establish a card effect. A repeated fresh target may
authorise another logical action under the current policy; uncertain input may
never be replayed. K76's displaced copy is detection evidence only; no action
semantics are assumed for it.

K77 shows an initial deal after Play with Solver inactive. Check the entire
terminal sequence through one Solver click and fresh observations, and stop
before further input on STOP or uncertain acknowledgement.
Remove card perimeter, stock/waste/foundation and inactive-banner evidence to
prove a generic no-HALO scene cannot authorise SolverReady. The earlier logged source comparison with 454 aligned material and 21 cleared
pixels remains a historical diagnostic; it no longer gates live Klondike input.


## Candidate 6 policy update from Charlie

The current Klondike controller follows fresh valid HALOs after acknowledged
input and settle. Remove old worker assertions that demand card-effect proof;
retain those analyser fixtures as test-only diagnostics. New worker tests must
cover the native K78/K79 SUIT-return then RIGHT recommendation, repeated fresh
HALOs, advisory-preview changes, finite action budgets, bounded no-HALO/Solver
recovery, unknown scenes, STOP and uncertain-input non-replay. Report acknowledged
Solver actions separately from independently verified completion. TriPeaks and
Pyramid retain their accepted shared policies.
