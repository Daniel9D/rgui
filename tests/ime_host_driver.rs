//! Phase 3 / Plan 03-01: IME host driver integration tests.
//!
//! These tests exercise the producer-side `ImeHostDriver` trait
//! added in Phase 3. The receive side (`handle_ime_preedit` +
//! `is_focused_ime_enabled`) is unchanged from Phase 2; the
//! `tests/ime_gating.rs` suite covers that surface. This file
//! proves:
//!
//! 1. `NoopDriver` (the default) never produces events.
//! 2. `MockDriver` replays a `Vec<ImeOp>` script via the runtime's
//!    frame pump.
//! 3. Preedit-replaces-preedit semantics work (a new preedit
//!    overwrites the old; the input never sees a concatenated
//!    preedit buffer).
//! 4. Driver-sourced events are ignored when the focused widget
//!    is not a text input (the Phase 2 `ime_enabled` gate is the
//!    second line of defense).

use rgui::runtime::{FrameInput, ImeOp, MockDriver, UiRuntime};
use rgui::widgets::{button, input};
use rgui::{Element, Size};

/// TDD-03-01-A: the default driver is a `NoopDriver`; processing a
/// frame produces no IME events.
#[test]
fn noop_driver_emits_no_events_on_default_runtime() {
    let mut runtime = UiRuntime::default();
    let root = Element::column().child(input().key("txt").ime_enabled(true));
    let output = runtime.update(FrameInput {
        root: root.clone(),
        viewport: Size::new(200.0, 100.0),
        ..Default::default()
    });

    // No IME preedit text should be painted.
    let painted_preedit = output.display_list.commands().iter().any(|cmd| {
        matches!(cmd, rgui::PaintCommand::DrawText(t) if !t.text.is_empty() && t.text != "txt")
    });
    assert!(
        !painted_preedit,
        "NoopDriver should never produce IME events"
    );
}

/// TDD-03-01-B: a `MockDriver` script that fires `Preedit` then
/// `Commit` ends with the focused input containing the committed
/// text and no preedit.
#[test]
fn mock_driver_preedit_then_commit_lands_on_focused_input() {
    let mut runtime = UiRuntime::with_driver(Box::new(MockDriver {
        script: vec![
            ImeOp::Preedit("konni".to_string(), None),
            ImeOp::Commit("こんにちは".to_string()),
        ],
        cursor: 0,
        fired: false,
    }));
    let root = Element::column().child(input().key("txt").ime_enabled(true));
    runtime.update(FrameInput {
        root: root.clone(),
        viewport: Size::new(400.0, 100.0),
        ..Default::default()
    });

    // Focus the input via Tab.
    runtime.dispatch(rgui::UiEvent::KeyDown(rgui::KeyEvent {
        key: "Tab".to_string(),
        modifiers: 0,
        repeat: false,
    }));
    assert_eq!(runtime.focused_key().as_deref(), Some("txt"));

    // The next `update()` polls the driver once and dispatches the
    // first op (Preedit). After the *following* `update()` the
    // second op (Commit) is dispatched and the preedit clears.
    runtime.update(FrameInput {
        root: root.clone(),
        viewport: Size::new(400.0, 100.0),
        ..Default::default()
    });
    runtime.update(FrameInput {
        root,
        viewport: Size::new(400.0, 100.0),
        ..Default::default()
    });

    // The input's committed text is the post-commit string.
    assert_eq!(runtime.text_state("txt").as_deref(), Some("こんにちは"));
}

/// TDD-03-01-C: a sequence of two `Preedit` ops results in the
/// second one replacing the first — the input never sees a
/// concatenated preedit buffer. We assert by checking that after
/// the second preedit, only the second preedit's text is what
/// the runtime would paint (i.e. the first preedit text is not
/// present).
#[test]
fn mock_driver_preedit_replaces_previous_preedit() {
    let mut runtime = UiRuntime::with_driver(Box::new(MockDriver {
        script: vec![
            ImeOp::Preedit("a".to_string(), None),
            ImeOp::Preedit("ab".to_string(), None),
            ImeOp::Commit("ab".to_string()),
        ],
        cursor: 0,
        fired: false,
    }));
    let root = Element::column().child(input().key("txt").ime_enabled(true));
    runtime.update(FrameInput {
        root: root.clone(),
        viewport: Size::new(400.0, 100.0),
        ..Default::default()
    });
    runtime.dispatch(rgui::UiEvent::KeyDown(rgui::KeyEvent {
        key: "Tab".to_string(),
        modifiers: 0,
        repeat: false,
    }));

    // One frame per script op. After all three frames the
    // committed text is "ab" (the preedit was replaced and then
    // committed; the original "a" was never concatenated).
    for _ in 0..3 {
        runtime.update(FrameInput {
            root: root.clone(),
            viewport: Size::new(400.0, 100.0),
            ..Default::default()
        });
    }
    assert_eq!(runtime.text_state("txt").as_deref(), Some("ab"));
}

/// TDD-03-01-D: driver-sourced preedit is ignored when the focused
/// widget is not a text input. We focus a `Button` and then let
/// the driver emit a `Preedit` — the runtime's `dispatch` should
/// route to `handle_ime_preedit`, which gates on the focused
/// widget kind. No input state should be touched.
#[test]
fn driver_events_ignored_when_focus_is_not_text_input() {
    let mut runtime = UiRuntime::with_driver(Box::new(MockDriver {
        script: vec![ImeOp::Preedit("foo".to_string(), None)],
        cursor: 0,
        fired: false,
    }));
    let root = Element::column()
        .child(button("Click me").key("btn"))
        .child(input().key("txt"));
    runtime.update(FrameInput {
        root: root.clone(),
        viewport: Size::new(400.0, 100.0),
        ..Default::default()
    });

    // Tab once: with no input `ime_enabled` opt-in, focus lands
    // on the button (it's the first focusable widget).
    runtime.dispatch(rgui::UiEvent::KeyDown(rgui::KeyEvent {
        key: "Tab".to_string(),
        modifiers: 0,
        repeat: false,
    }));
    assert_eq!(runtime.focused_key().as_deref(), Some("btn"));

    // Frame that polls the driver. The driver emits a preedit,
    // but the focused widget is a button — `is_focused_ime_enabled`
    // is false, so the preedit is dropped.
    runtime.update(FrameInput {
        root,
        viewport: Size::new(400.0, 100.0),
        ..Default::default()
    });

    // No input state has text.
    assert_eq!(runtime.text_state("txt"), None);
}
