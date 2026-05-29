use crate::{NodeId, Rect, Size, Vec2};

#[derive(Clone, Debug, PartialEq)]
pub enum Length {
    Px(f32),
    Percent(f32),
    Fr(f32),
    Auto,
    MinContent,
    MaxContent,
    FitContent(Box<Length>),
}

impl Length {
    pub fn resolve(&self, parent: f32) -> Option<f32> {
        match self {
            Self::Px(px) => Some(*px),
            Self::Percent(percent) => Some(parent * *percent),
            Self::FitContent(inner) => inner.resolve(parent),
            Self::Fr(_) | Self::Auto | Self::MinContent | Self::MaxContent => None,
        }
    }
}

impl From<f32> for Length {
    fn from(value: f32) -> Self {
        Self::Px(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Display {
    Flex,
    Grid,
    Block,
    Stack,
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Position {
    Relative,
    Absolute,
    Fixed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Overflow {
    Visible,
    Hidden,
    Clip,
    Scroll,
    Auto,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Edge<T> {
    pub top: T,
    pub right: T,
    pub bottom: T,
    pub left: T,
}

impl<T: Clone> Edge<T> {
    pub fn all(value: T) -> Self {
        Self {
            top: value.clone(),
            right: value.clone(),
            bottom: value.clone(),
            left: value,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum GridTrack {
    Fixed(Length),
    Fraction(f32),
    Auto,
}

impl GridTrack {
    pub const fn fr(value: f32) -> Self {
        Self::Fraction(value)
    }

    pub const fn fraction(&self) -> Option<f32> {
        match self {
            Self::Fraction(value) => Some(*value),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Justify {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlexDirection {
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlexWrap {
    NoWrap,
    Wrap,
    WrapReverse,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GridPlacement {
    pub start: Option<i32>,
    pub end: Option<i32>,
    pub span: Option<u32>,
}

impl GridPlacement {
    pub fn start(value: i32) -> Self {
        Self {
            start: Some(value),
            end: None,
            span: None,
        }
    }

    pub fn span(value: u32) -> Self {
        Self {
            start: None,
            end: None,
            span: Some(value),
        }
    }
}

impl Default for GridPlacement {
    fn default() -> Self {
        Self {
            start: None,
            end: None,
            span: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Constraints {
    pub min: Size,
    pub max: Size,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayoutBox {
    pub node: NodeId,
    pub key: Option<String>,
    pub local_rect: Rect,
    pub world_rect: Rect,
    pub content_size: Size,
    pub padding_rect: Rect,
    pub content_rect: Rect,
    pub clip_rect: Option<Rect>,
    pub scroll_offset: Vec2,
    pub z_index: i32,
}

impl LayoutBox {
    pub fn new(node: NodeId, rect: Rect) -> Self {
        Self {
            node,
            key: None,
            local_rect: rect,
            world_rect: rect,
            content_size: rect.size,
            padding_rect: rect,
            content_rect: rect,
            clip_rect: None,
            scroll_offset: Vec2::default(),
            z_index: 0,
        }
    }

    pub fn with_key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    pub fn visible_rect(&self) -> Rect {
        self.clip_rect.unwrap_or(self.local_rect)
    }

    pub fn clips_overflow(&self) -> bool {
        self.clip_rect.is_some()
    }

    pub fn scrollable_size(&self) -> Size {
        Size::new(
            (self.content_size.width - self.local_rect.size.width).max(0.0),
            (self.content_size.height - self.local_rect.size.height).max(0.0),
        )
    }

    pub fn viewport_size(&self) -> Size {
        self.local_rect.size
    }

    pub fn with_content_size(mut self, size: Size) -> Self {
        self.content_size = size;
        self
    }

    pub fn with_clip(mut self, clip: Rect) -> Self {
        self.clip_rect = Some(clip);
        self
    }

    pub fn with_scroll_offset(mut self, offset: Vec2) -> Self {
        self.scroll_offset = offset;
        self
    }

    pub fn with_z_index(mut self, z_index: i32) -> Self {
        self.z_index = z_index;
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LayoutDiagnostics {
    pub layout_errors: Vec<String>,
    pub layout_warnings: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LayoutDebugSnapshot {
    pub engine: String,
    pub taffy_node_count: usize,
    pub dirty_layout_node_count: usize,
    pub layout_error_count: usize,
    pub layout_warning_count: usize,
    pub measured_text_count: usize,
    pub measured_widget_count: usize,
    pub full_rebuild_count: usize,
    pub incremental_layout_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LayoutDirtyReason {
    StyleChanged,
    ChildrenChanged,
    TextChanged,
    WidgetStateChanged,
    ViewportChanged,
    FontChanged,
    ThemeChanged,
    ScaleFactorChanged,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LayoutResult {
    pub boxes: Vec<LayoutBox>,
    pub diagnostics: LayoutDiagnostics,
    pub debug: LayoutDebugSnapshot,
}

impl LayoutResult {
    pub fn push(&mut self, layout: LayoutBox) {
        self.boxes.push(layout);
    }

    pub fn box_for_node(&self, node: NodeId) -> Option<&LayoutBox> {
        self.boxes.iter().find(|layout| layout.node == node)
    }

    pub fn box_for_key(&self, key: &str) -> Option<&LayoutBox> {
        self.boxes
            .iter()
            .find(|layout| layout.key.as_deref() == Some(key))
    }
}
