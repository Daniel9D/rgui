---
phase: 03
phase_name: text-ime
generated: 2026-06-03
sources:
  - .planning/phases/03-text-ime/03-CONTEXT.md
analogs_scanned:
  - src/core/event.rs
  - src/runtime/runtime.rs
  - src/text_engine/system.rs
  - src/render/wgpu/glyphon_text.rs
  - src/state/input.rs
  - src/state/snapshot.rs
  - src/runtime/text_metrics.rs
  - src/widgets/spec.rs
  - tests/ime_gating.rs
  - tests/event_input_hardening.rs
---

# Phase 3 Pattern Map

Mapping each new file or behavior to its closest existing analog. Plan authors
should replicate the cited style for new code.

## Plan 03-01: IME host driver

| New construct | Analog | Where |
|---------------|--------|-------|
| `ImeHostDriver` trait (producer-side, `poll(&mut sink)`) | `FocusManager` trait (lifecycle: `request_focus` / `clear` / `focused`); small interface owned by the runtime, with test mocks | `src/core/event.rs:120-137` |
| `ImeEventSink` trait (sink shape) | `WidgetPainter` trait (small render-facing interface the runtime invokes) | `src/runtime/paint.rs:1-40` |
| `MockDriver { script: Vec<ImeOp>, cursor: usize }` | Existing `tests::focus_traversal` test mocks — concrete impl in `#[cfg(test)]` module | `src/core/event.rs::tests::focus_traversal` |
| `NoopDriver` (default for `UiRuntime::default()`) | Existing default fields in `UiRuntime` (e.g. `state: WidgetStateMap::default()`) | `src/runtime/runtime.rs:900-1000` |
| `driver: Box<dyn ImeHostDriver>` field on `UiRuntime` | Existing `Box<dyn WidgetPainter>` patterns in widget registry | `src/runtime/paint.rs` |
| Per-frame `driver.poll(&mut self)` call site | `runtime::update()` event-queue drain pattern (poll external, then process) | `src/runtime/runtime.rs:340-380` |
| `tests/ime_host_driver.rs` integration test shape | `tests/ime_gating.rs` (3 tests, same pattern of `UiRuntime::default()` + `runtime.update()` + `snapshot().input_state` assertions) | `tests/ime_gating.rs:1-120` |

**Code style for `ImeHostDriver`:**
- Use `&mut dyn ImeEventSink`, not generic `S: ImeEventSink`, to allow trait object storage.
- Methods take `&mut self` (drivers can have state — e.g. `MockDriver` has a cursor).
- No `async`; per `PROJECT.md` the hot path is sync. The trait is plain.

## Plan 03-02: CJK + Arabic shaping tests

| New construct | Analog | Where |
|---------------|--------|-------|
| `tests/text_shaping_cjk_rtl.rs` | Existing test file style: `use` declarations at top, helper fn `shape_string(...)` in same file, then `#[test] fn name() { ... }` per case | `tests/widgets_native.rs`, `tests/event_input_hardening.rs` |
| `scripts/ci-install-fonts.sh` | One-line idempotent script (apt-get install with `--no-install-recommends`); macOS branch documents `brew install --cask ...` for human use | (no existing analog; new) |
| `CONTRIBUTING.md` "System fonts" section | None yet (file may not exist) | (no existing analog; new) |
| `RGUI_REQUIRE_FONTS=1` env var | `RGUI_UPDATE_GOLDENS=1` precedent in `CLAUDE.md` | `CLAUDE.md` (top of file) |
| Test fixture: `shape_string(text: &str) -> Vec<ShapedGlyph>` | Existing `tests/visual_goldens.rs` helper functions drive `glyphon` directly | `tests/visual_goldens.rs` |

**Code style for shaping tests:**
- Use `tracing::warn!` (not `eprintln!`) for skip messages — consistent with rest of test suite.
- Detect Noto CJK via `fonts-noto-cjk` package check; if missing, skip with `return` at the start of the test.
- Use `Shaping::Advanced` explicitly in the test call — even though it's the default in `glyphon_text.rs`, the test pins it for clarity.

## Plan 03-03: TextCacheStats observability

| New construct | Analog | Where |
|---------------|--------|-------|
| `TextSystem::clear_caches() -> (usize, usize)` | Existing `clear_metrics_cache() -> usize` in `runtime/text_metrics.rs:80` | `src/runtime/text_metrics.rs:73-101` |
| `UiRuntime::text_cache_stats() -> TextCacheStats` | Existing `UiRuntime` public methods (e.g. `snapshot()`, `update()`); no doc on private fields | `src/runtime/runtime.rs:340-1000` |
| `UiRuntime::clear_text_cache()` | `UiRuntime::default()` factory; `clear_metrics_cache()` free function | `src/runtime/text_metrics.rs:80-87` |
| `RendererStats.text_cache: TextCacheStats` field | Existing `RendererStats.glyphon_enabled: bool` (Phase 1 added it) | `src/core/render.rs:483` |
| `UiSnapshot.text_cache: TextCacheStats` field | Existing snapshot field pattern (e.g. `focused: Option<NodeId>`, `tree_size: usize`) | `src/state/snapshot.rs` |
| `to_debug_json()` extension | Existing `to_debug_json()` impl — append a new key, not modify existing ones | `src/state/snapshot.rs` |
| `pub fn text_metrics_cache_stats() -> TextMetricsCacheStats` | `pub fn metrics_cache_stats() -> TextMetricsCacheStats` already exists at `text_metrics.rs:97` (just missing the `pub fn` already present — re-check) | `src/runtime/text_metrics.rs:97-101` |

**Code style for stats surface:**
- `TextCacheStats` is `#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]` already.
- `RendererStats` extension must be backward-compatible: append a field, don't reorder.
- `to_debug_json()` already uses a specific format; append `text_cache` after existing keys.

## Code excerpts to mirror (cited for plan authors)

```rust
// src/text_engine/system.rs:90-99 — TextSystem::cache_stats
pub fn cache_stats(&self) -> TextCacheStats {
    TextCacheStats {
        shape_hits: self.shape_hits,
        // ...
        shape_entries: self.shape_cache.len(),
        layout_entries: self.layout_cache.len(),
    }
}
```

```rust
// src/runtime/text_metrics.rs:80-87 — clear_metrics_cache shape
pub fn clear_metrics_cache() -> usize {
    METRICS_CACHE.with(|cell| {
        let mut cache = cell.borrow_mut();
        let n = cache.entries.len();
        cache.entries.clear();
        n
    })
}
```

```rust
// src/core/event.rs:120-137 — small trait owned by runtime
pub struct FocusManager { /* ... */ }
impl FocusManager {
    pub fn request_focus(&mut self, node: NodeId) { /* ... */ }
    pub fn clear(&mut self) { /* ... */ }
    pub fn focused(&self) -> Option<NodeId> { /* ... */ }
}
```
