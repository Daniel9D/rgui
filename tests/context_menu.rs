use rgui::runtime::{FrameInput, UiRuntime};
use rgui::widgets::{button, context_menu, menu_item};
use rgui::{Point, PointerButton, PointerEvent, Size, UiEvent};

fn update(runtime: &mut UiRuntime, root: rgui::Element) -> rgui::runtime::FrameOutput {
    runtime.update(FrameInput {
        root,
        viewport: Size::new(320.0, 180.0),
        ..FrameInput::default()
    })
}

fn has_hit_key(output: &rgui::runtime::FrameOutput, key: &str) -> bool {
    output
        .hit_test
        .entries()
        .iter()
        .any(|entry| entry.key.as_deref() == Some(key))
}

#[test]
fn secondary_click_opens_context_menu_and_outside_click_dismisses() {
    let app = button("File").key("file").context_menu(
        context_menu()
            .key("file-menu")
            .child(menu_item("Delete").key("delete")),
    );

    let mut runtime = UiRuntime::default();
    update(&mut runtime, app.clone());
    runtime.dispatch(UiEvent::PointerDown(PointerEvent {
        position: Point::new(12.0, 12.0),
        button: Some(PointerButton::Secondary),
        modifiers: 0,
    }));
    let open = update(&mut runtime, app.clone());
    assert!(has_hit_key(&open, "delete"));

    runtime.dispatch(UiEvent::PointerDown(PointerEvent {
        position: Point::new(300.0, 160.0),
        button: Some(PointerButton::Primary),
        modifiers: 0,
    }));
    let closed = update(&mut runtime, app);
    assert!(!has_hit_key(&closed, "delete"));
}
