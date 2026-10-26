# Verification — qmp-qemu-socket 2.9.145

This app-only revision implements Charlie's inverse paper-presence rule for DL/DR.
It is host-qualified; fresh Beast behaviour and Charlie acceptance remain pending.

## Scope and evidence

Only `native_empty` and its stock percentage constant change runtime behaviour.
At the existing per-channel light-paper floor 180, coverage of at least 45% is
nonempty and its inverse is empty. For a full 139×187 rectangle (25,993 pixels),
11,696 light pixels are empty and 11,697 are nonempty. All geometry and pixels
are checked; u128 cross-products avoid rounding and overflow. Rank/suit recognition,
card/empty exclusivity, scene/modal and all gesture guards remain required.

The retained survey uses 38 labelled card and 21 labelled empty observations across
37 paths, with repeated captures. The minimum full card is selected K♥ at
13,883/25,993 = 53.410534%; every labelled empty sample has zero. The retained
football Q♥ has 56.634479% light paper but only 0.596314% at RGB 245, so the existing
180 floor is required for cream paper. Black-card themes are outside this scope.

The unchanged football frame supplies stock-only evidence. Its DR already passed
144; thirteen tableau ranks exceed the existing shape-error cap. This revision
does not claim whole-board recognition for that appearance or repair the separate
automatic board-to-RECON handoff. No glyph thresholds or timing values change.

Four existing compiler inputs change (two manifests and two Rust files), with two
new fixture inputs. The app is 2.9.145; all 52 retained xSAR inputs are unchanged.
No new dependency, public API, unsafe code or concurrency primitive is introduced.

## Verification results

The new boundary regression fails against 144 at a single light pixel, while the
real cream/selected-card test passes. Both new tests pass with 145; the related
stock and temporal REDEAL regressions also pass. The previous any-pale-pixel veto
is explicitly replaced by the requested percentage semantics.

Full suite: **1436 passed; 0 failed; 0 ignored; 0 filtered**. There are **731 sealed
inputs**; every gate rechecks their bytes. Commands use selected nightly, locked,
offline, explicit x86_64-unknown-linux-gnu and all features. Tests include ignored
tests; rustdoc denies warnings/missing docs, and Clippy denies warnings.

| Gate | Exit | Seconds |
| --- | --- | --- |
| scoped-format | 0 | 0.114 |
| check | 0 | 10.085 |
| rustdoc | 0 | 2.168 |
| clippy | 0 | 10.485 |
| full-tests | 0 | 418.687 |
| release | 0 | 13.816 |

Native release/help also passed. Protected executable:
`/home/charlie/Documents/Pyramid/Validation/pyramid-stock-paper-20261025/native/qmp-qemu-socket-2.9.145`.

SHA-256: `aa13f545dc153b3be177804f8a06bbaa548f4a4858ac8e94b535eebd59cc7ae5`.

Prior xSAR 0.9.0 full 296 tests and exact pause-primitive Miri 3 are retained,
hash-compatible evidence, not newly run 145 tests. The qualified GUI is now launched;
no agent START or guest input occurred. Fresh gameplay acceptance remains pending.
See [manual checks](Live-Test-2.9.145.md).

## Launch for Charlie

After Charlie indicated readiness and the guard found no previous controller,
the protected 145 executable launched at UTC 2026-10-24T22:54:53.586810+00:00.
Process `1677835` resolved to `/home/charlie/Documents/Pyramid/Validation/pyramid-stock-paper-20261025/native/qmp-qemu-socket-2.9.145`; running bytes matched
SHA-256 `aa13f545dc153b3be177804f8a06bbaa548f4a4858ac8e94b535eebd59cc7ae5`. Session `/home/charlie/tmp/qmp-qemu-socket-1792882493629.log` started with v2.9.145.
Startup-cleanup evidence was checked: 0 image files
required preservation. No agent START or guest input occurred. `launch.json`
records actual provenance; `candidate.json` is the immutable prelaunch snapshot.

## Code Analysis

The runtime change is confined to [stock occupancy](../src/pyramid_recon/vision/native_colour.rs)
and reuses the checked pure paper counter. False absence is the relevant risk:
the supported white/cream corpus bounds the selected threshold, while high-paper
unreadable faces and simultaneous card/empty identities still refuse. The user
excluded black-card themes. Bounded u128 arithmetic and checked frame rectangles
introduce no unsafe code, allocation or new I/O. Host image proofs are distinct
from live guest acceptance. Existing input sequencing and no-replay rules remain.
