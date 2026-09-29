# Changelog

## 1.2.1 — 2026-09-30 (candidate 1; Beast verification pending)

- Add the Solver-driven Klondike Draw 1 profile, dynamic source-block detection,
  three-position RIGHT fan, stock draw/recycle and SUIT-source priority.
- Add an independent bounded Klondike controller, configurable settle/recapture
  values, finite Multi-Step, and one immediate-capture Solver recovery per
  unresolved context. Unknown results never enter shared completion/restart.
- Preserve the established TriPeaks/Pyramid policies and exact-byte PNG saving.
- Add two blank lines before definitions and statement blocks throughout the
  existing and new Rust source without running a formatter.
- Keep nightly, rustfmt configuration and dependency versions unchanged.


## 1.0.1 — 2026-09-26 (candidate; Beast verification pending)

- Record all 28 Pyramid card envelopes and hit points, plus Left, Move/Recycle and Right, from Issue #1's 1920x1080 screenshots.
- Replace broad Pyramid preview envelopes with labelled target rectangles and hit-point markers (Issue #3). Draw overlays only on the main preview; preserve original saved PNG bytes. Keep Pyramid input disabled pending behaviour and detector validation.
- Name Solver, Undo All and Undo as shared toolbar controls and display them in both mode previews.
- Update architecture, socket-path and capture documentation for Issue #2; remove stale project-stage labels while preserving calibration provenance.
- Synchronise the package version in Cargo.toml and Cargo.lock without changing dependencies.

## 1.0.0 — 2026-09-26

The following behaviour was already present in the 1.0.0 base commit:


- Allow the Level Up screen three seconds to settle after the score panel click. If a fresh frame already shows New Game, require a second capture of that stage before acting on it.
- Capture a fresh QMP PNG when Capture PNG is clicked, preview that image, and save precisely those original bytes when Save PNG or Enter is used. Add field focus and a Cut/Copy/Paste context menu.
- Record the Challenge Complete Continue button geometry for future challenge support, without enabling automated input.
- Remove inherited application and artefact names; default to `qmp-qemu-socket.sock`. `QMP_SOCKET_PATH` supports an existing QEMU socket while the launch configuration is migrated.

- Establish `qmp-qemu-socket` with guarded TriPeaks actions and read-only Pyramid calibration.
- Use QMP to capture the VM display, verify each guarded action, and retain private session logs and original PNG evidence.
- Keep the application version in `Cargo.toml` as the source for the window title and Parameters display.
