# Verification — qmp-qemu-socket 2.9.146 candidate

This app-only candidate adds complete native rank/suit recognition for the retained
Classic and Retro families on supported white/cream cards, and a fixed activity
badge. **Full host qualification passed: 1449 tests and 743 sealed inputs.** Fresh Beast behaviour and Charlie
acceptance remain pending. No 146 controller has been installed or launched, no
146 guest input has been delivered, and no push has occurred.

## Scope and identity contract

Bundled native-v23 privately selects `native_corner_ink_v1` in
[foreground_card.rs](../src/pyramid_recon/vision/foreground_card.rs). It reads a
checked 31×49 corner at card-relative `(3,4)`, estimates local paper and retains
complete seed-supported ink components. Complete Classic and Retro font families
compete coherently; rank and suit cannot borrow authority from unrelated families.
The native image is never resized. Translation is limited to −1 through +1 native
pixel in half-pixel steps. Missing, contradictory, unsupported or ambiguous ink
refuses, without a legacy fallback or deck-based identity inference.

| Decision | Required bound |
| --- | --- |
| Rank error | ≤220000 ppm |
| Separation from another rank | ≥80000 ppm |
| Suit error | ≤200000 ppm |
| Separation from another same-colour suit | ≥40000 ppm |
| Retro size guard | Applied after all label competitors are scored |
| Complete suit component | Exclusive bottom ≤51; reject crossing components whole |

Custom JSON, frozen readers and earlier constructors do not acquire this private
capability. The 145 full-pile stock rule remains: RGB channels all at least 180,
45% paper coverage means nonempty; its inverse means empty. Card identity,
card/empty exclusivity, scene/modal, exact position, fresh probe and STOP/context
checks remain required. No xSAR, public API or dependency version changes are made.
The separate automatic cleared-board-to-RECON handoff remains open.

## Retained pixels, direct labels and template split

The core regression corpus contains **510** directly labelled visible observations:
**87** from three unchanged actual boards and **423** Classic observations. Each
new board has 28 tableau labels and DL; the positive test checks actual `observe`
and all individual labels through both native banks. These observations do not
provide unseen stock, a complete 52-card deal, an original route or live authority.

| Retained new board | Source SHA-256 |
| --- | --- |
| Cream first | `e941c117e0e89f1ee32d027d2ccfe330f1d4560dec122e7871b5fc13c076be09` |
| Cream second | `314197bccc48cd33d2e5a47c81087709424b9d32fb1a716ec11e664aeaea6b33` |
| Retro | `f20abf37ca38d610d91f6d6e5ad55f013527ed32e995d8c6aa9a713b11376d1a` |

See [fixture provenance](../tests/fixtures/pyramid-recognition-146/README.md),
[direct labels](../tests/fixtures/pyramid-recognition-146/labels.json) and
[Retro raw corners](../assets/pyramid-glyphs/retro-corners-v1.json).
The Retro bank uses 13 exemplar cards, one per rank, contributing suit variants
covering all four suits. Its other **16 whole cards** are excluded from templates.
There is **no independent Retro holdout for 8, 10 or J**. All source frames were
exposed during development; no prospective unseen-source accuracy is claimed.
Retro DR's visible appearance does not establish a card identity or hidden contents.

Eleven focused foreground tests pass. They cover all 510 core observations,
whole-board integration, the raw exemplar/held-out partition, sparse metric
agreement with an independent dense definition, private-capability isolation,
missing/conflicting/unknown ink, suit/Queen clipping, missing `1` from `10`, intact
rank and same-colour suit replacement, external background replacement, competitor
ordering, body-art separation, bounds and actual refusal details. Synthetic
corruptions/recolours remain labelled test mutations rather than recorded captures.

## First failures and measured repair

The first full qualification found an occupied-RIGHT temporal REDEAL adapter
failure: `Stock effect could not be verified; input is not repeated`. Preserve
that result. Its synthetic LEFT substitution did not alter any of the retained
29 label crops. The intact KD at original `7|5`, origin `(1091,432)`, contained a
detached brown illustration fragment at card-relative `[23,50,24,52]` with two
strong dark pixels. The previous component envelope admitted the fragment into
red suit ink and rejected the otherwise intact card as mixed colour.

The repair bounds complete suit/joined components to exclusive bottom **51** and
rejects crossing components whole. It preserves genuine Retro DL antialias ink
at relative `(11,50)`, RGB `(251,224,220)`, which requires bottom51. The retained
510 positive bounds and 29 exhausted-stock labels support this measured boundary;
it does not establish arbitrary-font safety. The foreground regression and the
existing temporal-right adapter pass after the repair. Source/effect input guards
remain intact; no input is repeated to work around uncertain evidence.

The first Clippy failure and three subsequent iterator-only corrections are also
retained. Those corrections preserve the comparison/dense-test semantics. Full
qualification was restarted against the resulting source; earlier passing checks
do not qualify that changed revision.

## Activity badge and independent visual evidence

[badge.rs](../src/app/activity_status/badge.rs) draws a fixed **240×80 logical-point**
face with **3-point white border**, centred **DejaVu Sans Bold at 26 points** and a
10-point surrounding halo reserve. The font is embedded with its
[licence](../assets/fonts/DejaVu-LICENSE.txt); local font availability is not required.

| Displayed state | Face RGB | Animation |
| --- | --- | --- |
| WAITING | 52,56,62 | Static |
| RECON | 25,111,236 | Active halo |
| ROUTE | 234,159,0 | Active halo |
| SOLVE | 132,222,24 | Active halo |
| ERROR | 214,41,54 | Static |

Active halos have a 1.6-second period and 25 ms repaint requests; face colour and
geometry stay constant. The renderer neither sleeps nor spawns control work.
Independent inert screenshot/source review **PASS: 20 cases**, including window
sizes and pulse phases. Evidence:
`/tmp/pyramid-recognition-status146-6095c5ry/ui-preview-final/independent-visual-review.json`.
This is isolated renderer evidence, not a live worker, installed executable or
guest-acceptance claim.

## Full host qualification — passed

The full scope passed **1449 tests** and **743 sealed inputs**, with zero failed,
ignored or filtered tests. All six gates consumed unchanged sealed inputs. Use repository-selected
nightly, locked/offline dependencies, explicit `x86_64-unknown-linux-gnu` and all
features; include ignored tests and preserve package-scoped formatting. Strict
private rustdoc and Clippy deny warnings. Final output, durations and source seals are recorded below.

| Gate | Result | Duration |
| --- | --- | --- |
| scoped-format | `PASS (exit 0)` | `0.465 s` |
| check | `PASS (exit 0)` | `2.619 s` |
| strict private rustdoc | `PASS (exit 0)` | `1.817 s` |
| strict Clippy | `PASS (exit 0)` | `3.120 s` |
| full include-ignored tests | `PASS (exit 0)` | `405.902 s` |
| release | `PASS (exit 0)` | `13.418 s` |
| native release/help | `PASS (exit 0)` | `not separately timed` |

Final full-suite counts: `1449 passed; 0 failed; 0 ignored; 0 filtered`.
Verified seal count/report: `743 sealed inputs; qualification-current.json`.
Qualification report: `/home/charlie/Documents/Pyramid/Validation/pyramid-recognition-status-20261025/qualification-current.json`.
Qualified source digest: `4719184b1dac831bcc7234c9c8d5cc404d70d5d97f735b935bd72a0efa4f4f97`.
Protected executable: `/home/charlie/Documents/Pyramid/Validation/pyramid-recognition-status-20261025/native/qmp-qemu-socket-2.9.146`.
Executable SHA-256: `001f9b508b56755086d3d4c9ba44924aaff9394e0c439d89aa1efe2a1a2ca06e`.

Retained 145 evidence is 1436 full tests, 731 sealed inputs and its own protected
artifact; [145 verification](Verification-2.9.145.md) remains historical. Retained
xSAR 0.9.0 and Miri evidence must remain distinguished from checks newly run for 146.
All 19 matrix states and all 14 milestone states are unchanged.

## Launch and acceptance — pending

No 146 launch, installation, guest input or push is recorded. Do not infer current
process identity from an old launch record. After host qualification/protection and
when launch is authorised, record the resolved executable, matching running-byte
hash, PID and startup/session identity before fresh manual acceptance. See
[146 manual checks](Live-Test-2.9.146.md).

## Code Analysis

The recognition threat is a plausible wrong identity or false absence granting
input. Complete families, all competitors, same-colour suit gaps and ambiguity
refusal bound that risk for the measured fonts; no arbitrary-artwork guarantee is
made. Checked corner access and bounded component/translation work preserve image
bounds, with no unsafe code or glyph rescaling. Complete lower-art components are
rejected rather than clipped into a plausible suit. Rendering has fixed geometry,
a bounded halo and display-only repaint; it grants no gesture authority. Existing
scene/modal, occupancy, exact-position, STOP/context, one-operation acknowledgement
and non-replay guards remain required. Saved-pixel and inert-UI evidence do not
replace fresh Beast/Charlie acceptance.

## Separately retained 145 normal-series failure

During final 146 delivery, Charlie reported a 145 Continuous / Repeat normal games
stop after 52 of 54 verified actions and 69 acknowledged gestures. Evidence is at
`/home/charlie/Documents/Pyramid/Validation/normal-final-match-145-20261025`.
The Boards 1 of 3 header still matches. A flying 8♦ shadow invalidates the New
footer prerequisite, and the normal-series effect guard rejects Unknown before
action 54. This is separate from card recognition. Qualified 146 includes no
correction for this issue; no further input or replay was sent. The evidence
directory contains the read-only diagnosis in `inspection.json`.
