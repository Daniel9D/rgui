use rgui::runtime::{FrameInput, UiRuntime};
use rgui::widgets::text;
use rgui::{Element, Size};

#[test]
fn default_runtime_reports_taffy_first_layout_engine() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column().key("root").child(text("A").key("a")),
        viewport: Size::new(320.0, 200.0),
        ..Default::default()
    });

    assert_eq!(output.layout_engine, "taffy_first");
}

#[test]
fn root_defaults_to_viewport_size_under_taffy_first() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column().key("root").child(text("A").key("a")),
        viewport: Size::new(320.0, 200.0),
        ..Default::default()
    });

    let root = output
        .snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.layout_box("root"))
        .expect("root layout");

    assert_eq!(root.width, 320.0);
    assert_eq!(root.height, 200.0);
}

use rgui::{Overflow, Point, PointerButton, PointerEvent, UiEvent};

#[test]
fn runtime_snapshot_and_scrollbar_use_layout_owned_content_size() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column()
            .key("scroll")
            .height(80.0)
            .overflow(Overflow::Scroll)
            .child(text("A").height(120.0).key("a"))
            .child(text("B").height(120.0).key("b")),
        viewport: Size::new(240.0, 120.0),
        ..Default::default()
    });

    let snapshot = output.snapshot.as_ref().expect("snapshot exists");
    let scroll = snapshot.layout_box("scroll").expect("scroll layout");
    assert!(scroll.content_height >= 240.0);
    assert!(
        output
            .hit_test
            .entries()
            .iter()
            .any(|entry| entry.key.as_deref() == Some("scroll::__scrollbar_thumb"))
    );
}

#[test]
fn clipped_scroll_child_outside_visible_rect_does_not_hit() {
    let mut runtime = UiRuntime::default();
    runtime.update(FrameInput {
        root: Element::column()
            .key("scroll")
            .height(60.0)
            .overflow(Overflow::Scroll)
            .child(text("Tall").height(180.0).key("tall")),
        viewport: Size::new(240.0, 120.0),
        ..Default::default()
    });

    runtime.dispatch(UiEvent::PointerDown(PointerEvent {
        position: Point::new(12.0, 100.0),
        button: Some(PointerButton::Primary),
        modifiers: 0,
    }));

    assert_ne!(runtime.active_key().as_deref(), Some("tall"));
}
