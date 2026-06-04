//! Phase 4 / Plan 04-01: cross-window event types.
//!
//! `AppEvent` covers the four cross-window concerns a lib wants to
//! see, separate from per-window `UiEvent` (pointer / keyboard /
//! IME / resize):
//!
//! - `Quit` — the host wants the process to exit.
//! - `FocusWindow(id)` — the host wants to raise / focus a window.
//!   Returns `Consumed` only if the id matches this runtime's
//!   window; otherwise `Ignored` (routing is the host's job).
//! - `ThemeChanged(theme)` — the host's theme source changed;
//!   `Theme` lives in `rgui::core::Theme`.
//! - `AppShortcut(name)` — a host-defined cross-window shortcut
//!   (e.g. "Cmd+W" to close the active window). Bound via
//!   `AppShortcuts::register`.
//!
//! `AppEventOutcome` reports whether the runtime consumed the
//! event. The host uses this to decide whether to keep the event
//! (Quit → call `event_loop.exit()`) or continue routing (e.g.
//! focus events for a different window).
//!
//! `AppShortcuts` is the host's extensibility surface for
//! cross-window shortcuts. Closures receive `&mut UiRuntime` and
//! can mutate any per-window state. Dispatch lives on
//! `UiRuntime::dispatch_app_event` in `runtime.rs` so it can
//! reach the runtime's private fields.

use std::collections::HashMap;

use crate::core::Theme;

use super::{UiRuntime, WindowId};

/// Cross-window events the lib wants to see. Per-window events
/// (pointer, keyboard, IME, resize) stay as `UiEvent`.
#[derive(Debug, Clone, PartialEq)]
pub enum AppEvent {
    /// The host wants the process to exit. Always `Consumed`.
    Quit,
    /// Focus a specific window. `Consumed` if `target` matches
    /// this runtime's window; `Ignored` otherwise.
    FocusWindow(WindowId),
    /// The host's theme source changed. Updates the runtime's
    /// theme. Always `Consumed`.
    ThemeChanged(Theme),
    /// A host-defined cross-window shortcut (e.g. `"close_window"`).
    /// Looks up the name in `AppShortcuts`. `Consumed` if a
    /// binding is registered, `Ignored` otherwise.
    AppShortcut(String),
}

/// Whether the runtime consumed an `AppEvent`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppEventOutcome {
    /// The runtime handled the event. The host should not re-dispatch.
    Consumed,
    /// The runtime did not handle the event. The host is responsible
    /// for any remaining routing (e.g. focusing a different window).
    Ignored,
}

/// Host-defined bindings for `AppEvent::AppShortcut(String)`. Use
/// `register(name, closure)` to wire a shortcut; the closure is
/// invoked with `&mut UiRuntime` when the event fires.
#[derive(Default)]
pub struct AppShortcuts {
    /// Name → closure. `pub(crate)` so `UiRuntime::dispatch_app_event`
    /// can do the take / invoke / re-insert dance forced by the
    /// borrow checker (the closure needs `&mut self` as its
    /// argument, which conflicts with a method call on
    /// `&mut self.app_shortcuts`).
    pub(crate) bindings: HashMap<String, Box<dyn Fn(&mut UiRuntime) + Send + Sync>>,
}

impl AppShortcuts {
    /// Construct an empty `AppShortcuts`.
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    /// Register a shortcut binding. Replaces any existing
    /// binding for the same name.
    pub fn register<F>(&mut self, name: impl Into<String>, f: F)
    where
        F: Fn(&mut UiRuntime) + Send + Sync + 'static,
    {
        self.bindings.insert(name.into(), Box::new(f));
    }
}

impl std::fmt::Debug for AppShortcuts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppShortcuts")
            .field("bindings", &self.bindings.keys().collect::<Vec<_>>())
            .finish()
    }
}
