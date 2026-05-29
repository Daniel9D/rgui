use crate::state::WidgetState;

#[derive(Clone, Debug)]
pub struct CheckboxState {
    pub checked: bool,
    pub indeterminate: bool,
}

impl Default for CheckboxState {
    fn default() -> Self {
        Self {
            checked: false,
            indeterminate: false,
        }
    }
}

impl WidgetState for CheckboxState {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
