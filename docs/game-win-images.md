# GAME WIN image diagnostics

Start the built controller with:

```zsh
./target/release/qmp-qemu-socket --image
```

Every actual application start removes `/home/charlie/tmp/images`. With `--image`,
the application recreates this private directory (0700). Without the flag, it
leaves the directory absent and records no diagnostic PNGs. `--help` and invalid
arguments do not start the application or reset the directory. An unexpected
symlink or non-directory at the exact image path is refused before opening the UI.

## Recording boundary

Recording begins only when independently confirmed completion hands control to
the GAME WIN sequence. It saves the original PNG that supplied the entry evidence,
then every new capture through score counting, score skip, optional Level Up OK,
New Game, Play, Solver activation and fresh next-game readiness. Readiness uses
each game's existing actionable Solver/HALO evidence. The final verification
capture is included; subsequent ordinary card-play captures are not written.

The boundary applies to TriPeaks, Pyramid, Klondike, Free Cell and Spider. A finite
run which confirms a win and stops instead of restarting retains its entry PNG.
A successful return, recovered gameplay, STOP or error closes recording. All
saved files remain available until the next application start. A later GAME WIN
sequence appends to the same directory with new serial numbers.

With recording enabled, the worker holds one latest original PNG in memory while
playing, so the entry evidence can be retained without an extra screen grab.
Ordinary captures do not create files in this directory. Disabled recording does
not keep this additional PNG cache. Preview coalescing cannot drop recorded images.

## Capture and log correlation

Example filename: `000113.310_000001.png`.

| Part | Meaning |
| --- | --- |
| `000113.310` | Seconds and milliseconds since application startup, at QMP screendump completion |
| `000001` | Unique recording serial across the whole application session |
| `.png` | Original full-display QMP PNG bytes, saved with mode 0600 |

Each saved image has a `GAME WIN PNG` text-log entry containing its exact path,
capture elapsed time, serial and measured write duration. The entry image may
predate recording activation; its filename preserves the original acquisition
time. Every new capture also logs the QMP request/completion interval, screendump
duration and file-read duration. QMP acquires the displayed pixels within that
request/completion interval; the filename does not claim a guest-render timestamp.

Worker log prefixes use that same startup clock and are stamped when the worker
emits each message. Delayed UI polling no longer replaces those timestamps with
the later time the UI drains its event queue. UI-generated housekeeping messages
still use their own emission time. The PNG capture time is truncated to milliseconds
in both its filename and its `capture=+...s` field.

Recording copies existing PNG bytes synchronously before recognition or further
guest input. It does not re-encode frames or request extra screendumps. Disk writes
can add elapsed time; the measured `write` field makes this overhead visible.
If recording fails, the transition stops with a clear error and keeps earlier
evidence. No image count limit silently discards captures.

Candidate.8 keeps the 1000 ms first post-score wait and three total control
observations. A pending OK uses that same budget to confirm settled geometry
and a current exact-black central glyph before its single click. The fixed
caption remains diagnostic only, logged with `caption_gate=false`; a fading
caption cannot veto otherwise sufficient button evidence. Each readiness log
belongs to the captured frame preceding it; no additional capture is requested
by the visual check. See [Level Up readiness](level-up-readiness.md).

Copy the image directory and corresponding session log before the next launch,
because either a launch with `--image` or one without it removes the old images.
