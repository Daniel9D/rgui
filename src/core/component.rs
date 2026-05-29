use crate::{
    Constraints, DisplayList, Element, EventResult, LayoutBox, NodeId, Semantic, Theme, UiEvent,
};

#[derive(Default)]
pub struct StateStore;

#[derive(Default)]
pub struct CommandQueue;

pub struct ComponentCx {
    pub node_id: NodeId,
    pub theme: Theme,
    pub state: StateStore,
    pub commands: CommandQueue,
}

impl ComponentCx {
    pub fn new(node_id: NodeId, theme: Theme) -> Self {
        Self {
            node_id,
            theme,
            state: StateStore,
            commands: CommandQueue,
        }
    }
}

pub trait Component {
    fn render(&self, cx: &mut ComponentCx) -> Element;
}

pub struct PaintCx<'a> {
    pub display_list: &'a mut DisplayList,
}

pub trait Widget {
    fn measure(&self, constraints: Constraints) -> crate::Size;
    fn event(&mut self, event: &UiEvent) -> EventResult;
    fn semantics(&self) -> Semantic;
    fn paint(&self, cx: &mut PaintCx, layout: &LayoutBox);
}
