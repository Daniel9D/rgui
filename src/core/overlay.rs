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
    /// Bug fix 5.1: `OverlaySpec` only contains `Copy` fields and
    /// matches-on-enum (const-evaluable), so the constructor is
    /// `const fn`. Useful in tests and fixture builders.
    pub const fn new(owner: NodeId, layer: LayerKind) -> Self {
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

    /// Returns the overlays sorted by `LayerKind::order` (lowest first,
    /// drawn first / hit-tested first). Clones the underlying vec; if
    /// you have a `&mut self`, prefer [`Self::sort_in_place`] to avoid
    /// the allocation.
    pub fn ordered(&self) -> Vec<OverlaySpec> {
        let mut overlays = self.overlays.clone();
        overlays.sort_by_key(|overlay| overlay.layer.order());
        overlays
    }

    /// In-place sort using [`LayerKind::order`]. Reuses the caller's
    /// buffer, so this is allocation-free.
    pub fn sort_in_place(&mut self) {
        self.overlays.sort_by_key(|overlay| overlay.layer.order());
    }

    /// Borrow the overlays sorted by `LayerKind::order` without cloning.
    /// Caller must already own the vec mutably; use [`Self::ordered`] for
    /// an owned return.
    pub fn sorted_slice(&mut self) -> &[OverlaySpec] {
        self.sort_in_place();
        &self.overlays
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Bug fix 5.1: `OverlaySpec::new` is `const fn`. Verify by
    // constructing several in const context (one per LayerKind
    // branch) and asserting the derived `modal` and `focus_scope`
    // flags match the documented rules.
    const POPOVER: OverlaySpec = OverlaySpec::new(NodeId::from_raw(1), LayerKind::Popover);
    const MODAL: OverlaySpec = OverlaySpec::new(NodeId::from_raw(2), LayerKind::Modal);
    const TOOLTIP: OverlaySpec = OverlaySpec::new(NodeId::from_raw(3), LayerKind::Tooltip);

    #[test]
    fn overlay_spec_new_is_const_constructible() {
        // Modal layer → modal=true, focus_scope=true.
        assert!(MODAL.modal);
        assert!(MODAL.focus_scope);
        assert_eq!(MODAL.dismiss, DismissPolicy::EscapeOrOutsidePointer);

        // Popover layer → modal=false, focus_scope=true.
        assert!(!POPOVER.modal);
        assert!(POPOVER.focus_scope);

        // Tooltip layer → modal=false, focus_scope=false.
        assert!(!TOOLTIP.modal);
        assert!(!TOOLTIP.focus_scope);

        // Default placement is Bottom for every layer.
        assert_eq!(POPOVER.placement, Placement::Bottom);
        // Anchor defaults to the owner.
        assert_eq!(POPOVER.anchor, AnchorSpec::Node(NodeId::from_raw(1)));
    }
}
