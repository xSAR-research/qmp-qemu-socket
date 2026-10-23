# Native shared-word Card evidence — app 2.8.126

[sixes.png](../tests/fixtures/pyramid-card-cleared/sixes.png) and
[nines.png](../tests/fixtures/pyramid-card-cleared/nines.png) are unchanged native 1920×1080 sources. The Sixes
frame is from the actual protected-125 Single-board acquisition (0/12); Nines
0/15 is the retained prior refusal. Both are exposed calibration/regressions.
[provenance.json](../tests/fixtures/pyramid-card-cleared/provenance.json) records original paths, complete hashes and crop derivation.

`cleared.ink` is the complete [912,48,84,32] Sixes word crop, blue saturating-minus-6.
`boards.ink` is the complete [841,48,80,32] reserved Boards word, veto only.
`banner-top.rgb` and `banner-bottom.rgb` are complete native strips [814,41,292,7]
and [814,80,292,7]. No synthetic rank word or glyph is bundled. Runtime search is
unscaled/horizontal; Card needs all seven core anchors, blank word boundaries,
native banner, full-word distance ≤210000 ppm and the unchanged actual numeric
reader. The Boards veto permits one lost core and never authorises Boards input.
The worst exposed positive distance is 187190 ppm; the Score negative minimum is
537810 ppm. These finite cases are not a general visual-accuracy guarantee.

Host tests include native Sixes/Nines through the actual paired adapter, synthetic
prefix corruption, each missing letter, joined boundaries, missing banner/counter,
damaged Boards and real STOP/context revocation. Synthetic cases are not native
theme or rank-appearance evidence. Fresh Beast verification remains separate.
