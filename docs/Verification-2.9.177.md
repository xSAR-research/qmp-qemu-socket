# Verification — qmp-qemu-socket2.9.177

## Result and scope

Private refactoring separates original-source disappearance from board-position verification, separates input-free route planning from once-only execution requests, isolates three identity-bound Pyramid completion handlers and centralises common UI STOP revocation. Existing proof order, input permissions, observation budgets, failure outcomes, context binding and RESET correlation are preserved.

All six controller gates passed, including 1697 tests with zero failed, ignored or filtered, plus aligned locked check/release/protected help. The 56 xSAR inputs and package0.9.1 are unchanged; prior sixteen-gate library qualification remains cached. No fresh guest acceptance was performed.

RF08 keeps SourceTransition creation/lifetime and reachable absent/present branches.
One helper observes original-source disappearance; another verifies board position,
then same-action stock effect only after board success. Temporal staging, fatal
frame checks, commit/reject, original first-error retention, immutable challenge
binding and source-proof publication stay caller-owned. Lazy Perfect→normal→
intermediate Boards→target Boards→bound challenge fallback and error provenance remain.

RF06-A replaces the boolean dispatch with explicit planning/execution entrypoints.
Each validates its own prerequisites before reserving run identity; execution
resolves plan ID before consent consumption. Only execution revokes acquisition;
only planning begins planning statistics. Three event bodies move into the private
request module and reuse the existing identity-bound reducers. Context filtering,
RESET suppression/correlation, diagnostics and latest-frame delivery remain in poll_worker.

RF04 shares only the common STOP revocation and disconnect sequence. The ordinary
view still clears only generic owners on dispatch failure; the Pyramid view retains
its owner and typed diagnostic reporting. Exit still uses cooperative shutdown.
No source input, capture, wait, observation, recovery allowance or action count changed.

## Evidence

Focused existing regressions:121 computed-route tests and86 UI lifecycle tests passed.
Independent source reviews compared the moved event bodies token for token and the
observation common suffix byte for byte. No implementation-mirroring wrapper test
was added. [Source diff](../Evidence/Pyramid/Validation/refactor-177-20261104-qu2x3ban/source-diff.patch) and
[review](../Evidence/Pyramid/Validation/refactor-177-20261104-qu2x3ban/independent-review.json) record scope and preservation.

Six canonical nightly/locked/offline native all-feature commands passed:
scoped format, check, strict private rustdoc, Clippy, full1697 tests including ignored,
and release. [Source qualification](../Evidence/Pyramid/Validation/refactor-177-20261104-qu2x3ban/qualification-app.json).
947 frozen inputs; only own manifest/lock176→177 change afterward,945 identical.
[Alignment](../Evidence/Pyramid/Validation/refactor-177-20261104-qu2x3ban/aligned-qualification.json) covers check/release/protected-help.
56 unchanged xSAR inputs retain cached16 gates:303 full/14 default/two Miri.
The user profiles, Wayland features, znver4 and hardening configuration are preserved.

Published app HEAD8ea6d5c559c2b4d6d1c96c1dcf83ed128e778e53 and xSAR
HEADc4a9b839ece3668b8d46ca00676ae05c34a97943 remain unchanged. All four indices,
unrelated changes and prior proof bytes are guarded. Thirteen work-list checkbox
substitutions are the only draft changes; deferred milestones were not marked complete.
[Owned paths](../Evidence/Pyramid/Validation/refactor-177-20261104-qu2x3ban/owned-paths.txt) lists the complete promotion scope.

No new live guest captures, input, installation, staging, commit, push or sibling
edits were performed. Charlie's earlier testing is reported evidence, not fresh
acceptance of177. Targeted live checks remain: Pyramid A* RECON/plan/execute and STOP,
continuous Solver STOP in both views, RESET after STOP, and context-change refusals.

## Executable

```sh
/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/Pyramid/Validation/refactor-177-20261104-qu2x3ban/native/qmp-qemu-socket-2.9.177 --image
```

SHA256 `f061e760dc73d21c63696c0ea6b317f820f4a8a6a1457b2cfb6cb7f795729372`;12566688bytes;mode0700.
