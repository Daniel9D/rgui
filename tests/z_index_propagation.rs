use rgui::runtime::{FrameInput, UiRuntime};
use rgui::widgets::button;
use rgui::{Element, Point, Size};

#[test]
fn child_inside_elevated_parent_paints_and_hits_above_sibling() {
    let app = Element::row()
        .key("root")
        .child(
            Element::row()
                .key("elevated")
                .z_index(10)
                .child(button("Child").key("child")),
        )
        .child(button("Sibling").key("sibling"));
    let mut runtime = UiRuntime::default();
    let frame = runtime.update(FrameInput {
        root: app,
        viewport: Size::new(320.0, 120.0),
        ..FrameInput::default()
    });
    let hit = frame.hit_test.hit(Point::new(18.0, 18.0)).expect("hit");
    assert_eq!(hit.key.as_deref(), Some("child"));
    assert!(
        frame
            .display_list
            .commands()
            .iter()
            .any(|cmd| cmd.z_index() >= 10)
    );
}
