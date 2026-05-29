#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SizeU32 {
    pub width: u32,
    pub height: u32,
}

impl SizeU32 {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub origin: Point,
    pub size: Size,
}

impl Rect {
    pub const fn new(origin: Point, size: Size) -> Self {
        Self { origin, size }
    }

    pub fn max_x(self) -> f32 {
        self.origin.x + self.size.width
    }

    pub fn max_y(self) -> f32 {
        self.origin.y + self.size.height
    }

    pub fn contains(self, point: Point) -> bool {
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

    pub fn round_to_pixel(self) -> Rect {
        let x0 = self.origin.x.floor();
        let y0 = self.origin.y.floor();
        let x1 = self.max_x().ceil();
        let y1 = self.max_y().ceil();
        Rect::new(Point::new(x0, y0), Size::new(x1 - x0, y1 - y0))
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

pub fn physical_pixel_snap(rect: Rect, scale_factor: f32) -> Rect {
    Rect::new(
        Point::new(
            (rect.origin.x * scale_factor).round() / scale_factor,
            (rect.origin.y * scale_factor).round() / scale_factor,
        ),
        Size::new(
            (rect.size.width * scale_factor).round() / scale_factor,
            (rect.size.height * scale_factor).round() / scale_factor,
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
