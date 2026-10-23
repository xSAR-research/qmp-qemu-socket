# Native-v10 highlighted-clubs fixture

This immutable evidence set supports the approved exact clubs-suit append in
both reader banks. It does not authenticate a restored acquisition or authorise
input. Required host checks passed: 1,043 app tests, zero failed/ignored. Actual results are
in [Verification-2.6.108.md](../../../../docs/Verification-2.6.108.md).

| File | Bytes | SHA-256 |
| --- | ---: | --- |
| `profile.json` | 808,237 | `778b74051a4f12b383c8e79ac951ef332dabf188ce67583e1358cc2b957f5d51` |
| `baseline.png` | 3,744,756 | `ef3a0fd6a4fcbffd935ad6f5d14c2679d96d1aad53740ecd12ca21eb77e7aa21` |
| `latest.png` | 3,704,810 | `a6827e24bea61323a315fb3cd0c0e9458c60b0ccd4849ba9efdead6b0b0c8407` |
| `INCOMPLETE.json` | 1,579 | `a11b7989357ae4e6cab683da17ad66f487473b881aef0c62341d4071e5e60d87` |

[provenance.json](provenance.json) records original paths, crop, unchanged policy
and whole-profile rollback. Source `/home/charlie/tmp/.pyramid-recon-721903-0`
has four captures/two attempted and acknowledged D operations, complete tableau
and two known stock cards (4♣, 2♣). It reports restoration false, complete false,
publication not attempted, no export and no delivery uncertainty. The later
visible piles are LEFT A♠/RIGHT 2♣; do not infer a full 24-card stock or receipt.

The exact native clubs sample is `[1027,708,20,16]`, 960 row-major RGB bytes,
SHA-256 `b04d2a7ad13c0f3a04af1d4020df66c5f97dad7a480321ebf7dc10666c4468b4`.
No resampling, recolouring or masking transformation occurs. All 320 mask bits
remain selected at tolerance zero. Both suit banks increase from nine clubs
variants to ten. Remove the appended sample and restore the ID to reproduce
v9 SHA-256 `b6ffea89b108f222f1d1aee80a951e024588b465465200fc7daa612c20e21865`.
Rank/suit XOR caps 24/8, alignment ±1, all old samples, native geometry,
negative space, colour, complete-modal and positive-empty evidence remain.
Old v9 RIGHT refusal has best rank distance 9 (passing), best suit distance 9
(failing); the appended native suit is an exact zero-distance match.

The focused production-parser/recognition run passed **9 tests, zero failed or
ignored**, with 1,033 filtered, in 8.63 seconds. The preceding E0433 compile
failure in the separate new Fast-consumer test is retained in work-root
`first-native10-focused-failure/`; fully qualified atomic Ordering fixes that
test without changing production recognition. Focused cases overlap the full
suite. Full app 1,043/xSAR 266 tests and required checks passed in attempt 0002;
the stable 108 artifact identity is verified. Beast acceptance remains pending.

Production-parser/reader regressions cover both banks, bundled/file parity,
all 28 tableau plus both visible piles, initial positive-empty RIGHT, retained
historical glyphs/scenes/modals and v9 King/selected-card positions. Negative
cases cover erased/damaged/unknown clubs, colour contradiction, exact/runtime
cross-label ambiguity, parser caps, full mask/negative space and scene/modal/
empty guards. Derivative pixels are synthetic refusal tests, not new captures.
Retain all historical v9/v8 evidence. Fresh native acquisition/restoration,
route calculation and execution require separate live acceptance.
