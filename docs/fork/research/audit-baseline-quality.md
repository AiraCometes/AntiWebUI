# Audit: baseline quality

- Date: 2026-11-26
- Upstream commit: `6908e3119db4ecf7080f57de59870e083f2cbc98`
- Auditor: Claude Opus 5.5
- Plan item: [A7](phase-1-audit-plan.md#workstreams)

## Scope

Every Rust file under `src/`: rustfmt, Clippy with its default lint set, the future-incompatibility warnings rustc reports during the Clippy build, `unsafe` code and its `SAFETY:` comments, `#[allow]` attributes, function complexity and length, and copy-paste duplication.

`src/`, `Cargo.toml`, and `Cargo.lock` on the audited branch are identical to the upstream commit (`git diff --stat 6908e31 HEAD -- src Cargo.toml Cargo.lock` is empty), so every number here is upstream's.

Left out: Clippy's `pedantic`, `nursery`, and `restriction` groups, except the two `unsafe` lints named below; Clippy on macOS, whose target is not installed here; tests and coverage (A6); dependency rules (A1, A2).

## Method

Toolchain 1.99.0 from `rust-toolchain.toml` (rustfmt 1.10.0, Clippy 0.1.99), on WSL2 (Linux 6.18) inside the agent sandbox. The workspace has no `rustfmt.toml`, `clippy.toml`, or `[workspace.lints]`, so every tool ran with its defaults.

```sh
# Formatting
cargo fmt --all --check
# Clippy, default lints (1 min 33 s with the A6 build cached; 1.9 GB peak RSS)
cargo clippy --workspace --all-targets --locked --message-format=json
# Unsafe lints on Linux, then on Windows for the crates that have Windows code
cargo clippy --workspace --all-targets --locked --message-format=json \
  -- -W clippy::undocumented_unsafe_blocks -W clippy::missing_safety_doc
cargo clippy -p anti-webui-windows -p anti-webui-video -p anti-webui-wgpu --all-targets --locked \
  --target x86_64-pc-windows-msvc --message-format=json -- -W clippy::undocumented_unsafe_blocks
# Complexity (lizard, not pinned in mise.toml; run once through uvx)
uvx --from lizard@1.26.1 lizard -l rust --csv -x './target/*' src
# Duplication (jscpd, not pinned in mise.toml; run once through mise x)
mise x npm:jscpd@5.4.3 -- jscpd --format rust --min-tokens 100 --ignore '**/target/**' \
  --reporters json --output <dir> --silent src
```

Clippy diagnostics were de-duplicated by lint, file, and line, because a file built for both the library and its tests is reported twice.

The `unsafe` census is a script over the source text, because Clippy cannot see the macOS crate here. It finds `unsafe {`, `unsafe fn`, `unsafe impl`, and `unsafe extern` outside line comments. A block or impl counts as documented when `SAFETY` appears in the six lines up to and including it; an `unsafe fn` counts as documented when `# Safety` appears in the 25 lines above it or `SAFETY` in the six. Clippy's `undocumented_unsafe_blocks` is stricter: it wants a comment for each item, so one comment above two adjacent `unsafe impl`s counts once. Both results are given.

Lizard was chosen over rust-code-analysis because rust-code-analysis-cli's last release is 0.0.25 (2023) and lizard installs without a build. Lizard's Rust parser counts a closure as part of the function that contains it, so builder-style view functions show as one long function.

jscpd's default minimum is 50 tokens; 100 was used to leave out short builder chains that repeat by design.

## Findings

### Formatting

- Status: partial
- Evidence: `cargo fmt --all --check` exits 1 with 30 hunks in 20 files, 145 lines added and 100 removed. The most hunks are in `src/anti-webui-core/tests/stacking.rs` (6) and `src/anti-webui-core/tests/offscreen.rs` (3). Six of the files are in `anti-webui-wayland-shell`, three in `anti-webui-wgpu`, and two in `anti-webui-app`.
- Behaviour: the other 426 of the 446 `.rs` files under `src/` are already formatted. The hunks are ordinary rustfmt choices, such as joining a short `if … { "Modified" } else { "" }` onto one line (`src/anti-webui/examples/pdf.rs:208`) or splitting a long `.child(…)` call (`pdf.rs:145`).
- Quality notes: upstream evidently formats with rustfmt, but not before every commit. Whether the hunks come from a rustfmt version change or from edits made without formatting is not known.
- Label: Confirmed (counts); Unverified (cause; running rustfmt from upstream's toolchain at each file's last commit would tell).

### Clippy and rustc warnings

- Status: partial
- Evidence: 1,159 distinct warnings, no errors. 385 are in library code, 342 in tests, and 432 in examples.

| Lint | Count | Where |
|---|---:|---|
| `float_literal_f32_fallback` (rustc, `future_incompatible`) | 1,109 | Every crate that builds views |
| `clippy::chunks_exact_to_as_chunks` | 47 | Pixel loops, for example `src/anti-webui-core/src/images/decode.rs:199`, `src/anti-webui-wgpu/src/glyph_cache.rs:294`, `src/anti-webui-wayland/src/state.rs:709` |
| `clippy::clone_on_copy` | 1 | `src/anti-webui-md3e/src/components/progress.rs:109` |
| `clippy::type_complexity` | 1 | `src/anti-webui-md3e/src/components/text_field.rs:107` |
| `clippy::double_ended_iterator_last` | 1 | `src/anti-webui-core/tests/rich_text.rs:66` |

| Crate | Warnings |
|---|---:|
| `anti-webui-md3e` | 285 |
| `anti-webui` (examples) | 259 |
| `anti-webui-app` | 174 |
| `anti-webui-widgets` | 171 |
| `anti-webui-core` | 163 |
| `anti-webui-markdown` | 35 |
| `anti-webui-pdf` | 28 |
| `anti-webui-code` | 20 |
| `anti-webui-math` | 10 |
| `anti-webui-wgpu` | 5 |
| `anti-webui-wayland`, `anti-webui-wayland-shell` | 4 each |
| `anti-webui-video` | 1 |

- Behaviour: apart from the f32 fallback, upstream's code is close to Clippy-clean: 50 Clippy warnings in about 200,000 lines, 47 of them `chunks_exact_to_as_chunks`, which suggests the newer `as_chunks` API. The Windows-target run of `anti-webui-windows`, `anti-webui-video`, and `anti-webui-wgpu` adds no other default-lint warnings.
- Label: Confirmed

### The f32 literal fallback

- Status: present (a defect that will become a build failure)
- Evidence: the `setter!` macro (`src/anti-webui-style/src/styled.rs:3`–`15`) generates `fn $method(mut self, v: impl Into<$ty>)`. For the setters whose type is `f32`, such as `p` (`styled.rs:124`), `rounded` (`:159`), `opacity` (`:184`), and `text_size` (`:197`), a literal like `.p(4.0)` has no type of its own: `f64: Into<f32>` does not hold, so rustc falls back to `f32`. rustc 1.99 warns: "this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release", with a pointer to rust-lang/rust#154024 and the fix `4.0_f32`.
- Behaviour: the 1,109 warnings are all calls with a float literal. The most common methods are `rounded` (200), `gap` (171), `p` (125), `text_size` (103), `opacity` (88), `px` (75), and `border` (70). Every example, test, and widget that styles a view is affected, and so is every downstream user of the builder API.
- Quality notes: the fix belongs in the macro, not at the 1,109 call sites. Making the `f32` setters take `f32` directly types the literal; it breaks only callers that pass another type with `Into<f32>`, such as `u8` or `i16`. That is an API decision for upstream or for the fork's style API.
- Label: Confirmed (cause and counts); Inferred (that a macro change fixes every site, from the macro being the only generic parameter on these paths); Unverified (the Rust release that makes it an error; rust-lang/rust#154024 would say)

### `unsafe` code

- Status: present
- Evidence: source census, with Clippy's results after it.

| Crate | `unsafe {}` | Without `SAFETY` | `unsafe fn` | Without docs | `unsafe impl` | Without `SAFETY` | `unsafe extern` |
|---|---:|---:|---:|---:|---:|---:|---:|
| `anti-webui-windows` | 116 | 1 | 1 | 0 | 2 | 0 | 3 |
| `anti-webui-wgpu` | 110 | 0 | 5 | 1 | 2 | 0 | 8 |
| `anti-webui-video` | 108 | 0 | 6 | 0 | 12 | 0 | 19 |
| `anti-webui-wayland-shell` | 55 | 0 | 4 | 0 | 3 | 0 | 45 |
| `anti-webui-macos` | 46 | 38 | 0 | 0 | 13 | 8 | 1 |
| `anti-webui-app` | 7 | 0 | 0 | 0 | 0 | 0 | 1 |
| `anti-webui-wayland` | 6 | 0 | 0 | 0 | 0 | 0 | 0 |
| `anti-webui-core` | 5 | 0 | 1 | 0 | 2 | 0 | 1 |
| `anti-webui-x11` | 1 | 0 | 0 | 0 | 0 | 0 | 0 |
| `anti-webui-pdf` | 1 | 0 | 0 | 0 | 0 | 0 | 0 |
| Total | 455 | 39 | 17 | 1 | 34 | 8 | 78 |

- Clippy `undocumented_unsafe_blocks`, which also checks `unsafe impl`: 1 on Linux (`src/anti-webui-wgpu/src/lib.rs:94`) and 6 on the Windows target (`src/anti-webui-wgpu/src/lib.rs:94`, `src/anti-webui-wgpu/src/adapters.rs:20`, `:49`, `:50`, `src/anti-webui-windows/src/vblank.rs:39`, `src/anti-webui-windows/src/testing.rs:100`). All but `testing.rs:100` are a second or third `unsafe` right under one shared `SAFETY:` comment, such as `unsafe impl Send` and `unsafe impl Sync` together. `missing_safety_doc` finds nothing, because the one `unsafe fn` without a `# Safety` section, `open_for_dmabuf` (`src/anti-webui-wgpu/src/dmabuf.rs:175`), is private.
- Behaviour: no crate uses `#![forbid(unsafe_code)]` or `#![deny(unsafe_code)]`. The platform, GPU, video, and app crates hold 449 of the 455 `unsafe` blocks; `anti-webui-core`, `-style`, `-widgets`, `-md3e`, `-markdown`, `-code`, `-math`, and `-macros` have 5 or none. Outside macOS the `SAFETY:` practice holds almost everywhere. In `anti-webui-macos` 38 of 46 blocks and 8 of 13 impls have no comment; they are mostly `msg_send!` calls to the superclass, AppKit constants such as `kCAGravityTopLeft`, and the protocol impls that `objc2`'s `define_class!` requires (for example `src/anti-webui-macos/src/view.rs:274`–`304`, `:481`–`536`).
- Quality notes: A4a lists each `unsafe` site in `anti-webui-core` ([core audit](audit-capabilities-core.md)).
- Label: Confirmed (counts, Clippy results); Inferred (the macOS sites are routine `objc2` calls, from reading the listed lines without a macOS build)

### Lint suppressions and markers

- Status: present
- Evidence: 55 `#[allow(…)]` and `#![allow(…)]` attributes in `src/`: `dead_code` 23, `clippy::too_many_arguments` 20, `unused_mut` 4, `unused_imports` 2, `clippy::large_enum_variant` 2, and one each of `unused_variables`, `clippy::type_complexity`, `clippy::single_range_in_vec_init`, `clippy::only_used_in_recursion` (`rg -o 'allow\((clippy::[a-z_]+|dead_code|unused[a-z_]*)' src`). No `todo!()`, `unimplemented!()`, or `TODO`/`FIXME`/`HACK`/`XXX` comments.
- Behaviour: upstream records open work in `docs/design` (A8), not in code markers.
- Label: Confirmed

### Complexity

- Status: partial
- Evidence: lizard found 8,927 functions in 201,646 lines. In library code (not `tests/`, `benches/`, or `examples/`): 329 functions with cyclomatic complexity (CCN) above 10, 162 above 15, 48 above 25, and 11 above 50; 258 functions longer than 60 lines of code (NLOC), 83 longer than 100, 19 longer than 200. 29 functions take more than 7 parameters.

| Crate | Functions (library) | CCN > 15 | NLOC > 100 |
|---|---:|---:|---:|
| `anti-webui-core` | 1,641 | 57 | 18 |
| `anti-webui-wayland-shell` | 698 | 12 | 2 |
| `anti-webui-pdf` | 680 | 18 | 6 |
| `anti-webui-md3e` | 615 | 17 | 19 |
| `anti-webui-widgets` | 437 | 3 | 3 |
| `anti-webui-wgpu` | 340 | 6 | 15 |
| `anti-webui-app` | 337 | 4 | 4 |
| `anti-webui-macos` | 329 | 2 | 1 |
| `anti-webui-x11` | 298 | 7 | 2 |
| `anti-webui-windows` | 257 | 4 | 1 |
| `anti-webui-wayland` | 256 | 7 | 2 |
| `anti-webui-video` | 162 | 4 | 2 |
| `anti-webui-markdown` | 153 | 10 | 3 |
| `anti-webui-style` | 150 | 0 | 0 |
| `anti-webui-code` | 111 | 6 | 4 |
| `anti-webui-math` | 68 | 5 | 1 |

The most complex library functions:

| Function | CCN | NLOC | Shape |
|---|---:|---:|---|
| `into_view`, `src/anti-webui-pdf/src/lib.rs:1003` | 144 | 934 | The whole PDF viewer view, with its closures |
| `block`, `src/anti-webui-markdown/src/editor.rs:406` | 116 | 496 | Markdown block rendering |
| `handle`, `src/anti-webui-x11/src/state.rs:668` | 105 | 274 | X11 event `match` |
| `sync`, `src/anti-webui-core/src/text/editor.rs:1641` | 83 | 414 | Text editor state sync |
| `flex`, `src/anti-webui-core/src/layout.rs:2103` | 74 | 330 | Flex layout |
| `edit_key`, `src/anti-webui-core/src/ui.rs:1898` | 70 | 158 | Key handling for text input |
| `tone`, `src/anti-webui-md3e/src/dynamic/scheme.rs:282` | 70 | 78 | Colour role table |
| `handle`, `src/anti-webui-windows/src/wndproc.rs:311` | 66 | 238 | Window message `match` |
| `resolve_node`, `src/anti-webui-core/src/layout.rs:484` | 65 | 159 | Style resolution for layout |
| `editor_key`, `src/anti-webui-core/src/ui.rs:2059` | 65 | 124 | Key handling for editors |
| `measure`, `src/anti-webui-core/src/text/editor.rs:2118` | 58 | 220 | Text editor measurement |

- Behaviour: most of the high scores are one of three shapes: an OS event dispatcher (`x11/state.rs:668`, `windows/wndproc.rs:311`), a lookup table written as a `match` (`scheme.rs:282`, `macos/keyboard.rs:8`), or a view builder whose closures count as part of it (`pdf/lib.rs:1003`, `md3e/components/time_picker.rs:76`). The ones that are logic, not dispatch, sit in `anti-webui-core`'s text editor, key handling, and layout: `sync`, `measure`, `edit_key`, `editor_key`, `flex`, and `resolve_node`.
- Quality notes: these six core functions have many integration tests (`src/anti-webui-core/tests/text_editor.rs`, `layout.rs`), but A6's mutation sample of the neighbouring undo history still missed 2 of 9 mutants ([A6](audit-tests.md)). Their branch coverage was not measured.
- Label: Confirmed (counts); Inferred (the three shapes, from reading the top 15)

### Duplication

- Status: partial
- Evidence: jscpd found 106 clones of at least 100 tokens, 2,111 lines in all, 1.05% of 201,557 lines. By kind: 1,169 lines between test files, 817 between library files, 125 between examples.

| Between | Lines |
|---|---:|
| `anti-webui-app` (inside) | 472 |
| `anti-webui-wayland` (inside, mostly protocol tests) | 356 |
| `anti-webui-macos` and `anti-webui-windows` | 244 |
| `anti-webui-wayland` and `anti-webui-x11` | 145 |
| `anti-webui-pdf` (inside) | 103 |
| `anti-webui-md3e` and `anti-webui-widgets` | 90 |
| `anti-webui-macos` and `anti-webui-x11` | 85 |
| `anti-webui-core` and `anti-webui-markdown` | 75 |

- Behaviour: the largest clone is a 90-line Wayland test harness copied between `src/anti-webui-wayland/tests/layer_protocol.rs:58` and `lock_protocol.rs:59`, with 30 more lines shared with `popup_grab_protocol.rs`. Text-editor test setup is copied between `anti-webui-core/tests/text_editor.rs:32` and `anti-webui-markdown/tests/editor.rs:29`. In library code, the four backends repeat the window-context methods (`src/anti-webui-wayland/src/lib.rs:598` and `src/anti-webui-x11/src/lib.rs:562`; `src/anti-webui-macos/src/lib.rs:227` and `src/anti-webui-windows/src/lib.rs:242`) and the keyboard tables (`wayland/src/keyboard.rs:93` and `x11/src/keyboard.rs:114`). A4c records the cause: each backend implements the `Host` seam on its own ([platform abstraction](audit-capabilities-platform.md)).
- Label: Confirmed

## Baseline violations

These are the numbers fork changes must not make worse. Each is repeatable with the commands in [Method](#method).

| Check | Baseline |
|---|---|
| `cargo fmt --all --check` | Fails: 30 hunks in 20 files |
| `cargo clippy --workspace --all-targets --locked`, default lints | 1,159 warnings: 1,109 `float_literal_f32_fallback`, 47 `chunks_exact_to_as_chunks`, 3 others; 0 errors |
| Same, Windows target (`anti-webui-windows`, `-video`, `-wgpu`) | 10 `chunks_exact_to_as_chunks`; 0 errors |
| `clippy::undocumented_unsafe_blocks` | 1 on Linux, 6 on the Windows target; macOS not checked (38 blocks and 8 impls without `SAFETY:` by the source census) |
| `#[allow]` attributes | 55 |
| Library functions with CCN > 25 | 48 |
| Duplication, jscpd at 100 tokens | 1.05% (2,111 lines) |

## Risks and technical debt

| Item | Impact | Likelihood | Evidence |
|---|---|---|---|
| The f32 literal fallback becomes a hard error in a future Rust release | Every crate that styles a view, and every downstream app, stops compiling on that toolchain; the pinned 1.99.0 hides it until the pin moves | High | [The f32 literal fallback](#the-f32-literal-fallback); rust-lang/rust#154024 |
| No `[workspace.lints]`, `rustfmt.toml`, or CI gate for formatting and Clippy | New warnings arrive unnoticed; the fork has no baseline to hold without its own gate | High | [Method](#method); [CI plan](../quality/ci-and-quality-services.md) |
| macOS `unsafe` code mostly undocumented and not lint-checked here | A soundness bug in the AppKit glue is harder to review; no device to run it (A5) | Medium | [`unsafe` code](#unsafe-code) |
| Core text-editor and layout functions above CCN 50 | Changes to editing, IME, and flex layout are hard to review and easy to break in branches the tests miss | Medium | [Complexity](#complexity); A6 coverage |
| Backend context methods copied four times | A fix to one backend's window API misses the others | Medium | [Duplication](#duplication); [A4c](audit-capabilities-platform.md) |

## Open questions

- Which Rust release turns `float_literal_f32_fallback` into an error? Answered by rust-lang/rust#154024 and the release notes. It sets how long the toolchain pin can lag.
- Does upstream want the `setter!` fix? Answered by asking upstream (needs the maintainer's approval to contact them, per [AGENTS.md](../../../AGENTS.md#upstream)).
- What does Clippy say about `anti-webui-macos`? Answered by `rustup target add aarch64-apple-darwin` and a Clippy run with `--target aarch64-apple-darwin`, if the `objc2` build scripts allow cross-checking; otherwise on a Mac (A5).
- Which of the six core functions above CCN 50 have untested branches? Answered by branch coverage (`cargo llvm-cov --branch`, nightly) or a mutation run scoped to those functions.

## Inputs for gap analysis

- Phase 2 gate candidates: `cargo fmt --check` and `cargo clippy -D warnings` on fork-owned crates from the start, and a ratchet for upstream-owned crates that fails only when the counts in [Baseline violations](#baseline-violations) rise ([tool matrix](../quality/tool-adoption-matrix.md)).
- The fork's style API should take typed values, so that literals never depend on a compiler fallback.
- `clippy::undocumented_unsafe_blocks` can be denied on fork-owned crates now. Upstream already meets it everywhere but macOS, five sites that share one comment, and `src/anti-webui-windows/src/testing.rs:100`.
- A shared backend seam would remove the copied context methods ([A4c](audit-capabilities-platform.md)).
