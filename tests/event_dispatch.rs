use rgui::runtime::{FrameInput, UiCommand, UiRuntime};
use rgui::widgets::{button, input, text};
use rgui::{
    Color, Element, Overflow, PaintCommand, Point, PointerButton, PointerEvent, Size, UiEvent,
    Vec2, WheelDeltaMode, WheelEvent,
};

#[test]
fn hit_test_returns_topmost_visible_node() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::stack()
            .child(button("Back").key("back"))
            .child(button("Front").key("front").z_index(5)),
        viewport: Size::new(200.0, 100.0),
        ..Default::default()
    });

    let hit = output.hit_test.hit(Point::new(12.0, 12.0)).unwrap();
    assert_eq!(hit.key.as_deref(), Some("front"));
}

#[test]
fn pointer_press_routes_to_hit_target_and_sets_active() {
    let mut runtime = UiRuntime::default();
    runtime.update(FrameInput {
        root: Element::column().child(button("Save").key("save")),
        viewport: Size::new(200.0, 100.0),
        ..Default::default()
    });

    runtime.dispatch(UiEvent::PointerDown(PointerEvent {
        position: Point::new(14.0, 14.0),
        button: Some(PointerButton::Primary),
        modifiers: 0,
    }));

    assert_eq!(runtime.active_key().as_deref(), Some("save"));
}

#[test]
fn button_click_emits_click_command() {
    let mut runtime = UiRuntime::default();
    runtime.update(FrameInput {
        root: Element::column().child(button("Save").key("save")),
        viewport: Size::new(240.0, 100.0),
        ..Default::default()
    });

    runtime.dispatch(UiEvent::PointerDown(PointerEvent {
        position: Point::new(10.0, 10.0),
        button: Some(PointerButton::Primary),
        modifiers: 0,
    }));
    runtime.dispatch(UiEvent::PointerUp(PointerEvent {
        position: Point::new(10.0, 10.0),
        button: Some(PointerButton::Primary),
        modifiers: 0,
    }));

    let output = runtime.update(FrameInput {
        root: Element::column().child(button("Save").key("save")),
        viewport: Size::new(240.0, 100.0),
        ..Default::default()
    });

    assert!(output.commands.commands().iter().any(|cmd| {
        matches!(
            cmd,
            UiCommand::Click { key: Some(key), .. } if key == "save"
        )
    }));
}

#[test]
fn tab_moves_focus_to_next_focusable_node() {
    let mut runtime = UiRuntime::default();
    runtime.update(FrameInput {
        root: Element::column()
            .child(input().key("name"))
            .child(button("Save").key("save")),
        viewport: Size::new(240.0, 120.0),
        ..Default::default()
    });

    runtime.dispatch(UiEvent::KeyDown(rgui::KeyEvent {
        key: "Tab".to_string(),
        modifiers: 0,
        repeat: false,
    }));
    assert_eq!(runtime.focused_key().as_deref(), Some("name"));

    runtime.dispatch(UiEvent::KeyDown(rgui::KeyEvent {
        key: "Tab".to_string(),
        modifiers: 0,
        repeat: false,
    }));
    assert_eq!(runtime.focused_key().as_deref(), Some("save"));
}

#[test]
fn focus_survives_keyed_reorder() {
    let mut runtime = UiRuntime::default();
    let viewport = Size::new(240.0, 120.0);

    runtime.update(FrameInput {
        root: Element::column()
            .child(input().key("name"))
            .child(button("Save").key("save")),
        viewport,
        ..Default::default()
    });
    runtime.dispatch(UiEvent::KeyDown(rgui::KeyEvent {
        key: "Tab".to_string(),
        modifiers: 0,
        repeat: false,
    }));
    assert_eq!(runtime.focused_key().as_deref(), Some("name"));

    runtime.update(FrameInput {
        root: Element::column()
            .child(button("Save").key("save"))
            .child(input().key("name")),
        viewport,
        ..Default::default()
    });

    assert_eq!(runtime.focused_key().as_deref(), Some("name"));
}

#[test]
fn clipped_scrolled_child_does_not_receive_pointer_outside_viewport() {
    let mut runtime = UiRuntime::default();
    runtime.set_scroll_offset_for_key("viewport", Vec2::new(0.0, 80.0));
    runtime.update(FrameInput {
        root: Element::column()
            .key("viewport")
            .height(40.0)
            .overflow(Overflow::Scroll)
            .child(button("Hidden").height(120.0).key("hidden")),
        viewport: Size::new(200.0, 120.0),
        ..Default::default()
    });

    runtime.dispatch(UiEvent::PointerDown(PointerEvent {
        position: Point::new(20.0, 90.0),
        button: Some(PointerButton::Primary),
        modifiers: 0,
    }));
    runtime.dispatch(UiEvent::PointerUp(PointerEvent {
        position: Point::new(20.0, 90.0),
        button: Some(PointerButton::Primary),
        modifiers: 0,
    }));

    assert_eq!(runtime.command_count(), 0);
}

#[test]
fn pointer_move_updates_hovered_key() {
    let mut runtime = UiRuntime::default();
    runtime.update(FrameInput {
        root: Element::column().child(button("Save").key("save")),
        viewport: Size::new(200.0, 100.0),
        ..Default::default()
    });

    runtime.dispatch(UiEvent::PointerMove(PointerEvent {
        position: Point::new(20.0, 20.0),
        button: None,
        modifiers: 0,
    }));

    assert_eq!(runtime.hovered_key().as_deref(), Some("save"));
}

#[test]
fn pointer_move_updates_hover_visual_state() {
    let mut runtime = UiRuntime::default();
    let root = Element::column().child(button("Save").key("save"));
    runtime.update(FrameInput {
        root: root.clone(),
        viewport: Size::new(200.0, 100.0),
        ..Default::default()
    });
    runtime.dispatch(UiEvent::PointerMove(PointerEvent {
        position: Point::new(20.0, 20.0),
        button: None,
        modifiers: 0,
    }));

    let output = runtime.update(FrameInput {
        root,
        viewport: Size::new(200.0, 100.0),
        ..Default::default()
    });

    assert!(output.display_list.commands().iter().any(|command| {
        matches!(
            command,
            PaintCommand::DrawRect(cmd)
                if cmd.z_index == 0 && cmd.paint == rgui::Paint::Solid(Color::rgb(235, 238, 242))
        )
    }));
}

#[test]
fn wheel_scrolls_scrollable_container() {
    let mut runtime = UiRuntime::default();
    let viewport = Size::new(200.0, 200.0);
    let root = Element::column()
        .key("scroll")
        .height(60.0)
        .overflow(Overflow::Scroll)
        .child(text("Long content").height(200.0).key("content"));

    runtime.update(FrameInput {
        root: root.clone(),
        viewport,
        ..Default::default()
    });

    // Dispatch wheel event (positive delta = scroll down, content moves up)
    runtime.dispatch(UiEvent::Wheel(WheelEvent {
        delta: Vec2::new(0.0, 10.0),
        position: Point::new(100.0, 30.0),
        mode: WheelDeltaMode::Pixels,
    }));

    // Trigger update to apply scroll
    let output = runtime.update(FrameInput {
        root,
        viewport,
        ..Default::default()
    });

    let snapshot = output.snapshot.as_ref().unwrap();
    let content = snapshot.layout_box("content").unwrap();
    assert!(
        content.y < 0.0,
        "content y={}, expected < 0 after wheel scroll",
        content.y
    );
}

#[test]
fn wheel_scroll_uses_set_scroll_offset_for_key() {
    let mut runtime = UiRuntime::default();
    let viewport = Size::new(200.0, 200.0);

    // Verify set_scroll_offset_for_key + update produces scroll
    runtime.set_scroll_offset_for_key("scroll", Vec2::new(0.0, 30.0));
    let output = runtime.update(FrameInput {
        root: Element::column()
            .key("scroll")
            .height(60.0)
            .overflow(Overflow::Scroll)
            .child(text("Long content").height(200.0).key("content")),
        viewport,
        ..Default::default()
    });

    let content = output
        .snapshot
        .as_ref()
        .unwrap()
        .layout_box("content")
        .unwrap();
    assert!(content.y < 0.0, "set_scroll_offset_for_key should work");
}

#[test]
fn dispatch_handlers_preserve_click_focus_and_text_behavior() {
    let mut runtime = UiRuntime::default();
    let app = Element::column()
        .key("root")
        .child(input().key("name"))
        .child(button("Save").key("save").on_click("save"));
    runtime.update(FrameInput {
        root: app,
        viewport: Size::new(320.0, 180.0),
        ..FrameInput::default()
    });

    runtime.dispatch(UiEvent::PointerDown(PointerEvent {
        position: Point::new(12.0, 12.0),
        button: Some(PointerButton::Primary),
        modifiers: 0,
    }));
    runtime.dispatch(UiEvent::TextInput("A".to_string()));
    runtime.dispatch(UiEvent::PointerDown(PointerEvent {
        position: Point::new(12.0, 54.0),
        button: Some(PointerButton::Primary),
        modifiers: 0,
    }));
    runtime.dispatch(UiEvent::PointerUp(PointerEvent {
        position: Point::new(12.0, 54.0),
        button: Some(PointerButton::Primary),
        modifiers: 0,
    }));

    assert_eq!(runtime.text_state("name"), Some("A".to_string()));
    assert!(
        runtime
            .drain_commands()
            .iter()
            .any(|cmd| cmd.action() == Some("save"))
    );
}
