use std::any::{Any, TypeId};
use std::collections::HashMap;

use crate::core::NodeId;

pub trait WidgetState: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct StateArena {
    states: HashMap<(NodeId, TypeId), Box<dyn WidgetState>>,
}

impl Default for StateArena {
    fn default() -> Self {
        Self::new()
    }
}

impl StateArena {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
        }
    }

    pub fn insert<S: WidgetState>(&mut self, node: NodeId, state: S) {
        self.states
            .insert((node, TypeId::of::<S>()), Box::new(state));
    }

    pub fn get<S: WidgetState>(&self, node: NodeId) -> Option<&S> {
        self.states
            .get(&(node, TypeId::of::<S>()))
            .and_then(|s| s.as_any().downcast_ref::<S>())
    }

    pub fn get_mut<S: WidgetState>(&mut self, node: NodeId) -> Option<&mut S> {
        self.states
            .get_mut(&(node, TypeId::of::<S>()))
            .and_then(|s| s.as_any_mut().downcast_mut::<S>())
    }

    pub fn remove<S: WidgetState>(&mut self, node: NodeId) -> Option<Box<dyn WidgetState>> {
        self.states.remove(&(node, TypeId::of::<S>()))
    }

    pub fn contains<S: WidgetState>(&self, node: NodeId) -> bool {
        self.states.contains_key(&(node, TypeId::of::<S>()))
    }

    pub fn clear(&mut self) {
        self.states.clear();
    }

    pub fn len(&self) -> usize {
        self.states.len()
    }
}

pub mod button;
pub mod checkbox;
pub mod input;
pub mod overlay;
pub mod scroll;

pub use button::ButtonState;
pub use checkbox::CheckboxState;
pub use input::InputState;
pub use overlay::OverlayState;
pub use scroll::ScrollWidgetState;
