use rgui::runtime::{FrameInput, UiRuntime};
use rgui::widgets::{option, select};
use rgui::{Point, PointerButton, PointerEvent, Size, UiEvent};

fn click(runtime: &mut UiRuntime, point: Point) {
    runtime.dispatch(UiEvent::PointerDown(PointerEvent {
        position: point,
        button: Some(PointerButton::Primary),
        modifiers: 0,
    }));
    runtime.dispatch(UiEvent::PointerUp(PointerEvent {
        position: point,
        button: Some(PointerButton::Primary),
        modifiers: 0,
    }));
}

#[test]
fn select_default_value_seeds_runtime_state_once() {
    let mut runtime = UiRuntime::default();
    let first = select()
        .key("priority")
        .options([option("low", "Low"), option("medium", "Medium")])
        .default_value("medium");

    runtime.update(FrameInput {
        root: first.clone(),
        viewport: Size::new(240.0, 96.0),
        ..Default::default()
    });
    assert_eq!(
        runtime.selected_value("priority").as_deref(),
        Some("medium")
    );

    let frame = runtime.update(FrameInput {
        root: first,
        viewport: Size::new(240.0, 96.0),
        ..Default::default()
    });
    let rect = frame.hit_test.entries()[0].rect;
    click(
        &mut runtime,
        Point::new(rect.origin.x + 8.0, rect.origin.y + 8.0),
    );
    let open = runtime.update(FrameInput {
        root: select()
            .key("priority")
            .options([option("low", "Low"), option("medium", "Medium")])
            .default_value("medium"),
        viewport: Size::new(240.0, 96.0),
        ..Default::default()
    });
    let low = open
        .hit_test
        .entries()
        .iter()
        .find(|entry| entry.key.as_deref() == Some("priority::__option::0"))
        .expect("low option hit target")
        .rect;
    click(
        &mut runtime,
        Point::new(low.origin.x + 8.0, low.origin.y + low.size.height * 0.5),
    );
    assert_eq!(runtime.selected_value("priority").as_deref(), Some("low"));

    let rebuilt = select()
        .key("priority")
        .options([option("low", "Low"), option("medium", "Medium")])
        .default_value("medium");
    runtime.update(FrameInput {
        root: rebuilt,
        viewport: Size::new(240.0, 96.0),
        ..Default::default()
    });

    assert_eq!(runtime.selected_value("priority").as_deref(), Some("low"));
}

#[test]
fn selected_value_survives_option_reorder() {
    let mut runtime = UiRuntime::default();
    let first = select()
        .key("priority")
        .options([option("low", "Low"), option("medium", "Medium")])
        .default_value("medium");

    runtime.update(FrameInput {
        root: first,
        viewport: Size::new(240.0, 96.0),
        ..Default::default()
    });

    let reordered = select()
        .key("priority")
        .options([option("medium", "Medium"), option("low", "Low")]);
    runtime.update(FrameInput {
        root: reordered,
        viewport: Size::new(240.0, 96.0),
        ..Default::default()
    });

    assert_eq!(
        runtime.selected_value("priority").as_deref(),
        Some("medium")
    );
    assert_eq!(runtime.selected_index("priority"), Some(0));
}
