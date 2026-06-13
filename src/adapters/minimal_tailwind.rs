use crate::core::{Display, Edge, Length, Overflow, Style};

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
            "overflow-auto" => {
                style.overflow_x = Some(Overflow::Auto);
                style.overflow_y = Some(Overflow::Auto);
            }
            "overflow-hidden" => {
                style.overflow_x = Some(Overflow::Hidden);
                style.overflow_y = Some(Overflow::Hidden);
            }
            "overflow-visible" => {
                style.overflow_x = Some(Overflow::Visible);
                style.overflow_y = Some(Overflow::Visible);
            }
            "overflow-scroll" => {
                style.overflow_x = Some(Overflow::Scroll);
                style.overflow_y = Some(Overflow::Scroll);
            }
            "overflow-x-auto" => style.overflow_x = Some(Overflow::Auto),
            "overflow-y-auto" => style.overflow_y = Some(Overflow::Auto),
            "overflow-x-hidden" => style.overflow_x = Some(Overflow::Hidden),
            "overflow-y-hidden" => style.overflow_y = Some(Overflow::Hidden),
            "overflow-x-visible" => style.overflow_x = Some(Overflow::Visible),
            "overflow-y-visible" => style.overflow_y = Some(Overflow::Visible),
            "overflow-x-scroll" => style.overflow_x = Some(Overflow::Scroll),
            "overflow-y-scroll" => style.overflow_y = Some(Overflow::Scroll),
            _ => {}
        }
    }
    Ok(style)
}
