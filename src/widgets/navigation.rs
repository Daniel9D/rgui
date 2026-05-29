use crate::{Element, ElementKind, LinkSpec, WidgetKind, WidgetSpec};

/// Creates a link (anchor) element. Use [`Element::on_click`] to attach
/// navigation actions.
pub fn link(label: impl Into<String>) -> Element {
    let label = label.into();
    Element::new(ElementKind::Widget(WidgetKind::Link))
        .widget_spec(WidgetSpec::Link(LinkSpec {
            label: Some(label),
            ..Default::default()
        }))
}
