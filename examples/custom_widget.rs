//! Phase 6 / Plan 06-03: writing a custom widget.
//!
//! Demonstrates the `WidgetPainter` extension point. Defines a
//! `StatusPillPainter` that paints a green pill background, registers
//! it for `WidgetKind::Badge`, builds a `badge("online")` element, runs
//! `UiRuntime::update`, and prints a summary.
//!
//! Run with: `cargo run --example custom_widget`

use std::sync::atomic::{AtomicUsize, Ordering};

use rgui::core::Color;
use rgui::core::WidgetKind;
use rgui::runtime::FrameInput;
use rgui::runtime::UiRuntime;
use rgui::runtime::paint::{
    PaintCtx, PaintedCommand, WidgetPainter, register_widget_painter, unregister_widget_painter,
};
use rgui::widgets::badge;

static PAINT_CALLS: AtomicUsize = AtomicUsize::new(0);

/// A custom painter that paints a green pill background and counts
/// how many times `paint_content` was invoked.
struct StatusPillPainter;

impl WidgetPainter for StatusPillPainter {
    fn background_color(&self, _ctx: &PaintCtx<'_>) -> Color {
        Color::rgb(0, 128, 0)
    }

    fn paint_content(&self, ctx: &mut PaintCtx<'_>, cmds: &mut Vec<PaintedCommand>) {
        PAINT_CALLS.fetch_add(1, Ordering::SeqCst);
        cmds.push(ctx.draw_rect(ctx.rect, Color::rgb(0, 200, 0), 8.0, 0));
    }
}

static STATUS_PILL_PAINTER: StatusPillPainter = StatusPillPainter;

fn main() {
    // Register the custom painter for the Badge kind
    register_widget_painter(WidgetKind::Badge, &STATUS_PILL_PAINTER);

    // Build a badge element — when painted, the runtime will invoke
    // StatusPillPainter instead of the default Badge painter.
    let root = badge("online");

    // Render one frame
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root,
        ..Default::default()
    });

    println!(
        "Rendered {} paint commands (StatusPillPainter invoked {} time(s))",
        output.display_list.commands().len(),
        PAINT_CALLS.load(Ordering::SeqCst),
    );

    // Unregister on shutdown
    unregister_widget_painter(WidgetKind::Badge);
}
