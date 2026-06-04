# Phase 5: Render Path Stress - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-04
**Phase:** 05-render-path-stress
**Areas discussed:** stress-scene-scope

---

## stress-scene-scope

| Option | Description | Selected |
|--------|-------------|----------|
| Use what's available now | Build the stress scene from today's primitives: 10k-row plain `list()` (not windowed), no DnD, but the rest works (translucent layers, animated progress bars, IME preedit). Annotate with `TODO(phase-7/8)` comments. | ✓ |
| Paint-path only, defer the rest | Skip list + DnD. The example is purely a paint-path stress: 200-widget tree, overlapping translucent layers, animated progress bars, IME preedit, large text shaping (CJK + RTL). | |
| Defer REND-02, ship the rest | Skip 05-02 entirely. Phase 5 ships 05-01 + 05-03 + 05-04. REND-02 gets pushed to a post-Phase-8 phase. | |
| Mock the missing pieces | Build a stress scene that drives the same paint paths WindowedList / DnD will use, with hand-rolled stand-ins (a 10k-row layout that simulates virtualization by clipping, a fake drag overlay that updates a transform). | |

**User's choice:** Use what's available now (Recommended).
**Notes:** Phase 5 cannot fully satisfy REND-02 because WindowedList (Phase 8) and DnD (Phase 7) don't exist yet. Annotate the test source with `// TODO(phase-7): drag` and `// TODO(phase-8): windowed list` so the seams are visible in code review.

### Sub-decision 1: 10k list render mode

| Option | Description | Selected |
|--------|-------------|----------|
| Scrollable, viewport-clipped (Recommended) | The example renders a 10k-row `list()` in a `scroll_area()` of fixed height (~600px). Only the visible rows paint; off-screen rows are clipped by the scroll area. The `display_list` carries all 10k logical rows but the renderer drops off-screen items at lower time. The example is runnable; scroll up/down and watch the per-frame command count stay bounded. | ✓ |
| Full 10k rows in one frame | Render all 10k rows in a single non-scrolled column. The display list has 10k paint commands; the frame budget measurement (05-04) will show the cost. | |
| Both, with a toggle | Two scenes: 'windowed' (scrollable) and 'full'. The stress example switches between them on a keypress. | |
| Both via two separate examples | Two separate examples: `examples/stress_windowed.rs` and `examples/stress_full.rs`. | |

**User's choice:** Scrollable, viewport-clipped (Recommended).
**Notes:** This is the v0 of what Phase 8's `WindowedList` will automate. The test scrolls through 10 distinct positions over N=10 `update()` calls and asserts the visible-window command count is bounded regardless of scroll position.

### Sub-decision 2: Stress delivery (example vs test vs both)

| Option | Description | Selected |
|--------|-------------|----------|
| Headless test only (Recommended) | A `tests/stress_scene.rs` headless test that drives the full stress scene through `WgpuRenderer::new_headless` and `UiRuntime::update`, captures the per-frame `FrameOutput` and `PerformanceMetrics`, and asserts: (a) no panics, (b) `display_list` for the visible window is bounded (rows in the viewport only), (c) per-frame command count is < 5000 (or whatever the budget is). No winit, no window. CI-friendly. | ✓ |
| Winit example only | An `examples/stress_scene.rs` winit example that runs the stress scene in a real window. Interactive (you can scroll the 10k list, watch the progress bars animate). Not CI-friendly because it needs a window. | |
| Both: headless test + winit example | Both. The test pins the bounds; the example is the human-runnable demo. | |

**User's choice:** Headless test only (Recommended).
**Notes:** The interactive winit variant of the stress scene is a v1.x follow-up. 05-02 ships `tests/stress_scene.rs` only.

### Sub-decision 3: Stress test assertions

| Option | Description | Selected |
|--------|-------------|----------|
| Bounded command count per frame (Recommended) | After N=10 `update()` calls (simulating scroll positions), assert: (a) the visible-window's `display_list` has bounded command count (e.g. < 2000 commands regardless of the 10k list), (b) `RenderStats.display_command_count` is similarly bounded, (c) no wgpu error scope reports a validation error. The 8ms frame budget is a separate test (05-04), not duplicated here. | ✓ |
| No panics only | The test just runs and asserts it doesn't panic. No command-count bounds. The weakest assertion; useful as a smoke test. | |
| Bounds + per-frame timings | All of the above PLUS: capture per-frame `Instant::now()` deltas; assert mean frame time < 8ms over N frames; flag if any frame exceeds 16ms (1 frame miss at 60fps). The test prints timings. Catches both correctness (bounds) and rough performance regressions in one place; 05-04 becomes a tighter micro-benchmark. | |
| No assertions, just instrumentation | The test only PRINTS the metrics to stdout. No assertion. Useful for tuning but doesn't fail on regression. | |

**User's choice:** Bounded command count per frame (Recommended).
**Notes:** The 8ms frame budget is 05-04's job; 05-02 owns the bounds invariant only. The 2000 cap is a starting point; tune based on the real measurement.

---

## Claude's Discretion

- **05-01 — Second GPU backend target** (which backend, which CI runner, strict vs tolerance matching). The `RendererOptions.backends: wgpu::Backends` knob is already in place. The second backend is whichever platform's runner is available in CI (DX12 on `windows-latest`, Metal on `macos-latest`, Vulkan on `ubuntu-latest` if a GPU runner is configured). The matching strategy reuses the existing `pixel_diff_stats` tolerance. If a free runner with a real GPU is not available, the second backend is a software fallback gated behind a `REND_GOLDEN_BACKEND` env var.
- **05-03 — Validation layer placement** (always-on, CI-only, or feature-gated). The wgpu 29 API exposes validation through `wgpu::InstanceDescriptor.flags`. The default v1.x choice: enable validation in `tests/*` builds (catches regressions locally) and in CI, but not in `examples/*` (perf cost). A `validation-layers` Cargo feature is a fine-grained alternative if the test-only path proves too costly.
- **05-04 — Frame budget benchmark harness** (`cargo bench` vs custom test, warmup, variance). The 50-widget scene is the existing `visual_showcase` example's widget composition. The harness is a `tests/frame_budget.rs` that runs N=1000 `update()` calls and asserts mean < 8ms / p99 < 16ms. A `benches/` criterion binary is a v1.x follow-up if the test-only harness is too coarse.

## Deferred Ideas

- **winit-runnable `examples/stress_scene.rs`** — the interactive stress example with a real window. v1.x follow-up.
- **`benches/` criterion binary** for the frame budget — v1.x follow-up.
- **CI runners with real GPUs** — paid/self-hosted; v1.x follow-up. Until then, 05-01's "second backend" is a software fallback or runner-provided backend.
- **`WindowedList` (Phase 8) + DnD (Phase 7) in the stress scene** — explicitly out of Phase 5 scope; marked with `// TODO(phase-7/8)` comments in the test source.
- **Per-frame pixel-perfect diff against a "gold master"** — too strict for a real-GPU environment. The existing `pixel_diff_stats` tolerance is the v1.x contract.
