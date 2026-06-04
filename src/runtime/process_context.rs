//! Phase 4 / Plan 04-02: full `ProcessContext` (D-13).
//!
//! A `ProcessContext` bundles the per-process state that every
//! `UiRuntime` in the process shares:
//!
//! - `node_ids: NodeIdAllocator` — a process-global monotonic
//!   `NodeId` counter (D-14). `Arc<AtomicU64>` under the hood, so
//!   the counter is shared by every runtime that holds a clone
//!   of this `ProcessContext`.
//! - `a11y: Option<SharedAccessibility>` — an optional shared
//!   `AccessibilityBackend` (D-15). Hosts with a screen reader
//!   construct one and pass it to every runtime; hosts without
//!   use `ProcessContext::new()` (which sets a `none()` noop) or
//!   `ProcessContext::new_without_a11y()` (which sets `None`).
//!
//! The `ProcessContext` is intended to be constructed once per
//! process (in `main()`) and passed by `&` to every
//! `UiRuntime::for_window(id, &ctx)` call. The lib is responsible
//! for the context being cheaply cloneable internally — every
//! field is `Arc`-wrapped.

use std::fmt;

use crate::core::{AccessibilityBackend, SharedAccessibility};

use super::NodeIdAllocator;

#[derive(Clone)]
pub struct ProcessContext {
    node_ids: NodeIdAllocator,
    a11y: Option<SharedAccessibility>,
}

impl ProcessContext {
    /// Construct a `ProcessContext` with a fresh
    /// `NodeIdAllocator` (counter starting at 0) and the default
    /// no-op accessibility backend. This is the v1.x default;
    /// hosts that need a real screen reader can call
    /// [`ProcessContext::with_a11y`] instead.
    pub fn new() -> Self {
        Self {
            node_ids: NodeIdAllocator::new(),
            a11y: Some(SharedAccessibility::none()),
        }
    }

    /// Construct a `ProcessContext` with no accessibility backend.
    /// For hosts that explicitly want to skip the a11y path
    /// (e.g. embedded / headless / tests).
    pub fn new_without_a11y() -> Self {
        Self {
            node_ids: NodeIdAllocator::new(),
            a11y: None,
        }
    }

    /// Construct a `ProcessContext` with a custom
    /// `AccessibilityBackend`. The backend is wrapped in a
    /// `SharedAccessibility` (an `Arc<dyn ... + Send + Sync>`)
    /// and shared across every runtime that holds this context.
    pub fn with_a11y(backend: impl AccessibilityBackend + Send + Sync + 'static) -> Self {
        Self {
            node_ids: NodeIdAllocator::new(),
            a11y: Some(SharedAccessibility::new(backend)),
        }
    }

    /// The process-global `NodeId` counter. Used by
    /// `UiRuntime::for_window` to clone the `Arc` into the
    /// runtime.
    pub fn node_ids(&self) -> &NodeIdAllocator {
        &self.node_ids
    }

    /// The shared accessibility backend, if any. `Some` for the
    /// default `new()` (wraps a noop); `None` for
    /// `new_without_a11y()`.
    pub fn a11y(&self) -> Option<&SharedAccessibility> {
        self.a11y.as_ref()
    }
}

impl Default for ProcessContext {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for ProcessContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProcessContext")
            .field("node_ids", &self.node_ids)
            .field("a11y", &self.a11y.as_ref().map(|_| "..."))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_context_shares_node_ids_across_clones() {
        let ctx = ProcessContext::new();
        let ctx2 = ctx.clone();
        // Clone is shallow: the inner NodeIdAllocator is Arc-shared.
        assert_eq!(ctx.node_ids().current(), ctx2.node_ids().current());
        ctx.node_ids().fresh();
        assert_eq!(ctx2.node_ids().current(), 1);
    }

    #[test]
    fn process_context_default_has_noop_a11y() {
        let ctx = ProcessContext::default();
        assert!(ctx.a11y().is_some(), // some(SharedAccessibility::none())
            "default ProcessContext must wrap a noop a11y backend");
    }

    #[test]
    fn process_context_without_a11y_has_none() {
        let ctx = ProcessContext::new_without_a11y();
        assert!(ctx.a11y().is_none());
    }

    #[test]
    fn process_context_with_a11y_wraps_the_backend() {
        // The convenience constructor must produce a context
        // whose a11y is `Some(...)`. We pass the in-lib noop as
        // a stand-in; the real test of a custom backend is in
        // the integration test suite.
        let ctx = ProcessContext::with_a11y(crate::core::NullAccessibility);
        assert!(ctx.a11y().is_some());
    }
}
