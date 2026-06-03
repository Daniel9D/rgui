use crate::{LayerKind, NodeId, Point, Rect, Vec2};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerButton {
    Primary,
    Secondary,
    Middle,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointerEvent {
    pub position: Point,
    pub button: Option<PointerButton>,
    pub modifiers: u32,
}

/// Phase 1 / Plan 01-03: a synthetic cancel event for a captured
/// node that was just unmounted. The `node` is the captured
/// target; dispatch bypasses hit-testing and routes the event
/// directly there.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointerCancel {
    pub node: NodeId,
    pub button: Option<PointerButton>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyEvent {
    pub key: String,
    pub modifiers: u32,
    pub repeat: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WheelDeltaMode {
    Pixels,
    Lines,
    Pages,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WheelEvent {
    pub delta: Vec2,
    pub position: Point,
    pub mode: WheelDeltaMode,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImePreedit {
    pub text: String,
    pub cursor_byte_range: Option<(usize, usize)>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum UiEvent {
    PointerDown(PointerEvent),
    PointerMove(PointerEvent),
    PointerUp(PointerEvent),
    /// Synthetic pointer-cancel: emitted when a captured node is
    /// unmounted by the reconciler (Phase 1 / Plan 01-03). The
    /// `node` is the captured target — dispatch bypasses hit-testing
    /// and routes the event directly to that node so any drag
    /// handler can clean up.
    PointerCancel {
        node: NodeId,
        button: Option<PointerButton>,
    },
    Wheel(WheelEvent),
    KeyDown(KeyEvent),
    KeyUp(KeyEvent),
    TextInput(String),
    ImePreedit(ImePreedit),
    ImeCommit(String),
    FocusGained,
    FocusLost,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventPhase {
    Capture,
    Target,
    Bubble,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EventResult {
    pub handled: bool,
    pub stop_propagation: bool,
    pub prevent_default: bool,
}

impl EventResult {
    pub const fn ignored() -> Self {
        Self {
            handled: false,
            stop_propagation: false,
            prevent_default: false,
        }
    }

    pub const fn handled() -> Self {
        Self {
            handled: true,
            stop_propagation: false,
            prevent_default: false,
        }
    }

    pub const fn stop_propagation(mut self) -> Self {
        self.stop_propagation = true;
        self
    }

    pub const fn prevent_default(mut self) -> Self {
        self.prevent_default = true;
        self
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FocusManager {
    focused: Option<NodeId>,
}

impl FocusManager {
    pub fn request_focus(&mut self, node: NodeId) {
        self.focused = Some(node);
    }

    pub fn clear(&mut self) {
        self.focused = None;
    }

    pub const fn focused(&self) -> Option<NodeId> {
        self.focused
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShortcutScope {
    FocusedNode(NodeId),
    FocusScope(NodeId),
    Window,
    Application,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shortcut {
    pub chord: String,
    pub scope: ShortcutScope,
    pub action: String,
}

impl Shortcut {
    pub fn new(chord: impl Into<String>, scope: ShortcutScope, action: impl Into<String>) -> Self {
        Self {
            chord: chord.into(),
            scope,
            action: action.into(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ShortcutRegistry {
    shortcuts: Vec<Shortcut>,
}

impl ShortcutRegistry {
    pub fn register(&mut self, shortcut: Shortcut) {
        self.shortcuts.push(shortcut);
    }

    /// Phase 2 / Plan 02-02: when the focused node is a text input
    /// (an `Input`, `Textarea`, or `Select`), only modifier-prefixed
    /// shortcuts fire — bare letter / digit / punctuation /
    /// function-key shortcuts are suppressed so the user can type
    /// freely. Modifier-only chords (containing `Cmd+`, `Ctrl+`, or
    /// `Alt+`) always fire, even inside a text field.
    pub fn resolve(
        &self,
        chord: &str,
        focused: Option<NodeId>,
        focused_is_text_input: bool,
    ) -> Option<&str> {
        self.shortcuts
            .iter()
            .find(|shortcut| {
                shortcut.chord == chord
                    && matches!(
                        shortcut.scope,
                        ShortcutScope::FocusedNode(node) if Some(node) == focused
                    )
            })
            .or_else(|| {
                self.shortcuts.iter().find(|shortcut| {
                    shortcut.chord == chord && shortcut.scope == ShortcutScope::Window
                })
            })
            .or_else(|| {
                self.shortcuts.iter().find(|shortcut| {
                    shortcut.chord == chord && shortcut.scope == ShortcutScope::Application
                })
            })
            .and_then(|shortcut| {
                if focused_is_text_input && !is_modifier_chord(&shortcut.chord) {
                    None
                } else {
                    Some(shortcut.action.as_str())
                }
            })
    }
}

/// Phase 2 / Plan 02-02: a chord is "modifier-prefixed" if it
/// contains `Cmd+`, `Ctrl+`, or `Alt+`. These chords always fire
/// (e.g. `Cmd+K` to open a palette from inside a text field).
/// Bare chords like `"a"`, `"?"`, or `"Enter"` are suppressed
/// inside text inputs.
pub fn is_modifier_chord(chord: &str) -> bool {
    chord.contains("Cmd+") || chord.contains("Ctrl+") || chord.contains("Alt+")
}

#[derive(Clone, Debug, PartialEq)]
pub struct HitTestEntry {
    pub node: NodeId,
    pub key: Option<String>,
    pub rect: Rect,
    pub visible_rect: Option<Rect>,
    pub z_index: i32,
    pub layer: LayerKind,
    pub pointer_events: bool,
    pub order: usize,
}

impl HitTestEntry {
    /// Bug fix 5.1: this is a struct-literal constructor with no
    /// allocation or runtime work, so it is `const fn`. Callers
    /// that need a fully-initialized entry (e.g. tests or
    /// debug-overlay tables) can build one at compile time.
    pub const fn new(node: NodeId, rect: Rect, z_index: i32, layer: LayerKind) -> Self {
        Self {
            node,
            key: None,
            rect,
            visible_rect: None,
            z_index,
            layer,
            pointer_events: true,
            order: 0,
        }
    }

    pub fn with_key(mut self, key: Option<String>) -> Self {
        self.key = key;
        self
    }

    pub fn with_order(mut self, order: usize) -> Self {
        self.order = order;
        self
    }

    pub fn with_visible_rect(mut self, visible_rect: Option<Rect>) -> Self {
        self.visible_rect = visible_rect;
        self
    }

    pub fn hit_rect(&self) -> Rect {
        self.visible_rect.unwrap_or(self.rect)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct HitTestTree {
    entries: Vec<HitTestEntry>,
}

impl HitTestTree {
    pub fn push(&mut self, entry: HitTestEntry) {
        self.entries.push(entry);
    }

    pub fn entries(&self) -> &[HitTestEntry] {
        &self.entries
    }

    pub fn hit_test(&self, point: Point) -> Option<NodeId> {
        self.hit(point).map(|entry| entry.node)
    }

    pub fn hit(&self, point: Point) -> Option<&HitTestEntry> {
        // Bug fix 1.8: `max_by_key` returns the *last* element on tie, so two
        // overlapping entries with the same (layer, z_index, order) would
        // resolve to whichever was pushed later — usually correct, but not
        // guaranteed. Make the ordering total by also keying on the entry's
        // position in the underlying vec (so the most recently added entry
        // wins on a complete tie, which is the more intuitive behavior for
        // overlays pushed last).
        self.entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| entry.pointer_events && entry.hit_rect().contains(point))
            .max_by_key(|(idx, entry)| (entry.layer.order(), entry.z_index, entry.order, *idx))
            .map(|(_, entry)| entry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Size;

    // Bug fix 5.1: `HitTestEntry::new` is `const fn`. Verify by
    // constructing one in const context and asserting the field
    // values match the documented defaults.
    const HIT_ENTRY: HitTestEntry = HitTestEntry::new(
        NodeId::from_raw(7),
        Rect::new(Point::new(1.0, 2.0), Size::new(3.0, 4.0)),
        5,
        LayerKind::Popover,
    );

    #[test]
    fn hit_test_entry_new_is_const_constructible() {
        assert_eq!(HIT_ENTRY.node.raw(), 7);
        assert_eq!(HIT_ENTRY.rect.origin.x, 1.0);
        assert_eq!(HIT_ENTRY.rect.origin.y, 2.0);
        assert_eq!(HIT_ENTRY.z_index, 5);
        assert_eq!(HIT_ENTRY.layer, LayerKind::Popover);
        assert!(HIT_ENTRY.pointer_events);
        assert_eq!(HIT_ENTRY.order, 0);
        assert!(HIT_ENTRY.key.is_none());
        assert!(HIT_ENTRY.visible_rect.is_none());
    }

    // Phase 2 / Plan 02-02: shortcut suppression inside text inputs.
    mod shortcut_suppression {
        use super::*;

        fn reg() -> ShortcutRegistry {
            let mut r = ShortcutRegistry::default();
            r.register(Shortcut::new("a", ShortcutScope::Window, "approve"));
            r.register(Shortcut::new("Cmd+a", ShortcutScope::Window, "select_all"));
            r.register(Shortcut::new("?", ShortcutScope::Window, "help"));
            r.register(Shortcut::new("Enter", ShortcutScope::Window, "submit"));
            r
        }

        #[test]
        fn plain_letter_suppressed_in_text_input() {
            let r = reg();
            let action = r.resolve("a", None, true);
            assert_eq!(action, None, "bare letter must be suppressed inside a text input");
        }

        #[test]
        fn modifier_prefixed_chord_fires_in_text_input() {
            let r = reg();
            let action = r.resolve("Cmd+a", None, true);
            assert_eq!(action, Some("select_all"));
        }

        #[test]
        fn plain_letter_fires_outside_text_input() {
            let r = reg();
            let action = r.resolve("a", None, false);
            assert_eq!(action, Some("approve"));
        }

        #[test]
        fn digit_and_punctuation_suppressed_in_text_input() {
            let r = reg();
            assert_eq!(r.resolve("?", None, true), None);
            assert_eq!(r.resolve("Enter", None, true), None);
        }
    }
}
