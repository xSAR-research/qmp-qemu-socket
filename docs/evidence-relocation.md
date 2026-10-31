# Local evidence relocation

Completed 1 November 2026. Preserved project evidence now lives in
`Evidence/` at the repository root. `/Evidence/` is ignored and must never be
staged or pushed. No evidence files are tracked.

The former `/home/charlie/tmp/Evidence/` archive was copied across filesystems
without following symlinks. Every regular file was checked with SHA256; a final
rsync checksum dry run reported no differences before the source was removed.
Preserved: 25,131 regular files (23,936,585,464 bytes), 101 symlink payloads, file
modes and timestamps, and 263 hard-link groups. The five original top-level
entries keep their relative layout. No archived validation JSON or script was
rewritten. Historical absolute paths inside those files retain their original
meaning; the relocation verifier maps only the exact old Evidence-root prefix
to the new root when reading them.

Repository documentation now uses the new root. The active handoff points to
the relocation verifier. These are documentation and file-placement changes,
with no build, version change or fresh guest test. Existing Cargo manifest/lock
changes and automatic lifecycle metadata were preserved; the Cargo changes are
not newly qualified by this receipt.

Private inventories, digest checks, publication path lists and the complete
relocation receipt are under `Evidence/repository-docs/housekeeping-20261101/`.
The editable [refactoring tasks](refactoring-work-tasks.md) have not been stored
in long-term memory or indexed by XERJ/context-mode.
