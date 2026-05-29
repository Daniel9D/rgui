use std::sync::{Arc, Mutex};

use rgui::runtime::{FrameInput, UiRuntime};
use rgui::widgets::button;
use rgui::{Point, PointerButton, PointerEvent, Size, UiEvent};

#[test]
fn runtime_on_invokes_action_handler_for_click_command() {
    let called = Arc::new(Mutex::new(Vec::new()));
    let captured = called.clone();
    let mut runtime = UiRuntime::default();
    runtime.on("save", move |key| {
        captured.lock().unwrap().push(key.to_string())
    });

    let app = button("Save").key("save_button").on_click("save");
    runtime.update(FrameInput {
        root: app,
        viewport: Size::new(160.0, 80.0),
        ..FrameInput::default()
    });
    runtime.dispatch(UiEvent::PointerDown(PointerEvent {
        position: Point::new(12.0, 12.0),
        button: Some(PointerButton::Primary),
        modifiers: 0,
    }));
    runtime.dispatch(UiEvent::PointerUp(PointerEvent {
        position: Point::new(12.0, 12.0),
        button: Some(PointerButton::Primary),
        modifiers: 0,
    }));
    runtime.flush_command_handlers();

    assert_eq!(&*called.lock().unwrap(), &["save_button".to_string()]);
}
