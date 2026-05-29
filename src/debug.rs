#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DebugOptions {
    pub enabled: bool,
    pub toggle_shortcut: String,
}

impl Default for DebugOptions {
    fn default() -> Self {
        Self {
            enabled: cfg!(debug_assertions),
            toggle_shortcut: "Ctrl+Shift+I".to_string(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InspectorPanel {
    ElementTree,
    ResolvedStyle,
    Layout,
    Paint,
    HitTest,
    Events,
    Focus,
    Scroll,
    Overlays,
    Accessibility,
    Atlas,
}
