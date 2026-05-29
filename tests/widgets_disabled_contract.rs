use rgui::runtime::{FrameInput, UiRuntime};
use rgui::widgets::{button, checkbox, input, option, select, textarea};
use rgui::{Element, KeyEvent, Point, PointerButton, PointerEvent, Size, UiEvent, WidgetSpec};

fn update(runtime: &mut UiRuntime, root: Element) -> rgui::runtime::FrameOutput {
    runtime.update(FrameInput {
        root,
        viewport: Size::new(360.0, 180.0),
        ..Default::default()
    })
}

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

fn press(runtime: &mut UiRuntime, key: &str) {
    runtime.dispatch(UiEvent::KeyDown(KeyEvent {
        key: key.to_string(),
        modifiers: 0,
        repeat: false,
    }));
}

fn disabled(mut element: Element) -> Element {
    match element.widget_spec.as_mut().expect("widget spec") {
        WidgetSpec::Button(spec) => spec.disabled = true,
        WidgetSpec::Input(spec) => spec.disabled = true,
        WidgetSpec::Textarea(spec) => spec.disabled = true,
        WidgetSpec::Checkbox(spec) => spec.disabled = true,
        WidgetSpec::Radio(spec) => spec.disabled = true,
        WidgetSpec::Select(spec) => spec.disabled = true,
        other => panic!("unsupported disabled test widget: {other:?}"),
    }
    element
}

#[test]
fn disabled_button_does_not_emit_click_command() {
    let mut runtime = UiRuntime::default();
    let output = update(&mut runtime, disabled(button("Save").key("save")));
    let rect = output.hit_test.entries()[0].rect;

    click(
        &mut runtime,
        Point::new(rect.origin.x + 8.0, rect.origin.y + 8.0),
    );

    assert_eq!(runtime.command_count(), 0);
}

#[test]
fn disabled_checkbox_does_not_toggle() {
    let mut runtime = UiRuntime::default();
    let output = update(&mut runtime, disabled(checkbox().key("enabled")));
    let rect = output.hit_test.entries()[0].rect;

    click(
        &mut runtime,
        Point::new(rect.origin.x + 8.0, rect.origin.y + 8.0),
    );

    assert_eq!(runtime.bool_state("enabled"), Some(false));
}

#[test]
fn disabled_input_and_textarea_do_not_focus_or_edit() {
    let mut runtime = UiRuntime::default();
    update(
        &mut runtime,
        Element::column()
            .child(disabled(input().key("name")))
            .child(disabled(textarea().key("notes"))),
    );

    press(&mut runtime, "Tab");
    runtime.dispatch(UiEvent::TextInput("x".to_string()));

    assert_eq!(runtime.focused_key(), None);
    assert_eq!(runtime.text_state("name"), None);
    assert_eq!(runtime.text_state("notes"), None);
}

#[test]
fn disabled_select_does_not_open() {
    let mut runtime = UiRuntime::default();
    let root = disabled(
        select()
            .key("priority")
            .options([option("low", "Low"), option("high", "High")]),
    );
    let output = update(&mut runtime, root.clone());
    let rect = output.hit_test.entries()[0].rect;

    click(
        &mut runtime,
        Point::new(rect.origin.x + 8.0, rect.origin.y + 8.0),
    );
    let output = update(&mut runtime, root);

    assert!(output.hit_test.entries().iter().all(|entry| {
        !entry
            .key
            .as_deref()
            .is_some_and(|key| key.starts_with("priority::__option::"))
    }));
}

#[test]
fn disabled_select_option_cannot_be_selected() {
    let mut runtime = UiRuntime::default();
    let root = select()
        .key("priority")
        .options([option("low", "Low"), option("high", "High").disabled(true)]);
    let output = update(&mut runtime, root.clone());
    let rect = output.hit_test.entries()[0].rect;

    click(
        &mut runtime,
        Point::new(rect.origin.x + 8.0, rect.origin.y + 8.0),
    );
    let open = update(&mut runtime, root.clone());
    let high = open
        .hit_test
        .entries()
        .iter()
        .find(|entry| entry.key.as_deref() == Some("priority::__option::1"))
        .expect("high option hit target")
        .rect;
    click(
        &mut runtime,
        Point::new(high.origin.x + 8.0, high.origin.y + 8.0),
    );

    assert_eq!(runtime.selected_value("priority").as_deref(), Some("low"));
}
