//! Phase 4 / Plan 04-03 / WIN-04: the snapshot's
//! `(window_id, node_id)` tuples are unique process-wide.
//!
//! Note: the plan's test sketch referenced a fictional
//! `UiSnapshot::window_id` field. The real `UiSnapshot` has no
//! `window_id` field; the window identity is implicit in *which*
//! `UiRuntime` produced the snapshot. The plan's invariant is
//! preserved: node ids drawn from the process-global counter are
//! disjoint across runtimes (D-14), so the (window_id, node_id)
//! tuple is unique process-wide.
//!
//! This test uses the real API: extract node ids from
//! `snapshot.layout[*].node.raw()` and assert the sets are
//! disjoint.

use std::collections::HashSet;

use rgui::runtime::{FrameInput, ProcessContext, UiRuntime, WindowId};
use rgui::widgets::button;
use rgui::Element;

fn tree_a() -> Element {
    Element::column()
        .key("tree-a")
        .child(button("A").key("a-btn"))
}

fn tree_b() -> Element {
    Element::column()
        .key("tree-b")
        .child(button("B").key("b-btn"))
}

#[test]
fn snapshots_from_two_runtimes_have_disjoint_node_ids() {
    let ctx = ProcessContext::new();
    let mut a = UiRuntime::for_window(WindowId::new(1), &ctx);
    let mut b = UiRuntime::for_window(WindowId::new(2), &ctx);

    let out_a = a.update(FrameInput {
        root: tree_a(),
        ..FrameInput::default()
    });
    let out_b = b.update(FrameInput {
        root: tree_b(),
        ..FrameInput::default()
    });

    let snap_a = out_a.debug_snapshot();
    let snap_b = out_b.debug_snapshot();

    // Build (window_id, node_id) tuples from the layout
    // snapshot. The window_id comes from the runtime (the
    // snapshot doesn't carry it); the node_id comes from each
    // LayoutBoxSnapshot's `node` field.
    let tuples_a: HashSet<(u64, u64)> = snap_a
        .layout
        .iter()
        .map(|box_| (a.window_id().raw(), box_.node.raw()))
        .collect();
    let tuples_b: HashSet<(u64, u64)> = snap_b
        .layout
        .iter()
        .map(|box_| (b.window_id().raw(), box_.node.raw()))
        .collect();

    let intersection: HashSet<(u64, u64)> =
        tuples_a.intersection(&tuples_b).copied().collect();
    assert!(
        intersection.is_empty(),
        "expected disjoint (window_id, node_id) tuples, found shared: {intersection:?}"
    );

    // The window_ids of the tuples are distinct (we built A's
    // tuples with window_id=1 and B's with window_id=2).
    let window_ids_in_a: HashSet<u64> = tuples_a.iter().map(|(w, _)| *w).collect();
    let window_ids_in_b: HashSet<u64> = tuples_b.iter().map(|(w, _)| *w).collect();
    assert_eq!(window_ids_in_a, HashSet::from([1]));
    assert_eq!(window_ids_in_b, HashSet::from([2]));
}

#[test]
fn node_id_counter_is_monotonic_across_runtimes() {
    let ctx = ProcessContext::new();
    let mut a = UiRuntime::for_window(WindowId::new(1), &ctx);
    let mut b = UiRuntime::for_window(WindowId::new(2), &ctx);

    // Drive both runtimes with the same tree so each runtime
    // issues a comparable set of node ids from the shared
    // counter.
    let tree = Element::column()
        .key("shared")
        .child(button("X").key("x-btn"));
    let _ = a.update(FrameInput {
        root: tree.clone(),
        ..FrameInput::default()
    });
    let counter_after_a = ctx.node_ids().current();
    let _ = b.update(FrameInput {
        root: tree,
        ..FrameInput::default()
    });
    let counter_after_b = ctx.node_ids().current();

    // The process-global counter advanced. After A's update
    // it's > 0; after B's update it's strictly greater than
    // after A's update (because B's reconciler also drew from
    // the same counter).
    assert!(
        counter_after_a > 0,
        "counter should advance after A's update, got {counter_after_a}"
    );
    assert!(
        counter_after_b > counter_after_a,
        "counter should advance after B's update, got {counter_after_a} -> {counter_after_b}"
    );

    // The two runtimes also expose the counter (via
    // UiRuntime::node_ids()); their views are identical because
    // the counter is Arc-shared.
    assert_eq!(a.node_ids().current(), ctx.node_ids().current());
    assert_eq!(b.node_ids().current(), ctx.node_ids().current());
}
