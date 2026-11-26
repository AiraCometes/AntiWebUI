# Audit: licences

- Date: 2026-11-25
- Upstream commit: `6908e3119db4ecf7080f57de59870e083f2cbc98`
- Auditor: Claude Code, Claude Opus 5.5
- Plan item: [A2](phase-1-audit-plan.md#workstreams)

## Scope

The declared licences of every package in `Cargo.lock`, security advisories against it, package sources, the fonts and other assets in the repository, third-party code copied into upstream crates, and libraries loaded at run time (FFmpeg and the others found). The workspace licence itself is already recorded in the [Phase 0 inspection](phase-0-inspection.md#license-and-tracking-constraints) and is not repeated.

Left out: licences of system libraries that are linked or loaded but not shipped (listed only), patents beyond what upstream documents, and any legal reading. Nothing here is legal advice.

## Method

Toolchain: cargo 1.99.0, cargo-deny 0.22.0 (pinned in `mise.toml`). cargo-deny ran with a throwaway config outside the repository, so no `deny.toml` was added. Advisory database: `rustsec/advisory-db` at `10db6998b5d74ec438bcf0763125bd425a068a8f` (2026-11-24).

| Question | Command |
|---|---|
| Declared licence of each package | `cargo metadata --format-version 1 --locked --all-features`, grouped by the `license` field with a script |
| Licences per SPDX identifier | `cargo deny --locked --all-features list -f json` |
| Licences in the default facade build | `cargo tree --locked -p anti-webui -e normal --target <triple> --prefix none -f '{p} {l}'` for Linux, Windows, and macOS |
| Which crate pulls a package in | `cargo tree --locked --all-features -i <crate> -e normal --target all` |
| Advisories, licences, sources | `cargo deny --locked --all-features --config <scratch>/deny.toml check advisories licenses sources`, with `allow` set to the 13 licences in [Crate licences](#crate-licences), `multiple-versions = "allow"`, and `unknown-git = "warn"` |
| Assets in the repository | `git ls-files` without `.rs`, `.md`, `.toml`, and `.lock` files, grouped by folder; then the licence and notice files next to them |
| Embedded files | `rg -n 'include_bytes!\|include_str!' src` |
| Copied third-party code | `rg -n -i 'copyright\|SPDX\|ported from\|adapted from\|derived from' src` |
| Libraries loaded at run time | `rg -n 'Library::new\|dlopen' src`; the resolved features of `ash`, `khronos-egl`, and `wayland-sys` in `cargo metadata` |

## Findings

### Crate licences

- Status: present
- Evidence: `cargo metadata` over all 467 packages in `Cargo.lock`. Every package declares a `license` expression; none uses `license-file` only. cargo-deny lists 0 unlicensed crates.
- Behaviour: the expressions, by package count:

| Packages | Expression | Examples |
|---|---|---|
| 205 | `MIT OR Apache-2.0` | the 20 dual-licensed workspace crates, most of the ecosystem |
| 102 | `MIT` | `rquickjs`, `tree-sitter` and its 14 grammars, `wayland-sys`, `yeslogic-fontconfig-sys` |
| 64 | `Apache-2.0 OR MIT` | `async-*`, `zbus` family |
| 22 | `Unicode-3.0` | the `icu_*` crates, through `parley` and `fontique` |
| 11 | `Apache-2.0` | `anti-webui-symbols-outlined`, `codespan-reporting`, `gethostname`, `spirv`, `merman-unicode-linebreak` |
| 46 | other permissive `OR` choices (old `/` syntax included) | `bytemuck`, `objc2-*`, `zerocopy`, `rustix`, `adler2`, `moxcms` |
| 4 | `BSD-3-Clause` | `tiny-skia`, `tiny-skia-path`, `alloc-no-stdlib`, `alloc-stdlib` |
| 3 | `MPL-2.0` | `cssparser`, `cssparser-macros`, `dtoa-short` |
| 3 | `Zlib` | `foldhash`, `slotmap`, `zlib-rs` |
| 7 | single others or `AND` expressions | `arrayref` (BSD-2-Clause), `libloading` (ISC), `tiny-keccak` (CC0-1.0), `brotli` (BSD-3-Clause AND MIT), `encoding_rs` ((Apache-2.0 OR MIT) AND BSD-3-Clause), `unicode-ident` ((MIT OR Apache-2.0) AND Unicode-3.0) |

- No copyleft licence is required by any package. `r-efi` offers `LGPL-2.1-or-later` only as one choice beside MIT and Apache-2.0. The only weak copyleft that must be accepted is `MPL-2.0`, and only with the `markdown` feature (next section).
- A `licenses` check with an allow list of `MIT`, `Apache-2.0`, `Apache-2.0 WITH LLVM-exception`, `BSD-2-Clause`, `BSD-3-Clause`, `ISC`, `Zlib`, `0BSD`, `Unicode-3.0`, `Unlicense`, `BSL-1.0`, `CC0-1.0`, and `MPL-2.0` passes.
- Label: Confirmed

### Licences by feature

- Status: present
- Evidence: `cargo tree -p anti-webui` per target, and `cargo tree -i` for the crates below
- Behaviour:
  - The default facade build pulls 192 packages on Linux, 169 on Windows, and 159 on macOS. Beyond MIT and Apache-2.0 choices, they need `Unicode-3.0` (22 `icu_*` crates), `BSD-3-Clause` (`tiny-skia`, `tiny-skia-path`), `BSD-2-Clause` (`arrayref`), `ISC` (`libloading`), `Zlib` (`foldhash`, `slotmap`), and on Linux only `CC0-1.0` (`tiny-keccak`, through `xim` → `ahash` → `const-random` in the X11 backend). `Apache-2.0`-only crates: `codespan-reporting` (through `naga`) on every target, `spirv` on Linux and Windows, and `gethostname` (through `x11rb`) on Linux.
  - `MPL-2.0` (`cssparser`, `dtoa-short`), `BSL-1.0` choice (`ryu-js`), and the `url` crate come only with the `markdown` feature, through `merman`.
  - `brotli` (BSD-3-Clause AND MIT) comes only with the `pdf` feature, through `hayro`.
  - `rquickjs` (MIT, which compiles QuickJS from C) comes only with `pdf-js`. The tree-sitter grammars (MIT, compiled from C) come only with `code` or `markdown`.
- Label: Confirmed

### Advisories

- Status: present (one vulnerability)
- Evidence: `cargo deny … check advisories`, `RUSTSEC-2026-0592`
- Behaviour: `url` 2.5.8 has a stack overflow in `Url::origin()` on deeply nested `blob:` URLs; the fix is `url` 2.5.9. It reaches the workspace only through `merman-markup`, `merman-core`, and `merman-render` (feature `markdown`). `rg '\.origin\(\)'` over the `merman-*` 0.10.0-alpha.1 sources finds only `origin()` methods of `merman`'s own types, none on a `Url`. No unmaintained, unsound, or yanked notices were reported.
- Reading: the vulnerable function is likely not reached from AntiWebUI. The fix is a `Cargo.lock` update, which is upstream-owned. Label: Inferred, from the search above; a call through a re-export or a generic would not match it.
- Label: Confirmed (the finding)

### Package sources

- Status: present
- Evidence: `cargo deny … check sources`; [A1](audit-dependencies.md#external-dependency-counts)
- Behaviour: all sources are crates.io or the workspace, except the 10 `wgpu` family crates from the git fork `bea4dev/wgpu`. Those declare `MIT OR Apache-2.0`, as upstream wgpu does. The git source is a supply-chain and maintenance question for A3, not a licence one.
- Label: Confirmed

### Bundled fonts

- Status: present
- Evidence: the folders and files below; `include_bytes!` in the listed sources
- Behaviour:

| Crate | Files | Size | Licence and notice | Embedded by |
|---|---|---|---|---|
| `anti-webui-symbols-outlined` | 4,004 single-glyph `.ttf` files, `LICENSE`, `NOTICE` | 18 MB | Material Symbols Outlined, Apache-2.0, `NOTICE` names Google LLC, the source commit `10ecc14790612ca8fd164e3f4ffe9c16fc50d5b7`, font version 2.978, and how `tools/generate.py` split it. The crate declares `Apache-2.0` | `src/generated.rs`, one `include_bytes!` per icon |
| `anti-webui-math` | 19 `KaTeX_*.ttf`, `OFL.txt`, `NOTICE.md` | 536 KB | KaTeX 0.19.1 fonts, unchanged, SIL OFL 1.1. The crate declares `MIT OR Apache-2.0` | `src/fonts.rs` |
| `anti-webui-md3e` | `RobotoFlex.ttf`, `OFL.txt` | 1.8 MB | Roboto Flex, SIL OFL 1.1, copyright line of the Roboto Flex Project Authors. The crate declares `MIT OR Apache-2.0` | `src/lib.rs`, behind the default `bundled-fonts` feature |

- Roboto Flex is also embedded, by a relative path into `anti-webui-md3e/fonts`, in tests of `anti-webui-core`, `anti-webui-app`, and `anti-webui-wgpu` (`glyph_cache.rs`, under `#[cfg(test)]`), and in two `anti-webui` examples. None of those is in a library build.
- Quality notes: the `license` field of `anti-webui-math` and `anti-webui-md3e` does not mention OFL-1.1, so cargo-deny, `cargo about`, and crates.io see only `MIT OR Apache-2.0` for crates that carry OFL fonts. An SPDX expression such as `(MIT OR Apache-2.0) AND OFL-1.1` would show it. Label: Confirmed (the fields); Proposed (the expression, for the upstream-file policy in Phase 4)
- Label: Confirmed

### Other assets

- Status: present
- Evidence: `git ls-files`
- Behaviour:
  - `src/anti-webui-wayland-shell/protocols/hyprland-toplevel-export-v1.xml`: BSD-3-Clause, "Copyright © 2022 Vaxry", kept in the file's `<copyright>` element. Code generated from it is in the binary, so the notice belongs in a distributed app's notices. Label: Confirmed (licence); Inferred (the obligation)
  - Shaders: 4 `.wgsl` in `anti-webui-wgpu/src`, 9 in `anti-webui-md3e/shaders`. No foreign copyright line; taken as upstream's own. Label: Inferred
  - Test data: 13 SVG files (`anti-webui-app/tests/data/svg`), 5 media files (`anti-webui-video/tests/data`, made with the `ffmpeg -f lavfi` commands in the test sources' doc comments), `encrypted.pdf`, and `flex_wrap.txt`. Not shipped in libraries. Label: Confirmed (made by commands, for the media files); Unverified (the origin of the SVG and PDF files)
  - `image/` holds 4 README screenshots, 4.1 MB. Label: Confirmed
  - `anti-webui-pdf/src/script/api.js` is embedded with `include_str!` for the `pdf-js` sandbox. No foreign copyright line. Label: Confirmed (no line); Inferred (upstream's own)

### Copied third-party code

- Status: present (one case found)
- Evidence: `src/anti-webui-md3e/src/dynamic/mod.rs:1`–`16`
- Behaviour: the dynamic-colour module is a Rust port of Google's `material-color-utilities` (TypeScript, commit `194144d`), Apache-2.0. The module doc keeps the copyright line and the licence notice, and says what changed, as Apache-2.0 section 4 asks. The crate itself declares `MIT OR Apache-2.0`, so this part is in practice Apache-2.0 only. Label: Confirmed (notice); Inferred (the effect on the crate's licence)
- The search found no other port notice. Ports without a notice cannot be found this way. Label: Confirmed (search result); Unverified (absence)

### Libraries loaded at run time

- Status: present
- Evidence: the sources below; `docs/design/24-video.md` ("ライセンス", "FFmpeg の読み込み")
- Behaviour:

| Library | Loaded by | How | Licence (of the library, not shipped) |
|---|---|---|---|
| FFmpeg `avcodec`, `avformat`, `avutil`, `swscale`, `swresample` | `anti-webui-video/src/ffi.rs` | `libloading`, by file name per OS and major version (`file_name`), from a given folder or the system search path | LGPL-2.1-or-later, or GPL when the build enables GPL parts |
| `libpipewire-0.3.so.0` | `anti-webui-wayland-shell/src/audio/ffi.rs:277` (feature `audio`) | `libloading` | MIT |
| `libgbm.so.1` | `anti-webui-wgpu/src/gbm.rs:74` | `libloading` | MIT |
| Vulkan loader, `libEGL`, `libwayland-client` | `ash` (`loaded`), `khronos-egl` (`dynamic`), `wayland-sys` (`dlopen`), through wgpu and the Wayland backend | `libloading` inside those crates | Apache-2.0, MIT-style, MIT |
| `libfontconfig` | `yeslogic-fontconfig-sys`, through `fontique` | linked at build time (no `dlopen` feature resolved) | MIT-style |

- FFmpeg: `ffi.rs` declares the functions, field offsets, and constants by hand and says no header was copied (`ffi.rs:1`–`4`); `tools/offsets.c` is how the offsets were found. `VideoLibraries::LgplOnly` (`anti-webui-core/src/video.rs:43`) refuses a library whose `avcodec_license()` does not start with `LGPL` (`ffi.rs`, `license_allowed`). The default is `VideoLibraries::Any`, so a GPL build of the system FFmpeg is used unless an app opts out. `tools/build-ffmpeg-lgpl.sh` builds an LGPL FFmpeg with a fixed decoder list that includes H.264 and HEVC; the script leaves the patent question to whoever ships it. Upstream records that no lawyer has checked this design. Label: Confirmed
- Label: Confirmed

### Third-party notices

- Status: absent
- Evidence: no `THIRD-PARTY`, `NOTICE`, or `about.toml` file at the root; no `cargo about` or similar tool in the workspace
- Behaviour: an app built on AntiWebUI ships BSD-2-Clause, BSD-3-Clause, ISC, Zlib, Unicode-3.0, and Apache-2.0 code, the Material Symbols `NOTICE`, and OFL fonts. Each asks for its notice or licence text in a binary distribution. Nothing in upstream collects them today, so every app author has to. Label: Confirmed (absence); Inferred (the obligation, from the licences named)

## Baseline violations

The fork has no licence policy file yet. Against the plan's rule (cargo-deny, warnings only), the baseline is:

- 1 advisory: `RUSTSEC-2026-0592` (`url` 2.5.8), feature `markdown` only.
- 10 git-source packages (the wgpu fork).
- 0 unlicensed packages; 0 packages outside the 13-licence allow list.
- 2 crates (`anti-webui-math`, `anti-webui-md3e`) whose `license` field does not name the OFL fonts they embed.

Fork changes must not add a licence outside that allow list, or a new advisory, without a decision record.

## Risks and technical debt

| Item | Impact | Likelihood | Evidence |
|---|---|---|---|
| A closed-source app loads a GPL FFmpeg because the default is `VideoLibraries::Any` | High | Medium, for apps that use `video` on Linux distributions with GPL builds | [Libraries loaded at run time](#libraries-loaded-at-run-time) |
| App authors miss the notices for BSD, ISC, Unicode, Apache, and OFL parts, since nothing collects them | Medium | High | [Third-party notices](#third-party-notices) |
| Licence metadata of `anti-webui-math` and `anti-webui-md3e` hides OFL-1.1 from tools | Low | High | [Bundled fonts](#bundled-fonts) |
| The 18 MB, 4,003-file icon crate makes the repository and any crates.io package large | Medium | High, if published | [Bundled fonts](#bundled-fonts) |
| The `url` advisory stays until `merman` or `Cargo.lock` moves, because both are outside fork ownership | Low | High | [Advisories](#advisories) |
| Hand-written FFmpeg offsets break silently on a new FFmpeg major version | Medium | Medium | `ffi.rs` (`LAYOUTS`, `Majors`); A4b or A6 can check how a new version is detected |

## Open questions

- Should the fork's default become `VideoLibraries::LgplOnly`? A Phase 2 requirement; it changes behaviour for every app.
- Is a `deny.toml` with this allow list wanted now, as a report-only task like `quality:unused-deps`, or only in Phase 5 CI? A maintainer decision; adding it changes no upstream file.
- Where did the SVG test files and `encrypted.pdf` come from? Answered by the upstream author or the commits that added them (`git log --diff-filter=A -- <file>`).

## Inputs for gap analysis

- The licence allow list and the notice gap, for the Phase 2 non-functional requirements on distribution.
- The FFmpeg default and the `LgplOnly` switch, for the video requirement and its non-goals.
- The OFL and Apache-only parts, for the Phase 3 crate split (which crates may stay dual-licensed).
