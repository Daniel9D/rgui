# Research Summary

**Project:** rsgui (Rust wgpu GUI library)
**Researched:** 2026-06-03
**Status:** Research complete; STACK / FEATURES / ARCHITECTURE / PITFALLS written.

## Key Findings

**Stack:** Pinned and battle-tested. `wgpu 29` + `taffy 0.10.1` +
`glyphon 0.11.0` + `kurbo 0.11` is the right combination for a
retained-mode, GPU-accelerated Rust GUI in 2026. Confidence HIGH.

**Features:** 12 of 12 table-stakes features are in the codebase
in some form. The "robust" gap is 22 P0/P1/P2/P3 items, dominated
by incremental reconciliation, public API audit, and per-platform
integration testing. Confidence HIGH.

**Architecture:** The five-stage pipeline (`Element → UiTree → Layout
→ DisplayList → wgpu`) is clean and the boundaries are mostly
right. The `runtime ↔ widgets` boundary is the most fragile
(painters live in `runtime/paint.rs` but conceptually belong to
`widgets/`). The `runtime ↔ render/wgpu` boundary is correct
today but the `render/wgpu` API needs stabilization before v1.
Confidence HIGH.

**Pitfalls:** 12 critical mistakes identified. Three are already
fixed in the codebase (TextSystem::default per call, z-index
sorting, visual golden flakiness). Three are P0 work
(reconciliation, IME, multi-window). The rest are spread across
P1 / P2. Confidence HIGH.

## Recommended Starting Point

Phase 1 should be **incremental reconciliation** (P0-01). The
"robust" promise is currently gated on it: every frame rebuilds
the entire tree, redoes taffy, repaints everything. Without
incremental reconciliation, the lib is "feature-complete but
not performant", which is a v0.x state.

The P0 list is short and serial:
1. Incremental reconciliation + pointer-capture leak fix
2. Per-widget input correctness audit
3. IME under real drivers
4. Multi-window
5. Render-path stress test

P0 is roughly one milestone's worth of work. The v1.0 release
sits at the end of P1.

## What NOT to Do

- Don't try to make v1.0 a v0.1 of "everything". Pick the P0
  items, ship, then P1.
- Don't add a custom widget API surface (P2-01) before the
  v1.0 release. The current `register_widget_painter` is
  experimental.
- Don't try to be a webview / embed a browser. The Rust-native
  style system is the design.
- Don't try to replace `egui` or `iced`. Different design
  points; rsgui is retained-mode + spec-driven.

## Watch Out For

- **wgpu / glyphon version drift.** Treat the `wgpu 29` /
  `glyphon 0.11` pin as a pair. Bumping one means checking
  the other.
- **Vulkan validation layers** in CI. The test suite must run
  with validation enabled; otherwise buffer alignment issues
  sneak in.
- **IME driver coverage** is the only way to find caret /
  preedit bugs. Synthetic tests can confirm the paint path;
  real drivers find the OS integration.

## Files

- `STACK.md` — full version-pinned stack + alternatives + watch-outs
- `FEATURES.md` — table stakes vs differentiators vs anti-features + dependencies + complexity
- `ARCHITECTURE.md` — five-stage pipeline, component boundaries, build order
- `PITFALLS.md` — 12 critical mistakes with warning signs + prevention + phase mapping
