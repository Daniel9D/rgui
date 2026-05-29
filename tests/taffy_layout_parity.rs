use rgui::core::{
    Border, Edge, Element, GridPlacement, GridTrack, LayoutBox, LayoutResult, Length, NodeId, Style,
};
use rgui::layout::TaffyLayoutBackend;
use rgui::runtime::Reconciler;
use rgui::text_engine::TextSystem;

fn box_for(result: &LayoutResult, node: NodeId) -> &LayoutBox {
    result
        .boxes
        .iter()
        .find(|layout| layout.node == node)
        .expect("layout box exists")
}

fn node_for_key(tree: &rgui::runtime::UiTree, key: &str) -> NodeId {
    tree.nodes()
        .iter()
        .find(|node| {
            node.key
                .as_ref()
                .is_some_and(|node_key| node_key.as_str() == key)
        })
        .map(|node| node.id)
        .expect("keyed node exists")
}

fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= 1.0,
        "actual={actual} expected={expected}"
    );
}

#[test]
fn taffy_maps_column_with_padding_and_gap() {
    let root = Element::column()
        .padding(8.0)
        .gap(4.0)
        .child(Element::text("Hello").key("a"))
        .child(Element::text("World").key("b"));

    let mut reconciler = Reconciler::default();
    let tree = reconciler.reconcile(root);

    let mut backend = TaffyLayoutBackend::new();
    let mut text = TextSystem::default();
    let result = backend.build_from_tree(
        &tree,
        &mut text,
        rgui::core::Size::new(400.0, 600.0),
        &rgui::Theme::light(),
    );

    let a = box_for(&result, node_for_key(&tree, "a"));
    let b = box_for(&result, node_for_key(&tree, "b"));
    let root = result
        .boxes
        .iter()
        .find(|layout| layout.node == tree.root_node().id)
        .expect("root layout");

    assert_close(a.local_rect.origin.x, 8.0);
    assert_close(a.local_rect.origin.y, 8.0);
    assert_close(b.local_rect.origin.x, 8.0);
    assert!(b.local_rect.origin.y >= a.local_rect.origin.y + a.local_rect.size.height + 4.0);
    assert_close(root.content_rect.origin.x, 8.0);
    assert_close(root.content_rect.origin.y, 8.0);
}

#[test]
fn taffy_row_positions_children_side_by_side() {
    let root = Element::row()
        .gap(8.0)
        .child(Element::text("A").key("a"))
        .child(Element::text("B").key("b"));

    let mut reconciler = Reconciler::default();
    let tree = reconciler.reconcile(root);

    let mut backend = TaffyLayoutBackend::new();
    let mut text = TextSystem::default();
    let result = backend.build_from_tree(
        &tree,
        &mut text,
        rgui::core::Size::new(400.0, 600.0),
        &rgui::Theme::light(),
    );

    let a = box_for(&result, node_for_key(&tree, "a"));
    let b = box_for(&result, node_for_key(&tree, "b"));

    assert_close(a.local_rect.origin.y, b.local_rect.origin.y);
    assert!(b.local_rect.origin.x >= a.local_rect.origin.x + a.local_rect.size.width + 8.0);
}

#[test]
fn taffy_percentage_width_resolves_against_parent() {
    let root = Element::column().width(400.0).child(
        Element::text("Half")
            .width(Length::Percent(0.5))
            .key("half"),
    );

    let mut reconciler = Reconciler::default();
    let tree = reconciler.reconcile(root);

    let mut backend = TaffyLayoutBackend::new();
    let mut text = TextSystem::default();
    let result = backend.build_from_tree(
        &tree,
        &mut text,
        rgui::core::Size::new(400.0, 600.0),
        &rgui::Theme::light(),
    );

    let half = box_for(&result, node_for_key(&tree, "half"));

    assert_close(half.local_rect.size.width, 200.0);
}

#[test]
fn taffy_overflow_hidden_sets_clip_rect() {
    let root = Element::column()
        .height(40.0)
        .overflow(rgui::Overflow::Hidden)
        .child(Element::text("Overflow").height(100.0).key("child"));

    let mut reconciler = Reconciler::default();
    let tree = reconciler.reconcile(root);

    let mut backend = TaffyLayoutBackend::new();
    let mut text = TextSystem::default();
    let result = backend.build_from_tree(
        &tree,
        &mut text,
        rgui::core::Size::new(400.0, 600.0),
        &rgui::Theme::light(),
    );

    let root = box_for(&result, tree.root_node().id);

    assert_eq!(root.clip_rect, Some(root.local_rect));
}

#[test]
fn taffy_grid_tracks_and_placement_affect_layout() {
    let mut root_style = Style::default().display(rgui::core::Display::Grid);
    root_style.grid_template_columns =
        Some(vec![GridTrack::Fixed(Length::Px(80.0)), GridTrack::fr(1.0)]);
    root_style.grid_template_rows = Some(vec![GridTrack::Fixed(Length::Px(24.0))]);

    let mut placed_style = Style::default();
    placed_style.grid_column = Some(GridPlacement {
        start: Some(2),
        end: Some(3),
        span: None,
    });
    placed_style.grid_row = Some(GridPlacement::start(1));

    let root = Element::grid()
        .width(200.0)
        .height(60.0)
        .style(root_style)
        .child(Element::text("A").key("a"))
        .child(Element::text("B").key("b").style(placed_style));

    let mut reconciler = Reconciler::default();
    let tree = reconciler.reconcile(root);

    let mut backend = TaffyLayoutBackend::new();
    let mut text = TextSystem::default();
    let result = backend.build_from_tree(
        &tree,
        &mut text,
        rgui::core::Size::new(200.0, 60.0),
        &rgui::Theme::light(),
    );

    let a = box_for(&result, node_for_key(&tree, "a"));
    let b = box_for(&result, node_for_key(&tree, "b"));

    assert_close(a.local_rect.origin.x, 0.0);
    assert!(
        b.local_rect.origin.x >= 80.0,
        "explicit grid placement should move B into the second column"
    );
}

#[test]
fn taffy_style_mapping_includes_new_fields() {
    let mut style = Style::default().display(rgui::core::Display::Flex);
    style.border = Some(Border {
        color: rgui::Color::rgb(0, 0, 0),
        width: 2.0,
    });
    style.aspect_ratio = Some(16.0 / 9.0);
    style.position = Some(rgui::Position::Absolute);
    style.inset = Some(Edge {
        top: Length::Px(3.0),
        right: Length::Px(4.0),
        bottom: Length::Px(5.0),
        left: Length::Px(6.0),
    });
    style.grid_template_columns = Some(vec![
        GridTrack::Fixed(Length::Px(120.0)),
        GridTrack::fr(1.0),
    ]);
    style.grid_template_rows = Some(vec![GridTrack::Auto, GridTrack::Fixed(Length::Px(32.0))]);
    style.grid_column = Some(GridPlacement {
        start: Some(1),
        end: Some(3),
        span: None,
    });
    style.grid_row = Some(GridPlacement::span(2));

    let taffy_style = rgui::layout::to_taffy_style(&style);

    assert_eq!(taffy_style.display, taffy::Display::Flex);
    assert_eq!(taffy_style.flex_grow, 0.0);
    assert_eq!(taffy_style.flex_shrink, 1.0);
    assert_eq!(taffy_style.position, taffy::Position::Absolute);
    assert_eq!(taffy_style.aspect_ratio, Some(16.0 / 9.0));
    assert_eq!(
        taffy_style.border.left,
        taffy::LengthPercentage::length(2.0)
    );
    assert_eq!(
        taffy_style.inset.left,
        taffy::LengthPercentageAuto::length(6.0)
    );
    assert_eq!(
        taffy_style.inset.top,
        taffy::LengthPercentageAuto::length(3.0)
    );
    assert_eq!(taffy_style.grid_template_columns.len(), 2);
    assert_eq!(taffy_style.grid_template_rows.len(), 2);
    let taffy::GridTemplateComponent::Single(first_column) = taffy_style.grid_template_columns[0]
    else {
        panic!("expected single first column track");
    };
    assert_eq!(
        first_column
            .max_sizing_function()
            .definite_value(Some(500.0), |_, _| 0.0),
        Some(120.0)
    );
    let taffy::GridTemplateComponent::Single(second_column) = taffy_style.grid_template_columns[1]
    else {
        panic!("expected single second column track");
    };
    assert!(second_column.max_sizing_function().is_fr());
    assert!(taffy_style.grid_column.is_definite());
    assert_eq!(taffy_style.grid_row.start, taffy::GridPlacement::Span(2));
}

#[test]
fn intrinsic_widget_sizes_are_centralized_and_stable() {
    use rgui::WidgetKind;
    use rgui::layout::{WidgetIntrinsicInput, intrinsic_widget_size};

    let theme = rgui::Theme::light();
    let button = intrinsic_widget_size(
        WidgetIntrinsicInput {
            widget_kind: WidgetKind::Button,
            label_width: Some(40.0),
            known_width: None,
            known_height: None,
        },
        &theme.widgets.metrics,
    );
    let input = intrinsic_widget_size(
        WidgetIntrinsicInput {
            widget_kind: WidgetKind::Input,
            label_width: None,
            known_width: None,
            known_height: None,
        },
        &theme.widgets.metrics,
    );

    assert_eq!(button.width, 72.0);
    assert_eq!(button.height, 32.0);
    assert_eq!(input.width, 160.0);
    assert_eq!(input.height, 36.0);
}

#[test]
fn known_intrinsic_dimensions_override_defaults() {
    use rgui::WidgetKind;
    use rgui::layout::{WidgetIntrinsicInput, intrinsic_widget_size};

    let theme = rgui::Theme::light();
    let size = intrinsic_widget_size(
        WidgetIntrinsicInput {
            widget_kind: WidgetKind::Select,
            label_width: None,
            known_width: Some(220.0),
            known_height: Some(44.0),
        },
        &theme.widgets.metrics,
    );

    assert_eq!(size.width, 220.0);
    assert_eq!(size.height, 44.0);
}

#[test]
fn taffy_flex_grow_distributes_remaining_width() {
    let root = Element::row()
        .width(300.0)
        .child(Element::text("A").width(50.0).key("fixed"))
        .child({
            let mut element = Element::text("Grow").key("grow");
            element.style.flex_grow = Some(1.0);
            element
        });

    let mut reconciler = Reconciler::default();
    let tree = reconciler.reconcile(root);
    let mut backend = TaffyLayoutBackend::new();
    let mut text = TextSystem::default();
    let result = backend.build_from_tree(
        &tree,
        &mut text,
        rgui::core::Size::new(300.0, 120.0),
        &rgui::Theme::light(),
    );

    let grow = box_for(&result, node_for_key(&tree, "grow"));
    assert!(grow.local_rect.size.width >= 240.0);
}

#[test]
fn taffy_min_and_max_width_constrain_percent_width() {
    let mut child = Element::text("Constrained")
        .width(Length::Percent(1.0))
        .key("child");
    child.style.min_width = Some(Length::Px(120.0));
    child.style.max_width = Some(Length::Px(160.0));

    let root = Element::column().width(300.0).child(child);
    let mut reconciler = Reconciler::default();
    let tree = reconciler.reconcile(root);
    let mut backend = TaffyLayoutBackend::new();
    let mut text = TextSystem::default();
    let result = backend.build_from_tree(
        &tree,
        &mut text,
        rgui::core::Size::new(300.0, 120.0),
        &rgui::Theme::light(),
    );

    let child = box_for(&result, node_for_key(&tree, "child"));
    assert_close(child.local_rect.size.width, 160.0);
}

#[test]
fn taffy_absolute_position_uses_inset_mapping() {
    let mut child_style = Style::default();
    child_style.position = Some(rgui::core::Position::Absolute);
    child_style.inset = Some(Edge {
        top: Length::Px(10.0),
        right: Length::Auto,
        bottom: Length::Auto,
        left: Length::Px(20.0),
    });

    let root = Element::stack()
        .width(200.0)
        .height(120.0)
        .child(Element::text("Abs").style(child_style).key("abs"));

    let mut reconciler = Reconciler::default();
    let tree = reconciler.reconcile(root);
    let mut backend = TaffyLayoutBackend::new();
    let mut text = TextSystem::default();
    let result = backend.build_from_tree(
        &tree,
        &mut text,
        rgui::core::Size::new(200.0, 120.0),
        &rgui::Theme::light(),
    );

    let abs = box_for(&result, node_for_key(&tree, "abs"));
    assert_close(abs.local_rect.origin.x, 20.0);
    assert_close(abs.local_rect.origin.y, 10.0);
}

#[test]
fn taffy_stack_layout_overlays_flow_children_at_origin() {
    let root = Element::stack()
        .width(200.0)
        .height(120.0)
        .child(
            Element::text("Child1")
                .width(100.0)
                .height(50.0)
                .key("child1"),
        )
        .child(
            Element::text("Child2")
                .width(150.0)
                .height(60.0)
                .key("child2"),
        );

    let mut reconciler = Reconciler::default();
    let tree = reconciler.reconcile(root);
    let mut backend = TaffyLayoutBackend::new();
    let mut text = TextSystem::default();
    let result = backend.build_from_tree(
        &tree,
        &mut text,
        rgui::core::Size::new(200.0, 120.0),
        &rgui::Theme::light(),
    );

    let child1 = box_for(&result, node_for_key(&tree, "child1"));
    let child2 = box_for(&result, node_for_key(&tree, "child2"));

    // Verify both children start at (0, 0) relative to parent
    assert_close(child1.local_rect.origin.x, 0.0);
    assert_close(child1.local_rect.origin.y, 0.0);
    assert_close(child2.local_rect.origin.x, 0.0);
    assert_close(child2.local_rect.origin.y, 0.0);

    // Verify sizes are preserved
    assert_close(child1.local_rect.size.width, 100.0);
    assert_close(child1.local_rect.size.height, 50.0);
    assert_close(child2.local_rect.size.width, 150.0);
    assert_close(child2.local_rect.size.height, 60.0);
}
