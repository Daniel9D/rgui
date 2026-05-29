use crate::{
    ButtonSpec, CheckboxSpec, Element, ElementKind, InputSpec, RadioSpec, SelectOption,
    SelectPartStyles, SelectSpec, SliderSpec, Style, SwitchSpec, TextareaSpec, WidgetKind, WidgetSpec,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Ghost,
    Destructive,
    Link,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputVariant {
    Filled,
    Outlined,
    Underline,
    Plain,
}

pub fn button(label: impl Into<String>) -> Element {
    let label = label.into();
    Element::new(ElementKind::Widget(WidgetKind::Button))
        .widget_spec(WidgetSpec::Button(ButtonSpec {
            label: Some(label.clone()),
            ..Default::default()
        }))
        .child(Element::text(label))
}

pub fn input() -> Element {
    Element::new(ElementKind::Widget(WidgetKind::Input))
        .widget_spec(WidgetSpec::Input(InputSpec::default()))
}

pub fn checkbox() -> Element {
    Element::new(ElementKind::Widget(WidgetKind::Checkbox))
        .widget_spec(WidgetSpec::Checkbox(CheckboxSpec::default()))
}

pub fn radio() -> Element {
    Element::new(ElementKind::Widget(WidgetKind::Radio))
        .widget_spec(WidgetSpec::Radio(RadioSpec::default()))
}

pub fn select() -> Element {
    Element::new(ElementKind::Widget(WidgetKind::Select))
        .widget_spec(WidgetSpec::Select(SelectSpec::default()))
}

pub fn option(value: impl Into<String>, label: impl Into<String>) -> SelectOption {
    SelectOption::new(value, label)
}

pub struct SelectStylesBuilder<'a> {
    pub(crate) styles: &'a mut SelectPartStyles,
}

impl<'a> SelectStylesBuilder<'a> {
    pub(crate) fn new(styles: &'a mut SelectPartStyles) -> Self {
        Self { styles }
    }

    pub fn trigger(&mut self, style: Style) -> &mut Self {
        self.styles.trigger = Some(style);
        self
    }

    pub fn popover(&mut self, style: Style) -> &mut Self {
        self.styles.popover = Some(style);
        self
    }

    pub fn list(&mut self, style: Style) -> &mut Self {
        self.styles.list = Some(style);
        self
    }

    pub fn item(&mut self, style: Style) -> &mut Self {
        self.styles.item = Some(style);
        self
    }

    pub fn item_hovered(&mut self, style: Style) -> &mut Self {
        self.styles.item_hovered = Some(style);
        self
    }

    pub fn item_selected(&mut self, style: Style) -> &mut Self {
        self.styles.item_selected = Some(style);
        self
    }

    pub fn item_disabled(&mut self, style: Style) -> &mut Self {
        self.styles.item_disabled = Some(style);
        self
    }
}

/// Converts a value into a [`SelectOption`].
///
/// Implemented for:
/// - `&str` / `String` — uses the same string for both value and label.
/// - `(value, label)` tuples of any `Into<String>` pair — allows value and
///   label to differ (e.g. for i18n or enum-backed selects).
pub trait IntoSelectOption {
    fn into_select_option(self) -> SelectOption;
}

impl IntoSelectOption for &str {
    fn into_select_option(self) -> SelectOption {
        SelectOption::new(self, self)
    }
}

impl IntoSelectOption for String {
    fn into_select_option(self) -> SelectOption {
        SelectOption::new(self.clone(), self)
    }
}

impl<V, L> IntoSelectOption for (V, L)
where
    V: Into<String>,
    L: Into<String>,
{
    fn into_select_option(self) -> SelectOption {
        SelectOption::new(self.0, self.1)
    }
}

/// Convenience constructor that builds a [`select`] pre-loaded with options.
///
/// Accepts any iterable whose items implement [`IntoSelectOption`]:
/// ```
/// // Same value and label:
/// select_options(["Alpha", "Beta", "Gamma"])
///
/// // Different value and label (e.g. enum keys vs. display strings):
/// select_options([("en", "English"), ("fr", "French")])
/// ```
pub fn select_options(options: impl IntoIterator<Item = impl IntoSelectOption>) -> Element {
    let options: Vec<SelectOption> = options
        .into_iter()
        .map(IntoSelectOption::into_select_option)
        .collect();
    select().options(options)
}

pub fn textarea() -> Element {
    Element::new(ElementKind::Widget(WidgetKind::Textarea))
        .widget_spec(WidgetSpec::Textarea(TextareaSpec::default()))
}

pub fn scroll_area() -> Element {
    Element::column().overflow(crate::core::Overflow::Scroll)
}

/// Creates a switch (toggle) element.
pub fn switch() -> Element {
    Element::new(ElementKind::Widget(WidgetKind::Switch))
        .widget_spec(WidgetSpec::Switch(SwitchSpec::default()))
}

/// Creates a slider element for selecting a numeric value within a range.
pub fn slider() -> Element {
    Element::new(ElementKind::Widget(WidgetKind::Slider))
        .widget_spec(WidgetSpec::Slider(SliderSpec::default()))
}
