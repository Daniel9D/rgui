#[test]
fn runtime_frame_builder_does_not_allocate_transient_node_ids() {
    let source = include_str!("../src/runtime/runtime.rs");

    assert!(
        source.contains("reconciler: Reconciler"),
        "UiRuntime should own a Reconciler"
    );
    assert!(
        source.contains("tree: Option<UiTree>"),
        "UiRuntime should retain the latest reconciled UiTree"
    );
    assert!(
        !source.contains("next_node += 1"),
        "Frame building should use UiNode.id from the reconciled tree"
    );
    assert!(
        !source.contains("NodeId::from_raw(self.next_node"),
        "Frame building should not synthesize per-frame NodeId values"
    );
}

#[test]
fn runtime_has_widget_specific_painters() {
    let runtime_mod = include_str!("../src/runtime/mod.rs");
    let paint_source = include_str!("../src/runtime/paint.rs");

    assert!(runtime_mod.contains("pub mod paint;"));
    assert!(paint_source.contains("paint_widget"));
    assert!(paint_source.contains("WidgetKind::Input"));
    assert!(paint_source.contains("WidgetKind::Checkbox"));
}
