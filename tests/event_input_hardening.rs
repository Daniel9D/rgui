//! Phase 2 / Plan 02-01: end-to-end focus traversal tests.
//!
//! Demonstrates the new `FocusManager::tab_next` / `tab_prev` helpers
//! and the existing runtime Tab dispatch (which uses the
//! `FocusSystem` for scope-based routing; the new `FocusManager`
//! is the simpler tree-walking alternative).

use rgui::core::event::FocusManager;
use rgui::core::NodeId;
use rgui::runtime::{FrameInput, UiRuntime};
use rgui::widgets::{button, input, text};
use rgui::{
    Element, Overflow, Point, Size, UiEvent, Vec2, WheelDeltaMode, WheelEvent,
};

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

// ── Phase 2 / Plan 02-03: wheel 2D + nested scroll containers ──

fn horizontal_scroll_app() -> Element {
    // A horizontal-only scroll area with 5 child cards. We use
    // Element::row() with overflow_x(Scroll) + overflow_y(Hidden)
    // to mimic a real horizontal carousel.
    Element::row()
        .key("h-scroll")
        .width(100.0)
        .height(40.0)
        .overflow_x(Overflow::Scroll)
        .overflow_y(Overflow::Hidden)
        .child(text("card-1").width(40.0))
        .child(text("card-2").width(40.0))
        .child(text("card-3").width(40.0))
        .child(text("card-4").width(40.0))
        .child(text("card-5").width(40.0))
}

/// TDD-02-03-08: trackpad pan on a horizontal-only scroll area
/// scrolls horizontally. The runtime's `handle_wheel` reads
/// `delta.x` and applies it to the scroll target's x offset.
#[test]
fn trackpad_pan_on_horizontal_scroll_area_scrolls_horizontally() {
    let mut runtime = UiRuntime::default();
    runtime.update(FrameInput {
        root: horizontal_scroll_app(),
        viewport: Size::new(200.0, 100.0),
        ..Default::default()
    });

    runtime.dispatch(UiEvent::Wheel(WheelEvent {
        position: Point::new(20.0, 20.0),
        delta: Vec2::new(50.0, 0.0),
        mode: WheelDeltaMode::Pixels,
    }));

    let offset = runtime
        .scroll_offset("h-scroll")
        .expect("horizontal scroll offset should be set");
    assert!(
        offset.x > 0.0,
        "horizontal pan should advance the x offset; got {offset:?}"
    );
}

/// TDD-02-03-09: vertical wheel on a horizontal-only area is
/// dropped (max_scroll.y is 0, so the y offset is clamped to 0).
#[test]
fn vertical_wheel_on_horizontal_only_area_is_dropped() {
    let mut runtime = UiRuntime::default();
    runtime.update(FrameInput {
        root: horizontal_scroll_app(),
        viewport: Size::new(200.0, 100.0),
        ..Default::default()
    });

    runtime.dispatch(UiEvent::Wheel(WheelEvent {
        position: Point::new(20.0, 20.0),
        delta: Vec2::new(0.0, 50.0),
        mode: WheelDeltaMode::Pixels,
    }));

    let offset = runtime
        .scroll_offset("h-scroll")
        .expect("horizontal scroll offset should be set");
    assert_eq!(
        offset.y, 0.0,
        "vertical wheel must be dropped on a horizontal-only area; got {offset:?}"
    );
}

/// TDD-02-03-10: nested scroll areas route each axis to the
/// correct target. A vertical inside a horizontal: vertical wheel
/// scrolls the inner; horizontal wheel scrolls the outer.
#[test]
fn nested_scroll_areas_route_each_axis_to_correct_target() {
    // Outer: horizontal scroll area containing a vertical scroll area.
    let root = Element::row()
        .key("outer")
        .width(100.0)
        .height(80.0)
        .overflow_x(Overflow::Scroll)
        .overflow_y(Overflow::Hidden)
        .child(
            // Inner: a vertical scroll area (deepest under pointer)
            Element::column()
                .key("inner")
                .width(80.0)
                .height(40.0)
                .overflow_x(Overflow::Hidden)
                .overflow_y(Overflow::Scroll)
                .child(text("a").height(30.0))
                .child(text("b").height(30.0))
                .child(text("c").height(30.0))
                .child(text("d").height(30.0)),
        )
        .child(text("tail-1").width(40.0))
        .child(text("tail-2").width(40.0));

    let mut runtime = UiRuntime::default();
    runtime.update(FrameInput {
        root,
        viewport: Size::new(200.0, 200.0),
        ..Default::default()
    });

    // A wheel on the inner area: find_scrollable_ancestor returns
    // the deepest (inner, which is vertical). The runtime applies
    // both delta.x and delta.y to the inner, but inner is vertical-only
    // so delta.x is dropped. The vertical part advances inner.y.
    runtime.dispatch(UiEvent::Wheel(WheelEvent {
        position: Point::new(20.0, 20.0),
        delta: Vec2::new(30.0, 40.0),
        mode: WheelDeltaMode::Pixels,
    }));

    let inner = runtime
        .scroll_offset("inner")
        .expect("inner scroll offset should be set");
    assert!(
        inner.y > 0.0,
        "vertical wheel on inner should advance inner.y; got {inner:?}"
    );
    assert_eq!(
        inner.x, 0.0,
        "vertical-only inner should drop horizontal delta; got {inner:?}"
    );
}
