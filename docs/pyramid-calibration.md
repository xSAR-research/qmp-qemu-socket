# Pyramid target calibration — 1.0.1 candidate

Date: 2026-09-26. Request: [Issue #1](https://github.com/xSAR-research/qmp-qemu-socket/issues/1).
Overlay request: [Issue #3](https://github.com/xSAR-research/qmp-qemu-socket/issues/3).
Base: `a13e538871795d790b1d63939d089e0a22ba7f67` (1.0.0).

## Contract and evidence status

- **Verified from supplied images:** all 20 PNGs are 1920×1080. Coordinates include the guest window chrome and taskbar. Origin is top-left; x increases right, y increases down. Rectangles are half-open `(x, y, width, height)`.
- **Measured:** the row/column card origins below, Left/Move/Right faces, and shared toolbar icon positions. Card-face edges vary by one pixel through fractional rasterisation; a uniform 140×187 envelope covers the faces. Shadows and Solver halos are separate from these face envelopes.
- **Inferred:** the apex's full height is occluded, so it uses the shared card dimensions. Hit points are proposed geometric interiors, not observed successful input. Tableau points `(x+70, y+24)` lie in the exposed top strip, outside all lower-row rectangles.
- **Unknown:** guaranteed click effects, one-click pair removal, animation timing, complete halo classifier behaviour and Move-versus-Recycle state discrimination. Attachment ordering alone does not prove transitions or which side was clicked.

This candidate implements geometry and read-only overlays. Overlays are drawn by the main preview painter only; neither the capture dialog image nor the saved original PNG is annotated. It does not enable Pyramid Step Once, Multi-Step, reconnaissance, Undo All, confirmation clicks, Solver activation or any other Pyramid guest input. Empty executable profile lists and the worker's independent mode gate remain in place.

## Tableau targets

Rows are numbered from apex (1) to bottom (7), columns left to right. Stored priority remains Move, Left, Right, then row 7 to row 1. The preview outlines every slot even if its card has been removed; these are static calibration targets, not presence detections.

Every card envelope has width 140 and height 187.

| Target | x | y | Hit x | Hit y |
|---|---:|---:|---:|---:|
| r1c1 | 890 | 112 | 960 | 136 |
| r2c1 | 790 | 165 | 860 | 189 |
| r2c2 | 990 | 165 | 1060 | 189 |
| r3c1 | 690 | 218 | 760 | 242 |
| r3c2 | 890 | 218 | 960 | 242 |
| r3c3 | 1091 | 218 | 1161 | 242 |
| r4c1 | 589 | 272 | 659 | 296 |
| r4c2 | 790 | 272 | 860 | 296 |
| r4c3 | 990 | 272 | 1060 | 296 |
| r4c4 | 1191 | 272 | 1261 | 296 |
| r5c1 | 489 | 325 | 559 | 349 |
| r5c2 | 690 | 325 | 760 | 349 |
| r5c3 | 890 | 325 | 960 | 349 |
| r5c4 | 1091 | 325 | 1161 | 349 |
| r5c5 | 1291 | 325 | 1361 | 349 |
| r6c1 | 389 | 378 | 459 | 402 |
| r6c2 | 589 | 378 | 659 | 402 |
| r6c3 | 790 | 378 | 860 | 402 |
| r6c4 | 990 | 378 | 1060 | 402 |
| r6c5 | 1191 | 378 | 1261 | 402 |
| r6c6 | 1392 | 378 | 1462 | 402 |
| r7c1 | 288 | 432 | 358 | 456 |
| r7c2 | 489 | 432 | 559 | 456 |
| r7c3 | 690 | 432 | 760 | 456 |
| r7c4 | 890 | 432 | 960 | 456 |
| r7c5 | 1091 | 432 | 1161 | 456 |
| r7c6 | 1291 | 432 | 1361 | 456 |
| r7c7 | 1492 | 432 | 1562 | 456 |

## Lower panel and shared toolbar

| Target | Bounds (x, y, width, height) | Hit (x, y) | Definition |
|---|---|---|---|
| Pyramid Left | `(759, 678, 139, 187)` | `(828, 771)` | `pyramid::LEFT_TARGET` |
| Pyramid Move | `(920, 678, 80, 80)` | `(960, 718)` | `pyramid::MOVE_TARGET` |
| Pyramid Right | `(1022, 678, 139, 187)` | `(1091, 771)` | `pyramid::RIGHT_TARGET` |
| Shared Solver | `(585, 959, 35, 36)` | `(602, 977)` | `parameters::SHARED_SOLVER_CONTROL` |
| Shared Undo All | `(1301, 959, 38, 38)` | `(1320, 978)` | `parameters::SHARED_UNDO_ALL_CONTROL` |
| Shared Undo | `(1661, 960, 37, 36)` | `(1680, 978)` | `parameters::SHARED_UNDO_CONTROL` |

Move and Recycle occupy the same control; geometry does not establish which action is currently legal. Left may show a recycle indicator when empty. Right may show an empty-card outline. The toolbar bounds deliberately cover the visible icons rather than claiming the full invisible button hit areas.

The Undo All confirmation dialog is not calibrated by this evidence set. Its control must be captured and verified separately before any future automated confirmation. Ordinary Undo has its own shared coordinate. Recording a coordinate does not send input.

## Halo evidence for future detector work

The code does not use these samples as active probes. Exploratory 2×2 sample locations are recorded here to avoid losing useful evidence:

- Tableau: `(card_x+40, card_y+192, 2, 2)` on the bottom halo, when the card is highlighted and exposed.
- Left: `(789, 871, 2, 2)`; Right: `(1052, 871, 2, 2)`.
- Move: `(950, 753, 2, 2)` on its different control border. The card formula does not apply to Move.

Gold-shaped samples are present for the following tableau slots in these numbered source images:

| Slot | Images with matching sample |
|---|---|
| r4c1 | 09 |
| r4c2 | 07, 08 |
| r4c4 | 01 |
| r5c2 | 10 |
| r5c4 | 04 |
| r5c5 | 02, 03 |
| r6c2 | 15 |
| r6c5 | 09 |
| r6c6 | 07, 08 |
| r7c1 | 19 |
| r7c2 | 01 |
| r7c4 | 19 |
| r7c5 | 18 |

All other tableau probe placements are geometric extrapolations without a highlighted sample at that slot. Sample colours include Left/Right `(236,208,106)` and `(237,207,109)`; Move varies across screenshots, including `(243,200,82)` and `(253,209,90)`. These values do not establish a production colour threshold. Rank artwork, occlusion, transient animation and negative scenes require separate detector validation.

## Beast acceptance checks

1. Launch version 1.0.1 against the intended QMP socket. Select Pyramid, then Capture Frame.
2. Enable target overlays. Verify 28 face rectangles match the rows; pink hit markers sit in exposed top strips.
3. Verify Left, Move/Recycle and Right align with the lower panel; Solver, Undo All and Undo align with the bottom toolbar.
4. Inspect Params for the target bounds/hit points and `guest-input-authorised = false`. Pyramid execution controls must remain disabled.
5. Switch to TriPeaks and capture afresh. Its original three profile envelopes and shared toolbar must still align. No Pyramid target may appear in that mode.
6. Capture PNG, save it, and attach the fresh evidence plus relevant log if an alignment is wrong. Saving original PNG bytes does not include preview overlays; use a host screenshot of the application preview when reporting overlay alignment.

## Source attachment index

Numbers refer only to the order of image URLs in Issue #1 as read for this cycle, not temporal or pre/post ordering. Original PNG bytes are identified by SHA-256. The images remain on the Issue; they are not bundled into the source patch.

| Number | Attachment | SHA-256 |
|---:|---|---|
| 01 | [PNG](https://github.com/user-attachments/assets/090264d5-72c0-4810-9e5e-1a0ad8676e39) | `1d716a39b03793d90480b9da764365f9034ed52447c276d9e32dada593deb5fb` |
| 02 | [PNG](https://github.com/user-attachments/assets/566784b4-f31b-46e0-89ac-35bade4caf3b) | `158debb5a3f75a8573e60574fb0ee2c0ebb0b468d82e87c1ddd0d4106abfe0db` |
| 03 | [PNG](https://github.com/user-attachments/assets/313104e3-e02b-486a-a09c-7e9a0627c288) | `bd2e8f9e9042315d96fe180837c943a1c05192af01702189e40dac5264d6641d` |
| 04 | [PNG](https://github.com/user-attachments/assets/d5e0391a-5249-4e24-8fc1-865a4d04edf2) | `027228b614c86d4ba37eddac9a307864545e6b93364f687c6d110796adf4a334` |
| 05 | [PNG](https://github.com/user-attachments/assets/3d8b0f10-2f25-476b-9fa1-037a15646db6) | `0b7ecd4541697080aac687ed45707eb21c50b840cb525885ceb16bcadbba835b` |
| 06 | [PNG](https://github.com/user-attachments/assets/4f1151fd-3ace-4571-852f-f85d956651c0) | `4b0ac00cfe50906e2c41d38397d9efb135b8c0408bec2e5345a42ff734cd4a79` |
| 07 | [PNG](https://github.com/user-attachments/assets/b9689321-bb93-4633-944f-cd8b378157c8) | `f8d306c5bb54dabe2f494b82472fc366c6c9a51cb98fc52dd87b6e21130d6e97` |
| 08 | [PNG](https://github.com/user-attachments/assets/cc42d55c-ef1c-4fe6-ada1-0e09289bea35) | `abb34ba0991b66d70221ae7476d60cf357d35a1001f7dcbe60d65273ec97cfbd` |
| 09 | [PNG](https://github.com/user-attachments/assets/9a883543-5fce-43eb-97ea-9e1812dc6589) | `709f61472fa77fd011b282a5db99cc444370993d5fda28e7b635a5720a0c387d` |
| 10 | [PNG](https://github.com/user-attachments/assets/39d9375c-fc24-4ae8-9054-9e82034c6488) | `54598168b9c9d4596128298d767bab438e74c02ce9a1f84ffc921cd202f0d00c` |
| 11 | [PNG](https://github.com/user-attachments/assets/cab341b8-f44b-41ba-acc9-5a04dcf26ac1) | `0128d77e9a2e1a15fe293de4bd1e5213c8a83982d7f021b16ebf04b9bf3baeab` |
| 12 | [PNG](https://github.com/user-attachments/assets/01606b42-949b-44d0-8905-32215189f95c) | `4f7be1260d8ee4e448ec21c5d138f17a2e06d7308f0e69eb4107da8ceff7a9cf` |
| 13 | [PNG](https://github.com/user-attachments/assets/5bed27e1-754c-4891-bd12-a6a6a35a331c) | `3de038a307e753448ff938c1b0b021a20509842cc77e04e14a04859a45367c3f` |
| 14 | [PNG](https://github.com/user-attachments/assets/3b794d76-e6d5-4c62-968c-af630755eee3) | `6fc20afff852550f1e677c9cb3ab9fb6612f76442800ca9bc5756f6bdc37cf44` |
| 15 | [PNG](https://github.com/user-attachments/assets/d550e14c-b051-4294-a11f-a2233b9976ee) | `9f196b7e34ae053bd1cd7dc82cea5e2d54c2c8f1d5d0723f0c339785d690f0cd` |
| 16 | [PNG](https://github.com/user-attachments/assets/be228391-1336-4150-9b35-863c4a8af682) | `aa3a89bcfd91433285d885499920079b513971c5c0d5d5bf3c71ec5d38ab9bfe` |
| 17 | [PNG](https://github.com/user-attachments/assets/5867f96c-ee78-4c86-90bb-d41bca7692d7) | `50d0f134d153f9e86e0ce9b7fdfa84d83616b3eaa53d28a131be2645b21d88a0` |
| 18 | [PNG](https://github.com/user-attachments/assets/ec2e887e-5589-4e23-bb99-64547d241090) | `84b4ba2e92b29ed9600cca4cf20186a234a902b06b1ec6a085992253afbd4d9a` |
| 19 | [PNG](https://github.com/user-attachments/assets/6f1c7dc8-cfcd-41b6-8afb-4f33c45e1a71) | `20b8e6e77eb1dba45116aa4584d949fc47737a77f0029db55b2879d536acc001` |
| 20 | [PNG](https://github.com/user-attachments/assets/01a5c756-0b6d-4a2a-8ca1-48f0c1e3431c) | `e8b646f8c02ce841629e7e3da5ed181c5f17e093a004841bdf0060249da51653` |
