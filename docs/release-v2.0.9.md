# v2.0.9 — TriPeaks direct Level Up transition

Promoted source base: `cf4e5c696052d248b39c956cab6492099ab44020`.
The fresh checkout uses tree `e0ef5784294d9ad5af25a2341b16940eaa4cc656`.
The release label is **v2.0.9**, without a candidate suffix. Only the root
application versions change in Cargo.toml and Cargo.lock; dependency records
and the xsar revision remain unchanged.

## Reported failure and evidence

The final section of `qmp-qemu-socket-1791458400162.log` records operation 420
clicking TriPeaks row 1, column 2 at (960, 201) once. The worker proved the
last-card effect, then stopped after 20 result captures over 28.792 seconds
without recognising the reward panel. No final-card replay, Solver input or
terminal click followed that operation.

The supplied 1920 × 1080 image shows a foreground **LEVEL-UP!** modal covering
the Congratulations reward panel. The existing raised OK probe matches the
gold button completely; its established click point (960, 770) is appropriate.
The failure is terminal entry rather than missing OK geometry. The v2.0.8
final-card path required the uncovered Congratulations signature and waited
before reaching the older Level Up recovery, which itself requires prior
board-completion authority.

The screenshot was requested about 339 seconds after the stop. It identifies
an unsupported terminal state but does not establish the exact pixels in the
earlier 20 captures. The supplied attachment is retained unchanged as TP02:
1,741,560 bytes, SHA-256
`6b73803c9accba96b821473e3178aa18c2b02312ce635dc91dbe810b53b9bea7`.
The log reports saving 1,102,609 bytes, so byte identity with the earlier
saved QMP file is not asserted.

## Resulting behaviour

TriPeaks recognises two separate terminal types: the existing uncovered
Congratulations reward panel, and the supplied Level Up panel covering that
reward. The Level Up signature uses fixed title lettering and banner colours,
panel edges, visible dimmed reward lettering and border strips, and exactly
one existing OK control variant. Level, rank, animated central artwork,
pointer and desktop chrome do not supply terminal authority. A general Level
Up popup without the supported reward context is insufficient.

Both types require a delivered canonical top-row action from a fresh board
with exactly one exposed top-row card, the existing cursor-excluded material
effect threshold, and no fresh gameplay, target or occupied-row evidence.
Two consecutive fresh observations of the same terminal type confirm entry.
Switching between reward and Level Up restarts the consecutive count while
retaining the spent confirmation budget. An advisory preview cannot establish
completion.

A confirmed Level Up observation is passed into the shared post-game
controller at its existing OK stage. It sends no centre skip before OK. If
OK reveals the uncovered reward panel while awaiting New Game, two consecutive
fresh reward observations authorise the existing bounded skip; direct
OK-to-New-Game progression is also supported. Unknown panels do not authorise
a skip. Fresh evidence authorises each subsequent input; a direct-entry OK retry
requires the narrow Level Up signature again. STOP is checked after the fresh
QMP probe and before current or earlier-stage control input. Uncertain click
delivery stops without replay.

The existing 20-capture unsupported-animation allowance and four-capture
confirmation allowance remain unchanged. Unsupported panels permit only
bounded, cancellable observation. Fresh actionable or nonempty gameplay
returns to the established controller policy. The original reward-skip flow,
Pyramid completion rules and the three independent one-board controllers
retain their existing policies and coordinates.

## Verification and acceptance

The package validation records retain the actual compiler versions and exit
statuses for Cargo check, the locally executable test suite, strict private-item
documentation, Clippy and the release build. Host-only QMP tests are reported
separately. Live acceptance remains Gate 3 on Beast.

Production-worker QMP regressions use a private AF_UNIX listener. The delivery
executor refuses Unix socket creation with EPERM; these host tests are
compiled and explicitly ignored by default, rather than reported as locally
passed. Gate 3 on Beast must run `cargo test --locked -- --include-ignored`
before live launch. Native screenshot, detector, action-proof and confirmation
tests run without sockets.

Recorded frames and a simulated QMP peer cannot establish the original modal
timing or a live guest restart. Gate 3 must verify TriPeaks winning through
both supported terminal entries, a fresh OK click for the direct Level Up
entry, any subsequently revealed reward panel, the New Game and Play stages,
image-control updates, STOP,
and successful resumption on a fresh supported board.

The package contains complete replacement files, pinned source hashes and
modes, a guarded installer, its validation evidence and numbered Beast
instructions. The installer accepts only the exact promoted v2.0.8 baseline
or the complete installed v2.0.9 state at that baseline HEAD. Unknown edits,
mixed affected contents, staged affected files, links, wrong HEAD and checksum
failures refuse replacement. Applying backs up affected files privately and
does not change the Git index. Source rollback requires a rebuild before the
restored executable is launched.

## Code Analysis

The threat surfaces remain externally produced screen pixels, the selected
QMP socket, QMP replies, filesystem-backed PNG acquisition and the installer
payload. Visual recognition is calibrated evidence, not authentication of a
guest or its contents. Positive fixed-component checks, supported reward
context and immutable final-action evidence constrain which pixels can grant
completion and terminal input authority.

The implementation adds no unsafe Rust or dependency change. Frame dimensions,
stride, storage and probe bounds are checked before pixel access. Dynamic
artwork is excluded rather than read through unchecked coordinates. Mode-owned
terminal types keep a TriPeaks exception out of Pyramid's completion rules.

Capture, confirmation and click budgets bound unresolved progression. STOP
checks and uncertain-delivery refusal prevent replay at the new OK boundary;
they cannot retract an input already sent or interrupt an in-flight QMP call.
Unknown, damaged, ambiguous, transient or insufficient-effect evidence stops
with the latest diagnostic frame. The existing transport timeout and temporary
capture-file limitations remain unchanged.

The installer verifies complete bounded members, paths, hashes, modes and
expected source states before writing. Private backups and apply journals
support exact source restoration. These checks detect unexpected package or
source changes; they do not replace verification of the downloaded ZIP
against the independently supplied SHA-256.
