//! Phase 2 / Plan 02-04: IME composition gating tests.
//!
//! These tests verify the new `InputSpec::ime_enabled: bool` flag.
//! When `ime_enabled: true` (the default in user code that opts in),
//! `ImePreedit` / `ImeCommit` events route to the focused `Input`
//! and the preedit text is painted. When `ime_enabled: false`
//! (the default for Latin-keyboard apps), the runtime ignores
//! `ImePreedit` events and the focused input's `preedit` state
//! stays `None`.

use rgui::runtime::{FrameInput, UiRuntime};
use rgui::widgets::input;
use rgui::{Element, ImePreedit, Size, UiEvent};

/// TDD-02-04-09: preedit is stored when `ime_enabled: true`.
#[test]
fn ime_preedit_routes_to_input_when_ime_enabled() {
    let mut runtime = UiRuntime::default();
    let root = Element::column().child(input().key("txt").ime_enabled(true));
    runtime.update(FrameInput {
        root: root.clone(),
        viewport: Size::new(200.0, 100.0),
        ..Default::default()
    });

    // Focus the input via Tab.
    runtime.dispatch(UiEvent::KeyDown(rgui::KeyEvent {
        key: "Tab".to_string(),
        modifiers: 0,
        repeat: false,
    }));
    assert_eq!(runtime.focused_key().as_deref(), Some("txt"));

    // Dispatch an ImePreedit.
    runtime.dispatch(UiEvent::ImePreedit(ImePreedit {
        text: "あ".to_string(),
        cursor_byte_range: Some((0, 3)),
    }));

    // The preedit should be painted (text = "あ") in the next frame.
    let output = runtime.update(FrameInput {
        root,
        viewport: Size::new(200.0, 100.0),
        ..Default::default()
    });

    let painted = output.display_list.commands().iter().any(|cmd| {
        matches!(cmd, rgui::PaintCommand::DrawText(t) if t.text == "あ")
    });
    assert!(
        painted,
        "preedit text should be painted when ime_enabled is true"
    );
}

/// TDD-02-04-10: preedit is ignored when `ime_enabled: false`.
#[test]
fn ime_preedit_is_dropped_when_ime_disabled() {
    let mut runtime = UiRuntime::default();
    let root = Element::column().child(input().key("txt"));
    runtime.update(FrameInput {
        root: root.clone(),
        viewport: Size::new(200.0, 100.0),
        ..Default::default()
    });

    // Focus the input.
    runtime.dispatch(UiEvent::KeyDown(rgui::KeyEvent {
        key: "Tab".to_string(),
        modifiers: 0,
        repeat: false,
    }));
    assert_eq!(runtime.focused_key().as_deref(), Some("txt"));

    // Dispatch an ImePreedit — should be dropped (ime_enabled defaults to false).
    runtime.dispatch(UiEvent::ImePreedit(ImePreedit {
        text: "あ".to_string(),
        cursor_byte_range: Some((0, 3)),
    }));

    // Render — the preedit text should NOT appear.
    let output = runtime.update(FrameInput {
        root,
        viewport: Size::new(200.0, 100.0),
        ..Default::default()
    });

    let painted = output.display_list.commands().iter().any(|cmd| {
        matches!(cmd, rgui::PaintCommand::DrawText(t) if t.text == "あ")
    });
    assert!(
        !painted,
        "preedit text should be dropped when ime_enabled is false"
    );
}

/// ImeCommit still works regardless of `ime_enabled` — commit is
/// the only reliable way to insert CJK text, so we always honor it.
#[test]
fn ime_commit_always_works_regardless_of_ime_enabled() {
    let mut runtime = UiRuntime::default();
    let root = Element::column().child(input().key("txt"));
    runtime.update(FrameInput {
        root,
        viewport: Size::new(200.0, 100.0),
        ..Default::default()
    });

    runtime.dispatch(UiEvent::KeyDown(rgui::KeyEvent {
        key: "Tab".to_string(),
        modifiers: 0,
        repeat: false,
    }));

    runtime.dispatch(UiEvent::ImeCommit("日本".to_string()));

    assert_eq!(runtime.text_state("txt").as_deref(), Some("日本"));
}
