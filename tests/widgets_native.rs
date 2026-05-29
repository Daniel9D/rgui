use rgui::runtime::{FrameInput, UiRuntime};
use rgui::widgets::{button, checkbox, input, text};
use rgui::{
    Element, KeyEvent, PaintCommand, Point, PointerButton, PointerEvent, Role, Size, UiEvent,
};

#[test]
fn button_has_semantics_and_activates_on_click() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column().child(button("Save").key("save")),
        viewport: Size::new(200.0, 100.0),
        ..Default::default()
    });

    let semantic = output.semantics.by_key("save").unwrap();
    assert_eq!(semantic.role, Role::Button);

    runtime.dispatch(UiEvent::PointerDown(PointerEvent {
        position: Point::new(14.0, 14.0),
        button: Some(PointerButton::Primary),
        modifiers: 0,
    }));
    runtime.dispatch(UiEvent::PointerUp(PointerEvent {
        position: Point::new(14.0, 14.0),
        button: Some(PointerButton::Primary),
        modifiers: 0,
    }));

    assert_eq!(runtime.command_count(), 1);
}

#[test]
fn checkbox_toggles_uncontrolled_state_on_click() {
    let mut runtime = UiRuntime::default();
    runtime.update(FrameInput {
        root: Element::column().child(checkbox().key("enabled")),
        viewport: Size::new(200.0, 100.0),
        ..Default::default()
    });

    runtime.dispatch(UiEvent::PointerDown(PointerEvent {
        position: Point::new(14.0, 14.0),
        button: Some(PointerButton::Primary),
        modifiers: 0,
    }));
    runtime.dispatch(UiEvent::PointerUp(PointerEvent {
        position: Point::new(14.0, 14.0),
        button: Some(PointerButton::Primary),
        modifiers: 0,
    }));

    assert_eq!(runtime.bool_state("enabled"), Some(true));
}

#[test]
fn checkbox_state_survives_keyed_reorder() {
    let mut runtime = UiRuntime::default();
    let viewport = Size::new(240.0, 120.0);

    runtime.update(FrameInput {
        root: Element::column()
            .child(checkbox().key("enabled"))
            .child(button("Save").key("save")),
        viewport,
        ..Default::default()
    });

    runtime.dispatch(UiEvent::PointerDown(PointerEvent {
        position: Point::new(14.0, 14.0),
        button: Some(PointerButton::Primary),
        modifiers: 0,
    }));
    runtime.dispatch(UiEvent::PointerUp(PointerEvent {
        position: Point::new(14.0, 14.0),
        button: Some(PointerButton::Primary),
        modifiers: 0,
    }));
    assert_eq!(runtime.bool_state("enabled"), Some(true));

    runtime.update(FrameInput {
        root: Element::column()
            .child(button("Save").key("save"))
            .child(checkbox().key("enabled")),
        viewport,
        ..Default::default()
    });

    assert_eq!(runtime.bool_state("enabled"), Some(true));
}

#[test]
fn input_receives_text_when_focused() {
    let mut runtime = UiRuntime::default();
    runtime.update(FrameInput {
        root: Element::column().child(input().key("name")),
        viewport: Size::new(200.0, 100.0),
        ..Default::default()
    });

    runtime.dispatch(UiEvent::KeyDown(KeyEvent {
        key: "Tab".to_string(),
        modifiers: 0,
        repeat: false,
    }));
    runtime.dispatch(UiEvent::TextInput("A".to_string()));

    assert_eq!(runtime.text_state("name").as_deref(), Some("A"));
}

#[test]
fn input_placeholder_is_not_semantic_label() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: input().key("email").placeholder("Email address"),
        viewport: Size::new(320.0, 120.0),
        ..Default::default()
    });

    let node = output
        .semantics
        .nodes()
        .iter()
        .find(|node| node.key.as_deref() == Some("email"))
        .expect("input semantic node");

    assert_ne!(node.label.as_deref(), Some("Email address"));
    assert_eq!(node.value, None);
}

#[test]
fn input_default_value_is_semantic_value_not_label() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: input().key("name").default_value("Ada"),
        viewport: Size::new(320.0, 120.0),
        ..Default::default()
    });

    let node = output
        .semantics
        .nodes()
        .iter()
        .find(|node| node.key.as_deref() == Some("name"))
        .expect("input semantic node");

    assert_ne!(node.label.as_deref(), Some("Ada"));
    assert_eq!(
        node.value,
        Some(rgui::SemanticValue::Text("Ada".to_string()))
    );
}

#[test]
fn input_aria_label_is_intentional_semantic_label() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: input()
            .key("search")
            .placeholder("Search")
            .aria_label("Search field"),
        viewport: Size::new(320.0, 120.0),
        ..Default::default()
    });

    let node = output
        .semantics
        .nodes()
        .iter()
        .find(|node| node.key.as_deref() == Some("search"))
        .expect("input semantic node");

    assert_eq!(node.label.as_deref(), Some("Search field"));
}

#[test]
fn button_click_on_label_area_activates_button() {
    let mut runtime = UiRuntime::default();
    runtime.update(FrameInput {
        root: Element::column().child(button("Save").key("save")),
        viewport: Size::new(240.0, 120.0),
        ..Default::default()
    });

    runtime.dispatch(UiEvent::PointerDown(PointerEvent {
        position: Point::new(24.0, 24.0),
        button: Some(PointerButton::Primary),
        modifiers: 0,
    }));
    runtime.dispatch(UiEvent::PointerUp(PointerEvent {
        position: Point::new(24.0, 24.0),
        button: Some(PointerButton::Primary),
        modifiers: 0,
    }));

    assert_eq!(runtime.command_count(), 1);
}

#[test]
fn checkbox_checked_builder_initializes_visual_state() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column().child(checkbox().checked(true).key("enabled")),
        viewport: Size::new(200.0, 100.0),
        ..Default::default()
    });

    assert_eq!(runtime.bool_state("enabled"), Some(true));
    assert!(
        output.display_list.commands().len() >= 3,
        "checked checkbox should paint box plus inner mark"
    );
}

#[test]
fn primary_button_paints_differently_from_secondary_button() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column()
            .child(button("Primary").primary().key("primary"))
            .child(button("Secondary").key("secondary")),
        viewport: Size::new(260.0, 160.0),
        ..Default::default()
    });

    let rect_colors: Vec<_> = output
        .display_list
        .commands()
        .iter()
        .filter_map(|command| match command {
            PaintCommand::DrawRect(cmd) if cmd.z_index >= 0 => Some(format!("{:?}", cmd.paint)),
            _ => None,
        })
        .collect();

    assert!(rect_colors.len() >= 2);
    assert!(
        rect_colors.windows(2).any(|pair| pair[0] != pair[1]),
        "primary and secondary buttons should not paint identically"
    );
}

#[test]
fn checkbox_checked_mark_stays_inside_checkbox_bounds() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column().child(checkbox().checked(true).key("enabled")),
        viewport: Size::new(200.0, 100.0),
        ..Default::default()
    });

    let snapshot = output.snapshot.as_ref().expect("snapshot exists");
    let checkbox_layout = snapshot.layout_box("enabled").expect("checkbox layout");
    for command in output.display_list.commands() {
        if let PaintCommand::DrawRect(cmd) = command {
            if cmd.z_index > 0 {
                assert!(cmd.rect.origin.x >= checkbox_layout.x);
                assert!(cmd.rect.origin.y >= checkbox_layout.y);
                assert!(cmd.rect.max_x() <= checkbox_layout.x + checkbox_layout.width);
                assert!(cmd.rect.max_y() <= checkbox_layout.y + checkbox_layout.height);
            }
        }
    }
}

#[test]
fn heading_text_paints_larger_than_body_text() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column()
            .child(text("Heading").heading().key("heading"))
            .child(text("Body").key("body")),
        viewport: Size::new(320.0, 160.0),
        ..Default::default()
    });

    let sizes: Vec<f32> = output
        .display_list
        .commands()
        .iter()
        .filter_map(|command| match command {
            PaintCommand::DrawText(cmd) => Some(cmd.size),
            _ => None,
        })
        .collect();

    assert!(sizes.iter().any(|size| *size >= 24.0));
    assert!(sizes.iter().any(|size| (*size - 14.0).abs() < f32::EPSILON));
}

#[test]
fn checked_state_does_not_paint_checked_as_text_label() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column().child(checkbox().checked(true).key("enabled")),
        viewport: Size::new(200.0, 100.0),
        ..Default::default()
    });

    assert!(!output.display_list.commands().iter().any(|command| {
        matches!(command, PaintCommand::DrawText(cmd) if cmd.text.eq_ignore_ascii_case("checked"))
    }));
}

#[test]
fn input_copy_cut_paste_shortcuts() {
    let mut runtime = UiRuntime::default();
    runtime.update(FrameInput {
        root: Element::column().child(input().default_value("Hello").key("txt")),
        viewport: Size::new(200.0, 100.0),
        ..Default::default()
    });

    // 1. Focus the input using Tab key
    runtime.dispatch(UiEvent::KeyDown(KeyEvent {
        key: "Tab".to_string(),
        modifiers: 0,
        repeat: false,
    }));

    // Verify it is focused
    assert_eq!(runtime.text_state("txt").as_deref(), Some("Hello"));

    // 2. Select the whole text (we can simulate this by setting cursor/selection in InputState)
    // If we press Ctrl+C, let's make sure it doesn't crash:
    runtime.dispatch(UiEvent::KeyDown(KeyEvent {
        key: "c".to_string(),
        modifiers: 2, // Ctrl mask
        repeat: false,
    }));

    // If we press Ctrl+V, let's make sure it doesn't crash:
    runtime.dispatch(UiEvent::KeyDown(KeyEvent {
        key: "v".to_string(),
        modifiers: 2, // Ctrl mask
        repeat: false,
    }));
}

#[test]
fn input_ime_preedit_and_commit() {
    let mut runtime = UiRuntime::default();
    runtime.update(FrameInput {
        root: Element::column().child(input().key("txt")),
        viewport: Size::new(200.0, 100.0),
        ..Default::default()
    });

    // 1. Focus the input
    runtime.dispatch(UiEvent::KeyDown(KeyEvent {
        key: "Tab".to_string(),
        modifiers: 0,
        repeat: false,
    }));

    // 2. Dispatch an ImePreedit event
    runtime.dispatch(UiEvent::ImePreedit(rgui::core::ImePreedit {
        text: "nihon".to_string(),
        cursor_byte_range: Some((0, 5)),
    }));

    // Verify preedit is stored in the state but the value itself is still empty
    assert_eq!(runtime.text_state("txt").as_deref(), None);

    // Render a frame and verify that the preedit text is drawn
    let output = runtime.update(FrameInput {
        root: Element::column().child(input().key("txt")),
        viewport: Size::new(200.0, 100.0),
        ..Default::default()
    });

    let has_preedit_text = output.display_list.commands().iter().any(|command| {
        if let PaintCommand::DrawText(cmd) = command {
            cmd.text == "nihon"
        } else {
            false
        }
    });
    assert!(
        has_preedit_text,
        "Uncommitted preedit text must be painted inside the input field"
    );

    // 3. Commit the IME text
    runtime.dispatch(UiEvent::ImeCommit("日本".to_string()));

    // Verify it is committed and preedit is cleared
    assert_eq!(runtime.text_state("txt").as_deref(), Some("日本"));

    let output2 = runtime.update(FrameInput {
        root: Element::column().child(input().key("txt")),
        viewport: Size::new(200.0, 100.0),
        ..Default::default()
    });

    let has_committed_text = output2.display_list.commands().iter().any(|command| {
        if let PaintCommand::DrawText(cmd) = command {
            cmd.text == "日本"
        } else {
            false
        }
    });
    assert!(
        has_committed_text,
        "Committed text must be painted inside the input field"
    );
}
#[test]
fn default_theme_and_unstyled_widgets_use_one_resolved_style_path() {
    let themed = rgui::runtime::paint::debug_resolved_widget_style(rgui::WidgetKind::Button, true);
    let unthemed =
        rgui::runtime::paint::debug_resolved_widget_style(rgui::WidgetKind::Button, false);

    assert_eq!(themed.border_width, unthemed.border_width);
    assert_eq!(themed.border_radius, unthemed.border_radius);
}
