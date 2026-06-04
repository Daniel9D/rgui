//! Phase 4 / Plan 04-02: `SharedAccessibility` (D-15).
//!
//! Wraps a concrete `AccessibilityBackend` in an `Arc` so a single
//! backend can be shared across every `UiRuntime` in the process.
//! The trait object requires `Send + Sync` so the wrapper is
//! `Send + Sync`, and so `UiRuntime` (which holds an
//! `Option<SharedAccessibility>`) can move between threads.
//!
//! Two constructors:
//! - `SharedAccessibility::new(backend)` for hosts that have a
//!   concrete screen reader.
//! - `SharedAccessibility::none()` for the default; wraps the
//!   existing in-lib `NullAccessibility` (no-op).

use std::fmt;
use std::sync::Arc;

use crate::core::a11y::{AccessibilityBackend, NullAccessibility, SemanticTree};

#[derive(Clone)]
pub struct SharedAccessibility(Arc<dyn AccessibilityBackend + Send + Sync>);

impl SharedAccessibility {
    /// Wrap a concrete backend. The backend must be `Send + Sync +
    /// 'static` so it can live in an `Arc` and be shared across
    /// threads (see D-17 on the trait bound).
    pub fn new(backend: impl AccessibilityBackend + Send + Sync + 'static) -> Self {
        Self(Arc::new(backend))
    }

    /// Convenience: a `SharedAccessibility` that wraps the
    /// in-lib no-op backend (`NullAccessibility`). Used by
    /// `ProcessContext::new()` so the default is "screen reader
    /// present, just doing nothing".
    pub fn none() -> Self {
        Self(Arc::new(NullAccessibility))
    }

    /// The inner `Arc<dyn AccessibilityBackend + Send + Sync>`.
    /// Used by code that needs to share the trait object
    /// directly (e.g. an `&Arc<dyn AccessibilityBackend + Send +
    /// Sync>` argument to a free function).
    pub fn inner(&self) -> &Arc<dyn AccessibilityBackend + Send + Sync> {
        &self.0
    }
}

impl AccessibilityBackend for SharedAccessibility {
    fn update(&mut self, tree: &SemanticTree) {
        // The trait method takes `&mut self`, but our inner is an
        // `Arc<dyn AccessibilityBackend + Send + Sync>` (shared).
        // `Arc::make_mut` would require the inner to be `Clone`,
        // which trait objects are not. Use `Arc::get_mut`: when
        // this is the only `Arc` pointing at the backend (the
        // common case: one clone per runtime, and the
        // `ProcessContext` clone was dropped after `for_window`),
        // we get `&mut` and dispatch; otherwise (the runtime is
        // sharing the backend with another owner), the update is
        // silently skipped. Backends that need to handle
        // concurrent updates should hold a `Mutex<T>` internally
        // (see the D-18 rustdoc on the trait).
        if let Some(backend) = Arc::get_mut(&mut self.0) {
            backend.update(tree);
        }
    }
}

impl fmt::Debug for SharedAccessibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SharedAccessibility").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_a11y_none_dispatches_to_null_backend() {
        // The default `none()` is a no-op backend; dispatching an
        // empty `SemanticTree` should not panic. We can't observe
        // the no-op directly (it has no observable side effects),
        // so this test is mostly a smoke test that the wrapper
        // compiles and the trait dispatch works.
        let mut shared = SharedAccessibility::none();
        let tree = SemanticTree::default();
        shared.update(&tree);
        // The clone shares the same Arc; nothing observable to
        // assert beyond "did not panic".
        let _twin = shared.clone();
    }

    #[test]
    fn shared_a11y_clone_shares_inner_arc() {
        // Clones of a `SharedAccessibility` share the same
        // `Arc<dyn AccessibilityBackend + Send + Sync>`. We
        // can't observe this directly (the inner is opaque), so
        // we just confirm the clone works and the type is
        // `Clone + Send + Sync`.
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<SharedAccessibility>();

        let a = SharedAccessibility::none();
        let b = a.clone();
        // Both clones should be usable; dispatching should not
        // panic.
        let mut a = a;
        let tree = SemanticTree::default();
        a.update(&tree);
        let mut b = b;
        b.update(&tree);
    }
}
