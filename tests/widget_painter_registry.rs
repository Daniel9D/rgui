//! Integration test for API-04 + CUST-01..03: the
//! `register_widget_painter` / `unregister_widget_painter` round-trip
//! works end-to-end.
//!
//! Defines a `CountingPainter` with an `AtomicUsize` counter, registers
//! it for `WidgetKind::Badge`, builds a `badge("test")` element, runs
//! `UiRuntime::default()` + `update(FrameInput { .. })`, and asserts the
//! painter was invoked at least once. Then unregisters and confirms the
//! counter is unchanged on a second update.

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

struct CountingPainter;

impl WidgetPainter for CountingPainter {
    fn background_color(&self, _ctx: &PaintCtx<'_>) -> Color {
        Color::rgb(0, 0, 0)
    }

    fn paint_content(&self, _ctx: &mut PaintCtx<'_>, _cmds: &mut Vec<PaintedCommand>) {
        PAINT_CALLS.fetch_add(1, Ordering::SeqCst);
    }
}

static COUNTING_PAINTER: CountingPainter = CountingPainter;

#[test]
fn register_unregister_round_trip_invokes_painter() {
    // Reset the counter so the test is order-independent.
    PAINT_CALLS.store(0, Ordering::SeqCst);

    // Register the custom painter for the Badge kind.
    register_widget_painter(WidgetKind::Badge, &COUNTING_PAINTER);

    // Render one frame with a badge element.
    let mut runtime = UiRuntime::default();
    let _ = runtime.update(FrameInput {
        root: badge("test"),
        ..Default::default()
    });

    // The painter must have been invoked at least once during the
    // single frame of paint dispatch.
    let after_register = PAINT_CALLS.load(Ordering::SeqCst);
    assert!(
        after_register > 0,
        "CountingPainter should be invoked at least once after register (got {after_register})"
    );

    // Unregister the painter.
    let prev = unregister_widget_painter(WidgetKind::Badge);
    assert!(
        prev.is_some(),
        "unregister_widget_painter should return the previously-registered painter"
    );

    // Render another frame. The custom painter is no longer in the
    // registry, so the counter must not advance.
    let _ = runtime.update(FrameInput {
        root: badge("test"),
        ..Default::default()
    });
    let after_unregister = PAINT_CALLS.load(Ordering::SeqCst);
    assert_eq!(
        after_register, after_unregister,
        "CountingPainter should NOT be invoked after unregister"
    );
}
