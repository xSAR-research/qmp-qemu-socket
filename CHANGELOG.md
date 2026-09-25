# Changes

## 1.0.0 qmp-qemu-socket

### 1.0.0 - Project transition from Solitaire Solver to qmp-qemu-socket 2026-09-26

#### **New**

- Start qmp-qemu-socket project from Solitaire Solver project to focus on image processing and Djistra/A* algorthym solving.

#### **Fixed**

- Fixed an issue where constants were not in UPPER_CASE_SNAKE_SNAKE format.

#### **Changed**

- Replace application name from Solitaire Solver to qmp-qemu-socket.

## 2.0.0-rc.3 — snapshot candidate pending local validation

- Add a read-only Save snapshot button after Draw Targets, available on
  TriPeaks and Pyramid even if a dialog has no recognised Solver highlight.
- Prompt for an optional bounded identifying label. Save the original QMP PNG
  once under the configured screenshots directory with local date/time and a
  bounded collision suffix. Reject oversized or invalid guest captures.
- Preserve guarded TriPeaks execution and Pyramid's independent input gate.

## 2.0.0-rc.2 — candidate pending local validation

- Allow the preview holder to show the complete 1920x1080 guest frame when
  the host window has room, with space reserved for the log and EXIT control.
- Preserve one guest image pixel per host physical pixel and retain panning
  when the host window is smaller than the full image.
- Refresh the read-only frame when Game Type or Draw Targets changes;
  invalidate the earlier preview before any further guest input.
- Pyramid remains read-only calibration; guest-input authority is unchanged.

## 2.0.0-rc.1 — candidate pending local validation

- Single version source in `Cargo.toml`; title and Parameters read that value.
- Automatic one-shot read-only startup capture; mode/socket changes revoke an
  earlier preview. The worker stamps retained frames with their capture context.
- Native physical-pixel preview with nearest-neighbour rendering, two-axis
  scrolling, full-span Track guide and click-only coordinate reports.
- Two-row colour-coded controls; dependent actions appear after a valid capture;
  run-time controls remain visible but disabled during an active run.
- Draw Targets includes read-only TriPeaks Solver, Undo All and Undo outlines.
- The worker publishes board-counter changes at verified completion; `?/3`
  represents an unknown board when the only evidence is a new session counter.
- Pyramid remains selectable for read-only calibration with an independent
  worker-side prohibition on guest input.

Still required for final 2.0.0: original 1920x1080 settled QMP images covering
each Solver progress-bar third, local Cargo validation/release build and
Charlie's TriPeaks regression sign-off.
