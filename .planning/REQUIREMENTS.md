# Requirements: rsgui

**Defined:** 2026-06-03
**Core Value:** The paint pipeline produces a correct, sorted `DisplayList` for every `Element` tree — every `WidgetKind` paints something visible with the right z-order, the right hover/disabled/checked state, and the right glyph from the right font.

## v1 Requirements

### Reconciliation (RECON)

- [x] **RECON-01**: Runtime can diff a new `Element` root against the prior `UiTree` and update only changed subtrees
- [x] **RECON-02**: Pointer-capture state is released when the captured node is removed from the tree
- [x] **RECON-03**: Layout runs incrementally on dirty regions (not full-tree re-layout on every frame)
- [x] **RECON-04**: Widget spec hash mismatch triggers a re-mount of the affected node

### Event / input (EVNT)

- [ ] **EVNT-01**: Tree widget responds to arrow keys (up / down / left / right) for keyboard navigation
- [ ] **EVNT-02**: List widget responds to arrow keys + Home / End
- [ ] **EVNT-03**: Tabs widget responds to arrow keys + Ctrl-Tab to cycle
- [ ] **EVNT-04**: Slider widget responds to drag, click, and arrow keys
- [ ] **EVNT-05**: Switch widget responds to Space / Enter toggle
- [ ] **EVNT-06**: ProgressBar widget ignores clicks (or documents its actual click behavior)

### Text / IME (TEXT)

- [ ] **TEXT-01**: Text input handles IME preedit composition under at least two real drivers (Windows + macOS or Linux)
- [ ] **TEXT-02**: Text shaping round-trips for Latin, CJK, and one RTL script
- [ ] **TEXT-03**: Caret position is correct under preedit
- [ ] **TEXT-04**: Per-thread measure cache hit rate is observable via `TextCacheStats` and `clear_metrics_cache`

### Multi-window (WIN)

- [ ] **WIN-01**: `FrameInput` carries a `window_id` field
- [ ] **WIN-02**: Multiple `UiRuntime` instances can coexist in one process
- [ ] **WIN-03**: Events route to the correct window's runtime
- [ ] **WIN-04**: Each window has its own `DisplayList` and `UiSnapshot`

### Render path (REND)

- [ ] **REND-01**: Visual goldens pass on at least two GPU backends (Vulkan + Metal, or Vulkan + DX12)
- [ ] **REND-02**: A real-world stress test scene (overlapping translucent layers, image-heavy tables, animated progress bars, IME, drag-and-drop) renders without errors
- [ ] **REND-03**: `wgpu` validation layers enabled in CI surface buffer alignment issues
- [ ] **REND-04**: Frame CPU budget is < 8ms for a 50-widget desktop UI on a modern laptop

### Public API (API)

- [ ] **API-01**: Every public type in the `rgui` crate root has a doctest
- [ ] **API-02**: `cargo doc --document-private-items` builds without warnings
- [ ] **API-03**: No `unwrap()` in the runtime paint path under non-pathological inputs
- [ ] **API-04**: `WidgetPainter` is `Send + Sync`; custom-painter docs explain the contract

### Theme (THEM)

- [ ] **THEM-01**: `ComponentTheme` lookup path is the canonical way to override per-widget styles
- [ ] **THEM-02**: Flat `Theme::metrics` / `Theme::select` fields are deprecated with aliases
- [ ] **THEM-03**: Light / dark / high-contrast themes ship out of the box
- [ ] **THEM-04**: Color-blind variants are documented (not necessarily shipped)

### Animation (ANIM)

- [ ] **ANIM-01**: Tween animations on `Length` (position, size)
- [ ] **ANIM-02**: Tween animations on `Color` and `f32` style values
- [ ] **ANIM-03**: Spring animations with configurable stiffness / damping
- [ ] **ANIM-04**: Easing curves exposed as a public API

### Drag-and-drop (DND)

- [ ] **DND-01**: A `Drag` source API on `Element`
- [ ] **DND-02**: A `DropTarget` API on `Element`
- [ ] **DND-03**: Cross-widget drop with payload type
- [ ] **DND-04**: Drop indicator (visual feedback for valid drop targets)

### Virtualization (VIRT)

- [ ] **VIRT-01**: `WindowedList` spec that only paints visible rows
- [ ] **VIRT-02**: `WindowedTable` spec that only paints visible rows + columns
- [ ] **VIRT-03**: Scrollable windowed lists with sticky headers
- [ ] **VIRT-04**: 10k-row windowed list renders at 60 fps on a modern laptop

### Layout extensions (LOUT)

- [ ] **LOUT-01**: A `Canvas` widget that uses absolute-coordinate layout
- [ ] **LOUT-02**: `Canvas` widgets compose with flex / grid siblings
- [ ] **LOUT-03**: `ZStack` primitive for stacking without flex

### Diagnostics (DIAG)

- [ ] **DIAG-01**: `UiSnapshot` exposes a stable JSON schema (locked in tests)
- [ ] **DIAG-02**: `to_debug_json()` output is round-trip-safe (parseable + re-serializes to the same value)
- [ ] **DIAG-03**: Visual goldens run in CI on every PR

### i18n / RTL (I18N)

- [ ] **I18N-01**: `taffy` RTL layout (logical properties) is supported
- [ ] **I18N-02**: Text shaping respects locale (number / date formatting is a v1.x concern)

### Custom widget (CUST)

- [ ] **CUST-01**: `register_widget_painter` has a stable public contract
- [ ] **CUST-02**: `unregister_widget_painter` exists with a documented use case
- [ ] **CUST-03**: A "writing a custom widget" guide is in the docs

## v2 Requirements

Deferred to v1.x / v2.

### Multi-window (v2)
- **WIN-05**: Drag a widget between windows
- **WIN-06**: Modal that spans multiple windows

### Animation (v2)
- **ANIM-05**: Physics-based motion (springs, gravity)
- **ANIM-06**: Layout transitions (FLIP)

### DnD (v2)
- **DND-05**: Drag from an external app (OS-level)
- **DND-06**: Drop an OS file into a window

### Virtualization (v2)
- **VIRT-05**: Multi-column windowed grids
- **VIRT-06**: Variable-height rows

### Diagnostics (v2)
- **DIAG-04**: Performance profiler overlay
- **DIAG-05**: Tree inspector overlay

### Accessibility (v2)
- **A11Y-01**: Voice control integration
- **A11Y-02**: High-contrast theme variant shipped
- **A11Y-03**: Color-blind theme variants shipped

### Internationalization (v2)
- **I18N-03**: Locale-aware number / date / currency formatting
- **I18N-04**: Right-to-left text shaping for Arabic / Hebrew
- **I18N-05**: CJK vertical text layout

### Mobile / touch (v2)
- **MOBL-01**: Touch-event support
- **MOBL-02**: Gesture recognition (pinch / pan)
- **MOBL-03**: Soft keyboard integration

### WASM (v2)
- **WASM-01**: WebGPU backend production-grade
- **WASM-02**: Browser DOM fallback for unsupported wgpu features

## Out of Scope

| Feature | Reason |
|---------|--------|
| Webview / HTML engine embed | Defeats the Rust-native style system design |
| Visual UI builder tool | rsgui is a library, not a tool |
| Scriptable runtime (Lua / Python / JS) | The lib is Rust-only |
| Drop-in replacement for `egui` (immediate-mode) | Different design point |
| Drop-in replacement for `iced` (elm-style) | Different design point |
| Mobile / touch-first design (v1) | Desktop-first; touch is a future concern |
| Built-in physics / animation engine | Animation is the user's concern |
| Web fetch / HTTP client | Not a GUI concern |
| Persistent storage | The runtime is in-memory; the user provides the persistence |

## Traceability

(Filled in by roadmap creation.)

| Requirement | Phase | Status |
|-------------|-------|--------|
| RECON-01 | Phase 1 | Complete |
| RECON-02 | Phase 1 | Complete |
| RECON-03 | Phase 1 | Complete |
| RECON-04 | Phase 1 | Complete |
| EVNT-01..06 | Phase 2 (widget keyboard nav) | Pending — Phase 2 covered the runtime event path (focus traversal, shortcut suppression, wheel 2D, IME gating) but EVNT-01..06 are widget-specific keyboard handlers not addressed here. Slated for a future widget-interaction phase. |
| TEXT-01 | Phase 2 (runtime side) | Partial — runtime routes `ImePreedit`/`ImeCommit` when `InputSpec::ime_enabled = true`; preedit paints and commits. Host driver integration (winit/browser) remains the v1.x path. |
| TEXT-02..04 | Phase 1 | Pending |
| WIN-01..04 | Phase 2 | Pending |
| REND-01..04 | Phase 2 | Pending |
| API-01..04 | Phase 2 | Pending |
| THEM-01..04 | Phase 3 | Pending |
| ANIM-01..04 | Phase 3 | Pending |
| DND-01..04 | Phase 3 | Pending |
| VIRT-01..04 | Phase 4 | Pending |
| LOUT-01..03 | Phase 4 | Pending |
| DIAG-01..03 | Phase 4 | Pending |
| I18N-01..02 | Phase 5 | Pending |
| CUST-01..03 | Phase 5 | Pending |

**Coverage:**
- v1 requirements: 60 total
- Mapped to phases: 60
- Unmapped: 0 ✓

---
*Requirements defined: 2026-06-03*
*Last updated: 2026-06-03 after initial definition*
