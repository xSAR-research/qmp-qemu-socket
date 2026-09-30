# qmp-qemu-socket

A Rust desktop application for inspecting and controlling a QEMU VM through its
QMP Unix socket. The current guest is Windows 11 running Microsoft Solitaire &
Casual Games. TriPeaks, Pyramid and Klondike use fresh visual evidence and shared
QMP/capture facilities, with independent target and effect policies.

This is **v1.2.2, candidate 2**, based on committed v1.2.1 at
`1cc57c21e696759b5d61bbcee24b96a733e9c430`. `Cargo.toml` supplies the package
version shown in the window title and Parameters. Beast gameplay acceptance
is required before promotion.

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
rustfmt configuration are preserved. The candidate's spacing pass inserts
blank lines and checks that Rust tokens and comments are unchanged.

## Klondike Draw 1

Select **Klondike**, activate the guest Solver and capture a fresh frame.
Selection priority is **DRAW/RECYCLE → Solve or RIGHT → tableau bottom upwards → SUIT piles**.
An ordinary stock draw sends the confirmed `d` shortcut; exhausted-stock recycle
clicks its recognised highlighted area. RIGHT detection follows the up-to-three
card fan. A solid highlighted source block receives one click, including a whole
run or a source from a SUIT pile. The dark dashed destination is never clicked.
Newly exposed tableau cards reveal automatically.

Tall source runs can overlap the upper part of the guest toolbar. Klondike
recognises the measured dimmed closing edge and rejects internal card edges
while the side rails continue below them. The source click stays above the
toolbar; toolbar pixels cannot supply move-verification evidence.

**Params** provides independent Klondike action-settle and recapture intervals,
initially **750 ms** and **1000 ms**, editable from 0 to 5000 ms. They are
user-approved starting settings, not measured animation timings. Single Step
sends at most one gameplay operation; Multi-Step defaults to **10**, with a
positive range of **1–10000**, plus **0 = continuous until STOP or a guarded
stop condition**. Recovery remains bounded for each action in continuous mode.
Timing edits apply to the next run.

The worker validates the initial preview against a fresh frame, sends one
operation, settles, observes and verifies its effect. The accepted result is
reused for the next plan. A recognised Klondike scene with no eligible HALO may
receive one Solver activation, followed immediately by capture with **no added
post-click delay**. Normal pointer-settle/button-hold and QMP acknowledgements
still apply. Further unresolved captures are bounded and use the editable
recapture delay. An initial no-HALO preview permits recovery only: the recovered
preview is displayed and another explicit Step request is required for gameplay.
Recovery does not repeat an uncertain gameplay operation.

The separate **Solve** button can replace RIGHT when the guest offers automatic
finishing. Its measured button appearance authorises one click, followed by
settling and a fresh capture. The run then stops for result review, including
in continuous mode; it does not claim a verified win or send further input.
The lower-toolbar **Solver** control remains the hint/recommendation control.

Transfer verification requires source and destination changes. For RIGHT, a
bounded comparison of both opposite card corners also recognises replacement
faces with little changed central artwork. It compares pixels without decoding
ranks or suits. Logs record the separate proof measurements on unresolved effects.

Verified completion and restart are unsupported in this candidate.
Unknown scenes and unresolved effects stop with the latest frame and logs;
Klondike does not enter the existing three-board or post-game controller.
See [Klondike correction notes](docs/klondike-v1.2.2-candidate-2.md) for evidence,
verification boundaries and live checks.

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

## Pyramid gameplay

Select **Pyramid**, activate the guest's **Solver**, and capture a fresh frame.
**Single Step** and **Multiple Steps** require a concrete prediction belonging
to the selected mode and socket. The worker freshly validates the initial
prediction, clicks one target, waits the configured settle time (default 1000 ms
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
click; Multi-Step plans the next click from that fresh frame. Per-action
evidence logs include both pile measurements, the unique highlighted partner,
the qualifying pair's combined count and any removed partner.

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
sequence. Klondike does not enter that controller. After a verified board completion, the Solver header progress probe
selects redeal or game completion; the session board counter is advisory.
Before the first score-skip click, fresh frames check whether an actionable
new board or a terminal dialog has appeared. Two consecutive non-gameplay
frames are needed before a score-skip click; other transition frames receive
bounded input-free recaptures. A recovered old board does not count as a
completed game in the UI.
The controller waits three seconds after each score-panel click. If Level Up
is not yet visible, it makes an input-free follow-up capture before considering
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
