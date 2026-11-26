# Audit: core capabilities

- Date: 2026-11-25
- Upstream commit: `6908e3119db4ecf7080f57de59870e083f2cbc98`
- Auditor: Pi, DeepSeek V4.1 Flash (draft); reviewed by Claude Code, Claude Opus 5.5
- Plan item: [A4a](phase-1-audit-plan.md#workstreams)

## Scope

The two library crates everything else is built on: `src/anti-webui-core` (33,174 lines of source, 17,144 lines of integration tests) and `src/anti-webui-style` (2,261 lines). Covered capabilities: signals and reactivity, the node tree, the style system, the layout engine, text shaping, selection and editing and IME, coordinate transforms, input, and scrolling with settings. The upstream design documents read as the yardstick are `docs/design/01-pixel-model.md`, `02-reactivity-and-view.md`, `03-layout.md`, `04-text.md`, `08-scrolling-and-settings.md`, and `12-transform.md`.

Two things are counted for later use: how many of taffy's test fixtures exercise only layout features upstream has, and the state machines of focus, gestures, undo and redo, the reactive system, and the redraw scheduler.

Left out: the renderer, damage tracking, compositing, the glyph atlas, animation, GPU canvas, and accessibility (A4b); the OS backends, multi-window, and drag and drop to other programs (A4c); the full `unsafe`, Clippy, and formatting baseline (A7); the doc-by-doc comparison (A8). Core's `drag.rs`, `svg`, `video`, `images`, and `canvas` are mentioned only where they hold part of a state machine this file covers.

## Method

Reading order: the design documents above, then `reactive.rs`, `geometry.rs`, `layout.rs`, `tree.rs`, `view.rs`, `ui.rs`, `event.rs`, `platform.rs`, `scroll.rs`, `settings.rs`, `text/*`, and finally the style crate. Then the integration tests, and last the taffy fixtures.

| Question | Command |
|---|---|
| Size of each file | `wc -l src/anti-webui-core/src/*.rs src/anti-webui-core/src/text/*.rs src/anti-webui-style/src/*.rs` |
| Where a symbol is | `rg -n '^(pub )?(fn|struct|enum|impl|const|static)' src/anti-webui-core/src/<file>.rs` |
| Tests that run | `cargo test -p anti-webui-core -p anti-webui-style --locked -- --list`, then `rg -c ': test$'` |
| Doc tests | `cargo test -p anti-webui-core --doc --locked -- --list` |
| `#[test]` per file | `rg -c '#\[test\]' src/anti-webui-core/tests/*.rs` |
| `unsafe` count | `rg -n 'unsafe' src/anti-webui-core/src src/anti-webui-style/src`, then `rg -c 'SAFETY'` |
| Clock reads in core | `rg -n 'Instant::now|SystemTime|\\.elapsed\\(\\)' src/anti-webui-core/src` |
| Taffy tags and fixtures | `gh api repos/DioxusLabs/taffy/tags`, then a sparse clone of `v0.16.0` into `$TMPDIR` |
| Fixture classification | a Python script in `$TMPDIR/taffywork/classify.py` (below) |

Test count, from `--list`: 690 tests in the two crates. That is 651 `#[test]` functions (101 in `src/`, 550 in `tests/`), 24 doc tests, and 15 unit tests in `anti-webui-style`. One `#[test]` can stand for many cases: `tests/flex_wrap.rs::same_as_chromium` runs 82.

`unsafe`, for the numbers A7 will expand: 8 occurrences in `anti-webui-core` and none in `anti-webui-style`. Six have a `// SAFETY:` comment next to them; `MemoryPlane::new` (`video.rs:134`) is an `unsafe fn` documented with `# Safety` instead, and the `unsafe extern "system"` block at `text/mod.rs:271` has none, its call site at line 277 having one.

How the taffy fixtures were classified: a fixture counts as supported-only when no `style` attribute, no `<style>` rule, and no class in it uses a property or a value that `anti-webui-style` has no representation for. The property list is the field list of `style.rs`; the allowed values are the enums in `types.rs` (`Display`, `Position`, `Overflow`, `AlignItems`, `AlignSelf`, `AlignContent`, `JustifyContent`, `FlexWrap`, `BorderStyle`); anything else is unsupported, including `grid`, `float`, `order`, `direction`, `box-sizing: content-box`, `visibility`, `width: min-content | max-content | fit-content | stretch`, `calc()`, and `align-items: baseline`. The class `viewport` counts as unsupported because `scripts/gentest/test_base_style.css` gives it `width: max-content`. The base stylesheet's `div { display: flex; position: relative; box-sizing: border-box }` is what makes `border-box` and `ltr` free. The script also records whether a fixture has text in its body, since taffy shapes it in the Ahem font, which upstream does not have.

## Findings

### Checklist

| Capability from the [plan](phase-1-audit-plan.md#capability-checklist) | Status | Subsection |
|---|---|---|
| Fractional-scaling handling | present | [Layout engine](#layout-engine) |
| The integer physical-pixel model | present | [Layout engine](#layout-engine) |
| The layout engine | partial: flex and block, no grid | [Layout engine](#layout-engine) |
| Signal reactivity | present | [Signals and reactivity](#signals-and-reactivity) |
| The node tree | present | [Node tree](#node-tree) |
| The style system | present | [Style system](#style-system) |
| Text editing, selection, and IME | present | [Selection, editing, and IME](#selection-editing-and-ime) |
| Coordinate transforms and input | present | [Coordinate transforms](#coordinate-transforms), [Input](#input) |

Text shaping and scrolling with settings are also in this file. The renderer, damage tracking, compositing, the glyph atlas, animation, GPU canvas, accessibility, multi-window, and the OS backends are A4b and A4c. Labels: Confirmed, from the rows below.

### Signals and reactivity

- Status: present
- Evidence: `src/anti-webui-core/src/reactive.rs` (944 lines). `signal` (line 475), `memo` (571), `effect` (688), `batch` (365), `untrack` (374), `on_cleanup` (382), `create_root` (437), `Owner`, and `Binding` (612). The engine is `State::{Clean, Check, Dirty}` with `mark_observers` (154), `flush` (335), and `update_if_necessary` (300). Views subscribe by running a closure inside `effect`: dynamic styles at `view.rs:260`, `dyn_view` and `show` at `view.rs:1297`–`1316`, `text(move || …)` at `view.rs:1413`, sheets at `view.rs:514`, and `disabled` at `view.rs:541`. Design document: `02-reactivity-and-view.md`, sections "リアクティブ・プリミティブ" and "実装".
- Behaviour: the runtime is one thread-local `SlotMap` arena of reactive nodes; values live in `Rc<RefCell<Option<Box<dyn Any>>>>` and are borrowed only outside the runtime borrow, so a closure can write other signals while it runs. A write marks direct observers `Dirty` and everything further down `Check`, pushes effects into a `BinaryHeap` ordered by depth and then by marking order, and runs them shallowest first, because re-running a parent disposes its children. A `Check` node pulls its memo sources first and only re-runs if a value really changed, which is what keeps diamonds glitch-free (`reactive.rs`, test `glitch_free_diamond`). Dependencies are rebuilt on every run through a read cursor, so a source no longer read is unlinked and one read twice subscribes once. `batch` defers the flush to the outermost call. An effect that writes a signal it reads is caught by a 1,000,000-iteration guard with a message saying so.
- Quality notes: 10 unit tests in the module, covering rerun-on-change, memo equality, the diamond, batching, nested effect disposal, dependency churn, ordering, and dynamic dependencies. No `unsafe`, no threads, no `Mutex`: it is `Rc` and `RefCell` by design. Effects run synchronously inside the write that marked them; there is no scheduler queue that an event loop drains and no notion of time. A stale handle panics with "accessed a reactive value after its owner was disposed" (`reactive.rs:483`).
- Label: Confirmed

### Node tree

- Status: present
- Evidence: `src/anti-webui-core/src/tree.rs` (1,317 lines): `NodeId` (`slotmap` key), `Node`, `NodeKind`, `NodeStyle`, `Listeners`, `WindowState`, `Dirty`, `Document` (561), `Document::create` (828), `Document::remove` (1204), and `with_doc`. Builders and `IntoView` in `view.rs` (2,529 lines), `Ui` in `ui.rs`. Design document: `02-reactivity-and-view.md`, section "ノードツリー".
- Behaviour: a retained tree, not a virtual DOM. `NodeKind` has `Element`, `Text`, `Edit`, `Editor`, `Image`, `Video`, `Svg`, `SvgShape`, `SvgText`, `Drawing`, `Canvas`, `Fragment`, and `Popup`. Each window has one `Document` in thread-local storage, which is what lets an app run several windows on their own threads (`window.rs`). Nodes are created by builders through `create_node`, removed when their owner is disposed, and removing a node that had the focus, the selection, or the hover clears that. `Fragment` and `display: contents` generate no box. Nodes also carry the layout caches (`measure_cache`, `last_perform`), the paint inputs (`fill`, `overlay`, `filter`), the accessibility subtree, and the animation state (`transitions`, `presence`, `moving`), so the tree is the one place where per-node state lives.
- Quality notes: no unit tests in `tree.rs`, `view.rs`, `ui.rs`, or `event.rs`; everything goes through the integration tests, 22 of the 31 files in `tests/` reaching into the document with `with_doc`. Allocation: children are a `Vec<NodeId>`, listeners are `Rc<dyn Fn>`, styles are boxed in the state layers that are usually absent. No `unsafe`.
- Label: Confirmed

### Style system

- Status: present
- Evidence: `src/anti-webui-style`: `sheet.rs` (`Sheet`, `AccentParts`), `style.rs` (`Style` and `ComputedStyle`, generated by one `define_style!` macro), `styled.rs` (`Styled` with `setter!` and `flag!` builder methods), `transition.rs` (`Transition`, `Easing`, `Prop`, `AnimatedValue`), `types.rs` (923 lines of enums), `color.rs`. Resolution lives in core: `apply_sheet` (`layout.rs:425`), `resolve_node` (`layout.rs:484`), `resolve_styles` (308), `apply_transitions` (649). Design documents: `02-reactivity-and-view.md` ("ビュー構築") and `13-animation.md` for transitions.
- Behaviour: `Style` is a refinement whose every field is `Option`, and `ComputedStyle` is the resolved value. 109 properties, 35 of them inherited or merged. A node applies its layers in this order: the sheet (`base`, then `hover`, `focus`, `focus_visible`, `active`, `disabled`), its own base style, its dynamic styles in order, then `hover`, `focus`, `active`, `dragging`, `drag_over`, and the presence exit style. Inheritance is copied from the parent's computed style first, so a property unset here comes from above. All lengths are logical `f32`; the conversion to physical `i32` happens once in `Resolved::from_computed` (`layout.rs:71`), after the scale is known. Transitions and presence are applied while resolving, which is why `apply_transitions` can put a value that is neither the old nor the new one into the computed style.
- Quality notes: 15 unit tests (9 in `types.rs` for axes and merge, 6 in `transition.rs` for easing and interpolation), plus `tests/sheets.rs` (13 tests, including `what_an_element_is_given_goes_above_its_sheet_in_every_state` and `the_focus_is_only_shown_when_it_came_from_the_keyboard`) and `tests/transition.rs` (22 tests). No CSS text parser: styles are built in Rust only. `Style` is large, which `04-text.md` records as a debug-build stack cost. No `unsafe`.
- Label: Confirmed

### Layout engine

- Status: partial. Flexbox and block exist; grid, float, and `order` are absent.
- Evidence: `src/anti-webui-core/src/layout.rs` (2,844 lines): `RLen`, `Resolved`, `distribute` (170), `MeasureCache`, `same_layout`, `resolve_styles` (308), `build_stacking` (775), `layout_document` (925), `position_document` (996), `Engine` (1296), `Engine::compute` (1422), `compute_inner` (1463), `block` (2054), `flex` (2103). `geometry.rs`: `Scale` (5), `Scale::px` (23), `px_min1` (28), `pxf` (37), `logical_ceil` (42), `logical_floor` (47), `to_logical` (53), `Affine`, `RectI`. `types.rs`: `Display` (268) with `Block`, `Flex`, `None`, `Contents`; `Position` with `Static`, `Relative`, `Absolute`; `Overflow` with `Visible`, `Hidden`, `Scroll`; `FlexWrap`; `AlignItems`; `AlignContent`; `JustifyContent`. Design documents: `01-pixel-model.md` and `03-layout.md`.
- Behaviour, layout modes: `display: flex` with `row`, `column`, and their reverses, `nowrap`, `wrap`, and `wrap-reverse`, `justify-content` and `align-content` including `space-evenly`, `align-items`, `align-self`, `gap`, `grow`, `shrink`, and `basis`; `display: block` as vertical stacking with the container width; `display: none`; `display: contents`; `position: relative` and `absolute`, the latter with `inset` against the parent's padding box, taking its width from `left` and `right` when both are set and otherwise from its own size (`Engine::layout_absolute`, `layout.rs:2660`), and with pictures sized by the CSS 2.1 10.3.2, 10.6.2, and 10.4 rules (`replaced_size`, `layout.rs:92`); `overflow: visible | hidden | scroll`; `z-index` stacking, applied only where CSS applies it (`resolve_node`, after the resolved style). Intrinsic sizing recurses through children and text (`min-content` and `max-content` from parley's `calculate_content_widths`), and the result is cached per node.
- Behaviour, the integer model: every length becomes `i32` before layout, by the one rule `(logical as f64 * n / 120.0).round()` with `Scale` held as `n/120` — the same representation as `wp_fractional_scale_v1` (`geometry.rs:5`). Non-zero borders never collapse to zero (`px_min1`, rule R2). Percentages resolve against the containing block with `.round()` (`RLen::resolve`). Room left over is shared out inside the algorithm, not snapped afterwards: `distribute` splits `total` with `q = total / n` and `r = total % n` when the weights are equal, so widening a container only ever adds a pixel to one more child, and uses Sainte-Laguë for unequal weights so that growing the total never shrinks anyone (`distribute_equal_monotone`, `distribute_weighted_sums_and_monotone`). That is method C in `01-pixel-model.md`, and the same document's worked example at scale 1.5 is `tests/layout.rs::same_spec_same_size_at_fractional_scale`, which expects widths `50, 50, 50` and 2 px borders.
- Behaviour, fractional scaling: the scale arrives with `WindowEvent::Resized { size, scale }`, where `size` is the buffer the platform actually allocated, so the root's constraint is that buffer rather than a product core computed (R3). Popups are the exception: their surface size is `round(logical × scale)` computed in core (`layout.rs:1061`). A scale change calls `mark_style_all()` and `invalidate_all_layout()` (`ui.rs:944`–`948`), so every resolved style, measurement, and placement is redone. Text is shaped at the physical size (`scale.pxf(font_size)`, `text/mod.rs:576`) with line boxes quantised, while glyph x positions keep sub-pixel precision (R4). Paint-only quantities — radii, blur, font size, transforms — stay `f32` (`pxf`, R5). Coordinates handed back to the OS go through `to_logical` (R7).
- Behaviour, difference from a full pass: a clean subtree laid out with the same inputs returns its previous result (`Engine::compute`, `last_perform`), measurements are cached per `(known, avail_w, base)`, and only a style, text, or child change marks the path to the root dirty. `tests/layout.rs::incremental_layout_matches_full_layout` and `tests/incremental.rs::random_changes_match_a_full_update` compare that against dropping every cache.
- Quality notes: the flex engine has no unit tests of its own. The unit tests in `layout.rs` cover `replaced_size` against the CSS 2.1 10.4 table and `distribute`; the engine is tested through `tests/layout.rs` (10 tests), `tests/flex_wrap.rs` (1 test over 82 Chromium-compared cases, below), and the random differential test. `display: block` has no oracle to compare against, and neither percentage resolution nor absolute positioning does. No `unsafe`. Allocation: `Resolved` is cloned per child per pass (`self.doc.nodes[c].resolved.clone()`), which is the largest obvious per-node cost in the engine.
- Against `03-layout.md`: the code drops every cache on a scale change (`ui.rs:944`), as the document says ("スケール変更時はすべてのキャッシュを破棄する"). The document says hit-testing floors the pointer's `f32` coordinate; `RectI::contains` (`geometry.rs:316`) compares the `f32` point with integer edges using `>=` and `<`, which gives the same answer as flooring first. No divergence found. Label: Confirmed (reviewer check)
- Label: Confirmed

### Text shaping

- Status: present
- Evidence: `src/anti-webui-core/src/text/mod.rs` (1,103 lines): `TextSystem` with `fcx: FontContext` and `lcx: LayoutContext<Brush>`, `TextLayout::shape` (668) and `shape_rich` (680) with a `ShapeKey { version, fonts, style, rich }`, `prepare_fallbacks` (147), `prepare_fallback` (164), `variations` (236) for `wght` and `opsz`, `register_font` (332) and `register_font_as` (339), `STAND_INS` (39), `user_locale` (269, 283), `prepare_system_fonts`, `baselines_of` (481), `average_letter` (515). `text/rich.rs` for spans and inline boxes. Design document: `04-text.md`.
- Behaviour: parley 0.12 (`Cargo.toml:59`, feature `complex-scripts`) does shaping, bidi reordering, line breaking, and cursor movement; fontique does font discovery, reading fontconfig on Linux, DirectWrite on Windows, and CoreText on macOS. The locale comes from `LC_ALL`, `LC_CTYPE`, or `LANG`, else `GetUserDefaultLocaleName` on Windows (`text/mod.rs:269`), and is handed to parley. `prepare_fallbacks` asks fontique once per script before shaping, which `04-text.md` records as the fix for fontique re-querying fontconfig on every shape (176 µs to 10 µs on the machine it was measured on; the figure itself is upstream's). Metric stand-ins are appended to eight common family names so that "Arial" resolves to Liberation Sans rather than to the default sans (`STAND_INS`). Variable fonts take arbitrary axes through `FontAxes`; an axis value past the font's range is clamped (`tests/font_axes.rs::a_value_past_the_end_of_an_axis_is_the_end`), an axis the font lacks changes nothing, and `opsz` follows the font size unless set. Embedded fonts are registered into fontique and take the same shaping path; the cache key includes a font generation counter, so text shown before the font arrived is shaped again.
- Quality notes: 4 unit tests in the module, `tests/fonts.rs` 11, `tests/font_axes.rs` 24. The system font collection is built once per thread behind a `OnceLock` and warmed on another thread at start (`prepare_system_fonts`), which means the first text on a cold thread still waits for it. Shaping caches key on the text version plus the style, so a colour change does not reshape (the brush is a span index). One `unsafe` item there (an `extern` declaration and its call).
- Against `04-text.md`: the locale is read from `LC_ALL`, `LC_CTYPE`, then `LANG`, then the OS locale; the document names `LANG` "など" (and others), so this is detail, not a divergence.
- Label: Confirmed

### Selection, editing, and IME

- Status: present
- Evidence: `text/selection.rs` (308 lines): `TextPoint`, `DocSelection`, `Granularity`, `compare`, `selectable_text_nodes`, `node_range`, `hit_text_point`, `nearest_text`. `text/edit.rs` (875 lines): `EditData`, `Snapshot` (40), `EditKind` (47), `Preedit`, `snapshot` (422), `undo` (573), `redo` (592), `apply_ime` (679), `ime_state` (729). `text/editor.rs` (3,060 lines, the row-based `text_editor`): `Change`, `Step`, `UNDO_STEPS`, `step` (1008), `undo` (1030), `redo` (1034), `clear_history` (619). `event.rs` `ImeUpdate`; `platform.rs` `ImeState`, `ImePurpose`, `PlatformRequest::SetIme`; `ui.rs` `ime` (2276) and `sync_ime` (2326). Design document: `04-text.md`, sections "選択モデル", "編集モデル", "IME".
- Behaviour: one selection for the whole document, ordered by `doc_order`, which the position pass assigns in tree order; only visible nodes with `user_select: text` are selectable, and the property is inherited. A `TextEdit` keeps its own selection, as a browser's `<input>` does, and so does a `text_editor` row. Dragging selects the nearest text position: first the line box under the pointer, else the text node closest by vertical distance and then horizontal distance, with the pointer clamped into that node. Double click selects a word and triple click a paragraph, both from the platform's event time (`DOUBLE_CLICK_MS = 500`, `ui.rs:31`). `TextEdit` covers arrows, words, Home/End, lines with a remembered horizontal position, Backspace/Delete, Ctrl+A/C/X/V/Z, Ctrl+Shift+Z and Ctrl+Y for redo, Enter, mouse selection, middle-click paste, password masking, and two-way binding to a `Signal<String>`. IME state goes out through `PlatformRequest::SetIme` only when it changes, and only after layout has settled (`sync_ime` returns early while layout or positioning is dirty). Preedit text is inserted at the caret for layout, drawn underlined, and `ImeUpdate` is applied in the documented order: remove the old preedit, delete the surrounding text, insert the commit, then set the new preedit.
- Quality notes: `edit.rs` 10 unit tests, `editor.rs` 7, plus `tests/interaction.rs` 11 (including `ime_preedit_and_commit`, `japanese_double_click_selects_a_word`, and the password test), `tests/text_editor.rs` 13, `tests/text_editor_rows.rs` 9, `tests/text_drag.rs` 42, and `tests/rich_text.rs` 8. No `unsafe`. Memory: `EditData` undo keeps up to 200 `Snapshot`s, each cloning the whole string; `editor.rs` keeps up to 1,000 `Step`s of small change records, which is much cheaper on a large document. Undo coalescing is by edit kind only (Insert, Delete) and has no time window, so typing after a pause still joins the previous step.
- Label: Confirmed

### Coordinate transforms

- Status: present
- Evidence: `12-transform.md`; `geometry.rs` `Affine` (`f64`, `then_apply`, `apply`, `inverse`, `is_translation`, `is_whole_translation`, `is_axis_aligned`, `max_scale`, `bounds_of`); `tree.rs` `Node::layer_matrix` (486), `bounds_in_parent` (498), `Document::step_out` (895), `to_window` (911), `rect_to_window` (923), `from_window` (938), `from_window_space` (943), `window_rect` (952), `visible_part`, `can_be_seen`; `ui.rs` `hit` (2464) and `hit_stacked` (2524); `layout.rs` `same_layout` (270), `mark_outer_layers_stale` (361), `same_content` (385). Tests: `tests/transform.rs`.
- Behaviour: `translate`, `scale`, `rotate`, and `transform_origin` are paint-time only, so they never run layout. A node with a non-identity transform becomes the origin of a coordinate space: its descendants, itself included, are in that space (`Node::space == Some(T)`), the clips from outside it are kept in `layer_clip` in the parent's coordinates, and `outer_space` links outwards. Window coordinates are produced by walking `step_out` outwards, composing translations for scroll roots and `layer_matrix` for transformed nodes; `RectI` becomes the bounding box of the transformed corners when the transform is not a whole translation, and is translated exactly when it is. Hit-testing inverts the matrix at the transformed node and carries on into its children, so `hit()` returns early when `inverse()` is `None` (a scale of 0). A pixel-exact test covers this: `tests/transform.rs::the_pointer_finds_what_is_shown_where_it_is_shown` and `something_flattened_away_is_not_there`.
- Quality notes: 20 tests in `tests/transform.rs`, one unit test in `geometry.rs` (rounding is symmetric). `Affine` is `f64` on purpose, so a transform deep in a long list stays exact. The compositing half of this capability — painting a transformed subtree into a texture, the resolution `k`, bilinear sampling, settling to whole pixels — is in the renderer and belongs to A4b; core only decides when a subtree is a layer (`stale_layers`, `transform_moved`).
- Label: Confirmed

### Input

- Status: present
- Evidence: `event.rs` (434 lines): `WindowEvent` (22 variants), `PointerEvent` with `position`, `local`, `rect`, `button`, `modifiers`, `click_count`, `KeyboardEvent` with `prevent_default`, `ScrollEvent`, `PinchEvent`, `Key`, `NamedKey`, `Modifiers`, `ScrollPhase`, `PinchPhase`, `DragAction`, `DragActions`, `DragResult`, `ClipboardKind`. `ui.rs`: `handle_event` (814), `hit_test` (963), `set_hovered` (991), `pointer_moved` (1064), `Press`, `Drag`, `pointer_up`, `set_focus` (1680), `apply_pending_focus` (1737), `focus_next` (1749), `cancel_press` (1391), `key`, `ime` (2276). `drag.rs`: `Gesture` (493), `GestureConfig` (397), `DropConfig`, `EdgeScroll` (547), `DragSession`, `gesture_pressed` (746), `gesture_moved` (768), `gesture_released` (797), `cancel_gesture_for_focus`. `view.rs`: `on_click`, `on_aux_click` (620), `on_pointer_down`, `on_key_down`, `on_scroll`, `on_pinch`, `on_focus_change`, `on_focus_visible`, `on_hover`. Design document: `02-reactivity-and-view.md`, section "イベント".
- Behaviour: one enum in, dispatched by `handle_event`. Pointer presses hit-test the paint order, remembering the innermost node; moves set the hover chain from innermost outward, and a release still goes to the pressed node (implicit pointer capture), while a click goes to the deepest common ancestor of press and release. `click` is the primary button's, and other buttons go to `auxclick`. Keyboard events go to the focused node, else to the root, and bubble; `prevent_default` stops built-in editing and focus navigation. Tab and Shift-Tab move through focusable, visible, enabled nodes in preorder, scoped to a modal popup when one is open and skipping what is behind a fullscreen element. Escape cancels an in-progress grab or drag before anything else sees it.
- Behaviour, gestures: there is a press state machine (`Gesture`) but no tap, long-press, or double-tap recognizer. A press on a node with a grab or drag handler waits until the pointer has moved past the handler's threshold, then sends `GrabPhase::{Start, Move, End, Cancel}` with a velocity from the last three samples; a press on text starts a selection drag; on a scrollbar thumb, a scrollbar drag; on selected text, a move drag. Gesture state is `started`, `moved`, `cancelled`, `drags`, and `shows_grabbing`, plus `EdgeScroll` for autoscroll near the window edge. Nothing waits on a timer for a second click or a long press; the only press timing is `DOUBLE_CLICK_MS` for the click counter, which uses the platform's event time.
- Quality notes: `tests/interaction.rs` 11, `tests/drag.rs` 53, `tests/text_drag.rs` 42. No `unsafe`. The whole input path is `&mut self` and allocation-light, except that every press builds the ancestor chain and every move compares rects. Platform coverage is per backend and is A4c; core sees only `WindowEvent`.
- Label: Confirmed

### Scrolling and settings

- Status: present
- Evidence: `scroll.rs` (521 lines): `VelocityTracker` (3 samples, least squares), `Fling` (position as a function of time, `ease` for the cubic bezier), `Booster`, and the constants `LIFT_TIMEOUT = 200`, `MIN_SPEED = 30`, `MAX_DURATION = 2`, `BOOST_TIMEOUT = 50`, `FLING_SPEED_RATIO = 0.4`. `ui.rs`: `ScrollGesture` (60), `gesture_chain` (1467), `begin_scroll_gesture` (1525), `interrupt_scroll_gesture` (1577), `end_scroll_gesture` (1585), `deliver_scroll` (1594), `scroll_rest`, `fling`, `booster`, and `scroll_edge`. `settings.rs` (187 lines): `Settings` with `#[non_exhaustive]` `ScrollSettings`, `KineticSettings`, `RenderingSettings`, `ImageSettings`, `DebugSettings`; `settings()` and `update_settings()`; `broadcast` across UI threads. Design document: `08-scrolling-and-settings.md`.
- Behaviour: a scroll gesture is latched to the innermost scroller that can move in the direction of the first motion, and everything after it, inertia included, goes to the same nodes however the content moves; a `Single`-phase wheel scroll is matched per event instead. Fractions of a pixel are carried per scroller (`scroll_rest`) and dropped at the ends, so slow motion still moves. Where the platform leaves inertia to the application, `Fling` computes the position from the elapsed time rather than from the last frame, so dropped frames do not change the path, and a second fling in the same direction adds to the first under the conditions in the document. Settings are one thread-local `Signal<Settings>` in a root of its own, so reading them inside an effect subscribes to changes, and `update_settings` on any UI thread broadcasts to the others through `channel::ui_channel`. The units match the document: wheel notches times `wheel_step`, touchpad deltas times `touchpad_factor`, everything else as given.
- Quality notes: 16 unit tests in `scroll.rs`, including `matches_chromium` and `the_path_does_not_depend_on_the_frame_rate`; `tests/kinetic.rs` has 47, covering latching, fractions, start and stop conditions, settings, handlers, OS-driven momentum, and nested scrollers. `settings.rs` has no tests of its own; its broadcast path is exercised only through multi-window code. Not implemented, as `08-scrolling-and-settings.md` says: overscroll bounce, per-element overrides. The macOS and Windows backends pass the OS's inertia through instead of computing it.
- Where it differs from `08-scrolling-and-settings.md`: the document describes `Held` as never reaching handlers and `MomentumEnded` as always arriving after an interrupted fling; this audit did not trace every phase through `deliver_scroll`, so treat those two as Unverified.
- Label: Confirmed (except the row above)

### Taffy test fixtures

- Status: present as a counted source, not yet used
- Evidence: `DioxusLabs/taffy` tag `v0.16.0`; `test_fixtures/` has 1,691 HTML files; the classification script described in [Method](#method). Design document for the plan: [candidate test oracles](candidate-test-oracles-and-model-checking.md#part-1-taffy-as-a-differential-layout-oracle).
- Counts, by the folder taffy puts them in:

| Prefix | Fixtures | Supported only | of those, no text | of those, with text |
|---|---|---|---|---|
| `flex` | 700 | 569 | 522 | 47 |
| `grid` | 595 | 2 | 2 | 0 |
| `block` | 281 | 216 | 208 | 8 |
| `float` | 54 | 0 | 0 | 0 |
| `leaf` | 15 | 14 | 6 | 8 |
| `blockflex` | 13 | 12 | 11 | 1 |
| `blockgrid` | 14 | 0 | 0 | 0 |
| `gridflex` | 11 | 0 | 0 | 0 |
| `contain` | 8 | 0 | 0 | 0 |
| All | 1,691 | 813 (48%) | 749 | 64 |

- The top reasons a fixture is unsupported, counted once per fixture: `display: grid` 628, `grid-template-columns` 520, `grid-template-rows` 465, `grid-column` 295, `grid-row` 261, `align-items: baseline` 73, `float` 63, `direction` 51, `justify-self` 38, `flex-wrap: balance` 30, `grid-auto-columns` 24, `height: stretch` 19, `box-sizing` 18, `width: max-content` on `.viewport` 16, `flex-line-count` 14, `display: flow-root` 12. The two `grid` fixtures that pass are `grid_aspect_ratio_absolute_fill_width_from_inset.html` and `grid_aspect_ratio_absolute_height_overrides_inset.html`, which only use flex and absolute positioning.
- Behaviour: taffy writes the expected rectangles twice. One copy is a Rust test per fixture, generated from a browser run; the other is `tests/xml/<prefix>/<fixture>__<variant>.xml`, where `<expectations>` holds a `node` tree with `x`, `y`, `width`, and `height`. Each fixture has four variants: `border_box_ltr`, `border_box_rtl`, `content_box_ltr`, and `content_box_rtl`, 6,696 files in all. Upstream's model is `border-box` and left-to-right, so `border_box_ltr` is the variant that matches what upstream can express, and 1,674 fixtures have one.
- Reading for the differential test: 749 of the 813 usable fixtures have no text, so they are comparable without a font that upstream does not ship; the 64 with text need taffy's Ahem font registered through `register_font` for the measurement to match, since `04-text.md`'s text model reads the same font. The support limits, not the fixture count, are what matters: 52% of the fixtures cannot be used until grid, float, `baseline` alignment, and `direction` exist.
- How upstream tests layout today: `tests/flex_wrap.rs` parses `tests/data/flex_wrap.txt` (82 cases, 779 lines) into real `Div` builders and compares every node rectangle with a recorded Chromium result, allowing 1 px where Chromium's own value is fractional. `tools/chrome_layout.py` is what produced those values: it renders all cases in one page with `box-sizing: border-box`, runs `chromium --headless=new --dump-dom`, reads the `<pre id="out">` element, and rewrites the `=` lines. The oracle is a recorded run, so the test itself needs no browser. `tests/layout.rs` (10 tests) checks the integer model, flex grow, block and absolute layout, text measurement, and that a full pass equals an incremental one by hand. `tests/incremental.rs` (6 tests; `random_changes_match_a_full_update` at line 582 drives random trees and random events and compares against a full rebuild). Nothing compares block, percentage, or absolute layout against another engine.
- Label: Confirmed (the script's counts, rerun by the reviewer with the same result, and the file layout); Inferred ("supported": it rests on the script's mapping from CSS properties to `anti-webui-style` fields, which no layout run has tested; and that the 64 text fixtures would match with the Ahem font — not tried). The script was kept only in a scratch folder; Method gives its rules.

### State machines

Each table gives the states, the inputs, and how time and the platform reach the machine. Time reaching it directly means a `std::time::Instant` read inside the machine.

Focus, `ui.rs`: `set_focus` (1680), `apply_pending_focus` (1737), `focus_next` (1749), `WindowState::focused` and `focus_visible` (`tree.rs:550`, 552), `Node::focusable` (`tree.rs:208`).

| Item | Content |
|---|---|
| States | No focused node; focused node with `focus_visible = false`; focused node with `focus_visible = true`; focus pending on something behind a fullscreen element (dropped); focus pending through a `NodeRef` (`Document::pending_focus`) |
| Inputs | Tab or Shift-Tab, a press on a focusable node, `NodeRef::focus`, a focused node removed or disabled, a fullscreen element entering or leaving, `WindowEvent::Focused` |
| Order | Preorder over focusable, visible, not-disabled nodes, scoped to a modal popup, skipping what is behind a fullscreen element; wraps around |
| Time | Reads `Instant::now()` to reset `caret_epoch` on the edit it focuses (1705, 1712). Nothing else is timed |
| Platform | Focus of the window arrives as `WindowEvent::Focused`; the IME is told through `PlatformRequest::SetIme` |
| Label | Confirmed |

Gestures, `ui.rs` and `drag.rs`. There is no tap, long-press, or double-tap recognizer; the machine is a press that may become a grab or a drag.

| Item | Content |
|---|---|
| States | `Press { target, button, drag: Drag, gesture }` with `Drag` in `None`, `Select`, `Edit`, `Editor`, `Scrollbar`, `SelectedText`; a `Gesture` in pressed, started (after the threshold), moved, cancelled; `DragSession` in `Inside`, `Platform`, `External`; `EdgeScroll` near a window edge |
| Inputs | Pointer down, move, up; the modifiers; the platform's drag events (`DragEntered`, `DragMoved`, `DragLeft`, `DragDropped`, `DropData`, `DragSourceEnded`); Escape; the window losing focus |
| Transitions | Down: hit-test, pick a `Drag` kind, start a `Gesture` if the node has one, dispatch `pointer_down`. Move past the threshold: `GrabPhase::Start` or hand the drag to the platform. Move again: `GrabPhase::Move`, and autoscroll when near an edge. Up: `GrabPhase::End`, then a click or `auxclick` to the deepest common ancestor. Escape or lost focus: `Cancel`, and the click is suppressed |
| Time | `Gesture::began` is `Instant::now()` at the press (732) and the velocity is `began.elapsed()` (1644); `EdgeScroll` uses the `Instant` passed to `Ui::animate`; the click counter uses the platform's `time: u32` in milliseconds (`DOUBLE_CLICK_MS`) |
| Platform | Drags that leave the window become platform requests and come back as `WindowEvent::Drag*`; the platform supplies pointer coordinates in physical pixels |
| Label | Absent for tap, long-press, and double-tap; Confirmed present for the press-to-grab machine |

Undo and redo, `text/edit.rs` for `TextEdit` and `text/editor.rs` for `text_editor`.

| Item | Content |
|---|---|
| States | A stack of undo steps and a stack of redo steps. `EditData` keeps `undo: Vec<Snapshot>` of whole text plus selection, capped at 200; `EditorData` keeps `Vec<Step>` of `Change { at, removed, inserted }` with before and after selections, capped at `UNDO_STEPS = 1000` |
| Inputs | Insert, delete, replace, `set_text` from a bound signal, IME commit and surrounding delete, drop, a selection move in an editor (`keep_selection_after`), Ctrl+Z, Ctrl+Shift+Z, Ctrl+Y |
| Transitions | Each edit calls `snapshot(kind)` first, which returns early if the last edit was the same kind, so a run of typing is one step; undo pops to redo and restores text and selection; a new edit clears redo |
| Time | None. `last_edit` is an edit kind, not a timestamp, so there is no coalescing window |
| Platform | None: the shortcuts arrive as `KeyEvent`s and the clipboard as `WindowEvent::Paste` |
| Label | Confirmed |

The reactive system, `reactive.rs`.

| Item | Content |
|---|---|
| States | Per node: `Clean`, `Check`, `Dirty`, and a `Kind` of `Signal`, `Memo`, `Effect`, `Root`. Runtime: `batch_depth`, `flushing`, and a `pending` heap of effects |
| Inputs | `Signal::set`, `update`, `notify`; `Memo` re-evaluation; `effect` creation; `Owner::dispose`; `batch`, `untrack` |
| Transitions | Write: direct observers become `Dirty`, further ones `Check`, effects are pushed to the heap. `flush`: pop the shallowest, effect first; a `Check` node updates its memo sources, becomes `Dirty` only if one changed, and re-runs then. A re-run clears owned children and cleanups first and rebuilds its sources |
| Time | None. Effects run inside the write that marked them, or at the end of the outermost `batch` |
| Platform | None. The runtime is thread-local with no OS handle, which is what lets a window on another thread have its own |
| Label | Confirmed |

The redraw scheduler. Core decides, the app crate's runner asks the platform.

| Item | Content |
|---|---|
| States | `Document::dirty { style, layout, position, paint, content, carets }`, `stacking_dirty`, `pending_scroll`, `pending_scroll_to`, `reposition`, `settling`, `transitioning`, `stale_layers`; plus `Ui::fling`, `Ui::edge`, `pinching`, `press`, `drag`; plus `animation::frame_requested()` |
| Inputs | Any `WindowEvent`, a signal write that reaches an effect marking the document dirty, `animation::request_frame`, image and video results, `Ui::tick` |
| Transitions | `Ui::update` resolves styles, lays out, positions, builds stacking, and scrolls carets as the dirty flags say. `Ui::needs_redraw` is `animating() || dirty.paint || !scrolled.is_empty() || needs_update(d)` (499). `animating` is `fling || edge || frame_requested() || settling || transitioning || images.has_news()` (506). `Ui::next_deadline` (760) takes the earliest of the caret blink, the image frame, the video deadline, and the animation timers |
| Time | `animate(now)`, `paint(now)`, `tick(now)`, and `next_deadline(now)` take the time as a parameter, so the frame is deterministic. Inside, `Instant::now()` is still read for transition starts (`layout.rs:672`), caret epochs (`ui.rs:906`, 911, 1705, 1712), autoscroll, and animation starts (`animation.rs:102`, 259) |
| Platform | `anti-webui-app`'s `Runner::process` calls `needs_redraw()` after every event and requests a redraw (`src/anti-webui-app/src/runner.rs:122`); `Runner` answers the backend's `deadline` with `next_deadline` and its `timer` with `tick` (`runner.rs:376`–`402`). Core itself never asks the platform for a frame |
| Label | Confirmed |

## Baseline violations

These are what upstream is today; fork changes must not add to them.

- 8 `unsafe` occurrences in `anti-webui-core`, 0 in `anti-webui-style`; 6 `// SAFETY:` comments, 1 `unsafe fn` with a `# Safety` doc, and 1 `unsafe extern` declaration without one. A7 records the full baseline.
- No unit tests in `ui.rs`, `view.rs`, `tree.rs`, `event.rs`, `settings.rs`, or `text/selection.rs`. The flex engine has no unit tests of its own either.
- `display: block`, percentage resolution, and absolute positioning have no external oracle: only flex wrapping is compared with Chromium.
- The reactive runtime panics when a handle outlives its owner, and the effect-settling guard panics (`assert!` in `flush`) after 1,000,000 iterations. Both are deliberate, and both are reachable from user code.

## Risks and technical debt

| Item | Impact | Likelihood | Evidence |
|---|---|---|---|
| No clock seam: `Instant::now()` is read inside transitions, caret epochs, autoscroll, and animation starts, so a headless or recorded run cannot control time | Medium. The candidate oracle plan wants state-machine tests for the scheduler and gestures | High | [State machines](#state-machines); `layout.rs:672`, `ui.rs:906`, `drag.rs:1644`, `animation.rs:102` |
| The flex engine is checked against one recorded oracle and a random self-consistency test, so a wrong intrinsic size can agree with itself indefinitely | High for layout correctness | Medium | [Layout engine](#layout-engine); `tests/incremental.rs::random_changes_match_a_full_update` compares the engine with itself |
| Grid, float, `order`, `baseline` alignment, and `direction` are absent, which blocks 52% of taffy's fixtures and any CSS-shaped requirement | High for Phase 2 scope | Certain | [Taffy test fixtures](#taffy-test-fixtures) |
| Text measurement differs from a browser unless the same font is registered, so any differential layout test with text needs a font fixture first | Medium | High | [Taffy test fixtures](#taffy-test-fixtures); 254 of 1,691 fixtures have text |
| `EditData` undo stores whole strings, up to 200 of them | Low for a single input; high for a large text area | Low | [Selection, editing, and IME](#selection-editing-and-ime) |
| The style system has no text parser, so a stylesheet or a CSS-shaped requirement has no path in | Medium | Medium, if Phase 2 wants CSS syntax | [Style system](#style-system) |
| `Document` is thread-local and one per window, so the core is single-threaded by construction and every cross-thread path is explicit (`channel`, `shared`, `window`) | Low now; shapes any future multi-threaded renderer | Certain | `tree.rs`, `window.rs`, `channel.rs` |
| `Resolved` is cloned per child per layout pass | Low at current tree sizes, unknown at 2,000-row scale | Medium | `layout.rs:2103` onwards |
| Settings are one `Signal<Settings>`, so every reader re-runs on any settings change | Low; changes are rare | Low | `settings.rs` |

## Open questions

- Does the effect-settling guard ever fire in a real app, and what happens if it does? Answered by running the examples with a hook that logs the assert, or by turning the guard into a counter in a fork-owned build.
- Is the order of `hit()` in a transformed subtree with overlapping layers the same as the paint order? The tests cover a few cases; a randomised test would settle it. Answered by extending `tests/incremental.rs` to compare `hit` against a brute-force paint-order walk, which it already does for untransformed nodes.
- How much of the 1 px tolerance in `tests/flex_wrap.rs` hides real errors? Answered by lowering the tolerance per case and listing what fails.
- Does `prepare_fallbacks` still carry its cost saving on this machine? Answered by a benchmark report; the numbers in `04-text.md` are upstream's.
- Are `Held` and `MomentumEnded` delivered exactly as `08-scrolling-and-settings.md` says? Answered by reading `deliver_scroll` phase by phase, or by a test that records the phases a handler sees.
- Would the Ahem font make the 64 text-bearing taffy fixtures match? Answered by registering Ahem through `register_font` in a Phase 4 spike and comparing.

## Inputs for gap analysis

- [Checklist](#checklist): every capability A4a is responsible for is present; the layout engine is partial, and `display: block` and absolute positioning have no oracle.
- [Layout engine](#layout-engine): flex and block only, so any requirement that names grid, float, or `order` is a new feature, not a port.
- [Taffy test fixtures](#taffy-test-fixtures): 813 fixtures are usable for differential testing today, 749 of them without a font dependency; the rest become usable as features are added. The `border_box_ltr` XML variants give expected rectangles without running a browser.
- [State machines](#state-machines): undo and reactivity have no time inside, which makes them cheap to model. Focus (caret epoch), gestures (`Gesture::began`), and the redraw scheduler read `Instant::now()` directly (39 calls in `anti-webui-core/src`, test modules included), with no injected clock, which makes them the hard ones.
- [Baseline violations](#baseline-violations): the two panics and the missing unit tests are baseline facts a fork requirement can raise but not fix without editing upstream-owned files.
