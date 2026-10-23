# Native search-stop regression

These unchanged originals retain Charlie's 21 October 2026 counterexample:
the board has a winning route, but protected 2.8.120 stopped its Shortest route
search after retaining 2,000,000 histories in 2.105776793 seconds. That cutoff
does not prove the absence of a winning route.

`last-move.png` shows the operator's solved position before the final match:
6 of clubs in DL with 7 of diamonds at the fixed one-based location `1|1`.
`deal.json` is the original complete acquired inventory. The planning JSON/log
record the failed attempt; `provenance.json` records original paths and SHA-256.

The app regression independently replays a full Maximum cards win on this exact
inventory. A separate exact-search regression crosses the old history cutoff,
then requests cooperative STOP from a real progress callback and requires that
no executable route be returned. This deliberate test STOP is not a production
search bound or evidence that exact search found its minimum route.

These are native inputs and offline regression evidence. They do not establish
live acceptance of the corrected executable or authorise replay on the guest.
