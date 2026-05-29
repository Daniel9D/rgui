use rgui::layout::TaffyLayoutBackend;
use rgui::runtime::Reconciler;
use rgui::runtime::{FrameInput, UiRuntime};
use rgui::text_engine::TextSystem;
use rgui::widgets::text;
use rgui::{Element, LayoutDirtyReason, Size};

fn layout_with_backend(root: Element, viewport: Size) -> rgui::LayoutResult {
    let mut reconciler = Reconciler::default();
    let tree = reconciler.reconcile(root);
    let mut backend = TaffyLayoutBackend::new();
    let mut text = TextSystem::default();
    backend.build_from_tree(&tree, &mut text, viewport, &rgui::Theme::light())
}

fn assert_layout_boxes_match_by_node(left: &rgui::LayoutResult, right: &rgui::LayoutResult) {
    let mut left = left.boxes.clone();
    let mut right = right.boxes.clone();
    left.sort_by_key(|layout_box| layout_box.node);
    right.sort_by_key(|layout_box| layout_box.node);
    assert_eq!(left, right);
}

fn assert_keyed_layout_geometry_matches(left: &rgui::LayoutResult, right: &rgui::LayoutResult) {
    let mut left: Vec<_> = left
        .boxes
        .iter()
        .filter_map(|layout_box| {
            layout_box
                .key
                .as_ref()
                .map(|key| (key.clone(), layout_box.local_rect, layout_box.content_size))
        })
        .collect();
    let mut right: Vec<_> = right
        .boxes
        .iter()
        .filter_map(|layout_box| {
            layout_box
                .key
                .as_ref()
                .map(|key| (key.clone(), layout_box.local_rect, layout_box.content_size))
        })
        .collect();
    left.sort_by(|a, b| a.0.cmp(&b.0));
    right.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(left, right);
}

#[test]
fn repeated_identical_runtime_frames_preserve_layout_output() {
    let mut runtime = UiRuntime::default();
    let input = || FrameInput {
        root: Element::column()
            .key("root")
            .child(text("Hello").key("label")),
        viewport: Size::new(300.0, 100.0),
        ..Default::default()
    };

    let first = runtime.update(input());
    let second = runtime.update(input());

    assert_eq!(
        first.snapshot.as_ref().unwrap().layout,
        second.snapshot.as_ref().unwrap().layout
    );
    assert_eq!(second.layout_engine, "taffy_first");
}

#[test]
fn layout_dirty_reason_is_public_and_hashable() {
    let mut reasons = std::collections::HashSet::new();
    reasons.insert(LayoutDirtyReason::StyleChanged);
    reasons.insert(LayoutDirtyReason::ViewportChanged);

    assert!(reasons.contains(&LayoutDirtyReason::StyleChanged));
    assert_eq!(reasons.len(), 2);
}

#[test]
fn style_change_reports_dirty_layout_node_count() {
    let mut runtime = UiRuntime::default();
    let first = FrameInput {
        root: Element::column()
            .key("root")
            .child(text("Hello").key("label").width(80.0)),
        viewport: Size::new(300.0, 100.0),
        ..Default::default()
    };
    let second = FrameInput {
        root: Element::column()
            .key("root")
            .child(text("Hello").key("label").width(120.0)),
        viewport: Size::new(300.0, 100.0),
        ..Default::default()
    };

    runtime.update(first);
    let output = runtime.update(second);
    let snapshot = output.snapshot.as_ref().expect("snapshot exists");

    assert!(snapshot.layout_debug.dirty_layout_node_count >= 1);
}

#[test]
fn incremental_layout_matches_full_rebuild_after_style_change() {
    let viewport = Size::new(300.0, 100.0);
    let first = Element::column()
        .key("root")
        .child(text("Hello").key("label").width(80.0));
    let second = Element::column()
        .key("root")
        .child(text("Hello").key("label").width(120.0));

    let mut reconciler = Reconciler::default();
    let first_tree = reconciler.reconcile(first);
    let mut backend = TaffyLayoutBackend::new();
    let mut text_system = TextSystem::default();
    backend.build_from_tree(
        &first_tree,
        &mut text_system,
        viewport,
        &rgui::Theme::light(),
    );

    let second_reconciled = reconciler.reconcile_with_dirty(second.clone());
    let dirty = second_reconciled
        .tree
        .node_for_key("label")
        .map(|node| second_reconciled.tree.ancestors_inclusive(node))
        .unwrap_or_default();
    let incremental = backend.compute_incremental(
        &second_reconciled.tree,
        &mut text_system,
        viewport,
        &rgui::Theme::light(),
        &dirty,
    );
    let full = layout_with_backend(second, viewport);

    assert_keyed_layout_geometry_matches(&incremental, &full);
}

#[test]
fn unchanged_incremental_layout_reuses_existing_taffy_nodes() {
    let viewport = Size::new(300.0, 100.0);
    let root = Element::column()
        .key("root")
        .child(text("Hello").key("label"));

    let mut reconciler = Reconciler::default();
    let first_tree = reconciler.reconcile(root.clone());
    let mut backend = TaffyLayoutBackend::new();
    let mut text_system = TextSystem::default();

    let first = backend.build_from_tree(
        &first_tree,
        &mut text_system,
        viewport,
        &rgui::Theme::light(),
    );
    let second_tree = reconciler.reconcile_with_dirty(root).tree;
    let second = backend.compute_incremental(
        &second_tree,
        &mut text_system,
        viewport,
        &rgui::Theme::light(),
        &[],
    );

    assert_layout_boxes_match_by_node(&second, &first);
    assert_eq!(second.debug.taffy_node_count, first.debug.taffy_node_count);
    assert_eq!(second.debug.dirty_layout_node_count, 0);
}

#[test]
fn incremental_layout_matches_full_rebuild_after_text_change() {
    let viewport = Size::new(180.0, 100.0);
    let first = Element::column()
        .key("root")
        .child(text("Short").key("label"));
    let second = Element::column()
        .key("root")
        .child(text("A much longer label").key("label"));

    let mut reconciler = Reconciler::default();
    let first_tree = reconciler.reconcile(first);
    let mut backend = TaffyLayoutBackend::new();
    let mut text_system = TextSystem::default();
    backend.build_from_tree(
        &first_tree,
        &mut text_system,
        viewport,
        &rgui::Theme::light(),
    );

    let second_reconciled = reconciler.reconcile_with_dirty(second.clone());
    let dirty = second_reconciled
        .tree
        .node_for_key("label")
        .map(|node| second_reconciled.tree.ancestors_inclusive(node))
        .unwrap_or_default();
    let incremental = backend.compute_incremental(
        &second_reconciled.tree,
        &mut text_system,
        viewport,
        &rgui::Theme::light(),
        &dirty,
    );
    let full = layout_with_backend(second, viewport);

    assert_keyed_layout_geometry_matches(&incremental, &full);
}

#[test]
fn incremental_layout_matches_full_rebuild_after_child_reorder() {
    let viewport = Size::new(300.0, 100.0);
    let first = Element::row()
        .key("root")
        .gap(4.0)
        .child(text("A").key("a").width(20.0))
        .child(text("B").key("b").width(40.0));
    let second = Element::row()
        .key("root")
        .gap(4.0)
        .child(text("B").key("b").width(40.0))
        .child(text("A").key("a").width(20.0));

    let mut reconciler = Reconciler::default();
    let first_tree = reconciler.reconcile(first);
    let mut backend = TaffyLayoutBackend::new();
    let mut text_system = TextSystem::default();
    backend.build_from_tree(
        &first_tree,
        &mut text_system,
        viewport,
        &rgui::Theme::light(),
    );

    let second_reconciled = reconciler.reconcile_with_dirty(second.clone());
    let dirty = second_reconciled
        .tree
        .node_for_key("root")
        .map(|node| second_reconciled.tree.ancestors_inclusive(node))
        .unwrap_or_default();
    let incremental = backend.compute_incremental(
        &second_reconciled.tree,
        &mut text_system,
        viewport,
        &rgui::Theme::light(),
        &dirty,
    );
    let full = layout_with_backend(second, viewport);

    assert_keyed_layout_geometry_matches(&incremental, &full);
}

#[test]
fn incremental_preserves_scroll_parent_flex_shrink_override() {
    let viewport = Size::new(180.0, 80.0);
    let first = Element::column()
        .key("scroll")
        .height(80.0)
        .overflow(rgui::Overflow::Scroll)
        .child(text("Short").key("label").height(120.0));
    let second = Element::column()
        .key("scroll")
        .height(80.0)
        .overflow(rgui::Overflow::Scroll)
        .child(text("Much longer label").key("label").height(120.0));

    let mut reconciler = Reconciler::default();
    let first_tree = reconciler.reconcile(first);
    let mut backend = TaffyLayoutBackend::new();
    let mut text_system = TextSystem::default();
    backend.build_from_tree(
        &first_tree,
        &mut text_system,
        viewport,
        &rgui::Theme::light(),
    );

    let second_reconciled = reconciler.reconcile_with_dirty(second.clone());
    let dirty = second_reconciled
        .tree
        .node_for_key("label")
        .into_iter()
        .collect::<Vec<_>>();
    let incremental = backend.compute_incremental(
        &second_reconciled.tree,
        &mut text_system,
        viewport,
        &rgui::Theme::light(),
        &dirty,
    );
    let full = layout_with_backend(second, viewport);

    assert_keyed_layout_geometry_matches(&incremental, &full);
}

#[test]
fn incremental_preserves_stack_flow_child_grid_placement() {
    let viewport = Size::new(240.0, 120.0);
    let first = Element::stack()
        .key("stack")
        .child(text("A").key("a").width(40.0).height(20.0))
        .child(text("B").key("b").width(80.0).height(20.0));
    let second = Element::stack()
        .key("stack")
        .child(text("A updated").key("a").width(60.0).height(20.0))
        .child(text("B").key("b").width(80.0).height(20.0));

    let mut reconciler = Reconciler::default();
    let first_tree = reconciler.reconcile(first);
    let mut backend = TaffyLayoutBackend::new();
    let mut text_system = TextSystem::default();
    backend.build_from_tree(
        &first_tree,
        &mut text_system,
        viewport,
        &rgui::Theme::light(),
    );

    let second_reconciled = reconciler.reconcile_with_dirty(second.clone());
    let dirty = second_reconciled
        .tree
        .node_for_key("a")
        .into_iter()
        .collect::<Vec<_>>();
    let incremental = backend.compute_incremental(
        &second_reconciled.tree,
        &mut text_system,
        viewport,
        &rgui::Theme::light(),
        &dirty,
    );
    let full = layout_with_backend(second, viewport);

    assert_keyed_layout_geometry_matches(&incremental, &full);
}

#[test]
fn backend_reports_propagated_dirty_count_without_rebuilding_all_ancestor_styles() {
    let viewport = Size::new(240.0, 120.0);
    let first = Element::column()
        .key("root")
        .child(Element::column().key("parent").child(text("A").key("leaf")));
    let second = Element::column()
        .key("root")
        .child(Element::column().key("parent").child(text("B").key("leaf")));

    let mut reconciler = Reconciler::default();
    let first_tree = reconciler.reconcile(first);
    let mut backend = TaffyLayoutBackend::new();
    let mut text_system = TextSystem::default();
    backend.build_from_tree(
        &first_tree,
        &mut text_system,
        viewport,
        &rgui::Theme::light(),
    );

    let second_reconciled = reconciler.reconcile_with_dirty(second);
    let dirty = second_reconciled
        .tree
        .node_for_key("leaf")
        .into_iter()
        .collect::<Vec<_>>();

    let result = backend.compute_incremental(
        &second_reconciled.tree,
        &mut text_system,
        viewport,
        &rgui::Theme::light(),
        &dirty,
    );

    assert_eq!(backend.dirty_layout_nodes.len(), 1);
    assert!(result.debug.dirty_layout_node_count >= 3);
}
