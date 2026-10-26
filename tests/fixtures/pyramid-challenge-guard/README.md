# Continuous challenge-guard failure

The unchanged 1920 x 1080 `last-observed.png` comes from the retained
2.9.139 failure packet `pyramid-error-1792847195021-1000809-1`. Its SHA-256 is
`5b3d89f45e1ac97e4794126c53ace823ed7eabbf4e5dd4ddb49bfc67bb36592c`.

The guest displays Jacks 8/10 while a King of hearts moves across the header.
The strict challenge reader returns Unknown. The inner effect observer allowed
read-only retry, but the outer Continuous/Challenge session guard rejected that
same frame before final partial confirmation and stock continuation. The run
recorded 33 planned actions, 32 verified effects and 51 acknowledged gestures.

This original tests header classification and guard phase handling only. It does
not supply an ordered original route or execution authority. Separate native
driver tests use explicitly synthetic header/state compositions and an independently
replayed two-MATCH prefix; they do not replay this consumed live run.
