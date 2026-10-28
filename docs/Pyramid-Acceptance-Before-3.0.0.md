# Pyramid acceptance before the Free Cell 3.0.0 transition

Updated 28 October 2026 for the approved 2.9.159 correction. Fresh 159 guest
acceptance is pending. The earlier statement accepting Continuous standard-theme
A* solving was retracted after the 158 normal-header refusal. It must not be
used to close the normal-game acceptance row.

The [master implementation schedule](Pyramid-Implementation-Schedule.md) remains
the milestone authority: M01–M10 are Complete, M11 and M14 await verification,
M12 is In progress and M13 is Planned. The
[testing matrix](Pyramid-Testing-Matrix.md) retains nineteen detailed rows from
153. Reconcile their old Pending labels with the actual retained evidence and
new operator results before closing a milestone; an old Pending label alone
does not prove that a later test failed.

## Immediate correctness acceptance

| Area | Evidence still needed |
| --- | --- |
| M11 normal Continuous repetition | On the current executable, use Continuous / Repeat normal games and record all three boards, their verified clear effects, the supported reward/Level Up/New Game/Play sequence when present, and a fresh paired starting board followed by Recon in the next game. The retracted standard-theme run remains open. |
| 159 exposed coral effect | On the native Underwater appearance, confirm that the original 7|1 and 7|3 removal completes without replay, later actions continue from the verified position, and the chosen run reaches its declared completion/continuation boundary. The single retained effect frame and synthetic negative tests are host evidence only. |
| M11 challenge completion | Retain distinct Card, Score and Boards results, including target completion before full clearance and completion after clearance. Preserve the actual objective, numerator and denominator; record Kings-only/advisory behaviour where applicable. Confirm the supported terminal handover and fresh next challenge. |
| M14 partial routes and remaining stock | Demonstrate the approved Maximum matches path, remaining stock, RECYCLE/REDEAL effect evidence and fresh Recon where a target remains unmet. Confirm the intended Shortest/default reset and explicit objective handover. A valid partial plan alone does not prove guest execution or completion. |
| Preserved controls and modes | Reconcile Manual and Single board, optional/reused Recon, paired feedback, pause/resume during the three supported phases, STOP/context revocation and uncertain-input refusal against their retained receipts. New live runs are needed for gaps or changed behaviour. No stopped/failed route resumes or repeats uncertain input. |
| Diagnostics and run identity | Each acceptance record identifies the executable version, theme, mode, stop condition, route objective/method, request/generation and retained log/packet. Distinguish current failure evidence from an earlier mismatch or stale route filename. |

The operator reported more than an hour of successful 157 Continuous play.
That report remains useful evidence for its observed run. It does not close the
later 158 standard-header and Underwater failures. Earlier 155 operator results
for Klondike and Spider Continuous remain separate from Pyramid acceptance.

Use complete existing logs and packets. Keep ordinary gameplay recording and
the `--image` GAME WIN boundary unchanged; no extra captures are needed solely
to populate this checklist. Before reusing a failed board, establish a fresh
supported position and the authority required by the selected mode.

## Separately tracked work

M12 still needs its measured pacing/options evidence. This correctness release
does not implement new pacing controls. M13 generic xSAR naming and consumer
migration remain a separate planned change. RESET with guest Undo All remains
an optional future QoL idea and is excluded from 159.

The saved [Free Cell Block 1 plan](free-cell-block-1-plan.md) is complete.
The first qualifying Free Cell build is classified as 3.0.0, with its major
version already approved. Free Cell implementation, its own qualification and
human acceptance remain separate work; the version classification alone does
not authorise those edits. xSAR continues to version independently.
