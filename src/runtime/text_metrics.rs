use crate::core::{FontStyle, FontWeight, Size};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextMetrics {
    pub width: f32,
    pub height: f32,
    pub baseline: f32,
}

pub fn measure_text(
    text: &str,
    font_size: f32,
    weight: FontWeight,
    style: FontStyle,
    max_width: f32,
) -> TextMetrics {
    let glyph_count = text.chars().count() as f32;
    let weight_scale = if matches!(weight, FontWeight::Bold) {
        1.08
    } else {
        1.0
    };
    let style_scale = if matches!(style, FontStyle::Italic) {
        1.04
    } else {
        1.0
    };
    let width = (glyph_count * font_size * 0.58 * weight_scale * style_scale)
        .max(font_size)
        .min(max_width.max(font_size));
    let height = (font_size * 1.25).max(1.0);
    let baseline = font_size * 0.9;

    TextMetrics {
        width,
        height,
        baseline,
    }
}

pub fn metrics_size(metrics: TextMetrics) -> Size {
    Size::new(metrics.width, metrics.height)
}
