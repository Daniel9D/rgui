use rgui::runtime::{FrameInput, UiRuntime};
use rgui::widgets::text;
use rgui::{
    Element, Overflow, Point, PointerButton, PointerEvent, Size, UiEvent, Vec2, WheelDeltaMode,
    WheelEvent,
};

fn scroll_app() -> rgui::Element {
    Element::column()
        .key("scroll")
        .height(80.0)
        .overflow(Overflow::Scroll)
        .child(
            Element::column()
                .child(text("A").height(120.0))
                .child(text("B").height(120.0)),
        )
}

fn update(runtime: &mut UiRuntime) -> rgui::runtime::FrameOutput {
    runtime.update(FrameInput {
        root: scroll_app(),
        viewport: Size::new(320.0, 120.0),
        ..FrameInput::default()
    })
}

#[test]
fn scrollbar_geometry_uses_theme_metrics() {
    let mut theme = rgui::Theme::light();
    theme.widgets.metrics.scrollbar.width = 10.0;
    theme.widgets.metrics.scrollbar.padding = 12.0;
    theme.widgets.metrics.scrollbar.min_thumb_height = 34.0;

    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: scroll_app(),
        viewport: Size::new(320.0, 120.0),
        theme,
        ..FrameInput::default()
    });
    let scroll = output
        .snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.layout_box("scroll"))
        .expect("scroll layout");
    let thumb = output
        .hit_test
        .entries()
        .iter()
        .find(|entry| entry.key.as_deref() == Some("scroll::__scrollbar_thumb"))
        .expect("scrollbar thumb hit target")
        .rect;

    assert_eq!(thumb.size.width, 10.0);
    assert!(thumb.size.height >= 34.0);
    assert!((thumb.origin.x - (scroll.x + scroll.width - 24.0)).abs() < 0.01);
    assert!((thumb.origin.y - (scroll.y + 12.0)).abs() < 0.01);
}

#[test]
fn wheel_delta_is_normalized_and_clamped_immediately() {
    let mut runtime = UiRuntime::default();
    update(&mut runtime);

    runtime.dispatch(UiEvent::Wheel(WheelEvent {
        position: Point::new(20.0, 20.0),
        delta: Vec2::new(0.0, 10_000.0),
        mode: WheelDeltaMode::Lines,
    }));

    let offset = runtime.scroll_offset("scroll").expect("scroll offset");
    assert!(offset.y <= 160.0);
}

#[test]
fn scrollbar_thumb_captures_pointer_until_release() {
    let mut runtime = UiRuntime::default();
    let output = update(&mut runtime);
    let thumb = output
        .hit_test
        .entries()
        .iter()
        .find(|entry| entry.key.as_deref() == Some("scroll::__scrollbar_thumb"))
        .expect("scrollbar thumb hit target")
        .rect;
    let start = Point::new(
        thumb.origin.x + 1.0,
        thumb.origin.y + thumb.size.height * 0.5,
    );

    runtime.dispatch(UiEvent::PointerDown(PointerEvent {
        position: start,
        button: Some(PointerButton::Primary),
        modifiers: 0,
    }));
    assert_eq!(
        runtime.pointer_capture_key().as_deref(),
        Some("scroll::__scrollbar_thumb")
    );

    runtime.dispatch(UiEvent::PointerMove(PointerEvent {
        position: Point::new(start.x, 95.0),
        button: Some(PointerButton::Primary),
        modifiers: 0,
    }));
    runtime.dispatch(UiEvent::PointerUp(PointerEvent {
        position: Point::new(start.x, 95.0),
        button: Some(PointerButton::Primary),
        modifiers: 0,
    }));

    assert!(runtime.scroll_offset("scroll").expect("scroll offset").y > 0.0);
    assert!(runtime.pointer_capture_key().is_none());
}
