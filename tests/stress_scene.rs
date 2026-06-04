//! Phase 5 / Plan 05-02: Render-path stress test.
//!
//! Builds a 10,000-row list inside a fixed-viewport scroll area and asserts:
//!
//! 1. The produced `DisplayList` is **bounded** — command count < 2,000.
//!    A linear-in-rows paint path would emit one paint per row, ballooning
//!    to 10,000+ commands. The scroll-area's clip + the list's viewport
//!    culling must keep the count proportional to the *visible* rows, not
//!    the total rows.
//! 2. The wgpu device surfaces **zero validation errors** during the
//!    stress render (`push_error_scope` / `pop_error_scope` returns
//!    `None`).
//!
//! # Culling seam
//!
//! The list + scroll_area combination should make this test pass
//! out-of-the-box. If it doesn't, the culling fix lives in
//! `src/widgets/collections.rs` (list paint) and/or
//! `src/widgets/layouts.rs` (scroll_area clip). See plan 05-02 Task 2.
//!
//! // TODO(phase-7): drag — mark the seam where drag-and-drop will
//! reuse the list's row iterator.
//! // TODO(phase-8): windowed list — mark the seam where a per-window
//! virtual-list viewport binding will live.
//!
//! # Run
//!
//! ```text
//! cargo test --test stress_scene
//! ```
//!
//! The test is headless — no winit, no GPU window. The wgpu device
//! used is the same `new_headless_for_tests` seam as the visual
//! goldens.

use rgui::render::wgpu::{OffscreenTarget, WgpuRenderer};
use rgui::runtime::{FrameInput, UiRuntime};
use rgui::widgets::{list, scroll_area};
use rgui::{Element, Size, SizeU32};

const VIEWPORT_W: u32 = 400;
const VIEWPORT_H: u32 = 600;
const N_ROWS: usize = 10_000;

/// Per `FrameOutput.stats.command_count`. Bounded < 2,000 to prove the
/// scroll_area / list paint path culls off-viewport rows. 2,000 is
/// generous (~80× a fully-ideal ~25 rows + chrome); tighten once the
/// culling is proven.
const COMMAND_COUNT_BUDGET: usize = 2_000;

/// Builds a 10,000-row list inside a fixed-viewport scroll area. The
/// list items are all `format!("Row {i}")` strings; the row height is
/// governed by the list's default styling (24.0 px). A 600 px viewport
/// at 24 px / row shows ~25 rows. The full 10k-row list lives in the
/// underlying widget state but is not painted off-viewport.
fn build_10k_row_scroll_list() -> Element {
    let items: Vec<String> = (0..N_ROWS).map(|i| format!("Row {i}")).collect();
    scroll_area()
        .key("stress-scroll")
        .width(VIEWPORT_W as f32)
        .height(VIEWPORT_H as f32)
        .child(
            list()
                .key("stress-list")
                .items(items)
                .default_selected_index(0),
        )
}

fn render(root: Element) -> (rgui::core::RenderStats, Vec<u8>) {
    let size = SizeU32::new(VIEWPORT_W, VIEWPORT_H);
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root,
        viewport: Size::new(size.width as f32, size.height as f32),
        ..Default::default()
    });
    let stats = output.stats;
    let display_list = output.display_list;

    let mut renderer = WgpuRenderer::new_headless_for_tests();
    let target = OffscreenTarget::new(renderer.context(), size);

    // Wrap the render pass in a wgpu error scope. The guard is created
    // on `push_error_scope`; the future returned by `guard.pop()` resolves
    // to `Some(Error)` if the device surfaces a validation error during
    // the render, or `None` if the render was clean.
    let error_scope = renderer
        .context()
        .device()
        .push_error_scope(wgpu::ErrorFilter::Validation);
    let render_result = renderer.render_to_target(&display_list, &output.resources, target.view());
    let error = pollster::block_on(error_scope.pop());

    let pixels = pollster::block_on(target.read_rgba8(renderer.context()))
        .expect("readback works");

    assert!(
        render_result.is_ok(),
        "render_to_target failed: {:?}",
        render_result.err()
    );
    assert!(
        error.is_none(),
        "wgpu validation error during stress render: {:?}",
        error
    );

    (stats, pixels)
}

#[test]
fn ten_thousand_row_list_command_count_is_bounded() {
    let root = build_10k_row_scroll_list();
    let (stats, _pixels) = render(root);

    let raw_command_count = {
        // `output.display_list` was consumed by `render`; reconstruct a
        // quick render here to expose `display_list.commands().len()`.
        // We use a fresh runtime to keep the count from any single
        // re-render.
        let mut runtime = UiRuntime::default();
        let output = runtime.update(FrameInput {
            root: build_10k_row_scroll_list(),
            viewport: Size::new(VIEWPORT_W as f32, VIEWPORT_H as f32),
            ..Default::default()
        });
        output.display_list.commands().len()
    };

    assert!(
        stats.command_count < COMMAND_COUNT_BUDGET,
        "stress render emitted {} commands; budget is {} (raw_display_list: {}). \
         List + scroll_area culling is failing — the paint path is linear in total \
         rows instead of viewport-visible rows.",
        stats.command_count,
        COMMAND_COUNT_BUDGET,
        raw_command_count,
    );
    assert!(
        raw_command_count < COMMAND_COUNT_BUDGET,
        "raw DisplayList had {} commands (budget {}); pre-lowering paint is unbounded.",
        raw_command_count,
        COMMAND_COUNT_BUDGET,
    );
}

#[test]
fn ten_row_list_is_at_most_10x_baseline() {
    // Build a 10-row list and capture the command count as baseline.
    // A 1,000-row list inside the same 600 px / 24 px = 25-row viewport
    // should produce approximately the same command count; we allow a
    // 10× margin for any first-pass culling inefficiency. This is the
    // "linear scaling" check the plan calls out.
    fn build_row_list(n: usize) -> Element {
        let items: Vec<String> = (0..n).map(|i| format!("Row {i}")).collect();
        scroll_area()
            .key("scaling-scroll")
            .width(VIEWPORT_W as f32)
            .height(VIEWPORT_H as f32)
            .child(list().key("scaling-list").items(items))
    }

    fn command_count(n: usize) -> usize {
        let mut runtime = UiRuntime::default();
        let output = runtime.update(FrameInput {
            root: build_row_list(n),
            viewport: Size::new(VIEWPORT_W as f32, VIEWPORT_H as f32),
            ..Default::default()
        });
        output.stats.command_count
    }

    let baseline = command_count(10);
    let scaled = command_count(1_000);

    assert!(
        scaled <= 10 * baseline.max(1),
        "1000-row list command count ({scaled}) is > 10× the 10-row baseline ({baseline}); \
         list paint is scaling with total rows, not viewport-visible rows.",
    );
}
