# qmp-qemu-socket

Rust desktop agent for **vision** (calibrated detectors and effect proofs),
**allow-listed QMP↔QEMU control**, and an **egui** UI. It connects to an
existing QEMU QMP Unix socket, captures the guest display, and issues only
guarded mouse/keyboard actions the policy allows. It does not launch QEMU or
create the socket.

Microsoft Solitaire & Casual Games modes used here — TriPeaks, Pyramid, Klondike,
and future game types — are **disposable guest fixtures** for exercising 
capture, detection and guarded input on shared QMP facilities. They are not
the real purpose. Each fixture keeps independent target and effect policies.

The target is to build in Dijkstra / A* shortest path problem solving rather
than using the **Solver**, this will benefit drone route planning experience.

This is **v1.3.0, candidate 3**, based on pushed candidate 2 at
`656881ee3f906347bd6f5665c733892621b1cc98`. `Cargo.toml` supplies the package
version shown in the window title and Parameters. Original FC01–FC13 evidence
and Charlie's approved input contract now enable **Free Cell**. Existing
TriPeaks, Pyramid and Klondike execution stays intact.
See [Free Cell candidate notes](docs/freecell-v1.3.0-candidate-3.md).

## Build and source spacing

`rust-toolchain.toml` selects floating `nightly`, with Clippy and Rustfmt
components. `Cargo.toml` declares edition 2024 and minimum Rust `1.101.0`.
The channel name is not a dated compiler pin. Record the actual compiler and
Cargo versions used; stable compatibility has not been established here.

```text
rustc --version
cargo --version
cargo check --locked
cargo test --locked
cargo doc --locked --no-deps --document-private-items
cargo clippy --locked --all-targets
cargo build --locked --release
```

Do not run `rustfmt` or `cargo fmt` for this candidate. Charlie requested two
blank lines before Rust definitions (above their documentation/attributes)
and statement blocks, including existing source. The committed nightly and
rustfmt configuration and existing source spacing are preserved; no formatter
is part of this candidate.


## Free Cell

Board names are **CELL 1–4** at the upper left, **PLAY 1–8** across the tableau,
and **SUIT 1–4** at the upper right, numbered left to right. Select Free Cell,
then Single Step or Multiple Steps. Read-only Capture Frame and Capture PNG
never activate Solver or send gameplay input.

Each run starts with a fresh frame; its preview is advisory. Scan CELL first,
then PLAY bottom-up and left-to-right. One solid highlighted source block
receives one bottom-card click, including an entire run, followed by editable
settle and a fresh frame. Dashed destination guides are not sources. There is
no card-rank, recipient-card, position-change or changed-pixel move proof.
Repeated source positions are valid. SUIT piles are never scanned for input;
Free Cell has no Draw, Recycle or Solve operation.

Automatic SUIT transfers can temporarily leave no HALO. A positive win-entry
check is independent of source detection. Otherwise observe again, without
clicking an active Solver. A positively recognised Solver-off board may activate
Solver once, settle and capture anew. Unknown scenes or unresolved sources stop
after the bounded allowance, retaining the latest frame and complete log.

**Params** offers **action / automatic-transfer settle** and **re-observation**
intervals, initially **750/1000 ms**, editable from 0 to 5000 ms, plus **Delayed
observations per unresolved stage**, initially **20**, bounded **1–100**. These
starting values are not measured animation times. **Actions per Multi-Step**
defaults to **0 = continuous**; finite budgets are **1–10000**. Step Once sends
at most one source action. Settings are snapshotted for each run.

**One Free Cell board is one game.** GAME WIN entry requires the score/skip
panel or the completed-game New Game panel; missing HALO, isolated OK and Play
do not establish a win. Only continuous mode follows score skip → optional OK
→ New Game → Play → fresh board → Solver activation → fresh HALO. After score
counting, each fresh frame checks both OK and New Game. A game without a level
change proceeds directly to New Game; if both controls appear ready, input stops. Expected
controls use their local lettering and button body, not level, rank, medal,
fireworks or surrounding artwork. Each acknowledged click is sent once; an
unready next stage receives input-free observations, not click retries.

The original **FC01–FC13 and FC15** PNGs are included with hashes. FC15 shows
New Game after a win without LEVEL UP. **FC14 is absent**:
source probes exclude the toolbar beginning at **Y=947**, and a block without a
complete visible lower outline remains unsupported. See
[Free Cell Gate 1](docs/freecell-gate-1.md) and the candidate review for evidence,
validation limits and Beast checks. Native Rust validation was not available
in the delivery workspace; build/test and live GAME WIN validation remain
mandatory before acceptance.

## Klondike Draw 1

Select **Klondike**, activate the guest Solver and capture a fresh frame.
Selection priority is **DRAW/RECYCLE → RIGHT HALO → RIGHT Solve → tableau bottom
upwards → SUIT piles → completion evidence**.
An ordinary stock draw sends the confirmed `d` shortcut; exhausted-stock recycle
clicks its recognised highlighted area. RIGHT detection follows the up-to-three
card fan. A solid highlighted source block receives one click, including a whole
run or a source from a SUIT pile. The dark dashed destination is never clicked.
Newly exposed tableau cards reveal automatically.

Tall source runs can overlap the guest toolbar, which begins at **Y=947** in
the calibrated frame. Live tableau HALO detection reads only rows above that
boundary. A run continuing beneath it needs a visible closed top, connected
opposing rails reaching the boundary and bright card paper. Its reported bottom
is the visible cutoff, not an estimate of the hidden card edge. The existing
upper-card click stays above the toolbar. Solver and terminal controls retain
their separate areas; full-frame PNG capture and saving remain unchanged.

**Params** provides independent Klondike card/draw/recycle, Solve-animation and
recapture intervals, initially **750/750/1000 ms**, editable from 0 to 5000 ms. They are
user-approved starting settings, not measured animation timings. Single Step
sends at most one gameplay operation; Multi-Step defaults to **0**, with a
positive range of **1–10000**, plus **0 = continuous until STOP or a guarded
stop condition**. Recovery remains bounded for each action in continuous mode.
Timing edits apply to the next run.

The Klondike preview is advisory. The worker captures fresh supported scene and
canonical target evidence, sends one operation, settles and observes anew. Each
acknowledged gameplay action consumes one finite budget slot. Single Step stops
after that action and result; remaining finite slots and continuous mode follow
each fresh valid HALO, including repeated targets and changing stack geometry.

Missing HALOs receive at most three delayed input-free observations, then an
independent completion check. A recognised gameplay scene may receive one Solver
refresh per unresolved context, followed immediately by capture and at most three
further delayed observations. Normal pointer settle, hold and acknowledgements
still apply. Unknown scenes or uncertain delivery stop; no uncertain input is
replayed. Recovery alone does not consume a gameplay slot.

The separate **Solve** button can replace RIGHT when the guest offers automatic
finishing. Its stable button interior and complete lettering, with empty stock and a
recognised scene, authorise one click without waiting for the animated outer HALO.
The click is followed by its
separate editable animation delay and up to 20 delayed read-only completion
observations, using the existing post-game bound and editable Klondike
re-observation interval. Two consecutive positive observations are still required;
ordinary card recovery retains its three-observation bound.
An unresolved result stops for review without retrying Solve or Solver.
The lower-toolbar **Solver** control remains the hint/recommendation control.

Live Klondike play does not compare source or recipient card pixels. Logs count
acknowledged Solver-directed actions and keep their effects explicitly unproven.
The previous effect analyser and native regressions remain test-only diagnostics.
An acknowledged no-op retaining a valid HALO can produce another fresh action in
continuous mode; completion still requires independent positive evidence.

**One Klondike board is one game.** A win requires two fresh positive observations:
either an intact active Solver banner with every calibrated right-interior pixel
gold and none black, or independently recognised completed-game artwork. No black
on a green background or blue overlay is insufficient. Finite runs stop at the
confirmed win. Only continuous Multi-Step **0** may then advance through the
score-skip, Level Up OK, New Game, Draw 1 Play and fresh-board Solver
stages, one guarded click each, before resuming from a fresh actionable board.
After independent win confirmation, the expected OK, New Game and Play buttons
are checked locally for a gold body and printed-caption contrast. Rank, medal,
title, surrounding frame and fireworks colours do not gate these three clicks.
Their ordered readiness waits allow up to 20 delayed input-free observations;
fresh-deal Solver/HALO recovery retains its separate shorter bound.
Unknown stages, unsupported target evidence or uncertain delivery stop with
the latest frame and logs.
Klondike owns these controls independently of the shared three-board controller.
See [accepted Klondike boundary notes](docs/klondike-v1.2.6-candidate-11.md)
and [current architecture](docs/architecture.md) for policy and evidence limits.

## QMP connection

The default socket filename is `qmp-qemu-socket.sock` in `XDG_RUNTIME_DIR`.
The QEMU `-qmp unix:` path must match the path shown under Params. Set
`QMP_SOCKET_PATH` when launching the application to use a different existing
socket during a QEMU launch configuration change. Params also permits changing
the path in the running application, followed by a fresh read-only capture.

On the Beast the socket is `/run/user/1000/qmp-qemu-socket.sock`. Use that
absolute path; the application connects to QEMU's existing listener and does
not launch QEMU or create its socket. Temporary capture files are placed beside
this socket, independent of the repository working directory.

QMP captures and guarded actions use the 1920×1080 primary display calibration
with guest display and text scaling at 100%.

## TriPeaks late-HALO recovery

After a no-HALO result, TriPeaks takes up to three delayed input-free captures
before considering one Solver refresh on a recognised gameplay scene. A fresh
stock HALO proceeds directly to D. The delay is editable in Params, defaults
to 1000 ms and is snapshotted for the active run. After Solver, at most three
further delayed observations are allowed in that unresolved context. Persistent
absence stops with the latest frame; ordinary recovery cannot repeatedly click
Solver. Existing action-effect and board/redeal checks remain in place.

## Pyramid gameplay

Select **Pyramid**, activate the guest's **Solver**, and capture a fresh frame.
**Single Step** and **Multiple Steps** require a concrete prediction belonging
to the selected mode and socket. The worker freshly validates the initial
prediction, sends D for a highlighted MOVE/Recycle or clicks one card/pile target,
waits the configured settle time (default 1000 ms
after Move or 2000 ms after a card/pile click), and verifies the result. Each
verified result becomes the next planning frame without another pre-click
screenshot. **STOP** cancels the active run.

If a verified action has no eligible halo yet, the worker waits the configured
repeat interval (default 1000 ms) and
takes another QMP screenshot, up to a bounded limit. It does not click Solver
on an ordinary halo-free Move, card, Left or Right result. Solver recovery is
reserved for a positively verified new board with no halo after repeated
observations. An unverified action stops and displays its latest result frame
for inspection; use **Capture Frame** for a new approved preview before running
again.

**Params** exposes separate Pyramid **Move / Recycle**, **Card / Left / Right**,
and **repeat observation** delays, each editable from 0 to 5000 ms. They are
session settings captured when the next run starts; Restore execution defaults
sets them to 1000/2000/1000 ms. For a late MOVE halo after cards fly away, increase
**Card / Left / Right** and/or **repeat observation**. TriPeaks retains its own
draw/tableau settings. Double-click a number to type milliseconds, or drag it;
STOP any active run and close the snapshot dialog before editing. Labels identify
the click that starts the wait: a card-to-MOVE transition uses the card delay.
The package-derived candidate label is displayed in the title and Params.

The 1.2.0 baseline refactors the existing Solver-driven controller without
adding a game mode or self-solving algorithm. Shared post-game handling,
Pyramid result handling and session-file logging have focused modules. Rustdoc
comments describe functions and data contracts; unused provisional rank-reader
and card-history scaffolding has been removed. See [development notes](docs/development.md)
for naming, documentation and verification conventions.

Candidate 2 removed the unused BGRA pixel format and its conversion branches.
The current PNG decoder produces RGBA8: eight bits per channel, four bytes per
pixel. RGBA decoding and padded-row detection checks remain covered by tests.

Pyramid Left/Right verification can also use the positive removal of the single
other highlighted tableau card from the planning frame. This covers a pile
replacement whose visible change is too small outside the cursor exclusion.
When that unique pre-highlighted partner is the opposite pile, verification
measures both pile interiors together against the 128-pixel minimum, or accepts
the partner's positive disappearance. This also covers consecutive Left/Right
pairs whose replacement cards have similar faces. Unrelated pile changes are
excluded from this additional verification path.
The detector still requires recognised gameplay and known pile-face evidence;
a new halo or loss of the old halo alone does not prove the previous effect.
An identical **LEFT–RIGHT pair may repeat**: after the configured card settle
and a fresh capture, the same unique eligible pair's HALOs authorise the next
operation even when its pixels have not changed. A tableau card must remain
visible, and no final-card, board-complete or redeal phase may be pending.
This is logged and counted separately as **continued from fresh halo**, without
claiming removal or advancing a completion counter. Step Once still sends one
click; Multi-Step plans the next click from that fresh frame. Per-action evidence
logs include both pile measurements, the unique highlighted partner, the qualifying pair's combined count and any removed partner.

A final-apex redeal requires restored cards and `AnotherBoard` progress in two
consecutive fresh captures. A conflicting progress reading triggers bounded
input-free recaptures and logs the measured right-probe pixels. Old consumed
card state is reset before selecting a target on the verified new board. A
persistent conflict stops for inspection; it never authorises a score click.
The shared progress probe samples the bar interior at `(1050, 87, 38, 2)`.
The previous y=84 probe sampled its gold border; the supplied new-board PNG
provides the regression pixels for this correction.

The priority is **Move → Left → Right → Cards**, with cards scanned left to
right from row 7 up to the apex. Multiple highlighted cards are expected;
only the first eligible target is clicked. The guest Solver removes the pair
from that one click, and a highlighted King also needs only one click.

Only a clicked card whose removal was verified is marked consumed for the
current board. Move and the two piles remain repeatable. Missing halos, QMP
acknowledgements and arbitrary visual changes cannot mark a card consumed or
authorise a Move click. Unknown or unresolved results stop after bounded
observation. Mode/socket changes and **Clear Output** invalidate retained
progress; a confirmed new board also resets its card state.
If a guarded run stopped across a missed redeal, **Capture Frame** can recover
without cycling Game Type: a fresh halo on a card previously verified removed
invalidates the old board record. The same frame is re-scanned without another
screen dump or guest input. The separate automatic transition fault was tracked
in issue #7 and is now resolved.

**Detailed output** uses a fixed panel height, increased by about 2.5 lines.
When opened it reserves that space from the preview; further window growth
still goes to the image.

Automatic capture currently uses a temporary PNG beside the socket: QEMU writes
the file, the worker reads it and decodes RGBA pixels, and normal cleanup tries
to remove the file. Detection then operates on that decoded frame in memory.
This is not file-free or native-framebuffer capture. File-free automatic
capture remains a separate requirement that needs implementation and measurement.

## Capturing an original PNG

Click **Capture PNG** to request one fresh read-only QMP screenshot. The dialog
displays that capture while you enter an optional label. The label field has
keyboard focus; Enter or **Save PNG** writes the exact captured PNG bytes without
another screen dump. Right-click the field for Cut, Copy and Paste. **Recapture**
replaces the pending image, while **Cancel** discards it without saving.

The default output directory is `$HOME/Pictures/Screenshots`, then
`$HOME/Pictures`, then the system temporary directory. Override it with
`QMP_SNAPSHOT_DIR`. Files use a local timestamp, a bounded label and exclusive
mode-0600 creation. The image displayed in the dialog may be scaled to fit;
the PNG on disk retains the original 1920×1080 bytes.

## Post-game handling

TriPeaks and Pyramid use the same visual progress probe and guarded post-game
sequence. Klondike uses its separate one-board policy described above. After a
verified TriPeaks/Pyramid board completion, the Solver header progress probe
selects redeal or game completion; the session board counter is advisory.
Before the first score-skip click, fresh frames check whether an actionable
new board or a terminal dialog has appeared. Two consecutive non-gameplay
frames are needed before a score-skip click; other transition frames receive
bounded input-free recaptures. A recovered old board does not count as a
completed game in the UI.
The controller waits three seconds after each score-panel click. If Level Up is not yet visible, it makes an input-free follow-up capture before considering
a bounded score-skip retry. The general inter-stage wait remains one second.
A tall Level Up OK button may fill both calibrated layout probes; a strong
gold bridge and button interior must connect them before the lower click point
is used. Ambiguous layouts are recaptured with a bounded wait and no input.
When a fresh frame shows New Game already visible instead of Level Up, a
second fresh frame must confirm New Game before continuing.

Challenge Complete **Continue** geometry is recorded in Params for a future
challenge flow; no automatic Continue click is enabled.

## Diagnostic records

Private session logs use `QMP_SESSION_LOG_DIR` when set, otherwise `$HOME/tmp`
when it exists, otherwise the system temporary directory. On the Beast, keep
`/home/charlie/tmp` available for retained evidence. Log files use the
`qmp-qemu-socket-` prefix and mode 0600; the on-screen log is bounded separately
from the full session log.

See `docs/qmp-and-capture.md` and `docs/architecture.md` for the capture and
input contracts, and `docs/pyramid-execution.md` for Pyramid geometry,
verification rules and evidence limits. The candidate review records tests
actually run; live King, pair, pile, recycle and transition checks remain part
of Beast acceptance.

## Historical candidate evidence

The following entries retain the decisions and validation limits of their
deliveries. Earlier card-effect witnesses and toolbar/icon exceptions are
historical diagnostics; the current Klondike policy above supersedes them.


## v1.2.4 candidate 1 evidence

The latest ten-card column 6 source is 604 pixels high and closes at rows
991–993 beneath Undo All. Detection accepts only the measured overlap; its click
and effect proof remain above the toolbar. Solve recognition also accepts the
recorded positive red warming of its outer border without widening its inner
artwork, glyph, empty-stock or scene checks. Priority is unchanged.

The new settled Congratulations frame passes the existing terminal signatures.
The prior short Solve completion budget expired before that frame was captured.
This candidate uses the existing longer post-game observation bound after Solve,
with no additional gameplay or Solver input while waiting. See
[the candidate notes](docs/klondike-v1.2.4-candidate-1.md) for evidence and limits.

## v1.2.4 candidate 2 evidence

K44–K49 cover a horizontally displaced source, the later bottom-card failure,
Solve availability and Level Up at level 49. Unsupported post-action frames now
receive bounded input-free recapture before refusal. Bottom-card continuation
retains independent source evidence and remains distinct from full effect proof.
The Level Up signature avoids the measured text shadow and admits narrowly
bounded warm particle lighting. Timings and the terminal advancement budget stay.

The supplied settled Solve frame already selects Solve with candidate 1. Candidate2
logs empty-stock, artwork and glyph counts alongside scene/selected target for
fresh run observations, so another live refusal can be diagnosed without guessing
at later screenshot pixels. See [the candidate notes](docs/klondike-v1.2.4-candidate-2.md)
for provenance, bounded policies and Beast checks.

## v1.2.5 candidate 1 evidence

K50/K51 reconstruct the bottom 4-clubs transfer to SUIT after an Undo. The next
source is 5-clubs in column 3; the exposed card in column 1 is 5-diamonds. The
source retains mostly white paper and uneven printed-detail changes. A narrowly
guarded stable-paper comparison can support continuation to the fresh HALO;
the prior complete effect remains unverified. No rank or suit is read by code.

New Solve diagnostics show complete lettering repeatedly passing while the full
button template fails. Detection now uses the stable interior and lettering,
with stock and scene guards, independently of the animated outer ring. K52
adds the measured two-pixel centred Level Up label shift. Terminal diagnostics
report recognised stages and named guard failures. Exact frames after the last
reported OK click were not supplied, so their refusal remains a live check.
See [candidate notes](docs/klondike-v1.2.5-candidate-1.md).


## v1.2.6 candidate 2 installation recovery

The previous candidate was not applied: its preview and apply both refused the
changed HEAD, and the subsequent build/test/launch remained v1.2.5. This delivery
rebuilds exact-base/content records for f9167a0 while preserving the revised
project-purpose opening above. Download hashes prove package integrity; they
do not establish installation or executable identity. The candidate commands
verify installed source before each build/test/run step. Stop on any failure.

This candidate includes the prior measured New Game pointer-probe correction
and bounded read-only settling of a nearly recognised Solve control. Full Solve
input authority, STOP, mode/socket invalidation and uncertain-input refusal
remain. See `docs/klondike-v1.2.6-candidate-1.md` for the original evidence and
`docs/klondike-v1.2.6-candidate-2.md` for recovery and current verification limits.

## Klondike v1.2.6 candidate 3 correction

Candidate 3 retains the candidate 2 New Game/Solve fixes and adds narrowly
measured long-outline, clear-gutter and source-evidence handling. Stack reflow is compared
in card-relative coordinates; printed source support can authorise fresh-HALO
continuation while the complete previous effect remains unverified. Disjoint
chromatic felt ranges support the queen and King transfers under the same bounds. See
[the candidate 3 notes](docs/klondike-v1.2.6-candidate-3.md) for evidence and limits.

Packaging revision r2 is pinned to committed candidate 2 at
`7d869c22a6ca98179cd3efc8f83552d2edd1ada7`. It replaces the refused f916-based
package without changing candidate-3 Rust code or fixtures. Base and candidate
contents are checked as complete states. Rollback restores this committed base.
Mixed or unknown files stop; a HEAD refusal reports expected and actual commits.
Run `--verify-installed` before every build/run block; this requires candidate 3.

## Candidate 10 verification and output readability

Candidate 10 includes the candidate 9 Hint-shadow fix and the measured Undo All
border overlay correction. K91-K93 record level-101 restart buttons; K94 records
the later manual source frame with Undo All overlapping its lower gold edge.
These manual PNGs are evidence, not assertions of exact historical worker timing.
See [candidate 10 notes](docs/klondike-v1.2.6-candidate-10.md).

Detailed output uses separate selectable labels in a vertical scroll area.
**Text size** adjusts only those labels, from 10 to 24 logical points. Copy Output
and copied/logged text remain unchanged. Mouse-drag selection does not scroll
beyond the viewport. The visible buffer retains at most 2000 entries and clears
after every three completed games; the complete session file remains on disk.
Entries have no individual byte limit, and copying the complete file temporarily
loads its text into memory. The entry bound is not a fixed byte-memory bound.
