use rgui::runtime::{FrameInput, UiRuntime};
use rgui::widgets::{button, input, modal, popover, text};
use rgui::{Element, KeyEvent, LayerKind, PaintCommand, Point, Size, UiEvent};

fn click(runtime: &mut UiRuntime, point: Point) {
    runtime.dispatch(UiEvent::PointerDown(rgui::PointerEvent {
        position: point,
        button: Some(rgui::PointerButton::Primary),
        modifiers: 0,
    }));
    runtime.dispatch(UiEvent::PointerUp(rgui::PointerEvent {
        position: point,
        button: Some(rgui::PointerButton::Primary),
        modifiers: 0,
    }));
}

#[test]
fn popover_attached_to_button_opens_only_after_trigger_click() {
    let mut runtime = UiRuntime::default();
    let root = Element::column().child(
        button("Menu")
            .key("menu")
            .popover(popover().key("menu-popover").child(text("Item"))),
    );

    let closed = runtime.update(FrameInput {
        root: root.clone(),
        viewport: Size::new(300.0, 200.0),
        ..Default::default()
    });
    assert!(closed.snapshot.as_ref().unwrap().overlays().is_empty());

    let trigger = closed
        .hit_test
        .entries()
        .iter()
        .find(|entry| entry.key.as_deref() == Some("menu"))
        .expect("trigger hit target")
        .rect;
    click(
        &mut runtime,
        Point::new(trigger.origin.x + 8.0, trigger.origin.y + 8.0),
    );

    let opened = runtime.update(FrameInput {
        root,
        viewport: Size::new(300.0, 200.0),
        ..Default::default()
    });
    assert_eq!(opened.snapshot.as_ref().unwrap().overlays().len(), 1);
}

#[test]
fn popover_does_not_change_document_layout_size() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column()
            .child(
                button("Menu")
                    .key("menu")
                    .popover(popover().open(true).child(text("Item"))),
            )
            .child(text("Below").key("below")),
        viewport: Size::new(300.0, 200.0),
        ..Default::default()
    });

    let below = output
        .snapshot
        .as_ref()
        .unwrap()
        .layout_box("below")
        .unwrap();
    let overlays = output.snapshot.as_ref().unwrap().overlays();

    assert_eq!(overlays.len(), 1);
    assert!(below.y < 80.0);
}

#[test]
fn popover_panel_grows_with_child_text() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column().child(
            button("Open").key("open").popover(
                popover()
                    .open(true)
                    .key("menu")
                    .child(text("A long popover action label")),
            ),
        ),
        viewport: Size::new(420.0, 240.0),
        ..Default::default()
    });

    let panel = output
        .hit_test
        .entries()
        .iter()
        .find(|entry| entry.key.as_deref() == Some("menu"))
        .expect("popover panel hit-test exists");
    assert!(panel.rect.size.width >= 180.0);
}

#[test]
fn popover_child_button_uses_normal_button_painter() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column().child(
            button("Open").key("open").popover(
                popover()
                    .open(true)
                    .key("menu")
                    .child(button("Apply").key("apply")),
            ),
        ),
        viewport: Size::new(420.0, 240.0),
        ..Default::default()
    });

    assert!(
        output
            .semantics
            .nodes()
            .iter()
            .any(|node| node.key.as_deref() == Some("apply") && node.focusable),
        "portal button should have a semantic focusable node"
    );
}

#[test]
fn popover_child_button_keeps_distinct_node_identity() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column().child(
            button("Open").key("open").popover(
                popover()
                    .open(true)
                    .key("menu")
                    .child(button("Apply").key("apply")),
            ),
        ),
        viewport: Size::new(420.0, 240.0),
        ..Default::default()
    });

    let owner = runtime
        .node_for_key("open")
        .expect("popover owner has a reconciled node");
    let hit = output
        .hit_test
        .entries()
        .iter()
        .find(|entry| entry.key.as_deref() == Some("apply"))
        .expect("portal child has hit-test entry");
    let semantic = output
        .semantics
        .nodes()
        .iter()
        .find(|node| node.key.as_deref() == Some("apply"))
        .expect("portal child has semantic node");

    assert_ne!(hit.node, owner);
    assert_eq!(semantic.node, hit.node);
}

#[test]
fn popover_portal_input_can_focus_and_preserve_text_state() {
    let mut runtime = UiRuntime::default();
    let root = Element::column().child(
        button("Open").key("open").popover(
            popover()
                .open(true)
                .key("menu")
                .child(input().key("portal-input")),
        ),
    );

    let output = runtime.update(FrameInput {
        root: root.clone(),
        viewport: Size::new(420.0, 240.0),
        ..Default::default()
    });
    let input_hit = output
        .hit_test
        .entries()
        .iter()
        .find(|entry| entry.key.as_deref() == Some("portal-input"))
        .expect("portal input has hit-test entry")
        .clone();

    runtime.dispatch(UiEvent::PointerDown(rgui::PointerEvent {
        position: Point::new(input_hit.rect.origin.x + 4.0, input_hit.rect.origin.y + 4.0),
        button: Some(rgui::PointerButton::Primary),
        modifiers: 0,
    }));
    runtime.dispatch(UiEvent::TextInput("x".into()));
    runtime.update(FrameInput {
        root,
        viewport: Size::new(420.0, 240.0),
        ..Default::default()
    });

    assert_eq!(runtime.text_state("portal-input").as_deref(), Some("x"));
}

#[test]
fn modal_blocks_document_hit_test_outside_modal_bounds() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column()
            .child(button("Delete").key("delete"))
            .child(modal().key("confirm").open(true).child(button("Confirm"))),
        viewport: Size::new(320.0, 240.0),
        ..Default::default()
    });

    let hit = output.hit_test.hit(Point::new(14.0, 14.0)).unwrap();
    assert_ne!(hit.key.as_deref(), Some("delete"));
}

#[test]
fn modal_backdrop_blocks_document_button_hit_target() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column()
            .child(button("Document").key("document"))
            .child(
                modal()
                    .key("dialog")
                    .open(true)
                    .child(button("Ok").key("ok")),
            ),
        viewport: Size::new(420.0, 260.0),
        ..Default::default()
    });

    let top_entry = output
        .hit_test
        .hit(Point::new(12.0, 12.0))
        .expect("modal backdrop catches pointer");
    assert_ne!(top_entry.key.as_deref(), Some("document"));
}

#[test]
fn escape_dismisses_open_popover_before_next_frame() {
    let mut runtime = UiRuntime::default();
    let root = Element::column().child(
        button("Menu")
            .key("menu")
            .popover(popover().open(true).key("menu-popover").child(text("Item"))),
    );

    let output = runtime.update(FrameInput {
        root: root.clone(),
        viewport: Size::new(300.0, 200.0),
        ..Default::default()
    });
    assert_eq!(output.snapshot.as_ref().unwrap().overlays().len(), 1);

    runtime.dispatch(UiEvent::KeyDown(KeyEvent {
        key: "Escape".to_string(),
        modifiers: 0,
        repeat: false,
    }));

    let output = runtime.update(FrameInput {
        root,
        viewport: Size::new(300.0, 200.0),
        ..Default::default()
    });
    assert!(output.snapshot.as_ref().unwrap().overlays().is_empty());
}

#[test]
fn escape_dismisses_only_topmost_overlay() {
    let mut runtime = UiRuntime::default();
    let root = Element::column()
        .child(
            button("First")
                .key("first")
                .popover(popover().open(true).key("pop-first").child(text("A"))),
        )
        .child(
            button("Second")
                .key("second")
                .popover(popover().open(true).key("pop-second").child(text("B"))),
        );

    let output = runtime.update(FrameInput {
        root: root.clone(),
        viewport: Size::new(300.0, 200.0),
        ..Default::default()
    });
    assert_eq!(output.snapshot.as_ref().unwrap().overlays().len(), 2);

    runtime.dispatch(UiEvent::KeyDown(KeyEvent {
        key: "Escape".to_string(),
        modifiers: 0,
        repeat: false,
    }));

    let output = runtime.update(FrameInput {
        root,
        viewport: Size::new(300.0, 200.0),
        ..Default::default()
    });
    let overlays = output.snapshot.as_ref().unwrap().overlays();
    assert_eq!(overlays.len(), 1);
    assert_eq!(overlays[0].key.as_deref(), Some("pop-first"));
}

#[test]
fn outside_pointer_dismisses_open_popover_before_next_frame() {
    let mut runtime = UiRuntime::default();
    let root = Element::column().child(
        button("Menu")
            .key("menu")
            .popover(popover().open(true).key("menu-popover").child(text("Item"))),
    );

    runtime.update(FrameInput {
        root: root.clone(),
        viewport: Size::new(300.0, 200.0),
        ..Default::default()
    });

    runtime.dispatch(UiEvent::PointerDown(rgui::PointerEvent {
        position: Point::new(250.0, 180.0),
        button: Some(rgui::PointerButton::Primary),
        modifiers: 0,
    }));

    let output = runtime.update(FrameInput {
        root,
        viewport: Size::new(300.0, 200.0),
        ..Default::default()
    });
    assert!(output.snapshot.as_ref().unwrap().overlays().is_empty());
}

#[test]
fn open_popover_has_resolved_paint_and_hit_test_data() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column().child(
            button("Menu")
                .key("menu")
                .popover(popover().open(true).key("menu-popover").child(text("Item"))),
        ),
        viewport: Size::new(300.0, 200.0),
        ..Default::default()
    });

    let snapshot = output.snapshot.as_ref().unwrap();
    let overlay = snapshot.overlays().first().expect("resolved overlay");

    assert_eq!(overlay.key.as_deref(), Some("menu-popover"));
    assert!(overlay.rect.size.width >= 120.0);
    assert!(!overlay.modal);
}

#[test]
fn open_popover_paints_background_and_child_text_in_popover_layer() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column().child(
            button("Menu").key("menu").popover(
                popover()
                    .open(true)
                    .key("menu-popover")
                    .child(text("Profile").key("profile"))
                    .child(text("Settings").key("settings")),
            ),
        ),
        viewport: Size::new(320.0, 240.0),
        ..Default::default()
    });

    let commands = output.display_list.commands();
    assert!(commands.iter().any(|command| matches!(
        command,
        PaintCommand::PushLayer(spec) if spec.kind == LayerKind::Popover
    )));
    assert!(commands.iter().any(|command| matches!(
        command,
        PaintCommand::DrawText(cmd) if cmd.text == "Profile"
    )));
    assert!(
        commands
            .iter()
            .any(|command| matches!(command, PaintCommand::PopLayer))
    );
}

#[test]
fn modal_closed_does_not_paint_children_in_document_flow() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column().child(
            modal()
                .key("confirm")
                .open(false)
                .child(text("Confirm action?").heading())
                .child(button("OK").primary().key("modal-ok")),
        ),
        viewport: Size::new(320.0, 240.0),
        ..Default::default()
    });

    let commands = output.display_list.commands();
    // Modal children should NOT appear in document flow when closed
    let has_modal_children_in_document = commands
        .iter()
        .filter(|cmd| !matches!(cmd, PaintCommand::PushLayer(_) | PaintCommand::PopLayer))
        .any(|cmd| {
            matches!(
                cmd,
                PaintCommand::DrawText(cmd) if cmd.text.contains("Confirm")
                    || cmd.text.contains("OK")
            )
        });
    assert!(
        !has_modal_children_in_document,
        "closed modal children should not be in document flow"
    );
}

#[test]
fn modal_open_paints_children_in_modal_layer() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column().child(
            modal()
                .key("confirm")
                .open(true)
                .child(text("Confirm action?").heading())
                .child(button("OK").primary().key("modal-ok")),
        ),
        viewport: Size::new(320.0, 240.0),
        ..Default::default()
    });

    let commands = output.display_list.commands();
    // Modal layer should be pushed
    let has_modal_layer = commands
        .iter()
        .any(|cmd| matches!(cmd, PaintCommand::PushLayer(spec) if spec.kind == LayerKind::Modal));
    assert!(has_modal_layer, "open modal should push Modal layer");

    // Modal children should appear AFTER the PushLayer(Modal)
    let mut in_modal = false;
    let mut found_child = false;
    for cmd in commands {
        match cmd {
            PaintCommand::PushLayer(spec) if spec.kind == LayerKind::Modal => in_modal = true,
            PaintCommand::DrawText(cmd) if in_modal && cmd.text.contains("Confirm") => {
                found_child = true;
            }
            PaintCommand::PopLayer if in_modal => in_modal = false,
            _ => {}
        }
    }
    assert!(
        found_child,
        "open modal children should paint inside modal layer"
    );
    assert!(
        commands
            .iter()
            .any(|cmd| matches!(cmd, PaintCommand::DrawText(cmd) if cmd.text == "OK")),
        "declared modal button should preserve its child label"
    );
}

#[test]
fn popover_panel_has_hit_test_entry() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column().child(
            button("Menu").key("menu").popover(
                popover()
                    .open(true)
                    .key("menu-popover")
                    .child(text("Profile")),
            ),
        ),
        viewport: Size::new(320.0, 240.0),
        ..Default::default()
    });

    // The hit-test tree should contain an entry for the popover layer
    let has_popover_hit = output
        .hit_test
        .entries()
        .iter()
        .any(|entry| entry.layer == LayerKind::Popover);
    assert!(has_popover_hit, "popover panel should have hit-test entry");
}

#[test]
fn checkbox_checked_initial_value_preserved_after_toggle() {
    let mut runtime = UiRuntime::default();
    let viewport = Size::new(240.0, 120.0);

    // Initial: checkbox starts checked
    runtime.update(FrameInput {
        root: Element::column().child(rgui::widgets::checkbox().key("enabled").checked(true)),
        viewport,
        ..Default::default()
    });
    assert_eq!(runtime.bool_state("enabled"), Some(true));

    // User toggles off
    runtime.dispatch(UiEvent::PointerDown(rgui::PointerEvent {
        position: Point::new(14.0, 14.0),
        button: Some(rgui::PointerButton::Primary),
        modifiers: 0,
    }));
    runtime.dispatch(UiEvent::PointerUp(rgui::PointerEvent {
        position: Point::new(14.0, 14.0),
        button: Some(rgui::PointerButton::Primary),
        modifiers: 0,
    }));
    assert_eq!(runtime.bool_state("enabled"), Some(false));

    // Next frame: .checked(true) should NOT reset (because it's unchecked now)
    runtime.update(FrameInput {
        root: Element::column().child(rgui::widgets::checkbox().key("enabled").checked(true)),
        viewport,
        ..Default::default()
    });
    // Should still be false — user state preserved
    assert_eq!(
        runtime.bool_state("enabled"),
        Some(false),
        ".checked(true) should act as initial value, not override user toggle"
    );
}

#[test]
fn z_index_paint_and_hit_test_use_same_default() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column()
            .child(text("A").key("a"))
            .child(text("B").key("b").z_index(5)),
        viewport: Size::new(200.0, 120.0),
        ..Default::default()
    });

    // Both should have z_index from style: a=0 (default), b=5 (explicit)
    let snapshot = output.snapshot.as_ref().unwrap();
    let a_hit = output
        .hit_test
        .entries()
        .iter()
        .find(|e| e.key.as_deref() == Some("a"))
        .unwrap();
    let b_hit = output
        .hit_test
        .entries()
        .iter()
        .find(|e| e.key.as_deref() == Some("b"))
        .unwrap();
    let a_layout = snapshot.layout_box("a").unwrap();
    let b_layout = snapshot.layout_box("b").unwrap();

    assert_eq!(a_hit.z_index, 0, "default z_index should be 0");
    assert_eq!(b_hit.z_index, 5, "explicit z_index should be 5");
    // Hit entries should be at consistent positions
    assert!(a_layout.y < b_layout.y, "element 'a' should be above 'b'");
}

#[test]
fn popover_rect_grows_to_fit_multiple_children() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column().child(
            button("Menu").key("menu").popover(
                popover()
                    .open(true)
                    .key("menu-popover")
                    .child(text("Profile"))
                    .child(text("Settings"))
                    .child(button("Sign out").key("signout")),
            ),
        ),
        viewport: Size::new(360.0, 240.0),
        ..Default::default()
    });

    let overlay = output
        .snapshot
        .as_ref()
        .expect("snapshot")
        .overlays()
        .first()
        .expect("overlay");

    assert!(overlay.rect.size.height >= 96.0);
    assert!(overlay.rect.size.width >= 160.0);
}

#[test]
fn popover_panel_paints_border_or_shadow_chrome() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column().child(
            button("Menu").key("menu").popover(
                popover()
                    .open(true)
                    .key("menu-popover")
                    .child(text("Profile")),
            ),
        ),
        viewport: Size::new(320.0, 200.0),
        ..Default::default()
    });

    let chrome_count = output
        .display_list
        .commands()
        .iter()
        .filter(|command| {
            matches!(
                command,
                PaintCommand::DrawBorder(_) | PaintCommand::DrawShadow(_)
            )
        })
        .count();

    assert!(chrome_count >= 1);
}

#[test]
fn portal_children_preserve_heading_and_button_visual_style() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column().child(
            button("Menu").key("menu").popover(
                popover()
                    .open(true)
                    .key("menu-popover")
                    .child(text("Actions").heading().key("actions"))
                    .child(button("Apply").primary().key("apply")),
            ),
        ),
        viewport: Size::new(360.0, 240.0),
        ..Default::default()
    });

    assert!(output.display_list.commands().iter().any(|command| {
        matches!(command, PaintCommand::DrawText(cmd) if cmd.text == "Actions" && cmd.size >= 24.0)
    }));
    assert!(
        output.display_list.commands().iter().any(|command| {
            matches!(command, PaintCommand::DrawText(cmd) if cmd.text == "Apply")
        })
    );
}

#[test]
fn test_constrain_overlay_to_viewport() {
    use rgui::runtime::overlay_pass::constrain_overlay_to_viewport;
    use rgui::{Point, Rect, Size};

    // Sticking outside viewport on right/bottom
    let rect = Rect::new(Point::new(350.0, 250.0), Size::new(100.0, 50.0));
    let viewport = Size::new(300.0, 200.0);
    let constrained = constrain_overlay_to_viewport(rect, viewport);

    assert_eq!(constrained.origin.x, 200.0);
    assert_eq!(constrained.origin.y, 150.0);
    assert_eq!(constrained.size.width, 100.0);
    assert_eq!(constrained.size.height, 50.0);
}

#[test]
fn modal_panel_size_comes_from_taffy_content_layout() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: rgui::Element::column()
            .child(button("Document").key("doc"))
            .child(
                modal().key("confirm").open(true).child(
                    rgui::Element::column()
                        .key("modal-content")
                        .gap(12.0)
                        .padding(20.0)
                        .child(text("Confirm a long action").key("modal-title"))
                        .child(button("Confirm").key("modal-confirm")),
                ),
            ),
        viewport: Size::new(420.0, 280.0),
        ..Default::default()
    });

    let overlay = output
        .snapshot
        .as_ref()
        .and_then(|snapshot| {
            snapshot
                .overlays()
                .iter()
                .find(|overlay| overlay.key.as_deref() == Some("confirm"))
        })
        .expect("modal overlay snapshot");

    let content = output
        .snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.layout_box("modal-content"))
        .expect("modal content layout");
    let title = output
        .snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.layout_box("modal-title"))
        .expect("modal title layout");
    let button = output
        .snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.layout_box("modal-confirm"))
        .expect("modal button layout");

    assert!(overlay.rect.size.width >= content.width);
    assert!(overlay.rect.size.height >= content.height);
    assert!(button.y >= title.y + title.height + 12.0);
    assert!(
        overlay.rect.origin.x > 0.0,
        "modal panel should be centered horizontally"
    );
    assert!(
        overlay.rect.origin.y > 0.0,
        "modal panel should be centered vertically"
    );
}
