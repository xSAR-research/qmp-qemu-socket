# Retained 158 failures for the 159 correction

The two native 1920 by 1080 PNGs and `deal.json` are byte-original evidence.
They were copied before further operator testing. Tests perform no live guest
capture or input.

- `coral-effect.png` is the final effect observation from request 1 in 158,
  packet `pyramid-error-1793157728722-2469111-0`. Its first pair removed Five of
  spades at 7|1 and Eight of spades at 7|3. The original complete identity owner
  is `deal.json`; LEFT remains Eight of clubs and RIGHT is positively empty.
- `normal-two.png` is the complete starting board in request 4, packet
  `pyramid-error-1793158218294-2469111-3`. Its complete header says Boards 2 of 3;
  LEFT is Queen of spades and RIGHT is empty. The user's earlier Continuous
  standard-theme acceptance was explicitly retracted.

`underwater-coral-clearance.rgb` is every RGB channel from the fixed 144 by 34
clearance at (286,588), below original location 7|1. The complete native artwork
must match exactly. This measured proof is confined to that rectangle, and
positive paper presence still refuses. It distinguishes this known bare artwork
from material; it does not change the generic paper, aged-material, chroma or
surface thresholds, and it is not a general underwater-background classifier.

`normal-header-two.rgb` contains the complete 160 by 32 header at (1658,46).
It maps only to one completed board out of three, using the existing per-channel
limit 12, complete New-footer requirement and conflicting-ordinal refusal.

Both references are calibrated from the corresponding retained image. Those
positive tests are training regressions, not independent held-outs or fresh
guest acceptance. The unchanged earlier corpus supplies regression coverage.
Negative frames exist only as synthetic clones in tests: material overlays,
changed artwork, a shifted copy, erased remaining-card ink, damaged headers and
missing footers. The original PNGs remain unmodified.

The native entry tests use the real worker adapter and a private AF_UNIX mock.
Two repeated fixture payloads exercise two distinct mock acquisitions; they are
not two fresh captures of the running guest. The mock accepts only capability
negotiation and screendump, and STOP/context tests terminate after one capture.

`calibration.json` records the exact source and crop hashes. No other temporary
error reports, logs or screenshots are included in this fixture directory.
