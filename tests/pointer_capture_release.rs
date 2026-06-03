//! Integration tests for the `PointerCapture::release_matching` path
//! (Phase 1 / Plan 01-03).
//!
//! These exercise the data structure directly; the end-to-end runtime
//! wiring (reconciler → release_matching → event queue) is covered by
//! the runtime-level test suite.

use rgui::core::{NodeId, PointerButton, PointerCancel};
use rgui::runtime::PointerCapture;

#[test]
fn matching_key_produces_cancel_with_node() {
    let mut cap = PointerCapture::default();
    let node = NodeId::from_raw(42);
    cap.set("btn".to_string(), Some(node));
    let cancel: Option<PointerCancel> = cap.release_matching(&["btn".to_string()]);
    let cancel = cancel.expect("cancel for matching key");
    assert_eq!(cancel.node, node);
    assert_eq!(cancel.button, None);
    assert!(!cap.is_active());
}

#[test]
fn button_propagates_through_release() {
    // Even though `PointerCapture` doesn't currently store a button
    // (the existing struct only tracks key + node), the cancel
    // payload reserves space for it. A future enhancement could
    // thread the button through. This test asserts the current
    // contract: `button` is `None` for now, but the field is
    // accessible.
    let mut cap = PointerCapture::default();
    cap.set("btn".to_string(), Some(NodeId::from_raw(1)));
    let cancel = cap
        .release_matching(&["btn".to_string()])
        .expect("cancel");
    assert_eq!(cancel.button, None);
}

#[test]
fn multiple_unmounted_keys_release_only_matching() {
    // `PointerCapture` holds at most one active capture, so the
    // unmounted list containing multiple keys still produces at most
    // one cancel.
    let mut cap = PointerCapture::default();
    cap.set("menu".to_string(), Some(NodeId::from_raw(2)));
    let cancel = cap.release_matching(&["btn".to_string(), "menu".to_string()]);
    assert!(cancel.is_some());
    assert!(!cap.is_active());
}

#[test]
fn no_active_capture_produces_no_cancel() {
    let mut cap = PointerCapture::default();
    let cancel = cap.release_matching(&["btn".to_string()]);
    assert!(cancel.is_none());
}

#[test]
fn cleared_capture_can_be_released_again() {
    let mut cap = PointerCapture::default();
    cap.set("btn".to_string(), Some(NodeId::from_raw(1)));
    let _ = cap.release_matching(&["btn".to_string()]);
    // Second call: no-op because already cleared.
    let cancel = cap.release_matching(&["btn".to_string()]);
    assert!(cancel.is_none());
}

#[test]
fn cancel_event_struct_has_required_fields() {
    // Type-level check: the cancel struct has both `node` and
    // `button` fields and implements `Copy`/`Debug`/`PartialEq`.
    let cancel = PointerCancel {
        node: NodeId::from_raw(1),
        button: Some(PointerButton::Primary),
    };
    let _copy: PointerCancel = cancel; // Copy
    let _ = format!("{:?}", cancel); // Debug
    let _same: bool = cancel == cancel; // PartialEq
    assert_eq!(cancel.button, Some(PointerButton::Primary));
}
