use crate::core::{Element, Theme};

pub fn empty_theme_document() -> Theme {
    Theme::light()
}

pub fn text_document(value: impl Into<String>) -> Element {
    Element::text(value)
}
