//! Phase 4 / Plan 04-03 / WIN-02: two `UiRuntime` instances can
//! coexist in the same process.
//!
//! Note: the plan's test sketch referenced a fictional API
//! (`update_with`, `last_output`, `UiSnapshot::window_id`). The
//! actual API is `UiRuntime::update(FrameInput) -> FrameOutput` and
//! `FrameOutput::debug_snapshot() -> UiSnapshot` (the snapshot
//! has `layout`, `tree_nodes`, `display_list`, etc., not a
//! `window_id` field). This test uses the real API and proves the
//! same property: two runtimes built from the same `&ctx` have
//! disjoint state and independent snapshots.
//!
//! WIN-02 invariants verified here:
//! 1. Two `UiRuntime` instances constructed from the same
//!    `&ProcessContext` produce independent `FrameOutput`s.
//! 2. The two runtimes draw `NodeId`s from the same process-global
//!    counter (D-14) — the (window_id, node_id) tuple is unique
//!    process-wide. The sets of node ids referenced by the two
//!    snapshots are disjoint.

use std::collections::HashSet;

use rgui::runtime::{FrameInput, ProcessContext, UiRuntime, WindowId};
use rgui::widgets::{button, text};
use rgui::Element;

fn tree_a() -> Element {
    // Runtime A renders a column with 3 buttons. Runtime B
    // renders a column with 5 buttons. Different number of
    // children => different display list length => independent
    // snapshots.
    Element::column()
        .key("tree-a")
        .child(button("A1").key("a1"))
        .child(button("A2").key("a2"))
        .child(button("A3").key("a3"))
}

fn tree_b() -> Element {
    Element::column()
        .key("tree-b")
        .child(button("B1").key("b1"))
        .child(button("B2").key("b2"))
        .child(button("B3").key("b3"))
        .child(button("B4").key("b4"))
        .child(button("B5").key("b5"))
}

#[test]
fn two_runtimes_in_one_process_have_independent_snapshots() {
    let ctx = ProcessContext::new();
    let mut a = UiRuntime::for_window(WindowId::new(1), &ctx);
    let mut b = UiRuntime::for_window(WindowId::new(2), &ctx);

    // Drive both runtimes with their own trees.
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

    // The two runtimes are bound to different windows.
    assert_eq!(a.window_id(), WindowId::new(1));
    assert_eq!(b.window_id(), WindowId::new(2));

    // The trees differ in content, so the rendered snapshots
    // differ. The display-list length is the most reliable
    // surface for this: same trees produce the same display
    // list, different trees produce different display lists.
    assert_ne!(
        snap_a.display_list.len(),
        snap_b.display_list.len(),
        "two runtimes with different trees should produce different display_list lengths"
    );

    // The layout snapshot is per-runtime; the two runtimes
    // produced two different layout box sets, and the union
    // covers the disjoint `(window_id, node_id)` tuples from
    // the process-global id counter.
    let ids_a: HashSet<u64> = snap_a.layout.iter().map(|b| b.node.raw()).collect();
    let ids_b: HashSet<u64> = snap_b.layout.iter().map(|b| b.node.raw()).collect();
    let intersection: HashSet<u64> = ids_a.intersection(&ids_b).copied().collect();
    assert!(
        intersection.is_empty(),
        "expected disjoint node id sets, but found shared ids: {intersection:?}"
    );
}

#[test]
fn process_context_node_ids_are_shared_across_runtimes() {
    let ctx = ProcessContext::new();
    let mut a = UiRuntime::for_window(WindowId::new(1), &ctx);
    let mut b = UiRuntime::for_window(WindowId::new(2), &ctx);

    // Drive both runtimes with the same tree so each runtime
    // issues a comparable set of node ids from the shared counter.
    let tree = Element::column()
        .key("shared")
        .child(button("x").key("x-btn"))
        .child(text("x").key("x-text"));
    let out_a = a.update(FrameInput {
        root: tree.clone(),
        ..FrameInput::default()
    });
    let out_b = b.update(FrameInput {
        root: tree,
        ..FrameInput::default()
    });

    let snap_a = out_a.debug_snapshot();
    let snap_b = out_b.debug_snapshot();

    // Extract the set of node ids referenced by each runtime's
    // layout snapshot. LayoutBoxSnapshot has a `node: NodeId`
    // field, and NodeId has `raw() -> u64`.
    let ids_a: HashSet<u64> = snap_a.layout.iter().map(|b| b.node.raw()).collect();
    let ids_b: HashSet<u64> = snap_b.layout.iter().map(|b| b.node.raw()).collect();

    // The two runtimes drew from the same process-global counter,
    // so their node id sets are disjoint.
    let intersection: HashSet<u64> = ids_a.intersection(&ids_b).copied().collect();
    assert!(
        intersection.is_empty(),
        "expected disjoint node id sets, but found shared ids: {intersection:?}"
    );

    // The shared counter advanced by the total number of nodes
    // issued across both runtimes. The exact number depends on
    // the reconciler's id issuance; the contract is that the
    // counter is > 0 after a `update()` and that both runtimes
    // advanced it.
    let counter = ctx.node_ids().current();
    assert!(
        counter > 0,
        "expected the process-global counter to advance after updates, got {counter}"
    );
}
