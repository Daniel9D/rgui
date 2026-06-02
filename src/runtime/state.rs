//! Runtime state holders — booleans, drag, scroll offsets, pointer capture.
//!
//! These live in the runtime (not the tree) so they survive across re-renders
//! of the same `Element` (which is what makes a checkbox stay checked when
//! you only change its label on re-render).

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
    /// `true` once a drag has been started. Use this — not `origin.is_some()`
    /// — as the single source of truth for "is a drag in progress". A drag
    /// that has been started but whose `clear()` hasn't been called is still
    /// active; checking only `origin` is fragile (bug fix 1.11).
    pub const fn is_active(&self) -> bool {
        self.started
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
    pub const fn is_active(&self) -> bool {
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
    /// Per-key scroll offset (logical pixels).
    pub offsets: HashMap<String, Vec2>,
    /// Per-key scrollable bounds (logical pixels).
    pub bounds: HashMap<String, Vec2>,
    /// Per-key viewport rect (logical pixels).
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

/// Per-key / per-node boolean state. The single source of truth is
/// `by_node`; `by_key` is a transient hint seeded on first mount and
/// consulted only when a node id is unknown. Mutating one without the
/// other is forbidden — use the provided methods.
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

    /// Returns the new value.
    pub fn set(&mut self, key: String, node: NodeId, value: bool) -> bool {
        self.by_key.insert(key, value);
        self.by_node.insert(node, value);
        value
    }

    /// Idempotent: read the existing value if any, otherwise seed `seed` and
    /// return it. Always back-fills `by_node` from `by_key` so the two maps
    /// stay in sync after a remount.
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

    /// Toggle the boolean for this `key`/`node` pair. Returns the new value.
    /// Use this from the `Toggle` command handler.
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

    /// Toggle when you only have a key (e.g. from a `Toggle { key }` command
    /// before the node is mounted). Looks up the node by key in a separate
    /// `key_to_node` map provided by the caller.
    pub fn toggle_by_key<F>(&mut self, key: &str, key_to_node: F) -> bool
    where
        F: FnOnce() -> Option<NodeId>,
    {
        if let Some(node) = key_to_node() {
            return self.toggle(key, node);
        }
        // No node — fall back to a key-only toggle. Keeps by_key in sync
        // even when the node id is unknown, which is the most common case
        // for "controlled" checkboxes.
        let current = self.by_key.get(key).copied().unwrap_or(false);
        let next = !current;
        self.by_key.insert(key.to_string(), next);
        next
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drag_is_active_uses_started() {
        let mut d = DragState::default();
        assert!(!d.is_active());
        d.started = true;
        assert!(d.is_active());
        d.clear();
        assert!(!d.is_active());
    }

    #[test]
    fn bool_state_single_source_of_truth() {
        let mut s = BoolState::default();
        let n = NodeId::from_raw(42);
        s.set("cb".into(), n, true);
        assert_eq!(s.get_by_key("cb"), Some(true));
        assert_eq!(s.get_by_node(n), Some(true));
        // Toggle flips both.
        s.toggle("cb", n);
        assert_eq!(s.get_by_key("cb"), Some(false));
        assert_eq!(s.get_by_node(n), Some(false));
    }

    #[test]
    fn bool_state_get_or_init_seeds() {
        let mut s = BoolState::default();
        let n = NodeId::from_raw(1);
        assert_eq!(s.get_or_init("k", n, true), true);
        // Second call is idempotent.
        assert_eq!(s.get_or_init("k", n, false), true);
    }

    #[test]
    fn bool_state_toggle_by_key_works() {
        let mut s = BoolState::default();
        let n = NodeId::from_raw(2);
        // First: key-only path (no node lookup).
        let v = s.toggle_by_key("k", || None);
        assert_eq!(v, true);
        // Second: with node lookup.
        let v = s.toggle_by_key("k", || Some(n));
        assert_eq!(v, false);
        assert_eq!(s.get_by_node(n), Some(false));
    }
}
