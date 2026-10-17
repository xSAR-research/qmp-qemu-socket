# Combined capture candidate.13

This local source set contains application 2.2.2-candidate.13, xSAR
0.4.2-candidate.2 and Windows screen-grab server 0.1.2. The prior candidate.12
OK-confirmation change is retained. Source/build checks, Windows tests and live
guest acceptance are separate evidence; this document does not declare them passed.

## Compatibility and ownership

The application and server consume sibling `../xsar` sources. A path dependency
does not pin a Git revision; use the complete reviewed source/checksum set.
The [shared protocol contract](../../xsar/docs/tcp-capture-protocol.md) describes
the exact headers, bounds, formats and public client changes.

| Arch application selection | Matching server 0.1.2 | Previous v2 server |
| --- | --- | --- |
| `QMP_CAPTURE_WIRE=v2` | Explicit BGRA, one frame per connection | Compatible |
| `QMP_CAPTURE_WIRE=v3` | BGR8, one frame per connection | Refused/connection closed |
| `QMP_CAPTURE_WIRE=v3-persistent` (default) | BGR8, repeated fresh requests | Refused/connection closed |
| `QMP_CAPTURE_BACKEND=qmp` | Uses existing QMP capture instead | TCP server unused |

There is no automatic downgrade, QMP fallback, failed capture retry or guest
input replay. The previous Arch binary can use server 0.1.2 through v2. The new
Arch binary can explicitly select v2 with the previous server. These are
deliberate compatibility choices, not an instruction to replace either binary.

The worker retains one serial client for a complete controller context, address
and wire mode. Context changes, errors, disconnect and shutdown end that session.
Every accepted request causes a fresh full-display acquisition; there is no
background screenshot queue or reused screenshot. Before PNG encoding,
publication or analysis, the application requires 1920x1080 RGBA8, stride 7680
and exactly 8,294,400 bytes. Capture PNG and GAME WIN recording retain their
existing full-frame, same-acquisition ownership and timestamp boundaries.

## Server bounds and measurements

The server keeps its existing manual PowerShell launch and `0.0.0.0:9999`
listener. Networking/firewall settings are unchanged. A dedicated OS thread
owns synchronous capture, creating and releasing GDI resources per request.
The compatible bitmap is deselected and restoration checked before GetDIBits.
This package does not introduce DIB sections, reusable GDI resources, ROI,
compression, network changes or alternate controller rules.

| Resource or stage | Bound |
| --- | --- |
| Connected admitted clients | 2 |
| Synchronous capture | 1 executing |
| Waiting capture queue | 1 |
| Response leases across queue/capture/send | 2 |
| Idle connection | 120 s |
| Request remainder after first byte | 10 s absolute |
| Queue residence before capture starts | 2 s |
| Async wait for capture result | 10 s absolute |
| Response header and payload writes together | 10 s absolute |
| Rolling statistics | 256 samples per stage; 60 s summaries and shutdown |

BGR packing compacts an owned 8,294,400-byte BGRA allocation in place and sends
6,220,800 bytes. Its capacity remains allocated until that response is dropped;
two leases therefore bound backing pixel buffers to 16,588,800 bytes, in addition
to bounded headers/statistics, socket buffers and the active Win32 resources.
The host temporarily owns both BGR input and RGBA output during expansion.

Timeouts revoke queued/network work and discard later unwanted results. They
cannot interrupt a Win32 call already executing. A hung GDI call can retain its
single executing lease and delay clean process shutdown; it cannot create
additional capture threads. Do not describe a timeout as pre-emptive GDI cancellation.

Guest durations separate queue wait, allocation/setup, BitBlt, GetDIBits,
packing, capture preparation and socket writes. Host logs separate connection,
request write, header wait, allocation, receive, RGBA preparation and conversion.
PNG, detection and preview scheduling are separate. Scheduling into the latest
preview slot is not the time the UI displays a texture. Socket write completion
is not proof the host consumed or rendered the frame.

## Verification and measurement order

1. Finish locked host checks and the operator's native Windows build/tests
   before replacing either installed executable. Keep the prior matched pair
   and its hashes. The [Windows procedure](../../qemu-screen-grab-server/docs/verification-windows.md)
   builds into a separate directory and does not start the service.
2. On the intended guest, compare the corrected GDI baseline with explicit v2,
   then one-frame v3, then persistent v3, using the same new server binary.
   Retain command, package/source hashes, display settings, frame counts,
   guest/host stage summaries and warm-up policy for each run.
3. Collect comparable samples with identical `--image` settings and board state.
   Report medians/p95/max and failure/reconnect counts, separating capture,
   transfer, conversion, PNG and rendering. The 25% pixel-payload reduction is
   arithmetic, not a measured 25% latency reduction.
4. Verify raw native colours, alpha 255, exact black versus near-black, fresh
   frame identities, sequential requests, manual Capture PNG, and preserved
   GAME WIN entry/result evidence. Exercise disconnect, stalled/partial
   response, overload, STOP and Ctrl+C without uncertain input replay.
5. Run each of Klondike, Spider, Free Cell, Pyramid and TriPeaks through the
   intended GAME WIN transition. Record operator acceptance and failures
   separately from Linux socket tests and the earlier candidate.12 log.

Deployment remains a separate approval. If the new pair fails acceptance,
retain its first-failure evidence and return to the previous matched binaries
using the operator's recorded paths. Restoring one side while selecting an
incompatible protocol must fail visibly; it must not guess another format.
