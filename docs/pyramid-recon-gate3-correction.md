# Pyramid Gate 3 scene-profile correction

Date: 11 October 2026, Australia/Brisbane. Application package remains 2.1.0.

## Observed failures

Charlie supplied two actual worker diagnostic directories:
`.pyramid-recon-2880038-0` and `.pyramid-recon-2880038-2`, under his private
temporary directory. Each reports one capture, zero D/Undo All/OK attempts,
`Unknown native scene; input authority revoked`, incomplete inventory and
`publication_status=not_attempted`. Neither is successful reconnaissance or
complete planning-input JSON. Original diagnostics and the previous profile are
also backed up under `/tmp/pyramid-recon-gate3.jaJUvBhY/`.

The first frame shows an exhausted LEFT pile and RIGHT Queen of diamonds, so
it is not a valid initial state. The second shows the original complete tableau,
LEFT King of diamonds, empty RIGHT, disabled Undo All and Solver off. The second
failure therefore cannot be explained by starting with an exhausted pile.

Exact original PNGs are retained in the fixture manifest:

| Fixture | SHA-256 |
| --- | --- |
| `gate3-exhausted-refusal.png` | `9c3e3cde56674181f196fc42340927e02ab2a00c1050021e99d5bc09e2ad8c65` |
| `gate3-start-refusal.png` | `f6043267d22cbf3731df684bb5aefec25a2b6714b80a0c19b1d214f7d8e83baa` |

## Cause and bounded correction

Both frames exceed two chart-signature crops' RGB tolerances in exactly 16 and
one selected pixels, respectively, with maximum channel difference 10. Every
offending pixel lies in a fragment of text from the next chart row. This is a
calibration crop-boundary problem, not absent Solver-OFF or stock-arrow evidence.
The first crop's full separator occupies rows 169 and 170; the second occupies
rows 321 and 322. Those separators remain inside the corrected crops.

| Patch | Previous bounds | Corrected bounds | Retained pixels | Unchanged tolerance |
| --- | --- | --- | ---: | ---: |
| Complete K entry and separator | `[1810,132,78,50]` | `[1810,132,78,40]` | 3120 | 6 |
| Complete 9/4 entry and separator | `[1815,281,67,54]` | `[1815,281,67,42]` | 2814 | 8 |

The corrected RGB samples are unchanged pixel crops from the already retained
`current-start.png`, with every retained pixel selected. Neither threshold nor
glyph matching is relaxed. Spatial coverage is narrower: changes solely within
the excluded next-row strips are now intentionally irrelevant. Tests separately
erase K, 9 and 4, check retained edge-pixel channel boundaries and mutate the
excluded strips. The other scene patch, positive Solver-OFF, complete modal,
card/pile recognition and controller authority checks remain mandatory.

Both new captures now have maximum channel differences 6/7 in the corrected
crops. No runtime Rust logic, xSAR API, dependency, input policy, game controller
or package version changes. Rust edits add regression tests only. The actual
exhausted frame is recognised but still refused by the controller before input
or publication; no automatic restoration is authorised for an invalid start.

## Profile identity and operation

Path: `tests/fixtures/pyramid-recon/native-profile.json`.
Profile ID: `pyramid-native-1920-classic-blue-v2`; JSON schema remains 1.
SHA-256: `cd8a04cf5a2878fd7e6be5ecaa86cb472f9e2396a02ecb5b2f47d7423f5252cb`.
The previous profile and historical delivery manifest remain in the evidence
directories; new evidence must not be interpreted as the original delivery hash.

`run_checked` loads the selected profile before each operation. An idle existing
2.1.0 process therefore uses the corrected file on its next deliberate request;
the profile change does not update an already-running acquisition. Inspect the
calibration-loaded log for the new profile ID. No live retry, guest input,
process launch/termination, installation, Git staging or promotion is performed
by this correction.

## Validation

Evidence: `/tmp/pyramid-recon-gate3.jaJUvBhY/`. All Cargo commands use `--locked
--offline --config /tmp/pyramid-recon-build.5qifGKcG/local-xsar.toml` and the
unchanged local xSAR source. The first regression reproduced `Unknown` with the
previous profile; its failing output remains `native-first-failure.log`.
Focused reconnaissance tests pass: 46 passed, zero failed or ignored, including
the original captures, semantic-crop adverse cases and actual exhausted-start
zero-input/no-publication refusal.

| Check | Actual result | Evidence |
| --- | --- | --- |
| Full application tests, including ignored host tests | 688 passed, 0 failed, 0 ignored; 81.92 seconds | `full-tests.log` |
| Cargo check | Passed | `check.log` |
| All-target Clippy with `-D warnings` | Passed | `clippy.log` |
| Strict private rustdoc | Passed with `RUSTDOCFLAGS=-D warnings` | `doc.log` |
| Explicit native-host release build | Passed; executable SHA-256 unchanged | `release.log` |
| Original/profile provenance audit | 25 original hashes/sizes/dimensions and 14 exact RGB patches verified; only profile ID and two intended crops changed | `profile-audit.log` |

The executable remains SHA-256
`84c68c76e9ef518ef36a79e5e036163092394bc9b9ca92b07694ea4f1c51540d`.
The existing guarded launcher now checks the corrected profile hash, with its
previous version retained in the correction evidence directory. Live 24-card
acquisition and restoration remain unverified and require Charlie's next
controlled acceptance run.

## Code Analysis

The correction changes two bounded data crops, not runtime input authority.
Memory limits, checked geometry, safe Rust, channel caps, all selected-pixel
matching, STOP/context revocation, no uncertain replay and atomic no-overwrite
publication are unchanged. Narrower spatial coverage is an explicit trade-off,
covered by retained-glyph and boundary tests; it is not proof against every
unsupported scene. Unknown future rendering or glyphs still stop safely. A
successful offline classifier result cannot establish live stock order or
restoration and must not be reported as a complete deal export.
