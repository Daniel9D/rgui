use crate::{
    AvatarSpec, BadgeSpec, BadgeVariant, Element, ElementKind, IconSpec, ImageFit,
    ImageSpec, WidgetKind, WidgetSpec,
};

/// Convenience alias for [`Element::text`]. Creates a plain text node.
pub fn text(value: impl Into<String>) -> Element {
    Element::text(value)
}

/// Creates an icon element. The icon `name` is used by the renderer to look
/// up the glyph in the active icon set. The name is also set as the
/// accessibility label so screen readers can describe the icon.
pub fn icon(name: impl Into<String>) -> Element {
    let name: String = name.into();
    Element::new(ElementKind::Widget(WidgetKind::Icon))
        .widget_spec(WidgetSpec::Icon(IconSpec { name: name.clone() }))
        .aria_label(name)
}

pub fn divider() -> Element {
    Element::new(ElementKind::Widget(WidgetKind::Divider)).widget_spec(WidgetSpec::Divider)
}

/// Creates an image element. The `src` is a URL or resource identifier used
/// by the renderer to load and display the image.
pub fn image(src: impl Into<String>) -> Element {
    let src: String = src.into();
    Element::new(ElementKind::Widget(WidgetKind::Image))
        .widget_spec(WidgetSpec::Image(ImageSpec {
            src: Some(src),
            alt: None,
            fit: ImageFit::default(),
        }))
}

/// Creates a badge element displaying a small status label.
pub fn badge(text: impl Into<String>) -> Element {
    Element::new(ElementKind::Widget(WidgetKind::Badge))
        .widget_spec(WidgetSpec::Badge(BadgeSpec {
            text: text.into(),
            variant: BadgeVariant::default(),
        }))
}

/// Creates an avatar element. Displays an image if `src` is provided,
/// otherwise falls back to `initials` text.
pub fn avatar() -> Element {
    Element::new(ElementKind::Widget(WidgetKind::Avatar))
        .widget_spec(WidgetSpec::Avatar(AvatarSpec::default()))
}
