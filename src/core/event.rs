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

    pub fn resolve(&self, chord: &str, focused: Option<NodeId>) -> Option<&str> {
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
            .map(|shortcut| shortcut.action.as_str())
    }
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
    pub fn new(node: NodeId, rect: Rect, z_index: i32, layer: LayerKind) -> Self {
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
        self.entries
            .iter()
            .filter(|entry| entry.pointer_events && entry.hit_rect().contains(point))
            .max_by_key(|entry| (layer_order(entry.layer), entry.z_index, entry.order))
    }
}

fn layer_order(layer: LayerKind) -> i32 {
    match layer {
        LayerKind::Document => 0,
        LayerKind::Floating => 1,
        LayerKind::Popover => 2,
        LayerKind::Tooltip => 3,
        LayerKind::ContextMenu => 4,
        LayerKind::Modal => 5,
        LayerKind::Debug => 6,
    }
}
