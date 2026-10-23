# Live test — protected 2.8.122 Board challenge correction

App **1280 passed, 0 failed, 0 ignored, 0 filtered** in the complete native include-ignored scope (320.378 seconds). Locked check, strict private rustdoc, Clippy with warnings denied, release, formatting of twelve scoped Rust files and native/protected headless help passed. Unchanged xSAR **0.8.104** and console **0.8.4** retain their qualified 289/80 tests and feature rows 14/55/25/67/57/99; those checks were retained, not rerun.

## Preparation

The second board Charlie left ready can remain in the guest. Close the old controller normally after preserving wanted logs/images; --image startup clears /home/charlie/tmp/images. The five first-failure files have already been retained permanently. Run one controller and use fresh RECON/plan/receipt; never resume the failed run or retry its final input.

1. Launch the qualified protected executable:

   ```sh
   cd /home/charlie/repo/RUST/qmp-qemu-socket && /home/charlie/Documents/Pyramid/Validation/challenge-board-transition-20261021/release/qmp-qemu-socket-2.8.122 --image
   ```

2. Check app 2.8.122/native-v19 and SHA-256 `7ea7b0bf48e747a0f295985222cf99c0585a89135e7c507e2f13a79c386d4e73`. A rebuild does not replace an existing process.

3. Keep Pyramid / ShortestPath, Continuous, Shortest route and Stop at challenge target. Starting from the current complete second-board layout must use the observed 1/2 requirement and fresh RECON. Completing this board tests mid-challenge entry and target/reward stopping; it does not exercise the corrected first-board intermediate transition.

4. On a fresh two-board challenge, run the same settings. After the first clear, require observed 0/2→1/2, verified BoardClear, then an independently observed fresh starting board followed by RECON→plan→execute. The intermediate clear must not perform challenge rewards, NewGame or REDEAL. At the actual target, require the existing shared GAME WIN XP skip / optional Level Up OK, then stop before NewGame.

5. Retain the first failure and exact packet/process identity if any stage refuses. Do not replay uncertain input or reinterpret an intermediate counter as a complete challenge.

## Remaining acceptance

Fresh normal 1→2→3→GAMEWIN→Solver-free NewGame/repeat and mid-game entry; actual Score/Boards/supported-rank target progress and shared rewards/stop; both route objectives; Manual/SingleBoard; exact useful partial stop; STOP/context/uncertain refusal remain in the full acceptance scope. Search still has no automatic state/expansion/time/byte cutoff. Unknown ranks/themes refuse and M14 DRAW/REDEAL remains deferred. Keep the earlier unexplained TCP capture cancellation separate from demonstrated operator STOP.

Host regression success and the operator's 121 first-board clear do not establish fresh 122 end-to-end acceptance. See [verification](Verification-2.8.122.md) and [handoff](Game-Win-Handoff.md).
