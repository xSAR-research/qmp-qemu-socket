# Fresh Beast test — 2.9.127

Host checks are complete; all new live rows remain pending. The selected goal is blocked pending fresh Beast testing after three consecutive checks found only protected v2.8.125. The agent has not launched or replaced the controller.

1. Preserve wanted images/logs and close the old controller normally. `--image` clears `/home/charlie/tmp/images` on startup. Run only one controller. Launch the protected candidate:

```sh
cd /home/charlie/repo/RUST/qmp-qemu-socket && /home/charlie/Documents/Pyramid/Validation/no-challenge-redeal-20261022/release/qmp-qemu-socket-2.9.127 --image
```

2. Confirm the displayed version and running executable. Use fresh Recon/Plan or an authenticated still-current acquisition within this process; old files grant no execution authority. Select Mode **Continuous**, Stop condition **Continuous — no challenge**, and Shortest route for the new loop. The target-aware option remains separate.

| Case | State | Required result |
| --- | --- | --- |
| Reuse and optional Recon | Pending fresh 2.9.127 Beast | Recon once; change mode, route and stop choice; Plan/Start reuse it with no new scan. Optional Recon remains visible; changed export/context and STOP refuse reuse. |
| Paired feedback | Pending fresh 2.9.127 Beast | Every actual pair shows RIGHT/DR then LEFT/DL together in UI and retained log. Singles/exhaustion are honest; inventory/restoration, 24 D and paired pacing remain unchanged. |
| Full-clear no-challenge loop | Pending fresh 2.9.127 Beast | Replay goal plus all final action ACKs starts a three-second cancellable wait, then fresh Recon without PERFECT gating. Confirm several boards and Shortest reset. With positive Boards, Score and Card headers present, verify no-challenge keeps its default fallback and ignores target-aware Boards bypass and target stopping; ordinary scene/STOP guards still apply. |
| Same-inventory fallback | Pending fresh 2.9.127 Beast | An exhaustively unwinnable board switches Shortest to Maximum visibly without rescan; independently replay and verify useful partial play. Exercise at least two consecutive fallback/redeal generations; each new board must reset the worker, status and dropdown to Shortest before planning. Interruption must never trigger fallback. |
| Remaining stock and REDEAL | Pending fresh 2.9.127 Beast | Compare planned/actual RECYCLEs; consume only remaining passes, one ACK per D/effect; verify enabled allowance and one REDEAL, 1.5s wait, new Recon/Shortest. Repeat through consecutive fallback/redeal generations and exercise already-consumed passes. |
| Target-aware Boards bypass | Pending fresh 2.9.127 Beast | A no-win Boards deal sends no partial MATCH/KING for either initial objective; only remaining legal stock work and one enabled REDEAL; actual Boards counter stays unchanged after redeal. |
| Target-aware Card/Score precedence | Pending fresh 2.9.127 Beast | Actual reached target retires matching, D and REDEAL mid-board or during continuation; independently confirm completion and stop. Continue remains operator-owned. |
| Preserved modes/objectives | Pending fresh 2.9.127 Beast | Manual, Single board, normal three-board Continuous/restart and both objectives retain their appropriate controls/effect proofs. No-card Manual result retains Recon and cannot arm Solve. |
| STOP and refusals | Pending fresh 2.9.127 Beast | STOP/context at phase/probe/three-second/1.5s boundaries prevents next input. Zero/unknown/no-enabled allowance, wrong RIGHT, changed challenge binding, uncertain transport and stale next board stop without retry or cleanup. |

Protected SHA-256: `4dc0e7156d78d7bfa1924e0cc4bc93f247e27271285d32df0512789c6d31dc13`. [Verification](Verification-2.9.127.md). Keep exact first-failure packets; do not retry uncertain input. Two-deal/no-caption variants are negative cases until measured, not promised positive support.
