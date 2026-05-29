use rgui::core::{LayoutBox, LayoutResult, NodeId, Point, Rect, Size, Vec2};

fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect::new(Point::new(x, y), Size::new(width, height))
}

#[test]
fn layout_result_finds_boxes_by_node_and_key() {
    let root = NodeId::from_raw(1);
    let child = NodeId::from_raw(2);
    let mut result = LayoutResult::default();
    result.push(LayoutBox::new(root, rect(0.0, 0.0, 300.0, 200.0)).with_key("root"));
    result.push(LayoutBox::new(child, rect(8.0, 12.0, 120.0, 40.0)).with_key("child"));

    assert_eq!(
        result.box_for_node(root).unwrap().key.as_deref(),
        Some("root")
    );
    assert_eq!(result.box_for_key("child").unwrap().node, child);
    assert!(result.box_for_key("missing").is_none());
}

#[test]
fn layout_box_reports_scrollable_content_and_clip_state() {
    let node = NodeId::from_raw(7);
    let layout = LayoutBox::new(node, rect(0.0, 0.0, 100.0, 80.0))
        .with_content_size(Size::new(100.0, 180.0))
        .with_clip(rect(0.0, 0.0, 100.0, 80.0))
        .with_scroll_offset(Vec2::new(0.0, 40.0));

    assert_eq!(layout.scrollable_size(), Size::new(0.0, 100.0));
    assert!(layout.clips_overflow());
    assert_eq!(layout.visible_rect(), rect(0.0, 0.0, 100.0, 80.0));
}

#[test]
fn taffy_backend_populates_rich_metadata_for_all_nodes() {
    use rgui::layout::TaffyLayoutBackend;
    use rgui::runtime::Reconciler;
    use rgui::text_engine::TextSystem;
    use rgui::widgets::{button, text};
    use rgui::{Element, Overflow, Size};

    let root = Element::column()
        .key("viewport")
        .overflow(Overflow::Scroll)
        .height(100.0)
        .child(button("Click Me").key("btn").height(40.0))
        .child(text("Long text").key("body").height(120.0));

    let mut reconciler = Reconciler::default();
    let tree = reconciler.reconcile(root);
    let mut backend = TaffyLayoutBackend::new();
    let mut text_system = TextSystem::default();

    let result = backend.build_from_tree(
        &tree,
        &mut text_system,
        Size::new(200.0, 100.0),
        &rgui::Theme::light(),
    );
    println!("{:#?}", result);

    let viewport_box = result.box_for_key("viewport").unwrap();
    let btn_box = result.box_for_key("btn").unwrap();
    let body_box = result.box_for_key("body").unwrap();

    println!("btn style: {:#?}", tree.get(btn_box.node).unwrap().style);
    println!("body style: {:#?}", tree.get(body_box.node).unwrap().style);

    assert_eq!(viewport_box.key.as_deref(), Some("viewport"));
    assert_eq!(btn_box.key.as_deref(), Some("btn"));
    assert_eq!(body_box.key.as_deref(), Some("body"));

    // Scrollable content height is 40 (btn) + 120 (body) = 160.
    // Viewport height is 100. So max scroll y is 60.
    assert!(viewport_box.content_size.height >= 160.0);
    assert_eq!(
        viewport_box.scrollable_size().height,
        viewport_box.content_size.height - 100.0
    );

    // Verify clips_overflow and visible_rect are populated
    assert!(viewport_box.clips_overflow());
    assert_eq!(viewport_box.clip_rect, Some(viewport_box.local_rect));
}

#[test]
fn taffy_result_exposes_keyed_boxes_and_scrollable_content_size() {
    use rgui::runtime::Reconciler;
    use rgui::text_engine::TextSystem;
    use rgui::{Element, Overflow};

    let root = Element::column()
        .key("scroll")
        .height(80.0)
        .overflow(Overflow::Scroll)
        .child(Element::text("A").height(120.0).key("a"))
        .child(Element::text("B").height(120.0).key("b"));

    let mut reconciler = Reconciler::default();
    let tree = reconciler.reconcile(root);
    let mut backend = rgui::layout::TaffyLayoutBackend::new();
    let mut text = TextSystem::default();
    let result = backend.build_from_tree(
        &tree,
        &mut text,
        rgui::Size::new(240.0, 80.0),
        &rgui::Theme::light(),
    );

    let scroll = result.box_for_key("scroll").expect("scroll layout");
    assert_eq!(scroll.local_rect.size.height, 80.0);
    assert!(scroll.content_size.height >= 240.0);
    assert!(scroll.scrollable_size().height >= 160.0);
    assert!(scroll.clips_overflow());
}
