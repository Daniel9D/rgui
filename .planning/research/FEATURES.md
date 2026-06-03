# Features Research

**Domain:** Rust wgpu GUI library
**Researched:** 2026-06-03
**Confidence:** HIGH (grounded in the actual rsgui feature surface)

## Features Categorization

### Table Stakes (users expect these; v1 must-have)

- **Layout**: flex / grid via `taffy`; intrinsic sizing for text; scroll containers with track + thumb
- **Widgets**: button, input (text), checkbox, radio, select, textarea, tabs, tree, table, list, menu, menu-item, divider, icon, scroll area, modal, popover, tooltip
- **Events**: pointer (down / move / up), wheel, key, focus traversal, shortcuts
- **Theme**: light / dark, per-widget sizing baselines, per-variant styling, text color from theme
- **Accessibility**: role mapping, semantic tree, AccessKit backend (Win / mac), focus tracking
- **Diagnostics**: structured `UiSnapshot`, JSON dump, visual goldens, debug overlay
- **Adapters**: declarative XML (RML) for prototyping, basic CSS / Tailwind class utility

### Differentiators (competitive advantage)

- **Retained-mode + spec-driven widgets** (`*Spec` structs) — explicit, type-safe widget state; the widget data is decoupled from paint
- **Widget painter registry** — third parties can add custom widget kinds via `register_widget_painter` without forking the lib
- **Batched wgpu renderer** — `DisplayList` is flat, Z-sorted, suitable for `wgpu::RenderPass` batching
- **Lazy `AncestorIds` iterator** — event path walks don't allocate intermediate `Vec`s
- **Text-engine shape + layout caches** — repeated frames with stable text don't re-shape
- **Visual goldens with perceptual diff tolerance** — regression coverage without the flakiness of strict pixel equality
- **Structured `DisplayListError` (was: hand-rolled `String`)** — callers can match on specific failure modes (feedback #5.7)

### Anti-Features (deliberately NOT building)

- **Webview embed** (Tauri / wry) — defeats the Rust-native style system
- **Visual UI builder** — rsgui is a library, not a tool
- **Scriptable runtime** — no Lua / Python / JS engine; the lib is Rust-only
- **Layout engine in script** — the `RML` parser is for declarative UI text, not for runtime scripting
- **CSS / Tailwind full feature parity** — the adapters are utility helpers, not a layout engine
- **Built-in physics / animation** — `AnimationClock` exists but is thin; animation is the *user's* concern, not the lib's
- **Cross-widget DnD at the lib level** — the *Event* primitives are there (`PointerCapture`, `DragState`), but a generic `Drag` / `DropTarget` API is a v1.x feature
- **Multi-window at v1** — single-window is the v1 contract; multi-window is on the v2 roadmap

## Feature Dependencies

- **Layout** is the foundation. Every widget assumes layout has run. Without it, paint is undefined.
- **Theme** is the foundation. Every paint command reads from the resolved style. Without it, paint is monochrome.
- **Events** depend on hit-test, which depends on layout. The runtime runs hit-test after layout.
- **Accessibility** depends on the semantic tree, which is built during reconcile / pre-paint.
- **Diagnostics** depend on the runtime exposing structured data; `UiSnapshot` is built alongside paint.

## Complexity Notes

- **Layout (P0)**: The current full-rebuild-per-frame model is the cost ceiling. A incremental reconciliation is the most expensive single improvement on the roadmap.
- **Text shaping (P0)**: Cross-platform font fallback is hard. The current `glyphon` integration is good for Latin; CJK and RTL are weaker and need stress testing.
- **IME (P0)**: Preedit composition under wayland / Win32 / Cocoa have platform-specific gotchas. Real driver coverage is the only way to find them.
- **Theme v2 (P1)**: The `ComponentTheme` lookup is the long-term home for per-widget style overrides, but every widget today uses flat `Theme` fields. The migration is a deprecation alias dance across 25+ widget spec defaults.
- **Animation (P1)**: Tied to the layout and paint timeline. Adds a frame-time signal that flows through the runtime. Tread carefully: animation that animates layout means re-layout on every frame.

## Standard v1 Features (already shipped in some form)

These are "standard" for a GUI lib in 2026 and rsgui has them (or close to it):

- ✓ Element tree as source of truth
- ✓ Reconciliation into node tree (full-rebuild today; incremental pending)
- ✓ Layout with `taffy`
- ✓ Paint with 25+ widget painters
- ✓ Event dispatch with hit-test
- ✓ Focus management
- ✓ Theme system (light / dark, per-widget metrics)
- ✓ Basic a11y (role mapping, semantic tree, AccessKit backend)
- ✓ Declarative adapter (RML)
- ✓ Visual diagnostics (goldens, debug overlay, JSON dump)
- ✓ SVG / image rendering
- ✓ IME preedit (caret + underline)
- ✓ Drag state + pointer capture

## Standard v1 Features (NOT shipped — on the v1 roadmap)

- ✗ Incremental reconciliation (P0-01)
- ✗ Per-widget input audit (P0-02)
- ✗ IME under real drivers (P0-03)
- ✗ Multi-window (P0-04)
- ✗ Render-path stress test (P0-05)
- ✗ Public API documentation pass (P1-01)
- ✗ Theme v2 (per-widget overrides) (P1-02)
- ✗ Animation system (P1-03)
- ✗ Cross-widget DnD (P1-04)
- ✗ Virtualized list / table (P1-05)
- ✗ Canvas layout (P1-06)
- ✗ Custom widget public contract (P2-01)
- ✗ Layered theming (P2-02)
- ✗ i18n / RTL (P2-03)
- ✗ Per-platform integration tests (P2-04)
- ✗ Performance budgets (P2-05)
- ✗ Doc polish (P2-06)

## Differentiators That Are Aspirational (not committed yet)

- **Custom widget public contract** — `register_widget_painter` is in place; the public docs + reset / unregister pattern + a "writing a custom widget" guide are P2
- **Widget painter registry hot-path safety** — the registry is read on every paint node, but the lock is uncontended in normal use. A read-only `RwLock` would let the hot path skip the lock entirely

---
*Features research for: rsgui*
*Researched: 2026-06-03*
