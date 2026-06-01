use std::collections::HashMap;

use crate::core::{NodeId, Point, Rect, Vec2};

#[derive(Clone, Debug, PartialEq)]
pub struct DragState {
    pub source_key: Option<String>,
    pub source_node: Option<NodeId>,
    pub payload: Option<String>,
    pub origin: Option<Point>,
    pub started: bool,
}

impl Default for DragState {
    fn default() -> Self {
        Self {
            source_key: None,
            source_node: None,
            payload: None,
            origin: None,
            started: false,
        }
    }
}

impl DragState {
    pub fn is_active(&self) -> bool {
        self.origin.is_some()
    }

    pub fn clear(&mut self) {
        self.source_key = None;
        self.source_node = None;
        self.payload = None;
        self.origin = None;
        self.started = false;
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PointerCapture {
    pub node: Option<NodeId>,
    pub key: Option<String>,
}

impl PointerCapture {
    pub fn is_active(&self) -> bool {
        self.key.is_some()
    }

    pub fn set(&mut self, key: String, node: Option<NodeId>) {
        self.key = Some(key);
        self.node = node;
    }

    pub fn clear(&mut self) {
        self.key = None;
        self.node = None;
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct OpenOverlay {
    pub key: String,
    pub rect: Rect,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScrollState {
    pub offsets: HashMap<String, Vec2>,
    pub bounds: HashMap<String, Vec2>,
    pub rects: HashMap<String, Rect>,
}

impl ScrollState {
    pub fn set_offset(&mut self, key: String, offset: Vec2) {
        self.offsets.insert(key, offset);
    }

    pub fn offset(&self, key: &str) -> Option<Vec2> {
        self.offsets.get(key).copied()
    }

    pub fn clear(&mut self) {
        self.offsets.clear();
        self.bounds.clear();
        self.rects.clear();
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BoolState {
    by_key: HashMap<String, bool>,
    by_node: HashMap<NodeId, bool>,
}

impl BoolState {
    pub fn get_by_key(&self, key: &str) -> Option<bool> {
        self.by_key.get(key).copied()
    }

    pub fn get_by_node(&self, node: NodeId) -> Option<bool> {
        self.by_node.get(&node).copied()
    }

    pub fn set(&mut self, key: String, node: NodeId, value: bool) {
        self.by_key.insert(key, value);
        self.by_node.insert(node, value);
    }

    pub fn get_or_init(&mut self, key: &str, node: NodeId, seed: bool) -> bool {
        if let Some(value) = self.by_node.get(&node).copied() {
            return value;
        }
        if let Some(value) = self.by_key.get(key).copied() {
            self.by_node.insert(node, value);
            return value;
        }
        self.by_key.insert(key.to_string(), seed);
        self.by_node.insert(node, seed);
        seed
    }

    pub fn toggle(&mut self, key: &str, node: NodeId) -> bool {
        let current = self
            .by_node
            .get(&node)
            .copied()
            .or_else(|| self.by_key.get(key).copied())
            .unwrap_or(false);
        let next = !current;
        self.by_node.insert(node, next);
        self.by_key.insert(key.to_string(), next);
        next
    }
}
