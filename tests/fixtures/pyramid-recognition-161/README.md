# Third native Coral rendering regression for 2.9.161

The byte-original 1920 by 1080 PNG is the final observation from 2.9.160
Request 1: the first MATCH targeted 5 Spades at 7|1 and 8 Spades at 7|3.
Two gestures were acknowledged and zero logical actions were verified.
The final refusal was `execution slot 21 lacks positive absence`; the earlier
legend mismatch is separate. This fixture does not replay the failed request.
The original complete deal retains this request's provenance.

The complete native 144 by 34 footprint at (286, 588) differs from the two
earlier calibration bounds at 1,467 channels across 620 pixels, by at most
five levels. Its RGB file is a direct crop. `calibration.json` records hashes,
the original archive and all three measured references.

The correction extends each fixed channel's bounds only to the minimum and
maximum of its three measured values, inclusive. All 14,688 channels are
checked; 7,862 unchanged channels remain exact and the maximum span remains
26. There is no additional tolerance, rescaling, mask or inferred card identity.
Strong-white positive paper vetoes the exception. The existing calibrated
bare-art exception precedes the generic aged-material check: genuine bare
Coral itself contains aged-colour pixels. That ordering is unchanged.

Six regressions cover the complete remaining position and exact piles, crop
provenance, one-level excursions beyond both measured extrema, changes to
an exact channel, inserted white/aged/possible card material, remaining card
identity and unreadable-rank refusals, and the identical pattern shifted to
7|2. The two earlier calibration renderings retain their existing regressions.

These are three calibration originals and explicitly synthetic negatives.
They provide neither an independent held-out rendering nor fresh live effect
or GAME WIN acceptance. The rendering cause and coverage of future Coral
appearances remain unproven. Tests request no captures and send no guest input.
