# Documentation and evidence archive review

Status: Charlie approved the grouping, archive creation/verification and removal of only verified loose copies. Both private archives are complete and the corresponding loose History trees have been removed. No staging or publication occurred. Application remains 2.9.175; no program or dependency changes were made.

## Retention rules and result

| Group | Decision | Rationale |
| --- | --- | --- |
| Current release and standing guidance | Keep in `docs/` | 2.9.175 verification, active handoff, schedules, acceptance contracts, architecture, capture, terminal and operational guidance remain current even when their headings mention older versions. |
| Free Cell and editable refactor material | Keep unchanged | All 18 Free Cell documents (including earlier Solver context) and the editable work list retain identical bytes; upcoming A* work and its dependencies are protected. |
| Superseded verification reports | 73 reports moved to `Evidence/release-verification/` | Preserve qualification and failure provenance without filling the active docs directory. Current `Verification-2.9.175.md` stays in docs. These reports are excluded from the proposed History cleanup. |
| Other historical documents | 105 files moved to `History/docs/` | Old release/candidate notes, superseded plans, completed investigations and historical scoring proposals remain recoverable, with their original bytes. |
| Historical evidence | 115 case/group roots moved to `History/Evidence/` | Resolved investigations, older validation snapshots, obsolete goal drafts and publication/tmp material have archival value but no immediate operational role. Their internal layout and original scripts/JSON are preserved. |

The original docs inventory decreased from 220 to 42 retained documents before this review was added. The move inventory covers 37,237 payload entries and 22,727,551,487 regular-file bytes (about 22.73 GB; hard-linked content is counted per entry, so this is not unique disk usage). Same-filesystem renames retained device/inode, mode, size, nanosecond modification time and symlink target. Archived documents additionally retain their pre-move SHA-256. This was not a new full content rehash of the old archive. Of the 942 release-sealed inputs, 932 remain byte-identical; the other ten are fixture README files with link-target repairs only. All Rust, executable/calibration fixture payloads, Cargo configuration and the 56 xSAR inputs remain unchanged. The existing binary and host-test evidence are preserved; no new program qualification is claimed for this documentation-only work.

## Material intentionally left readily available

- The complete current 2.9.175 release/qualification case and protected executable.
- The previous 2.9.174 release case for rollback and the original standard-theme header failure/reproducer.
- The original Free Cell A* handoff, its 154/158/159 validation roots and the 160 follow-up case. Those roots are explicit handoff dependencies; they were not treated as obsolete solely because of version numbers.
- Unique Free Cell planning backups, the refactor-plan backup, the housekeeping inventory/prior proof, and the legacy resolver cited by the Free Cell handoff.
- All source, calibration assets and regression fixture payloads. Only links in fixture READMEs were repaired.

## References, privacy and recovery

Active Markdown links in 29 files were repaired to follow the moves. All protected Free Cell/work-list links resolve; the protected document bytes are unchanged. No new missing target was introduced by the moves. Existing unrelated missing targets are listed separately, not represented as repaired. Historical documents, archived provenance and scripts retain their original path text; use the location manifest and archive-aware resolver rather than rewriting their evidence.

Both `/Evidence/` and `/History/` are ignored and contain no tracked files. History includes private evidence: it must never be added to GitHub, including after creating tar.gz files. No long-term memory/index contains the editable work-list contents.

The local manifest records every old/new root and its rationale: [move plan](../Evidence/repository-docs/archive-review-20261101/move-plan.json). [Verification inventory](../Evidence/repository-docs/archive-review-20261101/moves-verified.json) records the pre-packaging checks. [Archive path resolver](../Evidence/repository-docs/archive-review-20261101/packaging/resolve-archive.py) accepts repository-relative paths, current absolute paths and the retired `/home/charlie/tmp/Evidence/` prefix. To reverse a root move before packaging, use the manifest in reverse order with destination-conflict checks; preserve the recorded link preimages as well.

The previous 2.9.175 sentinel's status mismatch from four automatic recall receipts and SQLite sidecars remains a historical failure. All compiler inputs, previous document preimages and release evidence matched before these moves. The new handoff points to a receipt for this reorganisation; it does not retroactively alter or relabel earlier proofs.

## Approved packaging and cleanup

Approval is preserved in the [packaging baseline](../Evidence/repository-docs/archive-review-20261101/packaging/baseline-state.json). The documentation archive contains 106 members; the evidence archive contains 37,062. Full gzip integrity, exact membership/type/metadata, six extended attributes, 263 within-archive hardlink families, separate GNU tar byte comparison, source rechecks and final SHA256 all passed before removing only 37,168 verified loose members, including the two empty source roots. The 73 older verification reports and all protected/current material remain loose in Evidence/docs as intended.

Active references now use member-specific recovery indexes. See [archive hashes and recovery](../History/RECOVERY.md), [verification](../Evidence/repository-docs/archive-review-20261101/packaging/archives-verified.json) and [cleanup receipt](../Evidence/repository-docs/archive-review-20261101/packaging/cleanup-complete.json). The current handoff verifies both archives and retained files. The earlier snapshot verifier's missing SQLite sidecar discrepancy remains historical; only those two transient absences were accepted before packaging, with all other pinned inputs/documents/evidence and repository state matching.
