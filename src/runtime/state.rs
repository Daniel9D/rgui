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

    /// Phase 1 / Plan 01-03: release the capture if the current key
    /// is in `unmounted_keys`. Returns the captured node id (if any)
    /// so the caller can emit a synthetic `PointerCancel` event to
    /// that node, allowing drag handlers to clean up.
    ///
    /// `PointerCapture` holds at most one active capture (the most
    /// recent), so this is a single check rather than a map walk.
    pub fn release_matching(
        &mut self,
        unmounted_keys: &[String],
    ) -> Option<crate::core::PointerCancel> {
        let key = self.key.as_ref()?;
        if unmounted_keys.iter().any(|k| k == key) {
            let cancel = crate::core::PointerCancel {
                node: self.node?,
                button: None,
            };
            self.clear();
            return Some(cancel);
        }
        None
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

/// Per-node `LayoutBox` cache, populated by the taffy backend after
/// each `LayoutCache::recompute` and read by the paint path via
/// `LayoutCache::get`.
///
/// ## Design
///
/// The cache is the *output* of incremental layout, indexed by
/// `NodeId`. The dirty set is the *input* — nodes that need
/// re-layout. A node is in the dirty set after:
///
/// - a structural change (mount / unmount of a descendant), or
/// - a style change on the node itself, or
/// - the node's ancestor was dirty (mark_dirty propagates up so the
///   paint path can walk from the root down).
///
/// `recompute` is the call into the taffy backend that produces new
/// `LayoutBox` values for the dirty subtree; the results are merged
/// into the cache and the dirty set is cleared.
///
/// The cache is **not** responsible for the taffy tree itself —
/// that's owned by `TaffyLayoutBackend` in `src/layout/taffy.rs`.
/// This is purely a memoization layer for the per-node box output.
#[derive(Clone, Debug, Default)]
pub struct LayoutCache {
    boxes: HashMap<NodeId, crate::core::LayoutBox>,
    dirty: std::collections::HashSet<NodeId>,
}

impl LayoutCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// Mark `node` as needing re-layout. Marks propagate *upward* to
    /// the ancestors so the paint path can walk from the root down.
    /// The cache itself doesn't know the parent chain, so callers
    /// should pass the highest dirty ancestor (typically the root of
    /// the dirty subtree).
    pub fn mark_dirty(&mut self, node: NodeId) {
        self.dirty.insert(node);
    }

    /// Remove entries for nodes that have been unmounted.
    pub fn clear_unmounted(&mut self, unmounted: &[NodeId]) {
        for node in unmounted {
            self.boxes.remove(node);
        }
    }

    /// Clear the dirty set without recomputing. Used after a `recompute`
    /// pass to reset for the next frame.
    pub fn clear_dirty(&mut self) {
        self.dirty.clear();
    }

    /// Borrow the cached `LayoutBox` for `node`, if any.
    pub fn get(&self, node: NodeId) -> Option<&crate::core::LayoutBox> {
        self.boxes.get(&node)
    }

    /// Insert a freshly-computed `LayoutBox`. Used by the taffy
    /// backend's `recompute` adapter.
    pub fn insert(&mut self, layout_box: crate::core::LayoutBox) {
        let node = layout_box.node;
        self.boxes.insert(node, layout_box);
    }

    /// True if `node` is currently in the dirty set.
    pub fn is_dirty(&self, node: NodeId) -> bool {
        self.dirty.contains(&node)
    }

    /// Number of cached `LayoutBox` entries.
    pub fn len(&self) -> usize {
        self.boxes.len()
    }

    /// True if the cache holds no entries.
    pub fn is_empty(&self) -> bool {
        self.boxes.is_empty()
    }

    /// Number of dirty nodes pending a `recompute`.
    pub fn dirty_count(&self) -> usize {
        self.dirty.len()
    }

    /// Snapshot for DIAG-01.
    pub fn stats(&self) -> LayoutCacheStats {
        LayoutCacheStats {
            entries: self.boxes.len(),
            dirty: self.dirty.len(),
        }
    }
}

/// Snapshot of [`LayoutCache`] state for DIAG-01.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LayoutCacheStats {
    pub entries: usize,
    pub dirty: usize,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod pointer_capture_release_tests {
    use super::*;

    #[test]
    fn release_matching_clears_capture_for_matching_key() {
        let mut cap = PointerCapture::default();
        cap.set("btn".to_string(), Some(NodeId::from_raw(7)));
        let cancel = cap.release_matching(&["btn".to_string()]);
        assert!(cancel.is_some(), "should produce a cancel for the matching key");
        let cancel = cancel.unwrap();
        assert_eq!(cancel.node, NodeId::from_raw(7));
        assert!(!cap.is_active(), "capture should be cleared");
    }

    #[test]
    fn release_matching_returns_none_for_non_matching_key() {
        let mut cap = PointerCapture::default();
        cap.set("btn".to_string(), Some(NodeId::from_raw(7)));
        let cancel = cap.release_matching(&["menu".to_string()]);
        assert!(cancel.is_none(), "no cancel should be produced for an unrelated key");
        assert!(cap.is_active(), "capture should still be active");
    }

    #[test]
    fn release_matching_handles_inactive_capture() {
        let mut cap = PointerCapture::default();
        let cancel = cap.release_matching(&["btn".to_string()]);
        assert!(cancel.is_none(), "no cancel should be produced for an inactive capture");
    }

    #[test]
    fn release_matching_works_for_multiple_keys() {
        let mut cap = PointerCapture::default();
        cap.set("menu".to_string(), Some(NodeId::from_raw(2)));
        let cancel = cap.release_matching(&["btn".to_string(), "menu".to_string()]);
        assert!(cancel.is_some(), "should match menu in the list");
        let cancel = cancel.unwrap();
        assert_eq!(cancel.node, NodeId::from_raw(2));
        assert!(!cap.is_active());
    }
}

#[cfg(test)]
mod layout_cache_tests {
    use super::*;
    use crate::core::{NodeId, Rect, Size};

    fn fake_box(node: NodeId, w: f32, h: f32) -> crate::core::LayoutBox {
        crate::core::LayoutBox::new(
            node,
            Rect::new(crate::core::Point::new(0.0, 0.0), Size::new(w, h)),
        )
    }

    #[test]
    fn mark_dirty_adds_to_set() {
        let mut cache = LayoutCache::new();
        let n = NodeId::from_raw(7);
        cache.mark_dirty(n);
        assert!(cache.is_dirty(n));
        assert_eq!(cache.dirty_count(), 1);
    }

    #[test]
    fn clear_dirty_resets_set() {
        let mut cache = LayoutCache::new();
        cache.mark_dirty(NodeId::from_raw(1));
        cache.mark_dirty(NodeId::from_raw(2));
        cache.clear_dirty();
        assert_eq!(cache.dirty_count(), 0);
    }

    #[test]
    fn get_returns_inserted_box() {
        let mut cache = LayoutCache::new();
        let n = NodeId::from_raw(3);
        cache.insert(fake_box(n, 100.0, 50.0));
        let b = cache.get(n).expect("inserted box should be present");
        assert_eq!(b.local_rect.size.width, 100.0);
        assert_eq!(b.local_rect.size.height, 50.0);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn get_returns_none_for_unknown() {
        let cache = LayoutCache::new();
        assert!(cache.get(NodeId::from_raw(999)).is_none());
    }

    #[test]
    fn clear_unmounted_removes_entries() {
        let mut cache = LayoutCache::new();
        let n1 = NodeId::from_raw(1);
        let n2 = NodeId::from_raw(2);
        cache.insert(fake_box(n1, 10.0, 10.0));
        cache.insert(fake_box(n2, 20.0, 20.0));
        cache.clear_unmounted(&[n1]);
        assert!(cache.get(n1).is_none());
        assert!(cache.get(n2).is_some());
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn stats_reports_counts() {
        let mut cache = LayoutCache::new();
        cache.insert(fake_box(NodeId::from_raw(1), 1.0, 1.0));
        cache.insert(fake_box(NodeId::from_raw(2), 2.0, 2.0));
        cache.mark_dirty(NodeId::from_raw(3));
        let stats = cache.stats();
        assert_eq!(stats.entries, 2);
        assert_eq!(stats.dirty, 1);
    }

    #[test]
    fn insert_overwrites_existing_entry() {
        let mut cache = LayoutCache::new();
        let n = NodeId::from_raw(5);
        cache.insert(fake_box(n, 10.0, 10.0));
        cache.insert(fake_box(n, 20.0, 20.0));
        let b = cache.get(n).expect("box should be present");
        assert_eq!(b.local_rect.size.width, 20.0);
        assert_eq!(cache.len(), 1);
    }
}
