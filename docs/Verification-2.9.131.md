# Verification — 2.9.131

App **2.9.131** passed **1347 tests, 0 failed, 0 ignored, 0 filtered** in the complete native include-ignored suite (411.666s command time). Locked all-target/all-feature check, strict private rustdoc, Clippy, release, scoped formatting and native/protected help passed. xSAR **0.8.104** and console **0.8.4** are unchanged; 18 shared/feature/private-consumer checks are retained by exact input/log hashes (289 shared, 80 console; features 14/55/25/67/57/99).

**Accepted working:** Charlie confirmed the stop condition **Continuous — no challenge** in m13393. The [operator acceptance record](</home/charlie/Documents/Pyramid/Validation/explicit-recon-recovery-20261022/operator-acceptance.md>) is separate from host qualification. The new recovery change and the remaining detailed cases still require fresh verification; the entire activated goal is incomplete.

Optional Recon was displayed in Single board and Continuous, but its handler silently returned for every non-Manual mode. The corrected handler permits Recon in every idle Pyramid workflow mode. Manual Plan/Solve remain mode-restricted, and busy, STOP, context and ownership checks remain.

Each explicit Start single board or target-agnostic Start continuous now requests a fresh read-only frame within the existing worker reservation, before inspecting a retained acquisition. Other automatic entry paths already capture fresh frames. No separate UI capture command is chained to Start. The new frame supplies current pixels; it cannot authenticate an old route, grant input, or bypass supported-scene, restoration or effect verification. A valid same-board Recon can still be reused across settings/planning changes.

No separate stale-preview Start latch was demonstrated. Matching failed completion already retires the run, route and receipt, and Uncertain is idle. The concrete handler defect and the requested explicit-start capture are covered by regressions. Unsupported dialogs/scenes continue to refuse; fresh pixels do not dismiss them.

Existing stock-boundary proofs, Kings-only advisory defaults, target-aware completion, NoChallenge target opt-out, same-inventory fallback, three-pass stock accounting, guarded single REDEAL, bounded next-board settling, DL-first paired feedback and operator-controlled search are preserved. Variable backgrounds and idle CPU remain separate backlog.

## Qualification provenance

Three new regressions exercise failed-session completion and all-mode optional Recon eligibility; actual AF_UNIX capture replacing seeded stale preview/adapter pixels after a retired reservation; and STOP/context revocation during that capture. Native servers accept capture traffic and reject guest operations. Both Single board and NoChallenge entry paths require exactly one fresh capture in this seam. These are isolated host tests, not new Beast gameplay. The retained native failure packets are from v2.9.129; they document earlier failures, while the handler root cause is established by source.

[Commands/input maps/log hashes](</home/charlie/Documents/Pyramid/Validation/explicit-recon-recovery-20261022/qualification.json>) · [Protected artifact](</home/charlie/Documents/Pyramid/Validation/explicit-recon-recovery-20261022/protected-artifact.json>) · [Relevant original failure packets/log prefix](</home/charlie/Documents/Pyramid/Validation/explicit-recon-recovery-20261022/evidence-manifest.json>) · [Operator acceptance](</home/charlie/Documents/Pyramid/Validation/explicit-recon-recovery-20261022/operator-acceptance.md>) · [Final audit](</home/charlie/Documents/Pyramid/Validation/explicit-recon-recovery-20261022/final-audit.json>).

## Code Analysis

The fix reuses the existing typed request/reservation and capture path; it adds no independent UI-to-worker capture race. Recon mode eligibility is widened only for Recon, with Plan/Solve still Manual-only. The read-only entry frame is bounded and remains advisory until existing acquisition/effect proofs. STOP/context checks run before and after capture; failures retire authority without retry or cleanup input. No dependencies, unsafe Rust, calibration/threshold changes or shared-crate edits.
