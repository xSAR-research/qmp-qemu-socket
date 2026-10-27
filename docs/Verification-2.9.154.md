# 2.9.154 — Klondike OK, current Failure Details and Pyramid QMP restart

Status: all three corrections approved in m00869 are applied in the checkout.
The combined source passed the six host gates with 1,531 tests and 791 sealed
inputs. The approved own-package version fields then changed from 2.9.153 to
2.9.154; aligned locked check, native release and protected `--help` passed.
The other 789 compiler inputs, including all 52 xSAR inputs, are identical.
Fresh Windows guest acceptance remains pending. No GUI launch, START, guest
input, installation, commit or push was performed for this delivery.

## Klondike LEVEL UP and Failure Details

The retained Computer Vision Continuous run positively established GAME WIN
and acknowledged its single score-skip operation. Fresh TCPv3 observations
then measured a settled OK body at `(839,723,237,71)`. Original observation 17
supplied exact-black point `(944,771)` while geometry needed confirmation.
Observations 18 and 19 retained matching settled geometry but contained no
exact RGB `(0,0,0)` pixels in the configured OK region. The finite three-plus-one
allowance expired before an OK click was sent.

Only the Klondike profile enables an additional complete current caption
appearance. Its compile-time 6,300-byte RGB reference comes exclusively from
observation 17: X=930–989 and Y=741–775. Every channel of all 2,100 pixels,
including O, K, their holes and surrounding material, must match within 12.
Held-out observations 18 and 19 differ by at most 10 and 3. The existing
exact-black scanner and other four profiles retain their policies.

Current settled body size and consecutive matching geometry remain mandatory.
The current body supplies vertical alignment; the derived point must lie
inside it. At most 2,100 additional borrowed RGB reads are performed within
the existing readiness-work bound. This adds no capture or input. Existing
waits and finite observation counts remain; the existing geometry delay
follows the selected profile's current caption proof. Completion evidence,
foreground-modal precedence, STOP/context, fresh VM/pointer probes, separate
acknowledgements and no uncertain-input replay remain required.

Failure Details had reopened an earlier Pyramid A* packet. Klondike sent
`WorkerState::Uncertain`, status and a log, but omitted the immutable
`WorkerEvent::RunFailure` already used by Spider. Its error branch now emits
that event with the current game, run settings, start time, attempted-input
flag and exact error. The existing UI handler replaces the old record and
clears the earlier Pyramid key. Reopening retains the new immutable snapshot.

The private Klondike case is `/home/charlie/Documents/Pyramid/Validation/klondike-level-up-153-20261026-c2vrsxdp`. It retains 19 original GAME WIN
images, five earlier Pyramid packet files and the 3,277,422-byte session
snapshot: 25 byte-verified originals. Images 15–19 are unchanged fixtures in
`tests/fixtures/klondike-level841-20261026`. Source history places the original
exact-black readiness policy before 2.9.153; this evidence does not identify
the cause of the pixel-colour changes or establish a newly introduced policy.

## Pyramid Continuous QMP restart

Two retained failures occurred 827 and 800 milliseconds after independent
post-Play readiness had been verified. The local error was
`QMP socket I/O failed: Resource temporarily unavailable (os error 11)`.
Search statistics described earlier completed planning. The packet's actual
automatic phase was `Starting next game`.

The production restart kept its owned QMP client alive while calling
`observed_entry()`, which opened another client on the same Unix monitor.
QEMU's socket character backend services one client until disconnect; the
second connection can wait for its greeting until the unchanged 750 ms
application read timeout expires. An ended Rust borrow does not drop its
owned client. Linux can report that timeout as EAGAIN without QEMU stderr.

The correction explicitly calls `drop(qmp)` after successful terminal
advancement and before `self.observed_entry()`. The latter still performs
its fresh entry checks and guarded acquisition. No timeout, transport API,
retry, VM configuration or input policy changes.

The isolated production-xSAR transport probe reproduced the exact error with
the retained lifetime in 789.926 ms. Releasing the first client permitted both
handshakes in 0.202 ms. Both tests passed and emitted zero guest-input commands.
The application native restart fixture now serves one connection through EOF
before accepting its successor, so its existing production restart tests
exercise this ownership boundary. The focused native restart and complete
application suite pass with that stricter peer.

Evidence and the probe are in `/home/charlie/Documents/Pyramid/Validation/pyramid-qmp-eagain-153-20261027-op57jvto`. The server inspection was
post-incident; no historical wire trace or new real QEMU connection is claimed.
The primary explanations are the
[QEMU socket backend](https://raw.githubusercontent.com/qemu/qemu/master/chardev/char-socket.c),
[QMP handshake specification](https://www.qemu.org/docs/master/interop/qmp-spec.html)
and [Linux socket timeout contract](https://man7.org/linux/man-pages/man7/socket.7.html).

## Actual host qualification

Commands used nightly, the owning package and
`--locked --offline --target x86_64-unknown-linux-gnu --all-features`.
Formatting covers changed ranges while preserving the repository's spacing.

| Gate | Result |
| --- | --- |
| Scoped nightly formatting | Passed |
| Locked check, all targets | Passed |
| Strict private rustdoc, warnings and missing docs denied | Passed |
| Clippy, all targets, warnings denied | Passed |
| Full tests including ignored tests | 1,531 passed; zero failed, ignored or filtered; 506.902 seconds including Cargo |
| Native release | Passed |
| Sequential-peer production native restart | Passed |
| Approved 2.9.154 locked check and release | Passed |
| Protected executable `--help` | Exit 0; no GUI launch; help does not print a version |

Nine new regressions cover held-out caption/geometry, the four unchanged
profiles, erased O/K/both, malformed-frame evidence reset, retained driver
sequence, STOP before the OK probe, uncertain OK without replay, immutable
worker failure delivery and replacement of stale Pyramid Failure Details.
Controlled repetitions after a simulated OK acknowledgement assert a safe
stop. They are not real post-click observations.

The earlier private two-fix qualification and its initial two outdated
shared-policy test failures remain retained separately. The combined results
above qualify the actual checkout with all three corrections and the stricter
QMP peer. Only two own-version fields changed afterwards; no full suite was
repeated for that metadata-only alignment.

Machine receipts, exact commands, immutable logs, before/aligned source seals,
application backups and version alignment are in `/home/charlie/Documents/Pyramid/Validation/pyramid-qmp-eagain-153-20261027-op57jvto/application-154`.
Protected executable: `/home/charlie/Documents/Pyramid/Validation/pyramid-qmp-eagain-153-20261027-op57jvto/application-154/native/qmp-qemu-socket-2.9.154`.
SHA-256: `8690918771efbfd45fa08417f7447b96e6dfd61b033f62c278142f52df417648`.

xSAR remains 0.9.0 at `d20dca8ad2320dcbbf869e22b35a2583317d4569`, with all
52 inputs unchanged. Its 296 tests and three Miri checks remain PASSED (Cached).
The relative path dependency is not a Git pin. No dependency, public xSAR API
or schema changed.

## Receipt and current acceptance boundaries

Before editing, Charlie's approved status-only reconciliation backed up the
old destination receipt and recorded the exact split Free Cell document
difference. The original sentinel then returned HANDOFF VERIFIED. The new
[current delivery handoff](handoff.md) binds the actual current dirty tree,
791 aligned inputs, documentation, receipts and protected 2.9.154 binary.
Unrelated local changes and all Git indices are preserved.

Fresh guest acceptance must demonstrate Klondike current-caption readiness,
one OK operation, a changed post-OK result and permitted continuation. Check
that a guarded Klondike error displays its current game/settings/error and
that reopening preserves it. Pyramid acceptance must cross the normal
reward/New Game/Play boundary into fresh entry and fresh Recon without the
overlapping-client timeout. Preserve existing STOP/context, freshness and
uncertain-delivery checks. No real guest acceptance is claimed here.

The later 2.9.153 request 9 report is a separate unresolved incident at
`/home/charlie/Documents/Pyramid/Validation/pyramid-slot23-153-20261027-o35s4q0t`. Its first retained slot 23 (`7|3`) mismatch occurred before action
2 was verified; actions through 10 subsequently passed. Independent pixel
inspection of the final frame supports absence at that slot and four other
removed positions. It does not run the Rust observer or authenticate the
unretained consumed trace. The packet does not retain the final mismatch or
pending expected state, so the final cause is not established. No fourth
code correction or deadline relaxation is included.

## Code Analysis

The caption matcher borrows a validated native frame, uses a compile-time
reference, checked coordinate operations and fixed loop bounds. It adds no
unsafe code, allocation, production thread, capture or input authority.
Missing pixels, excessive differences, lost geometry, STOP and uncertain
transport continue to withhold further input. Support is limited to the
retained native appearance; unrecorded renderings may safely refuse.

Explicit QMP ownership release closes the obsolete client before another
handshake. Errors already unwind owned resources. No acknowledged input is
repeated, and fresh scene/probe checks are retained. Failure Details reuses
the existing immutable event and UI owner; earlier evidence files remain intact.
