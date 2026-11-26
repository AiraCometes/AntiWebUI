# Audit: rendering capabilities

- Date: 2026-11-25
- Upstream commit: `6908e3119db4ecf7080f57de59870e083f2cbc98`
- Auditor: Pi, DeepSeek V4.1 Flash (draft); reviewed by Claude Code, Claude Opus 5.5
- Plan item: [A4b](phase-1-audit-plan.md#workstreams)

## Scope

The renderer half of the framework: `src/anti-webui-wgpu` (11,485 lines of Rust and 1,045 lines of WGSL) and `src/anti-webui-app` (5,189 lines of runner, presentation, pacing, and frame statistics, and 7,271 lines of pixel tests), plus the core modules those two consume: `scene.rs`, `damage.rs`, `paint.rs`, `animation.rs`, `canvas.rs`, `shader.rs`, `visibility.rs`, and `layer.rs`. The upstream design documents read as the yardstick are `05-rendering.md`, `09-shadows-stacking-popups.md`, `10-frame-pacing.md`, `11-gpu-fill.md`, `12-transform.md` (the compositing half), `13-animation.md`, `14-shaders.md`, `16-filters.md`, `29-content-visibility.md`, and `41-gpu-canvas.md`.

The wgpu fork itself is [A3](audit-wgpu-fork.md) and is not repeated here. Laying out, resolving styles, and deciding what to redraw are [A4a](audit-capabilities-core.md): this file says how the renderer acts on a `Scene` and a `Damage`, not how core decides the redraw. The Wayland, X11, macOS, and Windows backends, multi-window, and accessibility are A4c. Running the GPU tests, coverage, and benchmarks is A6, and the full `unsafe`, Clippy, and formatting baseline is A7. Decoding images, video, and SVG is out of scope: only what the renderer does with the result is covered.

Reading note: `anti-webui-core/src/layer.rs` is the Wayland `wlr-layer-shell` API, not compositing layers. Compositing layers are `scene::Layer` in `anti-webui-core/src/scene.rs` and `LayerCache` in `anti-webui-wgpu/src/renderer.rs`. Confirmed by reading both.

## Method

Reading order: the design documents above, then `scene.rs` and `damage.rs` (the contract with core), then `renderer.rs`, `lib.rs`, `glyph_cache.rs`, `path.rs`, `effects.rs`, `custom_shader.rs`, `canvas.rs`, and the platform modules `dmabuf.rs`, `gbm.rs`, `dxgi.rs`, `iosurface.rs`; then `anti-webui-app`'s `present.rs`, `present/*`, `pacing.rs`, `vsync.rs`, and `runner.rs`; then the tests and examples.

| Question | Command |
|---|---|
| Size of each file | `wc -l src/anti-webui-wgpu/src/*.rs src/anti-webui-wgpu/src/*.wgsl src/anti-webui-app/src/*.rs src/anti-webui-app/src/present/*.rs` |
| Where a symbol is | `rg -n '^(pub )?(fn\|struct\|enum\|impl\|const\|static\|type\|trait)' <file>` |
| Sections of a design document | `rg -n '^#{1,3} ' docs/design/<file>.md` |
| A document's wording | `sed -n '<range>p' docs/design/<file>.md` |
| Tests in the renderer crate | `cargo test -p anti-webui-wgpu --locked -- --list`, then `rg -c ': test$'` |
| Tests in both crates at once | `cargo test -p anti-webui-wgpu -p anti-webui-app --locked -- --list` — fails on this machine, see below |
| Tests in the app crate | `cargo test -p anti-webui-app --locked --no-default-features --features wayland -- --list`, then `rg -c ': test$'` |
| `#[test]` per file | `rg -c '#\[test\]' src/anti-webui-app/tests/*.rs src/anti-webui-app/src/*.rs src/anti-webui-wgpu/src/*.rs \| sort -t: -k2 -rn` |
| Which tests draw | `rg -n 'headless\|Gpu::headless\|render_rgba\|skipping' src/anti-webui-app/tests/<file>.rs` |
| `unsafe` per line | `rg -n 'unsafe' src/anti-webui-wgpu/src src/anti-webui-app/src`, then `rg -o 'unsafe'` to count occurrences rather than lines |
| `unsafe` with a `// SAFETY:` comment | a Python script over the same files: an occurrence counts as documented when `SAFETY` is on its line or the three lines above (rules in prose below, as the script stayed in `$TMPDIR`) |
| Fork checks | `mise run check` |

Test count, from `--list`: 38 tests in `anti-webui-wgpu` (plus one `ignore` doctest) and 191 in `anti-webui-app` (50 unit tests inside `src/`, 141 in `tests/`), 229 in all. The second crate's list needs `--no-default-features --features wayland`: `cargo test -p anti-webui-app --locked -- --list` fails here to link the test binaries, `rust-lld: error: unable to find library -lxkbcommon-x11`, because this machine has `libxkbcommon0` and `libxkbcommon-dev` but not `libxkbcommon-x11-0`. That is a fact about this machine, not about the crate: with the X11 backend compiled in the count may differ, and this audit did not measure by how much. Label: Unverified (the count with the `x11` feature).

`unsafe`, for the numbers A7 will expand: 125 occurrences of the keyword in `anti-webui-wgpu/src` and 7 in `anti-webui-app/src` (128 and 7 lines match the string; three of the first are the wgpu call `create_surface_unsafe`, not the keyword). 112 of the first and 6 of the second have `// SAFETY:` within the three lines above. The 14 without are `unsafe fn`s documented with `# Safety` (`Gpu::with_window` `lib.rs:177`, `add_window` `lib.rs:305`, `limit_vulkan_drivers` `drivers.rs:93`, `Swapchain::new` `dxgi.rs:107`, `open_for_dmabuf` `dmabuf.rs:175`), eight `unsafe extern "C" fn` type aliases in `gbm.rs:17`–`32`, and the `unsafe extern "system"` block in `anti-webui-app/src/clock.rs:51`. In the tests there is one: `anti-webui-app/tests/video.rs:94`, a `MemoryPlane` over a `Vec` that is kept alive, with its `// SAFETY:` comment. None in `anti-webui-wgpu`'s 38.

## Findings

### Checklist

| Capability from the [plan](phase-1-audit-plan.md#capability-checklist) | Status | Subsection |
|---|---|---|
| The wgpu renderer | present | [wgpu renderer](#wgpu-renderer) |
| Damage tracking | present, with limits | [Damage tracking](#damage-tracking) |
| Compositing layers and texture caches | present | [Compositing layers and texture caches](#compositing-layers-and-texture-caches) |
| The glyph atlas and subpixel rendering | partial: the atlas and 1/4 px positioning exist, LCD subpixel antialiasing does not | [Glyph atlas and subpixel positioning](#glyph-atlas-and-subpixel-positioning) |
| GPU canvas | present | [GPU canvas](#gpu-canvas) |
| Animation | present | [Animation](#animation) |
| Shaders and filters (also in this file) | present | [Shaders and filters](#shaders-and-filters) |
| Frame pacing (also in this file) | present | [Frame pacing](#frame-pacing) |
| Content visibility (also in this file) | present, decided entirely in core | [Content visibility](#content-visibility) |
| Images and shapes on the GPU path (also in this file) | present | [Images and shapes on the GPU path](#images-and-shapes-on-the-gpu-path) |

Labels: Confirmed, from the rows below.

### wgpu renderer

- Status: present
- Evidence: `src/anti-webui-wgpu/src/renderer.rs` (3,491 lines), `shader.wgsl` (671 lines), `effects.wgsl` (195), `path.rs` (1,518), `video.rs` (549), `lib.rs` (754). `Primitive` has ten variants (`core/src/scene.rs:539`). Design documents: `05-rendering.md` sections "入力: Scene" and "GPU 側", `09-shadows-stacking-popups.md` section "影", `11-gpu-fill.md` sections "単色の経路" and "箱の分割", `25-svg.md` (read for the shape pipeline).
- Behaviour, the pipelines: `Renderer::new` (`renderer.rs:648`) makes one render pipeline per kind of instance, all with premultiplied alpha blending and no depth buffer and no multisampling; the two `opaque` pipelines blend nothing, for what covers a whole pixel.

| Primitive | Instance | Pipeline | Notes |
|---|---|---|---|
| `Quad` | `SolidInstance`, `QuadInstance` | `solid`, `opaque`, `quad` | Sliced into solid and shaded pieces when it is worth it |
| `Shadow` | `QuadInstance` | `quad` | Up to four rectangles around a transparent hole |
| `Glyphs` | `GlyphInstance` | `glyph` | One atlas, one texture sample per glyph |
| `Image` | `PictureInstance` | `picture`, `opaque_picture` | Three bits of the batch choose one of eight samplers |
| `Video` | `VideoInstance` | `video` (`video.rs`) | Three planes, with a PQ/HLG-to-SDR conversion in the shader |
| `Canvas` | `PictureInstance` | `picture` | Drawn into its own texture first |
| `Path` | `PathInstance` | `path` (`path.rs`) | Rectangles of precomputed coverage |
| `Layer` | `LayerInstance` | `layer` | Samples the layer's texture through a matrix |
| `Shader` | `ShaderInstance` | one per compiled shader | Compiled from WGSL at run time |
| `Backdrop` | `BackdropInstance` | `backdrop` (`effects.rs`) | Reads what is already drawn |

- Behaviour, from `Scene` to draw calls: `Renderer::record` (`renderer.rs:2612`) plans layers, builds instance data (`try_prepare`, `1826`), uploads it (`queue.write_buffer` into per-kind buffers that grow and are reused, `InstanceBuffer::upload`, `511`), uploads glyphs, pictures, and video, and then opens passes: one for the window (split into segments around effect steps), and one per damaged region of each layer. Draw calls are runs of consecutive primitives of the same kind: `Batch` has 11 variants and `push_batch` (`3363`) extends the last batch of the same kind, so a run of glyphs with different clips is still one draw. Clips are per instance (a rectangle plus four radii) and evaluated in the fragment shader (`clip_coverage`), which is what keeps the batching simple.
- Behaviour, the box slicer: `slice_quad` (`renderer.rs:396`) splits a box into regions that do not overlap and cover everything it draws. An unrounded box is one solid piece, or a middle and four border bands when it has a border, the band colour being the border composited over the background on the CPU. A rounded box without a border is solid in the middle and the two side bands with the four corners shaded by the SDF, unless those solid areas come to less than `MIN_SOLID_PIXELS = 4096`, in which case the whole box does; with a border, the middle is solid when it is big enough and the four bands are shaded. When the borders and radii leave no inner rectangle at all, the whole box is shaded. Every shaded part is one `QuadInstance` carrying both a `shape` and an `area`, so a box and a shadow share the pipeline, which is what `09`'s 「描く範囲を減らす」 argues for on recording cost.
- Behaviour, presentation: `configure_window` (`lib.rs:352`) picks `Bgra8Unorm` or `Rgba8Unorm` (non-sRGB, so blending happens in gamma space, as browsers do), `PresentMode::Mailbox` when offered and `Fifo` otherwise, `desired_maximum_frame_latency: 2`, and the first available of `PreMultiplied`, `PostMultiplied` (macOS only), `Opaque`.
- Behaviour, backends per OS, from `Gpu::with_window` (`lib.rs:189`): Windows tries `DX12`, then `PRIMARY` (Vulkan, Metal, DX12), then `GL`, because only a DX12 swapchain on a DirectComposition visual has alpha for popups (`instance_desc`, `lib.rs:126`); everywhere else `PRIMARY` then `GL`, because Mesa's GL pulls in LLVM. `WGPU_BACKEND` in the environment replaces the list with `Backends::all()`. On Linux, `limit_vulkan_drivers` (`drivers.rs`) can restrict the Vulkan loader to the compositor's own GPU before any app thread exists.
- Behaviour, the Linux buffers: `dmabuf.rs` (1,449 lines) is the allocator and importer, not the presentation path itself. `DmabufContext::new` takes the raw Vulkan handles out of wgpu's HAL, requires exportable-and-importable sync-file semaphores and at least one exportable DRM format modifier, finds the render node from `VkPhysicalDeviceDrmPropertiesEXT`, and `allocate` first tries GBM then falls back to allocating with Vulkan and exporting. `gbm.rs` (184 lines) loads `libgbm.so.1` at run time with `libloading`, so GBM is not a build-time dependency; `ANTIWEBUI_ALLOCATOR=vulkan` skips it. The same file imports decoder buffers for video (`VideoImporter`, `dmabuf.rs:1063`). Which surface gets which buffer is `anti-webui-app/src/present/dmabuf.rs`, which is A4c's.
- Quality notes: 38 unit tests in the crate, none needing a GPU adapter: glyph cache packing and growth, the box slicer, the shape rasteriser, blur weights and level counts, shader parameter layout and error text, and driver-manifest parsing. Allocation: instance data and the draw list are rebuilt per frame, one `CommandEncoder` per frame, no persistent vertex or index data. Complexity: `renderer.rs` is 30% of the crate's Rust and holds pipelines, planning, instancing, and recording in one `impl`.
- Label: Confirmed

### Damage tracking

- Status: present, with limits
- Evidence: `core/src/damage.rs` (465 lines): `Damage` (`add` merges and caps at `MAX_RECTS = 8`, line 11), `diff` (142), `diff_content` (153), `backdrop_read` (201), `expand_for_backdrops` (214), `primitive_bounds` (273), `DamageHistory` with `LEN = 8` (107). Renderer: `render_damaged` (`renderer.rs:2586`), the damaged branch of `record` (`2770`–`2812`), the pruning in `try_prepare` (`1957`). Presenter: `Presenter::present` (`present.rs:331`), `damage::diff(prev, &scene)` (`375`), the early `Presented::Unchanged` (`386`). Design document: `05-rendering.md` section "ダメージトラッキング", subsections "1. 変更領域を求める（core）", "2. 部分描画（renderer）", and "3. 提示とコンポジタへの通知（presenter）".
- Behaviour, how the rectangles are found: the scene is a list of chunks keyed by `ChunkKey(node, part)`. Chunks are matched by key against the previous frame; an added or removed chunk damages its own primitives, one that moved before an already matched chunk damages both, and a chunk of equal length damages only the primitives that differ. A size or `clear` change damages everything. A primitive's rectangle is its own bounds intersected with its clip; a glyph run uses the font's `head` box plus padding for emboldening, skew, stroke, and quarter-pixel positions. Rectangles that merge for less than four times their wasted area are merged, and the list never exceeds 8, because each one costs a scissored pass.
- Behaviour, who computes it: core provides `diff`, `diff_content`, and `expand_for_backdrops`, but never calls them, and `Ui` holds no damage at all. `Presenter::present` is the only caller (`present.rs:375`), and `Renderer::render_damaged` the only consumer of the rectangles; a renderer handed damage draws all of a layer's content otherwise.
- Behaviour, how the rectangles are used: with damage, each rectangle is cleared by drawing one opaque solid instance (pushed first, at index `i`) and then every primitive touching it is redrawn inside a scissor of that rectangle; a primitive touching none is never instanced, so its cost is not paid at all. `render_damaged` clips the rectangles to the window and expands them for backdrops first. Layers are treated the same way in their own coordinates, with the damage converted through each layer's inverse matrix and torus-split into at most four pieces per texture edge crossing.
- Behaviour, presentation and buffer age: the presenter diffs against the last scene presented on that surface and does nothing when the result is empty, unless a held frame was dropped and its damage is still unsent (`TargetKind::Own(t) if t.has_unsent()`). Buffers we own are presented with the union of the last `age` frames' damage (`present/dmabuf.rs:348`–`357`, `present/iosurface.rs:162`, `dxgi.rs:224`), or with all of it when the age is unknown.

| Platform | Buffers | Rendered with damage | Compositor is told |
|---|---|---|---|
| Wayland and X11 | our dmabufs, 4 at most (`present/dmabuf.rs:22`), explicit or implicit sync | `renderer.render_damaged` (`dmabuf.rs:370`) | `wl_surface.damage_buffer` through `ctx.present_buffer(…, damage.rects(), sync)` (`dmabuf.rs:462`) |
| macOS | our `IOSurface`s, 3 at most (`present/iosurface.rs:19`) | `renderer.render_damaged` (`iosurface.rs:175`) | Nothing: Core Animation takes no damage |
| Windows, own DXGI swapchain | the swapchain's buffers | `renderer.render_damaged` into a kept offscreen, then a copy | `Present1` dirty rectangles (`dxgi.rs:274`–`298`) |
| Windows, wgpu surface | the offscreen kept frame, copied into the swapchain | same as above | Nothing |
| Anywhere as the fallback (`ANTIWEBUI_PRESENT=surface`, GL, no exportable dmabuf) | wgpu surface | No: `present_damaged` returns the whole window (`lib.rs:626`–`640`) | Nothing |

- Limits: the fallback row means a full redraw of the window per frame; only "nothing changed, present nothing" survives there. Overlapping rectangles redraw their overlap twice. The cost of a damaged frame is the number of rectangles times the primitives touching each, not the damaged area, which the per-layer scan of `content.primitives` in `try_prepare` shows.
- Quality notes: 7 unit tests in `damage.rs` (no change, changed primitive, inserted and removed chunks, reordering, size or colour change, merging and the cap, history unions). No `unsafe`. `ANTIWEBUI_DEBUG_DAMAGE=1` prints the rectangles, the buffer age, and the repainted percentage per frame (`present.rs:568`, turned on by `debug_enabled` at `:81`).
- Label: Confirmed for the code paths; Unverified for how the buffer-age path behaves on a compositor, which this audit did not drive

### Compositing layers and texture caches

- Status: present
- Evidence: `core/src/scene.rs:180` (`Layer`, `matrix`, `same_placement`, `is_pixel_aligned`), `core/src/paint.rs:895` (`fn layer`, a scroll root), `953` (`transformed_layer`), `1113` (`faded_layer`), `1173` (`layer_filter`), the keys at `paint.rs:940`, `1028`, `1161` and the special ones `DRAG_LAYER` (`paint.rs:96`) and `NOT_A_NODE | …` (`paint.rs:1998`). Renderer: `LayerCache` (`renderer.rs:215`), `create_layer_cache` (1217), `prepare_output` (1296), `plan_layers` (1346), `plan_layer` (1369), `keep_layer` (1485), `torus_parts` (303), `clear_layer_caches` (1213), `forget_target` (1121). Design documents: `05-rendering.md` section "スクロールレイヤー" (subsections "テクスチャ（トーラス）" and "合成") and `12-transform.md` sections "方式" and "レイヤー" under "レンダラ".
- Behaviour: a subtree becomes a layer for three reasons — it scrolls, it is transformed, or it needs a group opacity or a filter (`faded_layer` is where `Layer::filter` comes from). The cache key is the node's `NodeId` as a `u64`, so it is stable across frames; popups are separate targets and every cache records `owner`, the target id, so popups never share one. The texture holds the layer's content at `k` texels per content pixel, wrapped in both axes (the "torus"), so scrolling never moves a texel: content pixel `(x, y)` lives at `(kx mod w, ky mod h)`, and a damaged rectangle that crosses a texture edge is split into at most four pieces. `k` grows with the scale, is at least 2 when the layer is rotated, never decreases while the layer lives, is capped at `MAX_K = 4`, and is reduced when the texture would exceed `max_texture_dimension_2d`.

| Cache | Sized by | Kept until | Budget |
|---|---|---|---|
| Layer texture | visible area × `k`, rounded up by 16 px under 256 and 128 px above | the layer is not used in a frame for its target (`plan_layers`' `retain`), except that `keep_layer` holds a layer and its nested layers that the frame does not touch | None |
| Filter output of a layer | the layer's damaged rectangle × `k`, rounded to 64 px | it would be more than four times the needed area, or the layer goes | None |
| Effect pool textures | each step's rectangle | 8 frames without use (`POOL_KEEP_FRAMES`, `effects.rs:15`) | None |
| Window canvas, for backdrop reads | the window | nothing in the scene reads behind it (`record`, `2637`–`2648`) | One per target |
| `gpu_canvas` texture | the content box × `k`, capped at the device limit | the painter is gone (`gpu_canvases.retain(…)`, `2653`) | None |
| Picture (image or GIF frame) | the bitmap's own size, mip levels included | no `Arc<Bitmap>` is left (`pictures.retain(…)`, `2650`) | None |

- Memory, as the documents record it: a full-window scroll layer of 1620×1500 is about 10 MB, and 55 faded rows of a list are about 12 MB (`05`, `12`). Confirmed (as documented upstream); Unverified (not measured here). There is no total, and no attempt to trade resolution for memory except `k`'s cap.
- Quality notes: no unit test covers layer planning, eviction, or the torus. The pixel tests cover the results (`a_backdrop_inside_a_scrolled_layer_repaints_the_same_in_parts_as_at_once`, `a_filter_repaints_the_same_in_parts_as_at_once`). A layer that stops being painted, because the node is hidden or skipped by `content-visibility`, releases its texture at once and redraws its whole content when it returns.
- Label: Confirmed

### Glyph atlas and subpixel positioning

- Status: partial. The atlas, the packing, and quarter-pixel horizontal positions exist; LCD (RGB) subpixel antialiasing does not.
- Evidence: `glyph_cache.rs` (442 lines): `GlyphKey` (19: font id, font index, glyph id, size in 1/64 px, `subpixel: u8`, embolden, skew, stroke in 1/64 px, instance), `GlyphCache::place_run` (164), `rasterize` (227), `allocate` (128), `size_when_full` (99), `mostly_passing` (107), `clear` (85); `SUBPIXEL_STEPS = 4.0` (11), `SIZE_TO_STAY_AT = 2048` (14), `LASTING_INSTANCES = 8` (16), `ATLAS_INITIAL = 1024` (`renderer.rs:27`). Shader: `fs_glyph` (`shader.wgsl:264`). Design documents: `05-rendering.md` section "グリフキャッシュ", `17-font-axes.md` for the atlas-size rule, and `04-text.md` for shaping, which is [A4a](audit-capabilities-core.md).
- Behaviour, rasterising: swash, with `Format::Alpha` and hinting off, sources in the order colour outline, colour bitmap, outline; synthetic emboldening at `size / 24` and a synthetic skew; a stroke is an outline with round joins, so a colour glyph has no stroke to widen. The atlas is one square RGBA8 texture packed in shelves with one pixel of padding: mask glyphs are stored as `[a, a, a, a]`, colour glyphs are premultiplied on the CPU because swash produces straight alpha.
- Behaviour, positioning: the x position of a glyph is snapped to a quarter pixel and the remainder becomes part of the key; the y position is rounded to a whole pixel, so there is no vertical subpixel positioning. Variable font axes come through as normalised coordinates, interned to an instance index that is part of the key; the area each instance takes in the atlas is what `mostly_passing` measures.
- Behaviour, growth and eviction: when a glyph does not fit, `Renderer::prepare` clears the atlas and retries the same preparation (`1539`–`1546`), doubling the side up to the device limit unless the atlas is already at least 2048 px and more than half of it is taken by coordinates that come and go, in which case it stays the same size. That rule is `17-font-axes.md`'s, and it is the answer to a font whose axes can make images without end. There is no per-glyph eviction and no LRU.
- Behaviour, antialiasing: text is grayscale. A `SubpixelMask` from swash is averaged to one alpha (`glyph_cache.rs:291`), the shader multiplies the text colour by the atlas alpha (`fs_glyph`), and the atlas sampler is `Nearest`. Blending is premultiplied in a non-sRGB target, so coverage blends in gamma space, as it does in a browser without subpixel antialiasing.
- Quality notes: 6 unit tests, all on packing and the grow-versus-clear rule, one using Roboto Flex with two sets of coordinates. No `unsafe`. Upload cost is one `queue.write_texture` per new glyph, per frame in which it is new. A full atlas throws away every glyph, including those still on screen, and the same frame rasterises them again.
- Label: Confirmed

### GPU canvas

- Status: present
- Evidence: `wgpu/src/canvas.rs` (438 lines): `GpuPaint` (21: `Props`, `new`, `render`), `GpuInit` (38), `GpuFrame` (49) with `begin_pass` (67) and `begin_srgb_pass` (71), `gpu_canvas` (103), `portable` (137), `portable_canvas` (279), the thread-local `PAINTERS` table of `Weak`, `guarded` (431). Core: `core/src/canvas.rs` (186 lines): `CanvasData` (21), `Portable` (64), `GpuCanvas` (85), `gpu_canvas_from` (97), `set_paint` (57). Scene: `CanvasDraw` (`scene.rs:414`). Renderer: `push_canvas` (`renderer.rs:2326`), `create_canvas_texture` (2378), `draw_canvases` (2423). Design document: `41-gpu-canvas.md`, sections "API", "方式", "描くタイミング", "色とアルファ", and "メモリ".
- Behaviour: what the user draws is wgpu. The trait has a `Props` type compared with `PartialEq` — equal props mean no redraw, and a canvas that is still costs no frame at all — a one-time `new` on the window's device, and `render` into the content box. `GpuFrame` hands out the device, the queue, an encoder of its own, the target view in the window's format (`Rgba8Unorm`) and, unless the backend is GL, an sRGB view of the same texture (`srgb_views`, `renderer.rs:1103`). The renderer keeps one texture per canvas, as large as the content box times the layer's `k` and capped at `max_texture_dimension_2d`, and redraws only when `version`, the size, or the device changed.
- Behaviour, safety of user code: each canvas is drawn before the window's pass, in its own encoder submitted on its own, so a validation error in user code cannot discard the window's frame; `new` and `render` run inside a validation error scope (`guarded`) and are reported once. Compositing uses the picture pipeline, so rounded corners, clip, and `opacity` behave as they do for an image.
- Behaviour, `version`: core increments it when the effect's `update` says the props changed, then marks the node as repainted with `mark_shader_of` (`core/src/canvas.rs:181`), so the damage is exactly the box and the layer's content is reused.
- Quality notes: 8 pixel tests in `anti-webui-app/tests/gpu_canvas.rs`, all skipped without an adapter. No `unsafe` in `canvas.rs` (checked with `rg -n unsafe …/canvas.rs`, which found none). No depth buffer, no MSAA, and no way to ask for extra device features: `41-gpu-canvas.md`'s "やっていないこと" says the user brings those. A lost device is not recovered; a painter is remade only when it is asked to draw with a different device. A texture is kept while the painter lives, including while `content-visibility` skips the box, which `41` states.
- Label: Confirmed

### Shaders and filters

- Status: present
- Evidence: core `shader.rs` (661 lines): `ShaderSource` (159), `Shader` (189), `ShaderPaint` (244) with `inputs` and `reach`, `MAX_INPUTS = 4` (295), `BLUR_SIGMAS` and `blur_levels` (298, 304), `Image` (334) with `content`, `backdrop`, `blur`, `through`, and `reach`, `ShaderParams` (145). Renderer `custom_shader.rs` (420): `PRELUDE` and `ENTRY` (6, 7), `compile` (189), `check_shader` (119, naga validation without a GPU), `compile_shaders` (`renderer.rs:1555`, one thread per core), `shader_cache.rs` (107, a DX12 pipeline cache on disk). Filters: `effects.rs` (978) with `Step` (`Clear`, `Copy`, `Output`, `Gauss`, `Blur`, `Shader`), `blur` (654), the pool, `POOL_KEEP_FRAMES = 8` (15), and `effects.wgsl`. Design documents: `14-shaders.md` sections "API", "方式", "実行時の扱い", and `16-filters.md` sections "API", "方式" (subsections "ブラー: Dual Kawase", "式の評価", "描画先"), and "変更矩形の拡張".
- Evidence, the four built-in WGSL files:

| File | Lines | What it holds |
|---|---|---|
| `shader.wgsl` | 671 | Every built-in pipeline: solid, quad with the SDF and the blurred shadow, glyph, layer, picture, video, and the shape path's shader |
| `effects.wgsl` | 195 | `fs_down`, `fs_up` (Dual Kawase), `fs_gauss`, `fs_show` (the canvas to the surface), `fs_backdrop` |
| `custom_prelude.wgsl` | 105 | What a user shader may call: `Frag`, `AwBox`, `aw_shape_distance`, `aw_coverage`, `aw_rounded_box_distance`, and the `read0`–`read3` of the stage's inputs |
| `custom_entry.wgsl` | 74 | The vertex and fragment entry points wrapped around the user's `paint`, so a shader is one function |

- Behaviour, custom shaders: a shader is a `static` holding a name and WGSL that defines only `fn paint(f: Frag) -> vec4<f32>`. The renderer wraps it in the prelude, a `struct Params` generated from the Rust fields, the user's code, and the entry point. Parameters are packed by the derive so WGSL alignment holds, and the box, border, padding, radii, scale, `f.px`, and `f.ipx` arrive as `14` documents. `fill` paints under the background, `overlay` above the border and inner shadow. Compilation happens on first use or ahead of time through `preload()`; an error names the type and the user's line, is printed once, and draws nothing. `check_shader` validates without a device, which is what four of the crate's seven shader tests use.
- Behaviour, filters: `filter` and `blur` make the node a layer whose `LayerFilter` carries the image expression, the box, and a `reach`. The renderer walks the expression tree, works out each stage's rectangle (a stage shared by `clone` is evaluated once), and evaluates leaves first into pooled textures: a Dual Kawase chain down and up whose last level is mixed with the level above by `blur_levels`, or a separable 15-tap Gaussian when σ ≤ 1.68. `backdrop` reads what is already drawn; when a scene contains any backdrop read, the window is drawn into a full-size canvas texture and only the damaged rectangles are shown on the surface (`record`, `2770`–`2812`), which is `16`'s "描画先" decision. Damage is expanded by what each backdrop reads, until nothing more is added (`expand_for_backdrops`).
- Limits, from the documents' own lists: no `Image::previous()`, no recomputation of only the changed part of a filter, no backdrop inside a `filter`, no built-in drop shadow, no `ext-background-effect-v1` (`16`); no custom shadows and no compute shaders (`14`).
- Quality notes: 7 tests in `custom_shader.rs`, 4 in `effects.rs`, 13 pixel tests in `tests/shaders.rs`, 32 in `tests/filters.rs`. `PRELOADS` is a process-wide `Mutex<Vec<&'static ShaderSource>>` (`shader.rs:233`), the one shared mutable static on this path. Compiling a DX12 shader takes 80–430 ms in FXC (upstream's figure), which is why compilation is threaded and cached to disk.
- Label: Confirmed

### Animation

- Status: present
- Evidence: `core/src/animation.rs` (702 lines): `request_frame` (33), `frame_requested` (40), `run_frame_callbacks` (45), the per-window `Locals` and `swap_locals` (70, 78), `set_timeout` and `Timeout` (98), `Presence` (131), `RunningTransition` (135) with `redirect` (191), `Spring` (357), `Animated` (437), `animated` (451), `to` (474), `spring_to` (490), `next_frame` (534), `follow` (576), `clock` (591). Driving: `Ui::animate` (`ui.rs:628`) runs the callbacks, then `advance_transitions`, then the fling; `Ui::animating` (`ui.rs:506`) is what makes the runner ask for another frame. Design document: `13-animation.md`, sections "`transition`", "`animated`", "`enter` / `exit`", and "フレームごとの処理".
- Behaviour, three mechanisms: `transition` on a style property, resolved while styles are computed (the list, redirecting towards a new target, and the CSS reversing shortening are in `RunningTransition` and `anti-webui-style/src/transition.rs`, which [A4a](audit-capabilities-core.md) covers); `animated` and `follow`, a signal that moves by itself towards a target, with a `Transition` easing or a `Spring` integrated at a fixed 1 ms step from the delta to the target, so the frame rate does not change the path and a long pause is capped at 4 s; and presence, where `show` and `for_each` keep a leaving node for its exit style. Every frame the callbacks run inside one `batch`, so a `dyn_style` reading two moving values runs once.
- Behaviour, what a `transition` can move, by what it costs (`13-animation.md`, section "対象"):

| Group | Properties | Cost |
|---|---|---|
| Transforms and opacity of a transformed node | `Translate`, `Scale`, `Rotate`, `Opacity` | Compositing only |
| Colours, radii, shadows | `Opacity` of an untransformed node, `Background`, `BorderColor`, `OutlineColor`, `TextColor`, `Radius`, `Shadow` | Repaint |
| Size and position | `Width`, `Height`, `Margin`, `Padding`, `Inset`, `Gap`, `BorderWidth` | Layout |

`Prop` has 27 variants (`transition.rs:107`), so the document's table omits ten the code supports: `ImageOpacity`, `Blur`, `BackdropBlur`, `FontWeight`, `FontAxes`, `SvgFill`, `SvgStroke`, `StrokeWidth`, `FillOpacity`, and `StrokeOpacity`. That is doc ageing rather than a contradiction, and the cost of each extra follows from the sections above. Confirmed (the counts).

- Behaviour, which thread and which frames: everything is on the UI thread, with no locks and no `Send` bound — frame callbacks, timers, transitions, springs, and paint. A frame is requested by `request_frame`, which only sets the thread-local `WANTED` flag; `Ui::animating` reads it, `Runner::process` (`runner.rs:119`) or the end of a frame asks the platform, and `Ui::animate(now)` takes the time from the pacing clock (below) rather than from `Instant::now()`. A spring or a timer still reads `Instant::now()` inside the machine (`animation.rs:483`, `animation.rs:102`), which is [A4a](audit-capabilities-core.md)'s clock-seam risk inside the renderer's frame loop.
- Quality notes: unit tests in `animation.rs` (a redirected transition leaves one motion going, and others), and 22 in core's `tests/transition.rs`. Not implemented, as `13`'s "やっていないこと" says: keyframes, loops, delays, `Spring` inside a `transition`, `interpolate-size`, and a reduced-motion setting. The measurement tables in `13` are upstream's: Confirmed (as documented upstream); Unverified (not measured here).
- Label: Confirmed

### Frame pacing

- Status: present
- Evidence: `anti-webui-app/src/pacing.rs` (1,001 lines) and `vsync.rs` (297). `FrameAdvance` in `core/src/settings.rs:49` (`Auto`, `Enable`, `Disable`, `Custom`), `Pacing::max_advance` (`pacing.rs:358`), `begin_frame` (399), `frame_time` (414), `AutoAdvance` (128) with `observe` (187) and the trial constants (`MAX_ADVANCE = 4`, `CONTINUOUS_NS`, `LONGEST_GAP`, `WORTH_IT = 1.1`, `FIRST_RETRY`, `LONGEST_RETRY`, `CALM_FRACTION`), `FrameClock` (`vsync.rs:36`) with `frame_time` (122) and `place` (141), `PresentTrace` and `FrameRecord`, `frame_stats.rs` (297). Runner: `on_frame` (`runner.rs:250`), `on_surfaces_ready` (357), `on_may_render_ahead` (365), `on_timer` (375). Presenter: `can_hold_back` (`present.rs:203`), `commit_held` (214). Design document: `10-frame-pacing.md`, sections "方式", "見た目が変わらないフレーム", "設定", "Auto の判定", "スレッド", and "毎フレームの処理".
- Behaviour: the frame is produced as Chromium does it — render, submit, and hold the commit until the compositor's frame callback for the previous one — which only buffers we own can do. Today that is dmabuf on Linux only: the macOS `IOSurface` presenter answers `can_hold_back` with `false` (`anti-webui-app/src/present/iosurface.rs:86`), and Windows has no own presenter, so it uses the uninhabited fallback (`present.rs:15`–`17`, `present/no_dmabuf.rs:40`). Corrected on 2026-11-26 after A8. `Auto` starts with no rendering ahead, watches whether refreshes are being skipped, and tries one frame ahead for a bounded number of frames; it keeps the trial only if frames then come at least 1.1× faster, otherwise it retries after 8 s, doubling to 64 s, and it stops again once frames fit in half a refresh for 2 s. The most frames ahead is 4.
- Behaviour, one frame in order (`Runner::on_frame`, `runner.rs:250`): make the presenter if there is none; `pacing.begin_frame`, which takes in what became of earlier frames and learns when the last callback came; the time animations get from `pacing.frame_time()`; `ui.animate(now)`, which runs frame callbacks, advances transitions, and steps inertia; `ui.update()`, which resolves, lays out, positions, and builds stacking; `handle_requests`; `ui.paint_frame(now)`; the overlay; one `present` per surface (window, popups, drag icon); `pacing.end_frame` with whether anything was committed and whether the platform sends presentation feedback; then the decision about the next frame. Label: Confirmed.
- Behaviour, idle: after a frame, a redraw is asked for only if a surface must be retried (4 ms), something is animating, or the performance overlay still has a frame to show (`runner.rs:343`–`357`). With nothing animating and nothing dirty nothing is requested, so the process draws nothing until an event or a timer arrives. `tick` and `next_deadline` (`ui.rs:760`) supply the caret blink, image frames, video frames, and `set_timeout` deadlines.
- Behaviour, the time animations see: it is the frame's place on the screen's refresh grid when the last 8 frames all landed on the same refresh, and the frame's start otherwise; the grid and the period are estimated from `wp_presentation` timestamps by `10`'s "フレームの時刻を格子に揃える" rules, and `ANTIWEBUI_FRAME_TIME=start` goes back to the start time. A frame whose damage is empty commits nothing, so no callback could pace the next attempt: the runner then schedules it at the next refresh of the grid, or 4 ms later when the refresh is unknown.
- Quality notes: 33 unit tests in `pacing.rs`, 7 in `vsync.rs`, 2 in `frame_stats.rs`, 7 in `perf_overlay.rs`; none needs a GPU. The tables in `10-frame-pacing.md` (fps, latency, RMS error) are upstream's: Confirmed (as documented upstream); Unverified (not measured here).
- Label: Confirmed for the logic, by reading and by its unit tests; Unverified for what a compositor does with it, which this audit did not watch

### Content visibility

- Status: present, decided entirely in core
- Evidence: `core/src/visibility.rs` (211 lines): `Visibility` (15), `check` (39), `scroller_of` (112), `anchor_in` (121), `offset_in` (140), `WAITS = 8` (11). Paint side: `paint.rs:1095` (`if !node.visible || (node.offscreen && self.root != Some(id))`) and `paint.rs:563`. Design document: `29-content-visibility.md`, sections "API", "中身を飛ばすとき", "関わりがあるとき（`auto`）", and "スクロールが跳ねないようにする（scroll anchoring）".
- Behaviour: `auto` and `hidden` are core's decision, taken after positioning and before the frame's paint. A node becomes relevant when a box expanded by half the viewport each way meets the window, or when the focus, the selection, or a `scroll_into_view` target is inside it; a change marks only that node for layout, and the layout runs again in the same update, so no frame shows a box without its contents. Scroll anchoring keeps the visible place when estimated sizes are replaced by real ones.
- Behaviour, what the renderer sees: nothing. A skipped node produces no primitives, so damage, layer planning, and batching are untouched, and the saving happens before the scene exists. A layer whose node stops being painted releases its texture (see [Compositing layers and texture caches](#compositing-layers-and-texture-caches)), while a `gpu_canvas` texture is kept, which `41-gpu-canvas.md` says it is.
- Effect, from `29`: a 195 KB Markdown document paints in 71–76 ms instead of 154 ms, and scrolling a page costs 1.5 ms instead of 0.1 ms. Confirmed (as documented upstream); Unverified (not measured here).
- Quality notes: no unit test in `visibility.rs`; the behaviour is covered by core's and the Markdown crate's tests, which [A4a](audit-capabilities-core.md) lists. Not implemented, from `29`'s "これから": not creating the nodes of far-away blocks at all, and laying blocks out during idle frames.
- Label: Confirmed by reading, and the renderer's part is Confirmed absent by construction: no renderer code mentions `content-visibility` (`rg -n 'content_visibility\|relevant' src/anti-webui-wgpu/src` returns nothing)

### Images and shapes on the GPU path

- Status: present
- Evidence: images: `push_picture` (`renderer.rs:2255`), `create_picture` (2485), `upload_pictures` (2525), `picture_counts` (1166), `every_nth_pixel` (3158), and the eight samplers built in `Renderer::new` (860–890). Shapes: `path.rs` (1,518 lines) with `FLATTENING = 0.05`, `BAND = 4`, `BLOCK = 40`, `WHOLE_LIMIT = 8192`, `MANY_SHAPES = 48`, `STORE_WIDTH = 2048`, `STORE_MAX_ROWS = 4096`, `draw_state` (1154); `PathGeometry` and `PathDraw` in `scene.rs`; gradient rows built by `gradient_rows` (`path.rs:607`). Design documents: `22-images.md` and `25-svg.md`, read for the renderer's half only.
- Behaviour, images: a bitmap becomes one texture with every mip level the bitmap carries; a bitmap larger than `max_texture_dimension_2d` drops its finest levels, and one whose levels are all too large is decimated by `every_nth_pixel`, so something shows rather than nothing. The level is chosen as the smallest that is no smaller than the box on screen (the comment cites Chrome), and three bits of the sampler encode `pixelated` and the two repeat axes. A bitmap that is opaque, at full opacity, and unclipped goes through the unblended pipeline. The texture lives while an `Arc<Bitmap>` does (`pictures.retain(|_, p| p.bitmap.strong_count() > 0)`), and its pixels can be released once uploaded when this renderer is the only one drawing them.
- Behaviour, shapes: SVG arrives as `PathDraw` with an `Arc<PathGeometry>`. The renderer flattens the curves at 0.05 px, measures coverage into bands of 4 rows and blocks of 40 px, writes it into a store texture, and draws axis-aligned rectangles with one instance each, so a large fill costs almost nothing and a diagonal line costs its length. Fills and strokes with gradients resolve to a matrix and a focus that the shader evaluates. A frame that adds at least 48 shapes is rasterised on up to 8 threads.
- Quality notes: 27 pixel tests in `tests/images.rs`, 11 in `tests/svg.rs`, 12 unit tests in `path.rs` (areas of rectangles, circles and diagonals, holes, dashes, clips at an angle, fill rules), and 5 in `renderer.rs`. `path.rs` and `glyph_cache.rs` have no `unsafe`. Cost: a shape's coverage is rebuilt when its geometry or transform changes, and the store is cleared when it fills, at the price of rasterising that frame's shapes again.
- Label: Confirmed

### Tests that need a GPU, and what they do without one

- Status: present
- Evidence: `anti-webui-app/src/headless.rs` (`render_rgba` 12, `render_driven_rgba` 22, `render_png` 74), `Gpu::headless` (`wgpu/src/lib.rs:325`, one process-wide `OnceLock` instance and adapter, because the Vulkan loader crashed when instances came and went while tests ran side by side), and the skip pattern in every pixel test file, for example `tests/render.rs:6` and `:83` (`Err(e) => { eprintln!("skipping: {e}"); None }`).
- Behaviour: the 141 integration tests are pixel tests. They build a `Ui`, feed it events, paint, and compare bytes, and each takes a GPU adapter through `headless::render_rgba`, `headless::render_driven_rgba`, or `Gpu::headless()`; when there is none they print `skipping: …` and pass, or return early. One of them needs more: `tests/video_gpu.rs` (1 test, Linux only) also needs an FFmpeg this build knows and a DRM render node, and skips with a message otherwise. The 50 unit tests in `src/` (`pacing` 33, `vsync` 7, `perf_overlay` 7, `frame_stats` 2, and one settings test in `lib.rs`) and all 38 in `anti-webui-wgpu` are CPU-only and always run.
- Quality notes: the skip is a `return`, not `#[ignore]`, so a run without an adapter and a run with one both report success, and `--list` cannot tell them apart. That makes the renderer's only behavioural coverage conditional on the machine; A6 owns measuring it.
- Label: Confirmed

### Code against the design documents

Checked twice each, after reading the code and the sentence quoted.

| Where | The document says | The code does | Label |
|---|---|---|---|
| `05-rendering.md`, "グリフキャッシュ": 「アトラスはシェルフパッキング。満杯になったら拡張し、それでも足りなければクリアして次フレームで再構築する。」 | Rebuild on the next frame | `Renderer::prepare` clears the atlas and retries the same preparation, so those glyphs are rasterised again in the same frame (`renderer.rs:1539`–`1552`); the grow-or-stay decision is `mostly_passing`, which `17-font-axes.md` documents and `05` does not mention | Confirmed |
| `11-gpu-fill.md`, "残る課題": 「スクロールレイヤーの合成はグリフのパイプラインのまま。全画面の 1 枚を毎フレーム描くので、専用の経路にする余地がある。」 | Layer compositing still runs on the glyph pipeline | `vs_layer`/`fs_layer` is its own pipeline (`renderer.rs:875`, and `Batch::Layer` in `draw_batches`), which is what `12-transform.md`'s "レンダラ" says: 「合成は専用のパイプライン（`vs_layer` / `fs_layer`）で行う」. It does share the glyph bind-group layout, and it still draws one full-screen quad per layer | Confirmed |
| `14-shaders.md`, "やっていないこと": 「パイプラインキャッシュのディスク保存。」 | Compiled shaders are not saved to disk | `shader_cache.rs` saves a wgpu `PipelineCache` under `%LOCALAPPDATA%` on DX12, and the same document's "実行時の扱い" says 「DX12 では、コンパイルしたシェーダーをディスクに残し、次の起動ではコンパイルしない」 | Confirmed (the document contradicts itself; the code matches the second sentence) |
| `05-rendering.md`, "入力: Scene" lists three variants (`Shadow`, `Quad`, `Glyphs`) | Three kinds of primitive | Ten (`scene.rs:539`); the others are documented in `12`, `14`, `16`, `22`, `24`, `25`, and `41`, so this is an ageing snippet, not a rule broken | Confirmed (counts); Inferred (that the snippet was never meant to be exhaustive) |

What matched, having been compared and not listed above: the damage rules, the 8-rectangle cap, the buffer-age union, "nothing changed, present nothing", `Mailbox` then `Fifo`, the non-sRGB formats, Vulkan/Metal/DX12 before GL, the slicer's cases, the shadow hole and rim drawing, `k ≥ 1`, `k ≤ 4`, `k ≥ 2` when rotated, "never lower while the layer lives", the Dual Kawase level table and the σ ≤ 1.68 Gaussian, `reach` as the amount of damage expansion, the backdrop canvas, `MAX_INPUTS = 4`, the `content-visibility` rules, the canvas texture lifetime, the order of shadow and box, the clip rounding of an opaque background under an outer shadow (`paint.rs:1407`), and the render-ahead state machine's constants. Label: Confirmed.

## Baseline violations

These are what upstream is today; fork changes must not add to them.

- 125 `unsafe` occurrences in `anti-webui-wgpu/src` and 7 in `anti-webui-app/src`, 112 and 6 of them with a `// SAFETY:` comment within three lines. A7 records the baseline properly.
- No unit tests for layer planning, eviction, damage application, or the renderer's recording: those paths rest on the pixel tests, which skip without a GPU adapter.
- The wgpu-surface path (the fallback everywhere, and all of GL) ignores the damage and draws the full window, so the compositor learns only from a frame not being presented.
- A full glyph atlas discards every glyph, including those still on screen.
- `PRELOADS` in `core/src/shader.rs:233` is process-wide mutable state behind a `Mutex`, reached from every window's thread.
- The skip in the pixel tests is `eprintln!` plus `return`, so a machine without an adapter cannot be told from one with it in a test report.
- `anti-webui-app` does not link on a machine without `libxkbcommon-x11`, so its tests cannot even be listed there with the default features.

## Risks and technical debt

| Item | Impact | Likelihood | Evidence |
|---|---|---|---|
| Layer caches are dropped the frame a layer stops being painted, so hiding and showing a long list pays the full content draw again | Medium for tab-like UIs, and for `content-visibility` next to scroll layers | Medium | [Compositing layers and texture caches](#compositing-layers-and-texture-caches); the `retain` in `plan_layers` |
| No memory budget for layer, filter, effect-pool, or canvas textures; each is sized to the visible area times `k²`, up to 4 | High on large screens with many transformed rows, where the documents give 10–12 MB per screenful as normal | Medium | `create_layer_cache`, `prepare_output`, `effects.rs:15` |
| A full glyph atlas throws away glyphs still on screen and rasterises them again in the same frame | A visible hitch when many sizes or animated font axes are on screen | Low, unless axes animate | [Glyph atlas and subpixel positioning](#glyph-atlas-and-subpixel-positioning) |
| No LCD subpixel antialiasing, and vertical glyph positions rounded to whole pixels | Text looks unlike a browser's at 1× on RGB LCDs, and glyphs can shift half a pixel vertically | Certain (a design choice); low impact on HiDPI | `fs_glyph`, `glyph_cache.rs:164` |
| Gamma-space blending depends on choosing a non-sRGB format, and nothing tests it | A regression would look slightly different, never fail | Low | `configure_window` |
| The rendering tests pass with nothing drawn when no adapter is present, and this machine cannot link the X11 backend at all | CI and a laptop can both be green with the renderer unexercised | High in this environment | [Tests that need a GPU](#tests-that-need-a-gpu-and-what-they-do-without-one) |
| `content-visibility` and layer lifetime answer "keep the texture?" differently (freed for layers, kept for canvases) | Surprising memory behaviour when a skipped subtree returns | Medium | [Content visibility](#content-visibility) |
| Shader compilation is threaded, but `PRELOADS` is one global list shared by every window's thread | Benign now (behind a `Mutex`), but it couples windows | Low | `core/src/shader.rs:233` |
| `renderer.rs` is 3,491 lines in one `impl`, holding pipelines, planning, instancing, and recording | Hard to change safely, and the hardest file here for an agent to navigate | Certain | [Method](#method), file sizes |
| `11-gpu-fill.md` describes a pipeline split that no longer exists, and `14-shaders.md` denies a cache it has | A reader optimising compositing or start-up looks in the wrong place | Certain (two stale sentences) | [Code against the design documents](#code-against-the-design-documents) |

## Open questions

- Does scanning a layer's primitives per damaged rectangle cost anything visible with 200 layers? Answered by a `bench` example with `ANTIWEBUI_FRAME_STATS`, which this audit did not run (A6, A9).
- How much does releasing a layer's texture cost in a list that scrolls a layer in and out of view? Answered by an A9 measurement with `ANTIWEBUI_DEBUG_DAMAGE=1`, counting full layer redraws.
- Do the pixel tests pass on a machine with an adapter, and how long do they take there? Answered by A6 running them; this audit only listed them.
- Does compiling the X11 backend change the integration test count? Answered by installing `libxkbcommon-x11-0` and repeating the two `--list` commands.
- Is the buffer age ever wrong after a held frame is committed late? Answered by reading `present/dmabuf.rs`'s commit path against a compositor that releases out of order, or by a test that fakes the callbacks.
- Does `k` converge for a rotated layer, given that it never decreases while the layer lives and is capped at 4? Answered by a test that scales a node up and down repeatedly and counts `create_layer_cache` calls.
- Is the atlas rule of staying at 2048 px right for a UI that animates one `opsz` axis? Answered by animating one axis in an example and watching `atlas_size()`.

## Inputs for gap analysis

- [Checklist](#checklist): every capability A4b owns is present except LCD subpixel antialiasing; damage tracking and the glyph atlas are present with the limits listed.
- [Damage tracking](#damage-tracking): a requirement naming partial present or buffer age is a matter of the presenter, not the renderer, and on a wgpu surface the answer is "full redraw".
- [Compositing layers and texture caches](#compositing-layers-and-texture-caches): the layer is the unit of both caching and memory, and a memory budget does not exist today.
- [Code against the design documents](#code-against-the-design-documents): four statements in the rendering documents are stale, one of them contradicting its own file. A8 should carry these forward rather than re-derive them.
- [Tests that need a GPU](#tests-that-need-a-gpu-and-what-they-do-without-one): the pixel suite is the only coverage of the renderer, and it reports success on a machine that draws nothing. A Phase 2 claim about rendering needs a run on a machine with an adapter (A6).
- [Baseline violations](#baseline-violations): the `unsafe` numbers, the missing unit tests, and the fallback's full redraw are facts a fork requirement can raise but not fix without editing upstream-owned files.
