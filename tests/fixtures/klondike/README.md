# Klondike detector regression fixtures

These twelve derived PNGs retain original RGBA pixels only in the rectangles
listed in `manifest.json`; other pixels are opaque black. Their dimensions and
coordinates remain 1920 × 1080. This keeps the test suite below 20 MiB without
resizing, colour conversion or loss of pixels used by this detector's scene,
HALO and effect probes. They are test data, not replacements for the originals.
The manifest records original names and SHA-256 alongside each fixture hash.

K01-K11 correspond to the supplied evidence numbering. K12 is the original
`qmp-qemu-socket 260930 015843 Recycle-Stack.png` showing highlighted exhausted
stock. The five actual action pairs are K02/K03, K04/K05, K06/K07, K08/K09 and
K10/K11. Other adjacent timestamps are not assumed to be a single input trace.

The Rust tests load these files at runtime under `CARGO_MANIFEST_DIR`; they are
not linked into a release executable. Explicitly synthetic tests translate
real source-card pixels to each waste fan/foundation location and manipulate
mask/cursor/overlay pixels to exercise refusal paths. The synthetic recycle
after frame does not establish live recycle acceptance. Those action classes,
settings and input semantics are authorised by Charlie's 30 September 2026
instructions; new live effects still require observation on the Beast.

The profile remains bounded to the evidenced geometry, Draw 1, 100% guest
scaling and visible unclipped cards. Unknown, shifted, dimmed or sparse endgame
layouts stop; no completion or restart detector is included.
