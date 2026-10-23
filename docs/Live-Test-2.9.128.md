# Fresh Beast test — 2.9.128

Host qualification is complete; fresh Beast acceptance remains pending. First retest the two operations Charlie reported failing.

1. Preserve wanted images/logs and close the old controller normally. `--image` clears `/home/charlie/tmp/images`. Run one controller. Launch the protected candidate:

```sh
cd /home/charlie/repo/RUST/qmp-qemu-socket && /home/charlie/Documents/Pyramid/Validation/restored-acquisition-20261022/release/qmp-qemu-socket-2.9.128 --image
```

2. Select **A* Route Solution**, **Single board**, **Shortest route**. Confirm the Stop condition is disabled. Leave the board untouched, click **Recon (optional)** and expect restored/published acquisition with no missing-frame error. Click **Start single board** and expect reuse, planning and execution.

3. On another untouched starting position, test **Start single board** directly with no current receipt. This path should Recon, plan and execute. New process/game/deal requires fresh acquisition; old published JSON grants no authority. A Single board run stops at its normal verified endpoint.

4. Continue the original full-goal cases below. Continuous/no-challenge and Stop at challenge target remain separate lifetime choices. Supported native one-deal REDEAL only; zero/unknown/unmeasured variants must refuse.

| Case | State | Required result |
| --- | --- | --- |
| Optional Recon and Start single board | Pending fresh 2.9.128 Beast | On an untouched board, optional Recon finishes without Missing restored acquisition frame. Start single board then reuses that valid Recon with no extra scan, plans and executes. Separately exercise Start single board directly after fresh process acquisition state; it must finish Recon and continue to planning. |
| A* label and Shortest default | Pending fresh 2.9.128 Beast | On fresh startup, strategy label is exactly A* Route Solution and objective is Shortest route. Single board keeps Stop condition disabled. Explicit Maximum selection remains available and same-board option changes retain authenticated Recon while revoking its plan. |
| Reuse and optional Recon | Pending fresh 2.9.128 Beast | Recon once; change mode, route and stop choice; Plan/Start reuse it with no new scan. Optional Recon remains visible; changed export/context and STOP refuse reuse. |
| Paired feedback | Pending fresh 2.9.128 Beast | Every actual pair shows RIGHT/DR then LEFT/DL together in UI and retained log. Singles/exhaustion are honest; inventory/restoration, 24 D and paired pacing remain unchanged. |
| Full-clear no-challenge loop | Pending fresh 2.9.128 Beast | Replay goal plus all final action ACKs starts a three-second cancellable wait, then fresh Recon without PERFECT gating. Confirm several boards and Shortest reset. With positive Boards, Score and Card headers present, verify no-challenge keeps its default fallback and ignores target-aware Boards bypass and target stopping; ordinary scene/STOP guards still apply. |
| Same-inventory fallback | Pending fresh 2.9.128 Beast | An exhaustively unwinnable board switches Shortest to Maximum visibly without rescan; independently replay and verify useful partial play. Exercise at least two consecutive fallback/redeal generations; each new board must reset the worker, status and dropdown to Shortest before planning. Interruption must never trigger fallback. |
| Remaining stock and REDEAL | Pending fresh 2.9.128 Beast | Compare planned/actual RECYCLEs; consume only remaining passes, one ACK per D/effect; verify enabled allowance and one REDEAL, 1.5s wait, new Recon/Shortest. Repeat through consecutive fallback/redeal generations and exercise already-consumed passes. |
| Target-aware Boards bypass | Pending fresh 2.9.128 Beast | A no-win Boards deal sends no partial MATCH/KING for either initial objective; only remaining legal stock work and one enabled REDEAL; actual Boards counter stays unchanged after redeal. |
| Target-aware Card/Score precedence | Pending fresh 2.9.128 Beast | Actual reached target retires matching, D and REDEAL mid-board or during continuation; independently confirm completion and stop. Continue remains operator-owned. |
| Preserved modes/objectives | Pending fresh 2.9.128 Beast | Manual, Single board, normal three-board Continuous/restart and both objectives retain their appropriate controls/effect proofs. No-card Manual result retains Recon and cannot arm Solve. |
| STOP and refusals | Pending fresh 2.9.128 Beast | STOP/context at phase/probe/three-second/1.5s boundaries prevents next input. Zero/unknown/no-enabled allowance, wrong RIGHT, changed challenge binding, uncertain transport and stale next board stop without retry or cleanup. |

Protected SHA-256: `09f213c4ad6a60ad072c31888e7531f6735cf4ff897337e30472c7f7a0fb7a69`. [Verification](Verification-2.9.128.md). Keep original failures; do not retry uncertain input or overwrite the published 127 exports.
