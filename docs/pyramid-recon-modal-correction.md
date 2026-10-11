# Pyramid Gate 3 Undo All modal correction

Date: 11 October 2026, Australia/Brisbane. Application package remains 2.1.0.

## Live evidence and outcome

The failed run `.pyramid-recon-2887595-0` recorded 28 captures, 24 acknowledged
D operations, 28 tableau cards, 24 ordered stock cards and one acknowledged
Undo All click. The retained inventory contains exactly the 52 distinct standard
rank/suit identities. OK attempts are zero, restoration is false and complete
publication is `not_attempted`. This is useful live acquisition evidence, but
neither a restored deal nor valid route-input JSON.

The worker's final original PNG and Charlie's later screenshot show the same
specific Undo All confirmation. The board was left untouched at that dialog.
No assistant QMP input, cleanup, process launch/termination or resume occurred.

| Retained original | SHA-256 |
| --- | --- |
| `gate3-undo-baseline.png` | `4148fababc943fb3670ef705b00f1aa612bcaaa7a4523e67d18274617f507e94` |
| `gate3-undo-modal.png` | `1fea2be304f49907e103042e4784e04e1206173ebc3d2869a54e366a5253370b` |
| `gate3-undo-modal-operator.png` | `5ebaad57410791e09a77bdf029eef4ef259d573b1f659789d00d1da86dfb3f04` |

All are unchanged 1920x1080 RGB PNGs in the native fixture manifest. The exact
diagnostic is retained as `gate3-undo-incomplete.json`; the shared xSAR reader
must reject it as planning input. Original evidence, the v2 profile and its
provenance are backed up under `/tmp/pyramid-recon-modal.SefXQ3Qz/`.

## Measured calibration correction

Both new modal observations fail the previous uniform RGB tolerance 4 in the
same four text/button patches. The panel is exact; the Menu backdrop differs
by at most 3 and the fullscreen backdrop is exact. Visually the wording and
controls remain the specific Undo All confirmation. Differences concentrate
around rendered text, not an absent panel or a different modal target.

| Component | Selected pixels beyond the old limit | Measured maximum delta and new tolerance |
| --- | ---: | ---: |
| Title | 340 | 9 |
| Body text | 489 | 10 |
| Cancel caption | 282 | 9 |
| OK caption/interior | 115 | 8 |

The v3 data correction changes only the profile ID and those four tolerances.
Every ROI, exact source RGB sample, all-pixel mask, native point, other bound
and the seven-part same-frame conjunction remains unchanged. Panel and both
backdrop tolerances stay at 4; the global 24 cap is not raised. There is no
generic OK fallback, per-component alternative, dropped-pixel allowance,
animation retry change or automatic resumption.

This is a real widening of the accepted RGB channel ranges across every selected
pixel of the four patches, not only the previously mismatching text edges.
The measured limits have no additional slack. Two observations of the same modal
are repeatability evidence, not independent coverage of rendering environments.
Tests retain exact-limit/limit-plus-one rejection, all mandatory components,
derived wrong-text/generic-OK cases and other-game native scenes. No native
near-identical non-Undo confirmation was supplied, so these tests are not a
universal false-positive guarantee. Unknown future appearances still refuse.

## Runtime identity and recovery

Profile pathname remains `tests/fixtures/pyramid-recon/native-profile.json`.
Profile ID is `pyramid-native-1920-classic-blue-v3`, schema version 1.
SHA-256: `3dab5de86b93b1c1c156782d508f2b27b4ba2174c6b594781c97fd7ce91e4836`.
Runtime Rust and xSAR code are unchanged; source edits are regression tests.

The profile is loaded before each new operation, not cached for the process
lifetime. An idle existing 2.1.0 process therefore needs no restart solely for
this data correction. A failed operation has revoked its authority and cannot
be resumed by loading a new profile. Charlie must manually dismiss the specific
Undo All confirmation with OK, verify the original tableau/LEFT/empty RIGHT and
start a fresh scan with a new output pathname and renewed confirmation. That
manual restoration does not turn the earlier diagnostic into completed output.
Do not select New Game or the dialog's optional suppression checkbox.

## Validation

Evidence root: `/tmp/pyramid-recon-modal.SefXQ3Qz/`. Cargo commands retain
`--locked --offline --config /tmp/pyramid-recon-build.5qifGKcG/local-xsar.toml`.
`modal-first-failure.log` reproduces the original `Unknown` result with v2.
The focused suite passes 50 tests, with zero failed or ignored. It covers the
original and both new modal observations, all seven component erasures on both
new captures, every component's exact channel boundary and one-beyond refusal,
four labelled synthetic text-shift negatives, generic OK-only refusal and
seven existing Free Cell/Spider/TriPeaks native scenes. These native negatives
are not claimed to be near-identical competing Undo confirmations.

The controller test also requires a valid first modal followed by an unknown
second observation to send no OK. Existing STOP/context, uncertain-delivery
non-replay and two-frame restoration tests remain. A new diagnostic test proves
the 52-card inventory is unique while the incomplete document remains rejected
by xSAR's planning-input reader.

| Check | Actual result | Evidence |
| --- | --- | --- |
| Full application tests, including ignored host tests | 692 passed, 0 failed, 0 ignored; 82.32 seconds | `full-tests.log` |
| Cargo check | Passed | `check.log` |
| All-target Clippy with `-D warnings` | Passed | `clippy.log` |
| Strict private rustdoc | Passed with `RUSTDOCFLAGS=-D warnings` | `doc.log` |
| Native-host release build | Passed; executable checksum unchanged | `release.log` |
| Original/profile audit | 28 original hashes/sizes/dimensions; 14 exact RGB patches; only ID/four tolerances changed | `profile-audit.log` |
| Existing native negative-scene margins | Each of five foreground modal components rejects every tested scene by more than 200 RGB levels beyond tolerance | `native-negative-margins.log` |

The executable remains SHA-256
`84c68c76e9ef518ef36a79e5e036163092394bc9b9ca92b07694ea4f1c51540d`.
The guarded launcher now verifies the v3 profile hash; its previous version is
retained in the correction evidence directory. Live restoration/export still
require acceptance. No formatter, dependency, MSRV, package version, Git index,
installed executable or guest-input change is part of this correction.

## Code Analysis

The only runtime change is the bounded calibration-data acceptance range for
four known modal components. Safe Rust, memory limits, checked coordinates,
capture/input budgets, fresh confirmation, STOP and no-overwrite publication
are unchanged. Broader colour ranges introduce a documented recognition
trade-off; mandatory independent title/body/control/backdrop evidence and
negative tests remain. Acknowledged Undo All is not restoration, and retained
knowledge does not grant a failed run permission to send OK or publish JSON.
