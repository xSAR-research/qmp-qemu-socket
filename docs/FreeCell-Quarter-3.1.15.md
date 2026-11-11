# Free Cell quarter-card diagnostic — 3.1.15

The approved experiment changes only ABS Single Step. It implements the latest destination/up/left/destination request, superseding the earlier axis-separated source-lift proposal. Application 3.1.15 uses the additive xSAR 0.16.0 bounded four-point method. All 51 fresh paired host gates passed: 1,891 application, 365 library and 80 console tests, zero failed, ignored, measured or filtered. The diagnostic hypothesis still needs operator Windows acceptance.

## Native path and timing

The application positions the freshly proved source, presses LEFT once, keeps it held through four ABS updates, and releases once:

| Input | Native position or button |
| --- | --- |
| 1 | Proved source ABS; matching source ACK required |
| 2 | LEFT down |
| 3 | Exact effective destination `(x,y)` |
| 4 | Quarter-card up `(x,y-45)` |
| 5 | Quarter-card left from raised point `(x-34,y-45)` |
| 6 | Exact effective destination `(x,y)` again |
| 7 | LEFT up |

The native quarter offsets are integer floors of the existing Free Cell slot extent: width 137 / 4 = 34 and height 182 / 4 = 45. They are diagnostic displacements, not measurements of a continuing Windows-held gesture. PLAY targets retain their observed exposed-corner point. CELL and SUIT targets use their canonical slot centre `(slot_x+68,203)`, keeping all four held positions inside the same137×182 slot. Source coordinates remain the observed printed-corner coordinates. All native points and mappings are validated before input, without clamping or fallback.

Both existing buttons remain: six minimum2-second gaps (at least12seconds source-to-UP) or six minimum500-millisecond gaps (at least3seconds). Routing probes, sourceACK, socket work and synchronous logging can add time. Fast remains ABS-only. No separate click follows.

## Ownership and authority

The application computes game-specific points. xSAR's new `diagnostic_drag_path_with_timing` accepts exactly four held native points, validates/maps all of them before authority callbacks or operation I/O, refuses mapped source/final aliases, and permits intermediate aliases. It owns protocol pacing and existing captured-pointer release-only recovery. Existing `diagnostic_drag` and `diagnostic_drag_with_timing` keep their prior four-pixel-approach contracts; normal drag, REL, configuration, observer, dependencies and feature defaults are unchanged.

Source ACK gates DOWN. Fresh selected-pointer verification, STOP/context/plan/source guards remain. All six held-phase replies are collected only after UP. Post-DOWN uncertainty preserves the original cause and invokes existing targeted safetyUP; DOWN/motion are never replayed. Original pointer restoration requires confirmed UP and continuing authority. STOP may leave the diagnostic pointer selected. QMP cannot pin routing across the held interval.

Original52-card binding, relevant printed source/target/AUTO proofs, cached manualUNDOALL restart and two consecutive fresh final recorded endpoints within six observations remain unchanged. ACK is delivery evidence only. No scoring, targetOCR/search, Continue/NewGame or multiboard policy is introduced.

## Evidence limits

Saved native fixtures and local TCP/QMP peers qualify code paths, coordinates, timings and retirement. They do not prove Windows cursor/button state, game message handling, drag initiation or a successful drop. The hypothesis that an early held update is ignored remains unmeasured. Fresh Windows gameplay acceptance is for the operator after launching the qualified build. No agent guest input/capture, process replacement, installation or promotion is authorised by this contract.
