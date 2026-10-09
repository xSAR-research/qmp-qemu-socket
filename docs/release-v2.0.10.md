# v2.0.10 — TriPeaks terminal acquisition margin

The release retains the delivered v2.0.9 files over promoted v2.0.8 commit
`cf4e5c696052d248b39c956cab6492099ab44020`, tree
`e0ef5784294d9ad5af25a2341b16940eaa4cc656`. At intake Charlie's last promotion
report was unpushed. The installer recognises only this exact base, its complete
installed v2.0.9 state or the complete installed v2.0.10 state. A different HEAD
or unknown affected content is refused, rather than rebased or overwritten.

## Reported failure

The supplied v2.0.9 session log records nine successful TriPeaks wins and restarts
before action 1407 exhausts terminal acquisition. All five direct Level Up wins
first became positive candidates on capture 20 and confirmed on capture 21.
The four uncovered reward wins first matched on capture 5 and confirmed on 6.

Final action 1407 clicked the sole row-1 column-2 card at native (960,201). Its
canonical final-card context and material effect passed. None of 20 result
observations yielded a supported terminal candidate; acquisition stopped after
28.7998 seconds. No terminal or Solver input was sent and the card was not retried.

The later supplied screenshot passes the existing Level Up detector: title glyphs
319/320 and backdrop 745/757 each exceed 98%; dim prompt, borders and foreground
edges match fully; raised OK is 1000/1000 gold and legacy-low OK is 0/2400.
The pointer overlaps the title but stays within the existing tolerance. This
evidence does not call for changes to masks, colours, geometry or click points.

The screenshot was requested 164.626 seconds after the stop and prepared at
164.933 seconds. Its attachment contains 1,744,870 bytes, while the log reports
saving 1,105,809 bytes. Preserve the attachment unchanged as TP03, SHA-256
`3c9ad40600ff5723d2f73df6e1a343edbc5fd0663c5d38c6f1398ac7e9214fa9`.
It does not establish the pixels in the earlier 20 observations. The repeated
capture-20 successes strongly support extra acquisition margin; they do not
prove this failed run would necessarily recognise a terminal by capture 30.

## Resulting behaviour

Qualified TriPeaks final-card acquisition now has a dedicated allowance of
**30 total post-action captures**, counted from round 1. This is ten additional
possible observations, not 30 more after an animation is classified. The branch
still requires canonical sole-final-card context and material effect. Fresh
nonempty or actionable gameplay returns to the existing gameplay policy.

Terminal recognition is evaluated before the acquisition bound. A first positive
on capture 30 can therefore enter the existing four-capture confirmation phase,
including that first candidate. It normally confirms on 31; transient or mixed
kinds can consume the allowance through 33 before stopping. Confirmation does
not reset acquisition or combine different terminal types. Two consecutive fresh
positives of the same supported kind remain required.

The existing recognition predicates, raised OK selection, final-action effect
thresholds, observation interval, input guards and terminal order are unchanged.
The shared post-game limit remains 20 and the other games keep their existing
limits. Earlier positive candidates proceed immediately through the same
confirmation path; the larger allowance does not impose a new fixed wait.

Extra unresolved observations remain input-free and cancellable. They do not
authorise Solver, a centre skip, replay of the final card or a completion event.
An unsupported scene still stops with the latest frame when its allowance is
exhausted. The limit is a capture count, not an overall wall-clock deadline.

## Verification and Beast acceptance

The delivery records retain actual compiler versions and exit statuses for check,
tests, strict private-item rustdoc, Clippy and release build. Locally executable
native regressions check the unchanged TP03 signature, final-card proof,
confirmation policy and separate constants. Compiled host-only production QMP
regressions cover delayed candidates, exact acquisition exhaustion, cancellation
and fresh gameplay escape; those cases could not execute here.

This executor rejects AF_UNIX socket creation. Host-only QMP tests must remain
reported separately from locally passed tests and must run on Beast using
`cargo test --locked -- --include-ignored`. Compiled tests and recorded images
do not establish live animation timing or guest restart success.

Gate 3 on Beast verifies exact installed source and the newly built executable,
then a TriPeaks direct Level Up win and supported restart, reward entry, image
updates and STOP. Capture first-failure evidence if another unsupported transition
appears; do not replay uncertain input. The new 30-capture margin remains a
bounded reliability correction, not a guarantee about unknown future scenes.

The package provides complete files, pinned hashes/modes, a guarded installer,
runtime apply/refusal/idempotence/rollback evidence, all numbered Beast steps and
commit text. Rollback restores the exact original base or v2.0.9 profile and
requires rebuilding before launching the restored release.

## Code Analysis

The change alters only how long qualified final-card evidence may receive
input-free observation. External screen pixels still need supported terminal
recognition, fresh final-action proof and consecutive same-kind confirmation
before granting completion or terminal input authority. It adds no unsafe Rust,
dependency, coordinate, image-layout, QMP protocol or filesystem API change.

Existing checked frame/ROI indexing and widened coordinate arithmetic remain
intact. Acquisition is bounded at 30 observations; an active confirmation phase
retains its separate four-capture allowance. STOP remains cooperative and cannot
retract delivered input or instantly interrupt an in-flight capture/QMP call.
Unknown, transient, mixed or insufficient-effect evidence stops without replay.
The main cost is extra capture/wait time when the final-card result stays unknown.

The installer verifies exact known content, bounded regular members, paths,
hashes and modes before writing, preserves unrelated work and the Git index, and
retains private backups/journals for rollback to the actual starting profile.
Unknown source states refuse replacement. Package integrity checks complement
verification of the downloaded ZIP against the separately supplied SHA-256.
