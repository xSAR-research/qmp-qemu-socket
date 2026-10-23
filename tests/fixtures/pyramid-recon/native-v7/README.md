# Pyramid v7 native calibration fixture

This fixture records the bounded append-only repair approved on 19 October 2026.
Host verification **passed with the retained pre-existing package-format
failure**. It supplies native classifier evidence, not a
complete stock acquisition, live restoration or new guest input acceptance.

## Original and profile identity

`baseline.png` is the unchanged original 1920×1080 frame from the first refused
attempt. Its SHA-256 is
`f00076f38a2906dc17d1dc5b7c54806b6285f363f936924e5c785785e9173edf`.
The run captured one frame and attempted **zero guest inputs**; output publication
was not attempted. Undo All/restoration was unnecessary because no input was sent.
This no-input refusal is separate from the previously accepted v6 acquisition.

`profile.json` is schema 1 with ID `pyramid-native-1920-classic-blue-v7`.
It extends the retained v6 profile, SHA-256
`2d28864f120b4fc47600721a78658fd1880e97473377a90929de8aebe4b775fd`.
The prepared profile is **806,442 bytes**, SHA-256
`9517ce85382ec0c977e735c79871c1e2f8110b7265343dc1140ac45d506e3e08`.
Completed host results and the release hash are in
[Verification-2.4.101.md](../../../../docs/Verification-2.4.101.md).
`provenance.json` records crop sources and measured held-out results;
`INCOMPLETE.json` retains the first refusal. Historical
v4/v5/v6 fixture files remain unchanged.

## Source crops and held-out refusals

Source indices are zero-based original tableau slots. Human row/column locations
remain fixed and one-based. Both source identities were already recognised by v6.
They are separate from the held-out failure locations.

| Component | Source slot / row, column | Original RGB crop (x,y,width,height) | RGB SHA-256 |
| --- | --- | --- | --- |
| Rank 2 | 7 / 4,2 | `(792,278,27,24)` | `9250b21fe48cea0bb48b51673a7cbd122557f98a226e2f42202eded67a5cd1f7` |
| Spade | 25 / 7,5 | `(1096,462,20,16)` | `35415efed2a2501d217db7805338403d028c7d1ae10302ca8830c741f866a350` |

These are hashes of native crop **RGB bytes**, not hashes of newly encoded PNGs.
The rank crop contains 27×24 pixels and the suit crop 20×16 pixels. Pixels are
neither recoloured nor rescaled, and no complete card is synthesised.

| Held-out evidence | v6 best occupancy XOR distance | v7 best distance | Unchanged cap |
| --- | --- | --- | --- |
| Rank 2, slot 6 / row 4, column 1 | 28 | 19 | 24 |
| Spade, slot 23 / row 7, column 3 | 12 | 7 | 8 |

Both reader banks append the same two measured variants. Per reader, total rank
variants rise **34 → 35** and suit variants **36 → 37**; rank 2 rises **3 → 4 of 8**
and spades **9 → 10 of 16**. Existing variants, all ROIs/masks, caps 24/8,
alignment 1, colour/paper checks and board/dialog signatures remain unchanged.
The source and held-out samples demonstrate the reason for the repair without
weakening those bounds.

## Verification and live boundary

The locked check, 10 focused parser/reader regressions, all 876 tests including
ignored tests, strict private docs, Clippy and standalone new-module formatting
passed. The regressions cover the full new tableau/initial piles, all 29 v6
visible cards and 104 historical labelled observations in both reader banks.
Native release, executable `--help` and final preservation checks passed. Package
formatting fails on differences proved identical to pristine HEAD, without
legacy/sibling reformatting. The exact profile identity is recorded above; the
executable hash is in the verification record. The repository `profile.json` is the
stable operator selection; no temporary runtime copy has been created for this
repair. The 2.2.100 collector loads this compatible schema afresh for each
reconnaissance request; profile compatibility requires no upgrade/restart. The
Rust source addition is test-only and changes no production collector logic.

A successful saved-frame regression cannot prove the unseen ordered stock,
advancement, restored original board or current guest rules. Fresh v7 live
acquisition/restoration and M09 human console/game comparison remain pending.
Final checks found no collector process. PID 521945/2.2.100 is historical and its
exit time/cause are unknown. The next live run needs an operator application
launch. Existing GAME WIN images have been privately backed up with all hashes
checked and original bytes unchanged; see the verification record. Record the
actual new process, selected profile hash and export producer.
Neither profile selection nor a successful build establishes live acceptance.
Await Charlie's confirmation of the same untouched supported board and the
form's run confirmation. Every application start clears old images. No automatic
input, retry or deployment follows from these
fixture files.

See [the profile contract](../../../../docs/pyramid-recon-profile.md) for the
classifier/controller boundary and retained schema/recognition limits.
