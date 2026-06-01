use rgui::runtime::{FrameInput, UiRuntime};
use rgui::widgets::text;
use rgui::{Element, Position, Size, Style};

#[test]
fn fixed_position_warning_is_emitted_only_once_per_frame() {
    // Many fixed-position nodes should produce only one warning, not N.
    let mut root = Element::column();
    for _ in 0..5 {
        let fixed_node = Element::column()
            .style(Style {
                position: Some(Position::Fixed),
                width: Some(rgui::Length::Px(50.0)),
                height: Some(rgui::Length::Px(50.0)),
                ..Default::default()
            })
            .child(text("fixed"))
            .key("fixed");
        root = root.child(fixed_node);
    }

    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root,
        viewport: Size::new(800.0, 600.0),
        ..Default::default()
    });

    let snapshot = output
        .snapshot
        .as_ref()
        .expect("runtime should produce a snapshot");
    let fixed_warnings: Vec<&String> = snapshot
        .diagnostics
        .layout_warnings
        .iter()
        .filter(|w| w.contains("position=fixed currently behaves like absolute"))
        .collect();

    assert_eq!(
        fixed_warnings.len(),
        1,
        "expected exactly one fixed-position warning, got {}: {:?}",
        fixed_warnings.len(),
        snapshot.diagnostics.layout_warnings
    );
}

#[test]
fn no_fixed_position_warning_when_no_fixed_nodes_present() {
    let root = Element::column().child(text("normal"));
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root,
        viewport: Size::new(800.0, 600.0),
        ..Default::default()
    });

    let snapshot = output
        .snapshot
        .as_ref()
        .expect("runtime should produce a snapshot");
    let fixed_warnings: usize = snapshot
        .diagnostics
        .layout_warnings
        .iter()
        .filter(|w| w.contains("position=fixed"))
        .count();

    assert_eq!(
        fixed_warnings, 0,
        "no fixed-position warning should be emitted when no fixed nodes are present"
    );
}

