use rgui::Element;
use rgui::runtime::{DirtyFlags, FrameInput, Reconciler, UiRuntime, UiTree};
use rgui::widgets::{button, input, text};

#[test]
fn ui_tree_preserves_parent_child_relationships() {
    let root = Element::column()
        .key("root")
        .child(text("Title"))
        .child(button("Save"));

    let tree = UiTree::from_element(root);
    let root_id = tree.root();
    let children = tree.children(root_id);

    assert_eq!(children.len(), 2);
    assert_eq!(tree.parent(children[0]), Some(root_id));
    assert_eq!(tree.parent(children[1]), Some(root_id));
}

#[test]
fn reconciler_preserves_keyed_node_ids_across_reorder() {
    let mut reconciler = Reconciler::default();
    let first = Element::column()
        .child(text("A").key("a"))
        .child(button("B").key("b"));
    let first_tree = reconciler.reconcile(first);
    let a_before = first_tree
        .nodes()
        .iter()
        .find(|node| node.key.as_ref().map(|key| key.as_str()) == Some("a"))
        .unwrap()
        .id;

    let second = Element::column()
        .child(button("B").key("b"))
        .child(text("A").key("a"));
    let second_tree = reconciler.reconcile(second);
    let a_after = second_tree
        .nodes()
        .iter()
        .find(|node| node.key.as_ref().map(|key| key.as_str()) == Some("a"))
        .unwrap()
        .id;

    assert_eq!(a_before, a_after);
}

#[test]
fn changed_kind_marks_layout_paint_semantic_and_hit_test_dirty() {
    let mut reconciler = Reconciler::default();
    reconciler.reconcile(text("Name").key("field"));

    let output = reconciler.reconcile_with_dirty(button("Name").key("field"));
    let dirty = output
        .dirty_for_key("field")
        .expect("dirty flags for keyed node");

    assert!(dirty.contains(DirtyFlags::LAYOUT));
    assert!(dirty.contains(DirtyFlags::PAINT));
    assert!(dirty.contains(DirtyFlags::SEMANTIC));
    assert!(dirty.contains(DirtyFlags::HIT_TEST));
}

#[test]
fn changed_style_marks_layout_and_paint_dirty() {
    let mut reconciler = Reconciler::default();
    reconciler.reconcile(text("Name").key("field").width(80.0));

    let output = reconciler.reconcile_with_dirty(text("Name").key("field").width(120.0));
    let dirty = output
        .dirty_for_key("field")
        .expect("dirty flags for keyed node");

    assert!(dirty.contains(DirtyFlags::STYLE));
    assert!(dirty.contains(DirtyFlags::LAYOUT));
    assert!(dirty.contains(DirtyFlags::PAINT));
}

#[test]
fn changed_text_marks_text_layout_paint_and_semantics_dirty() {
    let mut reconciler = Reconciler::default();
    reconciler.reconcile(text("Before").key("label"));

    let output = reconciler.reconcile_with_dirty(text("After").key("label"));
    let dirty = output
        .dirty_for_key("label")
        .expect("dirty flags for keyed node");

    assert!(dirty.contains(DirtyFlags::TEXT));
    assert!(dirty.contains(DirtyFlags::LAYOUT));
    assert!(dirty.contains(DirtyFlags::PAINT));
    assert!(dirty.contains(DirtyFlags::SEMANTIC));
}

#[test]
fn changed_children_mark_parent_layout_dirty() {
    let mut reconciler = Reconciler::default();
    reconciler.reconcile(Element::column().key("root").child(text("A").key("a")));

    let output = reconciler.reconcile_with_dirty(
        Element::column()
            .key("root")
            .child(text("A").key("a"))
            .child(button("B").key("b")),
    );
    let dirty = output
        .dirty_for_key("root")
        .expect("dirty flags for keyed parent");

    assert!(dirty.contains(DirtyFlags::LAYOUT));
    assert!(dirty.contains(DirtyFlags::HIT_TEST));
}

#[test]
fn keyed_input_state_survives_reorder_through_state_arena() {
    let mut runtime = UiRuntime::default();
    let viewport = rgui::Size::new(320.0, 120.0);

    runtime.update(FrameInput {
        root: Element::column()
            .child(input().key("name").default_value("Ada"))
            .child(button("Save").key("save")),
        viewport,
        ..Default::default()
    });

    runtime.update(FrameInput {
        root: Element::column()
            .child(button("Save").key("save"))
            .child(input().key("name").default_value("Ada")),
        viewport,
        ..Default::default()
    });

    assert_eq!(runtime.text_state("name").as_deref(), Some("Ada"));
}
