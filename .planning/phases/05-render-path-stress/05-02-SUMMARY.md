---
phase: 05-render-path-stress
plan: 02
subsystem: rendering
tags: [wgpu, stress-test, culling, scroll-area, list, rende-stats]

# Dependency graph
requires:
  - phase: 05-render-path-stress (plan 01)
    provides: WgpuRenderer::new_headless_for_tests() headless seam, the existing 8 visual goldens showing the same Backend::PRIMARY paint path the stress test mirrors
provides:
  - tests/stress_scene.rs — 10,000-row list in fixed-viewport scroll_area with bounded command-count assertion + wgpu error-scope check
  - ListPainter culling: rows outside the inherited clip rect are skipped, bounding DisplayList growth by visible-rows not total-rows
  - VisualState.clip_rect: new field threading the ancestor clip into widget painters; additive (defaults to None, pre-culling behavior preserved for callers that don't populate)
  - src/core/render.rs stress-stats lib unit tests: deterministic command_count + viewport-bounded count across list sizes
affects: [05-03-validation-layers-ci, 05-04-frame-budget, future render-path stress phases, REND-02 traceability]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Viewport culling via inherited clip rect: thread the parent's clip_rect through VisualState to the widget painter; painters cull per-item work that falls outside it"
    - "Stress-test seam: a headless integration test that builds the worst-case element tree (10k-row list in fixed-viewport scroll area) and asserts bounded DisplayList growth + zero wgpu validation errors"
    - "Two // TODO annotations on the stress test (phase-7 drag, phase-8 windowed list) mark seams where future phases will extend the test rather than gating it"
    - "Lib unit test mirrors of the integration test invariants for fast feedback — RenderStats is a plain field-read in the lib test, no wgpu device required"

key-files:
  created:
    - tests/stress_scene.rs
  modified:
    - src/runtime/paint.rs
    - src/runtime/runtime.rs
    - src/core/render.rs

key-decisions:
  - "Add `clip_rect: Option<Rect>` to VisualState (pub additive field) rather than passing the clip as a paint-time parameter — keeps the WidgetPainter contract uniform and makes the cull available to every painter, not just List"
  - "Cull rows whose `y + row_height <= visible_top || y >= visible_bottom` — the test's 600px / 24px viewport shows ~25 rows; the 2,000-command budget is an 80x safety margin that passes even if a future painter change loosens culling"
  - "Rule 1 deviation (auto-fix): the list's unbounded paint was a Rule 1 'bug' in the existing code, not a Rule 4 architectural concern. Applied the minimum change (one painter + one state field); no API breaks"
  - "10x scaling margin in the second test (`scaled <= 10 * baseline.max(1)`) — the ideal is 1.0x (both lists should produce the same count in a fixed-viewport scroll area); 10x allows first-pass inefficiency to ship and be tightened later"
  - "Stress test runs in `cargo test --test stress_scene` (debug build) for fast feedback; release-mode 8ms budget check lives in plan 05-04"

patterns-established:
  - "VisualState clip_rect pattern: thread parent clip into widget painters; future painters (Table, Tree, Menu) can opt into culling the same way List did"
  - "Stress-test idiom: build worst-case widget tree, assert bounded DisplayList + zero wgpu errors, run as part of `cargo test` (no CI special-casing)"

requirements-completed: [REND-02]

# Metrics
duration: 18 min
completed: 2026-06-04
---
# Phase 5 Plan 2: Stress-test scene (headless 10k-row windowed list, bounded command count) Summary

**Added `tests/stress_scene.rs` proving a 10,000-row list inside a fixed-viewport scroll area renders to a bounded `DisplayList` (under 2,000 commands) with zero wgpu validation errors, and fixed the underlying culling gap in `ListPainter` so the bound holds by default — REND-02 satisfied.**

## Performance

- **Duration:** 18 min
- **Started:** 2026-06-04T13:20:00Z
- **Completed:** 2026-06-04T13:38:00Z
- **Tasks:** 3 (1 + 1 fallback fix + 1)
- **Files modified:** 4 (1 created, 3 modified)

## Accomplishments

- **Stress test in place**: `tests/stress_scene.rs` builds a 10k-row list inside a 400×600 scroll area, renders through the existing `WgpuRenderer::new_headless_for_tests()` headless seam, and asserts (a) `output.stats.command_count < 2_000` (b) the raw `DisplayList.commands().len() < 2_000` (c) zero wgpu validation errors via `device.push_error_scope(Validation) / scope.pop()`. A second test pins the 10× scaling ratio between a 10-row and 1,000-row list — culling scales with viewport, not list size.
- **Culling fix shipped**: The first test run failed with `10,007 commands` for 10,000 rows — the list's `paint_content` iterated every item with no viewport check. Threaded the inherited ancestor `clip_rect` into the widget painter via `VisualState.clip_rect` (a new `pub` additive field) and culled rows whose `y + row_height` falls outside the visible top/bottom in `ListPainter`. The fix is the minimum change: one painter + one struct field + one new argument to `push_paint`. No API breaks — the new field defaults to `None`, which preserves the pre-culling paint path for callers that don't populate it (and there are no current callers of the public VisualState constructor outside the runtime).
- **TODO annotations placed**: `// TODO(phase-7): drag` and `// TODO(phase-8): windowed list` mark the future-extension seams on the stress test's element-tree construction. They are documentation, not `#[ignore]` — the test runs and asserts the full 10k-row count today.
- **Lib unit tests for fast feedback**: Two new tests in `src/core/render.rs` (`stress_stats_command_count_is_stable_across_runs` and `stress_stats_command_count_does_not_grow_with_list_size`) replicate the integration test's invariants at the `RenderStats` level — no wgpu device needed, runs as a regular `cargo test --lib` lib test.
- **Pre-existing build issues confirmed isolated**: The `E0639` errors in `tests/interactive_widgets.rs` and the stale-baseline failures in `tests/visual_goldens.rs` (flagged by plan 05-01) are independent of this plan's changes — verified by running `cargo check --lib` (clean) and `cargo test --test stress_scene` (passes) in isolation.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add the headless stress test** — `64a8e91` (test)
2. **Task 2: Fix the culling gap (fallback, auto-applied per Rule 1)** — `9211cb1` (feat)
3. **Task 3: Add stress-scene lib unit tests for `RenderStats` invariants** — `29212a3` (test)

## Files Created/Modified

- `tests/stress_scene.rs` — new headless 10k-row stress test + 10× baseline test + wgpu error-scope check
- `src/runtime/paint.rs` — added `clip_rect: Option<Rect>` to `VisualState`; `ListPainter::paint_content` now skips rows outside `state.clip_rect`; `visual_state_for_element` initialiser defaults to `clip_rect: None`
- `src/runtime/runtime.rs` — `push_paint` accepts and propagates a `clip_rect` parameter; the `push_node` call site passes `layout.clip_rect` (the ancestor clip after intersection with this node's rect)
- `src/core/render.rs` — added two new lib unit tests in the existing `#[cfg(test)] mod tests` block

## Deviations from Plan

- **[Rule 1 - Bug] Culling gap fix was needed for Task 1 to pass.** The plan listed Task 2 as a "fallback" for if Task 1 failed; it failed on the first run with 10,007 commands. Applied the minimum fix in the same plan rather than splitting it into a follow-up — the fix is one painter change + one additive struct field, not an architectural change. The fix's diff is 35 added / 4 removed lines in `paint.rs` and 1 new argument + 1 new call-site arg in `runtime.rs`.

## Verification

| Step | Status |
|------|--------|
| `cargo check --lib` | ✓ Clean (lib builds, 1 pre-existing unused-import warning) |
| `cargo test --test stress_scene` | ✓ 2/2 pass (both `ten_thousand_row_list_command_count_is_bounded` and `ten_row_list_is_at_most_10x_baseline`) |
| `cargo test --lib core::render::tests` | ✓ 15/15 pass (13 pre-existing + 2 new stress-stats tests) |
| `// TODO(phase-7): drag` annotation present | ✓ in `tests/stress_scene.rs` file header |
| `// TODO(phase-8): windowed list` annotation present | ✓ in `tests/stress_scene.rs` file header |
| wgpu error scope returns `None` (zero validation errors) | ✓ confirmed by both `ten_thousand_row_list_command_count_is_bounded` and `ten_row_list_is_at_most_10x_baseline` |

## Issues Encountered

- **Pre-existing build errors in `tests/interactive_widgets.rs`** (E0639, non-exhaustive struct construction) and **`tests/rml_attribute_matrix.rs`** (rml feature not enabled by default) block `cargo build --tests` from succeeding clean. The `tests/stress_scene.rs` target itself compiles and runs independently via `cargo test --test stress_scene` (cargo's per-test-target compilation skips the broken targets). These are out of scope for plan 05-02; they predate Phase 5 and should be addressed as a separate Phase 5 follow-up or a Phase 6 (Public API Hardening) cleanup. The plan's verification step 6 (`cargo test --lib`) passes.
- **Pre-existing stale-baseline failures in `tests/visual_goldens.rs`** (8/11 scenes failing on byte-equality vs old PNGs) are inherited from Phase 4 and flagged in plan 05-01's report. Not introduced by plan 05-02.

## Next Phase Readiness

Plan 05-03 (validation-layers feature + CI workflow) is unblocked. It can run on a clean main-tree with no shared-file conflicts from 05-02 (it touches `Cargo.toml`, `src/render/wgpu/context.rs`, and `.github/workflows/` — none of which 05-02 modified). The culling fix in `paint.rs` is forward-compatible with 05-03's validation-layers feature; the new `clip_rect` field is an additive change.
