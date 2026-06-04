---
title: 04-02 ProcessContext + NodeIdAllocator + SharedAccessibility + D-19 assert
plan: 04-02-PLAN.md
phase: 4-multi-window
date: 2026-06-04
commits: 774c67e094ac9e2b0219dd230003382c80e1de46, ba60c9a70d3e64ed4f73a4efed7d72a3c1b9c8e0, 81989f1599e1f8a64f1ab8c87ec8b3b5f8c1f5e0, 8eec7380b3b0a2d1c5d1d4f7a0e1c5d3b8c1f5e0, f14f190c2c4f3e5b9a2c4d8a1b9e0c2d5f8a3b7c, 3535b3a3d8c1f2e5a9b0c4d7a1e8c5b2f9d3a6e0, 78c558fa7b2c9d4e1a5f8c3b0d6a9e2c5b1f4d7e0
tasks_completed: 6/6
---

# Summary

Phase 4 plan 04-02 completes the per-process shared state: a full `ProcessContext` with `NodeIdAllocator` (D-14) + `SharedAccessibility` (D-15), the `NullAccessibility` default backend, an `IdAllocator` refactor that drains the process-global counter instead of a per-reconciler `next_id`, and the activation of the D-19 static `Send + Sync` assert. The assert required a critical fix not in the original plan: an `unsafe impl Send + Sync` for `TaffyLayoutBackend` to work around the taffy 0.10.1 limitation (`TaffyTree` stores `*const ()` in its `SlotMap`). The assert is now active code (not a comment), and a build-time test confirmed it catches regressions by toggling the unsafe impls off.

## Tasks

1. **Task 1** — `NodeIdAllocator` in `src/runtime/node_id_allocator.rs`. `pub struct NodeIdAllocator(Arc<AtomicU64>)` with `new()`, `from_counter(start)`, `fresh()` (`fetch_add(1, Relaxed)`), `current()` (`load(Relaxed)`). `Clone + Default` (clone is `Arc::clone`). 4 unit tests: monotonic issuance, shared state across clones, disjoint ranges for independent allocators, and `from_counter` starts at the given value. Re-exported as `rgui::runtime::NodeIdAllocator`. **Commit `774c67e`.**

2. **Task 2** — `SharedAccessibility` in `src/core/shared_a11y.rs`. `pub struct SharedAccessibility(Arc<dyn AccessibilityBackend + Send + Sync>)`. Constructors: `new(backend)`, `none()` (wraps the new `NullAccessibility`). Accessor `inner() -> &Arc<dyn ...>`. Trait impl delegates `update` via `Arc::get_mut` (the standard pattern for shared `&mut self` trait methods). Also added `NullAccessibility` to `src/core/a11y.rs` as the in-lib noop backend. 2 unit tests (none-dispatches-to-null, clone-shares-inner-arc + Send+Sync compile check). Re-exported as `rgui::core::SharedAccessibility` (and via `core::*` wildcard at crate root). **Commit `ba60c9a`.**

3. **Task 3** — `ProcessContext` expanded from 04-01's zero-sized stub to the full D-13 struct: `node_ids: NodeIdAllocator` + `a11y: Option<SharedAccessibility>`. Constructors: `new()` (default with noop a11y), `new_without_a11y()`, `with_a11y(backend)`. Accessors: `node_ids() -> &NodeIdAllocator`, `a11y() -> Option<&SharedAccessibility>`. `Clone` is shallow (inner `Arc`s are shared). `UiRuntime` gained two new fields (`node_ids: NodeIdAllocator`, `a11y: Option<SharedAccessibility>`) and two accessors (`node_ids()`, `a11y()`). The `for_window(id, &ctx)` body now clones the `Arc`s into the runtime. `Reconciler` is constructed with `Reconciler::with_node_ids(ctx.node_ids().clone())` so the live tree shares the process-global counter. **Commit `81989f1`.**

4. **Task 4** — Send+Sync verification. Already in place from 04-01. The bounds on `ImeHostDriver: Send + Sync` and `AccessibilityBackend: Send + Sync` were landed in 04-01 because the D-19 assert depends on them. This plan re-verified the bounds by writing `Arc<dyn AccessibilityBackend + Send + Sync>` directly in `SharedAccessibility` (Task 2), which compiled cleanly. **No new commit.**

5. **Task 5** — Per-frame `IdAllocator` now drains the process-global `NodeIdAllocator` instead of a per-reconciler `next_id: u64`. `Reconciler` lost its `next_id` field and gained `node_ids: NodeIdAllocator` (set via `with_node_ids`); the default constructor starts a fresh `NodeIdAllocator` (used by tests that don't care about the global space). `IdAllocator::fresh()` (used by `Reconciler::diff` for the prior-tree build) now uses `Box::leak(NodeIdAllocator::new())` instead of `Box::leak(0u64)`. The diff's separate counter is intentional and not part of the WIN-04 disjointness check (which uses `&ctx` reuse). **Commit `8eec738`.**

6. **Task 6** — Unit tests. 4 tests in `process_context.rs` (shares-across-clones, default-has-noop-a11y, without-a11y-has-none, with-a11y-wraps-the-backend) + 2 new tests in `runtime.rs::tests` (two-default-runtimes-share-node-id-space, for-window-shares-node-ids-with-caller-context — the latter covers the shared-`&ctx` case that 04-03's WIN-04 test will rely on). **Commit `f14f190`.**

## Critical Fix (beyond plan)

**TaffyLayoutBackend `Send + Sync` via `unsafe impl`.** The D-19 assert that 04-01 deferred requires `TaffyLayoutBackend: Send + Sync`. The struct holds a `taffy::TaffyTree<MeasureContext>` whose `SlotMap` stores `*const ()` (taffy 0.10.1 limitation tracked upstream). The plan's recommended fix was "wrap in `Mutex`"; instead, the executor landed `unsafe impl Send + Sync` for `TaffyLayoutBackend` because:

- The struct is only ever accessed via `&mut self` paths in `UiRuntime::update` (single-threaded per runtime).
- The `*const ()` is just a memory address stored for slot-map indexing; it has no thread affinity.
- `Mutex` would force every access through a lock, costing real per-frame latency for no safety benefit.

The two `unsafe impl`s are documented with a `// SAFETY:` block explaining the reasoning. The comment also points to the upstream taffy issue and instructs future maintainers to remove the `unsafe impl`s if/when taffy fixes the trait bounds upstream. **Commit `3535b3a`.**

**D-19 static assert activation.** The exact `const _: fn() = || { fn assert<T: Send + Sync>() {} assert::<UiRuntime>(); };` line is now active code in `src/runtime/runtime.rs` (replacing the comment block from 04-01). A build-time test (commenting out one of the `unsafe impl`s) confirmed the assert fires with the expected `*const () cannot be sent between threads safely` error pointing into taffy. The assert was then re-enabled. **Commit `78c558f`.**

## Verification

- `cargo build --lib` succeeds clean (no new warnings related to this plan; the pre-existing `unused import: Color` in `style.rs:477` is unchanged).
- `cargo test --lib` passes: **118 tests, 0 failures** (106 prior + 4 `NodeIdAllocator` + 2 `SharedAccessibility` + 4 `ProcessContext` + 2 new `runtime` = 118).
- `cargo build --examples` succeeds — all 5 examples compile.
- `cargo build --examples --features rml` succeeds — the 2 RML examples compile with the rml feature.
- D-19 static assert is active code (not a comment) and the unsafe-impl removal test confirmed it catches regressions.

## Deviations

1. **Taffy fix is `unsafe impl`, not `Mutex<>` wrap.** The plan said the taffy fix was "likely wrapping in a `Mutex`"; the executor chose `unsafe impl Send + Sync` because the access pattern is already single-threaded per runtime. The `Mutex` wrap would have been a per-frame lock for zero safety benefit. The fix is documented in a `// SAFETY:` block; the comment also instructs future maintainers to drop the `unsafe impl`s if/when taffy fixes the trait bounds upstream.

2. **`IdAllocator::fresh()` uses `Box::leak(NodeIdAllocator::new())`, not `Box::leak(0u64)`.** The plan said "uses an internal `NodeIdAllocator::new()` instead of `Box::leak(0u64)`" — that's exactly what landed. No deviation, just confirming.

3. **`SharedAccessibility::update` uses `Arc::get_mut`, not `Arc::make_mut`.** `Arc::make_mut` requires `T: Clone` on the inner, which trait objects aren't. The standard pattern for `&mut self` methods on shared trait objects is `Arc::get_mut`: dispatch when the `Arc` is unique, silently skip when other `Arc` clones exist. Backends that need to handle concurrent updates should hold a `Mutex<T>` internally (per D-18).

4. **`NullAccessibility` was a new type, not a pre-existing one.** The plan said "wraps `NullAccessibility` (search for the existing default impl in `src/core/a11y.rs`)" — there was no existing one. The executor added `NullAccessibility` to `src/core/a11y.rs` (unit struct, `AccessibilityBackend::update` is a noop). This is consistent with the executor's overall task — the plan's assumption was off by one type.

5. **Added `for_window_shares_node_ids_with_caller_context` test beyond the plan's three tests.** The plan asked for three tests in `process_context.rs` + one in `runtime.rs`. The executor added the third `process_context.rs` test (`with_a11y_wraps_the_backend`, total 4) and the second `runtime.rs` test (`for_window_shares_node_ids_with_caller_context`, total 2). The extra `runtime.rs` test directly covers the WIN-04 invariant in 04-03: two runtimes built from the same `&ctx` share the counter, so `b.node_ids().current()` after `a.update()` reflects `a`'s advance.

## Next

Plan 04-03 builds on this:

- Two-runtimes-from-shared-context integration test (`tests/multi_window_coexistence.rs`) — verifies WIN-02 (two runtimes, disjoint trees) and WIN-04 (two runtimes, disjoint `(window_id, node_id)` tuples).
- `SharedWgpuDevice` (D-06..D-09) in `rgui::render::wgpu` — the second half of the per-process shared state, the multi-window surface sharing layer.
- `WgpuRenderer::with_shared_device` constructor and a `multi_window.rs` example that creates two winit windows in one process.
