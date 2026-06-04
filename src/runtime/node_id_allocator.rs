//! Phase 4 / Plan 04-02: `NodeIdAllocator` (D-14).
//!
//! Process-global monotonic `NodeId` counter shared by every
//! `UiRuntime` in the process. Hosts construct one (implicit, via
//! `ProcessContext::new()`) and pass `&ProcessContext` to every
//! `UiRuntime::for_window(id, &ctx)`. The counter is `Arc`-shared
//! across runtimes so the `(window_id, node_id)` tuple is unique
//! process-wide.
//!
//! The counter is implemented as `Arc<AtomicU64>` rather than
//! `Arc<Mutex<u64>>`: the only operation is monotonic
//! `fetch_add`, which is lock-free. `Ordering::Relaxed` is
//! sufficient because the only guarantee we need is uniqueness of
//! issued ids; no happens-before relationship needs to cross the
//! counter.

use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct NodeIdAllocator(Arc<AtomicU64>);

impl NodeIdAllocator {
    /// Construct a fresh allocator whose counter starts at 0.
    pub fn new() -> Self {
        Self(Arc::new(AtomicU64::new(0)))
    }

    /// Construct an allocator that resumes from `start`. Useful for
    /// tests that need a deterministic starting value.
    pub fn from_counter(start: u64) -> Self {
        Self(Arc::new(AtomicU64::new(start)))
    }

    /// Issue a fresh id and return it. The first call returns
    /// `0`, the second `1`, and so on. The counter is
    /// monotonically incremented; it never resets.
    pub fn fresh(&self) -> u64 {
        self.0.fetch_add(1, Ordering::Relaxed)
    }

    /// Read the next id that *would* be issued, without
    /// consuming it. Cheap; uses `Ordering::Relaxed` (the read is
    /// only an observability hook for tests and the
    /// `ProcessContext` debug print).
    pub fn current(&self) -> u64 {
        self.0.load(Ordering::Relaxed)
    }
}

impl fmt::Debug for NodeIdAllocator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NodeIdAllocator")
            .field("next", &self.current())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_id_allocator_fresh_is_monotonic() {
        let alloc = NodeIdAllocator::new();
        assert_eq!(alloc.fresh(), 0);
        assert_eq!(alloc.fresh(), 1);
        assert_eq!(alloc.fresh(), 2);
        assert_eq!(alloc.fresh(), 3);
        assert_eq!(alloc.fresh(), 4);
        assert_eq!(alloc.current(), 5);
    }

    #[test]
    fn node_id_allocator_clones_share_state() {
        let alloc = NodeIdAllocator::new();
        let twin = alloc.clone();
        assert_eq!(alloc.fresh(), 0);
        // The clone sees the same counter; its first issue is
        // one greater than the original's last issue.
        assert_eq!(twin.fresh(), 1);
        assert_eq!(alloc.fresh(), 2);
        assert_eq!(twin.current(), 3);
    }

    #[test]
    fn node_id_allocator_does_not_overlap() {
        // Two separately-constructed allocators must issue ids
        // from independent ranges. This is the per-runtime default
        // case (each `UiRuntime::default()` builds a fresh
        // `ProcessContext`); the WIN-04 test in 04-03 will exercise
        // the shared-counter case via `&ctx` reuse.
        let a = NodeIdAllocator::new();
        let b = NodeIdAllocator::new();
        assert_eq!(a.fresh(), 0);
        assert_eq!(b.fresh(), 0, "independent allocators start at 0");
        assert_eq!(a.fresh(), 1);
        assert_eq!(b.fresh(), 1);
        assert_eq!(a.current(), 2);
        assert_eq!(b.current(), 2);
    }

    #[test]
    fn node_id_allocator_from_counter_starts_at_value() {
        let alloc = NodeIdAllocator::from_counter(100);
        assert_eq!(alloc.fresh(), 100);
        assert_eq!(alloc.fresh(), 101);
        assert_eq!(alloc.current(), 102);
    }
}
