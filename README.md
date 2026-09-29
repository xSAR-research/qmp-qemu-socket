# qmp-qemu-socket

A Rust desktop application for inspecting and controlling a QEMU VM through its
QMP Unix socket. The current guest is Windows 11 running Microsoft Solitaire &
Casual Games. TriPeaks and Pyramid actions are guarded by fresh captures and
visual verification. Both use the same QMP controller and post-game flow; each
game provides its own target selection and effect checks.

The application version is `1.1.0`. `Cargo.toml` supplies the version shown in
the window title and Parameters.

## Build

The project requests Rust 1.100.0 in `rust-toolchain.toml`. This is the **nightly**
channel. Install it with `rustup default nightly`. It is required simply for
**rustfmt** to use the `unstable_features` flag set in `rustfmt.toml`. `clippy`
and `cargo test` work fine on stable.

```text
cargo test --locked
cargo build --locked --release
```

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
This build identifies itself as **v1.2.0**.

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

Both profiles use the same visual progress probe and guarded post-game
sequence. After a verified board completion, the Solver header progress probe
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
