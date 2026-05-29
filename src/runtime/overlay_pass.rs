use std::collections::HashSet;

use crate::core::{Element, ElementKind, LayerKind, NodeId, Point, Rect, Size, Style, WidgetKind};

use super::UiNode;

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedOverlay {
    pub key: Option<String>,
    pub layer: LayerKind,
    pub modal: bool,
    pub children: Vec<Element>,
    pub style: Style,
    pub kind: ElementKind,
    pub anchor_rect: Rect,
    pub context_menu_anchor: Option<Point>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PipelineOverlays {
    pub overlays: Vec<ResolvedOverlay>,
    pub modal_backdrop: Option<(NodeId, Rect)>,
}

pub fn resolve_overlays_for_node(
    node: &UiNode,
    rect: Rect,
    viewport: Size,
    dismissed_overlay_keys: &HashSet<String>,
    opened_overlay_keys: &HashSet<String>,
    open_context_menu_key: Option<&str>,
    context_menu_anchor: Option<Point>,
) -> PipelineOverlays {
    let mut overlays = PipelineOverlays::default();
    if let Some(overlay) = node.overlay.as_ref() {
        let overlay_key = overlay.key.as_ref().map(|key| key.as_str().to_string());
        let dismissed = overlay_key
            .as_ref()
            .is_some_and(|key| dismissed_overlay_keys.contains(key));
        let is_context_menu = matches!(overlay.kind, ElementKind::Widget(WidgetKind::Menu));
        let context_open = is_context_menu
            && node
                .key
                .as_ref()
                .is_some_and(|key| Some(key.as_str()) == open_context_menu_key);
        let opened_by_trigger = overlay_key
            .as_ref()
            .is_some_and(|key| opened_overlay_keys.contains(key));
        if (overlay.open || context_open || opened_by_trigger) && !dismissed {
            overlays.overlays.push(ResolvedOverlay {
                key: overlay_key,
                layer: layer_for_overlay_kind(&overlay.kind),
                modal: false,
                children: overlay.children.clone(),
                style: overlay.style.clone(),
                kind: overlay.kind.clone(),
                anchor_rect: rect,
                context_menu_anchor: if context_open {
                    context_menu_anchor
                } else {
                    None
                },
            });
        }
    }

    let modal_key = node.key.as_ref().map(|key| key.as_str().to_string());
    let dismissed = modal_key
        .as_ref()
        .is_some_and(|key| dismissed_overlay_keys.contains(key));
    if matches!(node.kind, ElementKind::Widget(WidgetKind::Modal)) && node.open && !dismissed {
        let backdrop = Rect::new(Point::new(0.0, 0.0), viewport);
        overlays.overlays.push(ResolvedOverlay {
            key: modal_key,
            layer: LayerKind::Modal,
            modal: true,
            children: Vec::new(),
            style: node.style.clone(),
            kind: node.kind.clone(),
            anchor_rect: backdrop,
            context_menu_anchor: None,
        });
        overlays
            .modal_backdrop
            .replace((backdrop_node_id(node.id), backdrop));
    }

    overlays
}

pub fn place_anchored_overlay(
    panel_size: Size,
    anchor_rect: Rect,
    context_menu_anchor: Option<Point>,
    viewport: Size,
) -> Rect {
    let origin = if let Some(pt) = context_menu_anchor {
        Point::new(pt.x, pt.y)
    } else {
        Point::new(anchor_rect.origin.x, anchor_rect.max_y())
    };
    let mut rect = Rect::new(origin, panel_size);
    if rect.max_y() > viewport.height {
        let anchor_top = if let Some(pt) = context_menu_anchor {
            pt.y
        } else {
            anchor_rect.origin.y
        };
        rect.origin.y = (anchor_top - panel_size.height).max(0.0);
    }
    constrain_overlay_to_viewport(rect, viewport)
}

fn backdrop_node_id(owner: NodeId) -> NodeId {
    NodeId::from_raw(u64::MAX - owner.raw())
}

fn layer_for_overlay_kind(kind: &ElementKind) -> LayerKind {
    match kind {
        ElementKind::Widget(WidgetKind::Tooltip) => LayerKind::Tooltip,
        ElementKind::Widget(WidgetKind::Modal) => LayerKind::Modal,
        ElementKind::Widget(WidgetKind::Menu) => LayerKind::ContextMenu,
        _ => LayerKind::Popover,
    }
}

pub fn constrain_overlay_to_viewport(mut rect: Rect, viewport: Size) -> Rect {
    if rect.max_x() > viewport.width {
        rect.origin.x = (viewport.width - rect.size.width).max(0.0);
    }
    if rect.origin.x < 0.0 {
        rect.origin.x = 0.0;
    }
    if rect.max_y() > viewport.height {
        rect.origin.y = (viewport.height - rect.size.height).max(0.0);
    }
    if rect.origin.y < 0.0 {
        rect.origin.y = 0.0;
    }
    rect
}
