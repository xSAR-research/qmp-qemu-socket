# Klondike v1.2.6 candidate 1

Base: committed v1.2.5 candidate 1,
`8125c9df1dd87c7d8e9ed0e928fbb7fdd7bae82b`. One fresh main checkout was taken
for this cycle. Its clean source matched all 19 delivered v1.2.5 file hashes.
The installer accepts this exact committed base, not an older candidate.

## New Game refusal

Log `qmp-qemu-socket-1790964873320.log` proves that the worker clicked the score
counting screen, recognised Level Up after a delayed observation, and acknowledged
its OK click at (960,795). Four subsequent Congratulations observations retained
all title, completed-board and New Game/Home lettering guards. The body guard
remained 5/6, so no New Game click was sent. The pointer in the later screenshot
was left by the OK operation.

Native K53 reproduces that refusal. The old Home-body sample at (990,827) expects
[27,85,166], but the visible pointer shadow produces [10,32,62]. Its other five
body samples match exactly. The single replacement sample at (1020,827),
[27,86,171], is byte-identical in native K28 and K53 and outside the recorded
pointer extent. All six body samples remain mandatory, with the same RGB
tolerance. Scene, title, completed-board and both control-lettering guards remain.
New Game still clicks the measured (792,840); no input coordinates or timings
have changed.

K53 is a later read-only screenshot rather than an exact saved worker frame.
It establishes the supplied settled scene and the probe/pointer collision;
the complete log establishes that the earlier action was refused. Recognition
of the later frame does not itself prove a live New Game, deal or Solver action.

## Pending Solve face

The log first records partial Solve artwork after action 99: interior 441/483
and glyph 304/342. After action 100, all 342 glyph samples match but only 459/483
interior samples match. Three tableau inputs follow such fully lettered frames
before the interior reaches 483/483 and Solve is selected. Four tableau inputs
follow the first partial appearance. No original early Solve PNG was retained,
so the location and cause of the 24 mismatching interior samples remain unknown.

The authoritative Solve predicate is unchanged: independently validated gameplay
scene, at least 90% empty-stock felt, and at least 98% of both interior and glyph
samples within the existing RGB tolerance. DRAW and RIGHT HALOs keep precedence.
No lower threshold authorises Solve input.

A separate diagnostic predicate requests observation when Solve is unavailable,
its stock and glyph checks pass, and at least 95% of the interior matches. Before
a lower-priority tableau or SUIT input, the worker defers that input and performs
at most three delayed, input-free observations using the existing editable
reobserve interval. Only the original full predicate can select Solve. A pending
face that never resolves stops with the latest frame and diagnostics. A vanished
candidate may continue only from a valid freshly classified scene and target;
it does not trigger a speculative Solver click.

The preceding gameplay result is evaluated and counted before this new planning
wait. Finite limits stop before waiting for another action. An originally approved
actionable preview that changes during initial waiting still requires review,
with no gameplay input. Continuous initial Solver recovery and within-run restart
retain their existing authority to plan from a freshly recovered valid target.
STOP, mode/socket invalidation, input acknowledgement and uncertain-input rules
remain mandatory.

Controlled derivatives reproduce the logged 459/483 and 342/342 counts; their
changed pixel locations are test choices, not recovered original early frames.
Tests cover full availability, disappearance, persistence, priority, finite
budgets, changed initial previews, continuous recovery, STOP and input failure.

## Beast verification

Require the complete local check sequence and confirm the launched release label
is v1.2.6, candidate 1. From a clean Klondike game, test continuous Solver-off
startup, source transfers, early Solve appearance, and score → Level Up → New Game
→ Play → Solver → next board. At an unresolved pending Solve, the log should show
bounded captures and zero deferred tableau/SUIT input. A recognised full Solve
still receives one click followed by its separate editable animation settle.

At the first stop retain the displayed screenshot and full log before Undo or
manual movement. Recheck Step Once, finite budgets, STOP, mode/socket changes,
unchanged-source refusal, exact-byte PNG saving and accepted TriPeaks/Pyramid
behaviour. Timings remain editable. No live QMP/guest input was tested in the
development workspace.

## Code Analysis

Visual predicates control guest input authority. The pending predicate grants
read-only observation only; independent scene and full Solve guards remain the
click boundary. New Game relocates one measured colour sample rather than allowing
a failed required sample. Unknown, missing, malformed or unresolved scenes stop
with evidence. These are calibrated visual policies, not independent card-rank
recognition or a universal proof of all guest scenes.

Runtime changes use safe Rust with validated native layouts, fixed bounded pixel
samples and widened threshold arithmetic. Pending waits have a fixed observation
budget and cooperative STOP. Socket timeouts and existing up-only release recovery
remain; uncertain non-idempotent inputs are never replayed. Shared capture and
three-board policies remain unchanged. The installer separately bounds archive
members, verifies exact base and affected content, preserves unrelated local work,
journals backups and refuses unknown edits or unsafe paths. Actual validation and
workspace limitations are recorded in the delivery review.
