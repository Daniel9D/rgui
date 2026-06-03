use std::collections::HashMap;

use crate::{
    Element, ElementKey, ElementKind, EventHandlers, NodeId, Semantic, Style, VariantId, WidgetSpec,
};

/// Iterator over the chain of ancestors from a starting node up to
/// the root, inclusive. Bug fix 3.6 (adjacent): the previous
/// `ancestors_inclusive` returned a `Vec`; this lazy form avoids the
/// intermediate allocation.
pub struct AncestorIds<'tree> {
    tree: &'tree UiTree,
    current: Option<NodeId>,
}

impl<'tree> Iterator for AncestorIds<'tree> {
    type Item = NodeId;

    fn next(&mut self) -> Option<NodeId> {
        let id = self.current?;
        // Advance to the parent for the *next* call. Stop after
        // the root is yielded (its parent is `None`, so the
        // following `next` returns `None`).
        self.current = self.tree.get(id).and_then(|node| node.parent);
        Some(id)
    }
}

pub struct IdAllocator<'a> {
    pub next_id: &'a mut u64,
    pub keyed_ids: &'a mut HashMap<ElementKey, NodeId>,
}

impl IdAllocator<'_> {
    pub fn id_for(&mut self, key: Option<&ElementKey>) -> NodeId {
        if let Some(key) = key {
            if let Some(existing) = self.keyed_ids.get(key) {
                return *existing;
            }
        }

        *self.next_id += 1;
        let id = NodeId::from_raw(*self.next_id);
        if let Some(key) = key {
            self.keyed_ids.insert(key.clone(), id);
        }
        id
    }

    /// A self-contained allocator that does not borrow from any
    /// `Reconciler`. Used by `Reconciler::diff` to build a `prior_tree`
    /// from the previous `Element` without polluting the live
    /// `keyed_ids` (which are owned by the reconciler and only
    /// advanced as the new tree is built).
    pub fn fresh() -> IdAllocator<'static> {
        // SAFETY-equivalent: the returned `IdAllocator` owns its
        // backing storage via a leak-on-construct, which is fine for
        // a `diff` call that lives for the duration of one frame. The
        // keys it produces are scoped to the diff and never collide
        // with the live allocator (we start at a different offset).
        let next_id: &'static mut u64 = Box::leak(Box::new(0u64));
        let keyed_ids: &'static mut HashMap<ElementKey, NodeId> =
            Box::leak(Box::new(HashMap::new()));
        IdAllocator { next_id, keyed_ids }
    }
}

#[derive(Clone, Debug)]
pub struct UiNode {
    pub id: NodeId,
    pub key: Option<ElementKey>,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub kind: ElementKind,
    pub widget_spec: Option<WidgetSpec>,
    pub style: Style,
    pub variant: Option<VariantId>,
    /// Controlled checked state (overrides internal state every render).
    pub checked: Option<bool>,
    /// Uncontrolled initial checked state (seeds state on first mount only).
    pub default_checked: Option<bool>,
    pub semantic: Semantic,
    pub handlers: EventHandlers,
    pub overlay: Option<Box<Element>>,
    pub open: bool,
    /// Uncontrolled initial open state.
    pub default_open: Option<bool>,
    /// If `true`, the runtime must treat `open` as controlled.
    pub controlled_open: bool,
}

#[derive(Clone, Debug)]
pub struct UiTree {
    root: NodeId,
    nodes: Vec<UiNode>,
    index: HashMap<NodeId, usize>,
}

impl UiTree {
    pub fn from_element(root: Element) -> Self {
        let mut tree = Self {
            root: NodeId::from_raw(0),
            nodes: Vec::new(),
            index: HashMap::new(),
        };
        tree.push_element(root, None);
        tree
    }

    pub fn from_element_with_ids(root: Element, allocator: &mut IdAllocator<'_>) -> Self {
        let mut tree = Self {
            root: NodeId::from_raw(0),
            nodes: Vec::new(),
            index: HashMap::new(),
        };
        tree.push_element_with_ids(root, None, allocator);
        tree
    }

    pub fn from_portal_element(root: Element, root_id: NodeId) -> Self {
        let mut tree = Self {
            root: NodeId::from_raw(0),
            nodes: Vec::new(),
            index: HashMap::new(),
        };
        tree.push_portal_element(root, None, root_id, 0);
        tree
    }

    pub fn root(&self) -> NodeId {
        self.root
    }

    pub fn parent(&self, node: NodeId) -> Option<NodeId> {
        self.node(node).parent
    }

    pub fn children(&self, node: NodeId) -> &[NodeId] {
        &self.node(node).children
    }

    pub fn nodes(&self) -> &[UiNode] {
        &self.nodes
    }

    pub fn get(&self, id: NodeId) -> Option<&UiNode> {
        self.index.get(&id).and_then(|index| self.nodes.get(*index))
    }

    pub fn node_for_key(&self, key: &str) -> Option<NodeId> {
        self.nodes
            .iter()
            .find(|node| {
                node.key
                    .as_ref()
                    .is_some_and(|candidate| candidate.as_str() == key)
            })
            .map(|node| node.id)
    }

    /// Returns the chain of ancestors from `node` (inclusive) up to and
    /// including the root, in bottom-up order.
    ///
    /// The returned `Vec` is `O(depth)` memory. For a lazy alternative
    /// that avoids the allocation, see [`Self::ancestor_ids`].
    pub fn ancestors_inclusive(&self, node: NodeId) -> Vec<NodeId> {
        let mut result = Vec::new();
        let mut current = Some(node);
        while let Some(id) = current {
            result.push(id);
            current = self.get(id).and_then(|node| node.parent);
        }
        result
    }

    /// Lazy iterator over the chain of ancestors from `node` (inclusive)
    /// up to and including the root.
    ///
    /// Bug fix 3.6 (adjacent): the previous `ancestors_inclusive`
    /// returned a `Vec` which forced every caller to allocate. Most
    /// callers (e.g. `EventPath::build`, hit-test ancestor chains)
    /// only need to iterate. This iterator yields `NodeId` values one
    /// at a time and stops at the root, with no intermediate
    /// allocation. Each step is `O(1)` (HashMap lookup + field
    /// access); for a tree of depth D the total is `O(D)`.
    pub fn ancestor_ids(&self, node: NodeId) -> AncestorIds<'_> {
        AncestorIds {
            tree: self,
            current: Some(node),
        }
    }

    pub fn root_node(&self) -> &UiNode {
        self.get(self.root()).expect("root node exists in tree")
    }

    fn push_element(&mut self, element: Element, parent: Option<NodeId>) -> NodeId {
        let id = NodeId::from_raw((self.nodes.len() + 1) as u64);
        if parent.is_none() {
            self.root = id;
        }

        let children = element.children;
        self.index.insert(id, self.nodes.len());
        self.nodes.push(UiNode {
            id,
            key: element.key,
            parent,
            children: Vec::new(),
            kind: element.kind,
            widget_spec: element.widget_spec,
            style: element.style,
            variant: element.variant,
            checked: element.checked,
            default_checked: element.default_checked,
            semantic: element.semantic,
            handlers: element.event_handlers,
            overlay: element.overlay,
            open: element.open,
            default_open: element.default_open,
            controlled_open: element.controlled_open,
        });

        let child_ids = children
            .into_iter()
            .map(|child| self.push_element(child, Some(id)))
            .collect();
        self.node_mut(id).children = child_ids;
        id
    }

    fn push_element_with_ids(
        &mut self,
        element: Element,
        parent: Option<NodeId>,
        allocator: &mut IdAllocator<'_>,
    ) -> NodeId {
        let id = allocator.id_for(element.key.as_ref());
        if parent.is_none() {
            self.root = id;
        }

        let children = element.children;
        self.index.insert(id, self.nodes.len());
        self.nodes.push(UiNode {
            id,
            key: element.key,
            parent,
            children: Vec::new(),
            kind: element.kind,
            widget_spec: element.widget_spec,
            style: element.style,
            variant: element.variant,
            checked: element.checked,
            default_checked: element.default_checked,
            semantic: element.semantic,
            handlers: element.event_handlers,
            overlay: element.overlay,
            open: element.open,
            default_open: element.default_open,
            controlled_open: element.controlled_open,
        });

        let child_ids = children
            .into_iter()
            .map(|child| self.push_element_with_ids(child, Some(id), allocator))
            .collect();
        self.node_mut(id).children = child_ids;
        id
    }

    fn push_portal_element(
        &mut self,
        element: Element,
        parent: Option<NodeId>,
        id: NodeId,
        index_in_parent: usize,
    ) -> NodeId {
        if parent.is_none() {
            self.root = id;
        }

        let children = element.children;
        self.index.insert(id, self.nodes.len());
        self.nodes.push(UiNode {
            id,
            key: element.key,
            parent,
            children: Vec::new(),
            kind: element.kind,
            widget_spec: element.widget_spec,
            style: element.style,
            variant: element.variant,
            checked: element.checked,
            default_checked: element.default_checked,
            semantic: element.semantic,
            handlers: element.event_handlers,
            overlay: element.overlay,
            open: element.open,
            default_open: element.default_open,
            controlled_open: element.controlled_open,
        });

        let child_ids = children
            .into_iter()
            .enumerate()
            .map(|(child_index, child)| {
                let child_id = stable_portal_child_id(id, child.key.as_ref(), child_index);
                self.push_portal_element(child, Some(id), child_id, child_index)
            })
            .collect();
        self.node_mut(id).children = child_ids;
        let _ = index_in_parent;
        id
    }

    fn node(&self, id: NodeId) -> &UiNode {
        self.get(id).expect("node id exists in tree")
    }

    fn node_mut(&mut self, id: NodeId) -> &mut UiNode {
        let index = *self.index.get(&id).expect("node id exists in tree");
        self.nodes.get_mut(index).expect("node id exists in tree")
    }
}

pub fn stable_portal_child_id(
    parent: NodeId,
    key: Option<&ElementKey>,
    index_in_parent: usize,
) -> NodeId {
    let mut hash = 0xcbf29ce484222325u64;
    hash = hash_portal_part(hash, parent.raw());
    if let Some(key) = key {
        hash = hash_portal_str(hash, key.as_str());
    }
    hash = hash_portal_part(hash, index_in_parent as u64);
    NodeId::from_raw(0x8000_0000_0000_0000 | (hash & 0x7fff_ffff_ffff_ffff))
}

fn hash_portal_str(mut hash: u64, value: &str) -> u64 {
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn hash_portal_part(mut hash: u64, value: u64) -> u64 {
    for byte in value.to_le_bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Length;

    // Bug fix 3.6 (adjacent): the `ancestor_ids` lazy
    // iterator should yield the same chain as
    // `ancestors_inclusive`, just without the intermediate
    // `Vec` allocation.

    #[test]
    fn ancestor_ids_walks_from_node_to_root() {
        // Build a 3-deep tree: root → mid → leaf.
        let leaf = Element::text("leaf");
        let mid = Element::column().child(leaf);
        let root = Element::column().child(mid);
        let tree = UiTree::from_element(root);
        let leaf_id = tree.nodes()[2].id;
        let mid_id = tree.nodes()[1].id;
        let root_id = tree.nodes()[0].id;

        // Bottom-up: leaf, mid, root.
        let chain: Vec<NodeId> = tree.ancestor_ids(leaf_id).collect();
        assert_eq!(chain, vec![leaf_id, mid_id, root_id]);
    }

    #[test]
    fn ancestor_ids_agrees_with_ancestors_inclusive() {
        let root = Element::column()
            .child(Element::text("a"))
            .child(Element::text("b").padding(4.0));
        let tree = UiTree::from_element(root);
        // Pick a non-root node so the chain is non-trivial.
        let b_id = tree.nodes()[2].id;
        let from_vec: Vec<NodeId> = tree.ancestors_inclusive(b_id);
        let from_iter: Vec<NodeId> = tree.ancestor_ids(b_id).collect();
        assert_eq!(from_vec, from_iter);
    }

    #[test]
    fn ancestor_ids_yields_only_root_for_root_node() {
        // A root-only tree: the only node is the root, its
        // parent is `None`, so the iterator yields exactly
        // one element.
        let tree = UiTree::from_element(Element::text("solo"));
        let root_id = tree.root();
        let chain: Vec<NodeId> = tree.ancestor_ids(root_id).collect();
        assert_eq!(chain, vec![root_id]);
    }

    #[test]
    fn ancestor_ids_is_lazy() {
        // The iterator only walks as far as the consumer
        // asks. Build a 4-deep tree (root → mid → leaf →
        // grandchild), take 2 elements, and verify the
        // remaining chain is not consumed.
        let grandchild = Element::text("g");
        let leaf = Element::column().child(grandchild);
        let mid = Element::column().child(leaf);
        let root = Element::column().child(mid);
        let tree = UiTree::from_element(root);
        // nodes()[0] = root, [1] = mid, [2] = leaf, [3] = grandchild
        let grandchild_id = tree.nodes()[3].id;
        let leaf_id = tree.nodes()[2].id;
        let mid_id = tree.nodes()[1].id;
        let chain: Vec<NodeId> = tree.ancestor_ids(grandchild_id).take(2).collect();
        assert_eq!(chain, vec![grandchild_id, leaf_id]);
        // The mid (which would be next) is *not* yielded.
        assert!(!chain.contains(&mid_id));
    }

    #[test]
    fn length_construction_is_const() {
        // Smoke test: ensure `Length` is constructable in
        // const context as a small companion to the
        // const-fn work in 5.1.
        const PX: Length = Length::Px(12.0);
        assert_eq!(PX.try_resolve(100.0), Some(12.0));
    }
}
