# Native-v14 aligned King diamond sample and exposed regression

Status: **app 2.6.113 required host checks, protected artifact and audit-only
preservation passed; final root documentation/preservation audit passed;
live acceptance pending**. Native-v13 and historical
verification remain immutable. The new data is calibration/exposed regression,
not a new held-out accuracy experiment.

The app-only native-v14 correction appends **one measured full 20×16 RGB
diamond sample to both reader banks, 12→13**, and selects bundled native-v14.
The source is fixed `5|3` K♦ from the original failed RECON capture. Its nominal
shape already fits at 69,137 ppm, but one strong-black face-art pixel makes
nominal colour nonexclusive; the best eligible old witness is 130,980 ppm,
above the unchanged 120,000 ppm cap. The measured clean crop uses native
`[896,354,20,16]`, the existing `(+1,-1)` alignment. This is a bounded calibration
addition for an exposed case, not a general artwork or colour-policy repair.

Schema 2 `paper_ink_l1_v2`, rank/suit caps 220,000/120,000 ppm, different-label
gap 80,000 ppm, full masks, nine ±1 shifts, paper support and exclusive colour
remain unchanged. Native-v13, its templates and explicit custom File behaviour
are retained. The host-verified 2.6.112 stock-only Undo All/OK pacing correction
is preserved; no wait, search, input-authority or other-game change is added.

## Exact source and delta

Original source directory: `/home/charlie/tmp/.pyramid-recon-45759-3`.
The full original baseline PNG and INCOMPLETE report are copied byte-for-byte.
The original latest PNG and start report remain in the source directory; the
operator's later PNG is corroboration rather than a substituted fixture source. Baseline/latest are each 3,771,011 bytes, SHA-256
`7263c1ccc78dd7eed213217bbbd739c9128adb7375fb0465a2263300b856d857`.
INCOMPLETE.json SHA-256 is
`6e86160486800e88dbbb863cf77ec6a02d981c247d147f795465701aeea7ed05`.
The failure reports one capture, zero guest-input attempts/acknowledgements,
restoration false, no uncertainty and publication not attempted. Neither a
complete deal, unseen stock nor a live receipt follows from these files.

The nominal suit ROI `[895,355,20,16]` contains face artwork in its final row.
Pixel `(910,370)` RGB `[100,100,96]` invalidates nominal red90/black1 colour
eligibility despite its diamond shape score 69,137 ppm. Only y=-1 shifts have
exclusive red89/black0; native-v13's best eligible witness is 130,980 ppm,
above the unchanged cap. The closest other label is spades at 401,501 ppm.

Append exactly one full 20×16/960-byte native RGB sample from
`[896,354,20,16]` to each diamond bank, 12→13 within the existing cap16.
This corresponds to the already permitted `(+1,-1)` position relative to the
nominal ROI. No pixel rescaling, recolouring, mask omission or synthetic art is
used. The 320-pixel mask is complete and tolerance remains zero. Appends are
identical between banks. Removing both entries and restoring the profile ID
must reproduce complete native-v13, 827,721 bytes/SHA-256
`507fbe1499cc6359b117022fb2cadb79f0737d4f3f77687bc763992acfc3edd8`.

[provenance.json](provenance.json) records the root-generated exact source
copies, directly labelled visible identities, shifted sample and profile delta:

| Asset | Bytes | SHA-256 |
| --- | ---: | --- |
| [profile.json](profile.json) | 832,593 | `2e1c1cc7135dc8f5310396bba52473ba53c6ad4eb4b99b3b0b1631eac57ea9f9` |
| Appended raw RGB `[896,354,20,16]` | 960 | `cb56876b7359a18432c4acd237931090aa341372298b009d8bceaac6d2cfdbed` |
| [baseline.png](baseline.png) | 3,771,011 | `7263c1ccc78dd7eed213217bbbd739c9128adb7375fb0465a2263300b856d857` |
| [INCOMPLETE.json](INCOMPLETE.json) | 1,874 | `6e86160486800e88dbbb863cf77ec6a02d981c247d147f795465701aeea7ed05` |

Production regressions, audit-only preservation and the final root
documentation/preservation audit passed for nine documents, 221 local links
and 21 review paths.
The original nominal raw suit-crop hash
`09f403dcabca1f6be2b5c0ea191e3319d0ad34c23dd8b45e8aacbbbb68540cbe`
describes the contaminated diagnostic crop, not the newly appended shifted one.

## Verification and scope boundary

Focused native-v14 production tests passed **7/0/0**, 1,093 filtered;
compile 13.83 s/tests 80.22 s, retained in the work root's `focused-v14.log`
and `focused-v14-result.json`. The full suite passed **1,100/0/0**,
with required Cargo commands/release/help and protected copy checks passed.
Attempt 0001's preservation audit stopped on automatic trigger-tree telemetry;
audit-only resumption validated original logs/frozen inputs and refreshed only
the two already declared operational hashes. It passed without any workload/help
repeat. Runtime preflight still found protected 111; final root documentation
audit passed. The focused count overlaps the full suite. The regression scope includes original old-profile refusal, exact crop bytes and whole-profile rollback,
both banks, baseline complete visible position, previous selected-neighbour
and King/seven cases, historical corpus, parsing bounds, paper/colour/ambiguity
and scene/modal/positive-empty refusals. A nominal template append alone is
known insufficient; its perfect shape witness remains colour-ineligible.

The larger goal remains **ACTIVE and incomplete**. Native Continuous is still
disabled pending implemented/validated progress, objective, terminal/redeal and
restart handling. Charlie reports Single board works; that report is separate
from a new 2.6.113 operator trial. M10 remains Awaiting verification, M11 Planned,
M12 In progress and M13 Planned. The intermittent idle-CPU report has not recurred
under SysProf and remains unresolved; this correction contains no CPU fix.
xSAR 0.6.102, console 0.8.4, indices and protected stable 107–112 remain frozen.
No guest/GUI/input, installation, process replacement or promotion is performed.

[Verification](../../../../docs/Verification-2.6.113.md) owns exact commands,
results, artifacts, runtime boundary and complete correction review paths.
[Live test](../../../../docs/Live-Test-2.6.113.md) requires fresh acquisition,
restoration and current authority before a later operator trial.
