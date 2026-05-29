use crate::{Size, Vec2};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AxisSet {
    pub horizontal: bool,
    pub vertical: bool,
}

impl AxisSet {
    pub const fn horizontal() -> Self {
        Self {
            horizontal: true,
            vertical: false,
        }
    }

    pub const fn vertical() -> Self {
        Self {
            horizontal: false,
            vertical: true,
        }
    }

    pub const fn both() -> Self {
        Self {
            horizontal: true,
            vertical: true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollbarPolicy {
    Auto,
    Always,
    Never,
    Overlay,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollState {
    pub offset: Vec2,
    pub content_size: Size,
    pub viewport_size: Size,
    pub axis: AxisSet,
    pub policy_x: ScrollbarPolicy,
    pub policy_y: ScrollbarPolicy,
    pub velocity: Vec2,
}

impl ScrollState {
    pub fn new(axis: AxisSet) -> Self {
        Self {
            offset: Vec2::new(0.0, 0.0),
            content_size: Size::new(0.0, 0.0),
            viewport_size: Size::new(0.0, 0.0),
            axis,
            policy_x: ScrollbarPolicy::Auto,
            policy_y: ScrollbarPolicy::Auto,
            velocity: Vec2::new(0.0, 0.0),
        }
    }

    pub fn max_offset(&self) -> Vec2 {
        Vec2::new(
            (self.content_size.width - self.viewport_size.width).max(0.0),
            (self.content_size.height - self.viewport_size.height).max(0.0),
        )
    }

    pub fn scroll_by(&mut self, delta: Vec2) -> Vec2 {
        let max = self.max_offset();
        let next_x = if self.axis.horizontal {
            (self.offset.x + delta.x).clamp(0.0, max.x)
        } else {
            self.offset.x
        };
        let next_y = if self.axis.vertical {
            (self.offset.y + delta.y).clamp(0.0, max.y)
        } else {
            self.offset.y
        };
        self.offset = Vec2::new(next_x, next_y);
        self.offset
    }

    pub fn consume_wheel_delta(&mut self, delta: Vec2) -> Vec2 {
        let before = self.offset;
        self.scroll_by(delta);
        Vec2::new(
            delta.x - (self.offset.x - before.x),
            delta.y - (self.offset.y - before.y),
        )
    }
}
