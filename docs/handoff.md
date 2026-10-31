# Active handoff — qualified 2.9.174 / unchanged xSAR 0.9.1

2.9.174 corrects native Boards challenge-header recognition using the union of
the five existing validated Boards literal ink supports, restricted to independently
proven complete word spans. All reference geometries and lengths are checked before
indexing. RGB tolerance 9, blue floor 6, word distance, letter cores, banner anchors,
boundaries and counter parsing remain unchanged. No new reference bank or dilation
is introduced. Score/Card recognition and input authority are unchanged.

The six controller gates passed, including 1687 tests with zero failures,
ignored or filtered tests. Qualification sealed 942 inputs; after qualification only
the application's manifest/lock version pair changed from 2.9.173 to 2.9.174,
leaving 940 inputs identical. Aligned locked check, release and protected help passed.
The 56 unchanged xSAR inputs retain cached sixteen-gate qualification (303 full
tests, 14 default tests and two Miri tests); xSAR remains modified/unreleased 0.9.1.
User profiles and CPU/hardening configuration are preserved. Evidence: `/home/charlie/repo/RUST/qmp-qemu-socket/Evidence/Pyramid/Validation/pyramid-challenge-174-proposal-9yca399e`.

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

## Current documentation and evidence relocation

The approved housekeeping moved the retained scoring note into
[docs/scoring-target.md](scoring-target.md), installed the editable
[refactoring task list](refactoring-work-tasks.md), and moved the preserved local
archive into the ignored repository `Evidence/` directory. No refactor was
implemented. The task-list contents are excluded from long-term memory and
search indexing.

Historical archive JSON/scripts are byte-identical. The active verifier remaps
only the exact former Evidence-root prefix and checks the new current snapshot.
The old 174 verifier and its qualified snapshot are retained unchanged; running
it directly still refers to historical absolute paths. Existing Cargo feature/lock
changes and automatic metadata are preserved in the relocation snapshot, and are
not newly qualified. No build, version increment, guest action, staging or push
was performed. See [relocation details](evidence-relocation.md).

## Destination Sentinel

Run exactly once before further edits:

```sh
python3 /home/charlie/repo/RUST/qmp-qemu-socket/Evidence/repository-docs/housekeeping-20261101/verify-relocation.py
```

Require HANDOFF VERIFIED or report the exact discrepancy. Preserve receipts.
Protected executable: /home/charlie/repo/RUST/qmp-qemu-socket/Evidence/Pyramid/Validation/pyramid-challenge-174-proposal-9yca399e/native/qmp-qemu-socket-2.9.174; SHA256 37cb60771f807c6adff064a5aa0fd2de37368a5adbae4734b7924a75113e2d96; mode 0700.
[Verification](Verification-2.9.174.md) records scope and evidence.

Fresh guest acceptance is pending. The retained standard-theme 0/3 frame is an
offline regression, including actual v23 Native::observe and paired_entry
classification; it is not a fresh capture or a complete live RECON/play run.
Startup deletes the fixed images directory; preserve existing evidence first.
No installation, guest input/captures, staging, commit, push or sibling edits occurred.
