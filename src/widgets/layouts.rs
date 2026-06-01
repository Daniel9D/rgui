use crate::{
    CardSpec, Element, ElementKind, Overflow, PrimitiveKind, WidgetKind, WidgetSpec,
};

/// Creates a card element — an elevated/bordered container for grouping content.
///
/// Sizing: by default the card **hugs its children** (auto-sized as a vertical
/// flex container). To force a fixed size, set an explicit `width()` and/or
/// `height()` on the element — those override the hug for that axis.
pub fn card() -> Element {
    Element::new(ElementKind::Widget(WidgetKind::Card))
        .widget_spec(WidgetSpec::Card(CardSpec::default()))
}

/// Creates a vertical scroll area. Children stack in a column; content that
/// exceeds the available height scrolls. To constrain the visible viewport,
/// set `width()` and/or `height()` on the element.
pub fn scroll_area() -> Element {
    Element::new(ElementKind::Primitive(PrimitiveKind::ScrollArea)).overflow(Overflow::Scroll)
}

