# Native-v8 static scene-anchor evidence

Retained: 19 October 2026. This fixture supports the approved 2.5.104 correction;
required host verification passed; a corrected-build live trial remains pending.
Original v7 and older profiles/PNGs remain byte-unchanged. Source paths, hashes,
transformation labels and limitations are recorded in [provenance.json](provenance.json).

## Exact retained files

| File | Bytes | SHA-256 | Source / purpose |
| --- | ---: | --- | --- |
| [profile.json](profile.json) | 793,900 | `0b459aa78701da2f07f6cf911ce6f78f7190b1432b66706f4f0135ed21f48a3e` | Exact proposed v8 profile from `/home/charlie/tmp/pyramid-selection-diagnosis-q5b9vs7x/profile-v8.proposed.json`; no transformation. |
| [selected-9D.png](selected-9D.png) | 3,752,287 | `2b017883ebc84502838e38922396114ac8363f084144d1ced3078b74a49ff7a7` | Exact later operator screenshot `qmp-qemu-socket 261019 055756 action11-selected-9D.png`; not the exact first controller failure frame. |
| [baseline.png](baseline.png) | 3,770,781 | `bd0f3ad8de437e1e58ec50fc930839e101623773c7fe000a74c78312dda55ae1` | Exact `/home/charlie/tmp/.pyramid-recon-667842-0/latest.png`, original/restored board for the acquired deal. |
| [deal.json](deal.json) | 4,751 | `b5ea639102822aef9a17b586bc926394725d5fc9f1beb56637207be2e26422b3` | Exact `/home/charlie/tmp/pyramid-recon-1792351991833-667842-1.json`, complete acquired inventory used for prior calculation. |

The source directory for the selected PNG is
`/home/charlie/Pictures/Screenshots/`; its complete literal source pathname is
preserved in provenance. The profile reference is unchanged v7
[baseline.png](../native-v7/baseline.png), SHA-256
`f00076f38a2906dc17d1dc5b7c54806b6285f363f936924e5c785e9173edf`.
The parent [v7 profile](../native-v7/profile.json) remains 806,442 bytes,
SHA-256 `9517ce85382ec0c977e735c79871c1e2f8110b7265343dc1140ac45d506e3e08`.
No source screenshot, deal or historical profile is edited to create a pass.

## Selected position and diagnosis

The selected frame contains 9♦ at fixed one-based location `6|3`; original
zero-based tableau indices 23/24 are removed, 26 cards remain, DL is 4♠ and DR
is 8♣. No remaining slot is compacted or renumbered. The restored baseline and
complete deal retain original card identity ownership and stock order.

The prior 2.5.103 route independently replayed 41 actions and verified ten before
refusing at action 11's second-gesture guard. A changing K-chart sample in v7
`scene[0]`, native `[1810,132,78,40]`, has 222/2,355 selected pixels outside
tolerance 6 and maximum RGB differences 172/133/151 in this later frame. The
other checked scene samples pass and selected rank/suit samples are identical.
This supports replacing mutable scene samples, not a glyph/tint relaxation.
The later screenshot does not reconstruct the exact original failing pixels.

## Exact v8 profile change

Only three JSON values differ from v7: `profile_id`, `board.scene[0]` and
`board.scene[1]`. ID is `pyramid-native-1920-classic-blue-v8`.

| Index | Native `[x,y,width,height]` | Mask | Selected pixels | Contrast | Tolerance |
| --- | --- | --- | ---: | ---: | ---: |
| 0 | `[1796,116,109,18]` | Complete rectangle | 1,962 | 96 | 4 |
| 1 | `[1796,388,109,20]` | Complete rectangle | 2,180 | 93 | 4 |
| 2 | `[931,695,60,50]` | Unchanged arrow | 3,000 | Unchanged | 4 |

The first two patches sample static right-border artwork with positive structure
and constrained negative space. All three patches remain mandatory. Card samples,
XOR caps, alignment, paper/colour checks, masks outside these two scene entries,
modal signatures and geometry remain unchanged. Full-modal recognition and exact
position/pile/absence evidence still apply. Solver is not visually gated; explicit
operator confirmation remains required and unavailable Solver is allowed.

## Checks and evidence limits

Retained pure scene measurements cover 32 historical files: 28 boards and four
modals. The historical 104 labels describe card observations, not 104 frames.
Adding this selected frame and restored baseline produces 34 files, 30 boards
and four modals. The measured v8 scene signatures accept those boards and refuse
those modals. These measurements are separate from the focused production Rust results,
full required checks and a corrected-build live trial. Thirteen focused native-v8
tests passed, zero failed/ignored. The complete required rerun passed 942 tests,
zero failed/ignored, and the remaining required host checks. Preserve the first
full 941-pass/one-failure attempt: an old generic-error assertion was corrected
to exact scene index/bounds without production changes. Corrected-build Beast
acceptance remains pending; these 13 tests overlap the 942-test total.

The focused current checks cover exact old-v7 refusal, selected/restored production
recognition and exact route-state comparison, default/file equivalence, exact
scene-index/bounds diagnostics, complete modal refusal, missing/corrupted patch
and tolerance boundaries, and unchanged old profile/card definitions. Actual
results must be read from [Verification-2.5.104.md](../../../../docs/Verification-2.5.104.md),
not inferred from fixture presence or historical passing totals.

This fixture grants no current input authority, session receipt or plan resume.
The interrupted plan is consumed. Use a fresh untouched board and successful
same-session acquisition before Calculate and separately confirmed Execute.
The [operator brief](../../../../docs/Live-Test-2.5.104.md) defines that later trial;
one tableau clear is separate from daily/regular whole-game completion.
