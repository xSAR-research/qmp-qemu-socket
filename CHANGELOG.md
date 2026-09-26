# Changelog

## Unreleased — local review candidate

- Allow the Level Up screen three seconds to settle after the score panel click. If a fresh frame already shows New Game, require a second capture of that stage before acting on it.
- Capture a fresh QMP PNG when Capture PNG is clicked, preview that image, and save precisely those original bytes when Save PNG or Enter is used. Add field focus and a Cut/Copy/Paste context menu.
- Record the Challenge Complete Continue button geometry for future challenge support, without enabling automated input.
- Remove inherited application and artefact names; default to `qmp-qemu-socket.sock`. `QMP_SOCKET_PATH` supports an existing QEMU socket while the launch configuration is migrated.

## 1.0.0 — 2026-09-26

- Establish `qmp-qemu-socket` with guarded TriPeaks actions and read-only Pyramid calibration.
- Use QMP to capture the VM display, verify each guarded action, and retain private session logs and original PNG evidence.
- Keep the application version in `Cargo.toml` as the source for the window title and Parameters display.
