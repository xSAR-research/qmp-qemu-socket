# Verification — 2.9.174

2.9.174 corrects native Boards challenge-header recognition using the union of
the five existing validated Boards literal ink supports, restricted to independently
proven complete word spans. All reference geometries and lengths are checked before
indexing. RGB tolerance 9, blue floor 6, word distance, letter cores, banner anchors,
boundaries and counter parsing remain unchanged. No new reference bank or dilation
is introduced. Score/Card recognition and input authority are unchanged.

## Qualification

The six controller gates passed, including 1687 tests with zero failures,
ignored or filtered tests. Qualification sealed 942 inputs; after qualification only
the application's manifest/lock version pair changed from 2.9.173 to 2.9.174,
leaving 940 inputs identical. Aligned locked check, release and protected help passed.
The 56 unchanged xSAR inputs retain cached sixteen-gate qualification (303 full
tests, 14 default tests and two Miri tests); xSAR remains modified/unreleased 0.9.1.
User profiles and CPU/hardening configuration are preserved. Evidence: `/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/Pyramid/Validation/pyramid-challenge-174-proposal-9yca399e`.

Four native_174 regressions pass: the exact retained Boards 0/3 frame, five true
background negatives including the adjacent (877,63) pixel, all thirteen letter
cores plus banner/counter negatives, and the production v23 Native::observe to
paired_entry Challenge classification. The byte-original PNG is preserved with
fixture provenance. Existing complete-word proof precedes reference support union;
neighbouring negative space remains subject to the original background check.

Source qualification input seal: 10c8488a88501ad37b646d056cd212c2ae089a520cd0771db3c82ed037b44777.
Full suite: 1687 tests, 147.55 seconds. Six gate and three alignment
command argv/environment/exit/log hashes are retained in qualification-app.json and
aligned-qualification.json. The library receipt is cached, not a new library run.
Frozen source review and applied-byte review found no production issues. Test-only
formatting differences changed no tokens or assertions. The accidental pre-application
filtered compile is retained separately and is not qualification evidence.

## Protected executable

```sh
/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/Pyramid/Validation/pyramid-challenge-174-proposal-9yca399e/native/qmp-qemu-socket-2.9.174 --image
```

SHA256: 37cb60771f807c6adff064a5aa0fd2de37368a5adbae4734b7924a75113e2d96. Size: 13404872 bytes. Mode: 0700.
Protected --help returned before image cleanup or GUI startup. No installation or
live guest action was performed. Full fresh guest acceptance remains pending.

## Preservation

Charlie explicitly accepted the automatic post-173 metadata updates and
continuing 174. baseline-state.json preserves the accepted two changed sealed files
(Tree Ring SQLite and TriggerTree read telemetry), eight recall receipts and SQLite
sidecars. All 939 prior compiler inputs, 17 documents and four repository histories,
indices and tracked diffs matched before application. The prior 173 sentinel's
HANDOFF REGRESSION remains historical. No metadata was deleted or broadly ignored.
The two transient SQLite sidecars subsequently became absent; the main store and
all eight receipt hashes remain exact. The last observed WAL was empty. The exact
absence and unchanged store are recorded in sqlite-sidecar-audit.json and sealed
by the new checkpoint; their disappearance cause is not established.

All four repository histories, indices and unowned changes are checked exactly.
Old evidence is retained; docs/handoff.md names this delivery's exact destination
sentinel. The checkpoint captures source, documents, evidence hashes and Git state.

## Owned paths

- CHANGELOG.md
- Cargo.lock
- Cargo.toml
- README.md
- docs/Pyramid-Acceptance-Before-3.0.0.md
- docs/Pyramid-Short-Term-Goal.md
- docs/Verification-2.9.174.md
- docs/development.md
- docs/handoff.md
- docs/milestones.md
- docs/pyramid-execution.md
- src/pyramid_progress/challenge/cleared.rs
- src/pyramid_progress/challenge/cleared/tests.rs
- src/worker/pyramid_session/native_tests.rs
- tests/fixtures/pyramid-entry-174/README.md
- tests/fixtures/pyramid-entry-174/boards-zero-of-three.png
- tests/fixtures/pyramid-entry-174/calibration.json

## Code Analysis

The correction belongs to the native challenge-header classifier. Existing complete
word and banner proofs constrain where literal ink support can apply. Checked
geometry/length validation precedes byte access across the existing Boards bank.
The change admits the already-bundled faint letter fringe while preserving RGB
thresholds and negative-space checks; it grants no new input or completion authority.
