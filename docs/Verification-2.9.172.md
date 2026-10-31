# Verification — 2.9.172

2.9.172 verifies the expected original card at its exact source location before
sending the logical action once. Two successive fresh supported observations must
then stop identifying that same source card. A different recognised card or an
undetermined rank/suit (`None`) qualifies; structural reader/layout errors do not.
An original card reappearing or an unsupported observation resets the pair.

RGB coverage, replacement/ancestor identity matching and pixel quietness no longer
veto MOVE/MATCH/KING action acceptance. The next action independently identifies
its own expected source on the retained frame before input. The existing capture
budgets, probes, STOP and no-replay rules remain. RECON uses two supported source
identity checks within its existing observation loop, while ordered identities,
unique inventory, exhaustion, Undo All, confirmation and restored-board publication
remain separate requirements. Final-clear, terminal and next-board claims retain
their semantic checks on the same captured frames; source disappearance alone is
not reported as an observed board clear.

REDEAL retains its separate appearance policy. PARAMS labels its RGB controls for
that scope. Glyph recognition, package dependencies, user Cargo profiles, host CPU
and hardening configuration are preserved. xSAR remains the unchanged modified,
unreleased 0.9.1 path dependency. No guest operation, new capture, installation,
staging, commit or push was performed. Fresh guest acceptance remains pending.

## Host qualification

All six controller gates pass, including 1678 include-ignored tests with zero failures/ignored/filtered (154.10 seconds). Aligned locked check, native release and protected --help pass. Library qualification is PASSED (Cached): 56 unchanged inputs, 303 full tests, 14 default tests and two focused Miri checks.

938 frozen paired compiler inputs; only the two own version files change after
full qualification, leaving 936 inputs identical. Source and aligned seals,
commands, exits, log hashes and protected binary are retained in /home/charlie/repo/RUST/qmp-qemu-socket/Evidence/Pyramid/Validation/pyramid-source-172-o9et1qlz.

Executable: /home/charlie/repo/RUST/qmp-qemu-socket/Evidence/Pyramid/Validation/pyramid-source-172-o9et1qlz/native/qmp-qemu-socket-2.9.172
SHA256: `59097363173a592be47f85e22ff0ca7415089c3ddba8fc30858cb9e7d7b68d80`

## Evidence limits

Retained frame derivatives and local QMP peers establish offline recognition,
source-disappearance and capture/input cadence. Synthetic header/glyph derivatives
are explicitly labelled and are not immediate runtime pre-action evidence.
No fresh guest effect, complete Coral RECON, restoration or theme-wide acceptance
is claimed. The historical 171 discrepancy is not relabelled as a pass.

## Owned paths

- CHANGELOG.md
- Cargo.lock
- Cargo.toml
- README.md
- docs/Pyramid-Acceptance-Before-3.0.0.md
- docs/Pyramid-Short-Term-Goal.md
- docs/Verification-2.9.172.md
- docs/development.md
- docs/handoff.md
- docs/milestones.md
- docs/pyramid-execution.md
- src/app.rs
- src/pyramid.rs
- src/pyramid_effect.rs
- src/pyramid_recon.rs
- src/pyramid_recon/paired_tests.rs
- src/pyramid_recon/vision.rs
- src/pyramid_recon/vision/redeal.rs
- src/pyramid_recon/vision/removal.rs
- src/pyramid_recon/vision/source_transition.rs
- src/worker.rs
- src/worker/computed_route.rs
- src/worker/computed_route/series_guard_tests.rs
- src/worker/computed_route/series_guard_tests/cadence_tests.rs
- src/worker/computed_route/wire_tests.rs
- src/worker/computed_route/wire_tests/redeal_tests.rs
- src/worker/computed_route/wire_tests/stock_empty_tests.rs
- src/worker/pyramid_execution.rs
- src/worker/pyramid_reconnaissance.rs
- src/worker/pyramid_reconnaissance/post_confirmation_tests.rs
- src/worker/pyramid_reconnaissance/starting_guard_tests.rs

## Code Analysis

The application owns source selection and recognition, the two fresh identity
checks, consumed action permits, input probes and STOP. Existing xSAR APIs and
feature contracts are unchanged. No additional capture or retry mechanism was
introduced. Reader `None` has the explicitly approved limited meaning of not
identifying the old source; it does not prove an empty stock pile or a restored
inventory. Completion and next-source authority remain separately checked.
