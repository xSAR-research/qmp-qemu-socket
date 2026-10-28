# Native transition and Recon evidence for application 2.9.157

These three PNGs are byte-original retained captures from application 2.9.156.
Their source hashes and original paths are recorded in the private delivery's
`fixture-originals.json`. No colour, crop, geometry or recognition limit changes
are calibrated from these files.

- `next-board.png`: Request 2, Continuous / Repeat normal games / Exact A*.
  This was transition observation six after Board 1 cleared, with the complete
  untouched Board 2 tableau, LEFT Seven of spades, positively empty RIGHT and
  the literal normal Board 2 of 3 header. The earlier five transition frames
  were not recorded. Its production native-v23 reader result was independently
  verified before the correction.
- `undo-baseline.png` and `undo-exhausted.png`: Request 4, Challenges / Exact A*,
  on the dark patterned background. These retain the same complete 28-card
  tableau before and after the acknowledged stock scan. The last image has
  positive empty LEFT, RIGHT Nine of spades and an unmatched Undo All icon.
  Endpoint images alone do not prove the intervening ordered stock or restoration;
  the original private JSON and session log preserve that evidence separately.

The native transition tests supply the original PNG through separate isolated
AF_UNIX mock captures. Five unqualified frames, repeated confirmations and test
clear-handoff/original-board state are explicitly synthetic. They exercise the
actual production capture, recognition, transition and authority checks without
contacting QEMU or sending guest input. Mock repetition is not a retained pair
of actual fresh guest captures and establishes no measured guest transition time.

The pure Recon event tests model a confirmed complete 24-draw history, one
calibrated Undo All attempt, two matching full dialog observations and two original
board observations. Missing dialog, STOP and uncertain Undo All delivery refuse
confirmation/publication and never retry. They do not establish live restoration.

These are the only native images promoted for the new regressions. Bulk evidence
and the operator's archive remain outside the repository. Fresh 2.9.157 guest
acceptance is a separate operator test.
