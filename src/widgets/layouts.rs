use crate::{CardSpec, Element, ElementKind, WidgetKind, WidgetSpec};

/// Creates a card element — an elevated/bordered container for grouping content.
pub fn card() -> Element {
    Element::new(ElementKind::Widget(WidgetKind::Card))
        .widget_spec(WidgetSpec::Card(CardSpec::default()))
}
