# Active handoff — application 3.1.16 / xSAR 0.16.1

All 51 fresh paired host gates passed with 1,893 application, 366 library and 80 console tests. The full suites report zero failed, ignored, measured or filtered tests. See [goal](FREE_CELL_goal_20261111G.md), [contract](FreeCell-Source-Midpoint-3.1.16.md) and [verification](Verification-3.1.16.md). The held ABS path is source-up45, the native floor midpoint of original source/effective destination, destination-up45 and exact destination, followed by UP. Both two-second and 500 ms choices use six waits. Fresh Windows acceptance is pending operator trial.

## Qualified executable and content

Capsule `Evidence/FreeCell/Validation/freecell-source-midpoint-316-20261111/` contains the 1,057-input final seal `5979ab6e76503cf04bd821870d3274639001bf4175afaf9e6717aae48f1d425c`, exact command/toolchain/configuration records, first-failure evidence, final suites and compilation-context artifacts.

Executable `/home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket` is version 3.1.16, 16831760 bytes, mode 0755, SHA-256 `c1f48045c41583b0a2646d1ec92edd82d59de7cdf4cf2f18bbcbcd0427ade699`. This is the mutable Cargo artifact. All four histories, index bytes and staged diffs are preserved.

Launch this qualified Cargo artifact with the following command.

```sh
/home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket
```

A normal Cargo launch from the repository root can rebuild the mutable artifact.

```sh
cd -- /home/charlie/repo/RUST/qmp-qemu-socket && cargo run --release
```

## Destination Sentinel

The prior 3.1.15 scoped sentinel passed before edits with independent expected closeout digest `d543b183ee3e5e2dc5bdc740b1f23adba1ae0c594df5c0d28c3962c4261ed9c0`, exactly `HANDOFF VERIFIED (scoped source/evidence; no cached build claim)`. The new 3.1.16 sentinel awaits its independent expected digest.

Before the next authorised mutation run `python3 ./verify-source-midpoint.py --expected-closeout <independent-approved-delivery-digest>` from this capsule. Obtain that expected digest from Charlie’s delivery/approval record. Add `--check-artifacts --check-toolchain` when claiming current cached qualification. Stop on the exact reported discrepancy. The capsule becomes immutable after final documentation, scans and closeout.

## Remaining acceptance and authority

Operator Windows acceptance remains pending. Each playback invocation requires fresh original-52 binding and relevant source/target/AUTO proofs, VM/pointer checks, STOP/context/source/run ownership and two fresh recorded endpoint matches. Ordinary failures can retain the immutable calculated report for an explicit manual UNDO ALL/fresh-52 restart. Positive terminal/probe completion and explicit revocation retire its authority.

Charlie has standing approval for PATCH increases in both repositories during DRAG debugging. Each instruction group still defines its implementation scope. GitHub publication commands and detailed cumulative messages are prepared for Charlie to execute.
