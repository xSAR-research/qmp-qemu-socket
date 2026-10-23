# Native-v15 depleted-stock and post-confirm modal evidence

Status: **calibration and exposed regression data; host Rust verification and
new operator acceptance are separate requirements**. All six PNGs are original
native 1920×1080 RGBA captures copied byte-for-byte. None is generated, resized,
recoloured or composed. The source failure packets remain untouched.

This fixture extends [native-v14](../native-v14/profile.json) with two explicitly
opt-in schema-2 fields: `board.execution_empty_stock` and
`dialog.post_confirm_ok`. It changes the profile ID to
`pyramid-native-1920-classic-blue-v15`. Removing those two fields and restoring
the old ID reproduces the **entire parsed native-v14 profile**. The parent
profile remains 832,593 bytes, SHA-256
`2e1c1cc7135dc8f5310396bba52473ba53c6ad4eb4b99b3b0b1631eac57ea9f9`.

The tableau and stock glyph banks, schema-2 graded matcher, thresholds, masks,
geometry, paper/colour/uniqueness guards, original scene/modal signatures,
control coordinates and existing tolerances are unchanged. Readers that do not
support the added fields must reject them. This fixture does not amend the
legacy profile or claim support for other artwork, scale, counts or scenes.

## Depleted stock with a remaining tableau card

[stock-empty.png](stock-empty.png) is the original
`/home/charlie/tmp/pyramid-error-1792433618738-65018-2/first-unmatched.png`.
It shows the Score 785/1,700 challenge, Easy, one deal left, a remaining K♣ at
fixed `1|1`, depleted LEFT marked with X, and cardless RIGHT. Its accompanying
[stock-empty-deal.json](stock-empty-deal.json) is the unchanged original
`deal.json` from that packet. The JSON is retained input evidence; this fixture
creation did not run a route search or replay.

The added `execution_empty_stock` object contains exactly two complete RGB
patches from that original:

| Field | Native `[x,y,width,height]` | Selected pixels | Inclusive RGB tolerance | Selected mean-channel contrast |
| --- | --- | ---: | ---: | ---: |
| `scene` | `[938,794,44,56]` | 2,464 of 2,464 | 4 | 221 |
| `left_empty` | `[767,712,60,60]` | 3,600 of 3,600 | 4 | 84 |

The scene patch selects the retained REDEAL icon, including its visible
one-deal count and surrounding negative space. It lies entirely in the
bounded central corridor `[920,678,80,187]`. The LEFT patch retains the original
LEFT ROI, with every pixel selected; the X marker is positive measured
card-absence evidence. Neither failed glyph recognition nor absent card ink
alone is an empty-stock proof. Contrast is the maximum minus minimum of
`floor((R+G+B)/3)` over selected pixels, using the existing minimum of 64.

These are scene and stock-face samples. They add no REDEAL coordinate, click
authority, allowance rule, challenge objective, standard-game identity or
terminal outcome. Existing RIGHT-empty and caller position/effect checks remain
separate requirements. The single retained icon/count is finite calibration,
not evidence for every possible deal allowance.

## Retained modal after acknowledged OK

[post-confirm-3.png](post-confirm-3.png),
[post-confirm-4.png](post-confirm-4.png) and
[post-confirm-5.png](post-confirm-5.png) are the untouched original `latest.png`
files from `/home/charlie/tmp/.pyramid-recon-65018-3`, `-4` and `-5` respectively.
All three show the complete Undo All dialog after acknowledged OK. Their
original INCOMPLETE reports record 24 acknowledged D operations, one
acknowledged Undo All and one acknowledged confirmation, followed by
`Unknown native scene; input authority revoked`, restoration false and
publication not attempted. The reports remain in their original directories;
[provenance.json](provenance.json) records their hashes and relevant ledger facts.

The added `post_confirm_ok` RGB is measured from `post-confirm-5.png`. It uses
**exactly** the original OK bounds `[763,611,65,38]`, the original 2,470-pixel
mask selecting 1,758 pixels, and the original inclusive tolerance 8. Selected
mean-channel contrast is 216. No button enlargement, mask change or tolerance
increase is involved.

| Original | Maximum selected channel difference from the new reference | Maximum unselected channel difference |
| --- | ---: | ---: |
| `post-confirm-3.png` | 4 | 7 |
| `post-confirm-4.png` | 4 | 10 |
| `post-confirm-5.png` | 0 | 0 |

The unselected difference of 10 remains outside the unchanged mask. All 1,758
selected pixels in each original match at tolerance 8. Full panel, title, body,
Cancel and dimmed-backdrop recognition remain separate conditions. A retained
modal after acknowledged confirmation is not permission to send another OK,
retry input, perform cleanup or publish an acquisition. ACKs do not prove that
the dialog disappeared.

[post-confirm-start.png](post-confirm-start.png) is the unchanged original
`/home/charlie/tmp/.pyramid-recon-65018-5/baseline.png`.
[post-confirm-restored.png](post-confirm-restored.png) is the unchanged original
`/home/charlie/tmp/.pyramid-recon-65018-6/latest.png`. These are separate retained
observations from different runs. They support exposed visible-position and
scene regression checks; their pairing does not establish a successful
same-run scan, restoration receipt, publication or new live effect.

## Exact assets and evidence boundary

The directory contains exactly the nine assets listed below and this README.
The SHA-256 values below cover complete file bytes. Raw crop hashes, masks,
selected luminance extrema, source paths and source ledger identities are in
[provenance.json](provenance.json).

| Asset | Bytes | SHA-256 |
| --- | ---: | --- |
| [profile.json](profile.json) | 893,279 | `c107cec6ab419cdb345447c4dec216b3a3d3f9981c881f2ab38e22b7bf143b49` |
| [stock-empty.png](stock-empty.png) | 4,386,014 | `ed33ce034e10f639815068dfdfdf8ad1ee18a7afe5fc0f1a0ccc011747236c27` |
| [stock-empty-deal.json](stock-empty-deal.json) | 4,750 | `2315688a1d441d228737ab404e171dcfb212768d6bdd52e63ffdde25c177bcc0` |
| [post-confirm-3.png](post-confirm-3.png) | 2,212,700 | `36a92ae00fcd183930fa6d48f4093fa9b9741d9573e097f305d81befdfeebd74` |
| [post-confirm-4.png](post-confirm-4.png) | 2,198,975 | `429c9eb940ff82a3c04d23c96a4bebd44771371a62d0aa7bce6ff7477241a122` |
| [post-confirm-5.png](post-confirm-5.png) | 2,198,597 | `863ea334461508a9077ba82b690fec177587c6a2e4b6eee43221a5ed746bb321` |
| [post-confirm-start.png](post-confirm-start.png) | 3,744,766 | `446d6b66a420a67936b1ff41a5be3e9db38af1dded1eac12c139df5f47935c81` |
| [post-confirm-restored.png](post-confirm-restored.png) | 3,744,734 | `d6cfba0501861ac24a27eae8ff3729ca036d0c238ab5caf27c97ddb513f9fbc5` |
| [provenance.json](provenance.json) | 9,905 | `685967a11225233267502cb7a8d538e08c17d764c69f6d9ad4a53a193375b3ad` |

Python checks performed during fixture creation established source/copy hash
parity, exact crop RGB, unchanged OK mask/bounds/tolerance, all three selected
OK matches, contrast/bounds and complete profile rollback to native-v14. They
do not constitute production Rust acceptance or host-suite results. No Cargo,
formatter, guest connection, input, new capture, process replacement, memory
capture or promotion was performed for this asset work.

All sources are exposed failure/calibration evidence. There is no prospective
or held-out accuracy claim, new successful acquisition claim or operator live
acceptance in this fixture. Original first-failure diagnostics are preserved.
