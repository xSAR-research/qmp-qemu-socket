# Pyramid Score challenges: scoring target and manual collection proposal

Discussion recorded: 29 October 2026.
Retention review: 31 October 2026.

**Historical design note — no implementation authority.** Retained for the scoring
observations and design trade-offs below. Existing Maximum behaviour was accepted
as sufficient; the unapproved collection/projection/manual-handoff proposals are
not current requirements. Statements labelled current below describe the original
2.9.162–2.9.164 discussion and must be checked against source before future work.

## Purpose and status

Record the scoring discussion, Charlie's verified observations and the proposed
way to collect points that a tableau-clearing route can leave behind. The
requirement belongs to a recognised **Score challenge**. It should apply
automatically because of that challenge objective, rather than depend on the
operator selecting **Maximum card matches**.

This discussion was recorded while Charlie was testing v2.9.162. It records no
new live-test acceptance. Charlie subsequently confirmed the existing Maximum
behaviour is sufficient for the current request. Version 2.9.164 adds only the
separately approved RECON route policy: Score/Card selects Maximum, Board selects
Shortest, and normal games preserve the selected objective between boards.

Automatic stock-King collection, extra DRAW-pair matching, King-free projection
and the Single board handoff below are historical proposals. They are outside
the 164 scope and have no implementation approval from this note. The Score-only
activation discussion below refers to those proposed scoring algorithms, not
the separate 164 challenge-kind route selection.

## Points actually awarded

Charlie has observed and confirmed that a successful removal operation awards
**five points** in the relevant Score challenges. A single King and a matching
pair each earn five points. This is **not five points per individual card**.

| Successful removal operation | Cards removed | Observed score increase |
| --- | --- | --- |
| One tableau King | One | +5 |
| One DRAW-pile King | One | +5 |
| A matching pair from the tableau | Two | +5 |
| A matching pair using one tableau card and one DRAW card | Two | +5 |
| A matching pair using DL and DR | Two | +5 |

These are Charlie's game observations, not a new independent automated score
measurement by the agent. A future implementation must check its predicted
benefit against actual displayed progress. It must not treat the number of
removed cards, acknowledged input operations or cleared boards as observed
score.

For `k` extra Kings and `p` extra pairs, the base benefit discussed here is
`5 * (k + p)` points. Four stock Kings could therefore provide twenty points;
a QUEEN/ACE pair provides five points, not ten. These examples do not claim
that every deal has four stock Kings or that all potential pairs are reachable.

## Why this is specific to Score challenges

Charlie reports that the score is displayed when playing a Score challenge.
Other challenge types display their own objective. Normal gameplay measures
boards cleared.

| Recognised context | Relevant displayed objective | Scope of this proposal |
| --- | --- | --- |
| Score challenge | Score progress and target | Apply the agreed scoring policy automatically |
| Another challenge type | That challenge's own objective | Preserve its existing objective and behaviour |
| Normal gameplay | Boards cleared | Preserve normal board-clearing behaviour |
| Unknown or unsupported context | No trustworthy objective | Do not infer a Score challenge or gain input authority |

The existing model has a typed `ChallengeKind::Score`. Future activation should
use positively recognised, current challenge evidence and its target/progress.
A generic number, decoration or old challenge selection must not stand in for
that evidence. Score display is part of the context Charlie described, not
permission to act on an otherwise unsupported scene.

This refines the original suggestion to attach King collection to **Maximum
card matches**. That route goal motivated the discussion, but it is not the
correct user-facing trigger for the requirement. The temporary **Single board**
condition below limits the proposed manual handoff; it does not redefine the
underlying reason to collect score.

## The problem on tight targets

Some Score challenges have very little spare scoring capacity. Charlie has
verified cases where the target requires collecting DRAW-pile King points.
The final available board can be unsolvable, leaving the player short of the
target if earlier boards were cleared without collecting available extra
points. There is then no assumption that another solvable board will make up
the deficit.

A route can clear the tableau successfully and still leave scoring operations
unused in the DRAW piles. Consequently, these are separate outcomes:

- A legal route was executed.
- The tableau was cleared.
- Additional stock points were collected.
- The actual Score-challenge target was reached.

Neither tableau-clear nor input acknowledgement proves the last two outcomes.
Recognised challenge completion can also occur before the tableau is empty and
must retire input normally; a collection policy must not override completion.

## Current planner behaviour and the earlier wording correction

The current **Maximum card matches** description seeks a win first and otherwise
counts removed tableau cards. Stock-only matches add no tableau progress.
That internal fallback count is not the game's displayed score.

Thus the earlier statement that stock Kings receive no fallback objective credit
means that the planner awards zero **tableau-removal credit** for that action.
It does not mean that the game awards zero points. A stock King can earn five
game points while adding zero to the planner's tableau count.

Stock King actions already exist in the physical rules. The present planning
goal is an empty tableau, and successor generation stops at that goal. An
automatic promise to collect all required stock Kings therefore has to account
for them before the final tableau removal. Appending a sweep after tableau-clear
does not resolve that boundary.

The application also validates partial endpoints using tableau counts and has
stock continuation that currently emits DRAW/RECYCLE operations. Those
contracts must be considered before changing the objective, permitting
King-only benefit or adding stock-pair collection. Adding game points to the
existing tableau-count scalar would mix different units.

## Longer-term King proposal: planning view and physical execution

Charlie's preferred approach is to derive a stock view without Kings after
RECON, calculate the route using that view, and preserve the original King
positions for removal as the physical DRAW piles are traversed.

The original complete RECON deal and original stock order must remain available.
Removing Kings from a derived planning view does not mean deleting their
identities from the original acquisition or assuming they have physically
disappeared. The application needs an explicit mapping between:

1. Original card identities and stock positions obtained by RECON.
2. The King-free stock view used for route calculation.
3. The complete physical sequence of MOVE, RECYCLE, MATCH and KING actions.

For example, a King at original stock position five can be predicted to arrive
at DL once the four preceding stock cards have been advanced or removed in the
relevant physical sequence. Four arbitrary actions or four tableau-only matches
do not establish that condition. `DL` and `DR` are the current visible pile tops,
not permanent names for original stock positions. Recycles and removals must
update the physical model consistently.

Charlie has asked for modular, self-contained instruction calls, with examples
such as `click_draw_left()` and `click_row_col(row, column)`. These are conceptual
interface examples, not a claim that these exact functions currently exist.
The intent is for each operation to own the deterministic mapping and bounded
input sequence. Tableau locations remain one-based fixed original positions.
The operation must still honour current context, STOP, VM/pointer checks,
delivery acknowledgements and verified effects; a predicted stock position
alone must not authorise a click.

The alternative discussed was a King policy in the existing full physical-deck
planner. That avoids the projection/lifting layer but still needs explicit
King progress, terminal rules and continuation handling. Neither approach was
implemented in v2.9.162. The projected approach remains Charlie's preferred
proposal to assess, not an agreed implementation contract.

Any lifted route must be independently replayed against the original deal before
it becomes executable. Inserted King removals must be part of that complete
physical sequence. Planning over a reduced stock does not by itself guarantee
that every original stock King is visited before an early tableau clear.

## Why extra DRAW-pair matching is harder

King removal does not consume a complementary rank needed to make thirteen.
A DRAW-pair match does: a card used for an extra stock pair is no longer
available to remove a future tableau card.

Charlie's example is a **QUEEN at DL and an ACE at DR**. They form a pair, but
before removing them he checks which QUEEN and ACE cards remain on the tableau:

| Remaining tableau ranks | Possible reason to retain the DRAW cards |
| --- | --- |
| A QUEEN remains | The DRAW ACE may be needed for that QUEEN |
| An ACE remains | The DRAW QUEEN may be needed for that ACE |
| Both ranks remain | Either or both DRAW cards may be needed |
| Neither rank remains | Neither DRAW card is required for a direct future tableau match |

The check concerns **all remaining tableau cards**, including covered cards
known from RECON, rather than only those currently exposed. Original identities
and current occupancy must stay separate so a removed card is not counted as
still required.

This is a useful conservative heuristic. A remaining QUEEN does not prove that
this particular DRAW ACE is essential: another ACE or a different route may
exist. Conversely, if neither QUEEN nor ACE remains, the direct tableau demand
for these two ranks is absent. That is the specific part of Charlie's reasoning
that is understood and recorded here.

Absence of tableau demand is not a proof of globally optimal stock play. A
candidate pair must still be accessible and legal, and its removal changes the
stock/pile sequence. General automatic pair collection would need to consider
card counts, alternative pairings, access order, remaining recycles and the
prepared route. Suit does not change the QUEEN/ACE rank complement, but card
identities still matter to physical replay and recognition.

Charlie considers these difficult cases relatively uncommon. A complete score
optimiser could be disproportionate as an immediate response, especially while
v2.9.162 is undergoing live testing and game-type regressions are a concern.

## Proposed stopgap: Single board manual handoff

When **Single board** is selected and the current board is positively recognised
as a **Score challenge**, execute a clear route only up to the position
immediately before its final tableau-clearing action, then stop for the operator.

This handoff should be automatic for that context. It should not require an
additional Maximum card matches selection or a new manual scoring checkbox.
It is a proposed future behaviour, not an existing v2.9.162 feature.

For a complete clear route, the intended sequence is:

1. Complete supported RECON, route calculation and independent physical replay.
2. Identify the complete logical action that would clear the remaining tableau.
3. Execute and verify the safe prefix preceding that action.
4. Withhold the entire final action, including every click or key operation it
   contains. Do not stop between movement, button-down and button-up events.
5. Retire automatic execution at the verified position and explain that the
   controller has stopped before the final move for manual score collection.
6. Leave the current board available to Charlie. Do not automatically clear it,
   drain the stock, REDEAL, advance, restart or send cleanup input after handoff.

The result is a planned manual handoff, not a reported board clear, Challenge
Complete result, generic failure or permission to continue from a stale plan.
The existing completion and transition machinery must not run as though the
withheld action had been delivered.

### Charlie's manual procedure

Charlie can use the game's UNDO control repeatedly to unwind earlier play and
revisit stock positions. He can then remove exposed Kings or match suitable
DRAW pairs, using the remaining tableau to judge whether a complementary card
is still needed. This preserves human judgement for the uncommon complex cases.

The proposal does not ask the controller to automate UNDO or decide which moves
Charlie should reverse. It does not claim that every game state offers unlimited
UNDO, that UNDO preserves earned score, or that every extra five points can be
recovered. The intended result must be checked against the final displayed
score after the manual sequence, including any reversals of earlier scoring.

Manual changes invalidate the prepared route and its execution authority. Any
later automatic work requires fresh supported evidence and the appropriate
acquisition/planning contract. No automatic resumption of the old route, silent
replay or assumption that UNDO restored the previous board is permitted.

## Boundaries to settle before implementing the stopgap

| Case | Required treatment or unresolved detail |
| --- | --- |
| Normal play or another challenge kind | Preserve existing behaviour; do not apply this scoring handoff |
| Continuous or Manual method | The proposed stopgap is scoped to Single board; agree any broader behaviour separately |
| Clear route with several actions | Execute and verify the prefix; withhold the final whole clearing action |
| Clear route with one action | The prefix is empty; hand off at the verified starting position without sending that action |
| No route or an empty route | Report the actual condition; do not manufacture a clear or a final move |
| Unsolvable board with a partial route | At the game's no-more-moves endpoint, leave its existing End game / Undo last move dialog for Charlie; the dialog supplies the manual recovery point |
| Target reached before the planned handoff | Recognised challenge completion takes precedence and retires input |
| Unknown challenge context or changed settings | Existing refusal/revocation rules take precedence |
| STOP or uncertain delivery | Preserve current stop/recovery behaviour; do not replay input to reach the handoff |
| Manual UNDO or other manual moves | Treat the old prepared route as invalid; agree supported reacquisition before resuming automation |

Charlie confirmed that an unsolvable route reaches a game dialog with two
options: **End game** and **Undo last move**. Choosing **Undo last move** gives
him access to manual UNDO, including repeated undo operations to revisit the
stock. The game already supplies this recovery point; the partial-route handoff
is resolved by that existing dialog and does not need an invented final
tableau-clearing move.

The two cases therefore have different endpoints. For a route that clears the
board, the proposal withholds the whole final clearing action. For an unsolvable
route, it leaves the existing dialog available for Charlie to choose **Undo last
move**. Automation must not choose either dialog option, start another board or
resume input after the manual handoff. A search limit or controller refusal is a
separate condition and must be reported as it occurred. This dialog behaviour is
Charlie’s reported game behaviour; this documentation task did not operate the
guest or newly verify it.

## Acceptance checks for a later implementation

These are proposed checks, not tests run by this documentation change:

- Activate from fresh recognised Score-challenge evidence and Single board
  selection, independently of the route-goal choice.
- Preserve other challenge kinds, normal gameplay and other games.
- Verify the complete pre-final position and prove that no part of the final
  clearing action or subsequent transition input was sent.
- Cover one-action routes and honest empty/no-route results. For an unsolvable
  route, confirm the existing End game / Undo last move dialog remains available
  for manual recovery and neither option is selected by the controller.
- Honour actual challenge completion, STOP, context changes and uncertain
  delivery at every existing authority boundary.
- Show a clear manual-handoff state and leave the game available for operator
  input without an automatic drain, REDEAL or restart.
- In a live Score challenge, confirm the displayed five-point increments for
  King and pair removals and the final net score after the chosen UNDO sequence.
- Confirm that manual changes cannot resume a cached route and that subsequent
  automatic work uses fresh supported acquisition and planning.

## Related source and delivery records

- [Workflow modes and route-goal descriptions](../src/pyramid_workflow_model.rs).
- [Typed challenge objectives](../src/pyramid_progress/challenge.rs).
- [Computed route validation and execution](../src/worker/computed_route.rs).
- [Existing stock continuation](../src/worker/computed_route/continuation.rs).
- [Physical Pyramid rules](../../xsar/src/games/pyramid/rules.rs) and
  [current tableau-based search objective](../../xsar/src/games/pyramid/search.rs).
- [v2.9.162 verification](Verification-2.9.162.md),
  [current milestone receipt](milestones.md) and
  [current handoff](handoff.md).

This note is retained at `docs/scoring-target.md`; its former location was
`doc/scoring-target.md`. Operational receipts remain in the same `docs/` directory.
No implementation, version increment, guest input, controller restart or promotion
is authorised by this historical design record.
