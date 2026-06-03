# Pitfalls Research

**Domain:** Rust wgpu GUI library
**Researched:** 2026-06-03
**Confidence:** HIGH (mix of Mavis's review notes and runtime lessons)

## Critical Mistakes

### 1. Full-rebuild reconciliation as the steady state
**Warning signs:** Frame CPU budget measured at >5ms with a moderately-sized
UI (>50 visible widgets). `taffy::compute_layout` is the dominant cost.
**Prevention:** Move to incremental reconciliation as a P0 priority, *before*
adding more features. The `DirtyFlags` infrastructure exists; the `Reconciler`
is partial; the integration is the work.
**Phase mapping:** P0-01 in the Active requirements.

### 2. Per-frame `TextSystem::default()` destroying the shape cache
**Warning signs:** `measure` or `shape` shows up in flame graphs as a
significant fraction of the frame; cache hit rate < 50%.
**Prevention:** Always use `paint_node_with_text(..., &mut text)` (or
`paint_node_themed`) with a *shared* `TextSystem` across sibling
nodes. Never call the old `paint_node` (the `TextSystem::default()`
per call) on a real frame.
**Phase mapping:** Already fixed (feedback #2.7) but the doc note
needs to be loud.

### 3. Painting the same widget kind with the wrong background color
**Warning signs:** Widgets that share a `WidgetKind` but appear in
different themes (e.g. `Tree` and `List` both use `theme.colors.background`
because the theme's surface and the page background should match)
silently render wrong if the painter uses `ctx.style.background`
instead of `ctx.theme.colors.background`.
**Prevention:** Every painter's `background_color` hook should
document the source (style vs theme). The current painters are
correct; the doc has to follow.

### 4. z-index sorting ignoring `PushLayer` / `PopLayer` / `PushClip` / `PopClip`
**Warning signs:** A pushed layer that draws with z=1000 is
re-ordered with siblings of z=0; the clip / layer bracket breaks.
**Prevention:** `PaintCommand::z_index()` returns `i32::MIN` for
the four stack ops. The `wgpu` renderer sorts by `z_index`
ascending; stack ops are at the top of the order, so they bracket
their content. The fix is in place (feedback #2.13) but worth
re-checking on every new command type.

### 5. Pointer-capture leak
**Warning signs:** A `PointerCapture` is set on a node that is
removed in the next `update()`; the next click goes nowhere or
goes to the wrong node.
**Prevention:** On reconcile, every captured key is checked against
the new tree; orphaned captures are released. Today this is *not*
implemented — the capture is keyed on the node and survives
rebuilds. The reconciliation work (P0-01) is the place to add this.

### 6. IME preedit fighting caret
**Warning signs:** The caret appears at the wrong offset during
composition, or the underline drifts off the preedit region.
**Prevention:** The paint path uses the layout's preedit-aware
metrics (`measure` returns preedit info as part of the layout).
Test with a real IME driver on at least two platforms before
claiming the caret is correct.

### 7. Event dispatch reentrancy
**Warning signs:** A handler triggers a state change that causes
another event to fire in the same frame; the second event sees
inconsistent state.
**Prevention:** The runtime should snapshot `VisualState` per
frame and dispatch against the snapshot. Today the runtime mutates
`VisualState` during dispatch; this is OK for the test suite but
will bite under reentrant handlers. The fix is in the dispatch
loop, not the handler.

### 8. Tree shape changes invalidating the hit-test cache
**Warning signs:** Pointer events on a newly-mounted widget
suddenly hit the wrong node. The cache was populated from the
*previous* tree.
**Prevention:** The `HitTestTree` is rebuilt every frame; the
`paint.rs` factory walks it. The risk is the *next* optimization
someone might add: a "reuse hit-test across frames" optimization
must invalidate the cache on tree change.

### 9. Atlas eviction blowing out the next frame's cache
**Warning signs:** A single bad image (taller than `row_height`)
triggers a full atlas eviction. The next frame re-uploads every
glyph and every image.
**Prevention:** The atlas already warns (feedback #1.7). The
long-term fix is a true LRU + a per-kind minimum reservation. For
v1, document the atlas size budget and the warning as the user
contract.

### 10. Visual golden flakiness
**Warning signs:** Goldens pass locally but fail on a CI box. The
failing diff is 1-2 units of channel difference.
**Prevention:** Strict pixel equality is replaced with a perceptual
tolerance (feedback #7.3). The current threshold (1 unit per
channel, 0.01% of pixels) is conservative; if it grows in the
future, the per-pixel tolerance is the thing to revisit, not the
ratio threshold.

### 11. `unwrap()` in the paint path
**Warning signs:** A production app crashes on a paint path
invariant that was "obvious" in the test.
**Prevention:** Every widget painter's `paint_content` and `paint`
methods must use `?` or graceful fallbacks. The current code does
this in most places; the v1 success criterion is "no `unwrap()` in
the runtime paint path under non-pathological inputs".

### 12. Bumping `wgpu` without bumping `glyphon`
**Warning signs:** New `wgpu` release changes the render-pass API;
`glyphon` was pinned to the previous API; the text atlas upload
silently fails.
**Prevention:** When bumping `wgpu`, check `glyphon`'s release
notes. The pinning in `Cargo.toml` is at `wgpu 29` / `glyphon
0.11`; treat that as a pair.

## Domain-Specific (wgpu)

- **WebGPU backend is finicky with timing.** `request_adapter` is
  async; in test environments use `pollster::block_on` and accept
  the latency cost. Production code should be `async fn` and
  surface the wait.
- **Vulkan validation layers** will flag buffer alignment issues
  that DX12 / Metal ignore. Always run the test suite with
  validation enabled in CI.
- **Surface re-creation on window resize** is platform-specific.
  Use `wgpu::Surface::configure` on resize; do not recreate the
  surface.

## Domain-Specific (Rust)

- **`Rc` vs `Arc`** — `Rc` in the layout path is fine (single-threaded
  per frame); `Arc` only for cross-thread state. The current code
  is `Rc`-clean in the hot path.
- **Send + Sync** — `WidgetPainter` is now `Send + Sync` (8.2 work).
  Verify all custom painters written by users are also `Send + Sync`.
- **Async** — the runtime is sync. The renderer is async (device
  request). Bridge with `pollster` in tests; in production expose
  async methods.

## Domain-Specific (Layout)

- **Text intrinsic sizing** — the heuristic `measure_text` is
  approximate (feedback #4.2 has a per-thread cache). Real
  shaping is slow; the cache hides the cost.
- **Flex grow / shrink cycles** — `taffy` handles these but a
  nested flex with a `MinContent` constraint can loop. Test with
  the worst case.

## Process Mistakes (made in this codebase)

- **`pub use core::*` at the crate root.** 193 items re-exported
  implicitly. Fix: `#[doc(hidden)] pub use core::*` (feedback #8.5).
- **Widget painters that fell through to `GenericPainter`.** Bug:
  Image / Switch / Slider / ProgressBar / Spinner / Badge /
  Avatar / Link / Alert / Card all rendered as a tinted rectangle
  with no foreground. Fix: dedicated painter per kind
  (feedback #2.2 part 1).
- **Hand-rolled `format!` JSON in `to_debug_json`.** Broke on
  string fields. Fix: `serde_json` (feedback #2.8).
- **Theme indirection.** `theme.widgets.X` was 2-field wrapper noise.
  Fix: `Theme::metrics` / `Theme::select` directly (feedback #3.8).

These are the kinds of "looks fine in the test, wrong in practice"
mistakes that the v1 success criteria are designed to catch.

## Phase Mapping Summary

| Pitfall | Phase |
|---------|-------|
| 1 (full-rebuild reconcile) | P0-01 |
| 2 (TextSystem::default) | Already fixed |
| 3 (background color) | Doc pass (P2-06) |
| 4 (z-index sorting) | Already fixed |
| 5 (capture leak) | P0-01 (paired with reconciliation) |
| 6 (IME) | P0-03 |
| 7 (reentrancy) | P1-01 (public API audit) |
| 8 (hit-test cache) | P2-01 (custom widget contract) |
| 9 (atlas eviction) | P2-04 (per-platform tests) |
| 10 (golden flakiness) | Already fixed (tolerance) |
| 11 (unwrap in paint) | P1-01 (public API audit) |
| 12 (wgpu/glyphon mismatch) | P2-04 (per-platform tests) |

---
*Pitfalls research for: rsgui*
*Researched: 2026-06-03*
