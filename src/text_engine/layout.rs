use crate::core::{Point, Rect, Size};
use std::ops::Range;

#[derive(Clone, Debug, PartialEq)]
pub struct TextLine {
    pub range: Range<usize>,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub baseline: f32,
    pub glyph_positions: Vec<TextGlyphPosition>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextGlyphPosition {
    pub byte_offset: usize,
    pub advance_x: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextGlyphRun {
    pub line_index: usize,
    pub glyph_start: usize,
    pub glyph_end: usize,
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextLayout {
    pub text: String,
    pub font_px: f32,
    pub width: f32,
    pub height: f32,
    pub baseline: f32,
    pub line_height: f32,
    pub glyph_count: usize,
    pub lines: Vec<TextLine>,
    pub glyph_runs: Vec<TextGlyphRun>,
}

impl TextLayout {
    pub fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }

    pub fn rect_for_baseline_origin(&self, baseline_origin: Point) -> Rect {
        Rect::new(
            Point::new(baseline_origin.x, baseline_origin.y - self.baseline),
            self.size(),
        )
    }

    pub fn caret_rect(&self, byte_offset: usize, origin: Point) -> Rect {
        let offset = byte_offset.min(self.text.len());
        let line = self
            .lines
            .iter()
            .find(|line| offset >= line.range.start && offset <= line.range.end)
            .or_else(|| self.lines.last());
        let Some(line) = line else {
            return Rect::new(origin, Size::new(1.0, self.line_height.max(1.0)));
        };

        let advance = self.advance_for_offset(line, offset);
        Rect::new(
            Point::new(origin.x + line.x + advance, origin.y + line.y),
            Size::new(1.0, self.line_height.max(1.0)),
        )
    }

    pub fn selection_rects(&self, range: Range<usize>, origin: Point) -> Vec<Rect> {
        let start = range.start.min(self.text.len());
        let end = range.end.min(self.text.len());
        if start >= end {
            return Vec::new();
        }

        self.lines
            .iter()
            .filter_map(|line| {
                let line_start = start.max(line.range.start);
                let line_end = end.min(line.range.end);
                if line_start >= line_end {
                    return None;
                }
                let x0 = self.advance_for_offset(line, line_start);
                let x1 = self.advance_for_offset(line, line_end);
                Some(Rect::new(
                    Point::new(origin.x + line.x + x0, origin.y + line.y),
                    Size::new((x1 - x0).max(1.0), self.line_height.max(1.0)),
                ))
            })
            .collect()
    }

    fn advance_for_offset(&self, line: &TextLine, byte_offset: usize) -> f32 {
        if line.range.end <= line.range.start {
            return 0.0;
        }
        if !line.glyph_positions.is_empty() {
            let clamped = byte_offset.clamp(line.range.start, line.range.end);
            let mut previous_midpoint = 0.0;
            for glyph in &line.glyph_positions {
                if clamped <= glyph.byte_offset {
                    return previous_midpoint;
                }
                previous_midpoint = glyph.advance_x;
            }
            return line.width;
        }
        let clamped = byte_offset.clamp(line.range.start, line.range.end);
        let prefix_chars = self.text[line.range.start..clamped].chars().count() as f32;
        let total_chars = self.text[line.range.clone()].chars().count().max(1) as f32;
        line.width * (prefix_chars / total_chars)
    }

    pub fn caret_index_for_point(&self, click_point: Point, origin: Point) -> usize {
        if self.lines.is_empty() {
            return 0;
        }

        let local_y = click_point.y - origin.y;

        let line = self
            .lines
            .iter()
            .min_by_key(|line| {
                let line_center = line.y + self.line_height * 0.5;
                let diff = (local_y - line_center).abs();
                (diff * 1000.0) as i32
            })
            .unwrap_or(&self.lines[0]);

        let local_x = click_point.x - origin.x - line.x;
        if local_x <= 0.0 {
            return line.range.start;
        }
        if local_x >= line.width {
            return line.range.end;
        }

        if !line.glyph_positions.is_empty() {
            for glyph in &line.glyph_positions {
                if local_x < glyph.advance_x {
                    return glyph.byte_offset;
                }
            }
            return line.range.end;
        }

        let pct = local_x / line.width;
        let total_chars = self.text[line.range.clone()].chars().count();
        let char_offset = (pct * total_chars as f32).round() as usize;

        self.text[line.range.start..]
            .char_indices()
            .nth(char_offset)
            .map(|(idx, _)| line.range.start + idx)
            .unwrap_or(line.range.end)
    }
}
