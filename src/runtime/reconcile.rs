//! Tree reconciliation between the user-supplied `Element` tree and
//! the runtime's `UiTree`.
//!
//! The reconciler takes the new element tree produced by the user
//! each frame and produces a minimal-diff update to the live
//! `UiTree` (allocating only new `NodeId`s, preserving keyed nodes,
//! and emitting dirty flags for downstream passes). The key
//! invariants are:
//! - The `keyed_ids` map makes consecutive frames with the same
//!   keyed structure stable — the same `NodeId` is reused.
//! - `DirtyFlags` is the only signal downstream passes need to
//!   re-layout / re-paint a node; everything else can be read
//!   from `UiTree` directly.
//! - `IdAllocator` saturates at `u64::MAX - 1` to leave room for
//!   the `backdrop_node_id` reserved range (see overlay_pass).

use std::collections::HashMap;

use crate::core::{Element, ElementKey, ElementKind, NodeId, Style};

use super::{DirtyFlags, IdAllocator, UiNode, UiTree};

#[inline]
fn spec_signature(spec: &crate::widgets::WidgetSpec) -> u64 {
    crate::widgets::spec::spec_signature(spec)
}

#[derive(Default)]
pub struct Reconciler {
    next_id: u64,
    keyed_ids: HashMap<ElementKey, NodeId>,
    keyed_fingerprints: HashMap<ElementKey, NodeFingerprint>,
}

#[derive(Clone, Debug, PartialEq)]
struct NodeFingerprint {
    kind: ElementKind,
    style: Style,
    text: Option<String>,
    child_keys: Vec<Option<ElementKey>>,
}

impl Reconciler {
    pub fn reconcile(&mut self, root: Element) -> UiTree {
        let mut allocator = IdAllocator {
            next_id: &mut self.next_id,
            keyed_ids: &mut self.keyed_ids,
        };
        let tree = UiTree::from_element_with_ids(root, &mut allocator);
        self.record_fingerprints(&tree);
        tree
    }

    pub fn reconcile_with_dirty(&mut self, root: Element) -> ReconcileOutput {
        let mut allocator = IdAllocator {
            next_id: &mut self.next_id,
            keyed_ids: &mut self.keyed_ids,
        };
        let tree = UiTree::from_element_with_ids(root, &mut allocator);
        let mut dirty_by_key = Vec::new();

        for node in tree.nodes() {
            let Some(key) = node.key.as_ref() else {
                continue;
            };
            if let Some(previous) = self.keyed_fingerprints.get(key) {
                let current = fingerprint_for_node(&tree, node);
                let mut dirty = DirtyFlags::default();

                if previous.kind != current.kind {
                    dirty.insert(DirtyFlags::LAYOUT);
                    dirty.insert(DirtyFlags::PAINT);
                    dirty.insert(DirtyFlags::SEMANTIC);
                    dirty.insert(DirtyFlags::HIT_TEST);
                }
                if previous.style != current.style {
                    dirty.insert(DirtyFlags::STYLE);
                    dirty.insert(DirtyFlags::LAYOUT);
                    dirty.insert(DirtyFlags::PAINT);
                    dirty.insert(DirtyFlags::HIT_TEST);
                }
                if previous.text != current.text {
                    dirty.insert(DirtyFlags::TEXT);
                    dirty.insert(DirtyFlags::LAYOUT);
                    dirty.insert(DirtyFlags::PAINT);
                    dirty.insert(DirtyFlags::SEMANTIC);
                }
                if previous.child_keys != current.child_keys {
                    dirty.insert(DirtyFlags::LAYOUT);
                    dirty.insert(DirtyFlags::PAINT);
                    dirty.insert(DirtyFlags::HIT_TEST);
                    dirty.insert(DirtyFlags::SEMANTIC);
                }

                if !dirty.is_empty() {
                    dirty_by_key.push((key.clone(), dirty));
                }
            }
        }

        self.record_fingerprints(&tree);
        ReconcileOutput { tree, dirty_by_key }
    }

    /// Diff a prior `Element` root against a new `Element` root and
    /// produce the minimal per-node update set: which nodes were
    /// mounted (newly added), unmounted (removed), and patched (kept
    /// in place but possibly dirty), plus per-node `DirtyFlags` for
    /// downstream passes.
    ///
    /// For v1 the diff is *positional*: prior[i] is paired with new[i].
    /// If the pair's `WidgetKind` is the same, it's a PATCH and state
    /// is preserved. If the kinds differ, the old is unmounted and the
    /// new is mounted (state reset). If the lists are different
    /// lengths, excess old are unmounted and excess new are mounted.
    ///
    /// Keyed reorder is a future enhancement; the current `keyed_ids`
    /// allocator ensures keyed nodes *do* preserve their `NodeId`
    /// across rebuilds, so a keyed node at the same position will be
    /// recognized as a PATCH (and stay alive) even when surrounding
    /// unkeyed nodes change.
    ///
    /// The returned `DiffOutput.tree` is the freshly-built tree for
    /// the new root. `mounted` / `unmounted` / `patched` are disjoint
    /// and cover every node in the union of prior + new.
    pub fn diff(&mut self, prior: Element, new: Element) -> DiffOutput {
        // Build the prior tree with a *fresh* allocator (so we can
        // compare ids back without polluting `self.keyed_ids`).
        let mut prior_allocator = IdAllocator::fresh();
        let prior_tree = UiTree::from_element_with_ids(prior, &mut prior_allocator);
        // Build the new tree with the *live* allocator — keyed nodes
        // will get the same `NodeId` as on the previous frame.
        let mut new_allocator = IdAllocator {
            next_id: &mut self.next_id,
            keyed_ids: &mut self.keyed_ids,
        };
        let new_tree = UiTree::from_element_with_ids(new, &mut new_allocator);
        let mut output = DiffOutput {
            tree: new_tree.clone(),
            mounted: Vec::new(),
            unmounted: Vec::new(),
            patched: Vec::new(),
            dirty: HashMap::new(),
        };
        Self::diff_node(
            &prior_tree,
            Some(prior_tree.root()),
            &new_tree,
            new_tree.root(),
            &mut output,
        );
        self.record_fingerprints(&new_tree);
        output
    }

    /// Snapshot of the reconciler's counters; the observability hook
    /// for DIAG-01.
    pub fn stats(&self) -> ReconcileStats {
        ReconcileStats {
            keyed_node_count: self.keyed_ids.len(),
            fingerprint_count: self.keyed_fingerprints.len(),
        }
    }

    fn diff_node(
        prior_tree: &UiTree,
        prior_node: Option<NodeId>,
        new_tree: &UiTree,
        new_node: NodeId,
        output: &mut DiffOutput,
    ) {
        let prior_style = prior_node
            .and_then(|id| prior_tree.get(id))
            .map(|n| n.style.clone());
        let new_style = new_tree.get(new_node).map(|n| n.style.clone());
        let prior_text = prior_node.and_then(|id| prior_tree.get(id)).and_then(text_for_node);
        let new_text = new_tree.get(new_node).and_then(text_for_node);
        let prior_children = prior_node
            .and_then(|id| prior_tree.get(id))
            .map(|n| n.children.clone())
            .unwrap_or_default();
        let new_children = new_tree
            .get(new_node)
            .map(|n| n.children.clone())
            .unwrap_or_default();

        let same_kind = match (prior_node, new_node) {
            (Some(p), n) => {
                let p_node = prior_tree.get(p);
                let n_node = new_tree.get(n);
                match (p_node, n_node) {
                    (Some(a), Some(b)) => kind_signature(a) == kind_signature(b),
                    _ => false,
                }
            }
            (None, _) => false,
        };

        match (prior_node, same_kind) {
            (Some(_p), true) => {
                // PATCH — state preserved
                output.patched.push(new_node);
                let mut dirty = DirtyFlags::default();
                if prior_style != new_style {
                    dirty.insert(DirtyFlags::STYLE);
                    dirty.insert(DirtyFlags::LAYOUT);
                    dirty.insert(DirtyFlags::PAINT);
                }
                if prior_text != new_text {
                    dirty.insert(DirtyFlags::TEXT);
                    dirty.insert(DirtyFlags::LAYOUT);
                    dirty.insert(DirtyFlags::PAINT);
                }
                if prior_children.len() != new_children.len() {
                    dirty.insert(DirtyFlags::STRUCTURE);
                    dirty.insert(DirtyFlags::LAYOUT);
                    dirty.insert(DirtyFlags::PAINT);
                    dirty.insert(DirtyFlags::HIT_TEST);
                }
                if !dirty.is_empty() {
                    output.dirty.insert(new_node, dirty);
                }
            }
            (Some(p), false) => {
                // Kinds differ — unmount old, mount new
                output.unmounted.push(p);
                output.mounted.push(new_node);
                output.dirty.insert(
                    new_node,
                    DirtyFlags::LAYOUT
                        | DirtyFlags::STRUCTURE
                        | DirtyFlags::PAINT
                        | DirtyFlags::HIT_TEST,
                );
            }
            (None, _) => {
                // New only — mount
                output.mounted.push(new_node);
                output.dirty.insert(
                    new_node,
                    DirtyFlags::LAYOUT
                        | DirtyFlags::STRUCTURE
                        | DirtyFlags::PAINT
                        | DirtyFlags::HIT_TEST,
                );
            }
        }

        // Recurse positionally
        let common = prior_children.len().min(new_children.len());
        for i in 0..common {
            Self::diff_node(
                prior_tree,
                Some(prior_children[i]),
                new_tree,
                new_children[i],
                output,
            );
        }
        for &old_child in &prior_children[common..] {
            output.unmounted.push(old_child);
            if let Some(parent) = new_node_optional(new_node) {
                output.dirty.insert(
                    parent,
                    DirtyFlags::LAYOUT
                        | DirtyFlags::STRUCTURE
                        | DirtyFlags::PAINT
                        | DirtyFlags::HIT_TEST,
                );
            }
        }
        for &new_child in &new_children[common..] {
            output.mounted.push(new_child);
            output.dirty.insert(
                new_node,
                DirtyFlags::LAYOUT
                    | DirtyFlags::STRUCTURE
                    | DirtyFlags::PAINT
                    | DirtyFlags::HIT_TEST,
            );
        }
    }

    fn record_fingerprints(&mut self, tree: &UiTree) {
        self.keyed_fingerprints.clear();
        for node in tree.nodes() {
            if let Some(key) = node.key.as_ref() {
                self.keyed_fingerprints
                    .insert(key.clone(), fingerprint_for_node(tree, node));
            }
        }
    }
}

/// Helper used by `diff_node` to keep the "no-op" return type tidy;
/// returns `Some(node)` so the caller can use it as a map key.
#[inline]
fn new_node_optional(node: NodeId) -> Option<NodeId> {
    Some(node)
}

fn text_for_node(node: &UiNode) -> Option<String> {
    if let Some(label) = crate::widgets::spec_label(&node.widget_spec.clone().unwrap_or(
        crate::widgets::WidgetSpec::Divider,
    )) {
        return Some(label);
    }
    match &node.kind {
        ElementKind::Text(spec) => Some(spec.text.clone()),
        _ => None,
    }
}

/// Compute a kind-level signature for an `ElementKind` / widget spec
/// pair. Two nodes with the same signature can be patched in place;
/// different signatures force an unmount + mount.
///
/// For nodes with a `WidgetSpec`, this delegates to `spec_signature`,
/// which encodes the actual `WidgetKind`. For nodes that *don't* have
/// a spec (e.g. plain `Row` containers, or `Text` leaves), we hash
/// the `ElementKind` discriminant so a `Row` and a `Column` are
/// recognized as different kinds (they would render and lay out
/// differently).
fn kind_signature(node: &UiNode) -> u64 {
    use std::hash::{Hash, Hasher};
    if let Some(spec) = &node.widget_spec {
        spec_signature(spec)
    } else {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        std::mem::discriminant(&node.kind).hash(&mut h);
        if let ElementKind::Text(text) = &node.kind {
            text.text.hash(&mut h);
        }
        h.finish()
    }
}

fn fingerprint_for_node(tree: &UiTree, node: &UiNode) -> NodeFingerprint {
    let text = text_for_node(node);
    let child_keys = node
        .children
        .iter()
        .map(|child_id| tree.get(*child_id).and_then(|child| child.key.clone()))
        .collect();

    NodeFingerprint {
        kind: node.kind.clone(),
        style: node.style.clone(),
        text,
        child_keys,
    }
}

#[derive(Clone, Debug)]
pub struct ReconcileOutput {
    pub tree: UiTree,
    dirty_by_key: Vec<(ElementKey, DirtyFlags)>,
}

impl ReconcileOutput {
    pub fn dirty_entries(&self) -> &[(ElementKey, DirtyFlags)] {
        &self.dirty_by_key
    }

    pub fn dirty_for_key(&self, key: &str) -> Option<DirtyFlags> {
        self.dirty_by_key
            .iter()
            .find(|(candidate, _)| candidate.as_str() == key)
            .map(|(_, dirty)| *dirty)
    }
}

/// The output of [`Reconciler::diff`] — the new tree plus a per-node
/// classification into `mounted` / `unmounted` / `patched`, with
/// per-node `DirtyFlags` for downstream passes.
#[derive(Clone, Debug)]
pub struct DiffOutput {
    pub tree: UiTree,
    pub mounted: Vec<NodeId>,
    pub unmounted: Vec<NodeId>,
    pub patched: Vec<NodeId>,
    pub dirty: HashMap<NodeId, DirtyFlags>,
}

impl DiffOutput {
    /// Counters for the observability hook (DIAG-01).
    pub fn counts(&self) -> DiffCounts {
        DiffCounts {
            mounted: self.mounted.len(),
            unmounted: self.unmounted.len(),
            patched: self.patched.len(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DiffCounts {
    pub mounted: usize,
    pub unmounted: usize,
    pub patched: usize,
}

/// Snapshot of [`Reconciler`] state for DIAG-01.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ReconcileStats {
    pub keyed_node_count: usize,
    pub fingerprint_count: usize,
}
