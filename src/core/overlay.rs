use crate::{LayerKind, NodeId, Rect};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AnchorSpec {
    Node(NodeId),
    Rect(Rect),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Placement {
    Top,
    Right,
    Bottom,
    Left,
    Center,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DismissPolicy {
    None,
    Escape,
    OutsidePointer,
    EscapeOrOutsidePointer,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OverlaySpec {
    pub owner: NodeId,
    pub anchor: AnchorSpec,
    pub placement: Placement,
    pub modal: bool,
    pub dismiss: DismissPolicy,
    pub focus_scope: bool,
    pub layer: LayerKind,
}

impl OverlaySpec {
    pub fn new(owner: NodeId, layer: LayerKind) -> Self {
        Self {
            owner,
            anchor: AnchorSpec::Node(owner),
            placement: Placement::Bottom,
            modal: matches!(layer, LayerKind::Modal),
            dismiss: DismissPolicy::EscapeOrOutsidePointer,
            focus_scope: matches!(
                layer,
                LayerKind::Modal | LayerKind::Popover | LayerKind::ContextMenu
            ),
            layer,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct OverlayManager {
    overlays: Vec<OverlaySpec>,
}

impl OverlayManager {
    pub fn register(&mut self, overlay: OverlaySpec) {
        self.overlays.push(overlay);
    }

    pub fn ordered(&self) -> Vec<OverlaySpec> {
        let mut overlays = self.overlays.clone();
        overlays.sort_by_key(|overlay| match overlay.layer {
            LayerKind::Document => 0,
            LayerKind::Floating => 1,
            LayerKind::Popover => 2,
            LayerKind::Tooltip => 3,
            LayerKind::ContextMenu => 4,
            LayerKind::Modal => 5,
            LayerKind::Debug => 6,
        });
        overlays
    }
}
