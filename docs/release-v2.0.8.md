# v2.0.8 — TriPeaks reward transition

Promoted source base: `d142fa7994dc3da961103d80fdad37d341d0e538`.
Charlie reported the candidate 7 commit and push on 8 October 2026; its complete
Git tree and signed commit were independently checked before this release.
The requested release label is **v2.0.8**, without a candidate suffix.

## Reported failure

The final section of `qmp-qemu-socket-1791453259757.log` records the last
TriPeaks target as row 1, column 3 at (1469, 201). Operation 146 sent that
card click once. Four subsequent captures had no HALO and no exposed rows.
The fourth capture failed the gameplay-scene gate, so missing-HALO recovery
stopped with 145 verified actions, 146 attempted actions and no Solver
recovery click for that final operation.

The supplied 1920×1080 PNG shows the Congratulations level reward overlay,
including its green title banner, blue panel and “Click anywhere to skip”
caption. Its changing level and reward artwork must not become calibration
requirements. The screenshot was captured manually after the guarded stop;
it is evidence of the displayed overlay, not the exact earlier worker frame.

The supplied attachment is retained unchanged as a regression fixture. Its
SHA-256 is `1cce002aeecee0d167b9b9e0099bce5361bbc9cadb77629ff18216eb670441d6`.
The attachment is 2,098,416 bytes, whereas the session log reports saving
1,333,621 bytes. Consequently this release does not claim byte identity with
the earlier saved QMP file.

## Resulting behaviour

TriPeaks has a separate, narrow reward-overlay discriminator. It measures
fixed title and skip-caption lettering, banner and panel structure while
excluding changing rank, level, reward values, animated artwork and the
recorded pointer location. This does not widen generic gameplay or gold-HALO
recognition.

The transition requires a just-delivered canonical top-row tableau action
from a fresh board showing exactly one exposed top-row card,
the existing cursor-excluded action-effect threshold, no fresh target or
exposed rows, and two consecutive fresh reward-overlay observations. The
first positive schedules an input-free, cancellable confirmation. Confirmed
terminal evidence can enter the existing bounded post-game flow without a
Solver click over the modal panel.

A partial or delayed terminal animation may receive input-free observations
until post-action round 20, only after the same final-card context and material
effect are established. The first positively recognised candidate starts a
separate confirmation allowance of at most four captures, including that
candidate. Unknown panels remain unclassified throughout the wait and receive
no Solver or skip input. A fresh recognised nonempty board returns to the
existing gameplay and redeal policies.

Reward-skip input keeps the existing click point, hold, fresh QMP probe and
bounded delivery policy. Each reward-skip attempt requires fresh positive
overlay evidence. An unknown modal panel or a disappearing reward overlay
does not authorise an arbitrary centre click. STOP and uncertain delivery
stop the sequence without replaying the last card or an uncertain skip.
Pyramid and the three independent one-board controllers retain their
existing completion policies.

## Solving methods and version identity

The README opens with the two application methods. Computer Vision follows
fresh recommendations from the guest Solver through the five calibrated
game controllers. Shortest path route calculation is currently read-only
Pyramid preparation. Card recognition, graph construction, Dijkstra/A*
search, route execution and replanning remain future work.

The README includes a compact Mermaid process diagram and ends with
Klondike, Spider, Free Cell, Pyramid and TriPeaks descriptions in menu order.
Current build, capture, private log and input-policy information remains
available, with historical delivery details linked rather than presented as
current behaviour.

Cargo.toml and the root Cargo.lock package entry both use `2.0.8`.
The displayed release label comes from Cargo's package version and has no
candidate suffix. The xsar revision remains
`1a719b359a51d1e1e3113224193a779c76de82f8`; dependency records are unchanged.

## Verification limits

Automated regression tests establish acceptance of the supplied frame and
refusal of incomplete or unrelated evidence. Recorded-frame tests exercise
the bounded decision sequence; they cannot establish the timing of the
original run or a successful future QMP restart. Live TriPeaks win/reward
progression on Beast is the remaining acceptance check.

Ten production-worker/QMP regressions use a private local Unix socket server.
The delivery executor rejects AF_UNIX socket creation with EPERM before the
worker runs. Those tests are compiled and explicitly marked as host tests;
they are not reported as locally passed. The Beast guide requires
`cargo test --locked -- --include-ignored` to run them with the ordinary suite.
They cover fresh reward confirmation, delayed animation, exact observation
bounds, STOP, insufficient effect, uncertain skip delivery, disappearing
reward evidence and return to the existing redeal policy.

Charlie reports that candidate 7 played all five game types correctly before
this TriPeaks GAME WIN stop. That report is retained as live feedback;
original CLASSIC gameplay screenshots for the future route-planning baseline
remain a separate evidence collection step.

## Code Analysis

Captured frames remain checked for native dimensions, stride and storage
before fixed pixel access. The change adds no unsafe Rust, dependency update
or extra transport command type. Fixed positive component checks keep
decorative gold and unknown overlays separate from terminal authority.

The terminal entry combines the current mode, the delivered card action,
its existing effect threshold and two consecutive fresh observations.
Reward-skip authority is checked again before delivery; bounded waits,
click budgets, STOP and uncertain-input refusal limit automatic continuation.
The retained preview remains advisory and supplies no input permission.

The installer pins complete file bytes and modes to the promoted base and
release states, backs up affected files privately and retains the Git index.
Unknown edits, staged affected files, links, wrong HEAD and checksum failures
refuse replacement. Rollback restores the exact pre-install source state and
requires a rebuild before launching the restored executable.
