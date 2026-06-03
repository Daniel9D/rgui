//! Unit tests for the `Reconciler::diff` algorithm.
//!
//! Phase 1 / Plan 01-01: the diff between a prior `Element` root and
//! a new `Element` root, producing `mounted` / `unmounted` / `patched`
//! vectors + per-node `DirtyFlags`.

use rgui::core::{Edge, Element, Length, NodeId};
use rgui::runtime::{DirtyFlags, Reconciler, UiTree};
use rgui::widgets::{self, WidgetSpec};

fn make_button(label: &str) -> Element {
    widgets::button(label)
}

fn make_row() -> Element {
    Element::row()
}

fn make_three_node_tree() -> Element {
    let mut root = make_row();
    root.children.push(make_button("Save"));
    root.children.push(make_button("Cancel"));
    root.children.push(make_button("Help"));
    root
}

fn div_with_text(label: &str) -> Element {
    Element::column().child(Element::text(label))
}

fn find_button_with_label(tree: &UiTree, label: &str) -> Option<NodeId> {
    tree.nodes().iter().find_map(|n| {
        if let Some(WidgetSpec::Button(bs)) = &n.widget_spec {
            if bs.label.as_deref() == Some(label) {
                return Some(n.id);
            }
        }
        None
    })
}

#[test]
fn identical_trees_produce_no_mount_or_unmount() {
    // For an identical tree, the diff must not classify any node as
    // mounted or unmounted — the runtime can carry over the prior
    // state. Patches are expected (the diff algorithm marks every
    // node at the same position with the same kind as a patch, which
    // is the path that lets the runtime re-use state).
    let mut r = Reconciler::default();
    let prior = make_three_node_tree();
    let new = make_three_node_tree();
    let diff = r.diff(prior, new);
    let counts = diff.counts();
    assert_eq!(counts.mounted, 0, "no new mounts on identical trees");
    assert_eq!(counts.unmounted, 0, "no unmounts on identical trees");
    // The number of patched nodes equals the number of prior + new nodes,
    // which is the count of nodes the runtime should carry state for.
    assert!(
        counts.patched >= 4,
        "expected at least 4 patched (root + 3 buttons), got {}",
        counts.patched
    );
}

#[test]
fn label_change_is_a_patch() {
    let mut r = Reconciler::default();
    let prior = make_three_node_tree();
    let mut new = make_three_node_tree();
    if let Some(WidgetSpec::Button(bs)) = &mut new.children[1].widget_spec {
        bs.label = Some("Cancelled".to_string());
    }
    let diff = r.diff(prior, new);
    let counts = diff.counts();
    assert_eq!(counts.unmounted, 0, "label change is a patch, not remount");
    assert_eq!(counts.mounted, 0);
    assert!(
        counts.patched >= 4,
        "all 4 nodes should be patched (root + 3 buttons)"
    );
    let middle = find_button_with_label(&diff.tree, "Cancelled").expect("button found");
    let dirty = diff.dirty.get(&middle).copied().unwrap_or_default();
    assert!(
        dirty.contains(DirtyFlags::TEXT)
            && dirty.contains(DirtyFlags::LAYOUT)
            && dirty.contains(DirtyFlags::PAINT),
        "label change must set TEXT+LAYOUT+PAINT, got {:?}",
        dirty
    );
}

#[test]
fn added_child_is_mounted() {
    let mut r = Reconciler::default();
    let prior = make_three_node_tree();
    let mut new = make_three_node_tree();
    new.children.push(make_button("Quit"));
    let diff = r.diff(prior, new);
    let counts = diff.counts();
    assert_eq!(counts.mounted, 1, "one new child mounted");
    assert_eq!(counts.unmounted, 0);
    assert!(counts.patched >= 1, "root was patched for child add");
    let root = diff.tree.root();
    let dirty = diff
        .dirty
        .get(&root)
        .copied()
        .unwrap_or(DirtyFlags::default());
    assert!(
        dirty.contains(DirtyFlags::STRUCTURE)
            && dirty.contains(DirtyFlags::LAYOUT)
            && dirty.contains(DirtyFlags::PAINT)
            && dirty.contains(DirtyFlags::HIT_TEST),
        "parent of added child must set STRUCTURE+LAYOUT+PAINT+HIT_TEST, got {:?}",
        dirty
    );
}

#[test]
fn removed_child_is_unmounted() {
    let mut r = Reconciler::default();
    let prior = make_three_node_tree();
    let mut new = make_three_node_tree();
    new.children.pop();
    let diff = r.diff(prior, new);
    let counts = diff.counts();
    assert_eq!(counts.unmounted, 1, "one child unmounted");
    assert_eq!(counts.mounted, 0);
    assert!(counts.patched >= 1, "root was patched for child removal");
    let root = diff.tree.root();
    let dirty = diff
        .dirty
        .get(&root)
        .copied()
        .unwrap_or(DirtyFlags::default());
    assert!(
        dirty.contains(DirtyFlags::STRUCTURE)
            && dirty.contains(DirtyFlags::LAYOUT)
            && dirty.contains(DirtyFlags::PAINT)
            && dirty.contains(DirtyFlags::HIT_TEST),
        "parent of removed child must set STRUCTURE+LAYOUT+PAINT+HIT_TEST, got {:?}",
        dirty
    );
}

fn find_checkbox_with_label(tree: &UiTree, label: &str) -> Option<NodeId> {
    tree.nodes().iter().find_map(|n| {
        if let Some(WidgetSpec::Checkbox(cs)) = &n.widget_spec {
            if cs.label.as_deref() == Some(label) {
                return Some(n.id);
            }
        }
        None
    })
}

#[test]
fn spec_kind_change_unmounts_old_mounts_new() {
    let mut r = Reconciler::default();
    let prior = make_three_node_tree();
    let mut new = make_three_node_tree();
    new.children[1] = widgets::checkbox();
    // Set a label via spec mutation (CheckboxSpec has label field)
    if let Some(WidgetSpec::Checkbox(cs)) = &mut new.children[1].widget_spec {
        cs.label = Some("Enable".to_string());
    }
    let diff = r.diff(prior, new);
    let counts = diff.counts();
    assert_eq!(counts.unmounted, 1, "old button unmounted");
    assert_eq!(counts.mounted, 1, "new checkbox mounted");
    let cb = find_checkbox_with_label(&diff.tree, "Enable").expect("checkbox found");
    let dirty = diff.dirty.get(&cb).copied().unwrap_or_default();
    assert!(
        dirty.contains(DirtyFlags::LAYOUT)
            && dirty.contains(DirtyFlags::STRUCTURE)
            && dirty.contains(DirtyFlags::PAINT)
            && dirty.contains(DirtyFlags::HIT_TEST),
        "spec-kind mismatch must set LAYOUT+STRUCTURE+PAINT+HIT_TEST on the new node, got {:?}",
        dirty
    );
}

#[test]
fn style_change_sets_style_layout_paint() {
    let mut r = Reconciler::default();
    let prior = div_with_text("hello");
    let mut new = div_with_text("hello");
    new.style.padding = Some(Edge::all(Length::Px(8.0)));
    let diff = r.diff(prior, new);
    let counts = diff.counts();
    assert_eq!(counts.mounted, 0);
    assert_eq!(counts.unmounted, 0);
    let root = diff.tree.root();
    let dirty = diff
        .dirty
        .get(&root)
        .copied()
        .unwrap_or(DirtyFlags::default());
    assert!(
        dirty.contains(DirtyFlags::STYLE)
            && dirty.contains(DirtyFlags::LAYOUT)
            && dirty.contains(DirtyFlags::PAINT),
        "style change must set STYLE+LAYOUT+PAINT, got {:?}",
        dirty
    );
}

#[test]
fn stats_reports_counters() {
    let mut r = Reconciler::default();
    let _ = r.diff(make_row(), make_row());
    let stats = r.stats();
    assert_eq!(stats.keyed_node_count, 0);
    assert_eq!(stats.fingerprint_count, 0);
}
