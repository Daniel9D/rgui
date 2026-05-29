use rgui::runtime::{FrameInput, UiRuntime};
use rgui::widgets::{button, popover};
use rgui::{Element, KeyEvent, Size, UiEvent};

fn overlay_app() -> Element {
    Element::row()
        .key("root")
        .child(button("Before").key("before"))
        .child(
            button("Open").key("open").popover(
                popover().open(true).key("menu").child(
                    Element::column()
                        .child(button("One").key("one"))
                        .child(button("Two").key("two")),
                ),
            ),
        )
        .child(button("After").key("after"))
}

fn update(runtime: &mut UiRuntime, root: Element) {
    runtime.update(FrameInput {
        root,
        viewport: Size::new(420.0, 240.0),
        ..FrameInput::default()
    });
}

fn tab() -> UiEvent {
    UiEvent::KeyDown(KeyEvent {
        key: "Tab".to_string(),
        modifiers: 0,
        repeat: false,
    })
}

#[test]
fn tab_cycles_inside_open_popover() {
    let mut runtime = UiRuntime::default();
    update(&mut runtime, overlay_app());

    runtime.dispatch(tab());
    assert_eq!(runtime.focused_key(), Some("one".to_string()));

    runtime.dispatch(tab());
    assert_eq!(runtime.focused_key(), Some("two".to_string()));

    runtime.dispatch(tab());
    assert_eq!(runtime.focused_key(), Some("one".to_string()));
}

#[test]
fn focus_scope_returns_to_document_when_overlay_closes() {
    let mut runtime = UiRuntime::default();
    update(&mut runtime, overlay_app());
    runtime.dispatch(tab());
    assert_eq!(runtime.focused_key(), Some("one".to_string()));

    runtime.dispatch(UiEvent::KeyDown(KeyEvent {
        key: "Escape".to_string(),
        modifiers: 0,
        repeat: false,
    }));
    update(&mut runtime, overlay_app());
    runtime.dispatch(tab());
    assert_eq!(runtime.focused_key(), Some("before".to_string()));
}
