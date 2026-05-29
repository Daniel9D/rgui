use rgui::runtime::{FrameInput, UiRuntime};
use rgui::widgets::input;
use rgui::{Point, PointerButton, PointerEvent, Size, UiEvent};

#[test]
fn input_caret_hit_testing_uses_theme_metrics_text_rect() {
    let mut runtime = UiRuntime::default();
    let mut theme = rgui::Theme::light();
    theme.widgets.metrics.input.horizontal_padding = 20.0;

    let output = runtime.update(FrameInput {
        root: input().key("input").default_value("abcdef"),
        viewport: Size::new(320.0, 100.0),
        theme,
        ..Default::default()
    });

    let hit = output
        .hit_test
        .entries()
        .iter()
        .find(|entry| entry.key.as_deref() == Some("input"))
        .expect("input hit entry");

    runtime.dispatch(UiEvent::PointerDown(PointerEvent {
        position: Point::new(
            hit.rect.origin.x + 21.0,
            hit.rect.origin.y + hit.rect.size.height / 2.0,
        ),
        button: Some(PointerButton::Primary),
        modifiers: 0,
    }));

    assert_eq!(runtime.focused_key().as_deref(), Some("input"));
    assert_eq!(runtime.text_cursor("input"), Some(0));
}
