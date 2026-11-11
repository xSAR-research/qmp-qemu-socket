# Free Cell AUTO follow-ups — approved application 3.1.9

Charlie approved application 3.1.9 using unchanged xSAR 0.14.1 on 10 November 2026.
Status: host qualification complete; operator live acceptance pending.

## Outcome

In the default **Observe recorded AUTO follow-ups** policy, Single Step performs
one explicit drag, even when that record is AUTO. It observes the consecutive
recorded AUTO follow-ups, requires two fresh exact combined endpoint proofs,
credits those records and stops. The next press performs the next planned move.
A twice-confirmed unchanged board is valid readiness; observed recorded AUTO
prefixes are reconciled before input. Expected moves never grant assumed credit.

The acceptance example is AC dragged to Clubs, followed automatically by 2C:
prove the combined board twice, credit both records and stop. Missing/different
follow-ups fail within six observations without replay or another drag.

## Bounds and acceptance

Keep immutable routes separate from accepted physical history. Preserve initial
52-card binding, printed identities/geometry, source/run/context/STOP fences,
fresh parent/VM/pointer checks, terminal and partial-route retirement. A late
AUTO contradicting the fresh transfer parent refuses before input. The retained
explicit-record policy remains available and is labelled honestly; neither policy
changes Solitaire settings. ABS transport, pacing, xSAR APIs and dependencies are
unchanged. REL remains a retired movement probe with no cursor credit.

Required host checks: focused sequencing/local-peer regressions, owning format,
locked check, full tests including ignored cases, strict private rustdoc, Clippy
and release build. Retain first failures, source/toolchain fingerprints, executable
identity and prior evidence. Do not edit xSAR, sibling packages, Git indices or
history. A live trial starts from the restored original deal or a fresh supported
initial board followed by RECON/SOLVE; the retired changed-board plan cannot resume.

See [contract](FreeCell-Auto-Followups-3.1.9.md) and [verification](Verification-3.1.9.md).
