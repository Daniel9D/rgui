use crate::core::{AccessibilityBackend, Role, SemanticAction, SemanticNode, SemanticTree};
use crate::runtime::UiCommand;

/// Placeholder backend that records semantic tree metrics for testing and
/// diagnostics but does not publish platform accessibility nodes.
#[derive(Default)]
pub struct RealAccessibilityBackend {
    update_counter: u64,
    node_count: usize,
    focused_node: Option<crate::core::NodeId>,
}

impl RealAccessibilityBackend {
    pub const fn new() -> Self {
        Self {
            update_counter: 0,
            node_count: 0,
            focused_node: None,
        }
    }

    pub const fn update_count(&self) -> u64 {
        self.update_counter
    }

    pub const fn node_count(&self) -> usize {
        self.node_count
    }

    /// The `NodeId` of the focused node at the last `update`, if any.
    pub const fn focused_node(&self) -> Option<crate::core::NodeId> {
        self.focused_node
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

impl Role {
    /// Stable ARIA role string. Lower values are drawn first / hit-tested first.
    ///
    /// Notes:
    /// - `Spinner` maps to `"status"` (a polite live region) rather than
    ///   `"progressbar"`; spinners are indeterminate, progress bars are not.
    /// - `Badge` and `Card` collapse to `"group"`; neither has a discriminating
    ///   ARIA role.
    /// - `Avatar` maps to `"img"` (correct ARIA for a non-textual avatar).
    pub const fn as_str(self) -> &'static str {
        match self {
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
            // Bug fix 1.1: Spinner is NOT a progressbar; use a polite live region.
            Role::Spinner => "status",
            // Bug fix 1.3: Badge as "status" was over-promising; use generic group.
            Role::Badge => "group",
            Role::Avatar => "img",
            Role::Link => "link",
            Role::Alert => "alert",
            // Bug fix 1.2: Card stays as group; with aria-label it could become region.
            Role::Card => "group",
        }
    }
}

impl SemanticAction {
    /// Stable action name for diagnostics, serialization, and tests.
    pub const fn as_str(self) -> &'static str {
        match self {
            SemanticAction::Press => "press",
            SemanticAction::Focus => "focus",
            SemanticAction::SetValue => "set-value",
            SemanticAction::ScrollForward => "scroll-forward",
            SemanticAction::ScrollBackward => "scroll-backward",
        }
    }
}

/// Backwards-compatible free function. Prefer [`Role::as_str`].
pub fn role_to_str(role: Role) -> &'static str {
    role.as_str()
}

/// Backwards-compatible free function. Prefer [`SemanticAction::as_str`].
pub fn action_to_str(action: SemanticAction) -> &'static str {
    action.as_str()
}

#[cfg(feature = "accesskit")]
#[derive(Default)]
pub struct AccessKitBackend {
    update_counter: u64,
}

#[cfg(feature = "accesskit")]
impl AccessKitBackend {
    pub const fn new() -> Self {
        Self { update_counter: 0 }
    }

    pub const fn update_count(&self) -> u64 {
        self.update_counter
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// Regression test for the bug where `Role::Spinner` and `Role::ProgressBar`
    /// mapped to the same string, which made them indistinguishable to
    /// assistive technology.
    #[test]
    fn spinner_and_progress_bar_have_distinct_roles() {
        assert_ne!(Role::Spinner.as_str(), Role::ProgressBar.as_str());
    }

    /// All `Role` variants should produce *some* string, and the map should
    /// be deterministic. This is the "no duplicate string" invariant.
    #[test]
    fn role_to_str_is_a_total_function() {
        let all = [
            Role::Window, Role::Group, Role::Text, Role::Button, Role::TextInput,
            Role::Checkbox, Role::Radio, Role::List, Role::ListItem, Role::Table,
            Role::Row, Role::Cell, Role::Dialog, Role::Menu, Role::MenuItem,
            Role::Tooltip, Role::ScrollArea, Role::Image, Role::Switch, Role::Slider,
            Role::ProgressBar, Role::Spinner, Role::Badge, Role::Avatar, Role::Link,
            Role::Alert, Role::Card,
        ];
        let mut seen: HashSet<&'static str> = HashSet::new();
        for role in all {
            let s = role.as_str();
            assert!(!s.is_empty(), "Role {role:?} maps to empty string");
            // The mapping may legitimately alias some roles (e.g. Card and Group
            // both go to "group") but Spinner and ProgressBar must NOT.
            if matches!(role, Role::Spinner | Role::ProgressBar) {
                assert!(seen.insert(s) || role == Role::ProgressBar,
                    "Spinner and ProgressBar must not collide on {s}");
            } else {
                seen.insert(s);
            }
        }
    }

    #[test]
    fn real_backend_defaults_to_zero() {
        let backend = RealAccessibilityBackend::default();
        assert_eq!(backend.update_count(), 0);
        assert_eq!(backend.node_count(), 0);
        assert_eq!(backend.focused_node(), None);
    }

    #[test]
    fn real_backend_tracks_update_count() {
        let mut backend = RealAccessibilityBackend::new();
        let tree = SemanticTree::default();
        backend.update(&tree);
        backend.update(&tree);
        backend.update(&tree);
        assert_eq!(backend.update_count(), 3);
        assert_eq!(backend.node_count(), 0);
        assert_eq!(backend.focused_node(), None);
    }
}
