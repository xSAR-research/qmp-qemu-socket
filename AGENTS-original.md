# qmp-qemu-socket — Codex instructions

## Authority and source

- Charlie is the sole developer and operator. Follow his current request and
  approved scope; ask only about a blocking ambiguity or disruptive design choice.
- The repository is `https://github.com/xSAR-research/qmp-qemu-socket`, branch
  `main`. Charlie's pushed main is the source of truth for each new patch cycle.
  Do not use the former solitaire-solver repository or its release numbering.
- At the start of a new cycle, obtain one fresh local checkout of main, record
  its exact commit and verify Charlie's announced package version. Do not repeat
  fetches or remote-currentness checks within that cycle.
- For corrections to an unpushed candidate, retain the same base and incorporate
  Charlie's reported local changes. Do not replace it with older main or silently
  carry it into a later cycle after Charlie has pushed a new baseline.
- Handoff reference, verified 29 September 2026: package `1.2.0`, commit
  `db506d11b46ce7180cd3be7fe05a66cfa056999e`. This records provenance; update it
  when a later cycle establishes a new base. A handoff that already fetched this
  cycle's base does not require another fetch merely because the chat changed.
- Inspect local Git status and relevant source before editing. Preserve unrelated
  edits and calibration files. Read-only local Git inspection needs no additional
  approval. Do not reset, discard, stash, commit, push or publish on Charlie's
  behalf unless authorised; Charlie normally commits and pushes tested code.
- Read selected GitHub Issue bodies and relevant comments as the request record.
  Draft updates freely; create, post, edit or close public Issues only when
  Charlie explicitly asks. Existing authorisation persists within its scope.
- The Beast's repository-root `AGENTS.md` is local and untracked. Preserve that
  status; do not `git add` it or change ignore rules without Charlie's request.
  It is absent from GitHub, so a fresh checkout receives these instructions from
  the handoff attachment. Keep its content separate from the committed code base.

## Gate workflow and deliverables

1. **Gate 1 — Intake and implementation:** establish the base, read relevant
   source and evidence, and implement the bounded mini-goal. Routine reversible
   implementation and checks are included in the request; do not add approval
   stops for each file or command.
2. **Gate 2 — Candidate delivery:** provide a downloadable tarball containing
   complete affected source/content files and a complete guarded extraction and
   installation script. State base, changes, actual tests, untested limits and
   concrete Beast verification steps. Stop for Charlie's verification.
3. **Gate 3 — Verification and correction:** diagnose Charlie's returned logs,
   screenshots or corrections, and deliver a complete revised candidate against
   the same base. Preserve his reported local changes and repeat until accepted.
4. **Gate 4 — Promotion:** Charlie commits and pushes the accepted candidate.
   The next change request starts a fresh cycle from that pushed main.

- Never substitute fragmented line edits for a complete deliverable.
- Use separate, numbered fenced command blocks with `echo` or `print` announcing
  the step. Keep one purpose per block; stop on failure. Do not enable shell exit
  options in Charlie's interactive shell or use a failed `cd` that allows later
  commands to run in the wrong directory.
- Installers must validate the exact base and recognised file contents, preserve
  unrelated work, support preview, retain backups/evidence and offer recoverable
  rollback. Refuse unknown affected-file changes before modifying files. Explicitly
  enumerate any deletion; never remove files merely because an archive lacks them.
- Rehearse installer success, idempotence, refusal and rollback in isolated local
  copies. Syntax checks alone do not establish runtime correctness. Avoid zsh
  special parameters such as `status` and `path` for script variables.
- Separate checks performed here from checks Charlie performed on the Beast.
  Never claim a download, checksum or build establishes live installation or
  successful gameplay. End substantial implementation reports with a dedicated
  `## Code Analysis` covering threat surfaces, memory safety, bounds and failure
  modes. Keep persona banter out of code, documents and public Issue updates.

## Host, paths and capture

- Beast repository: `/home/charlie/repo/RUST/qmp-qemu-socket/`.
- Downloads, installation evidence and requested diagnostic output:
  `/home/charlie/tmp/`. Session logging can use `QMP_SESSION_LOG_DIR` for this path.
- Charlie's host screenshots: `/home/charlie/Pictures/Screenshots/`; he attaches
  relevant files to chat. Preserve original evidence bytes; use clearly named
  copies and a name mapping when preparing evidence bundles.
- Host: Arch Linux, KDE Wayland. Guest: Windows 11 under QEMU, Microsoft Solitaire
  & Casual Games. Calibrated primary guest frame: 1920×1080, display/text scale 100%.
- QMP socket: `/run/user/1000/qmp-qemu-socket.sock`; `QMP_SOCKET_PATH` can select
  an explicitly requested existing listener. The app does not launch QEMU or
  create its socket. QEMU and the app must share access to capture paths.
- Automatic capture currently reserves a temporary PNG beside the socket, asks
  QEMU to write it, reads the bytes, decodes RGBA8, then attempts removal on normal
  cleanup. RGBA8 has 8 bits per channel, 4 bytes per pixel. It is neither file-free
  nor native-framebuffer capture. File-free capture is a separate open requirement
  requiring implementation and measurement.
- Manual **Capture PNG** retains one fresh original PNG and its decoded frame
  for preview. **Save PNG** saves those exact bytes without another capture or
  preview resampling. Preserve that identity contract.

## Architecture and input contract

- `GameMode` / `GameProfile` and typed actions in `src/game.rs` form the small
  game boundary. Share QMP, capture, cancellation, logging and reusable transition
  machinery; keep game-specific target/effect policy explicit and local.
- The worker owns QMP and capture I/O. `worker/post_game.rs` holds shared terminal
  stages; `worker/pyramid_execution.rs` holds Pyramid result/redeal handling.
  Keep egui responsive. Never add host-wide pointer or keyboard injection.
- UI predictions and overlays are advisory. Initial gameplay input requires a
  fresh frame reproducing a valid selected-mode/socket prediction. Before each
  input, check STOP and freshly probe VM run state/current absolute pointer.
- Require the QEMU HID Tablet as the current absolute pointer. Native guest
  coordinates satisfy `0 <= x < width`, `0 <= y < height`; never use host desktop
  or scaled preview coordinates directly. Preserve the checked integer transform
  `pixel * 32767 / extent` using widened arithmetic, without rounding, clamping
  or substituting `extent - 1`.
- Send one validated typed operation, settle, capture and evaluate the resulting
  state under that game's policy. Reuse an accepted result as the next planning
  frame: normally one initial capture plus one result capture per action, with
  additional observations only for recovery/transition policy.
- Preserve separate acknowledged pointer movement, down and up commands. QMP
  acknowledgement does not prove game effect. An uncertain non-idempotent input
  must never be replayed; existing up-only release recovery may complete.
- STOP is cooperative, including intentional waits; it cannot instantly interrupt
  an in-flight socket request. Preserve socket timeouts and bounded recovery where
  defined. Do not claim all existing observation loops have finite retry counts.
- Unknown scene, malformed layout or out-of-bounds target must not authorise
  input. No HALO alone does not authorise Draw, Solver activation or completion.
  Preserve settled recapture policy before classifying an unresolved result.
- Keep the latest-frame preview mailbox, bounded visible output and complete
  private session log. Preserve the distinction between effect verification,
  authorised fresh-HALO continuation and verified board/game completion.

## Existing game behaviour and Klondike scope

- At the v1.2.0 baseline, TriPeaks and Pyramid both support guarded Single Step,
  Multiple Steps and their established shared post-game sequence. Preserve them.
- TriPeaks requires its calibrated unique target; its Draw operation uses qcode
  `d` without pointer movement. Do not assume that shortcut works in other games.
- Pyramid priority is Move, Left, Right, then 28 tableau cards scanned row 7
  left-to-right upward to the apex. A highlighted pair needs one click, as does
  a highlighted King. Multiple simultaneous card HALOs are expected.
- Preserve Pyramid's editable Move/card/reobserve delays, consumed-card state and
  verified redeal recovery. Ordinary missing HALOs must not trigger extra Solver
  clicks. Completion needs the existing positive scene/board evidence; progress
  readings and advisory counters alone do not establish a win.
- Preserve the qualified repeated Left/Right fresh-HALO path: after settling and
  recapture, a valid fresh pair can authorise the next operation even when card
  faces look identical. This is logged separately; previous effect remains
  unproven and no consumed-card/completion counter advances. Do not restore an
  unconditional changed-pixel requirement or generalise this exception to other
  targets or modes without evidence.
- Klondike is the next Solver-driven extension and is not implemented at this
  baseline. Use the current handoff, original PNG evidence and Charlie's action
  notes to establish its target selection and input behaviour. Do not infer
  stock/recycle or win rules from another mode. Charlie confirms that one click
  anywhere inside the highlighted Klondike source block moves the whole run.
  Do not drag or click its dotted destination marker; group the outlined source
  cards as one action. K10/K11 in the handoff evidence show this transfer.
- New Klondike geometry and completion detection require their own evidence.
  Reuse shared mechanisms only after their preconditions are met; do not copy the
  three-board completion model merely because the toolbar is similar.
- Scope is following the guest's Solver recommendations. Independent card-rank
  recognition, Dijkstra/A* search and a universal solitaire framework are future
  work unless Charlie expressly includes them in the mini-goal.

## Rust, formatting and validation

- Preserve `rust-toolchain.toml`: channel `nightly`, components `clippy` and
  `rustfmt`, minimal profile. Preserve `rustfmt.toml`: `unstable_features = true`,
  style edition 2024, blank-line lower/upper bounds 0/2. Charlie intentionally
  chose nightly for these formatting options; do not switch to stable or change
  his global toolchain default. The baseline Cargo minimum is `1.100.0`.
- `nightly` is floating, not date-pinned. Record the actual `rustc --version` and
  `cargo --version` used for validation; do not claim an exact compiler version
  from the channel label. Ask before making a disruptive toolchain change.
- `Cargo.toml` is the version authority; keep its local `Cargo.lock` package entry
  aligned. Labels derive from `CARGO_PKG_VERSION`. Follow Charlie's agreed release
  numbering and identify candidate revisions distinctly when needed.
- Use idiomatic Rust naming, qualified module names for name collisions, meaningful
  `///` item documentation and `//!` module contracts. Document fields/variants,
  test helpers, bounds, failure conditions and non-obvious authority decisions.
  Ordinary local implementation comments use `//`. Do not suppress warnings
  broadly or reintroduce dormant capture formats without a real backend.
- Read `docs/development.md` and relevant module documentation. Prefer manifest
  and source facts over stale prose: the v1.2.0 README opening and development
  heading still mention 1.1.0. Correct such prose only within authorised scope.
- Run relevant checks from the repo root, each separately, stopping to diagnose
  failure: `cargo fmt --all -- --check`; `cargo check --locked`;
  `cargo test --locked`; `cargo doc --locked --no-deps --document-private-items`;
  `cargo clippy --locked --all-targets`; `cargo build --release --locked`.
  For documentation-only work, proportionate document/link checks suffice.
- Report exact successes, warnings and failures. If an environment prevents a
  test/build, retain its diagnostic and report the limitation; a filtered rerun
  cannot be reported as a full-suite pass. Do not silently relax checks.
- Live acceptance runs on the Beast. Verify source version, release executable
  identity and launched candidate; replacing a file does not update a running
  process. Exercise the changed behaviour and relevant existing-game regressions,
  including STOP/transitions where affected. Preserve first-failure evidence.
