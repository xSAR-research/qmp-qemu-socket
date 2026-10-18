# qmp-qemu-socket

Development **v2.2.100** permits the existing fourth, OK-only observation when the first full-size button appears on search three without current exact-black ink. Its fresh response still needs its own exact-black glyph and matching geometry; no fifth observation or input replay is added.

`2.2.100` is Charlie's approved numeric migration checkpoint, not a reconstruction of the candidate build count. xSAR independently migrates to `0.4.100`; the application continues to use its sibling path dependency. These versions remain provisional until their required tests pass. Subsequent ordinary changed revisions advance only the build component; milestone/minor and major transitions require Charlie's direction or approval, and the build counter never resets.

The Free Cell regression now distinguishes stale-glyph rejection from a legitimate late-body fourth observation. The latest recorded Spider sequence remains a known safe refusal: its third and fourth frames contain no exact-black glyph. This version preserves that limitation in a native regression; it does not change the colour threshold or add another observation.

The preceding **v2.2.2-candidate.14** reconnects before sending a capture request when the retained TCP peer has cleanly closed. It retains candidate.13's bounded persistent BGR8 transport and candidate.12's interrupted OK recovery. Pyramid exports explicit card labels, values 1..13 and suits through xSAR's schema-2 codec.
The preceding **v2.1.0** live Pyramid scan successfully restored the original board and exported its complete deal; the accepted five-game CV behaviour remains unchanged.
The local development set uses xSAR **0.4.100** from the sibling `../xsar` checkout and Windows screen-grab server **0.1.3**. The reconnect repair changes only the Arch client and remains compatible with version-3 servers 0.1.2 and 0.1.3. These unpromoted source versions require the reviewed source set; the previous published server does not implement version 3. See the [idle EOF repair and verification](docs/capture-idle-eof-candidate14.md) and [combined capture contract](docs/capture-combined-candidate13.md).

## Computer Vision solving

The implemented Computer Vision methodology follows the built-in **Solver** in Microsoft Solitaire & Casual Games. The Rust application captures a Windows 11 guest display through the guest TCP screen-grab server, recognises a supported source **HALO**, and sends the corresponding guarded mouse click or keyboard operation through QEMU's QMP Unix socket. An egui desktop interface displays the captured frame, selected game, execution controls and diagnostic output.

The guest supplies the move recommendation; the application supplies visual recognition, input delivery and game-specific transition handling. It does not independently choose moves by reading card identities. The five games provide repeatable environments for exercising capture, image processing and remote control while developing reusable primitives for route-planning experiments.

## Shortest Path route calculation

The second methodology is intended to build an independent state model and search for a route using **Dijkstra or A***, rather than follow the guest's Solver. The longer-term goal includes evaluating move costs, deciding when to defer an available move, and replanning when new information reveals an impasse. This work is intended to inform future drone route-planning experiments; no drone controller is implemented here.

Ordinary Pyramid preparation remains **read-only**: capture a board, inspect its calibrated regions and preview or save a PNG of that capture. A separate, explicitly confirmed **Guarded Pyramid reconnaissance → JSON** operation reads the original tableau and ordered DRAW cards, verifies Undo All restoration and publishes a validated complete deal through xSAR. It requires a native calibration profile and grants only bounded D, Undo All and specific confirmation-OK input. It does not enable Solver, ordinary gameplay, shortest-path search or calculated-route execution. See the [reconnaissance contract and acceptance boundary](docs/pyramid-reconnaissance.md); implementation and fixture tests are not live guest acceptance.

## Strategy selection and execution

Choose a solving strategy before any automatic QMP connection or capture begins. **Game Type** is a separate selection for Computer Vision solving, displayed in this order: **Klondike, Spider, Free Cell, Pyramid, TriPeaks**. The initial Computer Vision game remains TriPeaks. Shortest Path preparation currently selects Pyramid only.

Strategy, game and socket changes clear prior prediction and progress authority. Requests and results carry the selected strategy, game, socket and a selection generation, preventing a late worker result from restoring an earlier selection.

| Control | Behaviour |
| --- | --- |
| Capture Frame | Obtain a fresh read-only advisory image and prediction. |
| Single Step | Request at most one gameplay action from fresh supported evidence. Solver setup, where allowed, is separate from that gameplay budget. |
| Multiple Steps | Follow fresh supported targets up to a finite budget, or use **0 = continuous** until STOP or a guarded stop. |
| Params | Change the existing socket path and bounded, game-specific timings while idle. Timing edits apply to the next run. |
| STOP | Cancel further input and interrupt cancellable waits. An in-flight TCP or QMP operation may complete or time out before the worker returns. |
| Capture PNG | Capture, inspect, label and save one PNG without guest input or a second capture. |
| Start reconnaissance → JSON | Separately confirmed Pyramid acquisition with native recognition, bounded stock advances, verified restoration and no-overwrite output. STOP revokes restoration input too. |

A HALO is source evidence, not proof that a previous move succeeded. TriPeaks and Pyramid retain their own effect, removal and redeal checks. Klondike, Free Cell and Spider follow fresh supported Solver recommendations without comparing source and recipient card pixels; their logs distinguish acknowledged actions from proven effects. Missing HALOs, an exhausted stock pile and QMP acknowledgements alone never establish a win.

An explicit TriPeaks or Pyramid run may start from a current **No HALO** preview. The worker takes one fresh capture and up to three delayed observations. If unresolved, a positively recognised nonempty board may authorise one reserved Solver setup, followed by an immediate capture and up to three more delayed observations. Only a fresh canonical target authorises gameplay. A concrete preview still has to match its fresh initial target; ordinary capture and game selection remain read-only.

The general flow is:

```mermaid
flowchart TD
    A["Choose strategy"] -->|Preparation| B["Read-only Pyramid capture"]
    B --> C["Inspect regions and save PNG"]
    A -->|Computer Vision| D["Fresh capture and classification"]
    D --> E{"Supported evidence?"}
    E -->|Source| F["Guarded input and settle"]
    F --> D
    E -->|Completion| G["Game-specific terminal handling"]
    G --> D
    E -->|Unresolved| H["Bounded observation or stop"]
    H -->|Resolved| D
```

The diagram shows the controller's decision loop. Each game supplies its own recognition and completion rules; finite budgets, STOP and uncertain input can end the loop. Terminal restart is governed by the selected game's policy, rather than inferred from the absence of a source.

## QMP and display requirements

The application connects to an **existing local QMP Unix socket**. It does not launch QEMU or create its listener. The default filename is `qmp-qemu-socket.sock` under `XDG_RUNTIME_DIR`; use **Params** or `QMP_SOCKET_PATH` to select another existing absolute socket path. QMP remains required for health checks and input even when pixels arrive over TCP.

The current game calibration requires the primary guest display at **1920 × 1080**, with display and text scaling at **100%**. Frames are validated against that layout; detections do not rescale coordinates or guess a different board geometry. Before guest input, the worker checks QMP health, VM running state, a current absolute pointer, STOP and the selected game/strategy context. Uncertain input delivery stops without replaying the operation.

Automatic capture defaults to **TCP** at `127.0.0.1:9999`, selected by `QMP_CAPTURE_ADDR`. Use a numeric IP address and port, with brackets around IPv6 addresses; hostnames are rejected to keep name resolution outside the bounded request. The matching Windows server sends raw top-down BGR8 pixels; xSAR reconstructs the full RGBA8 frame with alpha 255 for detectors and previews. `QMP_CAPTURE_WIRE=v3-persistent` is the default. Explicit `v3` uses one connection per BGR8 frame; `v2` selects the preceding one-frame BGRA protocol for comparison or compatibility. There is no automatic protocol downgrade. Ordinary captures create no temporary PNG and do not encode PNG unless `--image` diagnostics are enabled. Existing QEMU forwarding and firewall configuration remain operator-managed.

Set `QMP_CAPTURE_BACKEND=qmp` explicitly to retain legacy QMP screendumps. That mode reserves a private temporary PNG beside the socket, reads and decodes its original bytes, and attempts cleanup on return; both QEMU and the application must access that directory. TCP failures stop the request without falling back to QMP. Capture configuration applies for the application session.

Decoded Computer Vision observations update the application's image control, including delayed HALO observations and post-game transitions. A bounded latest-frame slot replaces older pending images, so the UI shows the newest available observation rather than accumulating a frame queue. Display publication introduces no additional screen grab and does not grant input authority.

The sibling xSAR dependency supplies TCP/QMP transport, the card/deal model, strict schema-1/2 reader, schema-2 writer and no-overwrite filesystem publisher. Game calibration, capture/input authority, reconnaissance and restoration remain application-owned. Keep `qmp-qemu-socket` and `xsar` beside each other when building; no registry publication is implied.

## Captures and diagnostic records

**Capture PNG** obtains a fresh, read-only frame and encodes that same frame to PNG in TCP mode. Legacy QMP mode retains its original PNG. The dialog displays the capture while an optional label is entered. **Save PNG** or Enter writes the exact retained PNG bytes without another screen grab; **Recapture** replaces the pending capture, and **Cancel** discards it. Preview scaling and target overlays do not alter the saved image.

`QMP_SNAPSHOT_DIR` selects the save directory. Otherwise the application uses `$HOME/Pictures/Screenshots`, then `$HOME/Pictures`, when those directories exist, and finally the system temporary directory. Saved files use a local timestamp, a bounded label and exclusive mode-0600 creation.

Private session logs use `QMP_SESSION_LOG_DIR`, otherwise `$HOME/tmp` when it exists, otherwise the system temporary directory. The `qmp-qemu-socket-*.log` files retain the full session independently of the on-screen output. The visible buffer retains at most 2000 entries and rolls over after every three completed games. **Text size** changes the visible output labels only; copied and logged text are unchanged.

When a run stops on uncertain evidence, retain its log and an original PNG of the guest state before manually advancing it. A saved image establishes its own observed state; it should not be assumed to be byte-identical to a different capture named in an earlier log.

## Build and verification

The commands below use the sibling `../xsar` dependency and the committed
lockfile. Update that sibling checkout before building. Earlier external source
overrides and Git pins in release records describe historical validation. See [development conventions](docs/development.md) and the
[v2.2.0 validation record](docs/release-v2.2.0.md). The
[v2.0.11 notes](docs/release-v2.0.11.md) retain historical RGB-slice evidence.

Build from the repository root on the Linux development host. `rust-toolchain.toml` selects floating **nightly** with Clippy and Rustfmt components; `Cargo.toml` declares edition 2024 and minimum Rust **1.101.0**. The channel is not a dated compiler pin, so record the actual compiler and Cargo versions used. Stable compatibility is not established by the toolchain file.

1. Validate the source and documentation.

```zsh
rustc --version &&
cargo --version &&
cargo check --locked &&
cargo test --locked -- --include-ignored &&
RUSTDOCFLAGS='-D warnings' cargo doc --locked --no-deps --document-private-items &&
cargo clippy --locked --all-targets -- -D warnings
```

Capture regressions cover raw TCP pixels, retained snapshot PNGs, GAME WIN
recording, explicit legacy capture and rejection of TCP failures without fallback.
The integration tests require local Unix and loopback TCP socket creation. `--include-ignored`
runs that explicit host-test group as well as the ordinary suite; restricted
executors can run the ordinary suite but must report these tests as unverified.

2. Build the release executable in the repository's target directory.

```zsh
CARGO_TARGET_DIR="$PWD/target" cargo build --locked --release
```

3. Launch the validated release.

```zsh
QMP_CAPTURE_ADDR=127.0.0.1:9999 cargo run --locked --release
```

To select the preserved legacy path explicitly, use
`QMP_CAPTURE_BACKEND=qmp cargo run --locked --release`. Leaving
`QMP_CAPTURE_BACKEND` unset selects TCP; `QMP_CAPTURE_BACKEND=tcp` is equivalent.

Build and test results do not establish live guest behaviour. Verify the selected game, HALO handling, transitions, image updates and STOP against the intended guest configuration. Formatting is separate from these checks; follow the repository's documented source-spacing policy.

## Design and historical evidence

The [Pyramid reconnaissance contract](docs/pyramid-reconnaissance.md) and
[native profile contract](docs/pyramid-recon-profile.md) describe the new input
boundary, recognition evidence, limits and remaining live acceptance.
The [v2.2.0 validation report](docs/release-v2.2.0.md) records current schema-2
checks, the converted accepted deal and the crate-first promotion boundary.
The [v2.1.0 validation report](docs/release-v2.1.0.md) retains historical checks
and executable/profile identities for the preceding acquisition implementation.

The [v2.0.11 development notes](docs/release-v2.0.11.md) describe shared RGB
measurement, paired and Git-pinned validation, and the application promotion boundary.

The [v2.0.10 release notes](docs/release-v2.0.10.md) describe the bounded TriPeaks terminal acquisition allowance and verification. Supporting material includes [architecture](docs/architecture.md), [QMP and capture design](docs/qmp-and-capture.md), [Pyramid execution](docs/pyramid-execution.md) and [development conventions](docs/development.md).

Earlier implementation decisions and fixture limitations remain in the [development records](docs/), including the [strategy split](docs/strategy-v2.0.0-candidate-2.md), [CLASSIC baseline and Solver startup](docs/classic-v2.0.0-candidate-7.md), [Free Cell OK layouts](docs/freecell-v2.0.0-candidate-4.md) and [capture preview publication](docs/capture-preview-v2.0.0-candidate-5.md). Those dated records retain their original scope; earlier Klondike card-effect experiments do not describe its current Solver-led execution policy.

## Klondike

Klondike uses the calibrated **Draw 1** layout. Selection priority is **DRAW/RECYCLE → RIGHT HALO → RIGHT Solve → lowest tableau source → SUIT piles**. Draw sends the confirmed `d` shortcut; recycle clicks the recognised stock control. The changing waste fan and source-run geometry are measured from each fresh frame. A supported solid source receives one click, including a run or a source from a SUIT pile; dashed destinations are not clicked. The separate **Solve** control can authorise automatic finishing when its own lettering, interior, empty-stock and scene checks pass. The lower-toolbar **Solver** remains the recommendation control.

Each acknowledged gameplay operation consumes one budget slot, followed by editable settle and a fresh capture. Ordinary missing-HALO recovery is bounded and permits at most one Solver refresh per unresolved context; unknown scenes or uncertain delivery stop. Independent positive completion evidence is required before terminal handling. Finite runs stop at the confirmed win; continuous mode follows the supported score, optional Level Up, New Game, Draw 1 Play and fresh-board stages. CLASSIC blue backs and previously measured red backs remain supported; colour alone cannot authorise input.

## Spider

Spider scans ten **PLAY** columns, the lower-right **DRAW PILE** and the display-only **COLLAPSED SUITS** area. A highlighted Draw has priority and sends `d` to deal one card to each column. Otherwise the controller selects the lowest supported solid PLAY source, with left-to-right ties, and clicks its visible bottom-card area. Connected overlapping outlines form one source; dashed destinations and collapsed suits are never clicked. Source reads and clicks remain above the calibrated toolbar cutoff. After a released PLAY click, a movement-only pointer park at `(20, 500)` clears the source area before settling and recapturing; Draw does not park.

Stock recognition retains measured red support and uses the shared CLASSIC-blue predicate, alongside the existing closed-outline, occupancy, scene and active-Solver guards. Live play across all five game types has been reported successful. Automated CLASSIC-blue Draw fixture coverage remains synthetic; an original native blue Draw fixture has not yet been supplied. The controller may activate an inactive Solver once on a supported board, but does not refresh an active Solver. Missing HALOs and an exhausted stock do not prove completion. One board is one game: finite runs stop at recognised win entry, while continuous mode follows score skip, optional OK, New Game, Spider Play and fresh-board Solver activation through bounded observations.

## Free Cell

Free Cell checks **CELL 1–4** before the lowest supported source in **PLAY 1–8**, with left-to-right ties. One solid highlighted source block receives one bottom-card click, followed by editable settling and a fresh observation. Repeated source positions are valid, and a run extending behind the toolbar is accepted only from its visible source geometry and card paper. Dashed destination guides are not sources. The **SUIT 1–4** piles are display-only for input selection; there is no Draw, Recycle or Solve operation, and no card-back recognition is needed for its face-up layout.

Automatic suit transfers can temporarily remove the HALO. An inactive Solver receives a game-start wait and fresh capture before activation; an active Solver receives one delayed input-free observation before a possible refresh. Activation and refresh share one reserve per unresolved context, and further observations remain bounded. Unknown scenes and uncertain delivery stop. One board is one game. Independent win entry precedes the score-skip, optional Level Up OK, New Game and Play sequence, which restarts only in continuous mode. Local OK recognition includes the original layout and the evidenced higher and lower positions; rank, medal and surrounding artwork do not supply button authority.

## Pyramid

Computer Vision Pyramid uses 31 calibrated targets: **Move/Recycle, Left, Right**, then the 28 tableau cards from the bottom row upward and left to right. Multiple highlights are expected; the first eligible source supplies one operation, allowing the guest Solver to remove a pair or a King. A highlighted Move/Recycle uses `d`, while pile and tableau sources use one click. Unlike the three source-only controllers, Pyramid retains positive effect and card-removal checks. Only a clicked card whose removal is verified becomes consumed; lower controls remain repeatable. A narrowly supported repeated Left/Right pair may continue from fresh HALOs without claiming the preceding effect or advancing completion.

No-HALO startup may reserve one Solver setup only after positive nonempty-board evidence. Ordinary post-action and redeal observations retain their separate bounded policies, and unknown occupancy cannot invent an available card or completed board. Confirmed redeals reset consumed-card state; the shared progress and post-game controller handles the calibrated three-board game flow. Pyramid is also the sole current **Shortest Path preparation** fixture: that strategy captures and saves without gameplay, stock surveying, Undo All input or route calculation.

## TriPeaks

TriPeaks combines calibrated lower-panel controls with tableau scans that follow exposed rows and widen when fresh card evidence requires it. A supported stock recommendation sends `d`; a supported tableau recommendation clicks its fixed card location. The controller retains cursor-excluded action-effect checks and uses fresh accepted result frames to plan subsequent operations. Explicit no-HALO startup and late-HALO recovery use bounded observation and positively gated Solver setup; neither a missing source nor an acknowledgement establishes removal or completion.

TriPeaks uses the shared calibrated three-board progress, redeal and post-game flow. The final-win transition has its own narrow recognition path: only a qualifying delivered top-row card action, the existing effect threshold, no fresh actionable target or occupied row evidence, and two consecutive frames of the same supported terminal type can establish that completion. The supported types are the Congratulations reward panel and the separately recognised Level Up panel covering that reward. Qualified final-card acquisition permits up to 30 total post-action captures; a recognised candidate receives the separate four-capture confirmation allowance. A confirmed Level Up entry goes directly to its freshly recognised OK control without a preceding centre skip. If OK reveals the reward panel, two fresh positive observations authorise its bounded skip before New Game. The guarded terminal sequence and budgets remain bounded; changing level, XP and central rank artwork are not template conditions. Unrecognised transitions stop for inspection rather than being treated as a win.
