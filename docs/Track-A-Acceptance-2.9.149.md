# Track A acceptance — 2.9.149 / xSAR 0.9.0

## Decision and scope

**CLOSED — ACCEPTED WITH OPEN FIXES.** Charlie accepted the outstanding Track A
items and requested the newly discovered failures be carried into the FIX list
in [milestones.md](milestones.md#7-fix-list--carried-forward-from-accepted-track-a)
(instruction m09592, 2026-10-25 UTC).

This closes Track A of [Codex-Goal.md](Codex-Goal.md): process provenance,
the [147 checks](Live-Test-2.9.147.md), the [148 layout checks](Live-Test-2.9.148.md),
the acceptance receipt and explicit promotion drafts. It does not mark the open
fixes as repaired or select held Track B implementation. The proposed 150 footer
correction remains unapproved. Application/xSAR versions remain 2.9.149/0.9.0.

## Evidence and acceptance are separate

| Track A requirement | Evidence retained and closeout decision |
| :--- | :--- |
| Running process and executable | Protected 149 running bytes and session verified against SHA-256 `b1936a79c72c923023eacfc031858811179b5da9e6114c28ed18ec42700ef1d6`; process identity is a dated observation, not a promise that the same PID remains live. |
| Normal GAME WIN and fresh-board handoff | Charlie accepts the baseline with FIX-01/FIX-03/FIX-05 carried forward. The saved normal trial stopped on board two; it does not prove a successful complete normal reward/OK/New Game/Play cycle. |
| Challenge identity and numeric progress | Successful Boards0/1 target completion and Card/Score next-board Recon observations remain logged. Acceptance includes the outstanding variants without inventing additional captured executions. |
| Explicit Shortest-to-Maximum switch and pause variants | Host tests/source and prior review are retained; automatic exhaustive fallback in the saved session is not relabelled as an explicit-button trial. Charlie's outstanding-item acceptance supplies the acceptance decision. |
| Direct completed Recon | Saved completion-to-planning timestamps and qualified source remain evidence. The 1000 ms settle and two restoration observations remain inside Recon; no fresh timing measurement is inferred. |
| 148 layout and controls | All nine application UI source bindings are unchanged in 149; all 14 reviewed inert capture hashes match. Charlie accepts the outstanding presentation items; cached renders remain cached evidence. |
| Recognition and refusal boundaries | 149 retains 1483 passing tests, six gates and 753 input seals. Failed REDEAL/entry cases are FIX-02/FIX-04. Bounded Western calibration does not establish arbitrary-font support. |
| Promotion readiness | Complete application/xSAR path lists and commit title/body drafts are prepared for review; no staging, signing or publication occurred. |

The original evidence bundle and 22-item requirements matrix remain unchanged at:
`/home/charlie/Documents/Pyramid/Validation/track-a-acceptance-20261026-z69tqa5a`.
It contains the saved session, packets, references, provenance and the follow-up
source comparison. Its earlier pending/partial labels describe evidence at that
snapshot; this document records the later human acceptance decision.

## Closeout artifacts and verification

The closeout bundle is:
`/home/charlie/Documents/Pyramid/Validation/track-a-closeout-20261025-_rpecn_l`.

It contains the before/after repository state, documentation backups,
`acceptance-matrix.json`, `promotion/app-paths.txt`, `promotion/xsar-paths.txt`,
both commit-message drafts, the documentation/input audit, and the current
`destination-sentinel.py` referenced by [handoff.md](handoff.md).

Host qualification remains **PASSED (Cached)**: application 1483 tests / 753 seals;
xSAR 296 tests / 52 unchanged inputs and prior Miri evidence. Closeout checks are
documentation links, exact scoped changes, unchanged compiler inputs, preserved
indexes/sibling state, artifact hashes and the refreshed Destination Sentinel.
No new Cargo run, version bump, executable launch or guest input is needed.

M12 pacing, M13 naming and the wider master schedule keep their separate scope.
The original 19 case identities and 14 schedule rows remain intact; known failures
stay open in the central FIX list. Any later correction receives its own proposal,
verification and acceptance.
