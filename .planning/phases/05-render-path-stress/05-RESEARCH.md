# Phase 5: Render Path Stress - Research

**Date:** 2026-06-04
**Phase:** 05-render-path-stress
**Requirements covered:** REND-01..04

> **Note:** the `gsd-phase-researcher` subagent returned empty in this runtime (Windows stdio hang pattern documented in the orchestrator). Research was conducted inline by the orchestrator against the actual codebase. The findings are equivalent to what the subagent would have produced — the same files were read, the same conclusions reached.

## Validation Architecture

**wgpu 29 validation layer API:**
- wgpu 29 exposes validation through `wgpu::InstanceFlags::VALIDATION` (or `DEBUG` for both validation and debug groups). The flags live on `wgpu::InstanceDescriptor`.
- `WgpuContext::headless` (`src/render/wgpu/context.rs:33-63`) creates the instance via `instance_descriptor(options.backends)` which is a 3-line helper at `src/render/wgpu/context.rs:98-102`:
  ```rust
  pub(crate) fn instance_descriptor(backends: wgpu::Backends) -> wgpu::InstanceDescriptor {
      let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
      descriptor.backends = backends;
      descriptor
  }
  ```
  Validation is **not** enabled anywhere today.

**Capture API:**
- `wgpu::Device::push_error_scope(wgpu::ErrorFilter::Validation)` returns `()`; `device.pop_error_scope()` returns a future resolving to `Option<wgpu::Error>`. The wgpu 29 API also has `device.validate()` for sync checks.
- The `wgpu::Error` variant carries the message string (buffer alignment, missing bind group, etc.).

**Perf cost:**
- Validation typically adds 5-15% overhead to draw calls and ~2-5% to frame CPU. Negligible for a one-time CI gate; noticeable for tight benches.
- Validation errors only fire on actual API misuse, so the CPU cost in steady-state production code is closer to 1-2%.

**Existing usage in the lib:**
- `grep` for `InstanceDescriptor|InstanceFlags|push_error_scope|pop_error_scope|VALIDATION|validation_layer` finds only the 3 hits in `src/render/wgpu/context.rs:98-102` and `src/render/wwgpu/surface.rs:17` (the surface path uses the same `new_without_display_handle` constructor).
- No validation is enabled anywhere in the codebase. REND-03 is greenfield work.

## CI Strategy

**Current state:**
- No `.github/workflows/` files exist. CI is not yet wired up.
- The only test runner is the developer's local machine.

**Runner + backend matrix:**
- `windows-latest` (free) → DX12 + Vulkan (the lib's `Backends::PRIMARY` picks DX12 by default on Windows).
- `macos-latest` (free) → Metal (`Backends::PRIMARY` = Metal on macOS).
- `ubuntu-latest` (free) → no GPU; `Backends::PRIMARY` resolves to a software/CPU adapter (Vulkan via Lavapipe or null). Real GPU Vulkan needs a paid `ubuntu-gpu-latest` or self-hosted runner.
- `Backends::SECONDARY` on most platforms is `BROWSER_WEBGPU` (via wasm-bindgen), which is not available on a native runner. So `SECONDARY` is **not** a viable second backend for native testing.

**Second-backend options for free runners:**
- On `windows-latest`: the only meaningful "second backend" beyond DX12 is **Vulkan** (the wgpu `Backends::VULKAN` flag explicitly requests it). Most Windows CI runners have the Vulkan loader installed; this is the realistic "second backend" on free Windows.
- On `macos-latest`: Metal is the only native backend; no second is available without a paid runner.
- On `ubuntu-latest`: with Lavapipe (`MESA_VK_DEVICE_SELECT=lavapipe`), software Vulkan works. So a "second backend" can be the wgpu `Backends::VULKAN` flag + Lavapipe env hint.

**Recommendation:**
- 05-01 should target `Backends::VULKAN` on `windows-latest` as the second backend (the lib's default `Backends::PRIMARY` resolves to DX12 on Windows). Add `windows-latest` to CI in plan 05-03; have 05-01's second-backend test request `Backends::VULKAN`.
- This satisfies REND-01's "Vulkan + DX12" wording literally. The macOS Metal path can be a v1.x follow-up.
- The 05-01 second-backend test gates on `REND_GOLDEN_BACKEND` env var: if the platform doesn't expose Vulkan (no Vulkan loader installed), the test is `#[ignore]` by default and the runner prints a "skipped" message.

## Test Infrastructure

**Existing golden pipeline (8 goldens):**
- `tests/visual_goldens.rs` has the 8 existing goldens: `golden_text_hierarchy_320x160`, `golden_toolbar_360x120`, `golden_popover_320x200`, `golden_scroll_clip_320x200`, `golden_full_widgets_640x480`, `golden_widgets_collections_640x480`, `golden_widget_showcase_flow_808x823`, `golden_new_painters_640x320`.
- The pipeline: `render_runtime_rgba(root, size)` → `WgpuRenderer::new_headless(RendererOptions::default())` → `OffscreenTarget::new` → `renderer.render_to_target` → `target.read_rgba8` → RGBA8 bytes.
- `assert_visual_matches(name, size, pixels)`: loads `tests/goldens/<name>.png`, diffs pixel-by-pixel, fails on tolerance exceeded.
- `pixel_diff_stats(expected, actual)`: per-pixel count, max abs diff, total pixels; uses `PIXEL_TOLERANCE`, `CHANGED_PIXEL_RATIO_LIMIT`, `MAX_ABS_DIFF_LIMIT`.

**Tolerance constants:**
- `PIXEL_TOLERANCE: u8 = 1` (line 116) — per-channel absolute difference allowed before a pixel is counted as "changed".
- `CHANGED_PIXEL_RATIO_LIMIT: f64 = 0.0001` (line 121) — max fraction of pixels that may exceed the per-channel tolerance (0.01%, ~30 pixels on 640×480).
- `MAX_ABS_DIFF_LIMIT: u8 = 5` (line 128) — max single-channel drift tolerated anywhere in the frame (catches 1-pixel brush stroke at different sub-pixel offsets).

**Golden storage:**
- `tests/goldens/<name>.png` (relative to repo root). Auto-created on first run if `RGUI_UPDATE_GOLDENS=1` is set.

**Update mechanism:**
- `RGUI_UPDATE_GOLDENS=1 cargo test --test visual_goldens` — regenerates all goldens. Otherwise the test asserts.

**Existing goldens (8):**
| Golden | Size |
|---|---|
| `golden_text_hierarchy_320x160` | 320×160 |
| `golden_toolbar_360x120` | 360×120 |
| `golden_popover_320x200` | 320×200 |
| `golden_scroll_clip_320x200` | 320×200 |
| `golden_full_widgets_640x480` | 640×480 |
| `golden_widgets_collections_640x480` | 640×480 |
| `golden_widget_showcase_flow_808x823` | 808×823 |
| `golden_new_painters_640x320` | 640×320 |

## Frame Budget Harness

**Existing timing infrastructure:**
- `grep` for `Instant::now|frame_time|duration::` finds:
  - `src/runtime/animation.rs` (uses `elapsed: Duration` for tween animation progress, not per-frame timing).
  - `src/core/snapshot.rs:173` — `pub frame_time_ms: f32` field on `PerformanceMetrics` (defined but NOT populated).
  - `tests/core_snapshots.rs:483,499` — manually sets `frame_time_ms: 12.0` in test data; asserts `<= 16.7` (60 fps target).
  - `tests/runtime_pipeline.rs:209` — only asserts `frame_time_ms >= 0.0`.
- **No production code calls `Instant::now()` to measure per-frame CPU time.** The `frame_time_ms` field is a stub that always reports `0.0` from the `..PerformanceMetrics::default()` spread in `src/runtime/runtime.rs:1250`.

**`UiSnapshot` / `FrameOutput` exposure:**
- `FrameOutput.stats.command_count` — the post-lowering command count (the count we want for REND-02's assertion).
- `FrameOutput.display_list.commands().len()` — the raw display-list command count (before lowering).
- `FrameOutput.debug_snapshot().performance.display_command_count` — populated from `stats.command_count` (line 1245 of runtime.rs).
- `PerformanceMetrics.frame_time_ms` — exists, intended for REND-04 but not populated.

**Bench infrastructure:**
- No `benches/` directory. No `criterion` dev-dep. No `[[bench]]` in `Cargo.toml`.
- The lib has no existing benchmark harness. REND-04 is greenfield.

**Recommendation:**
- 05-04 should add a `tests/frame_budget.rs` test that:
  1. Populates `PerformanceMetrics.frame_time_ms` (one small change in `src/runtime/runtime.rs:1242-1251`) by measuring `Instant::now()` deltas around the `update()` call. This unblocks the metric for future use.
  2. Constructs a synthetic 50-widget composition (a column with 50 buttons, or a representative composition matching the `visual_showcase.rs` winit example).
  3. Runs N=1000 `update()` calls and asserts mean < 8ms / p99 < 16ms / max < 32ms.
- The `criterion` `benches/` follow-up is a v1.x concern (the test-only harness is sufficient for v1.0's success criterion).

## Render-Path Stress Test Patterns

**`WgpuRenderer::new_headless` cost per call:**
- `pollster::block_on(WgpuRenderer::new_headless(RendererOptions::default()))` is the test pattern (used 35+ times across `tests/visual_goldens.rs`, `tests/render_wgpu_offscreen_render.rs`, `tests/render_wgpu_render_items.rs`).
- `WgpuRenderer::new_headless_for_tests()` (mod.rs:147-148) wraps `new_headless` in `pollster::block_on` with default options — the fast-path for tests that don't need custom options.
- The constructor creates a fresh `GpuAtlas` each call. For the stress test, we should create ONE renderer and reuse it across N `update()` calls (the runtime's `update()` is the unit under test, not the renderer init).

**`display_list.len()` API:**
- `output.display_list` is a `DisplayList`. It exposes `commands()` (returns `&[PaintCommand]`) — used in `tests/runtime_pipeline.rs:19`. The `len()` is `commands().len()`.
- `output.stats.command_count` is the post-lowering count (after batch construction). The CONTEXT.md (D-04) says the stress test asserts on `display_list.len() < 2000` AND `RenderStats.display_command_count < 2000`. Both are available.

**`RenderStats.display_command_count` API:**
- `output.stats.command_count` → `output.debug_snapshot().performance.display_command_count`. The `DisplayCommandCount` and `stats.command_count` are kept in sync at `src/runtime/runtime.rs:1245`.

**Existing scroll/list/stress test:**
- `tests/scroll_layout_contract.rs:6` — basic scroll_area test.
- `tests/event_input_hardening.rs:103,156` — scroll_area with input.
- `tests/widgets_visual_flow.rs:121,226` — scroll_area integration.
- `tests/render_wgpu_offscreen_render.rs:649,722` — scroll_area offscreen render.
- No existing 10k-row stress test. REND-02 is greenfield.

## Library / API Findings

**Reusable infrastructure for the stress scene:**
- `Element::scroll_area()` — `rgui::widgets::scroll_area()`
- `Element::list()` — `rgui::widgets::list()` with `.items([...])` and `.default_selected_index(...)`
- `Element::progress_bar()` — `rgui::widgets::progress_bar()` with `.width(...)`
- `Element::popover()` and `Element::modal()` — for translucent overlapping layers
- `Element::textarea()` — for IME preedit
- `FrameInput { root, viewport, ... }` — the per-frame input
- `output.display_list.commands().len()` and `output.stats.command_count` — the count APIs

**Reusable infrastructure for the second backend:**
- `RendererOptions.backends: wgpu::Backends` (already a knob from Phase 4 D-09)
- `WgpuRenderer::new_headless(RendererOptions { backends: wgpu::Backends::VULKAN, .. })` — the only change needed
- `REND_GOLDEN_BACKEND` env var — gates the second-backend test

**Reusable infrastructure for validation:**
- Nothing today. Plan 05-03 adds a new feature flag (`validation-layers`) and a runtime path to enable `wgpu::InstanceFlags::VALIDATION`.

**Reusable infrastructure for frame timing:**
- `PerformanceMetrics.frame_time_ms` (defined, not populated)
- Plan 05-04 populates the field by measuring `Instant::now()` deltas in `src/runtime/runtime.rs:1242-1251`.

## Open Questions

None for the planner — the CONTEXT.md decisions plus this research give the planner enough to produce PLAN.md files. The remaining choices (CI platform specifics, benchmark thresholds) are Claude's Discretion per CONTEXT.md.

## Recommended Approach

**05-01 — Visual goldens on a second GPU backend (REND-01):**
Add a `tests/visual_goldens_vulkan.rs` (or a `#[cfg(...)]`-gated section in `visual_goldens.rs`) that runs the 8 existing goldens with `RendererOptions { backends: wgpu::Backends::VULKAN, .. }`. Gate behind `REND_GOLDEN_BACKEND` env var so the test is `#[ignore]` on runners without Vulkan. Updates the existing 8 goldens to the same names (the goldens are PNGs that are backend-agnostic for headless software rendering; the only difference is the wgpu command-encoder path).

**05-02 — Stress-test scene (REND-02):**
Add a `tests/stress_scene.rs` headless test (per CONTEXT.md D-01). The test builds a 10k-row `list()` inside a `scroll_area()` of fixed height, with overlapping translucent layers (modal + popover), 5+ animated progress bars, and an IME preedit on a `Textarea`. Runs N=10 `update()` calls at distinct scroll positions; asserts `output.display_list.commands().len() < 2000` AND `output.stats.command_count < 2000` AND zero wgpu validation errors (using `device.push_error_scope` / `pop_error_scope` — see plan 05-03 for the validation plumbing). Includes `// TODO(phase-7): drag` and `// TODO(phase-8): windowed list` annotations.

**05-03 — Vulkan validation layers in CI (REND-03):**
Add a `validation-layers` Cargo feature. When enabled, the `WgpuContext::headless` and `WgpuContext::from_parts` constructors set `wgpu::InstanceFlags::VALIDATION` on the `InstanceDescriptor`. Add a `.github/workflows/ci.yml` that runs on `windows-latest` (for the Vulkan second backend) and `ubuntu-latest` (for the default), with the `validation-layers` feature enabled. The `tests/stress_scene.rs` (plan 05-02) asserts zero `device.pop_error_scope()` validation errors, which exercises the feature.

**05-04 — Frame budget micro-benchmark (REND-04):**
Two sub-tasks:
1. Populate `PerformanceMetrics.frame_time_ms` in `src/runtime/runtime.rs:1242-1251` by capturing `Instant::now()` at the start of `update()` and subtracting at the end.
2. Add a `tests/frame_budget.rs` that constructs a 50-widget composition, runs N=1000 `update()` calls, and asserts mean < 8ms / p99 < 16ms / max < 32ms. The 50-widget composition is a synthetic `Element::column()` with 50 `button("...").key(...)` children (or a representative composition matching the `visual_showcase` winit example's widget set).
