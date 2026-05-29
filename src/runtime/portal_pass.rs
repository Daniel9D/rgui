use crate::core::{
    BorderCmd, Element, ElementKind, EventHandlers, HitTestEntry, HitTestTree, LayerKind, NodeId,
    PaintCommand, Point, Rect, RectCmd, SemanticNode, SemanticStates, SemanticTree, ShadowCmd,
    Size, Theme, ThemeMode, WidgetKind,
};
use crate::core::{Color, Paint};

use super::{UiNode, UiTree};
#[derive(Clone, Debug)]
pub struct PortalRootLayout {
    pub panel_rect: Rect,
    pub content_rect: Rect,
    pub layout: crate::core::LayoutResult,
}

#[derive(Clone, Debug)]
pub struct PortalRoot {
    pub owner: NodeId,
    pub key: Option<String>,
    pub layer: LayerKind,
    pub rect: Rect,
    pub modal: bool,
    pub children: PortalChildren,
    pub computed: Option<PortalRootLayout>,
    pub style: crate::core::Style,
    pub kind: ElementKind,
    pub anchor_rect: Rect,
    pub context_menu_anchor: Option<Point>,
    pub tree: Option<UiTree>,
}

#[derive(Clone, Debug)]
pub enum PortalChildren {
    Items(Vec<PortalItem>),
}

#[derive(Clone, Debug)]
pub struct PortalItem {
    pub node: NodeId,
    pub element: Element,
}

#[derive(Clone, Debug, Default)]
pub struct PortalTree {
    pub roots: Vec<PortalRoot>,
}

impl PortalTree {
    pub fn collect_from_overlay_pass(
        &mut self,
        node: &UiNode,
        owner_rect: Rect,
        viewport: Size,
        dismissed_overlay_keys: &std::collections::HashSet<String>,
        opened_overlay_keys: &std::collections::HashSet<String>,
        tree: &UiTree,
        open_context_menu_key: Option<&str>,
        context_menu_anchor: Option<Point>,
    ) {
        let overlays = super::overlay_pass::resolve_overlays_for_node(
            node,
            owner_rect,
            viewport,
            dismissed_overlay_keys,
            opened_overlay_keys,
            open_context_menu_key,
            context_menu_anchor,
        );

        for overlay in overlays.overlays {
            let mut children = overlay_children_to_portal_items(
                node.id,
                overlay
                    .key
                    .as_deref()
                    .or_else(|| node.key.as_ref().map(|key| key.as_str())),
                &overlay.children,
            );

            // For Modal/Popover/Tooltip widgets declared in tree, add UiNode children
            if matches!(
                node.kind,
                ElementKind::Widget(WidgetKind::Modal | WidgetKind::Popover | WidgetKind::Tooltip)
            ) {
                for child_id in &node.children {
                    if let Some(child_node) = tree.get(*child_id) {
                        children.push(PortalItem {
                            node: child_node.id,
                            element: ui_node_to_portal_element(tree, child_node),
                        });
                    }
                }
            }

            let root = PortalRoot {
                owner: node.id,
                key: overlay.key.clone(),
                layer: overlay.layer,
                rect: Rect::new(Point::new(0.0, 0.0), Size::new(0.0, 0.0)),
                modal: overlay.modal,
                children: PortalChildren::Items(children),
                computed: None,
                style: overlay.style,
                kind: overlay.kind,
                anchor_rect: overlay.anchor_rect,
                context_menu_anchor: overlay.context_menu_anchor,
                tree: None,
            };

            self.roots.push(root);
        }

        // Modal backdrop hit-test
        if let Some((backdrop_node, backdrop)) = overlays.modal_backdrop {
            self.roots.push(PortalRoot {
                owner: backdrop_node,
                key: Some("__modal_backdrop".to_string()),
                layer: LayerKind::Modal,
                rect: backdrop,
                modal: true,
                children: PortalChildren::Items(Vec::new()),
                computed: None,
                style: crate::core::Style::default(),
                kind: ElementKind::Primitive(crate::core::PrimitiveKind::Absolute),
                anchor_rect: backdrop,
                context_menu_anchor: None,
                tree: None,
            });
        }
    }

    pub fn paint_all(
        &self,
        viewport: Size,
        text: &mut crate::text_engine::TextSystem,
        display_list: &mut crate::core::DisplayList,
        hit_test: &mut HitTestTree,
        semantics: &mut SemanticTree,
        focusable_keys: &mut Vec<String>,
        overlay_focusable_keys: &mut Vec<String>,
        widget_kind_by_key: &mut std::collections::HashMap<String, WidgetKind>,
        theme: &Theme,
    ) {
        for root in &self.roots {
            if root.key.as_deref() == Some("__modal_backdrop") {
                // Only add backdrop hit-test, no paint (already handled in runtime)
                hit_test.push(
                    HitTestEntry::new(root.owner, root.rect, -1, root.layer)
                        .with_key(root.key.clone())
                        .with_order(usize::MAX),
                );
                continue;
            }

            display_list.push(PaintCommand::PushLayer(crate::core::LayerSpec::new(
                root.layer,
            )));

            if root.modal {
                // Dimmed backdrop
                display_list.push(PaintCommand::DrawRect(RectCmd {
                    rect: Rect::new(Point::new(0.0, 0.0), viewport),
                    paint: Paint::Solid(Color::rgba(0, 0, 0, 80)),
                    radius: 0.0,
                    opacity: 1.0,
                    z_index: 1000,
                }));
                // Centered modal panel
                let panel = root
                    .computed
                    .as_ref()
                    .map(|c| c.panel_rect)
                    .unwrap_or(root.rect);
                paint_panel(display_list, panel, 1001, root.modal, theme);
                // Hit-test for modal panel
                hit_test.push(
                    HitTestEntry::new(root.owner, panel, 1001, root.layer)
                        .with_key(root.key.clone())
                        .with_order(usize::MAX - 1),
                );
                // Paint children inside panel
                let content_rect = root
                    .computed
                    .as_ref()
                    .map(|c| c.content_rect)
                    .unwrap_or(panel);
                if let (Some(computed), Some(tree)) = (&root.computed, &root.tree) {
                    paint_portal_layout_node(
                        tree,
                        tree.root(),
                        &computed.layout,
                        content_rect.origin,
                        true, // skip_self = true
                        1003,
                        display_list,
                        hit_test,
                        semantics,
                        text,
                        focusable_keys,
                        overlay_focusable_keys,
                        widget_kind_by_key,
                        root.layer,
                        theme,
                    );
                }
            } else {
                let panel = root
                    .computed
                    .as_ref()
                    .map(|c| c.panel_rect)
                    .unwrap_or(root.rect);
                paint_panel(display_list, panel, 1000, root.modal, theme);
                // Hit-test for non-modal panel
                hit_test.push(
                    HitTestEntry::new(root.owner, panel, 1000, root.layer)
                        .with_key(root.key.clone())
                        .with_order(usize::MAX - 1),
                );
                let content_rect = root
                    .computed
                    .as_ref()
                    .map(|c| c.content_rect)
                    .unwrap_or(panel);
                if let (Some(computed), Some(tree)) = (&root.computed, &root.tree) {
                    paint_portal_layout_node(
                        tree,
                        tree.root(),
                        &computed.layout,
                        content_rect.origin,
                        true, // skip_self = true
                        1001,
                        display_list,
                        hit_test,
                        semantics,
                        text,
                        focusable_keys,
                        overlay_focusable_keys,
                        widget_kind_by_key,
                        root.layer,
                        theme,
                    );
                }
            }

            display_list.push(PaintCommand::PopLayer);
        }
    }
}

fn paint_panel(
    display_list: &mut crate::core::DisplayList,
    rect: Rect,
    z: i32,
    modal: bool,
    theme: &Theme,
) {
    let shadow_color = if theme.mode == ThemeMode::Dark {
        Color::rgba(0, 0, 0, 180)
    } else {
        Color::rgba(15, 23, 42, 80)
    };
    display_list.push(PaintCommand::DrawShadow(ShadowCmd {
        rect,
        color: shadow_color,
        blur_radius: 10.0,
        offset: Point::new(0.0, 4.0),
        z_index: z - 1,
    }));
    let radius = if modal {
        theme.radius.lg.top_left
    } else {
        theme.radius.md.top_left
    };
    display_list.push(PaintCommand::DrawRect(RectCmd {
        rect,
        paint: Paint::Solid(theme.colors.background.clone()),
        radius,
        opacity: 1.0,
        z_index: z,
    }));
    display_list.push(PaintCommand::DrawBorder(BorderCmd {
        rect,
        color: theme.colors.border.clone(),
        width: 1.0,
        radius,
        z_index: z + 1,
    }));
}

fn paint_portal_layout_node(
    tree: &UiTree,
    node_id: NodeId,
    layout: &crate::core::LayoutResult,
    parent_origin: Point,
    skip_self: bool,
    base_z: i32,
    display_list: &mut crate::core::DisplayList,
    hit_test: &mut HitTestTree,
    semantics: &mut SemanticTree,
    text: &mut crate::text_engine::TextSystem,
    focusable_keys: &mut Vec<String>,
    overlay_focusable_keys: &mut Vec<String>,
    widget_kind_by_key: &mut std::collections::HashMap<String, WidgetKind>,
    layer: LayerKind,
    theme: &Theme,
) {
    let Some(node) = tree.get(node_id) else {
        return;
    };
    let Some(layout_box) = layout.box_for_node(node_id) else {
        return;
    };
    let rect = Rect::new(
        Point::new(
            parent_origin.x + layout_box.local_rect.origin.x,
            parent_origin.y + layout_box.local_rect.origin.y,
        ),
        layout_box.local_rect.size,
    );

    if !skip_self {
        let element = ui_node_to_portal_element(tree, node);
        paint_portal_element(node.id, &element, rect, base_z, display_list, text, theme);
        push_portal_interaction(
            node,
            rect,
            base_z,
            layer,
            hit_test,
            semantics,
            focusable_keys,
            overlay_focusable_keys,
            widget_kind_by_key,
        );
    }

    for child in &node.children {
        paint_portal_layout_node(
            tree,
            *child,
            layout,
            rect.origin,
            false,
            base_z,
            display_list,
            hit_test,
            semantics,
            text,
            focusable_keys,
            overlay_focusable_keys,
            widget_kind_by_key,
            layer,
            theme,
        );
    }
}

fn push_portal_interaction(
    node: &UiNode,
    rect: Rect,
    base_z: i32,
    layer: LayerKind,
    hit_test: &mut HitTestTree,
    semantics: &mut SemanticTree,
    focusable_keys: &mut Vec<String>,
    overlay_focusable_keys: &mut Vec<String>,
    widget_kind_by_key: &mut std::collections::HashMap<String, WidgetKind>,
) {
    let key = node.key.as_ref().map(|key| key.as_str().to_string());
    if let Some(ref key) = key {
        hit_test.push(
            HitTestEntry::new(node.id, rect, base_z, layer)
                .with_key(Some(key.clone()))
                .with_order(usize::MAX - 2),
        );
    }

    let role = match node.kind {
        ElementKind::Text(_) => crate::core::Role::Text,
        ElementKind::Widget(WidgetKind::Button) => crate::core::Role::Button,
        ElementKind::Widget(WidgetKind::Input | WidgetKind::Textarea) => {
            crate::core::Role::TextInput
        }
        ElementKind::Widget(WidgetKind::Checkbox) => crate::core::Role::Checkbox,
        ElementKind::Widget(WidgetKind::Radio) => crate::core::Role::Radio,
        ElementKind::Widget(WidgetKind::Image) => crate::core::Role::Image,
        ElementKind::Widget(WidgetKind::Switch) => crate::core::Role::Switch,
        ElementKind::Widget(WidgetKind::Slider) => crate::core::Role::Slider,
        ElementKind::Widget(WidgetKind::ProgressBar) => crate::core::Role::ProgressBar,
        ElementKind::Widget(WidgetKind::Spinner) => crate::core::Role::Spinner,
        ElementKind::Widget(WidgetKind::Badge) => crate::core::Role::Badge,
        ElementKind::Widget(WidgetKind::Avatar) => crate::core::Role::Avatar,
        ElementKind::Widget(WidgetKind::Link) => crate::core::Role::Link,
        ElementKind::Widget(WidgetKind::Alert) => crate::core::Role::Alert,
        ElementKind::Widget(WidgetKind::Card) => crate::core::Role::Card,
        _ => crate::core::Role::Group,
    };
    let focusable = matches!(
        node.kind,
        ElementKind::Widget(
            WidgetKind::Button
                | WidgetKind::Input
                | WidgetKind::Textarea
                | WidgetKind::Switch
                | WidgetKind::Slider
                | WidgetKind::Link
        )
    );
    if let (Some(key), ElementKind::Widget(kind)) = (&key, &node.kind) {
        widget_kind_by_key.insert(key.clone(), *kind);
        if focusable {
            focusable_keys.push(key.clone());
            overlay_focusable_keys.push(key.clone());
        }
    }

    if key.is_some() || role != crate::core::Role::Group {
        semantics.push(SemanticNode {
            node: node.id,
            key,
            role,
            label: portal_label_for_node(node),
            description: None,
            value: portal_value_for_node(node),
            states: SemanticStates::default(),
            actions: Vec::new(),
            focusable,
            focus_order: None,
            keyboard_navigation: crate::core::KeyboardNav::None,
            bounds: rect,
        });
    }
}

fn paint_portal_element(
    node_id: NodeId,
    element: &Element,
    rect: Rect,
    z_index: i32,
    display_list: &mut crate::core::DisplayList,
    text: &mut crate::text_engine::TextSystem,
    theme: &Theme,
) {
    let node = portal_element_to_ui_node(node_id, element);
    let state = super::paint::visual_state_for_element(element);
    for painted in super::paint::paint_node_themed(&node, rect, z_index, &state, text, Some(theme))
    {
        display_list.push(painted.command);
    }
}

fn portal_element_to_ui_node(node_id: NodeId, element: &Element) -> UiNode {
    UiNode {
        id: node_id,
        key: element.key.clone(),
        parent: None,
        children: Vec::new(),
        kind: element.kind.clone(),
        widget_spec: element.widget_spec.clone(),
        style: element.style.clone(),
        variant: element.variant.clone(),
        checked: element.checked,
        default_checked: element.default_checked,
        semantic: element.semantic.clone(),
        handlers: EventHandlers::default(),
        overlay: None,
        open: element.open,
    }
}

fn overlay_children_to_portal_items(
    owner: NodeId,
    _overlay_key: Option<&str>,
    children: &[Element],
) -> Vec<PortalItem> {
    children
        .iter()
        .enumerate()
        .map(|(index, element)| PortalItem {
            node: crate::runtime::stable_portal_child_id(owner, element.key.as_ref(), index),
            element: element.clone(),
        })
        .collect()
}

fn portal_label_for_node(node: &UiNode) -> Option<String> {
    if let Some(spec) = node.widget_spec.as_ref() {
        match spec {
            crate::widgets::WidgetSpec::Button(spec) => return spec.label.clone(),
            crate::widgets::WidgetSpec::Input(spec) => return spec.aria_label.clone(),
            crate::widgets::WidgetSpec::Textarea(_) => return node.semantic.label.clone(),
            _ => {}
        }
    }

    match &node.kind {
        ElementKind::Text(spec) => Some(spec.text.clone()),
        ElementKind::Widget(WidgetKind::Button) => node
            .children
            .first()
            .and_then(|child| child_label_text(node, *child)),
        _ => node.semantic.label.clone(),
    }
}

fn child_label_text(_node: &UiNode, _child: NodeId) -> Option<String> {
    None
}

fn portal_value_for_node(node: &UiNode) -> Option<crate::core::SemanticValue> {
    match node.widget_spec.as_ref()? {
        crate::widgets::WidgetSpec::Input(spec) => spec
            .value
            .clone()
            .or_else(|| spec.default_value.clone())
            .filter(|value| !value.is_empty())
            .map(crate::core::SemanticValue::Text),
        crate::widgets::WidgetSpec::Textarea(spec) => spec
            .value
            .clone()
            .or_else(|| spec.default_value.clone())
            .filter(|value| !value.is_empty())
            .map(crate::core::SemanticValue::Text),
        _ => None,
    }
}

fn ui_node_to_portal_element(tree: &UiTree, node: &UiNode) -> Element {
    Element {
        key: node.key.clone(),
        kind: node.kind.clone(),
        widget_spec: node.widget_spec.clone(),
        children: node
            .children
            .iter()
            .filter_map(|child_id| tree.get(*child_id))
            .map(|child| ui_node_to_portal_element(tree, child))
            .collect(),
        style: node.style.clone(),
        variant: node.variant.clone(),
        checked: node.checked,
        default_checked: node.default_checked,
        semantic: node.semantic.clone(),
        event_handlers: crate::core::EventHandlers::default(),
        overlay: None,
        open: node.open,
    }
}
