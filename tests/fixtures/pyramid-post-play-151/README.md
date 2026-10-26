# Actual post-Play dealing frame (2.9.150)

Unchanged original from `new-game-entry-150-20261025-k2c0rknm/packet/last-observed.png`. SHA-256 `072f63a1f5c26df9a333b98a4f0f4972e35162a007c42a734d7a7d7a5f668388`. Native 1920×1080, captured at session +779.685 s after Play (+778.586 s), TCP session 9/request 479/frame 7360. The saved GAME WIN image `000779.685_000010.png` is pixel-identical. It is a fresh incomplete/moving board, not a retained Recon frame. Production v23 refuses internal slot 12 (5|3). The later manual START and Recon used new frame IDs 7361–7364; there is no saved complete follow-up PNG from that run.

Regression sequences pair these actual incomplete pixels with separately retained complete boards. `normal_start` explicitly composes genuine normal-one header/footer pixels into older complete fixtures; these are labelled compositions, not the original recorded settling sequence. No original ordered route or animation duration is inferred.

`dealing-run4.png` is the unchanged later failure packet `pyramid-error-1792963515394-412320-4/last-observed.png`: fresh TCP session10/request488/frame7848, +1277.125s after Play +1276.011s. Its cards are visibly still being dealt. The same full-header/uniqueness/pile policy applies; no complete successor image is retained from that exact sequence.
