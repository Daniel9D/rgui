use rgui::runtime::{FrameInput, UiRuntime};
use rgui::widgets::{button, modal, text};
use rgui::{Element, KeyEvent, Point, PointerButton, PointerEvent, Size, UiEvent};

fn update(runtime: &mut UiRuntime, root: Element) -> rgui::runtime::FrameOutput {
    runtime.update(FrameInput {
        root,
        viewport: Size::new(420.0, 260.0),
        ..Default::default()
    })
}

fn escape(runtime: &mut UiRuntime) {
    runtime.dispatch(UiEvent::KeyDown(KeyEvent {
        key: "Escape".to_string(),
        modifiers: 0,
        repeat: false,
    }));
}

fn click(runtime: &mut UiRuntime, point: Point) {
    runtime.dispatch(UiEvent::PointerDown(PointerEvent {
        position: point,
        button: Some(PointerButton::Primary),
        modifiers: 0,
    }));
}

fn dialog(close_on_escape: bool, close_on_outside_click: bool) -> Element {
    let mut root = modal()
        .key("dialog")
        .open(true)
        .child(text("Confirm").key("dialog-title"))
        .child(button("OK").key("ok"));

    if let Some(rgui::WidgetSpec::Modal(spec)) = root.widget_spec.as_mut() {
        spec.close_on_escape = close_on_escape;
        spec.close_on_outside_click = close_on_outside_click;
    }

    root
}

#[test]
fn modal_close_on_escape_false_does_not_close_on_escape() {
    let mut runtime = UiRuntime::default();
    let root = dialog(false, true);
    update(&mut runtime, root.clone());

    escape(&mut runtime);
    let output = update(&mut runtime, root);

    assert!(
        output
            .snapshot
            .as_ref()
            .unwrap()
            .overlays()
            .iter()
            .any(|overlay| overlay.key.as_deref() == Some("dialog"))
    );
}

#[test]
fn modal_close_on_escape_true_closes_on_escape() {
    let mut runtime = UiRuntime::default();
    let root = dialog(true, true);
    update(&mut runtime, root.clone());

    escape(&mut runtime);
    let output = update(&mut runtime, root);

    assert!(output.snapshot.as_ref().unwrap().overlays().is_empty());
}

#[test]
fn modal_close_on_outside_click_false_blocks_but_does_not_close() {
    let mut runtime = UiRuntime::default();
    let root = Element::column()
        .child(button("Behind").key("behind"))
        .child(dialog(true, false));
    update(&mut runtime, root.clone());

    click(&mut runtime, Point::new(4.0, 4.0));
    let output = update(&mut runtime, root);

    assert!(
        output
            .snapshot
            .as_ref()
            .unwrap()
            .overlays()
            .iter()
            .any(|overlay| overlay.key.as_deref() == Some("dialog"))
    );
    assert_eq!(runtime.command_count(), 0);
}

#[test]
fn modal_close_on_outside_click_true_closes_on_backdrop_click() {
    let mut runtime = UiRuntime::default();
    let root = Element::column()
        .child(button("Behind").key("behind"))
        .child(dialog(true, true));
    update(&mut runtime, root.clone());

    click(&mut runtime, Point::new(4.0, 4.0));
    let output = update(&mut runtime, root);

    assert!(output.snapshot.as_ref().unwrap().overlays().is_empty());
}
