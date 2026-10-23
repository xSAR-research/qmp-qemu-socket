# Daily challenge with unavailable Solver

This fixture retains the exact first native capture and failure report from the
19 October 2026 reconnaissance attempt in
`/home/charlie/tmp/.pyramid-recon-645935-1`. It shows a supported 1920×1080
CLASSIC BLUE Pyramid daily challenge, Medium, with a grey unavailable Solver
control. The board counter reads `Boards cleared 0/2` and the stock caption reads
`1 deal left`.

| File | Bytes | SHA-256 |
| --- | ---: | --- |
| `baseline.png` | 3,770,751 | `bac4632eca0bd9b5b31cc4e3f94bfa7123b5d6e204875f3d56505a2aac7718ac` |
| `INCOMPLETE.json` | 867 | `343d049b54c4cd1c1eb83b08e7511eefbe71d407b74f4f9f69e223aee154d76e` |

Both files are unchanged copies. The source `latest.png` is byte-identical to
`baseline.png`; only one PNG is retained here. The original report remains
incomplete: one capture, zero input attempts or acknowledgements, no publication
and no restoration. Recognising this saved frame after the authorised correction
does not turn that earlier run into a successful acquisition.

The directly visible identities, in fixed top-to-bottom row order, are:

| Row | Cards, left to right |
| --- | --- |
| 1 | 3♠ |
| 2 | 3♥ J♦ |
| 3 | J♥ 2♠ 7♥ |
| 4 | A♣ A♦ Q♦ 5♠ |
| 5 | 2♦ K♥ 6♣ 5♦ 3♦ |
| 6 | K♦ Q♣ 9♦ 7♠ 4♦ A♠ |
| 7 | 7♦ 9♠ 5♥ 8♦ 6♠ 10♦ 8♥ |

LEFT is K♣ and RIGHT is positively empty. These 29 visible identities do not
establish the remaining stock inventory or a complete planning deal.

The regression uses the unchanged native-v7 profile. Solver appearance is no
longer a visual acceptance gate; the application retains its explicit operator
confirmation. Supported scene masks, exact rank/suit recognition, positive-empty
evidence, modal classification, input guards and restoration rules still apply.
In-memory toolbar mutations exercise that boundary without changing this original
PNG, profile masks, glyph samples, thresholds or prior native fixtures.

The production `observe` and `verify_position` regressions, including this actual
daily frame, are in
[`daily_challenge_tests.rs`](../../../../src/pyramid_recon/vision/daily_challenge_tests.rs).
They test offline recognition and refusal. Fresh live inventory, restoration,
route execution and game completion require their own evidence.
