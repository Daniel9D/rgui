use std::collections::HashMap;

use crate::core::{Element, ElementKey, ElementKind, NodeId, Style};

use super::{DirtyFlags, IdAllocator, UiNode, UiTree};

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

fn fingerprint_for_node(tree: &UiTree, node: &UiNode) -> NodeFingerprint {
    let text = match &node.kind {
        ElementKind::Text(spec) => Some(spec.text.clone()),
        _ => None,
    };
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
