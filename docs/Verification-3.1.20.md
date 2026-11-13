# Verification — 3.1.20

All 50 fresh host gates passed, including 1,911 application, 381 library and 80 console tests. Complete suites have zero failed, ignored, measured or filtered tests.

Charlie reported successful 3.1.19 guest playback at 100 ms. Fresh Windows acceptance of this styled 3.1.20 build remains pending.

The reference demo, starting snapshot, format scope, command receipts, complete logs and retained initial failures are in `../Evidence/FreeCell/Validation/freecell-style-320-20261112/`.

Final compiler-input seal `386e6bc44568918ab6164428682ad144af1eb66ac4498d6e2c35b0582383fa13`. Toolchain/configuration seal `577f3299ff52c92186ab0281bc2b63d9021d281d99a19939e26c819915767a52`.

| Gate | Seconds | Log SHA-256 |
| --- | ---: | --- |
| check | 13.220 | `5479c5f8358e5041ee2b0b2807eeef09fbaef5c3f49b689eba70ec30783b18a9` |
| clippy | 12.869 | `f75c8b443dae478c69682aefca43195339040c5ae4d9de422def9606d8d4af50` |
| console-check | 1.218 | `3da790554c8214031b757bdadbc180e088e3a049a07fe0e8ffda67ad0876d0fd` |
| console-clippy | 1.113 | `918487dd6fde83046c2e9af7e16953988777d1224df01926e70dfb5708fc06b0` |
| console-release | 1.851 | `91bdbf54b1c7ee2ef0497d57d040e6b5b90a447fb4748f89931582f5612f4abe` |
| console-rustdoc | 1.723 | `90b8b89b26983a33df44091c0f58b58a367496d8ce3dc75bed1753669d62362c` |
| console-tests | 4.997 | `3196efe4a67c6f16474d6b6d27c1d315bd7fa15fec2cfc905a80ebc4039b0a7b` |
| format | 0.904 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| release | 60.971 | `f863f1db76378a9f138ab8780b04c5c140cb4ce4ece9b237e39e94f5631abff0` |
| rustdoc | 3.054 | `c70b9a49363678aacb207532af7367780eddddf4716617f8f31a178d90ce0163` |
| tests | 99.085 | `62e341e949f93b91bafe8b62663c09eef53123b74fbc47c2f9d7dcf7c37d83df` |
| xsar-check | 2.465 | `c3c711b094b2d7e334a99cb7e443a0de10c2c6ac448f459b381c02af61b6fb4e` |
| xsar-clippy | 2.352 | `81928d766fe4cd98d567235618d957f9f478497ff5a6f0f15029782ebd73dae1` |
| xsar-default-check | 0.845 | `ed2eb0607b1eaed0108bf6cd2f2c86d902a1baaba3728641550c52337becb03a` |
| xsar-default-tests | 1.031 | `c2ae8e8ae034c9e922ab01b2eebc54d7cf71d5818863a7e528ea6641505258e7` |
| xsar-feature-cards-check | 0.846 | `c4dece7b6267ac54266c82b0f016bcaca8b50309d913846f1eaefcb9856826bf` |
| xsar-feature-cards-tests | 1.201 | `cd8eea31b43d30eb7c604bad9467f2c234c89adb2f07ece6561ab734a2ac20a7` |
| xsar-feature-execution-check | 0.960 | `9d9c801c7cc1335d7ee4333dad6de5e4f4345e7b6f5d2aa5232f5903dade0926` |
| xsar-feature-execution-tests | 1.171 | `f93be16d814733e04b729886993fbd9e807ad9b1d66f37dbd21a684234d590e9` |
| xsar-feature-freecell-check | 0.882 | `ef7174c0030d17b1fa69c769a037e89e23b11d4652c3e2d828c787a708e42b20` |
| xsar-feature-freecell-tests | 1.171 | `3f6b8bb53efba0b7b3110c1034acb570407128dbd9fb4a1cb6c7eae0330606cb` |
| xsar-feature-freecell_json-check | 0.996 | `9a9f1e5f8b7c01bac02aec866200d5e0448455f1869996112833272f6eb0eedb` |
| xsar-feature-freecell_json-tests | 1.551 | `397802a4d808e1e57502ec223302530a836f5e41779cc0954c05f0fcfcaf57ef` |
| xsar-feature-freecell_rules-check | 0.954 | `aae850f0ebdd411c6821d0bd3ce0cd8c1eb9b177c4501237df4aca4ffe1f2f36` |
| xsar-feature-freecell_rules-tests | 12.362 | `aa6ee2c8fa041c910c427d5c9cb0c1bdc7b4edd9d6568352680b71302367f347` |
| xsar-feature-freecell_search-check | 1.184 | `c7b05668c21858d36ffccf449be0b424d5c9a255493fbcc206909d554beab3e7` |
| xsar-feature-freecell_search-tests | 12.640 | `6ef382231046f3d8498e99f2c5aa8d4ac28a6a8c918d054869d49e392b663c45` |
| xsar-feature-image_matching-check | 0.920 | `c2a0bac6dd2b5d0f99dfede35e9bfc9d3acbcd9d4fc1d23c5287fbc957822689` |
| xsar-feature-image_matching-tests | 1.372 | `4af72e82db3aacf83f6fa61464a636c14ce21dcdb33f1b6cca380482dce12e00` |
| xsar-feature-pixel_format-check | 0.844 | `3423c677a7dbab67ba6c6220f4ee841718c215a56ba39ab2aa5860e3b21a3fb2` |
| xsar-feature-pixel_format-tests | 1.166 | `486d12121fdedb7ffda6048da8536f586fea9a6434a5bd90c371c1501abe0180` |
| xsar-feature-png-check | 0.945 | `aae850f0ebdd411c6821d0bd3ce0cd8c1eb9b177c4501237df4aca4ffe1f2f36` |
| xsar-feature-png-tests | 1.595 | `ab42e9224c61beed504fd0fa1d523d584cdb4a9a65d28489185fb5f935cc4e7e` |
| xsar-feature-pyramid-check | 0.918 | `9d9c801c7cc1335d7ee4333dad6de5e4f4345e7b6f5d2aa5232f5903dade0926` |
| xsar-feature-pyramid-tests | 1.299 | `24c5050d4385b6ce9266898c03666eda02e7eb470b6f3d251d60e8afa0a92883` |
| xsar-feature-pyramid_json-check | 1.051 | `42611ff9c7c67c24cdec1805982d20799f51a5417f16098b711d6f2eadff460f` |
| xsar-feature-pyramid_json-tests | 2.718 | `55f396865d9cacf5970e3c99c3ddfcf251ebc5cbb6b734958c95964cf53636e3` |
| xsar-feature-pyramid_rules-check | 0.999 | `3ccc06fc7a3ed22d1b91b5f3d67db4b34a78f6e2f3b575abea74694e40922ef6` |
| xsar-feature-pyramid_rules-tests | 1.529 | `e165b3447baaca21f916fabb764fdd022d7bca715693e707960b13bd28908b3f` |
| xsar-feature-pyramid_search-check | 1.140 | `2bed150e1804412ccfbb7c8894d6896a9c638026e9a90f5990a5a1b0b2206722` |
| xsar-feature-pyramid_search-tests | 1.922 | `b414724cfb89f36d6505b71ab9eb44806f725edd988bc1868b119ad45fb65ee2` |
| xsar-feature-qmp-check | 1.360 | `b798b36aa220b2dfa37130d66b58d614d166968525df9302bd2b9eb8c2ba1d28` |
| xsar-feature-qmp-tests | 26.449 | `6205ccda5d8a78dd50ec1184ea6a99127d23a169a57efe21403fdb835df06dcd` |
| xsar-feature-search-check | 0.996 | `83e1e337146f3d19778c511800badcf3551ac08bb241a3552040f49ba484ce67` |
| xsar-feature-search-tests | 1.643 | `294cb4e1dfc0b9dc71fe23e15ed14b0c5a81148ac82d5df57340616aeb479809` |
| xsar-feature-tcp_capture-check | 0.923 | `c2a0bac6dd2b5d0f99dfede35e9bfc9d3acbcd9d4fc1d23c5287fbc957822689` |
| xsar-feature-tcp_capture-tests | 2.128 | `65daf5fbb378964de08cfdcc1f04c7ab74c73fcb071814e7250e02c5b64ab9b3` |
| xsar-release | 2.555 | `a76df053e443e26130d1bf1cea43a519f885ddaa285936496d263f4054a076fe` |
| xsar-rustdoc | 2.087 | `14f079f0d2bb37d3ee1d312f9564665f068af4ecf3fdc5c27f05eb39b9b57fd7` |
| xsar-tests | 43.722 | `5f9c0938bb814c58bb42343fb888f133196f0b7ad6193a511df44d5f0007b225` |

Toolchain and configuration recorded with every command follow.

```json
{
  "cargo": "cargo 1.102.0-nightly (c07ac2c4d 2026-11-06)\nrelease: 1.102.0-nightly\ncommit-hash: c07ac2c4d12237effc4b94f4543769b421790045\ncommit-date: 2026-11-06\nhost: x86_64-unknown-linux-gnu\nlibgit2: 1.9.6 (sys:0.21.0 vendored)\nlibcurl: 8.21.0-DEV (sys:0.4.90+curl-8.21.0 vendored ssl:OpenSSL/3.6.3)\nssl: OpenSSL 3.6.3 9 Jun 2026\nos: Arch Linux [64-bit]\n",
  "configuration": {
    "cargo_configuration": {
      "/home/charlie/.cargo/config.toml": "491df52ce82b6d88da07372cc8ce21cbbf9cd9a1cc92df900d3e8c11acc7b331",
      "/home/charlie/repo/RUST/qmp-qemu-socket/.cargo/config.toml": "0ca984153b0f53d0322af5f1e8cf24f1446870055afcdfb8cfd0dc1b2a8baf54",
      "/home/charlie/repo/RUST/xsar/.cargo/config.toml": "1463ec95c1bbf458d30bb3e9caa43485d296cf64ce77b977ae38e00d15447f8d"
    },
    "inherited_environment": {
      "CARGO_BUILD_TARGET": null,
      "CARGO_ENCODED_RUSTDOCFLAGS": null,
      "CARGO_ENCODED_RUSTFLAGS": null,
      "CARGO_HOME": null,
      "CARGO_TARGET_DIR": null,
      "RUSTC_WORKSPACE_WRAPPER": null,
      "RUSTC_WRAPPER": null,
      "RUSTDOCFLAGS": null,
      "RUSTFLAGS": null,
      "RUSTUP_HOME": null,
      "RUSTUP_TOOLCHAIN": null,
      "RUST_TEST_THREADS": null
    }
  },
  "rustc": "rustc 1.102.0-nightly (ac8310ba6 2026-11-09)\nbinary: rustc\ncommit-hash: ac8310ba643a3796cbe77d2e06944d7c8513999f\ncommit-date: 2026-11-09\nhost: x86_64-unknown-linux-gnu\nrelease: 1.102.0-nightly\nLLVM version: 23.1.5\n",
  "rustfmt": "rustfmt 1.12.0-nightly (ac8310ba64 2026-11-09)\n"
}
```

Fresh Cargo artifacts follow. Each is mutable build output.

- application — `/home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/qmp-qemu-socket`, SHA-256 `6009845aa6b54996bd78481e808db57646e11491f2cfd330fc79cd6005eb9e2a`, 16,842,960 bytes, mode 0755.

- console — `/home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/basic-route-planning`, SHA-256 `6d070465474d3ba81c8c116c6e0ad1b15544341907b671c9875f67314d27d6c4`, 784,016 bytes, mode 0755.

- library — `/home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/build/xsar/ea6c48d42fa14bdf/out/libxsar-ea6c48d42fa14bdf.rlib`, SHA-256 `961684f7462c3f607fd49a6e8cea3204c9927779ec3c5ded4116047e1eee4da7`, 1,319,356 bytes, mode 0644.

- library_application_consumer — `/home/charlie/repo/RUST/qmp-qemu-socket/target/x86_64-unknown-linux-gnu/release/build/xsar/c11b66caf529aec5/out/libxsar-c11b66caf529aec5.rlib`, SHA-256 `98979ad88e3a7593f823d6f7733323fbdfae136234d25a62e40e82a09f0de22d`, 1,519,120 bytes, mode 0644.

The original failed focused attempts remain retained with their first diagnostics.

- `style-focused-retry.json`, exit 101, retained log `style-focused-retry.log`, SHA-256 `aadb23d042c9280134b6b93e8ac42b7c8e7ab72835513b9332e219a9bffbd692`.

- `style-focused.json`, exit 101, retained log `style-focused.log`, SHA-256 `121ff75bcc324bfaf5198069ccc584e7547c85a3708ee7d7c4b10c4f883dedaf`.

Focused checks passed with stable compiler inputs and toolchain: three shared-style tests, eight calculation/playback/control/transport tests and three activity-header layout tests. The control tests exercise production widgets at 320, 640 and 1920 points, including active Stop/Pause/Cancel adjacency and native enabled-state click routing.

The style tests exported actual egui meshes and font textures. A retained software renderer produced twelve 840×240 and 1280×240 frames at one pixel per point, covering idle, hover, settled hover, pressed, focus and disabled states. The images show readable labels, distinct feedback and fixed action bounds. The standalone sample has a closed ComboBox approximately eight pixels below the button baseline. These images cover the shared widget sample, with full application, open-popup, GPU/compositor and Windows appearance reserved for manual acceptance. The still frames sample at 200 ms and do not measure the 120 ms animation duration.

See the [render receipt](../Evidence/FreeCell/Validation/freecell-style-320-20261112/ui-render/render-receipt.json), [visual inspection](../Evidence/FreeCell/Validation/freecell-style-320-20261112/ui-render/visual-inspection.json) and [contact sheet](../Evidence/FreeCell/Validation/freecell-style-320-20261112/ui-render/style-contact-sheet.png).

Host UI tests and local transport peers qualify geometry, controls and protocol behaviour. The running Windows guest supplies fresh gameplay acceptance.
