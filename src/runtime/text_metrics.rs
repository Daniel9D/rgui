use crate::core::{FontStyle, FontWeight, Size};
use crate::render::wgpu::constants::{TEXT_WIDTH_HEURISTIC, TEXT_WIDTH_HEURISTIC_BOLD};

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
    // Bug fix 4.1 note: `chars().count()` is O(n) and walks UTF-8.
    // The width heuristic is itself an approximation, so the
    // function is "fast and approximate" overall. The real shape cache
    // in `text_engine` is the source of truth for layout; this function
    // exists for callers that need a width estimate without paying the
    // full shaping cost. If profiling shows this is hot, replace with
    // a per-text-node cache keyed on (text, font_size, weight, style).
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
    let width_heuristic = if matches!(weight, FontWeight::Bold) {
        TEXT_WIDTH_HEURISTIC_BOLD
    } else {
        TEXT_WIDTH_HEURISTIC
    };
    let width = (glyph_count * font_size * width_heuristic * weight_scale * style_scale)
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
