# App 2.7.118 / xSAR 0.7.103 live brief

**Host qualification passed; fresh live acceptance is pending.** App tests passed
1,218/0/0, shared tests 278/0/0 and the private unchanged-source console 80/0/0,
with zero filtered. Strict required checks and both release builds passed. Final
documentation/preservation audit passed: 12 documents, 309 local links and 40 review
paths with exercised inputs preserved. The implementation and actual results are
described in [Verification-2.7.118.md](Verification-2.7.118.md).

The qualified test build enables Continuous on supported normal three-board
Pyramid, including actual board-two/three entry. It retains guarded completion,
Solver-free NewGame and fresh acquisition after restart. Default Least moves uses
weighted A* weight two; the exact option uses weight one. Planning statistics and
the actual proof label must remain visible.

When no complete route is returned, the planner may offer a beneficial tableau
removal route. Exhaustive search may prove maximum removals and the shortest route
at that count; a bounded result says only best found. Execution must finish at the
exact replayed non-clear position, report removed/remaining cards and stop every
automatic mode. It must not advance, restart, retry the last gesture or claim the
challenge was won. Rank-specific challenge scoring remains separate work.

## Preserve evidence before operator launch

Preserve current session/error logs, exported deals and relevant images before
closing the old controller. A rebuild does not replace a running executable.
`--image` clears `/home/charlie/tmp/images` at startup. Use one controller and keep
the guest Solver disengaged; an unavailable/grey Solver is allowed. Keep the
supported native 1920×1080, 100% CLASSIC BLUE layout and capture selection.

The protected launch path is
`/home/charlie/tmp/pyramid-continuous-fallback-validation-Vs54oXto/release/qmp-qemu-socket-2.7.118`.
It is 13,046,672 bytes, mode 0555, SHA-256
`73ed229ea6a00a36c117a33778c40a888b83b76cd0a2930f7f2a9ca8cbbd8e90`,
identical to the native build; both help checks passed. The previous 117 release
is preserved. A dated runtime observation still saw protected 116, PID 108737,
at `2026-10-20T14:54:50.396629+00:00`; verify the actual running version when testing.

1. Launch the qualified protected build after evidence preservation and closing
   the previous controller. This command has not been executed by Codex.

```sh
cd /home/charlie/repo/RUST/qmp-qemu-socket && /home/charlie/tmp/pyramid-continuous-fallback-validation-Vs54oXto/release/qmp-qemu-socket-2.7.118 --image
```

## Observe the complete workflow

- Manual: choose the planning method before Recon; acquire afresh, create a new
  plan and solve once. A saved path/export alone provides no execution authority.
- Continuous: start on a supported untouched normal board one, two or three, or
  a supported end scene. The native ordinal must determine actual progress.
  After full clears, require the verified next ordinal; after the third clear,
  require supported GAME WIN/restart evidence, Solver-free NewGame and a fresh
  normal board-one witness before acquisition repeats.
- Full routes: verify visible weighted/exact proof and actual planning statistics;
  the weighted option must not claim an exact minimum. Final ACK alone must not
  authorise completion. Retained Perfect artwork must still require the native-v17
  final-only proof, exact piles and the same-frame normal-series guard.
- Partial routes: verify a positive removal count, matching remaining tableau,
  exact final position and explicit automatic stop. Stock-only removals must not
  inflate the tableau score. No board-clear or restart may follow a partial result,
  including on board three.
- STOP or a changed method/mode/context must revoke subsequent authority. Unknown
  scenes and uncertain input must stop without replay or cleanup gestures.
- Retain the first failure packet, exact deal and session log. A new screenshot is
  a later observation; preserve original evidence. Reacquire and replan after a
  failed or completed run; never resume an old route.

Record the running executable identity, mode/method, observed starting ordinal,
complete action outcome, next ordinal/restart outcome, partial proof if used and
any first failure. Host tests do not supply fresh Beast acceptance. M10 remains
Awaiting verification and M11 remains Planned until the required human/live
evidence is accepted in the [master schedule](Pyramid-Implementation-Schedule.md).

No installation, running-process replacement, guest input, commit, push or thread
transfer was performed by preparing this brief.
