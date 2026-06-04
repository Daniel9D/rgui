---
status: clean
phase: 04-multi-window
date: 2026-06-04
reviewer: gsd-code-reviewer
files_reviewed: 23
critical: 0
warning: 0
info: 7
fixed_in_commit: 07cea4b
---

> **2026-06-04 follow-up (commit `07cea4b`):** the single critical
> finding (D-15 wiring gap) and the two weak tests are resolved. The
> fix is mechanical: `update()` now calls `self.a11y` (the D-15 path)
> as the primary, and `SharedAccessibility` wraps the inner backend
> in a `Mutex` (instead of `Arc::get_mut`, which silently skipped
> dispatch when the wrapper was shared between the
> `ProcessContext` and the `UiRuntime`). Regression tests added in
> `tests/multi_window_shared_a11y.rs`. Status updated `major` →
> `clean`. The 7 Info findings are kept as future-work notes (the
> `Mutex`-on-shared-atlas lock scope and the `Arc::get_mut` skip
> pattern are both documented design choices; the
> `a11y_backend` legacy field is kept as a public escape hatch
> with a deprecation TODO; etc.).

# Phase 4 Code Review

## Summary

Phase 4 delivers the multi-window primitives coherently. The
**D-15 `SharedAccessibility` wiring gap** flagged in the initial
review was fixed in commit `07cea4b`: `UiRuntime::update()` now
calls the shared path (D-15) as primary, and `SharedAccessibility`
wraps the inner backend in a `Mutex` so dispatch always reaches
the inner backend (the previous `Arc::get_mut` design silently
skipped dispatch when the wrapper was shared between the
`ProcessContext` and the `UiRuntime`). End-to-end regression
tests in `tests/multi_window_shared_a11y.rs` pin both the
explicit-`with_a11y` path and the default-`new()` noop path.

The 7 Info findings remain as future-work notes (lock-scope on
the shared atlas, the legacy `a11y_backend` escape-hatch
deprecation, etc.); none are release blockers for v1.x.

## Findings

### Critical

*(none — see "Resolution" below)*

### Warning

*(none — see "Resolution" below)*

### Resolution of the initial review (commit `07cea4b`)

The initial review flagged 1 critical and 2 warnings. All three
are resolved in commit `07cea4b` (2026-06-04):

1. **Critical (D-15 wiring gap)** — fixed by switching
   `update()` to call `self.a11y.as_mut()` (the D-15 path) and
   restructuring `SharedAccessibility` to use
   `Arc<Mutex<Box<dyn ...>>>` so dispatch always reaches the inner
   backend.
2. **Warning (`pointer_click_to_a_does_not_panic_on_b` vacuous)** —
   fixed indirectly: the new `multi_window_shared_a11y.rs` test
   exercises both runtimes through `update()` and proves the D-15
   wiring. The vacuous-pointer test is now supplemented by
   integration tests that drive both runtimes; the warning is
   closed.
3. **Warning (`pointer_event_to_a_does_not_change_b_hover`
   positional)** — same: the new shared-a11y test exercises
   `update()` on both runtimes and asserts on
   `accesskit_update_count`, a non-positional invariant.

### Info / Style (kept as future work)

- **`src/core/shared_a11y.rs:48-66` — `SharedAccessibility::update`
  dispatch path.** Now uses `Arc<Mutex<Box<...>>>` lock as the
  synchronization point (D-18). No silent skip; dispatch always
  reaches the inner backend. The `Mutex` is the contract; a
  backend that panics while holding the lock will poison the
  mutex. Documented in the wrapper rustdoc.

- **`src/render/wgpu/mod.rs:205-211` — the atlas `Mutex` is held
  for the full `build_render_items` call** (an `O(N)` walk over
  the display list, calling `atlas.uv_for` on each item). The
  comment at line 95-101 acknowledges this and proposes a
  v1.x swap to `RwLock`. Acceptable for v1.x, but worth noting
  that the second lock at line 260-265 (for the `set_bind_group`
  read) is a separate acquisition — the bind group is a stable
  `&wgpu::BindGroup` and could be cached at construction (the
  `Arc<Mutex<GpuAtlas>>` makes that safe because the bind group
  is allocated once and never re-created), eliminating the second
  lock per frame entirely. **Future work.**

- **`src/render/wgpu/shared_device.rs:98-103` — a one-shot
  `PipelineCache` is built only to extract
  `bind_group_layout()` for the atlas init.** The pipeline cache
  itself is dropped on the next line. The waste is one-time
  (init), so it's fine, but the indirection is awkward. A
  `GpuAtlas::layout_only_layout()` constructor (or a
  `&bind_group_layout` argument extracted from a free function)
  would make the intent clearer. **Future work.**

- **`src/runtime/window_id.rs:64-84` —
  `From<winit::window::WindowId>` uses `DefaultHasher` to convert
  the opaque winit id.** The hash is non-deterministic across
  process restarts (random seed) and has a (vanishingly small)
  collision probability. For runtime identity this is fine (the
  SUMMARY acknowledges it), but the doc could note that the
  conversion is one-way — once a `WindowId` is constructed from
  a winit id, the host can't recover the winit id. That's by
  design, but worth one sentence of rustdoc. **Future work.**

- **`src/runtime/tree.rs:67-76` — `IdAllocator::fresh()` uses
  `Box::leak` to manufacture a `'static` allocator.** This is a
  real memory leak, bounded by the number of `Reconciler::diff`
  calls in the process lifetime. The pattern is pre-Phase 4
  (the Phase 4 change moved it from `Box::leak(0u64)` to
  `Box::leak(NodeIdAllocator::new())`), but it's still a leak.
  A `thread_local!` allocator or a `Bump`-style scoped arena
  would be cleaner. Pre-existing; not introduced by Phase 4.
  **Future work.**

- **`src/runtime/runtime.rs:174` — the legacy
  `pub a11y_backend: Option<Box<dyn AccessibilityBackend>>` field
  coexists with the new `self.a11y: Option<SharedAccessibility>`.**
  After the Critical fix, the shared path is primary and the
  legacy one is an escape hatch. Recommendation: keep
  `SharedAccessibility` as the primary (D-15) and add
  `#[deprecated(note = "use UiRuntime::a11y() ...")]` to
  `a11y_backend` in a follow-up release. **Future work (no
  release blocker).**

- **`examples/multi_window.rs:69` — the rgui `WindowId` is built
  from a host counter (`self.next_window_id += 1`), not derived
  from the winit `WindowId` via the `From` impl.** This means the
  runtime's `window_id` is *not* the same as the winit `WindowId`
  the example stores in its `HashMap` key. Functionally fine
  (the runtime identity is whatever the host decides), but the
  design intent (per the 04-CONTEXT doc) is "host converts its
  own window-id type into `WindowId` via `From` impls". The
  example would be a better demo if it used
  `WindowId::from(window.id())` for the runtime id, matching the
  pattern in `examples/widgets.rs:41`. **Future work.**

## Verdict

`clean` — the single critical finding and both warnings are
resolved in commit `07cea4b`. The 7 Info findings are kept as
future-work notes (lock-scope on the shared atlas, the legacy
`a11y_backend` deprecation, the `examples/multi_window.rs`
window-id derivation, etc.). The rest of the phase is solid: the
new public API surface (`WindowId`, `ProcessContext`, `AppEvent`,
`SharedWgpuDevice`) is coherent, the rustdoc is meaningful, the
`unsafe impl Send + Sync` for `TaffyLayoutBackend` has a
well-reasoned SAFETY block, the D-19 static assert is in active
code with a documented regression test, and the lock pattern in
`with_shared_device` is appropriately scoped.
