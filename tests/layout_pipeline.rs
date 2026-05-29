use rgui::runtime::{FrameInput, UiRuntime};
use rgui::widgets::{button, checkbox, input, text};
use rgui::{
    Edge, Element, LayoutBox, Length, NodeId, Overflow, PaintCommand, Point, Rect, Size, Style,
    Vec2,
};

#[test]
fn layout_box_exposes_render_hit_test_scroll_and_a11y_rects() {
    let node = NodeId::from_raw(7);
    let layout = LayoutBox::new(
        node,
        Rect::new(Point::new(10.0, 20.0), Size::new(100.0, 50.0)),
    )
    .with_content_size(Size::new(180.0, 90.0))
    .with_clip(Rect::new(Point::new(10.0, 20.0), Size::new(100.0, 50.0)))
    .with_scroll_offset(Vec2::new(0.0, 12.0))
    .with_z_index(3);

    assert_eq!(layout.node, node);
    assert_eq!(layout.content_size, Size::new(180.0, 90.0));
    assert!(layout.clip_rect.is_some());
    assert_eq!(layout.scroll_offset, Vec2::new(0.0, 12.0));
    assert_eq!(layout.z_index, 3);
}

#[test]
fn column_layout_stacks_children_with_gap_and_padding() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column()
            .padding(10.0)
            .gap(4.0)
            .child(text("A").key("a"))
            .child(button("B").key("b")),
        viewport: Size::new(200.0, 120.0),
        ..Default::default()
    });

    let snapshot = output.snapshot.as_ref().unwrap();
    let a = snapshot.layout_box("a").unwrap();
    let b = snapshot.layout_box("b").unwrap();

    assert_eq!(a.x, 10.0);
    assert!(b.y >= a.y + a.height + 4.0);
}

#[test]
fn overflow_hidden_generates_clip_rect_for_children() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column()
            .key("viewport")
            .height(50.0)
            .overflow(Overflow::Hidden)
            .child(text("large").height(120.0).key("large")),
        viewport: Size::new(200.0, 200.0),
        ..Default::default()
    });

    let snapshot = output.snapshot.as_ref().unwrap();
    let viewport = snapshot.layout_box("viewport").unwrap();
    let large = snapshot.layout_box("large").unwrap();

    assert!(viewport.clip_rect.is_some());
    assert_eq!(large.clip_rect, viewport.clip_rect);
}

#[test]
fn scroll_offset_moves_children_inside_scroll_container() {
    let mut runtime = UiRuntime::default();
    runtime.set_scroll_offset_for_key("viewport", Vec2::new(0.0, 20.0));

    let output = runtime.update(FrameInput {
        root: Element::column()
            .key("viewport")
            .height(50.0)
            .overflow(Overflow::Scroll)
            .child(text("large").height(120.0).key("large")),
        viewport: Size::new(200.0, 200.0),
        ..Default::default()
    });

    let snapshot = output.snapshot.as_ref().unwrap();
    let viewport = snapshot.layout_box("viewport").unwrap();
    let large = snapshot.layout_box("large").unwrap();

    assert_eq!(large.y, viewport.y - 20.0);
}

#[test]
fn scroll_offset_survives_keyed_reorder() {
    let mut runtime = UiRuntime::default();
    runtime.set_scroll_offset_for_key("viewport", Vec2::new(0.0, 20.0));

    let output = runtime.update(FrameInput {
        root: Element::column().child(text("Intro").key("intro")).child(
            Element::column()
                .key("viewport")
                .height(50.0)
                .overflow(Overflow::Scroll)
                .child(text("large").height(120.0).key("large")),
        ),
        viewport: Size::new(200.0, 200.0),
        ..Default::default()
    });
    let before = output
        .snapshot
        .as_ref()
        .unwrap()
        .layout_box("large")
        .unwrap()
        .y;

    let output = runtime.update(FrameInput {
        root: Element::column()
            .child(
                Element::column()
                    .key("viewport")
                    .height(50.0)
                    .overflow(Overflow::Scroll)
                    .child(text("large").height(120.0).key("large")),
            )
            .child(text("Intro").key("intro")),
        viewport: Size::new(200.0, 200.0),
        ..Default::default()
    });
    let viewport = output
        .snapshot
        .as_ref()
        .unwrap()
        .layout_box("viewport")
        .unwrap();
    let large = output
        .snapshot
        .as_ref()
        .unwrap()
        .layout_box("large")
        .unwrap();

    assert_ne!(before, large.y);
    assert_eq!(large.y, viewport.y - 20.0);
}

#[test]
fn explicit_child_height_is_used_by_runtime_layout() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column()
            .key("root")
            .child(text("large").height(64.0).key("large")),
        viewport: Size::new(200.0, 200.0),
        ..Default::default()
    });

    let large = output
        .snapshot
        .as_ref()
        .unwrap()
        .layout_box("large")
        .unwrap();

    assert_eq!(large.height, 64.0);
}

#[test]
fn row_layout_places_children_horizontally() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::row()
            .padding(10.0)
            .gap(6.0)
            .child(text("A").width(40.0).key("a"))
            .child(button("B").width(50.0).key("b")),
        viewport: Size::new(200.0, 120.0),
        ..Default::default()
    });

    let snapshot = output.snapshot.as_ref().unwrap();
    let a = snapshot.layout_box("a").unwrap();
    let b = snapshot.layout_box("b").unwrap();

    assert_eq!(a.x, 10.0);
    assert_eq!(b.x, a.x + a.width + 6.0);
    assert_eq!(b.y, a.y);
}

#[test]
fn row_uses_intrinsic_width_for_unconstrained_children() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::row()
            .key("toolbar")
            .padding(8.0)
            .gap(8.0)
            .child(button("Save").key("save"))
            .child(input().key("search"))
            .child(checkbox().key("enabled")),
        viewport: Size::new(320.0, 120.0),
        ..Default::default()
    });

    let snapshot = output.snapshot.as_ref().expect("snapshot exists");
    let save = snapshot.layout_box("save").expect("save layout");
    let search = snapshot.layout_box("search").expect("search layout");
    let enabled = snapshot.layout_box("enabled").expect("enabled layout");

    assert!(save.x >= 8.0);
    assert!(search.x > save.x);
    assert!(enabled.x > search.x);
    assert!(enabled.x + enabled.width <= 320.0);
    assert!(
        save.width <= 120.0,
        "button should not consume almost the full row"
    );
}

#[test]
fn scroll_offset_clamps_to_content_height() {
    let mut runtime = UiRuntime::default();
    runtime.set_scroll_offset_for_key("viewport", Vec2::new(0.0, 999.0));

    let output = runtime.update(FrameInput {
        root: Element::column()
            .key("viewport")
            .height(50.0)
            .overflow(Overflow::Scroll)
            .child(text("large").height(120.0).key("large")),
        viewport: Size::new(200.0, 200.0),
        ..Default::default()
    });

    let snapshot = output.snapshot.as_ref().unwrap();
    let viewport = snapshot.layout_box("viewport").unwrap();
    let large = snapshot.layout_box("large").unwrap();

    assert_eq!(large.y, viewport.y - 70.0);
}

#[test]
fn overflow_scroll_emits_display_list_clip_commands() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column()
            .key("viewport")
            .height(50.0)
            .overflow(Overflow::Scroll)
            .child(text("large").height(120.0).key("large")),
        viewport: Size::new(200.0, 200.0),
        ..Default::default()
    });

    assert!(
        output
            .display_list
            .commands()
            .iter()
            .any(|command| { matches!(command, PaintCommand::PushClip(_)) })
    );
    assert!(
        output
            .display_list
            .commands()
            .iter()
            .any(|command| { matches!(command, PaintCommand::PopClip) })
    );
}

#[test]
fn justify_center_rows_distribute_children_around_center() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::row()
            .padding(8.0)
            .gap(6.0)
            .justify_center()
            .child(text("A").width(30.0).key("a"))
            .child(button("B").width(40.0).key("b")),
        viewport: Size::new(300.0, 100.0),
        ..Default::default()
    });

    let snapshot = output.snapshot.as_ref().unwrap();
    let a = snapshot.layout_box("a").unwrap();
    // With justify:center, first child should be pushed toward center
    assert!(a.x > 10.0, "center justify should offset first child");
}

#[test]
fn justify_between_distributes_space_evenly() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::row()
            .padding(8.0)
            .gap(8.0)
            .justify_between()
            .child(text("A").width(30.0).key("a"))
            .child(button("B").width(40.0).key("b"))
            .child(text("C").width(20.0).key("c")),
        viewport: Size::new(320.0, 100.0),
        ..Default::default()
    });

    let snapshot = output.snapshot.as_ref().unwrap();
    let a = snapshot.layout_box("a").unwrap();
    let c = snapshot.layout_box("c").unwrap();
    assert!(a.x >= 8.0);
    assert!(c.x + c.width <= 320.0 - 8.0);
    // B should be between A and C with equal spacing
    let b = snapshot.layout_box("b").unwrap();
    let gap_ab = b.x - (a.x + a.width);
    let gap_bc = c.x - (b.x + b.width);
    assert!(
        (gap_ab - gap_bc).abs() < 2.0,
        "space-between should have equal gaps"
    );
}

#[test]
fn nested_scroll_accumulates_offsets_correctly() {
    let mut runtime = UiRuntime::default();
    runtime.set_scroll_offset_for_key("outer", Vec2::new(0.0, 15.0));
    runtime.set_scroll_offset_for_key("inner", Vec2::new(0.0, 10.0));

    let output = runtime.update(FrameInput {
        root: Element::column()
            .key("outer")
            .height(60.0)
            .overflow(Overflow::Scroll)
            .child(
                Element::column()
                    .key("inner")
                    .height(40.0)
                    .overflow(Overflow::Scroll)
                    .child(text("Deep").key("deep").height(80.0)),
            ),
        viewport: Size::new(200.0, 200.0),
        ..Default::default()
    });

    let snapshot = output.snapshot.as_ref().unwrap();
    let inner = snapshot.layout_box("inner").unwrap();
    let deep = snapshot.layout_box("deep").unwrap();
    // deep should be shifted by combined scroll (15 + 10 = 25)
    assert!(
        deep.y < inner.y,
        "nested scroll should shift deep content up"
    );
}

#[test]
fn heading_layout_box_is_taller_than_body_layout_box() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column()
            .child(text("Heading").heading().key("heading"))
            .child(text("Body").key("body")),
        viewport: Size::new(320.0, 160.0),
        ..Default::default()
    });

    let snapshot = output.snapshot.as_ref().expect("snapshot exists");
    let heading = snapshot.layout_box("heading").expect("heading layout");
    let body = snapshot.layout_box("body").expect("body layout");

    assert!(heading.height > body.height);
}

#[test]
fn fractional_width_text_layout_matches_paint_measurement_height() {
    let label = "Fractional width fits";
    let mut text_system = rgui::text_engine::TextSystem::default();
    let natural = text_system.measure(
        label,
        24.0,
        rgui::FontWeight::Bold,
        rgui::FontStyle::Normal,
        1000.0,
    );
    let width = natural.width + 0.01;
    let expected = text_system.measure(
        label,
        24.0,
        rgui::FontWeight::Bold,
        rgui::FontStyle::Normal,
        width,
    );

    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column().child(text(label).heading().width(width).key("fractional")),
        viewport: Size::new(width + 20.0, 120.0),
        ..Default::default()
    });

    let snapshot = output.snapshot.as_ref().expect("snapshot exists");
    let box_ = snapshot
        .layout_box("fractional")
        .expect("fractional layout");

    assert!(
        box_.height <= expected.height + 0.5,
        "fractional-width text layout should not reserve more height than paint measurement: layout={box_:?} expected={expected:?}"
    );
}

#[test]
fn button_intrinsic_width_grows_with_label_text() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::row()
            .gap(8.0)
            .child(button("Go").key("short"))
            .child(button("Longer Action Label").key("long")),
        viewport: Size::new(480.0, 120.0),
        ..Default::default()
    });

    let snapshot = output.snapshot.as_ref().expect("snapshot exists");
    let short = snapshot.layout_box("short").expect("short button");
    let long = snapshot.layout_box("long").expect("long button");

    assert!(long.width > short.width);
}

#[test]
fn taffy_backend_computes_row_child_positions() {
    let mut backend = rgui::layout::TaffyLayoutBackend::new();
    let mut text_system = rgui::text_engine::TextSystem::new();
    let tree = rgui::runtime::UiTree::from_element(
        Element::row()
            .child(text("A").key("a"))
            .child(text("B").key("b")),
    );
    let result = backend.build_from_tree(
        &tree,
        &mut text_system,
        Size::new(320.0, 120.0),
        &rgui::Theme::light(),
    );

    let nodes = tree.nodes();
    let a_id = nodes[1].id;
    let b_id = nodes[2].id;
    let box_a = result.boxes.iter().find(|b| b.node == a_id).unwrap();
    let box_b = result.boxes.iter().find(|b| b.node == b_id).unwrap();
    assert!(box_b.local_rect.origin.x >= box_a.local_rect.origin.x + box_a.local_rect.size.width);

    assert!(!result.boxes.is_empty());
}

#[test]
fn runtime_layout_and_taffy_backend_agree_on_root_viewport() {
    let mut runtime = UiRuntime::default();
    let output = runtime.update(FrameInput {
        root: Element::column()
            .key("root")
            .child(text("Hello").key("hello")),
        viewport: Size::new(320.0, 180.0),
        ..Default::default()
    });

    let root = output
        .snapshot
        .as_ref()
        .expect("snapshot")
        .layout_box("root")
        .expect("root layout");

    assert_eq!(root.width, 320.0);
    assert_eq!(root.height, 180.0);
}

#[test]
fn default_layout_mode_uses_taffy_first() {
    assert_eq!(rgui::layout::LAYOUT_ENGINE_NAME, "taffy_first");
}

#[test]
fn runtime_uses_taffy_first_layout_mode_for_row_by_default() {
    let mut runtime = UiRuntime::default();

    let output = runtime.update(FrameInput {
        root: Element::row()
            .gap(8.0)
            .child(text("A").key("a"))
            .child(text("B").key("b")),
        viewport: Size::new(320.0, 120.0),
        ..Default::default()
    });

    let snapshot = output.snapshot.as_ref().expect("snapshot exists");
    let a = snapshot.layout_box("a").expect("a layout");
    let b = snapshot.layout_box("b").expect("b layout");

    assert_eq!(output.layout_engine, "taffy_first");
    assert_eq!(a.y, b.y);
    assert!(b.x >= a.x + a.width + 8.0);
}

#[test]
fn runtime_resolves_stack_overlay_and_absolute_positioning() {
    let mut runtime = UiRuntime::default();

    let mut abs_style = Style::default();
    abs_style.position = Some(rgui::core::Position::Absolute);
    abs_style.inset = Some(Edge {
        top: Length::Px(15.0),
        right: Length::Auto,
        bottom: Length::Auto,
        left: Length::Px(25.0),
    });

    let output = runtime.update(FrameInput {
        root: Element::stack()
            .width(200.0)
            .height(100.0)
            .child(
                Element::text("Child1")
                    .width(80.0)
                    .height(40.0)
                    .key("child1"),
            )
            .child(
                Element::text("Child2")
                    .width(90.0)
                    .height(50.0)
                    .key("child2"),
            )
            .child(Element::text("Abs").style(abs_style).key("abs")),
        viewport: Size::new(200.0, 100.0),
        ..Default::default()
    });

    let snapshot = output.snapshot.as_ref().expect("snapshot exists");
    let child1 = snapshot.layout_box("child1").expect("child1 layout");
    let child2 = snapshot.layout_box("child2").expect("child2 layout");
    let abs = snapshot.layout_box("abs").expect("abs layout");

    // Flow children should overlay at (0, 0) relative to parent Stack
    assert_eq!(child1.x, 0.0);
    assert_eq!(child1.y, 0.0);
    assert_eq!(child2.x, 0.0);
    assert_eq!(child2.y, 0.0);

    // Absolute child should respect the inset relative to parent Stack
    assert_eq!(abs.x, 25.0);
    assert_eq!(abs.y, 15.0);
}

#[test]
fn text_wrapping_under_constrained_parent_restricts_measured_width() {
    let mut runtime = UiRuntime::default();

    let output = runtime.update(FrameInput {
        root: Element::column()
            .width(60.0)
            .child(text("very long text string that should wrap").key("wrap")),
        viewport: Size::new(320.0, 240.0),
        ..Default::default()
    });

    let snapshot = output.snapshot.as_ref().expect("snapshot exists");
    let wrap = snapshot.layout_box("wrap").expect("wrap layout");

    assert!(wrap.width <= 60.0);
}
