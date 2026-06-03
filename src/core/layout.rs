use serde::Serialize;

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

/// Result of resolving a [`Length`] against a parent size.
///
/// Bug fix 3.11: the old `Option<f32>` return collapsed two
/// distinct conditions — "resolved to N pixels" and "cannot
/// resolve at this layer, defer to the layout pass" — into the
/// same `None`. This enum separates them so callers can decide
/// what to do with an intrinsic length (e.g. substitute a
/// default, fall back to a different sizing strategy) instead
/// of treating it as a generic missing value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ResolvedLength {
    /// A concrete pixel value resolved from `Px`, `Percent`, or
    /// `FitContent`.
    Concrete(f32),
    /// An intrinsic length (`Fr`, `Auto`, `MinContent`,
    /// `MaxContent`) that requires the layout pass to resolve.
    /// The numeric value is not knowable at this layer.
    Intrinsic,
}

impl ResolvedLength {
    /// `Some(px)` for `Concrete`, `None` for `Intrinsic`.
    /// Equivalent to `self.into()`.
    pub fn into_option(self) -> Option<f32> {
        match self {
            Self::Concrete(px) => Some(px),
            Self::Intrinsic => None,
        }
    }
}

impl From<ResolvedLength> for Option<f32> {
    fn from(r: ResolvedLength) -> Self {
        r.into_option()
    }
}

impl Length {
    /// Resolve the length relative to `parent`. Returns the
    /// richer [`ResolvedLength`] enum so callers can distinguish
    /// "concrete value" from "intrinsic, defer to layout pass".
    ///
    /// Bug fix 3.11: the previous return type was `Option<f32>`,
    /// which couldn't distinguish "deferred" from "unknown".
    /// For an `Option<f32>` view, use `.into()` or
    /// `into_option()`; for the boolean "is this intrinsic",
    /// use [`Length::is_intrinsic`].
    pub fn resolve(&self, parent: f32) -> ResolvedLength {
        match self {
            Self::Px(px) => ResolvedLength::Concrete(*px),
            Self::Percent(percent) => ResolvedLength::Concrete(parent * *percent),
            Self::FitContent(inner) => inner.resolve(parent),
            Self::Fr(_) | Self::Auto | Self::MinContent | Self::MaxContent => {
                ResolvedLength::Intrinsic
            }
        }
    }

    /// Convenience: `Some(px)` for concrete values, `None` for
    /// intrinsic. Equivalent to `self.resolve(parent).into()`.
    pub fn try_resolve(&self, parent: f32) -> Option<f32> {
        self.resolve(parent).into()
    }

    /// True if this length is intrinsic — i.e. requires the
    /// layout pass to resolve and cannot be turned into a pixel
    /// value at this layer.
    pub fn is_intrinsic(&self) -> bool {
        matches!(
            self,
            Self::Fr(_) | Self::Auto | Self::MinContent | Self::MaxContent
        )
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
    /// Bug fix 5.1: pure struct-literal constructor; no allocation
    /// or runtime work, so `const fn`. Useful in tests and
    /// debug-overlay code that wants to build a layout box at
    /// compile time.
    pub const fn new(node: NodeId, rect: Rect) -> Self {
        Self {
            node,
            key: None,
            local_rect: rect,
            world_rect: rect,
            content_size: rect.size,
            padding_rect: rect,
            content_rect: rect,
            clip_rect: None,
            scroll_offset: Vec2::new(0.0, 0.0),
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
        crate::core::geometry::scrollable_size(self.content_size, self.local_rect.size)
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Point;

    // Bug fix 5.1: `LayoutBox::new` is `const fn`. Verify by
    // constructing one in const context and asserting the field
    // values match the documented defaults (rect copied to all
    // geometry fields, scroll_offset zero, z_index 0).
    const LAYOUT: LayoutBox = LayoutBox::new(
        NodeId::from_raw(11),
        Rect::new(Point::new(2.0, 3.0), Size::new(40.0, 50.0)),
    );

    #[test]
    fn layout_box_new_is_const_constructible() {
        assert_eq!(LAYOUT.node.raw(), 11);
        assert_eq!(LAYOUT.local_rect.size, Size::new(40.0, 50.0));
        assert_eq!(LAYOUT.world_rect, LAYOUT.local_rect);
        assert_eq!(LAYOUT.content_size, Size::new(40.0, 50.0));
        assert_eq!(LAYOUT.scroll_offset, Vec2::new(0.0, 0.0));
        assert_eq!(LAYOUT.z_index, 0);
        assert!(LAYOUT.key.is_none());
        assert!(LAYOUT.clip_rect.is_none());
    }
}

#[cfg(test)]
mod length_resolve_tests {
    use super::*;

    // Bug fix 3.11: `Length::resolve` now returns a richer
    // `ResolvedLength` enum. Verify each `Length` variant
    // maps to the right arm and that the convenience
    // `try_resolve` / `is_intrinsic` helpers agree.

    #[test]
    fn resolve_px_is_concrete() {
        let r = Length::Px(12.0).resolve(100.0);
        assert_eq!(r, ResolvedLength::Concrete(12.0));
        assert_eq!(r.into_option(), Some(12.0));
    }

    #[test]
    fn resolve_percent_is_concrete() {
        let r = Length::Percent(0.5).resolve(200.0);
        assert_eq!(r, ResolvedLength::Concrete(100.0));
    }

    #[test]
    fn resolve_fit_content_delegates() {
        let r = Length::FitContent(Box::new(Length::Px(8.0))).resolve(100.0);
        assert_eq!(r, ResolvedLength::Concrete(8.0));
    }

    #[test]
    fn resolve_intrinsic_variants_are_intrinsic() {
        for variant in [
            Length::Fr(1.0),
            Length::Auto,
            Length::MinContent,
            Length::MaxContent,
        ] {
            let r = variant.resolve(100.0);
            assert_eq!(r, ResolvedLength::Intrinsic, "{variant:?}");
            assert_eq!(r.into_option(), None);
        }
    }

    #[test]
    fn try_resolve_agrees_with_into_option() {
        let concrete = Length::Px(5.0).try_resolve(100.0);
        assert_eq!(concrete, Some(5.0));
        let intrinsic = Length::Auto.try_resolve(100.0);
        assert_eq!(intrinsic, None);
    }

    #[test]
    fn is_intrinsic_classifies_variants() {
        assert!(!Length::Px(0.0).is_intrinsic());
        assert!(!Length::Percent(0.5).is_intrinsic());
        assert!(!Length::FitContent(Box::new(Length::Px(1.0))).is_intrinsic());
        assert!(Length::Fr(1.0).is_intrinsic());
        assert!(Length::Auto.is_intrinsic());
        assert!(Length::MinContent.is_intrinsic());
        assert!(Length::MaxContent.is_intrinsic());
    }

    #[test]
    fn resolved_length_from_impl_covers_both_arms() {
        let concrete: Option<f32> = ResolvedLength::Concrete(7.0).into();
        assert_eq!(concrete, Some(7.0));
        let intrinsic: Option<f32> = ResolvedLength::Intrinsic.into();
        assert_eq!(intrinsic, None);
    }
}
