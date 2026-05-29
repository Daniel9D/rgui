use crate::core::{CanvasSpec, Element, ElementKind};

#[derive(Default)]
pub struct CanvasBuilder {
    name: Option<String>,
}

impl CanvasBuilder {
    /// Sets the canvas name. The name is used by the renderer to look up the
    /// canvas paint callback; it must be non-empty.
    #[must_use]
    pub fn named(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Builds the canvas element.
    ///
    /// # Panics
    /// Panics if [`named`](Self::named) was never called, because a nameless
    /// canvas cannot be located by the renderer.
    #[must_use]
    pub fn build(self) -> Element {
        let name = self
            .name
            .filter(|n| !n.is_empty())
            .expect("CanvasBuilder::build() requires a non-empty name; call .named(\"my_canvas\") first");
        Element::new(ElementKind::Canvas(CanvasSpec { name }))
    }
}

pub fn canvas() -> CanvasBuilder {
    CanvasBuilder::default()
}
