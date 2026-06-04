# Phase 5: Render Path Stress - Context

**Gathered:** 2026-06-04
**Status:** Ready for planning

<domain>

## Phase Boundary

The wgpu render path today has been validated on one backend (`WgpuRenderer::new_headless` with `wgpu::Backends::PRIMARY`, which on Windows resolves to DX12, on Linux to Vulkan, on macOS to Metal). The lib has 6 visual goldens in `tests/visual_goldens.rs` (text hierarchy, toolbar, popover, scroll clip, full widgets, widget collections, widget showcase flow, new painters smoke — 8 actually) with a per-pixel diff tolerance (`PIXEL_TOLERANCE=1`, `CHANGED_PIXEL_RATIO_LIMIT=0.0001`, `MAX_ABS_DIFF_LIMIT=5`). There is no CI configuration, no `benches/`, no per-frame timing harness.

This phase delivers four things to make the render path shippable:

1. **Visual goldens on a second GPU backend** (REND-01). The current goldens run on whatever `wgpu::Backends::PRIMARY` resolves to. REND-01 wants a *second* backend — explicitly Vulkan + (Metal or DX12) on CI.
2. **A stress-test scene** (REND-02) — the real-world paint-path test the lib's render pipeline must survive.
3. **wgpu validation layers in CI** (REND-03) — the only way to catch buffer-alignment / lifetime issues before they ship.
4. **A frame-budget micro-benchmark** (REND-04) — a measured guarantee that a 50-widget UI's frame CPU cost is < 8ms on a modern laptop.

What's out of scope (later phases):
- `WindowedList` (VIRT-01..04, Phase 8) — REND-02's "10k-row windowed list" arrives in Phase 8.
- Drag-and-drop (DND-01..04, Phase 7) — REND-02's drag-and-drop stress arrives in Phase 7.
- A real `examples/stress.rs` winit example — Phase 5 ships a headless test only; the interactive stress example is a v1.x follow-up.

</domain>

<decisions>

## Implementation Decisions

### Stress scene (05-02)

The discussion scoped the stress scene. The 10k-row list paints inside a `scroll_area` of fixed viewport height; the off-screen rows are clipped at lower time. The scene is delivered as a `tests/stress_scene.rs` headless test (no winit, no example).

- **D-01: Stress scene is a `tests/stress_scene.rs` headless test, no winit example.** Drives the full stress scene through `WgpuRenderer::new_headless` and `UiRuntime::update`; no window needed. CI-friendly. The interactive winit variant of the stress scene is a v1.x follow-up.
- **D-02: The 10k-row list is rendered in a `scroll_area()` of fixed viewport height (~600px).** The `display_list` carries all 10k logical rows (today's `list()` spec lays them all out), but the renderer drops off-screen items at lower time. The test scrolls the viewport through 10 distinct positions over N=10 `update()` calls and asserts the visible-window's `display_list` is bounded regardless of scroll position.
- **D-03: The scene uses today's primitives only.** The REND-02 success criterion mentions "10k-row windowed list, animated progress bars, IME, drag-and-drop". Phase 5 ships the paint-path-stressable parts (overlapping translucent layers, 10k-row list in a scroll area, animated progress bars, IME preedit on a `Textarea`). `WindowedList` (Phase 8) and `Drag`/`DropTarget` (Phase 7) are annotated with `// TODO(phase-7): drag` and `// TODO(phase-8): windowed list` comments in the test source so the follow-up is visible.
- **D-04: The stress test asserts on bounded command count per frame.** Over N=10 `update()` calls at distinct scroll positions, the test asserts:
  1. `output.display_list.len() < 2000` (visible-window command count, regardless of scroll position).
  2. `RenderStats.display_command_count < 2000`.
  3. The wgpu error scope reports zero validation errors during the test.
  The 8ms frame budget is NOT asserted here (it's 05-04's job).
- **D-05: `// TODO(phase-7): drag` and `// TODO(phase-8): windowed list` comments are required at the stress-scene source lines for DnD and the windowed list.** These make the deferred pieces visible in code review and let a future grep for `TODO(phase-7)` / `TODO(phase-8)` find the seams.

### Claude's Discretion (05-01, 05-03, 05-04)

The user did not select the other three gray areas (second GPU backend, validation layer placement, frame budget benchmark harness). The planner has flexibility on:

- **05-01 — Second GPU backend.** The `RendererOptions.backends: wgpu::Backends` knob already exists (D-09 in 04-CONTEXT). The second backend is whichever platform's runner is available in CI (DX12 on `windows-latest`, Metal on `macos-latest`, Vulkan on `ubuntu-latest` with the `gpu-runner` image if available). The matching strategy is the existing `pixel_diff_stats` tolerance — strict byte equality where it holds, tolerant otherwise. If a free GitHub Actions runner with a real GPU is not available, the second backend is a software fallback (`wgpu::Backends::SECONDARY` or a different `PresentMode`/format combo) and the test is gated behind a `REND_GOLDEN_BACKEND` env var.
- **05-03 — Validation layers.** The wgpu 29 API exposes validation through the device descriptor's `required_features`/`required_limits` and through the `wgpu::InstanceDescriptor.flags` (validation/debug). The default v1.x choice: enable validation in `tests/*` builds (catches regressions locally) and in CI, but not in `examples/*` (perf cost). A `validation-layers` Cargo feature is a fine-grained alternative if the test-only path proves too costly.
- **05-04 — Frame budget benchmark.** The 50-widget scene is the existing `visual_showcase` example's widget composition (or a synthetic equivalent built from the same primitives). The harness is a `tests/frame_budget.rs` that runs N=1000 `update()` calls and asserts mean < 8ms / p99 < 16ms. A `benches/` criterion binary is a v1.x follow-up if the test-only harness is too coarse.

The planner is free to make these choices without re-asking; the executor will document the chosen approach in the plan's SUMMARY.

### Folded Todos

None — discussion stayed within phase scope.

</decisions>

<canonical_refs>

## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project context
- `.planning/PROJECT.md` — full project context, constraints (60fps, no unwrap in paint path, no serde on hot path)
- `.planning/REQUIREMENTS.md` — REND-01..04 (lines 38-43) are the v1 scope
- `.planning/ROADMAP.md` — Phase 5 entry (line 21) and 4 plan slots (05-01..04, lines 130-133)

### Prior phase decisions that apply here
- `.planning/phases/01-incremental-reconciliation/01-CONTEXT.md` — diffing is keyed by NodeId; the per-frame `display_list` is bounded by the dirty region.
- `.planning/phases/02-event-input-hardening/02-CONTEXT.md` — the receive-side IME gating; Phase 5's stress test exercises an `Input`/`Textarea` with preedit.
- `.planning/phases/03-text-ime/03-CONTEXT.md` — `ImeHostDriver: Send + Sync`; `TextCacheStats` exposes hit/miss for the per-thread measure cache.
- `.planning/phases/04-multi-window/04-CONTEXT.md` D-09 — `RendererOptions.backends: wgpu::Backends` is already a knob. Phase 5's 05-01 just changes the default for the second-backend test.

### Code to read
- `tests/visual_goldens.rs:1-206` — the existing visual-golden infrastructure: `render_runtime_rgba`, `golden_paths`, `assert_visual_matches`, `pixel_diff_stats`, the `PIXEL_TOLERANCE` / `CHANGED_PIXEL_RATIO_LIMIT` / `MAX_ABS_DIFF_LIMIT` constants. 05-01 reuses all of this; the change is a `RendererOptions { backends: wgpu::Backends::SECONDARY, .. }` in `render_runtime_rgba` (or a new `render_runtime_rgba_with_backends` helper).
- `src/render/wgpu/options.rs:5-22` — `RendererOptions` struct + Default. The 05-01 second backend test passes a non-default `backends` here.
- `src/render/wgpu/mod.rs:57-148` — `WgpuRenderer::new_headless` and `new_headless_for_tests`; both already take `RendererOptions`.
- `src/runtime/runtime.rs:1218-1232` — the `update()` path's `display_list` build; the stress scene's N=10 `update()` calls go through here.
- `src/runtime/runtime.rs:1078-1100` — the synthesized `RenderStats` (REND-02's `display_command_count` is read from here).
- `src/core/snapshot.rs:166-205` — `AccessibilityMetrics`, `RenderStats`; the `FrameOutput::debug_snapshot() -> UiSnapshot` chain Phase 5 reads from.
- `src/widgets/list.rs` (assumed location) — the `list()` builder; the stress scene builds a `list()` with 10k items and wraps it in a `scroll_area()`.
- `src/widgets/progress_bar.rs` (assumed location) — the `progress_bar()` builder; the stress scene has 5+ animated progress bars.
- `src/widgets/popover.rs` / `modal.rs` (assumed locations) — translucent layers via overlapping `popover`s or `modal`s.

### External references
- **wgpu 29** `wgpu::Backends` enum (`PRIMARY`, `SECONDARY`, `VULKAN`, `METAL`, `DX12`, `BROWSER_WEBGPU`, `GL`). The second-backend test picks one of `VULKAN`, `METAL`, `DX12`, `BROWSER_WEBGPU` based on the CI runner.
- **GitHub Actions runners** — `windows-latest` has DX12, `macos-latest` has Metal, `ubuntu-latest` has Vulkan if a `gpu-runner` image is configured (none of these are free; the linux Vulkan runner requires a self-hosted runner with a GPU).
- **wgpu 29 `InstanceDescriptor.flags`** — the validation-layer entry point. `wgpu::InstanceFlags::DEBUG` enables validation; `wgpu::InstanceFlags::VALIDATION` is the explicit validation toggle.

</canonical_refs>

<codebase_context>

## Existing Code Insights

### Reusable Assets
- `tests/visual_goldens.rs` — the 8 visual goldens + the `assert_visual_matches` machinery. 05-01 reuses this for the second backend: the only new code is a `RendererOptions { backends: SECONDARY, .. }` parameterization.
- `pixel_diff_stats` + `PIXEL_TOLERANCE` / `CHANGED_PIXEL_RATIO_LIMIT` / `MAX_ABS_DIFF_LIMIT` — the per-channel tolerance already accommodates driver non-determinism. 05-01 doesn't need new tolerance constants.
- `WgpuRenderer::new_headless` — already takes `RendererOptions`. 05-01's only change is the default `backends` field; 05-04's frame budget test reuses this constructor.
- `RenderStats` (in `UiSnapshot.performance.render_stats`) — exposes `display_command_count`, `atlas_upload_bytes`, `glyph_count`, etc. The 05-02 stress test asserts on `display_command_count`.
- `FrameOutput::debug_snapshot() -> UiSnapshot` — the post-Phase-4 public API for reading the per-frame stats.
- `Element::scroll_area()`, `Element::list()`, `Element::progress_bar()`, `Element::textarea()` — the stress scene's building blocks.

### Established Patterns
- **`RGUI_UPDATE_GOLDENS=1 cargo test --test visual_goldens` for golden regeneration** — the existing golden update flow. 05-01 mirrors this for the second backend (e.g. `RGUI_UPDATE_GOLDENS=1 cargo test --test visual_goldens_vulkan`).
- **Test-only `for_tests` constructors** — `WgpuRenderer::new_headless_for_tests` is the test fast-path. 05-04 may want a `for_bench` constructor that disables validation, or just configures via `RendererOptions`.
- **Headless test pattern** — `tests/visual_goldens.rs` runs in CI without a display. 05-02 follows the same pattern.
- **wgpu error scope** — `wgpu::Device::push_error_scope` / `pop_error_scope` is the validation-error capture API. The 05-02 stress test uses this to assert zero validation errors during the scene.
- **`u64` `NodeId` counter (Phase 4 D-14)** — process-global, the stress scene's 10k `list()` rows each get a fresh `NodeId` from this counter.

### Integration Points
- `src/runtime/runtime.rs:2554` (the default `UiRuntime`) — 05-02's stress test uses `UiRuntime::default()` (no need for `for_window`).
- `src/render/wgpu/mod.rs:165-180` — `WgpuRenderer::atlas()` and `atlas_mut()`. The stress test doesn't need these (it only reads stats), but a future stress test that uploads many images would.
- `Cargo.toml:21-35` — the feature flags. A new `validation-layers` feature is an option for 05-03 (Claude's discretion).
- `.github/workflows/` (does not exist) — 05-03 creates a new CI workflow file. The exact filename and trigger config is Claude's discretion.

</codebase_context>

<specifics>

## Specific Ideas

- **The 10k-row list inside a `scroll_area()`** is the v0 of what Phase 8's `WindowedList` will automate. The stress test's `// TODO(phase-8): windowed list` comment marks the seam. When Phase 8 lands, the stress test can swap `list()` + `scroll_area()` for `WindowedList` and the assertion (visible-window command count < 2000) becomes a regression test for Phase 8.
- **The `// TODO(phase-7): drag` comment** in the stress test marks the DnD seam. When Phase 7 lands, the stress test can wire a `Drag` source on one of the list rows and assert that the drag overlay's paint path is bounded.
- **Per-frame command count limit of 2000** is a starting point; tune based on the real measurement. If the first run shows mean 800 / max 1500, the 2000 cap is conservative; if it shows mean 3500, the cap is wrong and the off-screen culling in the renderer is the bug.
- **`WgpuRenderer::new_headless_for_tests`** already exists. 05-04's frame budget test should use it (no extra setup) but may need a `for_bench` variant if `new_headless_for_tests` is too slow (e.g. creates a fresh atlas every call).
- **Existing visual goldens can run on multiple backends in one test** — `visual_goldens.rs` could be parameterized with `RendererOptions { backends: wgpu::Backends::VULKAN, .. }` and the same `RGUI_UPDATE_GOLDENS` env var. This is the simplest 05-01 design: a new test file `tests/visual_goldens_vulkan.rs` (or a `#[cfg(...)]` gate inside `visual_goldens.rs`) that runs the existing 8 goldens on the second backend.

</specifics>

<deferred>

## Deferred Ideas

- **winit-runnable `examples/stress_scene.rs`** — the interactive stress example with a real window, scrollable 10k list, animated progress bars. 05-02 ships the headless test only; the winit variant is a v1.x follow-up that doesn't need CI infrastructure.
- **`benches/` criterion binary** for the frame budget — 05-04 ships a `tests/frame_budget.rs`; a criterion `benches/frame_budget.rs` is a v1.x follow-up if the test-only harness is too noisy.
- **CI runners with real GPUs** — the REND-01 success criterion wants goldens on TWO backends, which is hard on free GitHub Actions runners (no GPU on `ubuntu-latest`). A self-hosted runner with a GPU (or a paid runner like `ubuntu-gpu-latest`) is a v1.x follow-up; until then, 05-01's "second backend" is a software fallback or a runner-provided backend.
- **`WindowedList` (Phase 8) + DnD (Phase 7) in the stress scene** — explicitly out of Phase 5 scope; the `// TODO(phase-7/8)` comments in the test source mark the seams for future phases.
- **Per-frame pixel-perfect diff against a "gold master"** — too strict for a real-GPU environment. The existing `pixel_diff_stats` tolerance is the v1.x contract.

### Reviewed Todos (not folded)

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 05-render-path-stress*
*Context gathered: 2026-06-04 via inline discussion (GSD subagents unavailable in this runtime; discussed 1 of 4 gray areas)*
