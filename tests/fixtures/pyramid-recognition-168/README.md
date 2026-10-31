# Retained 2.9.167 joint RGB regressions

These byte-original failure pairs are offline evidence, not fresh live 168 acceptance.
The first-unmatched PNG is an earlier observation, not the immediate runtime pre-action
source. Its relevant corner nevertheless reproduces the recorded final 167 counts.

Coral pending action 16 removes original slots 17 (6|3) and 22 (7|2), preserving
DL 3S and DR KD. Original slots 21 and 23 through 26 were already absent.
Green pending action 1 removes original slot 22 (7|2) and consumes DL 6H; the
expected next DL is JH and DR remains empty. Every other tableau identity remains.

At strict per-channel delta greater than 26, the same-pixel AND count is
276/1519 for the Coral 6|3 corner and 343/1519 for the green 7|2 corner.
Both exceed ten percent. Two repeated-image test serials exercise the settling
state machine only; they are not two fresh guest observations. Whole-position
assertions retain exact identities, piles, scene and prior-removal history.

provenance.json records every unchanged source file hash and original private path.
