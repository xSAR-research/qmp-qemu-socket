# Klondike level 841: current OK caption without exact-black pixels

These five PNGs are byte-original GAME WIN captures from the reported
qmp-qemu-socket 2.9.153 Computer Vision Continuous run on 26 October 2026.
They come from `qmp-qemu-socket-1793007991222.log`; the first-failure case
retains the complete log, all 19 original GAME WIN images and the earlier
Pyramid failure packet separately. No extra guest capture was requested.

| Image | Reported sequence position |
| --- | --- |
| `010725.982_000015.png` | Independently confirmed completion before the single score-skip input |
| `010727.677_000016.png` | First OK search; no measurable settled button |
| `010728.791_000017.png` | Second OK search; first settled body and exact-black run at `(944,771)` |
| `010729.141_000018.png` | Third OK search; same body, zero exact-black pixels in the configured search region |
| `010729.531_000019.png` | Fourth and final OK search; same body and persistent absence of exact-black pixels |

The measured button is `(839,723,237,71)`. The RGB patch in
`src/terminal_controls/ok_readiness/klondike-ok-caption.rgb` is copied only
from image 17 at X=930–989, Y=741–775. Its 60 by 35 pixels contain both O
and K, their holes and surrounding button material. Its SHA-256 is
`dc6e38534584a97dbd978df0a7b0b148d34ff23edc192952ec01765b98763f0f`.
Images 18 and 19 are held out; their maximum per-channel differences from
this complete patch are 10 and 3 respectively. The matcher requires every
pixel and channel within 12, current measured settled geometry and matching
consecutive body measurements. It has no partial-match allowance.

The correction applies only to the Klondike profile. The shared exact-black
scanner and the other four profiles retain their prior policies. Unit tests
also erase O, K or both from a copy while preserving the measured button;
these controlled images must refuse. Those altered copies are synthetic
test inputs and are never presented as guest captures.

The production-driver regression consumes the original first three OK
observations and attempts OK once. Repeated image 19 after that simulated
acknowledgement deliberately tests an unchanged modal: the driver stops
without replaying OK or clicking its underlay. This sequence establishes
offline control behaviour only. No image showing the real effect of an OK
click was supplied, and fresh live guest acceptance remains outstanding.

The Failure Details correction uses the existing immutable worker failure
event. Separate tests verify that current Klondike run facts replace older
Pyramid A* fields and remain frozen when the details window is reopened.
