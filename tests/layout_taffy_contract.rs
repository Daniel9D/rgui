use rgui::runtime::{FrameInput, FrameOutput, UiRuntime};
use rgui::widgets::text;
use rgui::{Element, ElementKind, Length, Point, PrimitiveKind, Size, Style};

fn frame(root: Element, viewport: Size) -> FrameOutput {
    let mut runtime = UiRuntime::default();
    runtime.update(FrameInput {
        root,
        viewport,
        ..Default::default()
    })
}

fn box_element() -> Element {
    Element::new(ElementKind::Primitive(PrimitiveKind::Column))
}

#[test]
fn row_gap_and_alignment_are_reported_in_runtime_snapshot() {
    let output = frame(
        Element::row()
            .key("root")
            .width(300.0)
            .height(100.0)
            .gap(10.0)
            .child(box_element().key("a").width(40.0).height(20.0))
            .child(box_element().key("b").width(30.0).height(40.0)),
        Size::new(300.0, 100.0),
    );

    assert_eq!(output.layout_engine, "taffy_first");
    let snapshot = output.snapshot.as_ref().expect("snapshot exists");
    let a = snapshot.layout_box("a").expect("a layout");
    let b = snapshot.layout_box("b").expect("b layout");

    assert_eq!(a.x, 0.0);
    assert_eq!(b.x, 50.0);
}

#[test]
fn column_padding_and_gap_are_reported_in_runtime_snapshot() {
    let output = frame(
        Element::column()
            .key("root")
            .padding(8.0)
            .gap(6.0)
            .width(160.0)
            .height(180.0)
            .child(box_element().key("a").height(30.0))
            .child(box_element().key("b").height(40.0)),
        Size::new(160.0, 180.0),
    );

    let snapshot = output.snapshot.as_ref().expect("snapshot exists");
    let a = snapshot.layout_box("a").expect("a layout");
    let b = snapshot.layout_box("b").expect("b layout");

    assert_eq!(a.x, 8.0);
    assert_eq!(a.y, 8.0);
    assert_eq!(b.y, 44.0);
}

#[test]
fn root_auto_size_uses_viewport() {
    let output = frame(
        Element::column().key("root").child(text("A").key("a")),
        Size::new(640.0, 480.0),
    );
    let root = output
        .snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.layout_box("root"))
        .expect("root layout");

    assert_eq!(root.width, 640.0);
    assert_eq!(root.height, 480.0);
}

#[test]
fn explicit_percent_and_min_max_sizes_are_resolved_by_taffy() {
    let mut child_style = Style::default();
    child_style.width = Some(Length::Percent(0.5));
    child_style.height = Some(Length::Px(20.0));
    child_style.min_width = Some(Length::Px(150.0));
    child_style.max_width = Some(Length::Px(180.0));

    let output = frame(
        Element::row()
            .key("root")
            .width(400.0)
            .height(100.0)
            .child(box_element().key("child").style(child_style)),
        Size::new(400.0, 100.0),
    );
    let child = output
        .snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.layout_box("child"))
        .expect("child layout");

    assert_eq!(child.width, 180.0);
    assert_eq!(child.height, 20.0);
}

#[test]
fn stack_children_overlay_at_origin_unless_positioned() {
    let output = frame(
        Element::stack()
            .key("stack")
            .width(100.0)
            .height(100.0)
            .child(box_element().key("a").width(40.0).height(40.0))
            .child(box_element().key("b").width(60.0).height(60.0)),
        Size::new(100.0, 100.0),
    );
    let snapshot = output.snapshot.as_ref().expect("snapshot exists");
    let a = snapshot.layout_box("a").expect("a layout");
    let b = snapshot.layout_box("b").expect("b layout");

    assert_eq!(Point::new(a.x, a.y), Point::new(0.0, 0.0));
    assert_eq!(Point::new(b.x, b.y), Point::new(0.0, 0.0));
}

#[test]
fn tabs_percentage_padding_is_preserved_by_theme_header_minimum() {
    let mut style = rgui::Style::default();
    style.padding = Some(rgui::Edge::all(rgui::Length::Percent(0.2)));
    let root = rgui::widgets::tabs()
        .key("tabs")
        .style(style)
        .child(Element::text("Panel").key("panel"));

    let mut reconciler = rgui::runtime::Reconciler::default();
    let tree = reconciler.reconcile(root);
    let mut backend = rgui::layout::TaffyLayoutBackend::new();
    let mut text = rgui::text_engine::TextSystem::default();
    let result = backend.build_from_tree(
        &tree,
        &mut text,
        rgui::Size::new(300.0, 200.0),
        &rgui::Theme::light(),
    );

    let tabs = result.box_for_key("tabs").expect("tabs layout");
    let panel = result.box_for_key("panel").expect("panel layout");
    assert!(panel.local_rect.origin.y > tabs.local_rect.origin.y);
}
