# Phase 4: Multi-Window - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-04
**Phase:** 04-multi-window
**Areas discussed:** Ownership shape, Renderer sharing, Event routing, Send+Sync, Cross-cutting concerns (NodeId / Resize / A11y), Shared primitives

---

## Area 1: Multi-runtime ownership shape

| Option | Description | Selected |
|--------|-------------|----------|
| Per-window UiRuntime | Host owns a HashMap<WindowId, UiRuntime>. Each window has its own UiRuntime. Minimal lib surface change. | ✓ |
| Single UiRuntime + WindowSet | One process-level UiRuntime with internal Vec<WindowRuntime>. Bigger refactor. | |
| Both (UiRuntime = single window, UiRegistry = multi) | UiRegistry { windows: HashMap<WindowId, UiRuntime> }. Most expressive; bigger surface. | |

**User's choice:** Per-window UiRuntime (recommended).
**Notes:** Satisfies WIN-02 literally.

### Sub-question: where does window_id live?

| Option | Description | Selected |
|--------|-------------|----------|
| window_id on UiRuntime field | Set via for_window; every FrameInput stamped with the same id. | ✓ |
| FrameInput.window_id only | UiRuntime stays window-id-less. | |
| Both, with sanity check | Option + FrameInput mismatch returns an error. | |

### Sub-question: WindowId type

| Option | Description | Selected |
|--------|-------------|----------|
| Custom newtype (pub struct WindowId(u64)) | winit converts via a winit feature. Lib never depends on winit. | ✓ |
| Re-export winit's WindowId | Hard dependency on winit. Bloats the lib. | |
| Generic over HostId trait | More flexible; overkill. | |

### Sub-question: multi-window example structure

| Option | Description | Selected |
|--------|-------------|----------|
| Two windows, one tree each | examples/multi_window.rs with HashMap<WindowId, AppWindow>. | ✓ |
| No example; integration tests only | tests/multi_window.rs only. | |
| Both example + tests | Example + tests. | |

### Sub-question: existing winit example migration

| Option | Description | Selected |
|--------|-------------|----------|
| Migrate all winit examples to WindowId | All 5 examples gain WindowId. | ✓ |
| Defer example migration to Phase 8 | Keep existing examples as-is. | |
| Add WindowId but keep examples unchanged | No migration tax. | |

### Sub-question: WindowId invariant

| Option | Description | Selected |
|--------|-------------|----------|
| Invariant (set once via for_window) | No set_window_id method. | ✓ |
| Mutable setter | set_window_id for test reuse. | |

---

## Area 2: Per-window renderer sharing

| Option | Description | Selected |
|--------|-------------|----------|
| Shared device + per-window surface | SharedWgpuDevice primitive; multiple WgpuRenderers share device/queue/atlas. | ✓ |
| One renderer per window | Simpler, no device-sharing; more VRAM. | |
| Document one-per-window now, defer sharing | Trait boundary abstracted; future SharedDeviceRenderer. | |

**User's choice:** Shared device + per-window surface.

### Sub-question: device primitive

| Option | Description | Selected |
|--------|-------------|----------|
| SharedWgpuDevice struct | Wraps Arc<Device>, Arc<Queue>, Arc<Adapter>, Arc<Mutex<GpuAtlas>>. | ✓ |
| Pass &Device and &Queue around | Host owns the device; renderer borrows. | |
| Service registry pattern | GpuServices trait; extensible. | |

### Sub-question: atlas location

| Option | Description | Selected |
|--------|-------------|----------|
| Per-window atlas | Each window has its own GpuAtlas. | |
| Shared atlas on the device | SharedWgpuDevice owns the atlas; each renderer borrows. | ✓ |

### Sub-question: surface ownership

| Option | Description | Selected |
|--------|-------------|----------|
| Surface lives in the host, renderer borrows | Standard wgpu 29 + winit 0.30 pattern. | ✓ |
| Renderer owns the surface | Couples WgpuRenderer to winit. | |

### Sub-question: atlas sync

| Option | Description | Selected |
|--------|-------------|----------|
| Arc<Mutex<GpuAtlas>> | One lock per upload. | ✓ |
| Lock-free via RwLock | Per-frame bind group reads don't block. | |
| Lock-free via lock-free DS | Most performant; probably wrong tradeoff. | |

### Sub-question: device construction

| Option | Description | Selected |
|--------|-------------|----------|
| Async constructor | new() returns Future<RendererResult<SharedWgpuDevice>>. | ✓ |
| Sync constructor with pollster | new_sync() uses pollster::block_on. | |
| Both async and sync | Two methods. | |

---

## Area 3: Event routing

| Option | Description | Selected |
|--------|-------------|----------|
| Host filters by WindowId | Lib stays winit-free; host matches WindowId to runtime. | ✓ |
| Runtime subscribes via trait | HostEventBus trait; runtime pulls. | |
| Per-window event queue inside the lib | WindowEventQueue<Vec<UiEvent>>; runtime drains its own. | |

**User's choice:** Host filters by WindowId (recommended).

### Sub-question: host data shape

| Option | Description | Selected |
|--------|-------------|----------|
| HashMap<WindowId, AppWindow> | Host owns the map; AppWindow = { runtime, surface, renderer }. | ✓ |
| Per-window WindowRuntime helper | Lib bundles; host just stores. | |
| UI stack of runtimes | RuntimeSet { windows: HashMap<WindowId, UiRuntime> } on the lib. | |

### Sub-question: event dispatch

| Option | Description | Selected |
|--------|-------------|----------|
| dispatch_to_window on UiRuntime | pub fn dispatch_to_window(&mut self, event: UiEvent) -> bool. | ✓ |
| Stamped events: window_id in UiEvent | Event carries window_id; runtime filters. | |
| Per-window event queue, batched | Vec<UiEvent> queue per window. | |

### Sub-question: cross-window events

| Option | Description | Selected |
|--------|-------------|----------|
| AppEvent enum on the runtime | Quit, FocusWindow, ThemeChanged, AppShortcut. | ✓ |
| Host fans out, runtime sees only its own events | Cross-window handled entirely by host. | |
| Host-owned, no lib surface | No AppEvent; lib never sees cross-window events. | |

### Sub-question: AppEvent surface scope

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal: Quit + FocusWindow | Smallest surface. | |
| Broader: + Theme + AppShortcut | Lets the lib see theme + global-shortcut events. | ✓ |
| Minimal + user-extension hook | Custom(u32) variant for app-specific. | |

---

## Area 4: Trait Send + Sync

| Option | Description | Selected |
|--------|-------------|----------|
| Add Send + Sync to both traits | pub trait ImeHostDriver: Send + Sync. Most ergonomic. | ✓ |
| Wrap in Mutex<dyn Trait> in UiRuntime | UiRuntime: Send + Sync without changing the trait. | |
| Document single-threaded; defer | WIN-02 satisfied sequentially in one thread. | |

**User's choice:** Add Send + Sync to both traits (recommended).

### Sub-question: trait bound shape

| Option | Description | Selected |
|--------|-------------|----------|
| pub trait ImeHostDriver: Send + Sync | Bound on the trait itself. | ✓ |
| Box<dyn Trait + Send + Sync> | Bounds at use site. | |

### Sub-question: driver interior mutability

| Option | Description | Selected |
|--------|-------------|----------|
| Document Mutex / RwLock pattern | Trait rustdoc explains the pattern. | ✓ |
| Lock-free AtomicU64 for hot fields | Reduces mutex pressure. | |
| Send + Sync but no interior mutability guarantees | Document but don't enforce. | |

### Sub-question: static assert

| Option | Description | Selected |
|--------|-------------|----------|
| const _: () = { ... } assertion | Compile-time check. | ✓ |
| Test-only assert | Only fails in cargo test. | |
| Document only, no assertion | Risk of regression. | |

---

## Area 5: Cross-cutting concerns

### Sub-question: NodeId uniqueness

| Option | Description | Selected |
|--------|-------------|----------|
| Per-window NodeId is unique | Runtime-local counter. (window_id, node_id) disambiguates. | |
| Global NodeId across windows | Arc<AtomicU64>; (window_id, node_id) always unique process-wide. | ✓ |

### Sub-question: resize signal

| Option | Description | Selected |
|--------|-------------|----------|
| Existing FrameInput.viewport path | Host updates viewport; renderer.resize(viewport) directly. | ✓ |
| Dedicated Resized event | New AppEvent::Resized(WindowId, SizeU32). | |

### Sub-question: a11y scope

| Option | Description | Selected |
|--------|-------------|----------|
| Per-window a11y backend | Each UiRuntime has its own. | |
| Single a11y backend per process | All runtimes feed one shared backend. | ✓ |
| Per-window by default, opt into shared | Most flexible. | |

---

## Area 6: Shared primitives shape

| Option | Description | Selected |
|--------|-------------|----------|
| ProcessContext struct | node_ids + a11y bundled; one per process. | ✓ |
| Separate allocators, no ProcessContext | NodeIdAllocator + SharedAccessibility separately. | |
| Global statics (lib-owned) | OnceCell / Lazy; host calls rgui::init_process(). | |

**User's choice:** ProcessContext struct (recommended).

---

## Claude's Discretion

The user did not flag any area as "you decide". Areas where the plan + executor may make low-risk choices without re-asking:

- The exact `AppEventOutcome` return shape (`enum { Consumed, Ignored }`).
- The `SharedAccessibility` constructor surface (`SharedAccessibility::new(backend)`, `SharedAccessibility::none()`).
- Whether `UiRuntime::default()` (no-args) remains a backward-compat single-window entry point that constructs a hidden `ProcessContext` internally — recommended for v1.x.
- The exact `wgpu::Surface` lifetime story (wgpu 29 + winit 0.30 has a `'static` bound that needs careful plumbing).

## Deferred Ideas

- **Drag a widget between windows** (WIN-05) — v2
- **Modal that spans multiple windows** (WIN-06) — v2
- **SharedAtlas beyond Mutex** — v1.x
- **`UiRuntime::default()` semantics** — may be deprecated in v2
- **Process-level stats aggregation** — v1.x
- **Cross-window drag preview** — v2 follow-up
- **`RuntimeSet { windows: HashMap<WindowId, UiRuntime> }` helper struct** — v1.x
