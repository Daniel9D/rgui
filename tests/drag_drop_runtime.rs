use rgui::runtime::{FrameInput, UiRuntime};
use rgui::widgets::button;
use rgui::{Point, PointerButton, PointerEvent, Size, UiEvent};

#[test]
fn drag_events_are_emitted_for_captured_pointer_motion() {
    let mut runtime = UiRuntime::default();
    let app = button("Drag").key("drag_source").draggable("payload");
    runtime.update(FrameInput {
        root: app,
        viewport: Size::new(200.0, 100.0),
        ..FrameInput::default()
    });

    runtime.dispatch(UiEvent::PointerDown(PointerEvent {
        position: Point::new(12.0, 12.0),
        button: Some(PointerButton::Primary),
        modifiers: 0,
    }));
    runtime.dispatch(UiEvent::PointerMove(PointerEvent {
        position: Point::new(80.0, 12.0),
        button: Some(PointerButton::Primary),
        modifiers: 0,
    }));
    runtime.dispatch(UiEvent::PointerUp(PointerEvent {
        position: Point::new(80.0, 12.0),
        button: Some(PointerButton::Primary),
        modifiers: 0,
    }));

    let commands = runtime.drain_commands();
    assert!(commands.iter().any(|cmd| cmd.kind() == "DragStart"));
    assert!(commands.iter().any(|cmd| cmd.kind() == "DragEnd"));
}
