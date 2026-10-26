# Stock paper coverage — 2.9.145

`football-entry.png` is the unchanged 1920×1080 image from the retained
2.9.144 entry failure:
`/home/charlie/Documents/Pyramid/Validation/automatic-entry-144-20261025/failure-packet/last-observed.png`.
SHA-256: `df563e62c911a05f8bbfd373bc519b0bfa6f9dbd563ef93c2f1d6bc9081f65b0`.

The visible LEFT card is Q♥ and RIGHT is empty. The full 139×187 LEFT
footprint has 56.634479% light paper (all RGB channels at least 180);
only 0.596314% reaches 245. RIGHT has zero pixels at either floor.
This fixture tests pile coverage and identities; it does not establish
whole-board recognition. Thirteen tableau ranks exceed the current
shape-error limit in this appearance, independently of RIGHT emptiness.

The existing `pyramid-recon/originals/issue1-13.png` supplies the visually
labelled selected K♥ LEFT card. Its 13,883 of 25,993 light pixels (53.410534%)
are the minimum in the bounded retained white/cream stock-card survey.
The requested inverse rule uses 45% coverage, keeping an 8.41 percentage
point margin. Black-card themes are outside the requested support.

Pixel-count boundary cases in the unit tests are explicitly synthetic,
not extra guest captures or an acquired/restored deal. The stored image
is unchanged; the tests never claim a route or live acceptance.
