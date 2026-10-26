# Exhausted LEFT with an occupied RIGHT pile

The unchanged `last-observed.png` and `deal.json` come from the retained 2.9.136
Boards failure packet. The frame shows KS at `1|1`, 8S at `2|2`, exhausted LEFT
and 5H on RIGHT, with Boards progress 1/3. Their SHA-256 values are:

- PNG: `eb440094ad5a7236fc8da6bda12313e5864d9823109e641cd25c0a44c10c9d3b`.
- Deal: `a6709d79f45a17b1dd6779102977851522dc7af6b6fabf3ea4b194a956a750a7`.

`reconstructed-actions.json` is a **new legal reconstruction**, not the original
unrecorded route or evidence that these operations were delivered to the guest.
Its integer identities use xSAR's explicitly zero-based original identity API:
0–27 identify fixed tableau slots; 28–51 identify original stock ordinals.
The test resolves stock identities through the actual visible DL/DR tops and
independently applies/replays every action before using the late state.
The final two operations are ordered MATCH(DR, `2|2`) and KING(`1|1`).

Post-MATCH and post-KING images created in test memory are labelled synthetic
derivatives. They preserve the retained scene/header and erase only removed
cards, repainting pile glyphs from the independently expected identities. They
test position verification and ordered native adapter input, not a live guest
effect, calibration change, REDEAL allowance or resumed-run authority.

The synthetic post-KING frame also uses the complete original Boards 2/3 counter
crop from `pyramid-recon/native-v13/recon-king.png`. It is a labelled composition
that satisfies the existing one-increment final-clear contract, not a captured
post-KING guest frame. The original packet basename is
`pyramid-error-1792822252899-861483-6`.
