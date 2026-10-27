# Repeated Pyramid effect boundary, 2.9.155 regressions

These files preserve original native observations from two distinct reported failures.
No screenshot, glyph, colour, alignment or pixel value has been edited.

- `153-last.png`: request 9 final observed frame from packet `pyramid-error-1793030977355-1883669-5`.
- `154-last.png`: request 1 final observed frame from packet `pyramid-error-1793071413944-3808873-0`.
- `deal.json`: original complete deal from the 154 packet. Both packets have identical 28 tableau and 24 stock card identities; their provenance differs.

Both frames expose the same King of diamonds at DL with Four of clubs at DR.
The original 31 by 49 corner bytes are identical (SHA-256 `de85160f3a94e873612e0ad241f562386b653f7e91543b180403eb86afad9344`).
Two dark brown illustration pixels were incorrectly treated as strong black seeds,
despite failing the existing neutral/aged-blue colour rule already used by grown ink.
The correction requires two compatible strong seeds per black component while
retaining every original point and weight in each supported component. The original
neutral/aged-blue channel limits and two-seed minimum are reused. This preserves
the warm club-stem contours in the existing Football and cream-board fixtures.
Geometry, total strong-ink minima, complete rank/suit requirements, matching limits,
competitors and the mixed-ink veto remain unchanged.

The direct reader regression failed on both original frames before correction:
DL None, zero complete corners, unknown rank/suit rather than Empty.
After correction both full expected visible positions pass the actual reader.
The production NativeIo adapter also accepts both positions through isolated local
PNG/QMP fixtures, with zero guest input and no route delivery.

The legal eleven-action prefixes in the adapter tests are explicitly diagnostic
reconstructions from original identities, visible positions and delivery counts.
They are not the unsaved consumed solver traces. No new search is used.
The 153/154 first-mismatch frames described earlier recovered during play; neither
historic first reason identifies the final DL refusal.

Erased rank/suit and inserted neutral black ink are synthetic negative derivatives
created only in test memory. No derivative replaces these byte-original files.
This evidence proves offline recognition and adapter behaviour, not fresh guest
acceptance or permission to replay a stopped route.

## Original file hashes

- `153-last.png`: `2a68cc2c7345e8763396e1b8635a655ac9eca06422ca3cf7bc6733f8f7fe1604`
- `154-last.png`: `89266d363482bb73a379a8622d9890700f725e179a905fa77f39f8b7ebf40b4f`
- `deal.json`: `1311174ed8fa821aeb0984247e7ee6a5a2515eae19469b52620d51cf928d55e7`
