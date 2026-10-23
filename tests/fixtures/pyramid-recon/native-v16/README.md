# Native-v16 gold objective-highlighted rank seven

Status: **measured calibration and exposed regression; production Rust
verification and new operator acceptance remain separate requirements**.
This four-file fixture appends exactly one native rank-seven sample to each
reader bank. It inherits the complete [native-v15 profile](../native-v15/profile.json),
including the two earlier depleted-stock and post-confirm OK calibrations.
All ten native-v15 files remain unchanged.

## Original evidence and precise change

[gold-seven.png](gold-seven.png) is the original
`/home/charlie/tmp/.pyramid-recon-65018-11/baseline.png`, copied byte-for-byte:
native 1920×1080 RGBA, 3,812,059 bytes, SHA-256
`95d9e45b4c588e87970bc9b4cd909cccc7f63a668652c561ad5f38d0f431f7aa`.
Its challenge header reads **Sevens cleared 0/9**. Gold objective highlighting
surrounds 7♣ at fixed `3|3`, 7♥ at `5|2` and 7♠ at `7|2`. These are challenge
target-rank highlights, separate from the disabled Solver control/progress.
They do not establish a standard three-board ordinal or Solver authority.

The retained failure made one capture and zero guest-input attempts. The
original INCOMPLETE packet remains in its source directory. Its production
failure is rank-seven quality at internal slot 5, not the clubs suit:
rank `6845/29908` (228,868 ppm) exceeds 220,000 ppm, while clubs
`901/27825` (32,380 ppm) passes 120,000 ppm. Closest different rank 2 is
`19223/39689` (484,340 ppm), leaving a 255,472 ppm gap. The old profile
correctly refuses the identity rather than treating a nearest label as proof.

The new sample is the exact nominal rank crop `[1093,224,27,24]` from 7♣,
1,944 row-major RGB bytes, SHA-256
`0183f982e56b55eafd62a6883a22e58152244cc039dcfea9ddc629681ed1da04`.
All 648 pixels, including negative space, are selected; tolerance is zero.
The existing rank origin `[1091,218]`, ROI `[2,6,27,24]` and allowed ±1
alignment are unchanged. The source has 523 supported paper pixels and lower
median paper reference 231. No image resampling, recolouring, synthetic glyph,
mask omission, new alignment or threshold relaxation is used.

Append this same variant at index 1 to rank 7 in **both** `tableau_reader` and
`stock_reader`, increasing each rank-seven bank from one variant to two.
Change the profile ID to `pyramid-native-1920-classic-blue-v16`.
Removing both appended entries and restoring the ID reproduces the **entire
parsed native-v15 profile**, whose SHA-256 is
`c107cec6ab419cdb345447c4dec216b3a3d3f9981c881f2ab38e22b7bf143b49`.
All other JSON values remain identical: earlier glyph variants, all suit
samples, graded `paper_ink_l1_v2` policy, 220,000/120,000 ppm quality caps,
80,000 ppm different-label gap, geometry, full masks, paper/colour/uniqueness,
scene/modal/positive-empty evidence, coordinates and input authority.

## Exact diagnostic comparison

The private diagnostic reproduces the unchanged production integer equations,
including independent supported lower-median paper references, full-ROI ink,
exact fraction ordering/caps/gaps and same-witness exclusive colour. It
reproduced the original production failure fractions and all 29 prior
dark-theme rank/suit winner and competitor fractions. It did not run Rust,
connect to the guest or change any runtime profile.

| Highlighted card | Old rank error | Proposed rank error | Result |
| --- | --- | --- | --- |
| 7♣ `3|3` | `6845/29908`, 228,868 ppm | `0/26875`, 0 ppm | Refused → accepted |
| 7♥ `5|2` | `4934/28905`, 170,697 ppm | Same old variant and fraction | Accepted throughout |
| 7♠ `7|2` | `5781/29350`, 196,967 ppm | `5008/29353`, 170,612 ppm | Accepted throughout |

The old bank recognises 28 of 29 directly visible card identities, refusing
only 7♣. The in-memory proposal recognises 29 of 29 with no wrong labels or
refusals. RIGHT remains positively cardless; unknown identity is not itself
an empty-pile proof. Other card/scene/pile authority remains caller-owned.

The immutable native-v12 corpus was compared without relabelling its splits:

| Retained evidence split | Direct labels | Old correct | Proposed correct |
| --- | ---: | ---: | ---: |
| Development | 250 | 250 | 250 |
| Ten-failure development regression | 29 | 29 | 29 |
| Previously exposed validation | 115 | 115 | 115 |
| Exposed fifth-failure regression | 29 | 29 | 29 |
| Total | 423 | 423 | 423 |

All 423 identities are unchanged, with zero proposed wrong labels or refusals.
There are 33 historical rank-seven labels; 21 use improved rank-seven witnesses.
No non-seven winner or competitor changes in this finite corpus. The entire
tableau and stock banks are structurally identical, so the same proposal
applies to both; these are 423 direct labels, not duplicate independent labels
or a production Rust test count. Historical highlighted-clubs evidence is
already included; this correction adds no suit variant.

Full exact fractions and all 29 current plus 423 historical observations are
retained at
`/home/charlie/tmp/pyramid-gold-seven-audit-pklji4sd/full-comparative-report.json`,
1,706,105 bytes, SHA-256
`dd877cd144fb14ab2bcdfb80df5f37eb1dc1d43c3d6be30fb003b798f9f8d85c`.
[provenance.json](provenance.json) records its identity, complete source/crop,
mask, direct labels, objective, frozen parent hashes and comparative results.
The private raw RGB and variant JSON are diagnostics; the bundled sample is
embedded in [profile.json](profile.json).

## Frozen assets and limits

The directory contains exactly these three assets and this README:

| Asset | Bytes | SHA-256 |
| --- | ---: | --- |
| [profile.json](profile.json) | 902,743 | `732af82cb2befbb3a420f5ae04ec11dd0b59bd5957a005df17218674bc01244e` |
| [gold-seven.png](gold-seven.png) | 3,812,059 | `95d9e45b4c588e87970bc9b4cd909cccc7f63a668652c561ad5f38d0f431f7aa` |
| [provenance.json](provenance.json) | 19,995 | `1cdbd36cfb8a43892e9733e75eb71ec03f0750ba10c4ea82eb0f026866281e8e` |

A perfect self-sample score is calibration, not independent validation.
The other two highlighted sevens share this capture lineage, and no gold 7♦
or prospective board is supplied. Adding a seven variant can lower seven's
competing-label score for other ranks, potentially causing ambiguity or a
false match on untested artwork. These 423 retained positives do not establish
general accuracy or replace production damaged/blank/paperless/colour,
full-mask, cross-label ambiguity and unsupported-scene rejection tests.

The fixture grants no Solver, REDEAL, guest-input, restoration or publication
authority. Original failure evidence and all native-v15 assets are preserved.
No Cargo, formatter, new capture, guest operation, memory capture or promotion
was performed in this asset slice. Root-owned production tests and required
host checks, followed by operator acceptance, remain separate.
