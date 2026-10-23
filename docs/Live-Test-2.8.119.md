# Live test — app 2.8.119 / xSAR 0.8.104

The required host checks passed: app **1260**, xSAR **289**, private console **80**; zero failed, ignored or filtered. This brief requests fresh operator
evidence; host fixtures and prior reports do not establish live acceptance.
The complete goal remains open. See [verification](Verification-2.8.119.md).

Preserve the current controller's log/diagnostics and any wanted GAME WIN images,
then close it normally. Use one controller. The --image startup option clears
/home/charlie/tmp/images, so retain wanted images first. A read-only process
sample at 2026-10-21T05:21:45.302209+00:00 is recorded in protected-artifact.json; do not assume
that dated sample still identifies the current process.

1. Verify the protected executable. Expected SHA-256: 72637b8248ae83ca43617a982c688898c7c92cd2086e5432eedcd77b29471df5.

~~~sh
cd /home/charlie/repo/RUST/qmp-qemu-socket &&
sha256sum -- /home/charlie/Documents/Pyramid/Validation/continuous-challenges-20261021/release/qmp-qemu-socket-2.8.119
~~~

2. After preserving evidence and closing the old controller, launch this exact build.

~~~sh
cd /home/charlie/repo/RUST/qmp-qemu-socket &&
/home/charlie/Documents/Pyramid/Validation/continuous-challenges-20261021/release/qmp-qemu-socket-2.8.119 --image
~~~

Use native 1920x1080, 100% scale, CLASSIC BLUE. Select Pyramid and Shortest Path.
Keep the guest Solver disengaged; unavailable Solver is allowed. Begin each trial
on an untouched supported board, with a fresh Recon/Plan. Do not resume a consumed
plan or repeat uncertain input. Leave the guest untouched while it runs.

| Trial | Settings and required observation |
| --- | --- |
| Normal complete loop | Continuous / Repeat normal games: actual Boards 1 of 3 -> 2 of 3 -> 3 of 3 -> shared GAME WIN XP/optional OK -> Solver-free NewGame -> fresh board one and Recon. Exercise more than one game. |
| Mid-game entry | Start at actual normal board two and board three; observe the actual ordinal and remaining progression, without a local assumed zero. |
| Two-board Star Club | Continuous / Stop at challenge target: actual Boards 0/2 -> 1/2 -> confirmed completion, shared XP skip/optional OK, then stop before NewGame/Play/Solver. Perfect at 1/2 must not finish the challenge. |
| Rank-card challenge | Use measured Twos, Fours or Sevens: actual rank/target remain fixed, counts advance in their own units across supported clear-to-next-board transitions. Completion during an earlier whole MATCH retires the route suffix; both clicks must be acknowledged first. |
| Score challenge | Bind real Score/current/target; observe monotonic progress, completion and the same shared rewards, then stop. Do not substitute tableau-removal count for score. |
| Both route outcomes | Shortest route reports minimum MATCH/KING cost only when proved. Maximum card matches reports full win or beneficial tableau removal; stock-only removals add no score. Check total actions/draws/recycles separately. |
| Partial endpoint | A beneficial partial route independently replays and reaches its exact non-clear endpoint, then stops in every automatic mode, including board three. No DRAW-until-REDEAL or NewGame. |
| Manual and Single board | Fresh Manual Recon -> Plan -> Solve and Single-board automation retain receipt, foreground, completion and STOP behaviour. |
| Refusal | STOP/context changes, unknown/missing/regressed/changed objectives and uncertain input send no later input or cleanup/replay. |

Only Twos/Fours/Sevens active labels currently have native calibration. Unknown
ranks/themes refuse and require actual evidence before support can be claimed.
The common settled Challenge Complete patches have actual Fours/Aces evidence;
Score/Boards completion semantics follow Charlie's shared-GAME-WIN clarification
and have synthetic bound-event transport regressions, but need their fresh live
trial. An unread final numeric header is reported as unread, never fabricated.

For each trial retain executable/process identity, starting actual objective/count,
selected route/stop choices, route guarantee and match/action/draw/recycle counts,
last verified effect, progression and first failure packet. Capture PNG obtains
a new frame; it is not a substitute for a retained first failed observation.
No successful source test upgrades M10/M11 acceptance. No automatic redeal is
implemented; maximum-removal DRAW/REDEAL continuation is the later milestone.
