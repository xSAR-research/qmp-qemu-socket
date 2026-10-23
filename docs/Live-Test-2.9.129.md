# Fresh Beast test — 2.9.129

Host qualified; all twelve live cases pending. Installation/process replacement were not performed. No configuration keys, native profile or CLI flags changed. Use the protected candidate when the current run has finished.

1. Preserve wanted images/logs and close the old controller normally; run one controller. `--image` clears `/home/charlie/tmp/images`. Launch:

```sh
cd /home/charlie/repo/RUST/qmp-qemu-socket && /home/charlie/Documents/Pyramid/Validation/no-challenge-readiness-20261022/release/qmp-qemu-socket-2.9.129 --image
```

2. Select **A* Route Solution**, **Continuous**, **Shortest route**, **Continuous — no challenge** on an untouched starting board. Use fresh Recon in this process. After a full clear or guarded REDEAL, expect a waiting-for-settle message when needed, two equal complete positions, then guarded Recon and fresh planning with Shortest reset. Exercise at least two consecutive generations.

3. Retest Single board separately: exhaustive Shortest no-win should visibly fall back to Maximum with the same Recon, then stop at the verified partial endpoint. Only Continuous advances through stock/REDEAL. The final Weighted elapsed time describes the fallback pass.

4. Exercise STOP while waiting, unsupported/old/partial boards, disabled/unknown REDEAL allowance and changed context. No extra D, retry or cleanup is permitted. Continue the full carried-forward matrix:

| Case | State | Required result |
| --- | --- | --- |
| Optional Recon and Start single board | Pending fresh129 Beast | On an untouched board, optional Recon finishes without Missing restored acquisition frame. Start single board then reuses that valid Recon with no extra scan, plans and executes. Separately exercise Start single board directly after fresh process acquisition state; it must finish Recon and continue to planning. |
| A* label and Shortest default | Pending fresh129 Beast | On fresh startup, strategy label is exactly A* Route Solution and objective is Shortest route. Single board keeps Stop condition disabled. Explicit Maximum selection remains available and same-board option changes retain authenticated Recon while revoking its plan. |
| Reuse and optional Recon | Pending fresh129 Beast | Recon once; change mode, route and stop choice; Plan/Start reuse it with no new scan. Optional Recon remains visible; changed export/context and STOP refuse reuse. |
| Paired feedback | Pending fresh129 Beast | Display DL: card details \| DR: card details, DL first in UI and retained log. Preserve actual pile identities, logical stock order, 24 D, scan pacing, captures and restoration. Singles/exhaustion are honest. |
| Full-clear no-challenge loop | Pending fresh129 Beast | Replay goal plus all final action ACKs starts a three-second cancellable wait, then fresh Recon without PERFECT gating. Confirm several boards and Shortest reset. With positive Boards, Score and Card headers present, verify no-challenge keeps its default fallback and ignores target-aware Boards bypass and target stopping; ordinary scene/STOP guards still apply. |
| Same-inventory fallback | Pending fresh129 Beast | An exhaustively unwinnable board switches Shortest to Maximum visibly without rescan; independently replay and verify useful partial play. Exercise at least two consecutive fallback/redeal generations; each new board must reset the worker, status and dropdown to Shortest before planning. Interruption must never trigger fallback. |
| Remaining stock and REDEAL | Pending fresh129 Beast | Compare planned/actual RECYCLEs; consume only remaining passes, one ACK per D/effect; verify enabled allowance and one REDEAL, 1.5s wait, new Recon/Shortest. Repeat through consecutive fallback/redeal generations and exercise already-consumed passes. |
| Target-aware Boards bypass | Pending fresh129 Beast | A no-win Boards deal sends no partial MATCH/KING for either initial objective; only remaining legal stock work and one enabled REDEAL; actual Boards counter stays unchanged after redeal. |
| Target-aware Card/Score precedence | Pending fresh129 Beast | Actual reached target retires matching, D and REDEAL mid-board or during continuation; independently confirm completion and stop. Continue remains operator-owned. |
| Preserved modes/objectives | Pending fresh129 Beast | Manual, Single board, normal three-board Continuous/restart and both objectives retain their appropriate controls/effect proofs. No-card Manual result retains Recon and cannot arm Solve. |
| STOP and refusals | Pending fresh129 Beast | STOP/context at phase/probe/three-second/1.5s boundaries prevents next input. Zero/unknown/no-enabled allowance, wrong RIGHT, changed challenge binding, uncertain transport and stale next board stop without retry or cleanup. |
| Settled no-challenge next board | Pending fresh129 Beast | After a clear or guarded REDEAL handoff, supported animated card evidence waits with no input. Two consecutive equal complete untouched positions must precede guarded Recon. Changed/pending positions reset the pair; six-frame exhaustion, unsupported scene/dialog/layout, transport, STOP and context refuse. Cover consecutive generations and Shortest reset. |

Protected SHA-256 `6ecbb84469921767532321d13981329c5b10d7e3dc2454aba39f7f84d178cbea`. Existing three-second ACK-relative and1.5-second REDEAL waits remain policies, not timing measurements from stills. [Verification](Verification-2.9.129.md).

[Approved DL-first presentation clarification](</home/charlie/Documents/Pyramid/Validation/no-challenge-readiness-20261022/goal-supplement.md>) supersedes the immutable goal’s earlier DR-first display clause; all control and acceptance requirements remain.
