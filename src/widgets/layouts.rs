use crate::{CardSpec, Element, ElementKind, WidgetKind, WidgetSpec};

/// Creates a card element — an elevated/bordered container for grouping content.
///
/// Sizing: by default the card **hugs its children** (auto-sized as a vertical
/// flex container). To force a fixed size, set an explicit `width()` and/or
/// `height()` on the element — those override the hug for that axis.
pub fn card() -> Element {
    Element::new(ElementKind::Widget(WidgetKind::Card))
        .widget_spec(WidgetSpec::Card(CardSpec::default()))
}
