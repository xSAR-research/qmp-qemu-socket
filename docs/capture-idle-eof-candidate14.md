# Capture idle EOF repair — candidate.14

Application 2.2.2-candidate.14 uses the existing sibling xSAR
0.4.2-candidate.2 with a narrow TCP client repair. Version-3 servers 0.1.2
and 0.1.3 remain compatible. No server replacement, protocol migration or new
dependency is required. These remain unpromoted local sources; the path
dependency's lock entry does not identify its source bytes.

## Behaviour and limits

After a successful persistent capture, a server may close its idle connection.
The preceding client checked the retained socket before the next request but
ignored a clean-EOF result. It then sent on that socket and could report
`I/O error: truncated capture response`.

The client now checks the retained socket before deciding whether the connection
is reused. Positively observed clean EOF discards that socket and resets session,
request, display generation, frame and dimension evidence. The existing bounded
connect branch then opens one connection for the new request. Its initial wire
identities are `(session, request, generation) = (0, 1, 0)` and its successful
timing metadata reports `reused=false`. Connect timing covers that new attempt;
total acquisition timing includes the preflight check.

No request is replayed after sending begins. Unsolicited bytes, peek errors,
write/read failures, malformed responses and cancellation still stop the capture
and invalidate the retained state. A peer closing after a successful preflight
can still cause a safe failure. This repair addresses a positively observed idle
closure; it does not make every disconnect recoverable within one capture.

The five-second connect bound and ten-second absolute write/header/payload
bounds retain their existing cancellation checks. Full-frame native RGBA8,
opaque alpha, PNG ownership, GAME WIN diagnostics and input policy are unchanged.

## Source and regression evidence

The production change is confined to `ScreenCaptureClient::capture_once` in
`../xsar/src/tcp/mod.rs`. The application receives it through its existing path
dependency and worker cache; no application transport API or cache reset was added.
The application manifest and root lockfile identify the binary as candidate.14.

Four new private xSAR regressions exercise the production capture path:

- A public first capture returns while the peer remains open. Only then does the
  peer close its write side. A bounded, non-consuming peek positively establishes
  EOF before the second public capture. The new connection must use reset wire
  identities, accept a restarted frame identity and changed dimensions/generation,
  preserve the first owned frame, and send nothing on the old socket.
- Cancellation with a retained socket and observed EOF sends no request and
  opens no replacement connection.
- An unsolicited byte queued during idle time causes a protocol failure without
  sending or reconnecting.
- A truncated response after a reused request fails without another connection.
  Only a later explicit capture may reconnect with reset identities.

The first regression failed against the preceding production code with
`IoError("truncated capture response")`; all 13 private TCP tests passed after
the repair. Local checks used rustc 1.101.0-nightly (4ebd25825, 2026-10-15)
and Cargo 1.101.0-nightly (3960fa69b, 2026-10-12), locked and offline:

| Check | Observed result |
| --- | --- |
| xSAR default features | Check passed; 14 tests passed |
| xSAR TCP feature | Check passed; 56 tests and 3 doctests passed |
| xSAR all features | 197 tests and 5 doctests passed |
| Application TCP consumer integration | 10 tests passed |
| Complete application suite, including ignored tests | 858 passed; none ignored |
| Strict private rustdoc and Clippy, both repositories | Passed |
| Scoped formatting of the two modified Rust files | Passed |

Release build and executable identities are recorded in the delivery evidence.
Whole-tree `cargo fmt --all --check` reports pre-existing differences; the
modified Rust files pass using the repository's nightly formatting settings.
The original private tests remain an exact byte prefix, and unrelated application,
xSAR, server and console sources and Git indexes retain their pre-edit hashes.

## Operator verification

Use the validated candidate.14 executable and confirm its window/session label.
A build does not replace an installed executable or an already running process.

1. In persistent-v3 mode with the existing server, use Capture PNG once and
   retain the host/server logs. Do not run gameplay or another acquisition during
   the gap.
2. Leave the connection idle beyond the server's configured idle bound (currently
   120 seconds). Obtain a second Capture PNG. It should succeed on that first
   request when clean EOF has reached the client, with a new session,
   `reused=false`, and fresh full-resolution pixels. Correlate server logs rather
   than assuming every idle closure has the same cause.
3. Immediately capture again to confirm ordinary live reuse still works. Compare
   identities, RGB values, opaque alpha and PNG contents.
4. Verify STOP and a deliberate mid-response failure stop without request/input
   replay; keep first-failure logs. A later independent capture may recover.

Native Windows acquisition and these live checks require separate operator
evidence. The Pyramid unknown-scene/classification investigation remains separate.

## Recovery and review

The delivery evidence directory contains the pre-edit source copies, original
release executable, file modes/hashes, Git index hashes and validation logs.
Compare against that snapshot when reviewing this repair because the checkouts
already contain earlier unpromoted work. Preserve that work during any rollback.
Restore the saved pre-edit copies only after checking for subsequent edits; do
not reset either repository to its historical HEAD. No source or binary is
installed, staged, committed, pushed or published by this delivery.
