# qmp-qemu-socket

A Rust desktop application for inspecting and controlling a QEMU VM through its
QMP Unix socket. The current guest is Windows 11 running Microsoft Solitaire &
Casual Games. TriPeaks actions are guarded by fresh captures and visual
verification; Pyramid is available for read-only calibration.

The application version is `1.0.1`. `Cargo.toml` supplies the version shown in
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

Use the actual socket's absolute path in this application. If QEMU uses the
relative listener name `qmp-qemu-socket.sock` and starts in the Beast repository's
`target/release` directory, the corresponding application setting is
`/home/charlie/repo/RUST/qmp-qemu-socket/target/release/qmp-qemu-socket.sock`.
If QEMU instead uses an absolute runtime path, its working directory does not
change that location. The application connects to the socket QEMU creates; it
does not launch QEMU, create its listener or rename an existing listener.

QMP captures and guarded actions use the 1920×1080 primary display calibration.
No gameplay input is authorised for Pyramid.

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

TriPeaks waits three seconds after each score-panel click, including bounded
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
input contracts, and `docs/pyramid-calibration.md` for the Pyramid preview
geometry and its evidence limits.
