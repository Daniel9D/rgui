use crate::core::{Edge, Length, Style};

pub fn css_to_style(input: &str) -> Result<Style, String> {
    if input.trim().is_empty() {
        return Err("css adapter input is empty".to_string());
    }
    let mut style = Style::default();
    for declaration in input.split(';') {
        let Some((property, value)) = declaration.split_once(':') else {
            continue;
        };
        let property = property.trim();
        let value = value.trim();
        match property {
            "padding" => {
                if let Some(px) = parse_px(value) {
                    style.padding = Some(Edge::all(Length::Px(px)));
                }
            }
            "gap" => {
                if let Some(px) = parse_px(value) {
                    style.gap = Some(Length::Px(px));
                }
            }
            "width" => {
                if let Some(px) = parse_px(value) {
                    style.width = Some(Length::Px(px));
                }
            }
            "height" => {
                if let Some(px) = parse_px(value) {
                    style.height = Some(Length::Px(px));
                }
            }
            _ => {}
        }
    }
    Ok(style)
}

fn parse_px(value: &str) -> Option<f32> {
    value.strip_suffix("px")?.trim().parse().ok()
}
