# Pyramid native recognition regressions for 2.9.158

These three PNGs are unchanged native 1920×1080 originals. They are repository regression inputs; other error packets and private diagnostics remain outside the repository.

- `five-spades.png` is the retained dark-background Five of Spades refusal. The earlier REDEAL frame and subsequent explicit Recon have identical game-area pixels. Its fixed `5|4` corner supplies the complete extra Classic Five-of-Spades appearance. The reference requires an accepted Five rank and an unsplit separator; it is not added to other ranks' suit competition. The retained Seven of Clubs and Five/Ten of Clubs remain regression checks.
- `underwater.png` is the retained initial Underwater board: 28 labelled tableau cards, Eight of Clubs in LEFT and positively empty RIGHT.
- `underwater-three-nine.png` was supplied by Charlie using the application's Capture PNG at 100% scale. It shows the missing Threes and Nines, including highlighted Nines, Nine of Hearts in LEFT and Eight of Spades in RIGHT. It is a card-reference frame with a stock draw already made, and cannot prove an untouched starting position.

`calibration.json` records source hashes, native crop locations and the complete 13-rank/four-suit Underwater calibration. RGB references are extracted directly, without resizing, recolouring or resampling. The Classic reference lives in `src/pyramid_recon/vision/foreground_card/classic-spade-158.rgb`; the complete Underwater asset lives in `assets/pyramid-glyphs/underwater-corners-v1.json`.

The thirteen Underwater reference corners are training data. Tests also check the other 46 visible card corners across these same two originals, including the second Three, other Nine suits and highlights. This is bounded native evidence, not arbitrary-font or whole-deck acceptance. No black Three is present in these two frames.

Underwater's measured brown ink and native Ten width are confined to its own complete family. The fixed card-relative separator is y30. The genuine Queen of Hearts' rank and suit form a connected component, so this separator retains both complete glyphs. Its native raster envelope allows two pixels of measured red/brown variation. Classic, Retro and Western retain their existing ink, geometry and size policies. Matching quality, different-label gaps, same-colour competition and unique complete-card identity remain required.

Negative derivatives are generated in test memory and explicitly synthetic: missing rank/suit, a clipped Ten zero, mixed brown/red print, and erased ranks used to exercise latest-frame failure explanations. Original PNG bytes are never rewritten. Production entry/transition tests run through isolated AF_UNIX mocks with repeated retained frames; these are synthetic observations, not new guest captures or proof of live effects. Mocks permit capability/capture operations only. STOP/context revocation and the occupied-RIGHT entry refusal remain mandatory.
