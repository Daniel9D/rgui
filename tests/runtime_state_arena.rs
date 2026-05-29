use rgui::runtime::{FrameInput, UiRuntime};
use rgui::{Element, Point, PointerButton, PointerEvent, Size, UiEvent};

fn input_app() -> Element {
    Element::row()
        .key("root")
        .child(rgui::widgets::text("Name").key("label"))
        .child(rgui::widgets::input().key("name").default_value("abcd"))
}

#[test]
fn text_input_replaces_selection_in_state_arena_only() {
    let mut runtime = UiRuntime::default();
    runtime.update(FrameInput {
        root: input_app(),
        viewport: Size::new(320.0, 120.0),
        ..FrameInput::default()
    });

    runtime.dispatch(UiEvent::PointerDown(PointerEvent {
        position: Point::new(70.0, 18.0),
        button: Some(PointerButton::Primary),
        modifiers: 0,
    }));
    runtime.set_text_selection_for_key("name", 1..3);
    runtime.dispatch(UiEvent::TextInput("Z".to_string()));

    assert_eq!(runtime.text_state("name"), Some("aZd".to_string()));
    assert_eq!(runtime.debug_legacy_text_state_count(), 0);
}

#[test]
fn text_state_is_none_when_input_state_is_not_seeded() {
    let runtime = UiRuntime::default();

    assert_eq!(runtime.text_state("missing"), None);
    assert_eq!(runtime.debug_legacy_text_state_count(), 0);
}
