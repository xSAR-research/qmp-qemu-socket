# Literal native Pyramid board-count evidence

These fixtures support an application-owned reader of the literal headers
`Boards cleared 0/3`, `Boards cleared 1/3` and `Boards cleared 2/3` at native
1920×1080. They do not classify a standard game or grant completion, terminal,
restart or input authority. A challenge can display the same `/3` target.

The reader requires every RGB pixel in these half-open full-frame rectangles:

| Patch | Bounds `[x,y,width,height]` | RGB bytes |
| --- | --- | ---: |
| Left objective word | `[839,50,96,28]` | 8,064 |
| Right objective word | `[935,50,72,28]` | 6,048 |
| Complete fraction | `[1044,50,46,28]` | 3,864 per known value |

The five `.rgb` references contain original row-major RGB8 bytes. They retain
background and antialiased pixels, have no masks, and undergo no scaling,
translation, thresholding or OCR. Rust includes enforce their byte lengths at
compile time. The inclusive per-channel tolerance is **9**: the measured maximum
across the zero/one/two original left-label patches is 9 and the right-label
maximum is 3. The selected and restored one-of-three header patches are exactly
identical. Finite calibration evidence does not establish unseen-board accuracy.

[provenance.json](provenance.json) records every source, original SHA-256, direct
visible label, crop rectangle, raw crop SHA-256 and evidence boundary. The new
[zero-of-three original](zero-of-three.png) is an exact byte copy of the retained
first native capture `.pyramid-recon-27073-0/baseline.png`, SHA-256
`4920c6eb6996e7ba33f6a6f795d6786a15e802976098b228cc15bfc0ca2af436`.
The one/two/selected originals remain referenced in the frozen
[native-v13 fixture](../pyramid-recon/native-v13/README.md).

The two new full-frame negative PNGs retain the previously inspected native
Aces challenge-completion modal and the three-of-three `Perfect!` scene.
`negative-three-of-three-header.rgb` also preserves that original's exact
251×28 RGB header crop for explicit provenance checks. Neither negative supplies
positive terminal recognition. Existing full native originals for zero-of-two,
Twos goals, Score goals and Pyramid Level Up are referenced directly in the Rust
regressions; they are not copied or changed.

The tests cover exact known literals and independent selected one-of-three,
real other objectives/counts/modal frames, mismatched objective words, damaged
words/completed/target digits, every channel at tolerance 9 and refusal at 10,
every required pixel with a one-channel difference of 10, pairwise disjoint
known fractions, erased/mixed fractions, malformed references, invalid bounds,
invalid stride/truncated storage, unsupported dimensions, alpha and padded rows.
Synthetic mutations are labelled corruption/layout tests; they are not native
terminal or guest acceptance evidence. Root-owned Cargo verification remains
separate from fixture construction.
