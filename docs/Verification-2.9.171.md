# Verification — 2.9.171 — 30 October 2026

## Authority and scope

Charlie approved tolerance 5 and urgent 171 executable delivery (m04744), then
explicitly approved the audited handoff reconciliation and build (m04777).
The original failed 170 sentinel is preserved as history. No broader game,
recognition, quiet-settling, input, server or sibling change was authorised.

2.9.171 changes the default Pyramid RGB tolerance from 13 to 5 intensity
levels (5/255, about 1.96 percent). A pixel counts only when all three
same-coordinate absolute RGB differences are strictly greater than 5; alpha
is ignored. A difference of 5 is excluded and 6 qualifies. Joint coverage stays
strictly greater than 5.0 percent in the card-relative (3,4), 66-by-89 effect
quarter: 293 of 5,874 eligible pixels refuse and 294 pass. Existing guided cursor
exclusions retain their adjusted eligible denominator.

Live PARAMS still publishes a complete pair frozen per action or paired-D batch;
edits affect the next boundary. Glyph recognition and independent quiet settling
remain 31 by 49. The quiet byte tolerance 12, per-channel 1-percent/mean-2 limits,
two-capture requirement, exact identities, stock order, inventory, restoration,
terminal, STOP and no-replay checks are unchanged. No input, capture or timing
policy changed. User optimisation profiles and CPU/hardening flags are preserved.

The reported 170 Coral 4-spades/4-diamonds refusal was 289/5,874 (4.92 percent)
at tolerance 13. A later different-theme diamond report was 140/5,874 (2.38 percent),
also at 13. Immediate pre-batch pixels are missing for both reports; their actual
tolerance-5 counts and complete Recon/restoration outcomes cannot be reconstructed.
The earlier independent Coral settling stop is not established resolved.

All seven retained picture-card controls pass the tightened default, while their
exact historical counts remain tested explicitly at 13/5. These are aligned
artwork comparisons, not natural temporal transitions. Suitable retained evidence
for four other Coral picture-card suit pairs is still unavailable. Fresh complete
standard/Coral guest Recon, restoration and broader theme acceptance remain pending.

## Implementation and tests

Only production default RGB tolerance and its docstring change in parameters.rs.
Three default-sensitive refusal fixtures now test the strict new byte boundary.
Quarter tests establish 293/294 coverage and 5/6 RGB boundaries, with alpha ignored.
Historical picture counts remain explicitly bound to 13/5; the tightened default
also fully scans all seven controls and preserves their acceptance. A synthetic
RGB transition with deltas 81/6/25 passes 5 and refuses 13; it is not the missing
runtime source. Remaining identity/pile, quietness and boundary-freeze tests remain.
The source diff received independent read-only review with NO FINDINGS.

Focused shared-policy run: 10 passed. All 937 inputs are frozen; after complete
source gates only own manifest/lock version fields align 170 to 171, leaving
935 inputs identical. No dependency resolutions or library inputs changed.

## Host qualification

| Gate | Result | Elapsed |
| --- | --- | --- |
| scoped-format | PASS | 0.15 seconds |
| check | PASS | 3.75 seconds |
| rustdoc | PASS | 2.10 seconds |
| clippy | PASS | 12.49 seconds |
| full-tests | PASS | 149.80 seconds |
| release | PASS | 55.92 seconds |

All six controller gates pass, including **1,675 include-ignored tests**, zero failed, ignored or filtered; test time 149.57 seconds. Aligned locked check, release and protected --help pass. xSAR remains the unchanged modified/unreleased 0.9.1 path dependency; its sixteen-gate receipt (303 full tests, 14 default tests, feature checks and two focused Miri checks) is cached over 56 unchanged library inputs.


Commands use nightly, explicit package, locked/offline native x86_64-unknown-linux-gnu,
all features and include-ignored scope with eight test threads. User release/dev
optimisation profiles and CPU/hardening configuration are preserved byte-for-byte
apart from the own package version field. No unchanged library rebuild was run.

## Protected executable and evidence

Executable: /home/charlie/repo/RUST/qmp-qemu-socket/Evidence/Pyramid/Validation/pyramid-tolerance-171-5r42tdxy/native/qmp-qemu-socket-2.9.171
SHA256: c037258e67041f6d182605fe22afb427df9915a0c06609754959ef206ab0c9a3
Bytes: 13387904; mode 0700. Not installed or launched against guest.
Commands, log hashes, exact baseline/reconciliation, reported170packet snapshot,
alignment and destination-state.json: /home/charlie/repo/RUST/qmp-qemu-socket/Evidence/Pyramid/Validation/pyramid-tolerance-171-5r42tdxy.

## Complete owned paths

- `/home/charlie/repo/RUST/qmp-qemu-socket/CHANGELOG.md`
- `/home/charlie/repo/RUST/qmp-qemu-socket/Cargo.lock`
- `/home/charlie/repo/RUST/qmp-qemu-socket/Cargo.toml`
- `/home/charlie/repo/RUST/qmp-qemu-socket/README.md`
- `/home/charlie/repo/RUST/qmp-qemu-socket/docs/Pyramid-Acceptance-Before-3.0.0.md`
- `/home/charlie/repo/RUST/qmp-qemu-socket/docs/Pyramid-Short-Term-Goal.md`
- `/home/charlie/repo/RUST/qmp-qemu-socket/docs/Verification-2.9.171.md`
- `/home/charlie/repo/RUST/qmp-qemu-socket/docs/development.md`
- `/home/charlie/repo/RUST/qmp-qemu-socket/docs/handoff.md`
- `/home/charlie/repo/RUST/qmp-qemu-socket/docs/milestones.md`
- `/home/charlie/repo/RUST/qmp-qemu-socket/docs/pyramid-execution.md`
- `/home/charlie/repo/RUST/qmp-qemu-socket/src/parameters.rs`
- `/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_effect.rs`
- `/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_effect/quarter_tests.rs`
- `/home/charlie/repo/RUST/qmp-qemu-socket/src/pyramid_recon/vision/removal/tests.rs`

## Code Analysis

The correction changes the default intensity threshold while preserving the
joint RGB predicate, coverage, quarter sampling, live pair snapshots and semantic
checks. Quieter or similar artwork is not proof of complete expected state.
Fresh guest validation is pending, and the independent settling stop is not
established resolved. Colour coverage alone never grants input authority.
