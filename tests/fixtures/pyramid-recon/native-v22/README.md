# Native-v22 — measured Jack of diamonds

The unchanged native 123 first-failure frame is recon-jd.png. Its 28 tableau
cards and LEFT 8C are visible; the header is Sixes cleared 0/12, independently
of the operator's SingleBoard/Manual selection. Sixes challenge-target support
is not added. Recon stopped on its first capture, before any DRAW or Undo input.

The previous native-v21 reader accepts J at fixed 4|1 but refuses its diamond
with exact error 2869/23826 = logged 120414 ppm, beyond 120000 ppm. A single
measured 20×16 full RGB crop at [594,302,20,16] is appended to both suit banks.
Diamonds variants increase 13 to 14 under the unchanged capacity 16. The old
rank/suit limits 220000/120000 ppm, gap 80000 ppm, one-pixel alignment, paper
and exclusive-colour gates are unchanged. Removing the two identical appended
variants and restoring profile_id reconstructs native-v21 exactly as decoded.

provenance.json records exact source/crop/profile hashes. Both reported packet
states/logs and diagnostics are retained under the permanent validation root,
with one copy of four byte-identical baseline/latest native frames. This is an
exposed calibration regression, not a holdout, new capture or live acceptance.
