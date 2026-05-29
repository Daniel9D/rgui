use crate::state::WidgetState;

#[derive(Clone, Debug, Default)]
pub struct ButtonState {
    pub pressed: bool,
    pub hovered: bool,
    pub focused: bool,
    pub loading: bool,
}

impl WidgetState for ButtonState {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
