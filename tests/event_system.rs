use rgui::core::{HitTestEntry, LayerKind, NodeId, Point, Rect, Size};
use rgui::runtime::{EventPath, FocusScope, FocusSystem, Reconciler, TabIndex};
use rgui::widgets::button;

#[test]
fn event_path_contains_ancestors_in_order() {
    let root = rgui::Element::column()
        .key("root")
        .child(button("Click").key("btn"));

    let mut reconciler = Reconciler::default();
    let tree = reconciler.reconcile(root);

    let btn_node = tree
        .nodes()
        .iter()
        .find(|n| n.key.as_ref().is_some_and(|k| k.as_str() == "btn"))
        .expect("btn exists");
    let root_node = tree
        .nodes()
        .iter()
        .find(|n| n.key.as_ref().is_some_and(|k| k.as_str() == "root"))
        .expect("root exists");

    let hit = HitTestEntry::new(
        btn_node.id,
        Rect::new(Point::new(0.0, 0.0), Size::new(100.0, 32.0)),
        0,
        LayerKind::Document,
    );
    let path = EventPath::build(&hit, &tree);

    assert_eq!(path.target(), btn_node.id);
    let capture: Vec<_> = path.capture_phase().collect();
    let bubble: Vec<_> = path.bubble_phase().collect();
    assert_eq!(capture, vec![root_node.id]);
    assert_eq!(path.target_phase(), btn_node.id);
    assert_eq!(bubble, vec![root_node.id]);
}

#[test]
fn focus_scope_tab_cycles_through_entries() {
    let mut scope = FocusScope::new(rgui::runtime::FocusScopeId::from_raw(1));
    scope.push_entry(NodeId::from_raw(1), Some("a".into()), TabIndex::Auto);
    scope.push_entry(NodeId::from_raw(2), Some("b".into()), TabIndex::Auto);
    scope.push_entry(NodeId::from_raw(3), Some("c".into()), TabIndex::Auto);

    let first = scope.advance().expect("first entry");
    assert_eq!(first.1.as_deref(), Some("a"));

    let second = scope.advance().expect("second entry");
    assert_eq!(second.1.as_deref(), Some("b"));

    let third = scope.advance().expect("third entry");
    assert_eq!(third.1.as_deref(), Some("c"));

    let wraps = scope.advance().expect("wraps to first");
    assert_eq!(wraps.1.as_deref(), Some("a"));
}

#[test]
fn tabindex_none_is_skipped() {
    let mut scope = FocusScope::new(rgui::runtime::FocusScopeId::from_raw(1));
    scope.push_entry(NodeId::from_raw(1), Some("a".into()), TabIndex::None);
    scope.push_entry(NodeId::from_raw(2), Some("b".into()), TabIndex::Auto);

    let entry = scope.advance().expect("skips None");
    assert_eq!(entry.1.as_deref(), Some("b"));
}

#[test]
fn focus_system_manages_scopes() {
    let mut system = FocusSystem::new();
    let scope_a = system.create_scope();
    system.set_active(scope_a);

    if let Some(scope) = system.scope_mut(scope_a) {
        scope.push_entry(NodeId::from_raw(1), Some("first".into()), TabIndex::Auto);
        scope.push_entry(NodeId::from_raw(2), Some("second".into()), TabIndex::Auto);
    }

    let result = system.tab_forward().expect("tab forward works");
    assert_eq!(result.1.as_deref(), Some("first"));
}
