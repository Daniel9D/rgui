---
phase: 05-render-path-stress
plan: 04
subsystem: performance, rendering
tags: [performance, frame-budget, perf-metric, 8ms-budget, rende-stats]

# Dependency graph
requires:
  - phase: 05-render-path-stress (plan 02)
    provides: ListPainter culling (REND-02), which keeps the 50-widget UI's paint cost linear in visible widgets
  - phase: 05-render-path-stress (plan 03)
    provides: .github/workflows/ci.yml (the file Task 4 of this plan appends to)
provides:
  - PerformanceMetrics.frame_time_ms is now populated by UiRuntime::update (was zeroed)
  - tests/common/mod.rs: build_50_widget_ui() — the canonical 50-widget desktop UI used by the frame_budget test
  - tests/frame_budget.rs: 3 release-mode tests asserting 50-widget UI mean < 8ms CPU, max < 16ms
  - CI step `Frame budget test` (cargo test --release --test frame_budget) on ubuntu-latest + windows-latest
  - lib unit test `frame_time_ms_field_is_populated_by_runtime` in src/core/snapshot.rs
affects: [future perf-regression work, REND-04 traceability]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Frame-budget test pattern: build canonical N-widget UI, run K iterations of update(), assert mean+max over K against a hard budget. Release-mode-only gate via #[cfg(not(debug_assertions))]"
    - "Wall-clock-instrumented update(): capture std::time::Instant::now() at function entry, compute elapsed() at the PerformanceMetrics construction site, populate frame_time_ms in milliseconds. Avoids adding a profiler dependency for a one-off measurement"
    - "Shared test helper at tests/common/mod.rs: re-usable UI builder for any future perf-regression test; follows the Rust integration test 'shared mod' convention"

key-files:
  created:
    - tests/common/mod.rs
    - tests/frame_budget.rs
  modified:
    - src/runtime/runtime.rs
    - src/core/snapshot.rs
    - .github/workflows/ci.yml

key-decisions:
  - "Wall-clock Instant::now() over a profiler-based measurement: the budget is a hard ceiling, not a statistical regression; Instant::now() is monotonic with microsecond precision and adds zero deps. Profilers (perf / Instruments / VTune) are the right tool for *why* a frame is slow, not for asserting the budget is met"
  - "Release-mode gate via #[cfg(not(debug_assertions))]: debug builds disable optimizations + add bounds checks + debug assertions + format! strings throughout the paint path. A debug update is typically 30-80ms; the 8ms budget is meaningless there. The standard Rust pattern for 'this test only makes sense in release'"
  - "frame_time_ms is wall-clock duration of the entire update() call (event dispatch + layout + paint + display-list assembly). The wgpu submit + present cost is NOT measured here — that is a host-side concern and is profiled separately on the host's render path. Documented in the rustdoc at the construction site"
  - "50-widget composition chosen to be representative of a typical desktop UI (toolbar + search input + scrolling data + form controls + action buttons) — the exact mix from PROJECT.md's 60fps constraint. Trimmed to 50 by removing slider/progress widgets that don't have a public value-builder (the SliderSpec/ProgressBarSpec are #[non_exhaustive] and the Element builder API has no per-widget .value() for sliders)"
  - "Three frame_budget tests with different iteration counts and warmup strategies: 100-iter mean+max catches the per-frame budget, 100-iter-with-warmup isolates the steady-state cost from cold-start, 1000-iter throughput catches budget regressions that only show up over long runs"
  - "Used the existing `input().placeholder()` and `checkbox().checked()` setters on Element — no new builder API needed for the helper"

patterns-established:
  - "Perf-regression test pattern: tests/<name>_budget.rs with #[cfg(not(debug_assertions))], shared UI builder in tests/common/, 3 tests covering short/warmup/long-run budgets"
  - "frame_time_ms as the canonical paint-path cost measurement — populates the field at the update() entry/exit and exposes it via output.debug_snapshot().performance"

requirements-completed: [REND-04]

# Metrics
duration: 14 min
completed: 2026-06-04
---
# Phase 5 Plan 4: Frame budget micro-benchmark test Summary

**Populated `PerformanceMetrics.frame_time_ms` (was zeroed), built the canonical 50-widget desktop UI, and shipped a release-mode integration test that asserts the 8ms CPU budget — REND-04 satisfied. The CI workflow now runs the frame-budget test on every push.**

## Performance

- **Duration:** 14 min
- **Started:** 2026-06-04T14:00:00Z
- **Completed:** 2026-06-04T14:14:00Z
- **Tasks:** 4 (4 atomic commits)
- **Files modified:** 5 (2 created, 3 modified)

## Accomplishments

- **frame_time_ms now populated**: `UiRuntime::update` captures `std::time::Instant::now()` at function entry and computes `elapsed().as_secs_f32() * 1000.0` at the `PerformanceMetrics` construction site. The wall-clock duration (event dispatch + layout + paint + display-list assembly) is now written into `snapshot.performance.frame_time_ms`. The wgpu submit + present cost is intentionally excluded (host-side concern).
- **Sanity-check unit test**: `frame_time_ms_field_is_populated_by_runtime` in `src/core/snapshot.rs` builds a 3-child element tree, calls `update`, and asserts `frame_time_ms > 0.0`. Catches the regression where the field is accidentally zeroed again.
- **Shared UI helper**: `tests/common/mod.rs::build_50_widget_ui()` builds the canonical 50-widget desktop UI from PROJECT.md's 60fps constraint — 1 root + 1 toolbar row + 5 toolbar buttons + 1 input + 1 body column + (10 labels + 5 boxes × 3 children + 5 lists + 5 checkboxes) + 1 footer row + 2 footer buttons + 3 footer-note labels = 50 widgets. Uses the existing `Element::column()`/`row()`/`button()`/`text()`/`input()`/`list()`/`checkbox()` builders — no new public API was needed.
- **3 frame_budget tests** (all `#[cfg(not(debug_assertions))]`-gated):
  - `frame_budget_50_widget_ui_under_8ms` — 100 iterations, asserts mean < 8ms and max < 16ms (one-frame spike budget).
  - `frame_budget_first_frame_is_warmup_excluded` — discards the first 5 iterations (cache warmup), asserts iterations 5..100 mean < 8ms.
  - `frame_budget_50_widget_ui_throughput` — 1000 iterations, asserts per-frame mean < 8ms (catches budget regressions that only show up over long runs).
- **CI step added**: `Frame budget test` step on `ubuntu-latest` + `windows-latest`, after the Vulkan goldens step. Runs `cargo test --release --test frame_budget`. The 50-widget UI's 8ms mean holds on a release build; debug builds skip the test via the `cfg(not(debug_assertions))` gate.
- **Test runs in 0.55s** on this machine in release mode (100-iter test = 0.18s, 1000-iter throughput = 0.30s, warmup test = 0.07s) — fast enough to run on every push without slowing CI down.

## Task Commits

Each task was committed atomically:

1. **Task 1: Populate `frame_time_ms` in `UiRuntime::update`** — `4205973` (feat)
2. **Task 2: Add the 50-widget builder helper** — combined with Task 3 in `a9e1df3` (test)
3. **Task 3: Add `tests/frame_budget.rs`** — combined with Task 2 in `a9e1df3` (test)
4. **Task 4: Wire `cargo test --release --test frame_budget` into the CI workflow** — `7d885ca` (ci)

## Files Created/Modified

- `src/runtime/runtime.rs` — `update` captures `Instant::now()` at entry; `PerformanceMetrics` construction site computes `frame_start.elapsed().as_secs_f32() * 1000.0` and writes to `frame_time_ms`
- `src/core/snapshot.rs` — new lib unit test `frame_time_ms_field_is_populated_by_runtime`
- `tests/common/mod.rs` — new shared test helper `build_50_widget_ui()`
- `tests/frame_budget.rs` — new release-mode integration test with 3 budget assertions
- `.github/workflows/ci.yml` — new `Frame budget test` step in the test job

## Deviations from Plan

- **Tasks 2 + 3 combined into one commit (`a9e1df3`)**: the `tests/common/mod.rs` helper is only useful as the input to the `tests/frame_budget.rs` test, and splitting them would create an intermediate commit where the test file references a missing `mod common;`. The combined commit keeps `cargo test --release --test frame_budget` green at every commit.
- **50-widget composition**: the plan suggested 5 sliders + 5 progress bars in the mix. The `SliderSpec` and `ProgressBarSpec` are `#[non_exhaustive]` and the existing `Element` builder API has no per-widget `.value(f32)` setter. To stay within the 50-widget target without expanding the public API, the helper uses 5 lists + 5 checkboxes + 5 boxes instead. The mix is still representative of a desktop UI (toolbar + search + scrolling data + form controls + actions), and the file's docstring documents the exact widget count breakdown.

## Verification

| Step | Status |
|------|--------|
| `cargo build --lib` (default) | ✓ Clean |
| `cargo test --lib` | ✓ 16/16 pass (15 pre-existing + 1 new `frame_time_ms_field_is_populated_by_runtime`) |
| `cargo test --release --test frame_budget` | ✓ 3/3 pass (8ms / 16ms / 1000-iter budgets all met) |
| `cargo test --test frame_budget` (debug) | ✓ Compiles, tests skipped by `#[cfg(not(debug_assertions))]` |
| `output.debug_snapshot().performance.frame_time_ms > 0.0` for any non-trivial `update` | ✓ Verified by the new lib unit test |
| CI workflow includes the `cargo test --release --test frame_budget` step | ✓ Step added to test job in `ci.yml` |
| The 50-widget UI helper is reusable | ✓ In `tests/common/mod.rs`; can be imported by future perf tests |

## Issues Encountered

- **None substantive.** The release build was a fresh compile (40s) on this developer's machine because no prior release artifacts existed. CI caches via `Swatinem/rust-cache@v2` so the cost is one-time per CI runner.
- **Pre-existing Windows MSVC LNK1318 PDB-size limit** (flagged in plan 05-03's SUMMARY) is unaffected by this plan — the frame_budget test runs in release mode where the dep tree's debug symbols are stripped.

## Next Phase Readiness

Phase 5 is complete. All 4 plans have shipped:
- 05-01: REND-01 (visual goldens on second GPU backend)
- 05-02: REND-02 (stress-scene + list culling)
- 05-03: REND-03 (validation-layers + CI)
- 05-04: REND-04 (frame budget)

The next phase is Phase 6 (Public API Hardening) per ROADMAP.md. Phase 6 has its own plans to be created via `/gsd-plan-phase 6 ${GSD_WS}`.
