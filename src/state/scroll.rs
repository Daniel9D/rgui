use crate::core::Vec2;
use crate::state::WidgetState;

#[derive(Clone, Debug)]
pub struct ScrollWidgetState {
    pub offset: Vec2,
    pub max_offset: Vec2,
    pub dragging: bool,
}

impl Default for ScrollWidgetState {
    fn default() -> Self {
        Self {
            offset: Vec2::default(),
            max_offset: Vec2::default(),
            dragging: false,
        }
    }
}

impl ScrollWidgetState {
    pub fn clamp(&mut self) {
        self.offset.x = self.offset.x.clamp(0.0, self.max_offset.x);
        self.offset.y = self.offset.y.clamp(0.0, self.max_offset.y);
    }
}

impl WidgetState for ScrollWidgetState {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
