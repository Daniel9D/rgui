use crate::{Element, ElementKind, ModalSpec, PopoverSpec, TooltipSpec, WidgetKind, WidgetSpec};

pub fn modal() -> Element {
    Element::new(ElementKind::Widget(WidgetKind::Modal))
        .widget_spec(WidgetSpec::Modal(ModalSpec::default()))
}

pub fn popover() -> Element {
    Element::new(ElementKind::Widget(WidgetKind::Popover))
        .widget_spec(WidgetSpec::Popover(PopoverSpec::default()))
}

pub fn tooltip() -> Element {
    Element::new(ElementKind::Widget(WidgetKind::Tooltip))
        .widget_spec(WidgetSpec::Tooltip(TooltipSpec::default()))
}
