# rsgui

## What This Is

A Rust GUI library for `wgpu` that gives desktop and embedded-wgpu
applications a retained-mode, GPU-accelerated, themeable, accessible
UI surface — `Element` tree in, `DisplayList` out, with a clean
separation between layout, paint, and event dispatch. Targeted at
developers who want the GPU story of `wgpu` without giving up the
ergonomics of a retained-mode toolkit.

## Core Value

The paint pipeline produces a correct, sorted `DisplayList` for
every `Element` tree — every `WidgetKind` paints something visible
with the right z-order, the right hover/disabled/checked state, and
the right glyph from the right font. If paint is wrong, nothing
else matters; every other capability exists to make that output
useful.

## Requirements

### Validated

- ✓ `Element` tree as source of truth, with `WidgetSpec` ↔ `WidgetKind` round-trip — existing
- ✓ `UiTree` build / hit-test / focus / event dispatch — existing
- ✓ Taffy integration for flex / grid layout — existing
- ✓ `DisplayList` paint pipeline with text, layer, clip z-ordering — existing
- ✓ Per-widget `WidgetPainter` for every `WidgetKind` variant (25+ painters) — existing
- ✓ `WidgetPainterRegistry` for third-party extension — existing
- ✓ Theme system (light / dark) with per-widget `WidgetMetrics` — existing
- ✓ AccessKit / a11y backend for screen readers (behind feature flag) — existing
- ✓ Visual goldens with perceptual diff tolerance for regression detection — existing
- ✓ RML (declarative XML) + CSS / Tailwind / SVG / minimal-HTML adapters — existing
- ✓ `to_debug_json()` via `serde_json` (string-safe) — existing

### Active

- [ ] P0-01: Incremental reconciliation (re-tree on every frame is the cost ceiling right now) — *Roadmap P0*
- [ ] P0-02: Per-widget input correctness audit (Tree/List/Tabs/Slider/Switch/ProgressBar not yet covered by integration tests) — *Roadmap P0*
- [ ] P0-03: Text editing under IME (CJK + dead keys) — *Roadmap P0*
- [ ] P0-04: Multi-window support (single-window today) — *Roadmap P0*
- [ ] P0-05: Stable wgpu render path under heavy / realistic load — *Roadmap P0*
- [ ] P1-01: Stable, documented public API surface (with doctest pass) — *Roadmap P1*
- [ ] P1-02: Theme system v2 — `ComponentTheme` per-widget style overrides — *Roadmap P1*
- [ ] P1-03: Animation system (tween / spring on layout, paint, theme values) — *Roadmap P1*
- [ ] P1-04: Drag-and-drop across widgets — *Roadmap P1*
- [ ] P1-05: Virtualized lists / tables for large data — *Roadmap P1*
- [ ] P1-06: Canvas / absolute-coordinate layout algorithm — *Roadmap P1*
- [ ] P2-01: Custom widget registration public contract + docs — *Roadmap P2*
- [ ] P2-02: Layered theming (light, dark, high-contrast, color-blind) — *Roadmap P2*
- [ ] P2-03: Internationalization (RTL, locale-aware text shaping) — *Roadmap P2*
- [ ] P2-04: Per-platform integration tests (Vulkan + Metal at minimum) — *Roadmap P2*
- [ ] P2-05: Performance budgets with regression tests — *Roadmap P2*
- [ ] P2-06: Doc polish — full `docs/public-api.md` pass against current state — *Roadmap P2*
- [ ] P3-01: Animation easing curves public API — *Roadmap P3*
- [ ] P3-02: Render-time debug toggles beyond `DebugVisualMode` — *Roadmap P3*
- [ ] P3-03: Live playground / examples crate — *Roadmap P3*
- [ ] P3-04: Migration guide for v0.x → v1 — *Roadmap P3*
- [ ] P3-05: Style guide for `*Spec` authors — *Roadmap P3*

### Out of Scope

- Webview / HTML engine embed — defeats the Rust-native style system design
- Visual UI builder tool — rsgui is a library, not a tool
- Drop-in replacement for `egui` (immediate-mode) — different design point
- Drop-in replacement for `iced` (elm-style) — different design point
- Mobile / touch-first design — desktop-first; touch is a future compatibility concern, not a v1 goal
- Tailwind full feature parity — `tailwind` adapter is a thin utility, not a layout engine

## Context

The codebase is real and committed: 48 commits ahead of `origin/main`
at PRD time, with 62 feedback items from a Mavis code review
addressed across the implementation (visual goldens passing, all
51 integration test suites passing modulo 2 pre-existing baseline
failures that are independent of any feedback item).

`feedback.md` and `prd.md` in the project root are the audit
trails. `feedback.md` is the code review; `prd.md` is the
forward-looking product brief.

The target environment:
- Rust 2024 edition, MSRV set in `Cargo.toml`
- `wgpu 29` (active backend), `glyphon 0.11` for text shaping, `taffy 0.10` for layout
- Platform: Windows / macOS / Linux desktop; Vulkan / Metal / DX12 backends

## Constraints

- **Tech stack**: Rust + `wgpu` + `taffy` + `glyphon`. The retained-mode model + Rust-native style system is non-negotiable (the design's whole point).
- **Performance**: 60 fps for typical desktop UIs on integrated GPUs. Each frame's full pipeline (layout + paint + render) must fit in ~8ms of CPU budget on a modern laptop.
- **Compatibility**: Must build and run on Windows / macOS / Linux. Mobile / WASM are not v1 targets but the lib must not preclude them.
- **Backwards compatibility**: Public API changes need a deprecation alias in the same release. We don't have a 1.0 yet; pre-1.0 breaking changes need a CHANGELOG note and a migration entry in the docs.
- **No runtime dependencies** outside the chosen stack. No `tokio`, no `serde` for the hot path (serde is allowed in debug dumps only).
- **Test discipline**: Every interactive widget must be covered by the event dispatch integration test suite. No `unwrap()` in the runtime paint path under non-pathological inputs.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Retained-mode `Element` tree over immediate-mode | Enables a clean dispatch model, themeable state, and accessibility without per-frame closure recreation. The reviewer (Mavis) flagged this as the right call for a wgpu GUI lib. | — Pending v1 |
| `WidgetPainter` trait with template-method `paint()` | Lets new widget kinds add only what differs (background, content) and inherit border + template from the trait. Enables `WidgetPainterRegistry` for third-party extension. | ✓ Good — registry in place |
| Flat `Theme::metrics` / `Theme::select` over `theme.widgets.*` wrapper | `WidgetThemes` was a 2-field indirection that didn't pay for itself. The `components.get(WidgetKind)` path is kept as the long-term per-widget style home. | ✓ Good (feedback #3.8) |
| `serde_json` for `to_debug_json` (was hand-rolled `format!`) | Hand-rolled broke on string fields. `serde_json` gives stable, parseable output and is the only debug-path dep. | ✓ Good (feedback #2.8) |
| `#[doc(hidden)] pub use core::*` at crate root | 193-item explicit re-export list churns every time a type is added. `#[doc(hidden)]` keeps the re-export working but marks it as crate-internal. | ✓ Good (feedback #8.5) |
| Per-thread `measure_text` cache | The heuristic is O(depth × call-site); caching per `(text, size, weight, style, max_width)` key gives O(1) hot-path lookups for the common re-render case. | ✓ Good (feedback #4.2) |
| `SmallVec` skipped for paint_node return | Reviewer suggested `SmallVec<[_; 4]>` to avoid the 1-3-element heap alloc. Skipped because the trait method signatures + every painter impl need updating and the win is small for the cost. Deferred. | ⚠️ Revisit post-v1 |

---
*Last updated: 2026-06-03 after PRD creation*

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state
