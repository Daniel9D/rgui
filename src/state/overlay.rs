use crate::state::WidgetState;

#[derive(Clone, Debug)]
pub struct OverlayState {
    pub open: bool,
    pub dismissed: bool,
}

impl Default for OverlayState {
    fn default() -> Self {
        Self {
            open: false,
            dismissed: false,
        }
    }
}

impl WidgetState for OverlayState {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
