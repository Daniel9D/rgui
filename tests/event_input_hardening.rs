//! Phase 2 / Plan 02-01: end-to-end focus traversal tests.
//!
//! Demonstrates the new `FocusManager::tab_next` / `tab_prev` helpers
//! and the existing runtime Tab dispatch (which uses the
//! `FocusSystem` for scope-based routing; the new `FocusManager`
//! is the simpler tree-walking alternative).

use rgui::core::event::FocusManager;
use rgui::core::NodeId;
use rgui::runtime::{FrameInput, UiRuntime};
use rgui::widgets::{button, input};
use rgui::{Element, Size, UiEvent};

/// TDD-02-01-09: a form with N focusable widgets; Tab cycles
/// through them in DOM order. The runtime already supports this
/// via `focus_system.tab_forward()`; this test pins the behavior
/// so we don't regress.
#[test]
fn form_with_tab_cycles_through_focusable_widgets() {
    let mut runtime = UiRuntime::default();
    runtime.update(FrameInput {
        root: Element::column()
            .child(input().key("a"))
            .child(input().key("b"))
            .child(button("Save").key("c"))
            .child(button("Cancel").key("d"))
            .child(button("Submit").key("e")),
        viewport: Size::new(240.0, 200.0),
        ..Default::default()
    });

    let expected = ["a", "b", "c", "d", "e"];
    for (i, key) in expected.iter().enumerate() {
        runtime.dispatch(UiEvent::KeyDown(rgui::KeyEvent {
            key: "Tab".to_string(),
            modifiers: 0,
            repeat: false,
        }));
        assert_eq!(
            runtime.focused_key().as_deref(),
            Some(*key),
            "after {} tab(s), expected focus on {key}",
            i + 1
        );
    }

    // One more Tab wraps to the first.
    runtime.dispatch(UiEvent::KeyDown(rgui::KeyEvent {
        key: "Tab".to_string(),
        modifiers: 0,
        repeat: false,
    }));
    assert_eq!(runtime.focused_key().as_deref(), Some("a"));
}


/// TDD-02-01-10: when a candidate list is filtered to a modal's
/// subtree, Tab cycles inside that subtree. The runtime does this
/// filter when `ModalSpec::trap_focus: true`; here we exercise the
/// `FocusManager::tab_next` helper directly with a filtered list.
#[test]
fn focus_manager_tab_next_cycles_inside_filtered_subtree() {
    // Simulate: nodes 1..=5 are in the modal subtree; 6 is outside.
    let mut f = FocusManager::default();
    let modal_subtree: Vec<NodeId> = (1..=5).map(NodeId::from_raw).collect();

    // First Tab from None -> first in modal.
    let next = f.tab_next(modal_subtree.iter().copied());
    assert_eq!(next, Some(NodeId::from_raw(1)));

    // Subsequent Tabs cycle inside 1..=5.
    for expected in [2u64, 3, 4, 5, 1, 2] {
        let next = f.tab_next(modal_subtree.iter().copied());
        assert_eq!(next, Some(NodeId::from_raw(expected)));
    }
}
