# Free Cell challenge planning — approved 3.1.5 / xSAR 0.12.0

Charlie approved application 3.1.5 and xSAR 0.12.0, together with the required
application and basic-route-planning lockfile updates. Earlier evidence remains
historical. Patch increments need no separate approval; major/minor transitions
still require approval under Charlie's 9 November clarification.

Free Cell has no multi-board challenge continuation. For an ordinary board or a
challenge to solve, score, remove a particular card or remove cards of one rank:

1. Try a complete route to put all 52 cards on foundations first.
2. If the unlimited search exhausts its reachable graph without a complete route,
   return a route attaining the maximum reachable foundation count. Prefer lower
   deliberate cost and then fewer steps among equal scored explored endpoints.
3. Independently replay the complete or partial physical route before making it
   available. Cancellation, resource failure and replay failure expose no prefix
   and never prove a maximum. A zero-improvement result must be labelled honestly.
4. On successful, currently accepted RECON completion, select its exact published
   JSON automatically for SOLVE and retire the previous source/plan/browser state.
5. Permit acquired challenge information to remain opaque: saving banner pixels
   does not require semantic OCR before trying the route above. Preserve the
   original banner information and exact saved input.
6. Support the native highlighted challenge board in live initial binding and
   subsequent position proofs using positive measured card geometry and actual
   competing printed identities. Preserve independent scene/Solver/modal,
   inventory, ambiguity, STOP, fresh capture and explicit transfer/effect checks.
7. A positively recognised Challenge Complete event on the bound Free Cell run
   ends execution immediately. It does not prove all 52 cards were removed or
   justify advancing an unverified cursor. Retire the remaining route; leave
   Continue/restart/new-game controls to the operator.

Keep result types and UI messages distinct for full clearance, proved maximum
partial route, challenge completion, cancellation and failure. Partial exhaustion
must not become a resumable empty route or be reported as a game win.

Run relevant narrow tests, the xSAR default/isolated-feature/full matrices and
real consumer integration, plus the repository's locked complete tests including
ignored tests, strict private rustdoc, Clippy and release builds. Bind all results
and the delivered executable to exact inputs and toolchain; preserve first
failures, earlier incidents and all four Git histories/indices.

No installation, application launch, guest input, staging, commit, push or
publication is authorised by this build. Report retained/synthetic/local-peer
evidence separately from fresh guest/operator acceptance.
