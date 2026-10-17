# Pyramid v6 native calibration fixture

`baseline.png` is the unchanged original 1920×1080 Grandmaster frame retained
after a refused reconnaissance observation. Its SHA-256 is
`02b1f0f98c8a4c5b5d57b4075fceb2829d1175d8b80ef2455d94fc81fff1c988`.
It exposes 28 tableau identities and the initial LEFT identity. Those 29 visible
identities do not establish the remaining stock, input delivery or restoration.
No synthetic image supplies its calibration pixels.

`profile.json` is the separately loadable schema-1 profile
`pyramid-native-1920-classic-blue-v6`. Its source is the exact accepted local v5
profile, SHA-256
`b48e0b1a20aec2439422a1b967de1468d5553cc9a12077784356c3b68bcfd3e7`.
`provenance.json` records the original, visible identities and two extracted
components. The historical manifest and named v4/v5 records remain unchanged.
The top-level `../native-profile.json` remains v4, not v6.

v5 refused slot 13's Queen rank at best XOR distance 28 of the allowed 24.
Independent diagnostics also found spade distances 10 at slot 16 (King) and 9
at slot 19 (Jack), above the limit 8. Slots are zero-based logical indices.
The repair appends these exact native RGB crops to both readers:

| Component | Absolute bounds (x,y,width,height) | Source slot | Per-label count |
| --- | --- | --- | --- |
| Queen rank | `(1093,331,27,24)` | 13 | 2 → 3 of 8 |
| Spade suit | `(895,462,20,16)` | 24 | 8 → 9 of 16 |

Slot 24's existing spade already matches v5 at distance 6 of 8. It supplies a
genuine same-label rasterisation variant. Each bank now contains 34 rank and
36 suit variants. All older variants, v5 chart masks, scene/modal signatures,
ROIs, XOR limits 24/8, ±1 alignment and paper/ink/colour checks remain unchanged.
Pixels are not recoloured, rescaled or used to synthesise complete cards.

The operator's runtime copy is
`/home/charlie/tmp/pyramid-native-1920-classic-blue-v6.json`.
All eight v6 regressions pass through the production Rust parser and
recognisers, including all 28 tableau identities, LEFT Ten of spades and
positive-empty RIGHT. Both reader banks preserve all 104 historical
observations, including the 16 held-out LEFT observations. The full suite
passes 858 tests with none failed or ignored; locked check, strict private
rustdoc, all-target Clippy and native release build also pass. Fresh live
verification remains separate: neither this original nor a successful reader
regression proves a complete stock scan or a new original-board restoration.

See [the profile contract](../../../../docs/pyramid-recon-profile.md) for loading,
bounds, recognition failures and the controller's separate input responsibilities.
