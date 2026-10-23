# Native-v9 retained King-rank evidence

Retained: 19 October 2026. This fixture supports the host-verified application
2.5.107 King correction: all 977 tests and required host checks passed. Fresh
Beast acceptance remains pending. Native-v8 and every earlier source fixture remain byte-unchanged.

## Original refusal and classification

The original `/home/charlie/tmp/.pyramid-recon-710918-3` attempt retained one
native capture, zero attempted D/Undo All/confirmation inputs and no export.
K♠ at fixed one-based tableau location `3|2` has best old rank occupancy XOR
distance 28, above cap 24. Its suit distance is 2 against cap 8. The other 27
tableau cards and DL 4♣ pass. This is evidence for a native King rank rendering,
not a suit failure or a complete stock inventory.

Native-v9 appends the exact labelled King-rank crop from that frame to both
tableau and stock rank banks. All old variants, rank/suit caps 24/8, alignment
±1, masks, scene/modal signatures, geometry and paper/ink/colour checks remain
unchanged. The profile ID and bundled default selection identify the new version;
they do not grant input authority.

## Exact files and provenance

| File | Bytes | SHA-256 | Source / purpose |
| --- | ---: | --- | --- |
| [profile.json](profile.json) | 803,364 | `b6ffea89b108f222f1d1aee80a951e024588b465465200fc7daa612c20e21865` | Separately named `pyramid-native-1920-classic-blue-v9`; ID plus one appended K variant in each bank are the only changes. |
| [baseline.png](baseline.png) | 3,744,974 | `0b2f53ff00b32233bdbad78659f2c18304972338ae6d5af7a7a1640ece6b1310` | Exact original `.pyramid-recon-710918-3/baseline.png`, without transformation. |
| [INCOMPLETE.json](INCOMPLETE.json) | 964 | `bd2aa68a13f60e85d37d391a680c400085218818b9f9bbf3af4b5673213954f2` | Exact original diagnostic record: one capture, no input, publication not attempted, restored false. |
| [provenance.json](provenance.json) | Machine-readable provenance | Source paths and hashes above | Labels, crop bounds, unchanged base profile, limited visible inventory and original refusal. |

The exact row-major RGB crop has native bounds `[892,224,27,24]`, from fixed
`3|2` / zero-based logical slot 4. It has no resampling, mask derivation or colour
transformation. Raw RGB SHA-256 is
`12967fd5fbe85e756a890160803fdb8b6a7616bb81516e46a7a7ac674a79055f`.
Both banks increase King variants from one to two. The immutable parent
[profile](../native-v8/profile.json) remains SHA-256
`0b459aa78701da2f07f6cf911ce6f78f7190b1432b66706f4f0135ed21f48a3e`.
No derived or synthetic image is represented as the original failed frame;
the appended calibration sample remains traceable to its original pixels.
The nine focused production-reader, negative and provenance cases passed, zero
failed/ignored, 968 filtered, 8.42 s test time. They overlap the successful
977-test full suite, zero failed/ignored. The first compile failed in a test-only checkerboard integer type; its
source/log are retained and explicit `u32` coordinates fixed it without a
production change. The verification record preserves the exact failure.

## Evidence boundary

The single retained frame exposes tableau/pile observations only. It does not
prove ordered stock, 24 advances, Undo All, original-board restoration, an
independently legal route or a live success. No fixture mints a restored receipt,
permits retry/resume or authorises guest input. Preserve the original refusal.

See [profile details](../../../../docs/pyramid-recon-profile.md),
[verification](../../../../docs/Verification-2.5.107.md) and
[fresh live-test brief](../../../../docs/Live-Test-2.5.107.md). The broader approved
timing, completion, mode and search-statistics work remains outside native-v9.
