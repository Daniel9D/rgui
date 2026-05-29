use std::collections::HashMap;

use crate::{
    Element, ElementKey, ElementKind, EventHandlers, NodeId, Semantic, Style, VariantId, WidgetSpec,
};

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

    pub fn ancestors_inclusive(&self, node: NodeId) -> Vec<NodeId> {
        let mut result = Vec::new();
        let mut current = Some(node);
        while let Some(id) = current {
            result.push(id);
            current = self.get(id).and_then(|node| node.parent);
        }
        result
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
