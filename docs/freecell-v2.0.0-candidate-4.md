# Free Cell Level Up OK — v2.0.0, candidate 4

Base: `5d2d64bb6719af277802c717685d42da9e3bc59a`, promoted v2.0.0 candidate 3.

## Problem and resulting behaviour

Free Cell's Level Up artwork places the OK control at different vertical
locations. The worker has already entered the independently recognised
one-board win sequence, but the original local caption alignment allows only
eight pixels of movement. The supplied level-102 and level-80 screens exceed
that window, so the expected OK control is never ready.

The expected Free Cell OK check now tries the three measured whole-control
layouts: FC10 at offset 0, FC22 at -19 and FC23 at +12. It reuses the existing
checked local word/body helper. No level, medal, fireworks, frame decoration
or completed-card samples are added. The existing word/body thresholds and
click at (959, 812) remain: that point is inside gold on all three captures.

Spider explicitly retains its original offsets 0/-19 through the same helper.
Free Cell's additional +12 layout does not expand Spider's policy. Its existing
zero-offset rejection assertion now calls the zero-offset helper directly.

## Original supplied evidence

| Fixture | Original PNG | OK glyph bounds | Offset from FC10 |
| --- | --- | --- | --- |
| FC10 | Existing opening Level Up evidence | (934,798)..(982,824) | 0 |
| FC22 | qmp-qemu-socket 261008 093811 Fail-LEVEL-UP.png | (934,779)..(982,805) | -19 |
| FC23 | qmp-qemu-socket 261007 172628 FAIL-Level-Up.png | (934,810)..(982,836) | +12 |

The new fixtures are complete byte-identical copies of the supplied PNGs;
their names, sizes and SHA-256 hashes are recorded in the fixture manifest.
Both shifted captions match all 54 occupied cells at their measured location.
The level-102 original-window match is only 25%, below the existing 75%
threshold; shifted warm-body coverage is 94.79%. The level-80 shifted body
coverage is 94.78%. No session log was supplied for these two screenshots;
they establish readiness rejection rather than a specific stopped worker stage.

## Preserved behaviour

The optional Level Up branch, Score/New Game/Play order, one-shot terminal
input, STOP, socket/strategy invalidation, gameplay detectors and settle delays
are unchanged. An isolated OK still cannot establish a game win. No card-effect
pixel proof or HALO threshold change is introduced. Cargo version remains 2.0.0;
the candidate label advances to 4. The promoted xSAR Git pin and lockfile,
nightly configuration, two-blank-line style and exact-byte PNG saving remain.

## Verification

Focused Rust regressions include both native layouts, zero-layout rejection,
the retained click inside gold, expected-stage exclusivity and no independent
win entry from isolated OK. Existing artwork-isolation and warm-palette cases
also exercise FC22/FC23. Actual command outcomes and tool versions belong in
the candidate review and validation bundle; added tests are not reported as
executed solely because their source is present.

The Beast acceptance check is a continuous Free Cell game through Score,
optional Level Up OK, New Game and Play, including both with and without a
level increase. Retain the first unexpected frame and session log. The rare
card-back award dialog remains outside supported terminal handling.

## Code Analysis

- Threat surface: captured PNG/frame data enters existing layout validation;
  this patch adds only two original fixtures and two bounded local layouts.
- Memory safety: safe Rust, existing checked signed offsets and slice access;
  no unsafe code, arbitrary scan window or unchecked coordinate adjustment.
- Bounds: exactly three Free Cell offsets and existing 1920×1080 validation;
  Spider keeps its two layouts. Source-click and toolbar policies are untouched.
- Failures: missing local OK lettering/body still means not ready; no blind
  click, repeated uncertain input or new completion inference. The current
  fixed click is evidenced inside gold; another unsupported layout retains
  the existing bounded observations and guarded stop.
