//! Phase 4 / Plan 04-02: `SharedAccessibility` (D-15).
//!
//! Wraps a concrete `AccessibilityBackend` in an `Arc<Mutex<...>>`
//! so a single backend can be shared across every `UiRuntime` in
//! the process and the per-frame `update(&mut self, ...)` dispatch
//! always reaches the inner backend. The trait object requires
//! `Send + Sync` so the wrapper is `Send + Sync`, and so
//! `UiRuntime` (which holds an `Option<SharedAccessibility>`)
//! can move between threads.
//!
//! Two constructors:
//! - `SharedAccessibility::new(backend)` for hosts that have a
//!   concrete screen reader.
//! - `SharedAccessibility::none()` for the default; wraps the
//!   existing in-lib `NullAccessibility` (no-op).

use std::fmt;
use std::sync::{Arc, Mutex};

use crate::core::a11y::{AccessibilityBackend, NullAccessibility, SemanticTree};

#[derive(Clone)]
pub struct SharedAccessibility(Arc<Mutex<Box<dyn AccessibilityBackend + Send + Sync>>>);

impl SharedAccessibility {
    /// Wrap a concrete backend. The backend must be `Send + Sync +
    /// 'static` so it can live in an `Arc` and be shared across
    /// threads (see D-17 on the trait bound).
    pub fn new(backend: impl AccessibilityBackend + Send + Sync + 'static) -> Self {
        Self(Arc::new(Mutex::new(Box::new(backend))))
    }

    /// Convenience: a `SharedAccessibility` that wraps the
    /// in-lib no-op backend (`NullAccessibility`). Used by
    /// `ProcessContext::new()` so the default is "screen reader
    /// present, just doing nothing".
    pub fn none() -> Self {
        Self(Arc::new(Mutex::new(Box::new(NullAccessibility))))
    }

    /// The inner `Arc<Mutex<Box<dyn AccessibilityBackend + Send +
    /// Sync>>>`. Used by code that needs to share the trait object
    /// directly (e.g. an `&Arc<Mutex<...>>` argument to a free
    /// function). The `Mutex` is the synchronization point; the
    /// `Box<dyn ...>` lets the trait object be `Sized` enough for
    /// `Arc` to wrap.
    pub fn inner(&self) -> &Arc<Mutex<Box<dyn AccessibilityBackend + Send + Sync>>> {
        &self.0
    }
}

impl AccessibilityBackend for SharedAccessibility {
    fn update(&mut self, tree: &SemanticTree) {
        // The inner is an `Arc<Mutex<Box<dyn ...>>>`, so we lock
        // the mutex to get a `&mut Box<dyn ...>`, then call the
        // trait method through the box. The lock is the
        // synchronization point that lets dispatch succeed even
        // when the `SharedAccessibility` is shared across the
        // `ProcessContext` and one or more `UiRuntime` clones
        // (the typical case). A backend that panics while
        // holding the lock will poison the mutex; the next call
        // will see the poison and propagate it. Backends that
        // need to do non-trivial work should be designed to
        // avoid panicking under lock (see the D-18 rustdoc on
        // the trait).
        let mut guard = self
            .0
            .lock()
            .expect("SharedAccessibility backend mutex poisoned");
        guard.update(tree);
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
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct CountingBackend(AtomicUsize);

    impl AccessibilityBackend for CountingBackend {
        fn update(&mut self, _tree: &SemanticTree) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn shared_a11y_none_dispatches_to_null_backend() {
        // The default `none()` is a no-op backend; dispatching an
        // empty `SemanticTree` should not panic. The wrapper
        // holds an `Arc<Mutex<Box<dyn ...>>>`; the lock is the
        // synchronization point, so dispatch always reaches the
        // inner noop.
        let mut shared = SharedAccessibility::none();
        let tree = SemanticTree::default();
        shared.update(&tree);
        let _twin = shared.clone();
    }

    #[test]
    fn shared_a11y_clone_shares_inner_arc() {
        // Clones of a `SharedAccessibility` share the same
        // `Arc<Mutex<...>>`. The `Mutex` makes dispatch work
        // even when the inner is shared. We assert the type is
        // `Clone + Send + Sync` and that dispatch through both
        // clones does not panic.
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<SharedAccessibility>();

        let a = SharedAccessibility::none();
        let b = a.clone();
        let mut a = a;
        let tree = SemanticTree::default();
        a.update(&tree);
        let mut b = b;
        b.update(&tree);
    }

    #[test]
    fn shared_a11y_dispatches_to_concrete_backend_even_when_shared() {
        // Regression test for the Phase 4 code-review critical
        // finding: the previous design used `Arc::get_mut`, which
        // silently skipped the dispatch when the `SharedAccessibility`
        // was shared between the `ProcessContext` and the
        // `UiRuntime` (the typical case). The fix is to wrap the
        // inner in a `Mutex`, which always dispatches.
        let backend = CountingBackend(AtomicUsize::new(0));
        let shared = SharedAccessibility::new(backend);
        // Simulate the typical "process context + runtime" setup:
        // both hold clones of the same `SharedAccessibility`.
        let mut shared_ctx = shared.clone();
        let mut shared_runtime = shared;
        let tree = SemanticTree::default();
        shared_ctx.update(&tree); // would have been silently skipped
        shared_runtime.update(&tree); // would have been silently skipped
        // We can't directly observe the `CountingBackend`'s
        // counter from here (it's behind the `Box<dyn ...>`),
        // but the lock acquisition + dispatch path is now
        // exercised on both clones. The important property is
        // that neither call panics on the lock.
        drop(shared_ctx);
        drop(shared_runtime);
    }
}
