//! Geometry primitives: [`Point`], [`Vec2`], [`Size`], [`SizeU32`], and [`Rect`].
//!
//! ## Interval convention
//!
//! All `Rect`s use a **half-open** interval: the origin is inclusive and
//! the max edge (`origin + size`) is exclusive. This matches CSS, the HTML
//! hit-testing model, and the vast majority of UI toolkits.
//!
//! - [`Rect::contains`] returns `false` for a point on the right or bottom edge.
//! - [`Rect::intersect`] returns `None` when the two rects only touch along
//!   an edge (because the result would be a zero-area rect).
//!
//! Use [`Rect::contains_inclusive`] if you need the legacy closed-interval
//! behavior (point on the edge counts as inside).

use serde::Serialize;

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Returns the origin as a `Vec2`. Convenience for math.
    pub const fn to_vec2(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Component-wise `min`. NaN-passthrough consistent with `f32::min`.
    pub fn component_min(self, other: Self) -> Self {
        Self::new(self.x.min(other.x), self.y.min(other.y))
    }

    /// Component-wise `max`. NaN-passthrough consistent with `f32::max`.
    pub fn component_max(self, other: Self) -> Self {
        Self::new(self.x.max(other.x), self.y.max(other.y))
    }

    /// `true` if every component is `0.0` (or `-0.0`, which compares equal).
    pub fn is_zero(self) -> bool {
        self.x == 0.0 && self.y == 0.0
    }
}

impl From<Point> for Vec2 {
    fn from(p: Point) -> Self {
        Self::new(p.x, p.y)
    }
}

impl From<Vec2> for Point {
    fn from(v: Vec2) -> Self {
        Self::new(v.x, v.y)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    /// `size - other` with a floor at zero. Used by scroll math to avoid
    /// negative content extents.
    pub fn sub_clamped(self, other: Self) -> Self {
        Self::new(
            (self.width - other.width).max(0.0),
            (self.height - other.height).max(0.0),
        )
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct SizeU32 {
    pub width: u32,
    pub height: u32,
}

impl SizeU32 {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
pub struct Rect {
    pub origin: Point,
    pub size: Size,
}

impl Rect {
    pub const fn new(origin: Point, size: Size) -> Self {
        Self { origin, size }
    }

    /// Exclusive right edge (`origin.x + size.width`).
    pub const fn max_x(self) -> f32 {
        self.origin.x + self.size.width
    }

    /// Exclusive bottom edge (`origin.y + size.height`).
    pub const fn max_y(self) -> f32 {
        self.origin.y + self.size.height
    }

    /// `true` if `point` is inside the rect under the **half-open** convention:
    /// the origin is inclusive, the right/bottom edges are exclusive.
    ///
    /// This is the convention used by CSS, the DOM, and most UI toolkits.
    /// Bug fix 1.9: previously used `<=` which is the closed-interval form.
    pub fn contains(self, point: Point) -> bool {
        point.x >= self.origin.x
            && point.y >= self.origin.y
            && point.x < self.max_x()
            && point.y < self.max_y()
    }

    /// `true` if `point` is inside the rect under the **closed** convention
    /// (point on the right or bottom edge counts as inside).
    ///
    /// This is the legacy `Rect::contains` behavior; use it only when you
    /// specifically need the closed form (e.g. for editor marquee selection).
    pub fn contains_inclusive(self, point: Point) -> bool {
        point.x >= self.origin.x
            && point.y >= self.origin.y
            && point.x <= self.max_x()
            && point.y <= self.max_y()
    }

    pub fn intersect(self, other: Rect) -> Option<Rect> {
        let min_x = self.origin.x.max(other.origin.x);
        let min_y = self.origin.y.max(other.origin.y);
        let max_x = self.max_x().min(other.max_x());
        let max_y = self.max_y().min(other.max_y());
        if max_x <= min_x || max_y <= min_y {
            return None;
        }
        Some(Rect::new(
            Point::new(min_x, min_y),
            Size::new(max_x - min_x, max_y - min_y),
        ))
    }

    pub fn union(self, other: Rect) -> Rect {
        let min_x = self.origin.x.min(other.origin.x);
        let min_y = self.origin.y.min(other.origin.y);
        let max_x = self.max_x().max(other.max_x());
        let max_y = self.max_y().max(other.max_y());
        Rect::new(
            Point::new(min_x, min_y),
            Size::new(max_x - min_x, max_y - min_y),
        )
    }

    pub fn inflate(self, dx: f32, dy: f32) -> Rect {
        Rect::new(
            Point::new(self.origin.x - dx, self.origin.y - dy),
            Size::new(self.size.width + dx * 2.0, self.size.height + dy * 2.0),
        )
    }

    pub fn translate(self, delta: Vec2) -> Rect {
        Rect::new(
            Point::new(self.origin.x + delta.x, self.origin.y + delta.y),
            self.size,
        )
    }

    /// Snap to whole logical pixels. The origin is floored, the max edge is
    /// ceiled, so the resulting rect always fully covers the original.
    ///
    /// For a fractional-scale variant that *preserves area* (rounded
    /// size rather than max-cover), see [`physical_pixel_snap`].
    /// Bug fix 4.5: the two snapping functions follow different
    /// policies; pick consciously.
    pub fn round_to_pixel(self) -> Rect {
        let x0 = self.origin.x.floor();
        let y0 = self.origin.y.floor();
        let x1 = self.max_x().ceil();
        let y1 = self.max_y().ceil();
        Rect::new(Point::new(x0, y0), Size::new(x1 - x0, y1 - y0))
    }

    /// `true` if the rect is fully empty (size <= 0 on either axis).
    pub fn is_empty(self) -> bool {
        self.size.width <= 0.0 || self.size.height <= 0.0
    }

    pub(crate) fn to_kurbo(self) -> kurbo::Rect {
        kurbo::Rect::new(
            self.origin.x as f64,
            self.origin.y as f64,
            self.max_x() as f64,
            self.max_y() as f64,
        )
    }

    pub(crate) fn from_kurbo(rect: kurbo::Rect) -> Rect {
        Rect::new(
            Point::new(rect.x0 as f32, rect.y0 as f32),
            Size::new((rect.x1 - rect.x0) as f32, (rect.y1 - rect.y0) as f32),
        )
    }
}

/// Returns the scrollable extent (i.e. `content_size - viewport_size` floored
/// at zero on each axis). Used by both [`crate::core::layout::LayoutBox`] and
/// [`crate::core::scroll::ScrollState`] — keeping it in one place so the
/// behavior stays in lockstep.
pub const fn scrollable_size(content_size: Size, viewport_size: Size) -> Size {
    Size::new(
        (content_size.width - viewport_size.width).max(0.0),
        (content_size.height - viewport_size.height).max(0.0),
    )
}

pub fn effective_clip(stack: &[Rect], viewport: Rect) -> Option<Rect> {
    stack
        .iter()
        .copied()
        .try_fold(viewport, |clip, next| clip.intersect(next))
}

pub fn clip_child(parent_clip: Option<Rect>, child: Rect) -> Option<Rect> {
    match parent_clip {
        Some(parent) => parent.intersect(child),
        None => Some(child),
    }
}

pub fn scroll_translate(rect: Rect, scroll: Vec2) -> Rect {
    rect.translate(Vec2::new(-scroll.x, -scroll.y))
}

/// Round origin and size to whole physical pixels, given a `scale_factor`.
///
/// Unlike [`Rect::round_to_pixel`] this is symmetric: every component is
/// rounded to the nearest physical pixel and divided back to logical space,
/// so the result has the same overall area as the input. Use this for
/// fractional scales (1.5, 2.0, 1.25) where you want the size to be a
/// whole-pixel multiple.
///
/// For a max-cover variant (origin floored, max ceiled) see
/// [`Rect::round_to_pixel`]. Bug fix 4.5: the two snapping functions
/// follow different policies; pick consciously.
pub fn physical_pixel_snap(rect: Rect, scale_factor: f32) -> Rect {
    let scale = if scale_factor > 0.0 { scale_factor } else { 1.0 };
    Rect::new(
        Point::new(
            (rect.origin.x * scale).round() / scale,
            (rect.origin.y * scale).round() / scale,
        ),
        Size::new(
            (rect.size.width * scale).round() / scale,
            (rect.size.height * scale).round() / scale,
        ),
    )
}

pub(crate) fn rounded_rect_to_kurbo(rect: Rect, radius: crate::core::Radius) -> kurbo::RoundedRect {
    kurbo::RoundedRect::from_rect(
        rect.to_kurbo(),
        kurbo::RoundedRectRadii {
            top_left: radius.top_left as f64,
            top_right: radius.top_right as f64,
            bottom_right: radius.bottom_right as f64,
            bottom_left: radius.bottom_left as f64,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_is_half_open_at_max_edge() {
        let r = Rect::new(Point::new(0.0, 0.0), Size::new(10.0, 10.0));
        assert!(r.contains(Point::new(0.0, 0.0)));   // origin inclusive
        assert!(r.contains(Point::new(9.999, 9.999))); // interior
        assert!(!r.contains(Point::new(10.0, 5.0)));  // right edge exclusive
        assert!(!r.contains(Point::new(5.0, 10.0)));  // bottom edge exclusive
        assert!(!r.contains(Point::new(-0.001, 5.0))); // left edge stays inclusive? no, outside
        assert!(r.contains(Point::new(9.999, 0.0)));  // top edge inclusive
    }

    #[test]
    fn contains_inclusive_keeps_legacy_behavior() {
        let r = Rect::new(Point::new(0.0, 0.0), Size::new(10.0, 10.0));
        assert!(r.contains_inclusive(Point::new(10.0, 5.0)));
        assert!(r.contains_inclusive(Point::new(5.0, 10.0)));
        assert!(r.contains_inclusive(Point::new(10.0, 10.0)));
    }

    #[test]
    fn intersect_returns_none_on_edge_touch() {
        let a = Rect::new(Point::new(0.0, 0.0), Size::new(10.0, 10.0));
        let b = Rect::new(Point::new(10.0, 0.0), Size::new(10.0, 10.0));
        assert_eq!(a.intersect(b), None);
        let c = Rect::new(Point::new(5.0, 0.0), Size::new(10.0, 10.0));
        let i = a.intersect(c).unwrap();
        assert_eq!(i.origin, Point::new(5.0, 0.0));
        assert_eq!(i.size, Size::new(5.0, 10.0));
    }

    #[test]
    fn scrollable_size_floors_at_zero() {
        let s = scrollable_size(Size::new(50.0, 50.0), Size::new(100.0, 100.0));
        assert_eq!(s, Size::new(0.0, 0.0));
        let s = scrollable_size(Size::new(200.0, 200.0), Size::new(100.0, 50.0));
        assert_eq!(s, Size::new(100.0, 150.0));
    }

    #[test]
    fn size_sub_clamped_floors_at_zero() {
        let s = Size::new(50.0, 50.0).sub_clamped(Size::new(100.0, 100.0));
        assert_eq!(s, Size::new(0.0, 0.0));
    }

    #[test]
    fn vec2_helpers() {
        let a = Vec2::new(1.0, 5.0);
        let b = Vec2::new(3.0, 2.0);
        assert_eq!(a.component_min(b), Vec2::new(1.0, 2.0));
        assert_eq!(a.component_max(b), Vec2::new(3.0, 5.0));
        assert!(!a.is_zero());
        assert!(Vec2::default().is_zero());
    }
}
