# Partial endpoint with a recyclable empty DRAW pile

`last-observed.png` and `deal.json` are unchanged inputs from the retained
`pyramid-error-1792763849460-665890-8` packet, recorded by application 2.9.133.
The frame shows Score 1,520/2,500, six remaining tableau cards, an empty LEFT
circle, RIGHT Jack of spades, a recyclable white DRAW arrow and one deal left.
It does not show the final exhausted-stock X or grant an immediate REDEAL.

The original session log reported 30 partial-route actions, 29 verified actions,
50 acknowledged gestures and no RECYCLE. Its first refusal was
`execution board scene[2] mismatch at [931,695,60,50]`. No action CSV was retained.

`reconstructed-route.csv` is a newly constructed offline test route for this
complete deal, **not the original recorded route** and not a maximum-removal
proof. Tests validate every operation through xSAR rules and its independent
replay. Its visible endpoint matches the retained frame: original slots 0, 1, 2,
4, 5 and 8 remain; LEFT is empty and RIGHT is Jack of spades. It has 30 actions,
seven MOVEs and zero RECYCLEs. Original tableau locations remain one-based.

The local-QMP test checks the production partial-observation adapter and the
first guarded RECYCLE. Returning the same old frame afterwards must refuse the
unverified effect without retry. Existing exhausted-stock tests cover the
separate enabled-REDEAL operation; this fixture proves no full live sequence.
It creates no acquisition receipt and grants no guest resumption authority.

| Unchanged input | SHA-256 |
| --- | --- |
| `last-observed.png` | `4bdb5574e2a6b8f29ae740c8fd3ee46abe7ce53ccf47a477dca5c51dec323b7f` |
| `deal.json` | `d7168ec105222e47b71d7998451837f28fd32520f5616e372239f75a9263c46f` |
