# Live test — 2.9.151 normal GAME WIN readiness

The executable is host-qualified and prepared for testing; no agent GUI/guest run occurred.

```sh
python3 /home/charlie/Documents/Pyramid/Validation/pyramid-post-play-readiness-20261025/launch-with-images.py --image
```

Close the old controller first. The guarded launcher preserves existing `/home/charlie/tmp/images` before normal startup cleanup. `--image` records GAME WIN sequence PNGs only. Source/binary and owned Wayland are checked. START is explicit/operator-owned; the GUI is not installed into PATH.

Use supported native 1920×1080/100% Pyramid, guest Solver off, A* / Continuous / Shortest / Repeat normal games. Start with checklistC01–C03: let two normal three-board series finish and enter each next game automatically. Expect pending dealing status if necessary, two equal full normal-one captures, an independent fresh entry pair, fresh guarded Recon, full 28+24 acquisition, mandatory UndoAll/OK restoration with two frames and direct planning. No manual START/RECON is needed between games.

Retain session log, GAME WIN PNGs, actual new Recon export and any failure packet. Compare increasing session/request/frame IDs; preview appearance alone does not establish freshness. Replayed retired OK/NewGame/Play, guessed cards, premature counter acceptance, Solver/HALO activation or resuming failed routes are failures. A six-observation exhaustion safely refuses with no further input.

[Full25-case acceptance checklist](Acceptance-Tests-2.9.151.md) covers all 13 outcomes and TracksA/B. [Host verification](Verification-2.9.151.md) distinguishes actual and composed fixtures. Earlier Queens challenge completion is separate and still pending; C08 is not passed by this host result.
