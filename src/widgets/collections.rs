use crate::{
    Element, ElementKind, ListSpec, MenuItemSpec, MenuSpec, TableSpec, TabsSpec, TreeItemSpec,
    TreeSpec, WidgetKind, WidgetSpec,
};

/// Creates a tabs container. Use `.tabs(["…", "…"])` to set the
/// tab labels (the source of truth for both the strip and the
/// active child) and `.default_active_index(n)` to seed the
/// initial selection. Add children via `.child(...)` for each
/// tab's content.
pub fn tabs() -> Element {
    Element::new(ElementKind::Widget(WidgetKind::Tabs))
        .widget_spec(WidgetSpec::Tabs(TabsSpec::default()))
}

pub fn tree_item(label: impl Into<String>) -> TreeItemSpec {
    TreeItemSpec {
        label: label.into(),
        expanded: false,
        children: Vec::new(),
    }
}

impl TreeItemSpec {
    #[must_use]
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    #[must_use]
    pub fn child(mut self, child: TreeItemSpec) -> Self {
        self.children.push(child);
        self
    }
}

/// Creates a tree element. Use `.items([tree_item("a"), …])` to seed
/// the tree; chain `.expanded(true)` and `.child(item)` on each
/// `tree_item` to nest and expand.
pub fn tree() -> Element {
    Element::new(ElementKind::Widget(WidgetKind::Tree))
        .widget_spec(WidgetSpec::Tree(TreeSpec::default()))
}

/// Creates a table element. Use `.columns(["Name", "Status"])`
/// to set the column headers, `.rows([["a", "b"], …])` for the
/// body, and `.default_selected_row(n)` to pre-select a row.
pub fn table() -> Element {
    Element::new(ElementKind::Widget(WidgetKind::Table))
        .widget_spec(WidgetSpec::Table(TableSpec::default()))
}

/// Creates a list element. Use `.items(["a", "b", …])` to set the
/// entries and `.default_selected_index(n)` to pre-select.
pub fn list() -> Element {
    Element::new(ElementKind::Widget(WidgetKind::List))
        .widget_spec(WidgetSpec::List(ListSpec::default()))
}

/// Creates a menu (dropdown / popup) container. Children must be
/// `menu_item(...)` elements. A menu does not have an open state of
/// its own — the parent element decides when to show it.
pub fn menu() -> Element {
    Element::new(ElementKind::Widget(WidgetKind::Menu))
        .widget_spec(WidgetSpec::Menu(MenuSpec::default()))
}

/// Creates a context menu element. Unlike a regular menu, the context menu
/// starts closed (`open(false)`) and is intended to be shown on right-click
/// or a long-press trigger via [`Element::context_menu`].
pub fn context_menu() -> Element {
    menu().open(false)
}

/// Creates a single menu item element backed by a [`MenuItemSpec`].
///
/// Use [`Element::on_click`] to attach an action, [`Element::label`] to
/// change the label, and `.disabled(true)` to disable it.
pub fn menu_item(label: impl Into<String>) -> Element {
    let label = label.into();
    Element::new(ElementKind::Widget(WidgetKind::MenuItem))
        .widget_spec(WidgetSpec::MenuItem(MenuItemSpec {
            label,
            ..Default::default()
        }))
}
