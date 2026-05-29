use crate::{
    Align, Edge, ElementKey, FontWeight, InputSpec, Length, NodeId, Overflow, SelectOption,
    SelectSpec, Style, TextStyle, TextareaSpec, VariantId, WidgetSpec,
};

/// Default (uncontrolled) heading font size in logical pixels.
pub const HEADING_SIZE_PX: f32 = 24.0;

#[derive(Clone, Debug, PartialEq)]
pub struct Element {
    pub key: Option<ElementKey>,
    pub kind: ElementKind,
    pub widget_spec: Option<WidgetSpec>,
    pub children: Vec<Element>,
    pub style: Style,
    pub variant: Option<VariantId>,
    /// Controlled checked state — overrides internal toggle state on every render.
    pub checked: Option<bool>,
    /// Uncontrolled initial checked state — only seeds state the first time the
    /// node is mounted; does not override state on subsequent renders.
    pub default_checked: Option<bool>,
    pub semantic: Semantic,
    pub event_handlers: EventHandlers,
    pub overlay: Option<Box<Element>>,
    pub open: bool,
}

impl Element {
    pub fn new(kind: ElementKind) -> Self {
        Self {
            key: None,
            kind,
            widget_spec: None,
            children: Vec::new(),
            style: Style::default(),
            variant: None,
            checked: None,
            default_checked: None,
            semantic: Semantic::default(),
            event_handlers: EventHandlers::default(),
            overlay: None,
            open: false,
        }
    }

    #[must_use]
    pub fn row() -> Self {
        Self::new(ElementKind::Primitive(PrimitiveKind::Row))
    }

    #[must_use]
    pub fn column() -> Self {
        Self::new(ElementKind::Primitive(PrimitiveKind::Column))
    }

    #[must_use]
    pub fn grid() -> Self {
        Self::new(ElementKind::Primitive(PrimitiveKind::Grid))
    }

    #[must_use]
    pub fn stack() -> Self {
        Self::new(ElementKind::Primitive(PrimitiveKind::Stack))
    }

    #[must_use]
    pub fn absolute() -> Self {
        Self::new(ElementKind::Primitive(PrimitiveKind::Absolute))
    }

    #[must_use]
    pub fn text(value: impl Into<String>) -> Self {
        Self::new(ElementKind::Text(TextSpec { text: value.into() }))
    }

    #[must_use]
    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(ElementKey::new(key));
        self
    }

    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    #[must_use]
    pub fn padding(mut self, value: f32) -> Self {
        self.style.padding = Some(Edge::all(Length::Px(value)));
        self
    }

    #[must_use]
    pub fn gap(mut self, value: f32) -> Self {
        self.style.gap = Some(Length::Px(value));
        self
    }

    #[must_use]
    pub fn align_center(mut self) -> Self {
        self.style.align_items = Some(Align::Center);
        self
    }

    #[must_use]
    pub fn justify_center(mut self) -> Self {
        self.style.justify_content = Some(crate::Justify::Center);
        self
    }

    #[must_use]
    pub fn justify_between(mut self) -> Self {
        self.style.justify_content = Some(crate::Justify::SpaceBetween);
        self
    }

    #[must_use]
    pub fn width(mut self, value: impl Into<Length>) -> Self {
        self.style.width = Some(value.into());
        self
    }

    #[must_use]
    pub fn height(mut self, value: impl Into<Length>) -> Self {
        self.style.height = Some(value.into());
        self
    }

    #[must_use]
    pub fn overflow(mut self, value: Overflow) -> Self {
        self.style.overflow_x = Some(value);
        self.style.overflow_y = Some(value);
        self
    }

    #[must_use]
    pub fn z_index(mut self, value: i32) -> Self {
        self.style.z_index = Some(value);
        self
    }

    #[must_use]
    pub fn heading(mut self) -> Self {
        let mut text = self.style.text.unwrap_or_else(TextStyle::default);
        text.size = Length::Px(HEADING_SIZE_PX);
        text.weight = FontWeight::Bold;
        self.style.text = Some(text);
        self
    }

    /// Set the **controlled** checked state. This overrides internal toggle
    /// state on every render, making the widget fully controlled.
    #[must_use]
    pub fn checked(mut self, value: bool) -> Self {
        self.checked = Some(value);
        self
    }

    /// Set the **uncontrolled** initial checked state. This seeds the internal
    /// toggle state only on first mount and is ignored on subsequent renders.
    #[must_use]
    pub fn default_checked(mut self, value: bool) -> Self {
        self.default_checked = Some(value);
        self
    }

    #[must_use]
    pub fn widget_spec(mut self, spec: WidgetSpec) -> Self {
        self.widget_spec = Some(spec);
        self
    }

    #[must_use]
    pub fn label(mut self, value: impl Into<String>) -> Self {
        let value = value.into();
        self.semantic.label = Some(value.clone());
        if let Some(ref mut spec) = self.widget_spec {
            match spec {
                WidgetSpec::Button(bs) => bs.label = Some(value),
                WidgetSpec::Checkbox(cs) => cs.label = Some(value),
                WidgetSpec::Radio(rs) => rs.label = Some(value),
                _ => {}
            }
        }
        self
    }

    #[must_use]
    pub fn alt(mut self, value: impl Into<String>) -> Self {
        let value = value.into();
        if let Some(WidgetSpec::Image(spec)) = self.widget_spec.as_mut() {
            spec.alt = Some(value);
        }
        self
    }

    #[must_use]
    pub fn placeholder(mut self, value: impl Into<String>) -> Self {
        let value = value.into();
        if let Some(ref mut spec) = self.widget_spec {
            match spec {
                WidgetSpec::Input(InputSpec { placeholder, .. })
                | WidgetSpec::Textarea(TextareaSpec { placeholder, .. })
                | WidgetSpec::Select(SelectSpec { placeholder, .. }) => {
                    *placeholder = Some(value);
                }
                _ => {}
            }
        }
        self
    }

    #[must_use]
    pub fn default_value(mut self, value: impl Into<String>) -> Self {
        let value = value.into();
        if let Some(ref mut spec) = self.widget_spec {
            match spec {
                WidgetSpec::Input(InputSpec { default_value, .. })
                | WidgetSpec::Textarea(TextareaSpec { default_value, .. }) => {
                    *default_value = Some(value);
                }
                WidgetSpec::Select(SelectSpec { default_value, .. }) => {
                    *default_value = Some(value);
                }
                _ => {}
            }
        }
        self
    }

    #[must_use]
    pub fn options(mut self, options: impl IntoIterator<Item = SelectOption>) -> Self {
        if let Some(WidgetSpec::Select(spec)) = self.widget_spec.as_mut() {
            spec.options = options.into_iter().collect();
        }
        self
    }

    #[must_use]
    pub fn default_selected_index(mut self, index: usize) -> Self {
        if let Some(ref mut spec) = self.widget_spec {
            match spec {
                WidgetSpec::Select(select) => select.selected_index = Some(index),
                WidgetSpec::List(list) => list.selected_index = Some(index),
                _ => {}
            }
        }
        self
    }

    #[must_use]
    pub fn tabs(mut self, tabs: impl IntoIterator<Item = impl Into<String>>) -> Self {
        if let Some(WidgetSpec::Tabs(spec)) = self.widget_spec.as_mut() {
            spec.tabs = tabs.into_iter().map(Into::into).collect();
        }
        self
    }

    #[must_use]
    pub fn default_active_index(mut self, index: usize) -> Self {
        if let Some(WidgetSpec::Tabs(spec)) = self.widget_spec.as_mut() {
            spec.active_index = Some(index);
        }
        self
    }

    #[must_use]
    pub fn items<T>(mut self, items: impl IntoIterator<Item = T>) -> Self
    where
        T: IntoCollectionItem,
    {
        match self.widget_spec.as_mut() {
            Some(WidgetSpec::Tree(spec)) => {
                spec.items = items
                    .into_iter()
                    .filter_map(|item| match item.into_collection_item() {
                        CollectionItem::Tree(item) => Some(item),
                        _ => None,
                    })
                    .collect();
            }
            Some(WidgetSpec::List(spec)) => {
                spec.items = items
                    .into_iter()
                    .filter_map(|item| match item.into_collection_item() {
                        CollectionItem::List(item) => Some(item),
                        _ => None,
                    })
                    .collect();
            }
            Some(WidgetSpec::Menu(_)) => {
                self.children = items
                    .into_iter()
                    .filter_map(|item| match item.into_collection_item() {
                        CollectionItem::Menu(item) => Some(item),
                        _ => None,
                    })
                    .collect();
            }
            _ => {}
        }
        self
    }

    #[must_use]
    pub fn columns(mut self, columns: impl IntoIterator<Item = impl Into<String>>) -> Self {
        if let Some(WidgetSpec::Table(spec)) = self.widget_spec.as_mut() {
            spec.columns = columns.into_iter().map(Into::into).collect();
        }
        self
    }

    #[must_use]
    pub fn rows<const N: usize, R, S>(mut self, rows: R) -> Self
    where
        R: IntoIterator<Item = [S; N]>,
        S: Into<String>,
    {
        if let Some(WidgetSpec::Table(spec)) = self.widget_spec.as_mut() {
            spec.rows = rows
                .into_iter()
                .map(|row| row.into_iter().map(Into::into).collect())
                .collect();
        }
        self
    }

    #[must_use]
    pub fn default_selected_row(mut self, index: usize) -> Self {
        if let Some(WidgetSpec::Table(spec)) = self.widget_spec.as_mut() {
            spec.selected_row = Some(index);
        }
        self
    }

    #[must_use]
    pub fn styles(
        mut self,
        configure: impl FnOnce(&mut crate::widgets::SelectStylesBuilder<'_>),
    ) -> Self {
        if let Some(WidgetSpec::Select(spec)) = self.widget_spec.as_mut() {
            let mut builder = crate::widgets::SelectStylesBuilder::new(&mut spec.styles);
            configure(&mut builder);
            if let Some(trigger) = spec.styles.trigger.clone() {
                self.style = self.style.merge_over(trigger);
            }
        }
        self
    }

    #[must_use]
    pub fn aria_label(mut self, value: impl Into<String>) -> Self {
        let value = value.into();
        self.semantic.label = Some(value.clone());
        if let Some(ref mut spec) = self.widget_spec {
            if let WidgetSpec::Input(input) = spec {
                input.aria_label = Some(value);
            }
        }
        self
    }

    #[must_use]
    pub fn primary(mut self) -> Self {
        self.variant = Some(VariantId::new("primary"));
        self
    }

    #[must_use]
    pub fn variant(mut self, variant: impl Into<String>) -> Self {
        self.variant = Some(VariantId::new(variant));
        self
    }

    #[must_use]
    pub fn on_click(mut self, action: impl Into<String>) -> Self {
        self.event_handlers.on_click_action = Some(action.into());
        self
    }

    #[must_use]
    pub fn draggable(mut self, payload: impl Into<String>) -> Self {
        self.event_handlers.draggable_payload = Some(payload.into());
        self
    }

    #[must_use]
    pub fn popover(mut self, overlay: Element) -> Self {
        self.overlay = Some(Box::new(overlay));
        self
    }

    #[must_use]
    pub fn context_menu(mut self, menu: Element) -> Self {
        self.overlay = Some(Box::new(menu.open(false)));
        self
    }

    #[must_use]
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    #[must_use]
    pub fn child(mut self, child: Element) -> Self {
        self.children.push(child);
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ElementKind {
    Primitive(PrimitiveKind),
    Widget(crate::WidgetKind),
    Component(NodeId),
    Canvas(CanvasSpec),
    Text(TextSpec),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrimitiveKind {
    Row,
    Column,
    Grid,
    Stack,
    Absolute,
    ScrollArea,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CanvasSpec {
    pub name: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextSpec {
    pub text: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct EventHandlers {
    pub pointer_down: bool,
    pub key_down: bool,
    pub on_click_action: Option<String>,
    pub draggable_payload: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Semantic {
    pub role: Option<String>,
    pub label: Option<String>,
}

pub enum CollectionItem {
    Tree(crate::TreeItemSpec),
    List(String),
    Menu(Element),
}

pub trait IntoCollectionItem {
    fn into_collection_item(self) -> CollectionItem;
}

impl IntoCollectionItem for crate::TreeItemSpec {
    fn into_collection_item(self) -> CollectionItem {
        CollectionItem::Tree(self)
    }
}

impl IntoCollectionItem for &str {
    fn into_collection_item(self) -> CollectionItem {
        CollectionItem::List(self.to_string())
    }
}

impl IntoCollectionItem for String {
    fn into_collection_item(self) -> CollectionItem {
        CollectionItem::List(self)
    }
}

impl IntoCollectionItem for Element {
    fn into_collection_item(self) -> CollectionItem {
        CollectionItem::Menu(self)
    }
}
