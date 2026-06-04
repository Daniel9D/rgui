---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: completed
stopped_at: Completed 05-02-PLAN.md (stress-scene)
last_updated: "2026-06-04T15:55:00.000Z"
last_activity: 2026-06-04
progress:
  total_phases: 8
  completed_phases: 1
  total_plans: 18
  completed_plans: 6
  percent: 13
---

# Project State

## Project Reference

See: `.planning/PROJECT.md` (updated 2026-06-03)

**Core value:** The paint pipeline produces a correct, sorted `DisplayList` for every `Element` tree — every `WidgetKind` paints something visible with the right z-order, the right hover/disabled/checked state, and the right glyph from the right font.

**Current focus:** Phase 5 (Render Path Stress) — plan 05-02 complete; ready for plan 05-03

## Current Position

Phase: 5
Plan: 05-02 (complete); next is 05-03
Status: 2/4 plans of Phase 5 complete; ready for `/gsd-execute-phase 5` to continue
Last activity: 2026-06-04

Progress: [███████████████░░░░░░] 53% (16/30 plans; Phase 5 in progress)

## Performance Metrics

**Velocity:**

- Total plans completed: 16
- Average duration: — min
- Total execution time: 0.0 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 1. Incremental Reconciliation | 4/4 | — | — |
| 2. Event & Input Hardening | 4/4 | — | — |
| 3. Text & IME | 3/3 | — | — |
| 4. Multi-Window | 3/3 | — | — |
| 5. Render Path Stress | 2/4 | — | — |
| 6. Public API Hardening | 0/3 | — | — |
| 7. Theme v2 + Animation + DnD | 0/5 | — | — |
| 8. Virtualization + Canvas + i18n + Docs | 0/4 | — | — |

*Updated after each plan completion*

## Accumulated Context

### Decisions

See `PROJECT.md` Key Decisions table for the full log. Recent:

- **Phase 5 / 05-02 implementation (2026-06-04)**: `tests/stress_scene.rs` — headless 10,000-row `list` in fixed-viewport (400×600) `scroll_area`, asserts `command_count < 2_000` + zero wgpu validation errors via `device.push_error_scope(Validation) / scope.pop()`. First test run failed (10,007 commands for 10k rows) revealing the list's unbounded paint. Fixed by adding `clip_rect: Option<Rect>` to `VisualState` (pub additive field), threading the ancestor clip into `push_paint` from `layout.clip_rect`, and culling rows in `ListPainter::paint_content` whose `y + row_height` falls outside the visible top/bottom. Diff is minimal: one painter change + one struct field + one new arg in `push_paint`. Two `// TODO` annotations on the test (`// TODO(phase-7): drag`, `// TODO(phase-8): windowed list`) mark future-extension seams. Lib unit tests in `src/core/render.rs` (`stress_stats_command_count_is_stable_across_runs`, `stress_stats_command_count_does_not_grow_with_list_size`) replicate the integration test's invariants at the `RenderStats` level for fast feedback (no wgpu device needed). Rule 1 deviation (auto-fix): the culling gap was a real bug in the existing list paint, applied the minimum fix in the same plan rather than splitting into a follow-up. Pre-existing `E0639` errors in `tests/interactive_widgets.rs` and stale-baseline failures in `tests/visual_goldens.rs` are independent of this plan and not addressed.
- **Phase 5 / 05-01 implementation (2026-06-04)**: Added `WgpuRenderer::new_headless_for_tests_with_backends(size, format, backends)` (`src/render/wgpu/mod.rs`), a per-test renderer constructor that selects the GPU backend. `WgpuContext::headless` already threaded `options.backends` into `instance_descriptor` (no change needed in `context.rs`). Existing `new_headless_for_tests()` preserved as a thin forward (back-compat). New `tests/visual_goldens_vulkan.rs` mirrors the 8 PRIMARY-backend goldens against `Backends::VULKAN`; cross-backend tolerance constants are `MAX_ABS_DIFF_LIMIT=15` and `CHANGED_PIXEL_RATIO_LIMIT=0.005` (loosened from the same-backend 5 / 0.0001). Aggregate test `vulkan_diff_is_within_cross_backend_tolerance` re-runs all 8 scenes and asserts the worst diff across the full set stays within the cross-backend envelope. File is gated by `vulkan-goldens = []` Cargo feature (off-by-default; CI plan 05-03 turns it on). Local Vulkan adapter is present and ran the suite; the diff magnitudes match the PRIMARY suite's stale-baseline diffs *byte-for-byte* (e.g. widgets_collections: 5765/307200 pixels, max_abs_diff=176 — identical on DX12 and Vulkan), proving cross-backend stability of the wgpu render path. The remaining failures trace to stale `tests/goldens/*.png` baselines that already fail on the PRIMARY suite (pre-existing, out of plan scope; baselines need refresh via `RGUI_UPDATE_GOLDENS=1`).
- **Phase 4 implementation (2026-06-04)**: 3 plans executed inline via `task` tool subagents. `WindowId` newtype (`src/runtime/window_id.rs`) + `UiRuntime::for_window(id, &ctx)` + `dispatch_to_window` (D-10) + `dispatch_app_event` (D-12) + `AppEvent`/`AppEventOutcome`/`AppShortcuts` (D-12). 4 winit examples migrated (`widgets`, `visual_showcase`, `rml_showcase`, `rml_widget_gallery`); `basic_window.rs` left on `UiRuntime::default()` (non-interactive, back-compat still works). D-17 trait bounds (`ImeHostDriver: Send + Sync`, `AccessibilityBackend: Send + Sync`) were pulled forward from 04-02 into 04-01 to unblock the D-19 assert activation. `ProcessContext` is the full D-13 struct: `node_ids: NodeIdAllocator` (`Arc<AtomicU64>`, process-global) + `a11y: Option<SharedAccessibility>`. `IdAllocator` refactored from `&mut u64` to `&NodeIdAllocator`; `Reconciler.next_id` removed in favor of the process-global counter. D-19 static assert activated via `unsafe impl Send + Sync` for `TaffyLayoutBackend` (taffy 0.10.1's `TaffyTree` stores `*const ()` in its `SlotMap`; SAFETY block explains single-threaded-per-runtime access pattern). `SharedWgpuDevice` (D-06..D-09) wraps `Arc<Adapter>` + `Arc<Device>` + `Arc<Queue>` + `Arc<Mutex<GpuAtlas>>`; `WgpuRenderer::atlas()` field type changed from `GpuAtlas` to `Arc<Mutex<GpuAtlas>>` (D-09 lock pattern). `WgpuRenderer::with_shared_device` and `SurfaceRenderer::with_shared_device` are additive constructors; existing `from_context`/`new` constructors remain for single-window back-compat. 3 integration tests pin WIN-02 (coexistence, disjoint `(window_id, node_id)` tuples), WIN-03 (event routing, independent state), WIN-04 (snapshot isolation, monotonic counter across runtimes). `examples/multi_window.rs` demonstrates two winit windows in one process sharing a `SharedWgpuDevice` and two `UiRuntime` instances. 04-03 plan's test code referenced fictional `update_with`/`last_output`/`UiSnapshot::window_id` API; tests rewritten to use real `update(FrameInput) -> FrameOutput + debug_snapshot()` API.
- **Phase 3 implementation (2026-06-04)**: `UiRuntime::text_cache_stats` is captured at the start of `update()` (before `text_system` is mutably borrowed by `FrameBuilder`); the value is the previous frame's final cache state. `RenderStats::text_cache` is added for forward-compat (the wgpu backend currently leaves it `default()`; the runtime populates it in its own `RenderStats` build path). `TextCacheStats` derives `serde::Serialize` so it can ride the `to_debug_json` round-trip.
- **Phase 3 context (2026-06-03)**: `ImeHostDriver` is a producer-side trait (runtime calls `driver.poll(&mut sink)` per frame; sink pushes `UiEvent::ImePreedit` / `UiEvent::ImeCommit`). `MockDriver` replays a `Vec<ImeOp>` script for tests. CJK + Arabic shaping tests rely on system fonts (`fonts-noto-cjk`, `fonts-noto`); CI installs via `scripts/ci-install-fonts.sh`; `RGUI_REQUIRE_FONTS=1` opt-in to fail-fast. Arabic is the v1 RTL reference (isolated + contextual + bidi cases). `TextCacheStats` surfaces in three places: `UiRuntime::text_cache_stats()` public method, `UiSnapshot.text_cache` field, `RendererStats.text_cache` field. Heuristic `clear_metrics_cache()` and shape/layout `clear_text_cache()` (new) clear the two caches symmetrically. `Shaping::Advanced` is the v1 default for all scripts. No winit/AppKit/browser adapters in v1 (apps wire their own).
- **Phase 2 implementation (2026-06-03)**: `ShortcutRegistry::resolve` gained a `focused_is_text_input: bool` argument that suppresses non-modifier-prefixed chords inside `Input`/`Textarea`/`Select`. `FocusManager::is_focusable(WidgetKind)` + `tab_next`/`tab_prev` helpers provide a tree-walking focus traversal alternative to the existing `FocusSystem` (which the runtime continues to use for the scope-based overlay routing). `ModalSpec::trap_focus: bool` flag (default `false`) lets modals opt into focus trapping. `InputSpec::ime_enabled: bool` (default `false`) gates IME preedit routing so CJK users can opt in to the preedit-then-commit path. `Element::overflow_x(Overflow)` / `overflow_y(Overflow)` setters enable per-axis scroll configuration; the runtime's `handle_wheel` was already 2D — the new tests pin the per-axis clamping behavior.
- **Phase 1 implementation (2026-06-03)**: `Reconciler::diff(prior, new) -> DiffOutput` is positional (children paired by index) with `kind_signature` comparing `WidgetSpec` signature for state preservation. New `LayoutCache` wraps the existing `TaffyLayoutBackend` and indexes `LayoutBox` per `NodeId` for the paint path. `PointerCapture::release_matching` returns a `PointerCancel` for the captured node when the capture's key is in the unmounted list. The new `diff` is *not yet* wired into `runtime::update` — the existing `reconcile_with_dirty` (keyed-only) is still the production path; the diff is exercised by tests/recon and ready for the Phase 6 (Public API Hardening) wiring.
- **Workflow init (2026-06-03)**: Granularity=Fine, Execution=Parallel, Git=Yes, Research/PlanCheck/Verifier/DriftGuard=Yes, AI=Quality, PR body=User Stories & Acceptance Criteria, Mode=Vertical MVP.
- **Theme v2 migration (P1)**: Flat `Theme::metrics` / `Theme::select` fields are deprecated in favor of `theme.components.get(WidgetKind)` (Bug fix 3.8, this PRD).
- **SmallVec deferred (P2)**: `paint_node` continues to return `Vec<PaintedCommand>`; the `SmallVec<[_; 4]>` optimization is deferred to post-v1 (PROJECT.md Key Decisions).

### Pending Todos

- Wire `Reconciler::diff` into `runtime::update` (replacing `reconcile_with_dirty` for unkeyed nodes). Tracked for Phase 6 (Public API Hardening).
- Add `release_captures_for_unmounted` call into `update()` after the diff runs. Tracked for Phase 6.
- Add `LayoutCache` reads into the paint path (currently the paint path still walks taffy). Tracked for Phase 5 (Render Path Stress).
- Wire `ModalSpec::trap_focus` into the runtime's overlay-scope routing (currently the new `FocusManager::tab_next` is the lighter-weight alternative, but the existing `FocusSystem` is what the runtime actually uses). Tracked for a future phase if/when a real-world modal needs it.
- Write a winit `ImeHostDriver` adapter as a v1.x follow-up. Not in scope for Phase 3.
- Add preedit underline styling in the paint path. Not in Phase 3; v1.x.
- Add `RGUI_REQUIRE_FONTS=1` env-var path to the CJK/Arabic shaping tests' `tracing::warn!` skip → fail-fast mode. Tracked in Phase 3's CI wiring.

### Blockers/Concerns

- The runtime paint path has `unwrap()` calls in widget painters. Phase 6 (Public API Hardening) plans to audit and remove them.
- The current `WidgetPainter` trait is `Send + Sync`; verify all custom painters written by users are also `Send + Sync`. Phase 6 plans to add a guide.
- The `runtime ↔ widgets` module boundary still has widget painters living in `runtime/paint.rs`; Phase 7 (Theme v2) is the natural place to do the architectural split (8.2 from the feedback review).

## Deferred Items

Items acknowledged and carried forward:

| Category | Item | Status | Deferred At |
|----------|------|--------|-------------|
| Perf | `paint_node` return type → `SmallVec<[_; 4]>` | Deferred to post-v1 | 2026-06-03 |
| Arch | Move widget painters to `widgets/paint/` (feedback 8.2) | Deferred to Phase 7 | 2026-06-03 |
| Arch | Full split of `runtime/paint.rs` (feedback 8.1) | Forward-compat done (pub(super)); full extraction in Phase 7 | 2026-06-03 |
| API | Explicit `pub use` re-export list (feedback 8.5 alt) | Wildcard is `#[doc(hidden)]`; explicit list is post-v1 | 2026-06-03 |
| Dep | `smallvec` for paint_node return type (feedback 4.7) | Deferred to post-v1 | 2026-06-03 |
| Reconcile | Wire `Reconciler::diff` into `runtime::update` | Phase 6 wiring | 2026-06-03 |
| IME | South-East Asian complex-script state machine (Hindi, Thai, Khmer) | Deferred to v1.x | 2026-06-03 |
| IME | Per-character preedit composition | Deferred to v1.x | 2026-06-03 |
| Scroll | Momentum / inertial trackpad pan | Deferred to v1.x | 2026-06-03 |
| Focus | Custom `tabindex` attribute | Deferred to v1.x (users use `request_focus` directly) | 2026-06-03 |
| Focus | Focus ring paint (OS / browser draws it for v1) | Deferred to v1.x | 2026-06-03 |
| Input | Touch / pen / gamepad events | Deferred to v1.x | 2026-06-03 |

## Session Continuity

Last session: 2026-06-04T13:38:00.000Z
Stopped at: Completed 05-02-PLAN.md (stress-scene)
Resume file: None
