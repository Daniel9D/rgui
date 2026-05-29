use crate::core::{AccessibilityBackend, Role, SemanticAction, SemanticNode, SemanticTree};
use crate::runtime::UiCommand;

/// Placeholder backend that records semantic tree metrics for testing and
/// diagnostics but does not publish platform accessibility nodes.
pub struct RealAccessibilityBackend {
    update_counter: u64,
    node_count: usize,
    focused_node: Option<crate::core::NodeId>,
}

impl RealAccessibilityBackend {
    pub fn new() -> Self {
        Self {
            update_counter: 0,
            node_count: 0,
            focused_node: None,
        }
    }

    pub fn update_count(&self) -> u64 {
        self.update_counter
    }

    pub fn node_count(&self) -> usize {
        self.node_count
    }
}

impl AccessibilityBackend for RealAccessibilityBackend {
    fn update(&mut self, tree: &SemanticTree) {
        self.node_count = tree.nodes().len();
        self.focused_node = tree
            .nodes()
            .iter()
            .find(|n| n.states.focused)
            .map(|n| n.node);
        self.update_counter += 1;
    }
}

pub fn role_to_str(role: Role) -> &'static str {
    match role {
        Role::Window => "window",
        Role::Group => "group",
        Role::Text => "text",
        Role::Button => "button",
        Role::TextInput => "text-input",
        Role::Checkbox => "checkbox",
        Role::Radio => "radio",
        Role::List => "list",
        Role::ListItem => "list-item",
        Role::Table => "table",
        Role::Row => "row",
        Role::Cell => "cell",
        Role::Dialog => "dialog",
        Role::Menu => "menu",
        Role::MenuItem => "menu-item",
        Role::Tooltip => "tooltip",
        Role::ScrollArea => "scroll-area",
        Role::Image => "image",
        Role::Switch => "switch",
        Role::Slider => "slider",
        Role::ProgressBar => "progressbar",
        Role::Spinner => "progressbar",
        Role::Badge => "status",
        Role::Avatar => "img",
        Role::Link => "link",
        Role::Alert => "alert",
        Role::Card => "group",
    }
}

pub fn action_to_str(action: SemanticAction) -> &'static str {
    match action {
        SemanticAction::Press => "press",
        SemanticAction::Focus => "focus",
        SemanticAction::SetValue => "set-value",
        SemanticAction::ScrollForward => "scroll-forward",
        SemanticAction::ScrollBackward => "scroll-backward",
    }
}

#[cfg(feature = "accesskit")]
pub struct AccessKitBackend {
    update_counter: u64,
}

#[cfg(feature = "accesskit")]
impl AccessKitBackend {
    pub fn new() -> Self {
        Self { update_counter: 0 }
    }

    pub fn update_count(&self) -> u64 {
        self.update_counter
    }
}

#[cfg(feature = "accesskit")]
impl Default for AccessKitBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "accesskit")]
impl AccessibilityBackend for AccessKitBackend {
    fn update(&mut self, _tree: &SemanticTree) {
        self.update_counter += 1;
    }
}

pub fn command_for_action(
    node: &SemanticNode,
    action: SemanticAction,
    value: Option<String>,
) -> Option<UiCommand> {
    let key = node.key.clone()?;
    match action {
        SemanticAction::Press => Some(UiCommand::Click {
            key: Some(key),
            action: None,
        }),
        SemanticAction::Focus => Some(UiCommand::Focus { key }),
        SemanticAction::SetValue => value.map(|value| UiCommand::SetText { key, value }),
        SemanticAction::ScrollForward | SemanticAction::ScrollBackward => None,
    }
}
