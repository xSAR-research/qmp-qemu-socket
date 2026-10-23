# Partial normal Pyramid header references

This fixture calibrates the complete native `Boards: 1 of 3` header and the
measured `New` footer text. The explicit result is zero completed boards with a
target of three. **Normal ordinals two and three are not calibrated.** No glyph
is invented or derived from the centred challenge counter.

The production entry point is `pyramid_progress::observe_normal`. The existing
centred `observe` reader, its tolerance of nine and all its fixtures remain
unchanged. A header match does not authenticate Pyramid, a supported card/deck
scene, an initial or restored board, a consecutive-frame witness, a terminal,
or permission to send input. Those remain caller requirements. The exhausted
original is intentionally a positive literal-header example.

## Complete RGB rectangles

Bounds use native half-open `(x, y, width, height)` coordinates in a checked
1920×1080 frame. Every RGB channel of every pixel, including negative space, is
required. There is no mask, resize, shift or alignment search. Alpha and row
padding carry no evidence.

| Asset | Bounds | RGB bytes | Inclusive per-channel tolerance | SHA-256 |
| --- | --- | ---: | ---: | --- |
| [header-one.rgb](header-one.rgb) | `(1658, 46, 160, 32)` | 15,360 | 12 | `8d551d804798fc58c85924994d23ad6de3a03026d3ea26a15b83613fd9753077` |
| [footer-new.rgb](footer-new.rgb) | `(212, 998, 58, 31)` | 5,394 | 10 | `5b6e41f625d4ce074222fdc4abfef0bb86d7bcead7758ca95c909cace25b302f` |

Both exact row-major RGB arrays come from
[current-start.png](../pyramid-recon/originals/current-start.png), SHA-256
`4aff43e30e8dfa3218467fbedd9169b6c224613f97a7aa976722b8d39d24ac35`.
Its external native screenshot is byte-identical. The inclusive bounds of 12
and 10 are the maximum observed differences against that exact reference across
the seven supplied originals, with no added margin. This is finite calibration
evidence, not a general recognition-accuracy or independent holdout claim.

## Retained native positives

The parent evidence audit visually classified all seven distinct original PNGs
as the literal `Boards: 1 of 3` with `New`. The table reports complete-patch
maximum RGB channel differences against the reference. Six originals are reused
from existing fixtures. The seventh is copied byte-for-byte as
[normal-one-variant.png](normal-one-variant.png); it is not a synthetic frame.

| Repository original | Source PNG SHA-256 | Header maximum | Footer maximum |
| --- | --- | ---: | ---: |
| [current-exhausted.png](../pyramid-recon/originals/current-exhausted.png) | `b352d1645b15b2cb58d2a7a64e8b25a845ca91c955e3d66861cb76be1d352ef6` | 0 | 0 |
| [current-start.png](../pyramid-recon/originals/current-start.png) | `4aff43e30e8dfa3218467fbedd9169b6c224613f97a7aa976722b8d39d24ac35` | 0 | 0 |
| [issue1-20.png](../pyramid-recon/originals/issue1-20.png) | `e8b646f8c02ce841629e7e3da5ed181c5f17e093a004841bdf0060249da51653` | 8 | 7 |
| [normal-one-variant.png](../pyramid-standard-progress/normal-one-variant.png) | `70bceaad598f3b3cc8d0e9d76d4e6e193eb01666e55f2932700a262fedf7e3ae` | 12 | 10 |
| [gate3-exhausted-refusal.png](../pyramid-recon/originals/gate3-exhausted-refusal.png) | `9c3e3cde56674181f196fc42340927e02ab2a00c1050021e99d5bc09e2ad8c65` | 11 | 10 |
| [gate3-start-refusal.png](../pyramid-recon/originals/gate3-start-refusal.png) | `f6043267d22cbf3731df684bb5aefec25a2b6714b80a0c19b1d214f7d8e83baa` | 11 | 10 |
| [gate3-undo-baseline.png](../pyramid-recon/originals/gate3-undo-baseline.png) | `4148fababc943fb3670ef705b00f1aa612bcaaa7a4523e67d18274617f507e94` | 11 | 10 |

## Rejection and evidence limits

The tests reference existing native centred zero/one/two challenge headers,
Twos and Score objectives, a daily target of two, challenge-complete/Perfect!
scenes and a Pyramid gold reward. These are observed-artwork negatives; no
single-board mode is inferred from an absent header or a filename. Missing
native normal ordinals two and three remain an explicit calibration gap.

Labelled synthetic derivatives exercise word/ordinal/target/footer damage,
negative-space damage, every required pixel, inclusive tolerance boundaries,
ambiguous label metadata, invalid layout and padded/alpha-preserving layout.
They provide no additional native ordinal or mode evidence. Changing an
unmeasured card pixel intentionally leaves the literal header observation
unchanged; the caller must enforce scene and deck requirements independently.

[provenance.json](provenance.json) records source paths, hashes, exact rectangles,
per-source measurements, the partial-calibration boundary and existing negative
references. No guest operation, new screenshot, acquisition, completed game,
terminal control, Continuous acceptance or live trial is represented here.
