# qmp-qemu-socket

A Rust desktop application for inspecting and controlling a QEMU VM through its
QMP Unix socket. The current guest is Windows 11 running Microsoft Solitaire &
Casual Games. TriPeaks and Pyramid actions are guarded by fresh captures and
visual verification. Both use the same QMP controller and post-game flow; each
game provides its own target selection and effect checks.

The application version is `1.0.2`. `Cargo.toml` supplies the version shown in
the window title and Parameters.

## Build

The project requests Rust 1.98.1 in `rust-toolchain.toml`.

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
prediction, clicks one target, waits 500 ms, and verifies the result. Each
verified result becomes the next planning frame without another pre-click
screenshot. **STOP** cancels the active run.

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

Automatic capture currently uses a temporary PNG beside the socket: QEMU writes
the file, the worker reads it and decodes RGBA pixels, and normal cleanup tries
to remove the file. Detection then operates on that decoded frame in memory.
This is not file-free or native-framebuffer capture. File-free automatic
capture remains a separate requirement that needs implementation and measurement.

## Capturing an original PNG

Click **Capture PNG** to request one fresh read-only QMP screenshot. The dialog
displays that capture while you enter an optional label. The label field has
keyboard focus; Enter or **Save PNG** writes the exact captured PNG bytes without
another screendump. Right-click the field for Cut, Copy and Paste. **Recapture**
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
The controller waits three seconds after each score-panel click, including bounded
retries, before looking for Level Up OK. The general inter-stage wait remains
one second. When a fresh frame shows New Game already visible instead of Level
Up, a second fresh frame must confirm New Game before the guarded sequence
continues. Other ambiguous frames remain subject to bounded retry and STOP.

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
