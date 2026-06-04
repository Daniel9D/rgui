//! Phase 4 / D-15 regression: the runtime's `update()` must call
//! the `SharedAccessibility` wired in via `ProcessContext::with_a11y`.
//!
//! This test is the end-to-end version of the code-review
//! critical finding (REVIEW.md, 04-REVIEW.md, "Critical #1").
//! The unit test in `src/core/shared_a11y.rs` covers the
//! wrapper; this test covers the runtime wiring.
//!
//! Contract:
//! 1. A counting `AccessibilityBackend` is wrapped in
//!    `SharedAccessibility` and bound to a `ProcessContext` via
//!    `with_a11y`.
//! 2. A `UiRuntime` is built from the context.
//! 3. `update()` is called.
//! 4. The shared backend's `update` count advances by at least 1.
//! 5. The runtime's reported `accesskit_update_count` (in the
//!    `PerformanceMetrics`) is > 0.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use rgui::core::{AccessibilityBackend, SemanticTree};
use rgui::runtime::{FrameInput, ProcessContext, UiRuntime, WindowId};
use rgui::widgets::text;
use rgui::Element;
use rgui::SharedAccessibility;

struct CountingBackend {
    count: Arc<AtomicUsize>,
}

impl CountingBackend {
    fn new() -> (Self, Arc<AtomicUsize>) {
        let count = Arc::new(AtomicUsize::new(0));
        (
            Self {
                count: count.clone(),
            },
            count,
        )
    }
}

impl AccessibilityBackend for CountingBackend {
    fn update(&mut self, _tree: &SemanticTree) {
        self.count.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn process_context_with_a11y_dispatches_through_update() {
    let (backend, count) = CountingBackend::new();
    let shared = SharedAccessibility::new(backend);
    let ctx = ProcessContext::with_a11y(shared);

    let mut runtime = UiRuntime::for_window(WindowId::new(1), &ctx);

    let pre = count.load(Ordering::SeqCst);
    let _out = runtime.update(FrameInput {
        root: Element::column().key("root").child(text("hi").key("hi")),
        ..FrameInput::default()
    });
    let post = count.load(Ordering::SeqCst);

    assert!(
        post > pre,
        "expected SharedAccessibility update to advance the counter: pre={pre}, post={post}"
    );
}

#[test]
fn default_process_context_still_dispatches_a_noop_a11y_update() {
    // The default `ProcessContext::new()` ships
    // `SharedAccessibility::none()` (a `NullAccessibility` noop).
    // The runtime's `update()` should still drive the noop
    // backend, advancing the `accesskit_update_count` metric in
    // the frame's `PerformanceMetrics`. This pins the D-15 wiring
    // for the default path (the common case where the host
    // doesn't ship a screen reader).
    let ctx = ProcessContext::new();
    let mut runtime = UiRuntime::for_window(WindowId::new(2), &ctx);

    let out = runtime.update(FrameInput {
        root: Element::column().key("root").child(text("hi").key("hi")),
        ..FrameInput::default()
    });
    let snap = out.debug_snapshot();
    let a11y_count = snap
        .performance
        .accessibility
        .accesskit_update_count;
    assert!(
        a11y_count > 0,
        "expected at least one accessibility update after update(), got {a11y_count}"
    );
}
