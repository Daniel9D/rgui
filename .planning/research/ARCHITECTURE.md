# Architecture Research

**Domain:** Rust wgpu GUI library
**Researched:** 2026-06-03
**Confidence:** HIGH (grounded in the actual `src/` module structure)

## Component Overview

The rsgui architecture is a five-stage pipeline:

```
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│  Element     │    │   UiTree     │    │   Layout     │
│  (authoring) │ ─► │ (reconciled) │ ─► │   (taffy)    │
└──────────────┘    └──────────────┘    └──────────────┘
                                              │
                                              ▼
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│   Render     │    │   Paint      │    │   Hit-test   │
│  (wgpu pass) │ ◄─│ (DisplayList)│ ◄─ │  (events)    │
└──────────────┘    └──────────────┘    └──────────────┘
```

Each stage is a function in `src/runtime/`. The stages are pure
data transformations; no stage holds mutable state across frames
(except the `TextSystem` caches, which is the design's whole point).

## Components

### 1. `core/` — Data model and primitives

- `Element`, `ElementKind`, `WidgetSpec`, `WidgetKind`, `PrimitiveKind`
- `Style`, `ResolvedStyle`, `ResolvedWidgetStyle`, `Theme`, `ThemeMode`
- `LayoutBox`, `LayoutResult`, `LayoutDebugSnapshot`
- `PaintCommand`, `DisplayList`, `PaintedCommand`, `DisplayListError`
- `HitTestEntry`, `HitTestTree`, `EventPath`
- `Rect`, `Size`, `Point`, `Vec2`, `Radius`, `LayerKind`

`core` is the only module every other module depends on. It
contains no rendering logic. **Boundary rule:** never import
`runtime` from `core`.

### 2. `runtime/` — Frame pipeline

- `runtime.rs` — `UiRuntime::update(FrameInput) -> FrameOutput`; the orchestrator
- `tree.rs` — `UiTree`, `UiNode`, `AncestorIds`
- `events.rs` — `dispatch_event`, `EventPath`, `FocusEntry`, `FocusSystem`
- `state.rs` — `BoolState`, `DragState`, `PointerCapture`, `ScrollState`
- `frame.rs` — `FrameInput`, `FrameOutput`
- `paint.rs` — `paint_node`, `paint_node_with_text`, `paint_node_themed`; the painter trait + registry + factory
- `reconcile.rs` — `Reconciler` (partial; full-rebuild today)
- `portal_pass.rs` — portal composition
- `overlay_pass.rs` — modal / popover composition
- `text_metrics.rs` — heuristic text measurement (per-thread cache)
- `debug.rs` — `format_frame_dump`, `DebugVisualMode`

### 3. `text_engine/` — Text shaping

- `TextSystem` — wrapper around `cosmic-text` + `glyphon`
- `measure` / `measure_wrapped` / `measure_intrinsic` — text sizing
- `shape` / `shape_with_size` — glyph shaping
- Shape and layout caches keyed on `(text, font_id, size, weight, width)`

### 4. `render/wgpu/` — GPU backend

- `WgpuRenderer` — owns the device, queue, surface, pipeline
- `OffscreenTarget` — headless render target (tests)
- `Pipeline` — render pipeline state
- `Item`, `RenderItem` — GPU-side command primitives
- `Atlas` — texture atlas for glyphs and images
- `BitmapText` — fallback text rendering (the `bitmap-text-fallback` feature)

### 5. `widgets/` — Public widget constructors

- `button`, `input`, `checkbox`, `radio`, `select`, `textarea`
- `tabs`, `tree`, `table`, `list`
- `menu`, `menu_item`, `context_menu`
- `popover`, `modal`, `tooltip`
- `card`, `badge`, `link`, `alert`
- `progress_bar`, `spinner`, `switch`, `slider`
- `image`, `avatar`, `divider`, `icon`, `canvas`
- `scroll_area`, `text`, `column`, `row`, `grid`, `stack`, `absolute`

### 6. `a11y.rs` — Accessibility

- `Role`, `SemanticAction`, `role_for_widget_kind`
- `AccessibilityBackend` (trait) + `RealAccessibilityBackend` (AccessKit) + `HeadlessBackend` (tests)

### 7. `adapters/` — Authoring surfaces

- `RML` parser (XML) — behind the `rml` feature
- `css_to_style` (CSS subset) — behind the `css` feature
- `classes_to_style` (Tailwind) — behind the `tailwind` feature
- `parse_element` (HTML minimal) — behind the `html` feature

### 8. `svg.rs` and `images.rs` — Asset decoders

- `rasterize_svg_bytes` — SVG → RGBA pixels
- `image::open` (via the `image` crate) — PNG / JPEG / etc.

## Data Flow

Per frame:

1. **Authoring**: User code constructs an `Element` tree (or runs through the RML parser).
2. **Reconcile**: `runtime::update` builds a fresh `UiTree` from the `Element`. (Future: diff against the previous tree.)
3. **Compute visual state**: `visual_state_for_element` (and friends) compute per-node `VisualState` (hovered, focused, etc.) from the tree and event-derived state.
4. **Layout**: `taffy` is invoked; each node gets a `LayoutBox` (rect, content rect, scroll offset).
5. **Hit-test**: The layout feeds a `HitTestTree`; pointer events walk the tree.
6. **Event dispatch**: For each event, the hit-tested path becomes an `EventPath`; per-phase handlers run, with `WidgetKind`-aware shortcuts and the registry's lookup for custom widgets.
7. **Paint**: For each node, `paint_node_themed` produces a list of `PaintCommand`s (rect, border, text, image, svg, path, shadow, push/pop layer, push/pop clip). Commands are sorted by `z_index` with stack ops (i32::MIN) first.
8. **Render**: The `DisplayList` is uploaded to `wgpu`; the `WgpuRenderer` walks it, batches by pipeline, and submits commands.
9. **Diagnostics**: After paint, the `UiSnapshot` is updated; `to_debug_json()` can be called; visual goldens can be diffed.

## Build Order (suggested for v1.0)

1. **Reconciliation + Layout** — the cost ceiling today; without this, "robust" is a stretch
2. **Text / IME** — without this, anything but English text is a regression risk
3. **Public API** — audit + doctests, so the v1.0 release is honest
4. **Theme v2** — per-widget overrides, the long-promised `ComponentTheme` path
5. **Render-path stress** — real-world test scene, fix what breaks
6. **Animation + DnD + Virtualization** — the v1.x feature surface
7. **Custom widget contract** — docs, examples, the public story for "I added a custom widget to my app"
8. **i18n + per-platform tests** — the v1.x quality bar
9. **Doc polish** — every public type through doctest

## Component Boundaries

The most fragile boundary is **`runtime` ↔ `widgets`**. The `paint.rs`
factory knows every `WidgetKind`; each `WidgetPainter` is a Rust impl
in `paint.rs`. A future refactor (8.2) would move the widget painters
to `widgets/paint/`, putting the boundary in the right place (a
widget definition is a user-facing concept; the runtime should not
own it). The blocker is the `pub(super)` visibility in
`paint.rs` for the shared helpers; the visibility wall was lowered
in 8.1 prep but the actual extraction is a follow-up.

The other fragile boundary is **`runtime` ↔ `render/wgpu`**. Today
the runtime is the orchestrator and `render/wgpu` is the
implementation. A clean separation would have the runtime emit a
`DisplayList` and a separate `render` crate consume it. The current
shape is "runtime imports render" which is correct (the runtime
asks the renderer to consume its output), but the `render/wgpu` API
is not stable enough to make this a v1 promise.

## State Boundaries

- **Reconciled state** lives in `UiTree`. Owned by `UiRuntime` for the lifetime of the runtime.
- **Per-element state** (text content, scroll offset, focus) lives in `BoolState` / `ScrollState` etc. on the runtime. Cleared on rebuild (future: persisted across rebuild via id-keyed maps).
- **Theme state** lives in the `Theme` passed in `FrameInput`. Read-only during paint.
- **Snapshot state** (`UiSnapshot`) is rebuilt each frame in the runtime and held until the next `update()`.
- **Display list state** is in `FrameOutput.display_list`, owned by the caller.

## Coupling Notes

- **`taffy` is the layout engine.** Replacing it is a v2-level change; the v1 commitment is to keep up with taffy releases.
- **`glyphon` is the text engine.** Same. The interaction between `cosmic-text`'s shaping and `glyphon`'s atlas upload is the most fragile single point in the renderer.
- **`wgpu` is the GPU API.** The `render/wgpu` module is the only place that depends on wgpu. Swap in a `render/dx12` or `render/vulkan` would be a parallel implementation, not a replacement.

---
*Architecture research for: rsgui*
*Researched: 2026-06-03*
