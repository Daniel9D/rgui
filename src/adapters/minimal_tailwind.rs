use crate::core::{Display, Edge, Length, Style};

pub fn classes_to_style(classes: &str) -> Result<Style, String> {
    if classes.trim().is_empty() {
        return Err("tailwind adapter class list is empty".to_string());
    }
    let mut style = Style::default();
    for class in classes.split_whitespace() {
        match class {
            "flex" => style.display = Some(Display::Flex),
            "grid" => style.display = Some(Display::Grid),
            "hidden" => style.display = Some(Display::None),
            "gap-0" => style.gap = Some(Length::Px(0.0)),
            "gap-1" => style.gap = Some(Length::Px(4.0)),
            "gap-2" => style.gap = Some(Length::Px(8.0)),
            "gap-3" => style.gap = Some(Length::Px(12.0)),
            "gap-4" => style.gap = Some(Length::Px(16.0)),
            "p-0" => style.padding = Some(Edge::all(Length::Px(0.0))),
            "p-1" => style.padding = Some(Edge::all(Length::Px(4.0))),
            "p-2" => style.padding = Some(Edge::all(Length::Px(8.0))),
            "p-3" => style.padding = Some(Edge::all(Length::Px(12.0))),
            "p-4" => style.padding = Some(Edge::all(Length::Px(16.0))),
            _ => {}
        }
    }
    Ok(style)
}
