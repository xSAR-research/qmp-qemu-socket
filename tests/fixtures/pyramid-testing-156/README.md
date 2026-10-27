# Pyramid testing observations for 2.9.156

These five 1920×1080 PNGs are byte-for-byte copies of the observations named in
Charlie's `Testing_v2.9.155.md` and the subsequent request 12 report. They were retained while the qualified 2.9.155
executable remained open. Copying them requested no capture or guest input.

| Fixture | Original reported packet | Measured role |
| --- | --- | --- |
| `score-entry.png` | `pyramid-error-1793081742552-15609-0` | Untouched Score 0 / 2,500 entry. |
| `normal-final.png` | `pyramid-error-1793082428568-15609-1` | Normal third-board empty tableau, exhausted DL and exact remaining Q♣ in DR. |
| `normal-final-king-spades.png` | `pyramid-error-1793094093873-15609-8` | Independently reported normal third-board final with exhausted DL and exact remaining K♠ in DR. |
| `score-complete.png` | `pyramid-error-1793088204595-15609-2` | Large Challenge Complete counting dialog, with actual Score 1,755 / 1,700 overshoot. |
| `normal-reward-small.png` | `pyramid-error-1793089969712-15609-7` | Small normal Congratulations counting dialog. |

Only these five reported packets are diagnostic inputs. Other errors in the running
session log were explicitly excluded because they followed operator mode exits.
The private evidence manifest retains original file hashes and the testing table;
the repository contains only observations used by the regression tests and their
calibration provenance.

`calibration.json` gives the original PNG, checked native rectangle, byte count and
SHA-256 for every tightly packed RGB reference. The complete Score word keeps the
existing inclusive channel limit of nine. Each new terminal appearance compares
all pixels in its own coherent title, structure and instruction roles exactly;
changing XP/level values are excluded. No recognition limit was increased.

The normal final-clear correction proves exhausted DL independently of the exact
replayed DR card. It retains all tableau absence/material vetoes, native scene and
modal checks, and final-phase authority. Ordinary position recognition remains
strict.

The small reward was originally observed under `Continuous - no challenge`.
The new tests exercise its recognition and the existing guarded
`Repeat normal games` restart path. They do not establish a no-challenge restart.
The restart fixture supplies a private third-clear permit, retained Level Up
pixels and explicitly synthetic New Game / Play menus. Its repeated images after
mock acknowledgements are scripted test inputs, not observations of guest effects.

Negative tests modify copies in memory: erase complete scene roles, alter a
challenge binding or expected DR, retain an expected tableau card, and revoke
STOP/context authority. These derivatives are not native calibration samples.
The original PNGs are never edited. Passing host tests does not replace fresh
Beast acceptance or establish support for untested board themes.
