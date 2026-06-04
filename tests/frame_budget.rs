//! Phase 5 / Plan 05-04 (REND-04): Frame-budget micro-benchmark.
//!
//! Builds a 50-widget desktop UI (the canonical mix from PROJECT.md's
//! 60 fps / 8ms constraint) and asserts `UiRuntime::update` over 100
//! iterations meets the 8ms mean / 16ms max per-frame CPU budget.
//!
//! # Release-mode gate
//!
//! The 8ms budget is a release-mode assertion. Debug builds run
//! 10x slower (bounds checks, debug assertions, no inlining) and the
//! 8ms budget is not meaningful there. The tests are gated with
//! `#[cfg(not(debug_assertions))]` and the file header says so.
//!
//! # Run
//!
//! ```text
//! cargo test --release --test frame_budget
//! ```
//!
//! The CI workflow (plan 05-03) runs this target on every push.

#![cfg(not(debug_assertions))]

mod common;

use rgui::runtime::{FrameInput, UiRuntime};
use rgui::Size;

/// Per PROJECT.md's 60 fps constraint: each frame's full pipeline
/// must fit in ~8ms of CPU budget on a modern laptop. The hard
/// ceiling for a single frame is 16.67ms (60fps = 1/60s); we
/// measure mean and max across 100 iterations of `update`.
const MEAN_BUDGET_MS: f32 = 8.0;
const MAX_BUDGET_MS: f32 = 16.0;
const ITERATIONS: usize = 100;

fn measure_iteration_budget() -> (f32, f32) {
    let mut runtime = UiRuntime::default();
    let root = common::build_50_widget_ui();
    let viewport = Size::new(1024.0, 768.0);

    // Warmup: first call may allocate caches / build the tree.
    let _ = runtime.update(FrameInput {
        root: root.clone(),
        viewport,
        ..Default::default()
    });

    let mut frame_times = Vec::with_capacity(ITERATIONS);
    for _ in 0..ITERATIONS {
        let output = runtime.update(FrameInput {
            root: root.clone(),
            viewport,
            ..Default::default()
        });
        let frame_time_ms = output
            .debug_snapshot()
            .performance
            .frame_time_ms;
        frame_times.push(frame_time_ms);
    }

    let sum: f32 = frame_times.iter().sum();
    let mean = sum / ITERATIONS as f32;
    let max = frame_times.iter().copied().fold(0.0_f32, f32::max);
    (mean, max)
}

#[test]
fn frame_budget_50_widget_ui_under_8ms() {
    let (mean, max) = measure_iteration_budget();

    assert!(
        mean < MEAN_BUDGET_MS,
        "50-widget UI mean frame time {mean:.2}ms exceeds the {MEAN_BUDGET_MS}ms budget \
         over {ITERATIONS} iterations (max: {max:.2}ms). The paint / event-dispatch path \
         has regressed. Profile with `cargo flamegraph` and check the recent commits.",
    );
    assert!(
        max < MAX_BUDGET_MS,
        "50-widget UI worst frame {max:.2}ms exceeds the {MAX_BUDGET_MS}ms single-frame \
         budget over {ITERATIONS} iterations (mean: {mean:.2}ms). A single frame is over \
         the 60fps frame budget — check for GC pauses, cache misses, or O(n^2) paint.",
    );
}

#[test]
fn frame_budget_first_frame_is_warmup_excluded() {
    let mut runtime = UiRuntime::default();
    let root = common::build_50_widget_ui();
    let viewport = Size::new(1024.0, 768.0);

    // Discard the first 5 iterations (warmup; first call often
    // allocates caches and is not representative of steady-state).
    for _ in 0..5 {
        let _ = runtime.update(FrameInput {
            root: root.clone(),
            viewport,
            ..Default::default()
        });
    }

    let mut sum = 0.0_f32;
    let mut count = 0;
    for _ in 0..(ITERATIONS - 5) {
        let output = runtime.update(FrameInput {
            root: root.clone(),
            viewport,
            ..Default::default()
        });
        let frame_time_ms = output
            .debug_snapshot()
            .performance
            .frame_time_ms;
        sum += frame_time_ms;
        count += 1;
    }
    let mean = sum / count as f32;

    assert!(
        mean < MEAN_BUDGET_MS,
        "50-widget UI mean frame time (post-warmup) {mean:.2}ms exceeds the {MEAN_BUDGET_MS}ms \
         budget. The first 5 iterations were discarded as warmup.",
    );
}

#[test]
fn frame_budget_50_widget_ui_throughput() {
    // Heavier version of the first test: 1000 iterations. If this
    // passes but the first test fails, the budget is being met on
    // average but the first 100 iterations have a cold-start cost.
    let mut runtime = UiRuntime::default();
    let root = common::build_50_widget_ui();
    let viewport = Size::new(1024.0, 768.0);

    let heavy_iterations = 1_000usize;
    let start = std::time::Instant::now();
    for _ in 0..heavy_iterations {
        let _ = runtime.update(FrameInput {
            root: root.clone(),
            viewport,
            ..Default::default()
        });
    }
    let elapsed = start.elapsed();
    let mean = elapsed.as_secs_f32() * 1000.0 / heavy_iterations as f32;

    assert!(
        mean < MEAN_BUDGET_MS,
        "50-widget UI mean frame time (1000 iter throughput) {mean:.2}ms exceeds the \
         {MEAN_BUDGET_MS}ms budget. The paint / event-dispatch path is too slow in steady state.",
    );
}
