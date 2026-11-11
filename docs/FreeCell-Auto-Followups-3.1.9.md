# Free Cell action and AUTO follow-ups — 3.1.9

Status: approved application correction; all six host gates passed. xSAR stays 0.14.1.

## Execution policy

**Observe recorded AUTO follow-ups** is the default application policy. It does
not change Solitaire settings. Readiness accepts two successive fresh matches of
the current physical board or an exact contiguous recorded AUTO prefix. Only
observed records are credited. The next unapplied record receives one explicit
drag, including when the planner marked that record AUTO.

After delivery, build only the exact recorded child and its consecutive AUTO
successors. Send no input for those follow-ups. Within six fresh observations,
require two successive exact matches of the final combined endpoint. Transient
partial progress is diagnostic evidence; it does not confirm the transfer or
required follow-ups. A missing/different effect stops with the original evidence,
without replay, fallback, arbitrary legal children or guessed cursor advancement.
Single Step then stops after this one explicit transfer, regardless of AUTO tags.
The next press starts from fresh readiness and the verified cursor. Remaining-route
playback repeats the same action/proof sequence.

**Execute each recorded transfer** retains explicit per-record execution and
requires the immediate recorded endpoint twice. Single Step also stops after one
transfer. Switching the frozen transfer policy requires CALCULATE again.

## Evidence and authority

The approved example is AC manually moved to Clubs and 2C automatically following:
only two fresh exact combined proofs can advance past both records. Initial
source authentication still binds all 52 physical identities. A new process does
not inherit the stopped 3.1.8 run's live history. If an AUTO occurs after readiness
but before the fresh transfer-parent capture, the stale parent refuses with no
input; there is no retry. STOP is rechecked after captures and before input.

Full/partial/Challenge Complete outcomes keep their existing distinct retirement
rules. REL Single Step remains a bounded movement probe: no route credit and no
resumable permit. xSAR 0.14.1, queried-pointer selection, ten held ABS positions,
two-second diagnostic waits, raw QMP logs and release-only recovery are unchanged.

## Verification and live acceptance

Focused tests cover unchanged readiness, AC/AUTO2C, already observed prefixes,
one-drag stopping, missing or only singly observed combined effects, malformed
prefixes, STOP/terminal, REL and remaining-route execution. Local TCPv3/Unix-QMP
tests use explicitly synthetic retained-pixel 52-card boards: one drag plus
combined endpoint proofs, and late-AUTO refusal before any probe/input.

These checks are host/local-peer evidence. Fresh 3.1.9 Windows timing and the
operator's next planned column transfer remain unverified until the next trial.
Restore the original deal or use a fresh supported initial board, RECON/SOLVE,
select Observe recorded AUTO follow-ups and ABS, then Single Step. Confirm the
one-drag/automatic-follow-up stop before requesting the next planned transfer.
See [verification](Verification-3.1.9.md).

## Code Analysis

All changes are private application sequencing and UI contracts. Exact recorded
positions bound candidates and six observations bound each proof. Full identity,
geometry and ownership checks precede input; acknowledgements prove delivery
only. No new unsafe code, dependency, public library API or gesture is introduced.
Uncertain effects stop and retire authority; they never trigger input replay.
