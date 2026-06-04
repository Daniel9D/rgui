---
status: major
phase: 04-multi-window
date: 2026-06-04
reviewer: gsd-code-reviewer
files_reviewed: 23
critical: 1
warning: 2
info: 7
---

# Phase 4 Code Review

## Summary

Phase 4 delivers the multi-window primitives coherently and the new
types / trait bounds / static assert are well-structured, but a
**D-15 feature (`SharedAccessibility`)** is half-implemented: the
`ProcessContext::with_a11y` constructor stores the backend in
`UiRuntime.a11y` but `UiRuntime::update()` never reads that field —
only the legacy `a11y_backend: Option<Box<...>>` field is called.
As a result, a host using the documented D-15 API gets silent
no-op behavior. The two event-routing integration tests are also
weak (vacuously pass), which allowed the same kind of wiring
regression to slip through.

## Findings

### Critical

- **`src/runtime/runtime.rs:1217-1220` — `SharedAccessibility`
  (D-15) is wired into `UiRuntime::for_window` (`a11y: ctx.a11y()
  .cloned()` at line 227) but never read by the runtime's update
  path.** The call site at line 1217 reads
  `self.a11y_backend.as_mut()` (a legacy `pub` `Option<Box<dyn
  AccessibilityBackend>>` field, always `None` for
  `for_window`), not the new `self.a11y: Option<SharedAccessibility>`.
  Consequence:
  - `ProcessContext::new()` (the documented default) wraps
    `SharedAccessibility::none()` in `self.a11y`, but no backend
    is ever called — `update()` falls through to the always-`None`
    `a11y_backend`. The runtime currently has *no* a11y backend
    running, even the noop.
  - `ProcessContext::with_a11y(my_backend)` is a documented
    public API (D-15). A host that uses it to wire a screen
    reader will have the backend silently ignored.
  - The test
    `process_context_with_a11y_wraps_the_backend`
    (`src/runtime/process_context.rs:126-133`) only checks the
    wrapper, not the dispatch, so the bug is not caught.
  Suggested fix: in `update()` after the
  `builder.semantics` is built, replace
  ```rust
  if let Some(backend) = self.a11y_backend.as_mut() {
      backend.update(&builder.semantics);
      self.a11y_update_count += 1;
  }
  ```
  with
  ```rust
  if let Some(shared) = self.a11y.as_mut() {
      shared.update(&builder.semantics);
      self.a11y_update_count += 1;
  }
  ```
  (or call both, with the legacy one deprecated). Add a regression
  test that drives a counting backend through
  `ProcessContext::with_a11y` and asserts `update_count` advances
  after `update()`.

### Warning

- **`tests/multi_window_event_routing.rs:69-104` —
  `pointer_click_to_a_does_not_panic_on_b` is vacuous.** The test
  sends `PointerDown`/`PointerUp` to runtime A, then calls
  `a.update(...)`, and asserts `b.command_count() == 0`. Runtime B
  never receives any events and never receives an `update()`; its
  command count is 0 by construction. To prove event isolation, B
  must also be exercised after A's events (e.g. call
  `b.update(...)` with the same tree, and assert B's command count
  is still 0 and B's per-window state is unchanged from its
  pre-A-event baseline).

- **`tests/multi_window_event_routing.rs:32-66` —
  `pointer_event_to_a_does_not_change_b_hover` is weak.** The
  assertion `hover_a != hover_b` only holds if A's hit test
  succeeds at `(10, 20)` on an 800×600 default viewport with a
  default-positioned button and B's hit test fails at `(999,
  999)`. This is a positional bet on default button layout, not
  an isolation proof. The plan asked for "B's focused node is
  unchanged" — the test should snapshot B's hover key before any
  event, dispatch the event to A only, dispatch a benign event
  to B, and assert B's hover key is `==` to its pre-event
  snapshot (or `== None` if B never had hover).

### Info / Style

- **`src/core/shared_a11y.rs:48-66` — `SharedAccessibility::update`
  silently skips when the `Arc` is shared.** The `Arc::get_mut`
  pattern is sound (and the trait rustdoc on D-18 documents the
  `Mutex<T>`-internally idiom), but the silent skip is a behavior
  change from a `Box<dyn>`-based wrapper, which always dispatches.
  A custom backend author who follows the trait doc but doesn't
  hold a `Mutex` will get no updates and no diagnostic. Consider
  logging (via `log::warn!` or `eprintln!` behind a debug flag)
  on the first skip so the surprise is at least audible.

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
  lock per frame entirely.

- **`src/render/wgpu/shared_device.rs:98-103` — a one-shot
  `PipelineCache` is built only to extract
  `bind_group_layout()` for the atlas init.** The pipeline cache
  itself is dropped on the next line. The waste is one-time
  (init), so it's fine, but the indirection is awkward. A
  `GpuAtlas::layout_only_layout()` constructor (or a
  `&bind_group_layout` argument extracted from a free function)
  would make the intent clearer.

- **`src/runtime/window_id.rs:64-84` —
  `From<winit::window::WindowId>` uses `DefaultHasher` to convert
  the opaque winit id.** The hash is non-deterministic across
  process restarts (random seed) and has a (vanishingly small)
  collision probability. For runtime identity this is fine (the
  SUMMARY acknowledges it), but the doc could note that the
  conversion is one-way — once a `WindowId` is constructed from
  a winit id, the host can't recover the winit id. That's by
  design, but worth one sentence of rustdoc.

- **`src/runtime/tree.rs:67-76` — `IdAllocator::fresh()` uses
  `Box::leak` to manufacture a `'static` allocator.** This is a
  real memory leak, bounded by the number of `Reconciler::diff`
  calls in the process lifetime. The pattern is pre-Phase 4
  (the Phase 4 change moved it from `Box::leak(0u64)` to
  `Box::leak(NodeIdAllocator::new())`), but it's still a leak.
  A `thread_local!` allocator or a `Bump`-style scoped arena
  would be cleaner. Pre-existing; not introduced by Phase 4.

- **`src/runtime/runtime.rs:174` — the legacy
  `pub a11y_backend: Option<Box<dyn AccessibilityBackend>>` field
  coexists with the new `self.a11y: Option<SharedAccessibility>`.**
  After the Critical fix above, one of them should be deprecated
  (or both kept, with one as the legacy escape hatch). The
  presence of both is confusing to readers and creates two
  semantically-overlapping seams. Recommendation: keep
  `SharedAccessibility` as the primary (D-15) and either remove
  `a11y_backend` or mark it `#[deprecated]`.

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
  pattern in `examples/widgets.rs:41`.

## Verdict

`major` — one critical finding (D-15 wiring gap) that makes the
shared accessibility feature non-functional. The fix is
mechanical (a few lines in `update()` plus a regression test) and
the rest of the phase is solid: the new public API surface
(`WindowId`, `ProcessContext`, `AppEvent`, `SharedWgpuDevice`) is
coherent, the rustdoc is meaningful, the `unsafe impl Send + Sync`
for `TaffyLayoutBackend` has a well-reasoned SAFETY block, the
D-19 static assert is in active code with a documented regression
test, and the lock pattern in `with_shared_device` is
appropriately scoped. The two weak integration tests are a
secondary concern but should be tightened to match the WIN-02/03
contracts.
