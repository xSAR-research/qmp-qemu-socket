# Verification — application 3.1.10 / xSAR 0.14.2

Status: host-qualified. [Goal](FREE_CELL_goal_20261111A.md) and
[contract](FreeCell-Corrections-3.1.10.md) define the approved scope. Fresh Windows
acceptance of the no-delay double-destination gesture remains an operator trial.

## Fresh qualification

All 51 fresh paired host gates passed: 1,881 application, 351 library and 80 console tests, zero failed/ignored/filtered.
Full test scopes included ignored cases with none filtered. The 51-gate schedule
covers scoped formatting, locked all-target checks, strict private rustdoc with
`-D warnings`, all-target Clippy with `-D warnings`, full owning tests and release
builds, plus library dependency-free defaults, all sixteen isolated features and
the actual console consumer (0.8.5). All Cargo gates used `+nightly --locked
--offline --target x86_64-unknown-linux-gnu` and their recorded package/features.

All 1,037 compiler/configuration/test inputs and the toolchain/configuration
were checked before and after each gate. Final seal SHA256 `26df2a0869736d5777ab668afe91eedf3f3de8b26436fc6655a6a2fd39a438fe`;
toolchain `rustc 1.102.0-nightly (ac8310ba6 2026-11-09)`. Edition 2024/MSRV 1.101.0 and the existing CPU/hardening,
target and dependency defaults were preserved. See `Evidence/FreeCell/Validation/freecell-corrections-310-20261111/qualification-summary.json`
for exact commands, durations, source pins and complete toolchain records.

Focused tests passed eighteen library QMP cases and 207 application Free Cell
cases. They cover exact source-ACK/DOWN/DEST/DEST/UP packets, independent raw-wire
and observer comparison, release before held ACK waits, no configured pacing in
ABS, STOP/write/command/routing failures, cached UNDO ALL/policy retries, wrong
52-card source refusal with zero input, restored source rebinding, both native
inventories, unchanged ambiguity limits and every required gold-edge pixel.

## Preserved first failures

The first application focus passed 206 and failed one native STOP mock. A peer
observing DOWN cannot prove the no-delay client has not already sent/confirmed UP.
The held-wait mock now uses paced REL; synchronous observer tests directly cover
every held ABS STOP boundary. Original `app-focused-first.log/json` remain.
An argument-only harness call failed before Cargo and is retained separately.
The first full pipeline stopped on an unused `mut` in a native test under strict
Clippy. Its logs and original input seal remain; removing `mut` changed no runtime
behaviour, and the final pipeline passed against a separate final seal.

## Artifact and preservation

Qualified mutable executable:

```sh
/home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket
```

Version 3.1.10, 16829824 bytes, mode 0755, SHA256
`5595fa12ee546bf78fc3d4bbb7f9561aca2bb19239a722638f755e28163f0027`. `artifact.json` and `paired-artifacts.json` bind actual Cargo JSON
outputs. The standalone all-features library rlib is recorded separately from the
application-consumer feature/profile context; their hashes need not be equal.

The prior 3.1.9 sentinel returned `HANDOFF VERIFIED (scoped source/evidence; no
cached build claim)` before edits using independent closeout SHA256
`8bd533ddf8306fc9dd4d1ba3fe3dcc0702503765feffa00745a20ba2feab9b67`.
All previous 1,033 compiler/configuration/test inputs matched the starting snapshot.
All four HEADs, index bytes and staged diffs, unrelated server changes, old release
bodies and prior immutable evidence are preserved. Only owning package versions
and their lock entries changed; no dependency update or source override occurred.

Final scoped secrets/content/link checks and closeout are recorded in the same
evidence directory. A new destination sentinel needs Charlie's supplied/approved
independent expected closeout digest. Default coverage is source/evidence/Git;
mutable artifacts and current toolchain require the additional checks named in
[handoff](handoff.md). No new-sentinel pass is claimed using a self-derived digest.

## Live limits and Code Analysis

The operator reports about 35 successful earlier 3.1.9 moves with AUTO enabled.
During closeout Charlie confirmed playing through the calculated A* winning
route successfully. This is recorded as operator route validation. The new
no-delay gesture awaits its own trial.

Local QMP/TCP peers, retained native frames and synthetic pixel compositions prove
host seams, not fresh Windows timing or live acceptance. The agent did not launch
the GUI, replace a process, capture/input to the guest, install or promote.

Input remains bounded by checked coordinates, fresh all 52-card binding/parent proofs,
STOP/context/source ownership and two exact fresh combined endpoints within six.
Immutable cache retention never keeps uncertain live permission. A new explicit
retry starts cursor zero; hard invalidation cannot be reversed by stale completion.
No new unsafe code, public signatures, schemas or dependencies. Uncertain held
delivery permits targeted release only, never DOWN/motion replay or guessed effect.
