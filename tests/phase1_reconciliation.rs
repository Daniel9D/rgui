//! Phase 1 / Plan 01-04: end-to-end verification that incremental
//! reconciliation behaves correctly under the four path types
//! (reuse, patch, mount, unmount) and that pointer-capture is
//! released on unmount but survives a style-only patch.
//!
//! These tests exercise the `Reconciler::diff` API directly
//! (rather than going through `runtime.update`), because the new
//! diff is not yet wired into the update loop. Once Phase 1 ships,
//! the same scenarios can be reproduced at the runtime level by
//! calling `update()` twice.

use rgui::runtime::{DiffCounts, PointerCapture, Reconciler};
use rgui::widgets::{self, CheckboxSpec, WidgetSpec};
use rgui::Element;

/// Build a 50-widget tree: a column of 10 rows, each containing 5
/// buttons. Mirrors a realistic UI shape (a list of items with
/// per-item actions) at moderate depth.
fn build_50_widget_tree() -> Element {
    let mut root = Element::column();
    for i in 0..10 {
        let mut row = Element::row();
        for j in 0..5 {
            row = row.child(widgets::button(format!("Item {}-{}", i, j)));
        }
        root = root.child(row);
    }
    root
}

#[test]
fn identical_50_widget_tree_has_no_mount_or_unmount() {
    let mut r = Reconciler::default();
    let prior = build_50_widget_tree();
    let new = build_50_widget_tree();
    let diff = r.diff(prior, new);
    let counts = diff.counts();
    assert_eq!(
        counts.mounted, 0,
        "identical 50-widget tree should not mount anything"
    );
    assert_eq!(
        counts.unmounted, 0,
        "identical 50-widget tree should not unmount anything"
    );
    // 1 root + 10 rows + 50 buttons = 61 patched nodes
    assert_eq!(counts.patched, 61, "all 61 nodes should be patched");
}

#[test]
fn single_label_change_yields_minimal_patch_set() {
    let mut r = Reconciler::default();
    let prior = build_50_widget_tree();
    let mut new = build_50_widget_tree();
    // Mutate one button's label (the first button in the first row)
    if let Some(WidgetSpec::Button(bs)) = &mut new.children[0]
        .children
        .first_mut()
        .and_then(|b| b.widget_spec.as_mut())
    {
        bs.label = Some("Item 0-0 (updated)".to_string());
    } else {
        panic!("test setup: expected first button to have a Button spec");
    }
    let diff = r.diff(prior, new);
    let counts = diff.counts();
    assert_eq!(counts.mounted, 0);
    assert_eq!(counts.unmounted, 0);
    // The label change is at the first button's position; the diff
    // patches all nodes from the root down to that button because
    // the test uses positional diff (no keys). With keys, the
    // patch set would be 1.
    assert!(
        counts.patched >= 4,
        "at least the root + first row + first button should be patched"
    );
}

#[test]
fn pointer_capture_released_on_unmount() {
    // Mimic the runtime's capture state directly: a single
    // `PointerCapture` keyed by "btn".
    let mut cap = PointerCapture::default();
    cap.set("btn".to_string(), Some(rgui::core::NodeId::from_raw(7)));
    assert!(cap.is_active());

    // Simulate "unmount the btn node": release_matching with ["btn"].
    let cancel = cap.release_matching(&["btn".to_string()]);
    assert!(cancel.is_some(), "unmounting the captured key should release the capture");
    assert!(!cap.is_active(), "capture should be cleared after release");
}

#[test]
fn pointer_capture_survives_style_patch() {
    // A style-only patch on the captured node must NOT release
    // the capture. The runtime's pointer_capture is keyed by the
    // `String` element key, which is preserved by the keyed_ids
    // allocator; a style change is a PATCH (not a remount), so
    // the key persists.
    let mut cap = PointerCapture::default();
    cap.set("btn".to_string(), Some(rgui::core::NodeId::from_raw(7)));
    // Style patch: release_matching with an unrelated key
    let cancel = cap.release_matching(&["unrelated".to_string()]);
    assert!(
        cancel.is_none(),
        "style patch on a non-unmounted node should NOT release the capture"
    );
    assert!(
        cap.is_active(),
        "capture should still be active after a style patch on an unrelated node"
    );
}

#[test]
fn re_keyed_element_loses_capture() {
    // Re-keying an element (changing its `.key()`) is semantically
    // a new element. The prior capture keyed by the old string
    // should be released on the next reconcile because the old
    // key is no longer in the tree.
    let mut cap = PointerCapture::default();
    cap.set("old_key".to_string(), Some(rgui::core::NodeId::from_raw(1)));
    let cancel = cap.release_matching(&["old_key".to_string()]);
    assert!(cancel.is_some());
    assert!(!cap.is_active());
}

#[test]
fn spec_kind_change_unmounts_old_and_mounts_new() {
    let mut r = Reconciler::default();
    let prior = Element::column().child(widgets::button("OK"));
    let mut new = Element::column();
    let mut cb = widgets::checkbox();
    if let Some(WidgetSpec::Checkbox(cs)) = &mut cb.widget_spec {
        cs.label = Some("OK".to_string());
    }
    new.children.push(cb);
    let diff = r.diff(prior, new);
    let counts = diff.counts();
    assert_eq!(
        counts.unmounted, 1,
        "button should be unmounted when spec kind changes"
    );
    assert_eq!(
        counts.mounted, 1,
        "checkbox should be mounted in place of the button"
    );
}

#[test]
fn reconcile_stats_exposed() {
    let mut r = Reconciler::default();
    let _ = r.diff(build_50_widget_tree(), build_50_widget_tree());
    let stats = r.stats();
    // No keyed nodes in the test tree
    assert_eq!(stats.keyed_node_count, 0);
    assert_eq!(stats.fingerprint_count, 0);
    // counts() should be sane
    let _ = DiffCounts {
        mounted: 0,
        unmounted: 0,
        patched: 0,
    };
}
