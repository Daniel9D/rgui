use crate::{Element, ElementKind, ModalSpec, PopoverSpec, TooltipSpec, WidgetKind, WidgetSpec};

/// Creates a modal element — a centered overlay that blocks the main UI.
///
/// Sizing: by default the modal **hugs its children** (auto-sized as a
/// vertical flex container) up to the root-level `max_size` cap
/// (`min(480, viewport-32)` on each axis). To force a fixed size, set an
/// explicit `width()` and/or `height()` on the element.
///
/// Overflow: defaults to `Hidden` so content past the cap is clipped.
pub fn modal() -> Element {
    Element::new(ElementKind::Widget(WidgetKind::Modal))
        .widget_spec(WidgetSpec::Modal(ModalSpec::default()))
}

/// Creates a popover element — a small floating panel anchored to a trigger.
///
/// Sizing: by default the popover **hugs its children** (auto-sized as a
/// vertical flex container) up to the root-level `max_width` cap
/// (`viewport-16`). To force a fixed size, set an explicit `width()` on the
/// element.
///
/// Overflow: defaults to `Hidden` so content past the cap is clipped.
pub fn popover() -> Element {
    Element::new(ElementKind::Widget(WidgetKind::Popover))
        .widget_spec(WidgetSpec::Popover(PopoverSpec::default()))
}

/// Creates a tooltip element — a small floating label for hover/focus hints.
///
/// Sizing: by default the tooltip **hugs its children** (auto-sized as a
/// vertical flex container) up to the root-level `max_width` cap
/// (`viewport-16`). To force a fixed size, set an explicit `width()` on the
/// element.
///
/// Overflow: defaults to `Hidden` so content past the cap is clipped.
pub fn tooltip() -> Element {
    Element::new(ElementKind::Widget(WidgetKind::Tooltip))
        .widget_spec(WidgetSpec::Tooltip(TooltipSpec::default()))
}
