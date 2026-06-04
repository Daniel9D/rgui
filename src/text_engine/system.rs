use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use crate::core::{FontStyle, FontWeight, ShapedGlyph, ShapedText, Size, TextEngine, TextSpec};
use crate::render::wgpu::constants::{TEXT_WIDTH_HEURISTIC, TEXT_WIDTH_HEURISTIC_BOLD};
use crate::text_engine::layout::{TextGlyphRun, TextLine};

use super::{CosmicTextEngine, TextGlyphPosition, TextLayout};

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

/// Cache statistics for the text system. Used to surface hit-rate
/// and capacity info to the renderer-stats hook. Bug fix 4.8:
/// without a stats hook, a misconfigured app (e.g. one that
/// scrolls a 10k-character text field) would silently grow the
/// cache without anyone noticing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct TextCacheStats {
    pub shape_hits: u64,
    pub shape_misses: u64,
    pub layout_hits: u64,
    pub layout_misses: u64,
    pub shape_entries: usize,
    pub layout_entries: usize,
}

pub struct TextSystem {
    engine: CosmicTextEngine,
    font_system: glyphon::cosmic_text::FontSystem,
    shape_cache: HashMap<TextShapeKey, ShapedText>,
    layout_cache: HashMap<TextShapeKey, TextLayout>,
    shape_hits: u64,
    shape_misses: u64,
    layout_hits: u64,
    layout_misses: u64,
}

impl Default for TextSystem {
    fn default() -> Self {
        Self {
            engine: CosmicTextEngine::default(),
            font_system: glyphon::cosmic_text::FontSystem::new(),
            shape_cache: HashMap::new(),
            layout_cache: HashMap::new(),
            shape_hits: 0,
            shape_misses: 0,
            layout_hits: 0,
            layout_misses: 0,
        }
    }
}

impl TextSystem {
    /// Return a snapshot of the current cache statistics. Useful for
    /// observability tools and for tuning the cache size. Note that
    /// the underlying caches are plain `HashMap`s (not LRU), so
    /// `shape_entries` will grow unbounded over the lifetime of
    /// the `TextSystem`. If you see runaway growth, file a bug.
    pub fn cache_stats(&self) -> TextCacheStats {
        TextCacheStats {
            shape_hits: self.shape_hits,
            shape_misses: self.shape_misses,
            layout_hits: self.layout_hits,
            layout_misses: self.layout_misses,
            shape_entries: self.shape_cache.len(),
            layout_entries: self.layout_cache.len(),
        }
    }

    /// Phase 3 / Plan 03-03: clear both the shape and layout
    /// caches, returning `(shape_evicted, layout_evicted)`. Hit /
    /// miss counters are NOT reset — they accumulate over the
    /// lifetime of the `TextSystem` so the hit rate stays
    /// meaningful across explicit clears.
    pub fn clear_caches(&mut self) -> (usize, usize) {
        let shape_evicted = self.shape_cache.len();
        let layout_evicted = self.layout_cache.len();
        self.shape_cache.clear();
        self.layout_cache.clear();
        (shape_evicted, layout_evicted)
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
            self.layout_hits += 1;
            return layout.clone();
        }
        self.layout_misses += 1;

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
            TEXT_WIDTH_HEURISTIC_BOLD
        } else {
            TEXT_WIDTH_HEURISTIC
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
            self.shape_hits += 1;
            return shaped.clone();
        }
        self.shape_misses += 1;

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
            self.shape_hits += 1;
            return shaped.clone();
        }
        self.shape_misses += 1;

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

#[cfg(test)]
mod tests {
    use super::*;

    // Bug fix 4.8: cache_stats unit test. The shape and layout
    // caches should each track hits and misses independently.
    #[test]
    fn cache_stats_record_hits_and_misses() {
        let mut sys = TextSystem::default();
        let initial = sys.cache_stats();
        assert_eq!(initial.shape_hits, 0);
        assert_eq!(initial.shape_misses, 0);
        assert_eq!(initial.layout_hits, 0);
        assert_eq!(initial.layout_misses, 0);

        // First call: both shape and layout miss.
        let _ = sys.measure("hello", 14.0, FontWeight::Normal, FontStyle::Normal, 200.0);
        let after_first = sys.cache_stats();
        assert!(after_first.layout_misses >= 1, "first measure should miss layout cache");
        assert_eq!(after_first.layout_hits, 0);

        // Second call with the same key: layout cache hit.
        let _ = sys.measure("hello", 14.0, FontWeight::Normal, FontStyle::Normal, 200.0);
        let after_second = sys.cache_stats();
        assert!(
            after_second.layout_hits >= 1,
            "second measure with same key should hit layout cache"
        );
        assert_eq!(after_second.layout_entries, after_first.layout_entries);
    }

    // Phase 3 / Plan 03-03: clear_caches evicts the entries from
    // both caches and returns the count. The hit/miss counters
    // are NOT reset by clear_caches — they accumulate so the
    // hit rate stays meaningful across explicit clears.
    #[test]
    fn clear_caches_evicts_shape_and_layout_entries() {
        let mut sys = TextSystem::default();
        // Populate the layout cache (which also touches the
        // shape path inside `layout_with_cosmic`).
        let _ = sys.measure("a", 14.0, FontWeight::Normal, FontStyle::Normal, 100.0);
        let _ = sys.measure("b", 14.0, FontWeight::Normal, FontStyle::Normal, 100.0);
        let _ = sys.measure("c", 14.0, FontWeight::Normal, FontStyle::Normal, 100.0);
        // Also shape distinct strings so the shape cache is
        // non-empty (shape_with_size populates it directly).
        let _ = sys.shape_with_size("x", 100.0, 14.0, FontWeight::Normal, FontStyle::Normal);
        let _ = sys.shape_with_size("y", 100.0, 14.0, FontWeight::Normal, FontStyle::Normal);
        let _ = sys.shape_with_size("z", 100.0, 14.0, FontWeight::Normal, FontStyle::Normal);

        let before = sys.cache_stats();
        assert!(before.layout_entries >= 3, "expected at least 3 layout entries");
        assert!(before.shape_entries >= 3, "expected at least 3 shape entries");

        let (shape_evicted, layout_evicted) = sys.clear_caches();
        let after = sys.cache_stats();
        assert_eq!(shape_evicted, before.shape_entries);
        assert_eq!(layout_evicted, before.layout_entries);
        assert_eq!(after.shape_entries, 0);
        assert_eq!(after.layout_entries, 0);
    }

    #[test]
    fn clear_caches_preserves_hit_and_miss_counters() {
        let mut sys = TextSystem::default();
        // Generate some hits and misses.
        let _ = sys.measure("a", 14.0, FontWeight::Normal, FontStyle::Normal, 100.0);
        let _ = sys.measure("a", 14.0, FontWeight::Normal, FontStyle::Normal, 100.0);
        let _ = sys.measure("b", 14.0, FontWeight::Normal, FontStyle::Normal, 100.0);

        let before = sys.cache_stats();
        assert!(before.layout_misses >= 2, "expected at least 2 layout misses");
        assert!(before.layout_hits >= 1, "expected at least 1 layout hit");

        let _ = sys.clear_caches();
        let after = sys.cache_stats();
        assert_eq!(
            after.layout_misses, before.layout_misses,
            "clear_caches must not reset miss counters"
        );
        assert_eq!(
            after.layout_hits, before.layout_hits,
            "clear_caches must not reset hit counters"
        );
    }
}
