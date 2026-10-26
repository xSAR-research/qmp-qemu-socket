# Verification — 2.9.135

Status: **host qualification passed; fresh live acceptance pending**.
This scoped correction qualifies application **2.9.135** on the published
2.9.134 source, with unchanged xSAR 0.8.105, console 0.8.5 and server 0.1.4.
The current correction earns one patch increment. The next specifically approved
minor milestone starts at patch `0`; no existing release is renumbered.

## Behaviour under verification

- Exact automatic partial-endpoint observation can use the existing measured
  stock-boundary proof, with temporary scope restored on success or error.
- The retained six-card fixture is unchanged. Its offline reconstructed route
  is replayed through rules and the independent explicit-pile model, without
  claiming it was the original route or proving optimality.
- Partial proof can reach one guarded RECYCLE; unchanged post-input pixels must
  refuse continuation without retry. Standard and STOP boundaries are preserved.
- The native session resets Maximum to Shortest at the next board and publishes
  that method in the actual progress event. Kings-specific defaults remain.
- Planned and residual stock-drain MOVE request 250 ms. Computer Vision remains
  at 1,000 ms; planned RECYCLE remains 1,000 ms; stock-drain RECYCLE and Recon D remain
  500 ms, with unchanged MATCH/KING/final timing.
  Logs measure monotonic wait elapsed independently of requested duration.

## Qualification

All required host checks passed on the same **675 sealed application/xSAR inputs**:

| Check | Result |
| --- | --- |
| Scoped nightly format, eight owned Rust files | Passed; inherited spacing retained and sibling files untouched |
| Personal version-numbering skill validator | Passed |
| Locked all-target/all-feature Cargo check | Passed |
| Retained partial-endpoint native regressions | 2 passed |
| Residual stock MOVE, planned MOVE/CV isolation and native method reset | 1 passed each |
| Strict private rustdoc | Passed with `-D warnings -D missing_docs` |
| Clippy, all targets/features | Passed with `-D warnings` |
| Full native tests, including ignored | **1364 passed, 0 failed, 0 ignored, 0 filtered** |
| Native release build and protected executable help | Passed |

Cargo used `+nightly`, `--package qmp-qemu-socket`, `--locked`, `--offline`,
`--all-features` and explicit target `x86_64-unknown-linux-gnu`. The full test
command used `-- --include-ignored --test-threads=8` and took **398.254 seconds**;
the release build took **41.416 seconds**. Toolchain: Rust **1.101.0-nightly**,
commit `7189d39d9ce936881b8132c6addeaf3e295cd455` (22 October 2026).

The private `redeal-move-20261024` evidence bundle contains exact argv, input and
log hashes, candidate provenance and the independent review. Both concrete
review findings were resolved: residual MOVE uses 250 ms, and the retained-frame
regression traverses partial confirmation before stock draining.

The protected native candidate's SHA-256 is
`427518c718af25a10bc4dcc4ec22857184c6afe246119ca5d3e9f25b36fcfa20`.
It matches Cargo's unique `qmp-qemu-socket` binary artifact for package 2.9.135.
`--help` exits successfully without launching the GUI; it does not display a
version. An initial evidence-helper assertion expecting a version in help was
corrected using the Cargo artifact and exact executable hash, without rebuilding
or overwriting the protected copy. That helper failure is retained separately.

The source, fixture and policy diffs and local documentation links were checked.
Existing live acceptance and milestone case outcomes remain unchanged.

## Evidence limits

After Charlie confirmed 2.9.133 had exited and a fresh Score challenge was ready,
the protected 2.9.135 candidate was launched on 24 October 2026
(17:11:03 UTC on 23 October). PID **843736** resolved to the protected executable;
its running-executable SHA-256 matched the qualified candidate above. The new
session log independently reported `qmp-qemu-socket v2.9.135 started` and awaited
a strategy choice. The private bundle records launch identity and session output.
All 147 prior GAME WIN images were copied and hash-verified before startup's
normal image-directory cleanup. The agent did not press START.

No fresh live outcome or timing acceptance has yet been recorded. PATH
installation and signed publication remain separate. The retained packet belongs
to 2.9.133. A full new partial-route → DRAW/RECYCLE → REDEAL → restored next board
trial and measured 250 ms reliability remain necessary for Charlie's acceptance.
See the [live-test brief](Live-Test-2.9.135.md),
[active goal](Pyramid-Short-Term-Goal.md) and
[fixture provenance](../tests/fixtures/pyramid-recon/native-partial-stock/README.md).
