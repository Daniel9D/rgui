use crate::core::{Element, ElementKind, PrimitiveKind};
use crate::widgets::{button, input as input_widget};

pub fn parse_element(input: &str) -> Result<Element, String> {
    if input.trim().is_empty() {
        return Err("html adapter input is empty".to_string());
    }
    let trimmed = input.trim();
    if trimmed.starts_with("<button") {
        return Ok(button(inner_text(trimmed)));
    }
    if trimmed.starts_with("<input") {
        return Ok(input_widget());
    }
    if trimmed.starts_with("<div") {
        return Ok(Element::new(ElementKind::Primitive(PrimitiveKind::Column))
            .child(Element::text(inner_text(trimmed))));
    }
    Ok(Element::text(trimmed))
}

fn inner_text(input: &str) -> String {
    let Some(start) = input.find('>') else {
        return input.to_string();
    };
    let Some(end) = input.rfind('<') else {
        return input[start + 1..].to_string();
    };
    input[start + 1..end].trim().to_string()
}
