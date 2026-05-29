use crate::{Point, Rect, Size, TextSpec};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FontFamilyId(u64);

impl FontFamilyId {
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FontSource {
    System(String),
    File(String),
    Bytes(Vec<u8>),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ShapedText {
    pub size: Size,
    pub baseline: f32,
    pub glyph_count: usize,
    pub glyphs: Vec<ShapedGlyph>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ShapedGlyph {
    pub font_id: u64,
    pub glyph_id: u32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub advance: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextPosition {
    pub byte_offset: usize,
}

impl TextPosition {
    pub const fn new(byte_offset: usize) -> Self {
        Self { byte_offset }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextRange {
    pub start: usize,
    pub end: usize,
}

impl TextRange {
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextSelection {
    pub anchor: TextPosition,
    pub head: TextPosition,
}

impl TextSelection {
    pub const fn caret(position: TextPosition) -> Self {
        Self {
            anchor: position,
            head: position,
        }
    }

    pub fn range(self) -> TextRange {
        TextRange::new(
            self.anchor.byte_offset.min(self.head.byte_offset),
            self.anchor.byte_offset.max(self.head.byte_offset),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextHit {
    pub position: TextPosition,
}

pub trait TextEngine {
    fn load_font(&mut self, source: FontSource) -> FontFamilyId;
    fn shape(&mut self, spec: &TextSpec, bounds: Size) -> ShapedText;
    fn hit_test(&self, shaped: &ShapedText, point: Point) -> TextHit;
    fn caret_rect(&self, shaped: &ShapedText, position: TextPosition) -> Rect;
    fn selection_rects(&self, shaped: &ShapedText, range: TextRange) -> Vec<Rect>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextInputState {
    pub text: String,
    pub selection: TextSelection,
    pub composing: Option<crate::ImePreedit>,
    pub scroll_x: f32,
    pub scroll_y: f32,
}

impl TextInputState {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            selection: TextSelection::caret(TextPosition::new(0)),
            composing: None,
            scroll_x: 0.0,
            scroll_y: 0.0,
        }
    }

    pub fn commit_text(&mut self, value: &str) {
        let range = self.selection.range();
        self.text.replace_range(range.start..range.end, value);
        let caret = range.start + value.len();
        self.selection = TextSelection::caret(TextPosition::new(caret));
        self.composing = None;
    }
}
