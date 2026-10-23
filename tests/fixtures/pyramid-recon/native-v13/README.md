# Native-v13 exact diamond calibration and exposed regressions

Status: **app 2.6.111 host verified; exact fixtures and protected artifact
verified; live acceptance pending**. The final documentation/preservation audit
passed for 9 documents, 200 local links and 27 review paths. This fixture extends the selected native-v12 schema-2
profile only with measured native 20×16 diamond RGB variants in both reader
banks. Original v12 assets, policy, exposure roles and verification stay intact.

The approved app-only correction appends measured native 20×16 diamond RGB
variants to both reader banks and selects bundled native-v13. Schema 2
`paper_ink_l1_v2`, rank/suit maxima 220,000/120,000 ppm, different-label gap
80,000 ppm, full ROI, nine ±1 shifts, paper support and exclusive colour remain
unchanged. No matcher or input-authority change is included. Literal custom
File profiles retain their selected schema, strict parsing and no fallback.

The priority evidence is the PLAY refusal for covered K♦ at fixed `3|2`, beside
cyan-highlighted 6♦ at `4|2`, after 27 of 42 route actions and 42 acknowledgements.
Separate RECON refusals expose K♦ at fixed `5|2` beside lower face artwork and
7♦ at fixed `3|2`. These original failures remain failures; finite labelled
regressions do not establish fresh acquisition, restoration or live success.

Manual/Fast and Single board keep their existing timing and search budgets.
The prior 45- and 42-action routes used **Fast**, with heuristic weight two;
they do not establish minimum action counts. **Fewest actions** selects exact
unit-cost A* with weight one, subject to its existing resource bounds. Select
it before a fresh RECON: changing method invalidates the acquisition receipt.
Never replay the interrupted route or use an old export as current authority.

**Native Continuous remains unavailable** pending an implemented and validated
standard-three progress/terminal contract. Retained source headers show 1/3
(PLAY selected/restored) and 2/3 (RECON king); these sources remain uncalibrated
and unverified for automatic progress. Positive 0/3, GAME WIN and restart-target
evidence are still needed. The larger goal is blocked; **M10 Awaiting
verification; M11 Planned**. xSAR 0.6.102 and console 0.8.4 remain frozen,
together with prior stable executables.


## Exact profile delta

The profile ID is `pyramid-native-1920-classic-blue-v13`. It appends two exact
diamond variants to each bank, increasing 10→12. Each sample is untransformed
native RGB, 20×16, 960 bytes, full 320-pixel mask and tolerance zero. Existing
rank/other-suit variants and the first ten diamond variants remain literal.
Removing only the appends and restoring the ID reproduces the complete v12
JSON. Both production banks contain identical diamond variants.

| Profile | Bytes | SHA-256 |
| --- | ---: | --- |
| Native-v13 `profile.json` | 827,721 | `507fbe1499cc6359b117022fb2cadb79f0737d4f3f77687bc763992acfc3edd8` |
| Parent native-v12 `profile.json` | 817,977 | `f8f1bc8b8a7e28d0ad3433494c47096f5ca5e0b546275912d845adf6afdd0813` |

Crops use absolute native `[x, y, width, height]`; row-major RGB hashes exclude
PNG encoding and alpha. No crop image is generated or used as independent
evidence. The source PNGs remain complete original captures.

| Appended offset | Original source | Exact crop | RGB bytes | Raw RGB SHA-256 |
| --- | --- | --- | ---: | --- |
| 0; bank variant 10 | `recon-king.png` | `[695, 354, 20, 16]`; existing `dy=-1` alignment | 960 | `7f352960d746d92e1967593971db867c0608b1bbb47c90ff9f3ac9ac76cd16f8` |
| 1; bank variant 11 | `selected.png` | `[895, 248, 20, 16]`; nominal alignment, legitimate neighbouring halo retained | 960 | `0a3b92079aa687173ca5e0293117f87b6db22622681f0e610e84a347c940c5d2` |

### Original-copy inventory

[provenance.json](provenance.json) records these eight exact original copies,
the two samples and evidence boundaries. All source hashes and byte counts
were checked against the retained fixture bytes. The `recon-seven.png` source
is exposed regression evidence and contributes no new template.

| Retained filename | Original path | Bytes | SHA-256 |
| --- | --- | ---: | --- |
| `selected.png` | `/home/charlie/tmp/pyramid-error-1792417065229-27073-1/first-unmatched.png` | 4,052,565 | `af3cbbc3b38a3cf8291bbea9ca45396f20f4c9be7e7dbf034dbdb3196baaf970` |
| `restored.png` | `/home/charlie/tmp/.pyramid-recon-27073-1/baseline.png` | 3,694,444 | `c5eebd228dcb0bfd7ad43ead2e793699c25b1d3af85e1fb25f4bf004ff04481e` |
| `deal.json` | `/home/charlie/tmp/pyramid-error-1792417065229-27073-1/deal.json` | 4,750 | `068d27acda9401a4203dff7a4867d6ccfdcf1b76481d70df1bdeee99ffdec9f9` |
| `error-state.json` | `/home/charlie/tmp/pyramid-error-1792417065229-27073-1/error-state.json` | 4,183 | `9eab79a193b9fcd5ac461f6bfdb5a6ebf94a0e94878aa372afa0ec9b7a60f852` |
| `recon-king.png` | `/home/charlie/tmp/.pyramid-recon-27073-2/baseline.png` | 3,756,144 | `4129c5cfe89c83f396a2c59601927707752447c6ea99541cbd185748a0312856` |
| `recon-king-INCOMPLETE.json` | `/home/charlie/tmp/.pyramid-recon-27073-2/INCOMPLETE.json` | 1,872 | `e068fe469120414130219faf412b65170c0c963fff47f04624c30d4b8b19b369` |
| `recon-seven.png` | `/home/charlie/tmp/.pyramid-recon-27073-4/baseline.png` | 3,774,242 | `12dd83cf991f7cce41e790acd90d287cb44babcd318b480a24377b3819f6018b` |
| `recon-seven-INCOMPLETE.json` | `/home/charlie/tmp/.pyramid-recon-27073-4/INCOMPLETE.json` | 1,867 | `e46b3e76140d5a4fce5f9e19ec1c288102570e4b5690a0b3556435de5a48504a` |

No generated artwork, masking, recolouring, rescaling, inferred card identity
or relaxed matching is included.
Schema 2 remains `paper_ink_l1_v2`: rank cap 220000 ppm, suit cap 120000 ppm,
different-label gap 80000 ppm at scale 1,000,000. Exact checked ratios decide;
diagnostic floor ppm does not decide. All nine ±1 shifts remain the full search
window; paper and exclusive colour must co-locate with the winning witness.
Schema 1 retains legacy XOR 24/8; custom File profiles remain literal.

## Source failures and exposure

The covered PLAY K♦ at fixed `3|2` sits beside cyan-highlighted 6♦ `4|2`.
The route stopped after 27/42 actions with 42 ACKs; preserve the original ledger
and do not equate delivery with a successful effect or replay authority.
The separate RECON K♦ `5|2` nominal suit crop contains one black pixel from
lower face artwork; an existing one-pixel upward shift has clean colour.
The separate RECON 7♦ `3|2` is an exposed native failure now covered in
both banks without adding a seven-source template. These samples are calibration/exposed regressions for v13,
not new holdouts. Source reports, complete boards/partial positions and
original card locations remain distinct; never compact remaining slots.

## Focused production result

The first focused attempt passed **9 tests, 0 failed/ignored**, 1,078 filtered,
compile 11.70 s/test 60.24 s. It exercises complete-profile rollback, exact RGB
samples, strict schema admission, old refusals, original zero-input reports,
positive-empty/scene/modal guards and colour/paper/shape rejection derivatives.

Both production banks retain all 423 v12 identities: 846 decisions. Each new
RECON capture has directly labelled 28-card tableau + LEFT, with positive empty
RIGHT: 58 identities × 2 banks = 116 decisions. The partial PLAY frame has eleven
remaining tableau identities and two visible piles: 13 × 2 = 26 decisions,
plus seventeen positive tableau absences in whole-position verification.
Total explicit identity decisions are **988**; observe/verify-position checks
add overlapping calls and are not additional independent observations/tests.

The complete two RECON positions and exact partial PLAY position now pass while
legacy v12 reproduces their retained refusal strings. The 7♦ exploratory score
is 65,269 ppm versus the old 128,673 ppm refusal; these are floored diagnostics,
not decision substitutes. Required host checks passed **1,087 tests, zero
failed/ignored**, including these nine. All Cargo commands exited zero in 0001;
the final preservation audit stopped on automatic trigger-tree read telemetry.
Audit-only finalisation 0002 rechecked retained logs and frozen inputs, without
repeating Cargo/tests, and passed artifact/help/preservation checks. The final
documentation/preservation audit passed for 9 documents, 200 local links and 27 review paths.
All three captures are exposed development/calibration regressions for v13;
none measures independent general accuracy. Saved visible evidence proves
neither unseen stock, complete acquisition, restoration, fresh receipt nor
guest acceptance.

[Verification](../../../../docs/Verification-2.6.111.md) owns exact host results
and the closed review scope. [Operator brief](../../../../docs/Live-Test-2.6.111.md)
requires a fresh same-session acquisition for a live trial. The work root is
`/home/charlie/tmp/pyramid-diamond-v13-validation-ugnvhgv9`. The protected
`release/qmp-qemu-socket-2.6.111` is 12,733,440 bytes/mode 0555, exact native
parity, SHA-256 `32638935cae71f240a6c6e733168dbcd1217bad916af0bd3fa10c26aac97b55b`.
Read-only preflight at 2026-10-19T14:08:38.312919+00:00 still observed old
protected 2.6.110 PID 27073 and zero images at that instant. Preserve evidence
and close the old app deliberately before operator launch; startup clears
images. No guest/GUI/input/install/promotion or process control was performed.
