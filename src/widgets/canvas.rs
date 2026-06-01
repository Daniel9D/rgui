use crate::core::{CanvasSpec, Element, ElementKind};

/// Creates a named canvas element. The `name` is used by the renderer to look
/// up the paint callback; it must be non-empty.
pub fn canvas(name: impl Into<String>) -> Element {
    Element::new(ElementKind::Canvas(CanvasSpec { name: name.into() }))
}
