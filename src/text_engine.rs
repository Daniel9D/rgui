use crate::core::{
    FontFamilyId, FontSource, Point, Rect, ShapedText, Size, TextEngine, TextHit, TextPosition,
    TextRange, TextSpec,
};

pub mod layout;
pub mod system;

pub use layout::{TextGlyphPosition, TextLayout};
pub use system::{TextShapeKey, TextSystem};

#[derive(Default)]
pub struct CosmicTextEngine {
    next_font_id: u64,
}

impl TextEngine for CosmicTextEngine {
    fn load_font(&mut self, _source: FontSource) -> FontFamilyId {
        self.next_font_id += 1;
        FontFamilyId::from_raw(self.next_font_id)
    }

    fn shape(&mut self, spec: &TextSpec, bounds: Size) -> ShapedText {
        ShapedText {
            size: bounds,
            baseline: 12.0,
            glyph_count: spec.text.chars().count(),
            glyphs: Vec::new(),
        }
    }

    fn hit_test(&self, _shaped: &ShapedText, _point: Point) -> TextHit {
        TextHit {
            position: TextPosition::new(0),
        }
    }

    fn caret_rect(&self, _shaped: &ShapedText, _position: TextPosition) -> Rect {
        Rect::new(Point::new(0.0, 0.0), Size::new(1.0, 14.0))
    }

    fn selection_rects(&self, _shaped: &ShapedText, _range: TextRange) -> Vec<Rect> {
        Vec::new()
    }
}
