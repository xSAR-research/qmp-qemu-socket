# Verification — qmp-qemu-socket 2.9.167 / xSAR 0.9.1

Qualified 29 October 2026. Charlie approved temporal rank/suit removal
verification in m02218, a reusable equal-length packed u32-slice API in m02267,
the safe plain loop in m02271, xSAR 0.9.1 in m02281, and only one outer
whole-call timer in m02303. Root was the sole implementation/review owner;
no 167 subagent was used.

## Behaviour

Computed-route execution now samples the existing 31 by 49 rank/suit corner
at face-relative (3,4) from the fresh verified occupied source. Each RGB channel
is compared independently using absolute differences: a byte changes only when
its difference is greater than 12. At least two channels must each change at
least 30 percent of the 1,519 pixels and have mean absolute difference at least
16. Both lighter and darker changes count; alpha is zero and ignored by policy.

Two fresh supported after-captures must show the expected removal. Their corner
appearances must agree within one percent changed pixels and mean difference
two in every RGB channel. The existing two-second ordinary effect window,
settles, capture transport and input sequence are unchanged. A currently
recognised card vetoes removal. Full scene, remaining-card identities, both
piles and terminal/header checks must also pass before history is committed.

The private original-slot history supplies absence for every already removed
slot and for final-clear paths. A static background cannot create that history.
This replaces background exceptions in computed-route removal verification
across themes, including their old special-case locations. Static acquisition,
read-only/historical readers and separate Solver-led policies retain their
existing recognition rules. No new background crop or tolerance was added.

## Actual verification

All checks used the repository nightly toolchain, explicit owning package,
locked offline resolution and native x86_64-unknown-linux-gnu target.
Full tests include ignored tests; the controller used eight test threads.
xSAR checks cover all seven individual optional features and their combined
configuration. Its default remains dependency-free.

| Owner | Gate | Result | Elapsed |
| --- | --- | --- | --- |
| controller | scoped-format | PASS | 0.43 s |
| controller | check | PASS | 11.62 s |
| controller | rustdoc | PASS | 1.42 s |
| controller | clippy | PASS | 12.12 s |
| controller | full-tests | PASS | 646.66 s |
| controller | release | PASS | 46.51 s |
| xSAR | check-default | PASS | 0.12 s |
| xSAR | tests-default | PASS | 0.40 s |
| xSAR | check-feature-qmp | PASS | 3.02 s |
| xSAR | check-feature-png | PASS | 4.02 s |
| xSAR | check-feature-image_matching | PASS | 0.22 s |
| xSAR | check-feature-pyramid_json | PASS | 3.97 s |
| xSAR | check-feature-pyramid_search | PASS | 0.48 s |
| xSAR | check-feature-execution | PASS | 0.22 s |
| xSAR | check-feature-tcp_capture | PASS | 0.28 s |
| xSAR | check-all | PASS | 2.91 s |
| xSAR | rustdoc | PASS | 0.32 s |
| xSAR | clippy | PASS | 1.29 s |
| xSAR | full-tests | PASS | 16.78 s |
| xSAR | release | PASS | 10.61 s |
| xSAR | miri-packed_bytes_measure_both_directions_and_every_position | PASS | 25.56 s |
| xSAR | miri-packed_bytes_empty_lengths_and_tolerance_boundaries | PASS | 7.47 s |

Controller full suite: 1,660 passed, zero failed/ignored/filtered, 646.43 s test
time. xSAR full suite: 299 passed across 14 test summaries, zero failed/ignored;
default suite: 14 passed. Two new focused Miri tests pass; the older three Miri
results remain retained evidence. Eight new temporal tests, nine focused native
cadence tests and three new xSAR tests pass. The byte-pair regression checks
all 256 by 256 input values.

First focused compile/test failures and the deliberately interrupted serial
adapter run are retained, not called passes. Native fixtures starting mid-route
seed explicit test-only verified history; full driver/cadence tests build
history through actual before/after transitions. Three provisional captures
were reduced to two fresh settled appearances within the unchanged two-second
effect budget. No timing window was extended.

Both aligned locked checks and release builds pass after the approved version
increments. Only four manifest/lock inputs changed among 905 qualified inputs;
901 other inputs remain byte-identical. The release JSON identifies application
2.9.167 and xSAR 0.9.1. A packaging helper incorrectly expected a version in
usage-only --help. Its failed assertion and partial receipt are preserved;
artifact IDs and exact binary hashes finished the receipt without repeating
builds. Protected --help exits zero before image cleanup/GUI.

## Protected executable and reproducible evidence

[qmp-qemu-socket 2.9.167](/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/Pyramid/Validation/pyramid-temporal-167-0xd_0f1s/native/qmp-qemu-socket-2.9.167)

SHA256: 4953ced55c93b440c17b870f743f524a0282dd67d748149d07064f231d198417

Size: 16378648 bytes; mode 0700; not installed.

Private case: /home/charlie/repo/RUST/qmp-qemu-socket/Evidence/Pyramid/Validation/pyramid-temporal-167-0xd_0f1s

qualification-inputs.json seals the source qualification; qualification-app.json
and qualification-xsar.json bind every command, exit, duration and log hash.
aligned-inputs.json and aligned-qualification.json bind approved versions and
the protected binary. The measured outer-only comparator example is in
comparator-whole-call-timing.log. Its one 7.314 us observation is not a latency
guarantee. Retained actual image/deal/error-state provenance is in the new
fixture directory; an initial board used for offline corner comparison is
explicitly not called the immediate pre-action frame.

## Fresh acceptance matrix

| Theme | Required fresh regression | Evidence boundary |
| --- | --- | --- |
| Coral / Underwater | Former bottom-row failures, exposed row six, rechecks of earlier removals, King and pair | Retained Coral before/after corner and full-position tests pass; fresh run pending |
| Xbox | Cards over the bright logo, followed by exposed higher rows | Retained entry/failure recognition is covered; no authenticated removal pair or fresh temporal pass claimed |
| Wild West | Aged thin-looking cards, King, red/black pair, later exposed row | Retained two-frame rank/suit recognition remains covered; these are not a removal pair |
| Classic Blue | Ordinary removals through normal final clear | Standard-theme live control run pending |
| Cream and Retro | Short King/pair and later-row checks | Other retained appearance controls; fresh temporal checks pending |

Include normal final clear and challenge completion, unchanged/selected cards,
small visual changes, moving or partial cards, and STOP/context revocation.
Record failures without replay. These checks do not require every theme/mode
combination. No further Coral screenshots were requested before offline testing;
a genuine Xbox or Wild West occupied/removed pair would extend offline evidence.

Fresh guest acceptance and future rendering coverage remain pending.
The previously retracted normal Continuous acceptance stays open. RECON's
acknowledged D with no observed stock advancement is separate and unresolved;
167 does not claim a fix. Route-objective policy is unchanged from 164. No
King/scoring/projection, Single-board withholding or Free Cell change is included.
No guest input, new capture, installation, staging, commit, push or publication
was performed for this delivery. Preserve GAME WIN images before another
controller startup and leave any active original session log in place.

## Complete owned paths

These 42 paths belong to this paired change relative to the preserved baseline.
Ignored compiler fixtures and font assets sealed by the previous checkpoint
are unchanged and are not promotion paths.

- /home/charlie/repo/RUST/qmp-qemu-socket/CHANGELOG.md
- /home/charlie/repo/RUST/qmp-qemu-socket/Cargo.lock
- /home/charlie/repo/RUST/qmp-qemu-socket/Cargo.toml
- /home/charlie/repo/RUST/qmp-qemu-socket/README.md
- /home/charlie/repo/RUST/qmp-qemu-socket/docs/Pyramid-Acceptance-Before-3.0.0.md
- /home/charlie/repo/RUST/qmp-qemu-socket/docs/Pyramid-Short-Term-Goal.md
- /home/charlie/repo/RUST/qmp-qemu-socket/docs/Verification-2.9.167.md
- /home/charlie/repo/RUST/qmp-qemu-socket/docs/development.md
- /home/charlie/repo/RUST/qmp-qemu-socket/docs/handoff.md
- /home/charlie/repo/RUST/qmp-qemu-socket/docs/milestones.md
- /home/charlie/repo/RUST/qmp-qemu-socket/docs/pyramid-execution.md
- /home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_recon/vision.rs
- /home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_recon/vision/final_clear.rs
- /home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_recon/vision/foreground_card.rs
- /home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_recon/vision/normal_clear.rs
- /home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_recon/vision/removal.rs
- /home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_recon/vision/removal/tests.rs
- /home/charlie/repo/RUST/qmp-qemu-socket/src/worker/computed_route.rs
- /home/charlie/repo/RUST/qmp-qemu-socket/src/worker/computed_route/continuation.rs
- /home/charlie/repo/RUST/qmp-qemu-socket/src/worker/computed_route/series_guard_tests.rs
- /home/charlie/repo/RUST/qmp-qemu-socket/src/worker/computed_route/series_guard_tests/board_count_tests.rs
- /home/charlie/repo/RUST/qmp-qemu-socket/src/worker/computed_route/series_guard_tests/cadence_tests.rs
- /home/charlie/repo/RUST/qmp-qemu-socket/src/worker/computed_route/series_guard_tests/effect_155.rs
- /home/charlie/repo/RUST/qmp-qemu-socket/src/worker/computed_route/series_guard_tests/exhausted_left_tests.rs
- /home/charlie/repo/RUST/qmp-qemu-socket/src/worker/computed_route/series_guard_tests/native_123_tests.rs
- /home/charlie/repo/RUST/qmp-qemu-socket/src/worker/computed_route/series_guard_tests/target_completion_tests.rs
- /home/charlie/repo/RUST/qmp-qemu-socket/src/worker/computed_route/wire_tests.rs
- /home/charlie/repo/RUST/qmp-qemu-socket/src/worker/computed_route/wire_tests/redeal_tests.rs
- /home/charlie/repo/RUST/qmp-qemu-socket/src/worker/computed_route/wire_tests/stock_empty_tests.rs
- /home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recognition-167/deal.json
- /home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recognition-167/error-state.json
- /home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recognition-167/initial.png
- /home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recognition-167/last-observed.png
- /home/charlie/repo/RUST/qmp-qemu-socket/tests/fixtures/pyramid-recognition-167/provenance.json
- /home/charlie/repo/RUST/xsar/CHANGELOG.md
- /home/charlie/repo/RUST/xsar/Cargo.lock
- /home/charlie/repo/RUST/xsar/Cargo.toml
- /home/charlie/repo/RUST/xsar/README.md
- /home/charlie/repo/RUST/xsar/docs/milestones.md
- /home/charlie/repo/RUST/xsar/examples/packed-byte-comparison.rs
- /home/charlie/repo/RUST/xsar/src/image_matching.rs
- /home/charlie/repo/RUST/xsar/src/image_matching/packed_comparison.rs

## Code Analysis

The xSAR loop uses checked lengths and bounded u64 accumulators, independent
to_le_bytes positions and no unsafe code, allocation or packed-word subtraction.
RGB decisions exclude the zero alpha position. Native corner reads use checked
frame layout and geometry. The removal mask is private, bound to original
slots and expected deletion-only transitions, with no resurrection or identity
replacement. Stale capture serials and recognised surviving cards refuse
progress. Failed scene/identity/pile/terminal checks cannot commit provisional
history. Uncertain input is not replayed. The numeric decision thresholds
are qualified by the retained/synthetic tests, not a guarantee for unseen
rendering, every partial-card animation or every theme.

## Preservation audit

The first closeout helper stopped at one trigger-tree session telemetry change.
Its files/scans counters and all 35 prior recent events are unchanged; one read
of AGENTS.md at 11:06:15 UTC was appended by the tool hook. telemetry-delta.json
binds its exact old/new hashes and event. The file was preserved, not edited by
the agent or included in promotion paths. The owned-path derivation was also
corrected to consult both the baseline Git copies and previous compiler seals;
unchanged ignored fixtures/fonts are excluded. No source or checks changed.
