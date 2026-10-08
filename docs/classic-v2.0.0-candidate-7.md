# v2.0.0, candidate 7 — CLASSIC baseline and Solver startup

Promoted source base: `98f974083246044d04db5cf4298fd07590733250`.
Previous local candidate: v2.0.0, candidate 6.

## Accepted scope

Charlie approved Gate 2 on 8 October 2026. CLASSIC is the same blue card back
across all game types. Its selection changes the back colour, not calibrated
board coordinates. The application does not change the guest's theme.

| Game Type order | CLASSIC baseline |
| --- | --- |
| Klondike | Existing blue stock/tableau recognition uses the common blue predicate. |
| Spider | Stock presence and highlighted Draw checks also accept the common blue predicate. |
| Free Cell | Face-up cards; no card-back recognition required. |
| Pyramid | Card backs are not visible; no card-back recognition required. |
| TriPeaks | Existing geometry and source-HALO detection remain applicable. |

Changing menu order does not change the separately configured startup game.

## Recognition and evidence

The common CLASSIC-blue predicate requires blue at least 100, blue minus red
at least 40, and blue minus green at least 15. These are the established
Klondike boundaries. Colour alone never authorises a Draw or source click.
Spider keeps its closed source outline, 80% stock occupancy, positively
recognised gameplay scene and active Solver banner requirements.

The supplied candidate 6 log proves that the red-backed Klondike scene was
recognised, Solver was activated once, a fresh column-six target was found
and one source click was acknowledged. The next highlighted stock had less
than 50% supported-back evidence and less than 90% felt evidence. Its PNG was
named in the log but was not supplied, so the exact RGB cause is unknown.

Candidate 7 reports these two pixel counts and the 13,900-pixel denominator
on that refusal. It does not widen the red palette from a text-only report.
The prior red scene fixtures and tests remain available, but CLASSIC is the
shared live acceptance baseline. Existing measured red support is retained.

No native CLASSIC-blue Spider Draw PNG has been supplied. Tests recolouring
the existing native red Draw fixture exercise palette handling only; they
are labelled synthetic and do not establish acceptance of a real blue Draw
scene. A native blue Draw HALO frame and its log complete that verification.

## Missing-HALO startup

Previously the UI rejected no-HALO TriPeaks/Pyramid run requests. Their
initial worker planning also only recaptured: Pyramid stopped after six
observations, while TriPeaks waited until STOP. Existing post-action or
redeal Solver recovery did not cover this startup path.

An explicit Step Once or Multi-Step request may now begin from a current
no-HALO preview. Fresh recognised, non-empty gameplay must be established
before Solver input. Initial planning obtains one fresh capture and up to
three delayed observations. If unresolved, it reserves at most one Solver
operation, checks STOP and QMP input readiness, then captures immediately
after acknowledgement and permits up to three further delayed observations.
Persistent uncertainty stops with the latest frame retained.

A HALO appearing before the Solver operation skips activation. A no-HALO
request may adopt only the resulting fresh canonical, mode-owned action;
an existing concrete preview still requires equality with its fresh target.
Solver setup does not consume the gameplay budget, so Step Once sends at
most one gameplay action. Existing effects, Pyramid consumed-slot history,
completion/redeal and terminal handling retain their own policies.

Strategy selection, Game Type changes, manual capture and snapshot capture
remain input-free. Every decoded observation continues to publish through
the bounded latest-preview slot. STOP is checked during waits and before
input. An uncertain Solver delivery consumes its reserve and is not retried.

## Documentation validation

The candidate 6 documentation command exited successfully with two warnings
because unquoted RGB arrays were parsed as intra-doc links. Candidate 7
renders them as inline code and validates documentation with
`RUSTDOCFLAGS='-D warnings'`.

## Code Analysis

The guest-input boundary remains the QMP allow-list with absolute-pointer
readiness checks and checked frame geometry. The common colour predicate
adds no unsafe Rust, file access, network access or guest input. Channel
differences use signed 16-bit arithmetic, avoiding unsigned underflow.

Stock fractions remain bounded by their calibrated rectangles; blue, felt,
paper and source gold provide distinct evidence. Unknown interiors, scenes,
terminal panels and absent cards cannot acquire startup Solver authority.
One reserved Solver operation and finite observation budgets bound failure
behaviour; STOP and uncertain acknowledgement prevent replay.

The installer verifies pinned complete files, private backups, source states,
file modes and the Git index. Unknown affected edits, staged affected paths,
links, wrong HEAD and checksum mismatches stop before source replacement.
Rollback preserves later unknown edits rather than overwriting them. Live
Spider CLASSIC calibration remains a stated limitation until original-frame
verification on Beast.
