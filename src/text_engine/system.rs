use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use crate::core::{FontStyle, FontWeight, ShapedGlyph, ShapedText, Size, TextEngine, TextSpec};

use super::{CosmicTextEngine, TextGlyphPosition, TextLayout};
use crate::text_engine::layout::{TextGlyphRun, TextLine};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TextShapeKey {
    pub text_hash: u64,
    pub font_stack_hash: u64,
    pub size_bits: u32,
    pub width_bits: u32,
    pub weight: FontWeight,
    pub style: FontStyle,
}

impl TextShapeKey {
    pub fn new(text: &str, width: f32, weight: FontWeight, style: FontStyle) -> Self {
        Self::new_with_size(text, width, 14.0, weight, style)
    }

    pub fn new_with_size(
        text: &str,
        width: f32,
        size: f32,
        weight: FontWeight,
        style: FontStyle,
    ) -> Self {
        Self {
            text_hash: stable_hash(text),
            font_stack_hash: stable_hash("system-ui"),
            size_bits: size.to_bits(),
            width_bits: width.to_bits(),
            weight,
            style,
        }
    }
}

pub struct TextSystem {
    engine: CosmicTextEngine,
    font_system: glyphon::cosmic_text::FontSystem,
    shape_cache: HashMap<TextShapeKey, ShapedText>,
    layout_cache: HashMap<TextShapeKey, TextLayout>,
}

impl Default for TextSystem {
    fn default() -> Self {
        Self {
            engine: CosmicTextEngine::default(),
            font_system: glyphon::cosmic_text::FontSystem::new(),
            shape_cache: HashMap::new(),
            layout_cache: HashMap::new(),
        }
    }
}

impl TextSystem {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn measure(
        &mut self,
        text: &str,
        size: f32,
        weight: FontWeight,
        style: FontStyle,
        max_width: f32,
    ) -> TextLayout {
        let font_px = size.max(1.0);
        let width_limit = max_width.max(font_px);
        let key = TextShapeKey::new_with_size(text, width_limit, font_px, weight, style);
        if let Some(layout) = self.layout_cache.get(&key) {
            return layout.clone();
        }

        let layout = if let Some(real) =
            self.layout_with_cosmic(text, font_px, weight, style, width_limit)
        {
            real
        } else {
            self.measure_estimated(text, font_px, weight, style, width_limit)
        };

        self.layout_cache.insert(key, layout.clone());
        layout
    }

    pub fn measure_wrapped(
        &mut self,
        text: &str,
        size: f32,
        weight: FontWeight,
        style: FontStyle,
        width: f32,
    ) -> TextLayout {
        self.measure(text, size, weight, style, width)
    }

    pub fn measure_intrinsic(
        &mut self,
        text: &str,
        size: f32,
        weight: FontWeight,
        style: FontStyle,
    ) -> TextLayout {
        let font_px = size.max(1.0);
        let estimated_width = (text.chars().count().max(1) as f32 * font_px).max(font_px);
        self.measure(text, size, weight, style, estimated_width)
    }

    fn layout_with_cosmic(
        &mut self,
        text: &str,
        font_px: f32,
        weight: FontWeight,
        style: FontStyle,
        max_width: f32,
    ) -> Option<TextLayout> {
        use glyphon::cosmic_text::{
            Attrs, Buffer, Family, Metrics, Shaping, Weight as CosmicWeight, Wrap,
        };

        let line_height = (font_px * 1.2).ceil();
        let metrics = Metrics::new(font_px, line_height);
        let mut buffer = Buffer::new(&mut self.font_system, metrics);

        let cosmic_weight = match weight {
            FontWeight::Thin => CosmicWeight::THIN,
            FontWeight::ExtraLight => CosmicWeight::EXTRA_LIGHT,
            FontWeight::Light => CosmicWeight::LIGHT,
            FontWeight::Normal => CosmicWeight::NORMAL,
            FontWeight::Medium => CosmicWeight::MEDIUM,
            FontWeight::Semibold => CosmicWeight::SEMIBOLD,
            FontWeight::Bold => CosmicWeight::BOLD,
            FontWeight::ExtraBold => CosmicWeight::EXTRA_BOLD,
            FontWeight::Black => CosmicWeight::BLACK,
            FontWeight::Number(n) => CosmicWeight(n as u16),
        };

        let attrs = Attrs::new().family(Family::SansSerif).weight(cosmic_weight);

        if style == FontStyle::Italic {
            // cosmic-text handles italic via font selection
        }

        buffer.set_size(
            &mut self.font_system,
            Some(max_width),
            Some(line_height * 50.0),
        );
        buffer.set_text(&mut self.font_system, text, &attrs, Shaping::Advanced, None);
        buffer.set_wrap(&mut self.font_system, Wrap::Word);
        buffer.shape_until_scroll(&mut self.font_system, false);

        let layout_runs: Vec<_> = buffer.layout_runs().collect();
        if layout_runs.is_empty() {
            return None;
        }

        let glyph_count: usize = layout_runs.iter().map(|run| run.glyphs.len()).sum();

        let mut lines = Vec::new();
        let mut glyph_runs = Vec::new();
        let mut glyph_start = 0usize;
        for (line_index, run) in layout_runs.iter().enumerate() {
            let line_y = run.line_y;
            let line_top = run.line_top;
            let run_width: f32 = run.glyphs.iter().map(|g| g.w).sum();
            let baseline = line_y - line_top;
            let glyph_positions = run
                .glyphs
                .iter()
                .map(|glyph| TextGlyphPosition {
                    byte_offset: glyph.start,
                    advance_x: glyph.x + glyph.w * 0.5,
                })
                .collect();
            lines.push(TextLine {
                range: 0..text.len(),
                x: 0.0,
                y: line_y,
                width: run_width,
                baseline,
                glyph_positions,
            });
            let glyph_end = glyph_start + run.glyphs.len();
            glyph_runs.push(TextGlyphRun {
                line_index,
                glyph_start,
                glyph_end,
                x: 0.0,
                y: line_y,
            });
            glyph_start = glyph_end;
        }

        let total_width = lines
            .iter()
            .map(|l| l.width)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(font_px);
        let total_height = layout_runs
            .last()
            .map(|run| run.line_y + line_height)
            .unwrap_or(line_height);
        let baseline = lines.first().map(|l| l.baseline).unwrap_or(font_px * 0.8);

        Some(TextLayout {
            text: text.to_string(),
            font_px,
            width: total_width.min(max_width).max(font_px),
            height: total_height,
            baseline,
            line_height,
            glyph_count,
            lines,
            glyph_runs,
        })
    }

    fn measure_estimated(
        &self,
        text: &str,
        font_px: f32,
        weight: FontWeight,
        style: FontStyle,
        max_width: f32,
    ) -> TextLayout {
        let glyph_count = text.chars().count();
        let weight_factor = if weight == FontWeight::Bold {
            0.64
        } else {
            0.58
        };
        let style_factor = if style == FontStyle::Italic {
            1.04
        } else {
            1.0
        };
        let advance = font_px * weight_factor * style_factor;
        let raw_width = glyph_count as f32 * advance;
        let line_height = (font_px * 1.2).ceil();
        let layout = TextLayout {
            text: text.to_string(),
            font_px,
            width: raw_width.min(max_width).max(font_px.min(max_width)),
            height: line_height,
            baseline: (font_px * 0.8).ceil(),
            line_height,
            glyph_count,
            lines: vec![TextLine {
                range: 0..text.len(),
                x: 0.0,
                y: 0.0,
                width: raw_width.min(max_width).max(font_px.min(max_width)),
                baseline: (font_px * 0.8).ceil(),
                glyph_positions: text
                    .char_indices()
                    .enumerate()
                    .map(|(index, (byte_offset, _))| TextGlyphPosition {
                        byte_offset,
                        advance_x: index as f32 * advance + advance * 0.5,
                    })
                    .collect(),
            }],
            glyph_runs: Vec::new(),
        };
        layout
    }

    pub fn shape(
        &mut self,
        text: &str,
        width: f32,
        weight: FontWeight,
        style: FontStyle,
    ) -> ShapedText {
        let key = TextShapeKey::new(text, width, weight, style);
        if let Some(shaped) = self.shape_cache.get(&key) {
            return shaped.clone();
        }

        let shaped = self.engine.shape(
            &TextSpec {
                text: text.to_string(),
            },
            Size::new(width, f32::INFINITY),
        );
        self.shape_cache.insert(key, shaped.clone());
        shaped
    }

    pub fn shape_with_size(
        &mut self,
        text: &str,
        width: f32,
        size: f32,
        weight: FontWeight,
        style: FontStyle,
    ) -> ShapedText {
        let key = TextShapeKey::new_with_size(text, width, size, weight, style);
        if let Some(shaped) = self.shape_cache.get(&key) {
            return shaped.clone();
        }

        let shaped = if let Some(layout) = self.layout_with_cosmic(text, size, weight, style, width)
        {
            self.shaped_from_layout(&layout)
        } else if let Some(real) = self.shape_with_cosmic(text, width, size, weight, style) {
            real
        } else {
            let mut shaped = self.engine.shape(
                &TextSpec {
                    text: text.to_string(),
                },
                Size::new(width, f32::INFINITY),
            );
            let scale = (size / 14.0).max(0.5);
            shaped.size = Size::new(shaped.size.width * scale, shaped.size.height * scale);
            shaped.baseline *= scale;
            shaped
        };

        self.shape_cache.insert(key, shaped.clone());
        shaped
    }

    fn shaped_from_layout(&self, layout: &TextLayout) -> ShapedText {
        ShapedText {
            size: layout.size(),
            baseline: layout.baseline,
            glyph_count: layout.glyph_count,
            glyphs: Vec::new(),
        }
    }

    fn shape_with_cosmic(
        &mut self,
        text: &str,
        max_width: f32,
        font_px: f32,
        weight: FontWeight,
        style: FontStyle,
    ) -> Option<ShapedText> {
        use glyphon::cosmic_text::{Attrs, Buffer, Family, Metrics, Shaping, Wrap};
        let _ = style; // font style handled by font selection in future

        let mut font_system = glyphon::cosmic_text::FontSystem::new();
        let line_height = (font_px * 1.2).ceil();
        let metrics = Metrics::new(font_px, line_height);
        let mut buffer = Buffer::new(&mut font_system, metrics);

        let cosmic_weight = match weight {
            FontWeight::Thin => glyphon::cosmic_text::Weight::THIN,
            FontWeight::ExtraLight => glyphon::cosmic_text::Weight::EXTRA_LIGHT,
            FontWeight::Light => glyphon::cosmic_text::Weight::LIGHT,
            FontWeight::Normal => glyphon::cosmic_text::Weight::NORMAL,
            FontWeight::Medium => glyphon::cosmic_text::Weight::MEDIUM,
            FontWeight::Semibold => glyphon::cosmic_text::Weight::SEMIBOLD,
            FontWeight::Bold => glyphon::cosmic_text::Weight::BOLD,
            FontWeight::ExtraBold => glyphon::cosmic_text::Weight::EXTRA_BOLD,
            FontWeight::Black => glyphon::cosmic_text::Weight::BLACK,
            FontWeight::Number(n) => glyphon::cosmic_text::Weight(n as u16),
        };

        let attrs = Attrs::new().family(Family::SansSerif).weight(cosmic_weight);
        buffer.set_size(&mut font_system, Some(max_width), Some(line_height * 50.0));
        buffer.set_text(&mut font_system, text, &attrs, Shaping::Advanced, None);
        buffer.set_wrap(&mut font_system, Wrap::Word);
        buffer.shape_until_scroll(&mut font_system, false);

        let layout_runs: Vec<_> = buffer.layout_runs().collect();
        if layout_runs.is_empty() {
            return None;
        }

        let mut glyphs = Vec::new();
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = 0.0f32;

        for run in &layout_runs {
            let font_id = 1u64; // system-ui font
            for glyph in run.glyphs.iter() {
                let gx = glyph.x;
                let gy = run.line_y;
                let gw = glyph.w;
                let gh = line_height;
                let adv = glyph.w;
                min_x = min_x.min(gx);
                max_x = max_x.max(gx + gw);
                max_y = max_y.max(gy + gh);

                glyphs.push(ShapedGlyph {
                    font_id,
                    glyph_id: glyph.glyph_id as u32,
                    x: gx,
                    y: gy,
                    width: gw,
                    height: gh,
                    advance: adv,
                });
            }
        }

        let total_width = (max_x - min_x).max(font_px);
        let total_height = max_y;
        let baseline = layout_runs
            .first()
            .map(|r| r.line_y - r.line_top)
            .unwrap_or(font_px * 0.8);

        Some(ShapedText {
            size: Size::new(total_width, total_height),
            baseline,
            glyph_count: glyphs.len(),
            glyphs,
        })
    }

    pub fn shape_cache_len(&self) -> usize {
        self.shape_cache.len()
    }
}

fn stable_hash(value: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}
