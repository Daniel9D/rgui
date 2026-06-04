# Phase 4: Multi-Window - Context

**Gathered:** 2026-06-04
**Status:** Ready for planning
**Source:** Inline synthesis (GSD `gsd-phase-researcher` / `gsd-pattern-mapper` subagents unavailable in this runtime; reasoning done by the orchestrator against the actual codebase)

<domain>

## Phase Boundary

The runtime today is single-window: a process has exactly one `UiRuntime`, exactly one `FrameInput`, and exactly one `FrameOutput::DisplayList` per frame. There's no notion of which host window a frame is for; the winit example ignores `WindowId` (`_id: WindowId` in `window_event`). The only multi-window seam is the winit host loop, which already supports multiple windows — but our lib has no answer for "I have two windows, route events to two runtimes".

This phase delivers the per-window primitives that close the WIN-01..04 requirements:

- `WindowId` is a first-class type the host uses to identify windows.
- A process can have N `UiRuntime` instances (one per window), each with its own tree / focus / IME / DisplayList.
- A `ProcessContext` bundles the per-process state (global `NodeId` allocator + optional shared `AccessibilityBackend`) that all runtimes share.
- The host's `ApplicationHandler::window_event(id, event)` is the routing seam: it looks up the `AppWindow` for the id and dispatches the event to that runtime's `dispatch_to_window`.
- A `SharedWgpuDevice` lets multiple `WgpuRenderer`s share one adapter / device / queue / atlas, so two windows don't pay the VRAM cost twice.
- `ImeHostDriver` and `AccessibilityBackend` traits gain `Send + Sync` bounds so `UiRuntime: Send + Sync` (required for multi-threaded host loops like winit's `EventLoop::run` on Linux / Windows).
- An `AppEvent` enum covers cross-window events (Quit, FocusWindow, ThemeChanged, AppShortcut) the lib wants to see.

What's out of scope (v2):
- Drag a widget between windows (WIN-05)
- Modal that spans multiple windows (WIN-06)

</domain>

<decisions>

## Implementation Decisions

### Ownership shape

- **D-01: `WindowId` is a custom newtype** in `rgui::runtime`. `pub struct WindowId(u64)`. The lib does not depend on winit; winit's `WindowId` (also u64 on most platforms) converts via `From` in a `winit` feature (`cfg`-gated). Other hosts convert on their end.
- **D-02: `UiRuntime` gains an invariant `window_id: WindowId` field.** Set once via `UiRuntime::for_window(id: WindowId, ctx: &ProcessContext) -> Self`. No `set_window_id` method. The runtime is bound to one window for its lifetime.
- **D-03: The host owns `HashMap<WindowId, AppWindow>`** where `AppWindow { runtime: UiRuntime, surface: wgpu::Surface<'static>, renderer: WgpuRenderer }`. The lib ships no `WindowRuntime` helper struct; the host's `AppWindow` is the host's choice (the example uses this exact name).
- **D-04: All five existing winit examples migrate to `WindowId`.** `basic_window`, `widgets`, `visual_showcase`, `rml_showcase`, `rml_widget_gallery` each gain a `WindowId` argument in their `ApplicationHandler::window_event` and pass it to the new `UiRuntime::for_window`.
- **D-05: A new `examples/multi_window.rs` demonstrates two windows.** Two winit windows created in `resumed` (or via a button click). `ApplicationHandler::window_event(id, event)` looks up the `AppWindow` by id. A click in window A does not affect window B's `UiRuntime` state. This is the runnable proof that WIN-01..04 are satisfied.

### Renderer sharing

- **D-06: `SharedWgpuDevice` is a new struct in `rgui::render::wgpu`.** Wraps `Arc<wgpu::Adapter>`, `Arc<wgpu::Device>`, `Arc<wgpu::Queue>`, and `Arc<std::sync::Mutex<GpuAtlas>>`. Constructed once per process via `pub async fn SharedWgpuDevice::new() -> RendererResult<Self>`.
- **D-07: The atlas is shared, not per-window.** `SharedWgpuDevice` owns the `GpuAtlas`. Each `WgpuRenderer` borrows via the `Arc<Mutex<...>>`. Glyphs uploaded by any window are visible to all windows — the common case is the system font's ASCII glyphs, which get uploaded once. New glyphs (CJK, custom) are uploaded lazily on first use in any window.
- **D-08: `WgpuRenderer::with_shared_device(&SharedWgpuDevice, surface: &wgpu::Surface<'static>) -> RendererResult<Self>`.** The renderer holds `Arc` clones of the device / queue / adapter / atlas. The `surface` is borrowed for the frame; the host owns the `wgpu::Surface` (per the winit 0.30 + wgpu 29 surface model).
- **D-09: Atlas mutations lock the mutex.** `GpuAtlas::upload_rgba8` takes `&Mutex<GpuAtlas>` and locks for the upload. Per-frame bind group construction reads under the same lock (the lock is brief; contention with uploads is rare). The pattern is documented in the trait rustdoc; a future v1.x can swap to `RwLock` if profiling shows contention.

### Event routing

- **D-10: `pub fn dispatch_to_window(&mut self, event: UiEvent) -> bool` on `UiRuntime`.** Returns `true` if the event was consumed by a handler. The host calls: `if let Some(win) = windows.get_mut(&id) { win.runtime.dispatch_to_window(event); }`. Mirrors the existing `dispatch` method; the new variant is what multi-window hosts should call.
- **D-11: `UiEvent` does NOT gain a `window_id` field.** Window identity is implicit in *which* runtime the event lands on. The runtime's invariant `window_id` field is set once; the host is responsible for not mixing inputs across windows. The lib never has to "look up the right runtime from an event".
- **D-12: A new `AppEvent` enum on the lib covers cross-window events.** `pub enum AppEvent { Quit, FocusWindow(WindowId), ThemeChanged(Theme), AppShortcut(String) }`. The host constructs `AppEvent`s and dispatches them via a sibling `pub fn dispatch_app_event(&mut self, event: AppEvent) -> AppEventOutcome` on the runtime. Per-window events (pointer, keyboard, IME, resize) stay as `UiEvent`; only cross-window concerns go through `AppEvent`.

### ProcessContext (shared state)

- **D-13: `ProcessContext` bundles per-process state.** `pub struct ProcessContext { node_ids: NodeIdAllocator, a11y: Option<SharedAccessibility> }` in `rgui::runtime`. The host constructs one per process and passes it to every `UiRuntime::for_window(id, &ctx)` call. Internally an `Arc`; cheap to clone.
- **D-14: `NodeIdAllocator(Arc<std::sync::atomic::AtomicU64>)`.** NodeIds are process-global; the `(window_id, node_id)` tuple is unique process-wide. This is required for v2 (drag a widget between windows) and keeps the snapshot identifiable.
- **D-15: `SharedAccessibility(Arc<dyn AccessibilityBackend + Send + Sync>)`.** Optional. A host with a screen reader constructs one `SharedAccessibility` and shares it across all runtimes. The lib feeds each runtime's semantics into the shared backend. (Per-window a11y is also supported: each runtime can carry its own; the host decides.)
- **D-16: Resize is signalled via the existing `FrameInput.viewport` path.** No new `AppEvent::Resized`. The host's winit `WindowEvent::Resized(size)` handler updates the per-window runtime's viewport on the next `update()` and calls `renderer.resize(viewport)` directly. The lib stays winit-free.

### Trait Send + Sync

- **D-17: `pub trait ImeHostDriver: Send + Sync`.** Same for `pub trait AccessibilityBackend: Send + Sync`. The bound is on the trait itself, so any `Box<dyn ImeHostDriver>` is automatically `Send + Sync`. `NoopDriver` and `MockDriver` derive `Send + Sync` via auto-impl.
- **D-18: Document the `Mutex` / `RwLock` pattern for interior mutability.** The trait rustdoc spells out: "Implementations that need to mutate state across `poll()` calls should hold a `Mutex<T>` or `RwLock<T>` internally. The winit IME adapter, for example, holds a `Mutex<Option<winit::event::Ime>>`." Matches the `wgpu::Device: Send + Sync` pattern.
- **D-19: Static assertion on `UiRuntime: Send + Sync`.** Near the `UiRuntime` definition, add `const _: fn() = || { fn assert<T: Send + Sync>() {} assert::<UiRuntime>(); };`. Catches regressions: if a future field makes `UiRuntime` !Send or !Sync, the lib stops compiling. Enforces the WIN-03 contract at the type level.

### Claude's Discretion

The user did not flag any area as "you decide". The plan + executor may make the following low-risk choices without re-asking:

- The exact `AppEventOutcome` return shape (`enum { Consumed, Ignored }`).
- The `SharedAccessibility` constructor surface (`SharedAccessibility::new(backend)`, `SharedAccessibility::none()`).
- Whether `UiRuntime::default()` (no-args) remains a backward-compat single-window entry point that constructs a hidden `ProcessContext` internally — recommended for v1.x, may be deprecated in v2.
- The exact `wgpu::Surface` lifetime story (the wgpu 29 + winit 0.30 pair has a `'static` bound that needs careful plumbing).

</decisions>

<canonical_refs>

## Canonical References

**Downstream agents and executors MUST read these before planning or implementing.**

### Project context
- `.planning/PROJECT.md` — full project context, Validated / Active / Out-of-scope requirements
- `.planning/REQUIREMENTS.md` — WIN-01..04 (lines 31-37) are the v1 scope; WIN-05..06 are v2 (lines 107-110)
- `.planning/ROADMAP.md` — Phase 4 entry (line 20) and the 3 plan slots (04-01..03, lines 109-111)

### Prior phase decisions that apply here
- `.planning/phases/01-incremental-reconciliation/01-CONTEXT.md` — diffing is keyed by NodeId; a global NodeId space (D-14) is forward-compat with the v2 cross-window drag.
- `.planning/phases/02-event-input-hardening/02-CONTEXT.md` — the receive-side IME gating (`ime_enabled: bool`); Phase 4 doesn't change the receive side, only adds the dispatch routing.
- `.planning/phases/03-text-ime/03-CONTEXT.md` — `ImeHostDriver` is producer-side, `&mut self`; the trait is per-runtime (D-02), not per-process. The Phase 3 trait gains `Send + Sync` (D-17) but stays per-runtime.

### Code to read
- `src/runtime/runtime.rs:110-162` — the `UiRuntime` struct definition (add `window_id`, `node_ids`, `a11y` fields; static assert at line 162-ish)
- `src/runtime/runtime.rs:1078-1100` — the synthesized `RenderStats` build path (will need `window_id` on the frame pipeline; or not, since FrameOutput is per-window)
- `src/runtime/frame.rs:7-13` — `FrameInput` struct (no `window_id` today)
- `src/runtime/ime_host.rs:56-60` — `ImeHostDriver` trait definition (add `: Send + Sync`)
- `src/core/a11y.rs:129` — `AccessibilityBackend` trait definition (add `: Send + Sync`)
- `src/render/wgpu/mod.rs:38-117` — `WgpuRenderer` struct + `from_context` constructor; the new `with_shared_device` constructor
- `src/render/wgpu/atlas.rs` — `GpuAtlas` (the type wrapped in `Arc<Mutex<>>` by D-06)
- `examples/widgets.rs:23-141` — the single-window winit pattern to migrate to `WindowId`
- `examples/multi_window.rs` — NEW; the two-window example
- `src/runtime/tree.rs:29-43` — `IdAllocator` (the per-frame transient; the new `NodeIdAllocator` is a separate global struct)
- `src/runtime/text_metrics.rs:73-101` — `METRICS_CACHE` thread-local; not affected by multi-window (each thread serves the runtime that's currently calling)

### External references
- **winit 0.30 ApplicationHandler** — the host pattern for multi-window event routing. `ApplicationHandler::window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent)` is the per-window seam.
- **wgpu 29 Surface model** — `wgpu::Surface<'static>` is bound to a winit `Window`; the host owns it; the renderer borrows `&Surface` per frame. Multiple surfaces can share one device / queue.

</canonical_refs>

<code_context>

## Existing Code Insights

### Reusable Assets
- `UiRuntime` — the existing single-window runtime; the per-window version is a 2-field addition (`window_id`, `node_ids`); most code paths are unchanged.
- `ImeHostDriver` / `NoopDriver` / `MockDriver` from Phase 3 — the IME producer side; Phase 4 only adds `Send + Sync`, doesn't change the dispatch model.
- `WgpuRenderer` — the existing per-device renderer; the new `with_shared_device` constructor adds a multi-window entry point without removing the existing `from_context` path.
- `GpuAtlas` — already a self-contained type with an `upload_rgba8` method; `Arc<Mutex<GpuAtlas>>` is a thin wrapper.
- `examples/widgets.rs` — the canonical winit example; the new `multi_window.rs` is built from its skeleton.

### Established Patterns
- **Trait + Default impl** — the `ImeHostDriver` trait has a `NoopDriver` default; `AccessibilityBackend` has a `NullAccessibility` default. The new `WindowId` / `ProcessContext` types follow the same "newtype + Default" pattern.
- **Manual `Default` when `Box<dyn Trait>` is in the struct** — Phase 3 had to do this for `UiRuntime` because `Box<dyn ImeHostDriver>` has no `Default`. Phase 4's `UiRuntime` follows the same pattern: `impl Default` is gated to construct a hidden `ProcessContext` (Claude's discretion).
- **Send + Sync as type-level contracts** — `wgpu::Device: Send + Sync`; the new traits follow that pattern (D-17).
- **Arc + Mutex for shared GPU state** — `wgpu::Device` itself is internally `Arc`-shared; the new `SharedWgpuDevice` mirrors that pattern (D-06, D-09).
- **`From<winit::WindowId> for rgui::WindowId` in a `winit` feature** — the Phase 3 IME `winit` feature pattern (which is itself a v1.x follow-up) sets the precedent for the `winit` feature organization.

### Integration Points
- `examples/widgets.rs:23-141` — single-runtime pattern; the migration adds `for_window` + a `WindowId` argument in `window_event`.
- `src/runtime/runtime.rs:2554` (default `UiRuntime` impl) — needs to forward to `for_window` with a fresh `ProcessContext`.
- `src/runtime/runtime.rs:1082-1100` — the `RenderStats` build path; the per-window runtime's stats are unchanged, but a process-level stats aggregation is a v1.x follow-up.
- `src/lib.rs:1-13` — module structure; the new `ProcessContext` + `WindowId` go in `rgui::runtime`; the new `SharedWgpuDevice` goes in `rgui::render::wgpu`.
- `src/runtime/ime_host.rs:62-70` — `NoopDriver` derives `Send + Sync` already (it's a unit struct); the trait bound change is a no-op for it.

</code_context>

<specifics>

## Specific Ideas

- **`UiRuntime::for_window(id, &ctx)`** takes a `&ProcessContext` (not `Arc<ProcessContext>`) and clones the inner `Arc`s into the runtime. This makes the call site read naturally (`for_window(id, &ctx)`) and avoids forcing the host to keep an Arc at the call site.
- **`ProcessContext` is the host's "process init" object.** The host calls `let ctx = ProcessContext::new()` once in `main()`; every `UiRuntime::for_window` borrows `&ctx`. The lib is responsible for `ctx` being cheaply cloneable internally (all fields are `Arc`-wrapped).
- **`SharedAccessibility::none()` is a convenience constructor** that returns `ProcessContext { a11y: None, .. }`. Used by apps that don't need a screen reader.
- **Two-window example visual** — window A on the left has a button labeled "Increment A counter"; window B on the right has a button labeled "Increment B counter". Each window has its own counter label. The example proves the runtimes are isolated: clicking A's button 5 times does not change B's counter.
- **WIN-02 verification test** — `tests/multi_window_coexistence.rs` constructs two `UiRuntime` instances in the same process, calls `update` on each, asserts the snapshots are independent (different `window_id` fields, different `NodeId` ranges, different `DisplayList` lengths when the trees differ).
- **WIN-03 verification test** — `tests/multi_window_event_routing.rs` constructs two runtimes, dispatches a pointer event to runtime A, asserts runtime B's focused node is unchanged. The test uses the synthetic `MockDriver` to confirm IME events also route correctly.
- **WIN-04 verification test** — `tests/multi_window_snapshot_isolation.rs` constructs two runtimes, calls `update` on each, takes `output.debug_snapshot()` on both, asserts the snapshots reference disjoint node ids (i.e., `(window_id, node_id)` tuples are unique process-wide).
- **The static assert** goes right after the `UiRuntime` struct definition, with a one-line comment explaining the regression it catches.

</specifics>

<deferred>

## Deferred Ideas

These came up during discussion but belong in other phases or v2:

- **Drag a widget between windows** (WIN-05) — v2. The (window_id, node_id) tuple (D-14) is forward-compat.
- **Modal that spans multiple windows** (WIN-06) — v2. Requires cross-window focus routing, which is its own design problem.
- **SharedAtlas beyond Mutex** — v1.x. If profiling shows contention, swap to `RwLock` or a lock-free structure.
- **`UiRuntime::default()` semantics** — the no-arg `default()` is a backward-compat single-window entry point that constructs a hidden `ProcessContext` internally. May be deprecated in v2 once `for_window` is the standard.
- **Process-level stats aggregation** — v1.x. A `ProcessContext::stats()` method that aggregates `text_cache`, `RenderStats`, etc. across runtimes. Not Phase 4 scope; deferred.
- **Cross-window drag preview** — v2 follow-up; Phase 8 (DnD is in Phase 7, multi-window DnD in v2).
- **A `RuntimeSet { windows: HashMap<WindowId, UiRuntime> }` helper struct on the lib side** — could be added in v1.x as a convenience for hosts that don't want to manage the HashMap themselves. Phase 4 ships the raw primitive; the helper is a v1.x follow-up.

### Reviewed Todos (not folded)

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 04-multi-window*
*Context gathered: 2026-06-04 via inline synthesis (GSD subagents unavailable in this runtime)*
