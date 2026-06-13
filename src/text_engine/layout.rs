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
        // Bug fix TE-5: bound-safe slice. The previous direct
        // `self.text[line.range.start..clamped]` could panic if
        // `clamped > self.text.len()` (e.g. with malformed
        // multi-byte text or a future line range calculation
        // that overshoots). Use `get(..).unwrap_or("")` so the
        // fallback handles edge cases gracefully.
        let prefix = self
            .text
            .get(line.range.start..clamped)
            .unwrap_or("");
        let prefix_chars = prefix.chars().count() as f32;
        let total_slice = self
            .text
            .get(line.range.clone())
            .unwrap_or("");
        let total_chars = total_slice.chars().count().max(1) as f32;
        line.width * (prefix_chars / total_chars)
    }

    pub fn caret_index_for_point(&self, click_point: Point, origin: Point) -> usize {
        if self.lines.is_empty() {
            return 0;
        }

        let local_y = click_point.y - origin.y;

        // Bug fix TE-4: the previous `unwrap_or(&self.lines[0])`
        // eagerly evaluated `&self.lines[0]` (because references
        // are `Copy` and `unwrap_or` takes by value). On an
        // empty `lines` vec this would panic with index out of
        // bounds *before* `min_by_key` returned its `None`.
        // Use `unwrap_or_else` so the fallback is only computed
        // when the iterator is empty — and guard with an
        // `expect` so the empty case is impossible to reach (we
        // already early-returned above when `lines.is_empty()`).
        let line = self
            .lines
            .iter()
            .min_by_key(|line| {
                let line_center = line.y + self.line_height * 0.5;
                let diff = (local_y - line_center).abs();
                (diff * 1000.0) as i32
            })
            .unwrap_or_else(|| self.lines.first().expect("lines non-empty"));

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

#[cfg(test)]
mod tests {
    use super::*;

    // Bug fix TE-4: `caret_index_for_point` on an empty
    // `lines` vec used to panic because `unwrap_or(&self.lines[0])`
    // evaluated `&self.lines[0]` eagerly when `lines` was empty.
    // We now early-return 0 for the empty case.
    #[test]
    fn caret_index_for_point_on_empty_lines_returns_zero() {
        let layout = TextLayout {
            text: String::new(),
            font_px: 14.0,
            width: 0.0,
            height: 0.0,
            baseline: 0.0,
            line_height: 14.0,
            glyph_count: 0,
            lines: Vec::new(),
            glyph_runs: Vec::new(),
        };
        assert_eq!(
            layout.caret_index_for_point(Point::new(5.0, 5.0), Point::new(0.0, 0.0)),
            0
        );
        assert_eq!(
            layout.caret_rect(0, Point::new(0.0, 0.0)),
            Rect::new(Point::new(0.0, 0.0), Size::new(1.0, 14.0))
        );
        assert_eq!(layout.selection_rects(0..5, Point::new(0.0, 0.0)), Vec::<Rect>::new());
    }
}
