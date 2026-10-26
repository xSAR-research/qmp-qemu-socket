# Native foreground recognition evidence for 2.9.146

These three unchanged 1920×1080 PNGs are retained operator failure frames from
2.9.145. No image was recoloured, reconstructed or recaptured for the positives.
The original packets remain in Charlie's private validation archive.

| Fixture | Original archive | SHA-256 |
| --- | --- | --- |
| `cream_first.png` | `grey-board-entry-145-20261025` | `e941c117e0e89f1ee32d027d2ccfe330f1d4560dec122e7871b5fc13c076be09` |
| `cream_second.png` | `grey-board-entry-145-second-20261025` | `314197bccc48cd33d2e5a47c81087709424b9d32fb1a716ec11e664aeaea6b33` |
| `retro.png` | `board-theme-entry-145-third-20261025` | `f20abf37ca38d610d91f6d6e5ad55f013527ed32e995d8c6aa9a713b11376d1a` |

`labels.json` records independently read rank, suit, native origin and one-based
row/column for all 28 tableau cards and the visible DL card in each frame: 87
observations, not 87 unique card identities. Tests compare the entire observed
boards and both reader banks against these labels. DR has no independently visible
face-up identity in the retro frame; its low-paper classification follows the
explicit stock policy and never invents face-down contents.

The private runtime asset `assets/pyramid-glyphs/retro-corners-v1.json` contains
unchanged 31×49 RGB corner crops from thirteen labelled cards of the retro board.
Those cards cover all thirteen ranks and all four suits. All sixteen remaining
whole-card locations are held out from that family. Ranks 8, 10 and J have no
independent same-font held-out observation; the tests and documentation do not
claim otherwise. The cream frames provide no new calibration templates. Existing
Classic calibration remains separate and all 423 previous labelled observations
are retained as regressions.

Synthetic tests alter only in-memory copies for missing rank/suit, clipped Q tails
and suit stems, missing first digit of 10, conflicting ink, unknown shapes,
ambiguous labels and changed external background. Intact donor glyph replacements
must return the donor identity, including a same-colour heart-to-diamond change.
No synthetic endpoint is presented as a recorded route or live guest acceptance.
