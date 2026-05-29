use crate::{
    Element, ElementKind, ListSpec, MenuItemSpec, MenuSpec, TableSpec, TabsSpec, TreeItemSpec,
    TreeSpec, WidgetKind, WidgetSpec,
};

pub fn tabs() -> Element {
    Element::new(ElementKind::Widget(WidgetKind::Tabs))
        .widget_spec(WidgetSpec::Tabs(TabsSpec::default()))
}

pub fn tab(label: impl Into<String>) -> Element {
    Element::text(label)
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

pub fn tree() -> Element {
    Element::new(ElementKind::Widget(WidgetKind::Tree))
        .widget_spec(WidgetSpec::Tree(TreeSpec::default()))
}

pub fn table() -> Element {
    Element::new(ElementKind::Widget(WidgetKind::Table))
        .widget_spec(WidgetSpec::Table(TableSpec::default()))
}

pub fn list() -> Element {
    Element::new(ElementKind::Widget(WidgetKind::List))
        .widget_spec(WidgetSpec::List(ListSpec::default()))
}

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
/// Use [`Element::on_click`] to attach an action and [`Element::variant`]
/// or style overrides to control appearance.
pub fn menu_item(label: impl Into<String>) -> Element {
    let label = label.into();
    Element::new(ElementKind::Widget(WidgetKind::Menu))
        .widget_spec(WidgetSpec::Menu(MenuSpec {
            items: vec![MenuItemSpec {
                label: label.clone(),
                ..Default::default()
            }],
        }))
        .child(Element::text(label))
}
