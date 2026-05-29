use std::collections::{HashMap, HashSet};
use taffy::prelude::TaffyGridLine;

use crate::core::{
    Display, ElementKind, FlexDirection, FontStyle, FontWeight, LayoutBox, LayoutResult, NodeId,
    Point, PrimitiveKind, Rect, Size, WidgetKind,
};
use crate::text_engine::TextSystem;

use super::intrinsic::{WidgetIntrinsicInput, intrinsic_widget_size};
use super::taffy_mapping::to_taffy_style;

pub const LAYOUT_ENGINE_NAME: &str = "taffy_first";

#[derive(Clone, Copy, Default)]
pub struct LayoutCx<'a> {
    pub active_tab_by_key: Option<&'a HashMap<String, usize>>,
    pub scroll_offsets_by_key: Option<&'a HashMap<String, crate::Vec2>>,
}

impl LayoutCx<'_> {
    pub fn empty() -> Self {
        Self::default()
    }
}

#[derive(Clone, Debug)]
pub enum MeasureContext {
    Text {
        text: String,
        font_size: f32,
        font_weight: FontWeight,
        font_style: FontStyle,
    },
    Widget {
        widget_kind: WidgetKind,
        label: Option<String>,
        font_size: f32,
        font_weight: FontWeight,
        font_style: FontStyle,
    },
    Leaf,
}

pub struct TaffyLayoutBackend {
    taffy: taffy::TaffyTree<MeasureContext>,
    node_map: HashMap<NodeId, taffy::NodeId>,
    inverse_map: HashMap<taffy::NodeId, NodeId>,
    child_map: HashMap<NodeId, Vec<taffy::NodeId>>,
    pub dirty_layout_nodes: std::collections::HashSet<NodeId>,
}

impl Default for TaffyLayoutBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl TaffyLayoutBackend {
    pub fn new() -> Self {
        let mut taffy = taffy::TaffyTree::new();
        taffy.disable_rounding();
        Self {
            taffy,
            node_map: HashMap::new(),
            inverse_map: HashMap::new(),
            child_map: HashMap::new(),
            dirty_layout_nodes: std::collections::HashSet::new(),
        }
    }

    pub fn compute_incremental(
        &mut self,
        tree: &crate::runtime::UiTree,
        text_system: &mut TextSystem,
        viewport: Size,
        theme: &crate::Theme,
        dirty_nodes: &[NodeId],
    ) -> LayoutResult {
        self.compute_incremental_with_cx(
            tree,
            text_system,
            viewport,
            theme,
            &LayoutCx::empty(),
            dirty_nodes,
        )
    }

    pub fn compute_incremental_with_cx(
        &mut self,
        tree: &crate::runtime::UiTree,
        text_system: &mut TextSystem,
        viewport: Size,
        theme: &crate::Theme,
        layout_cx: &LayoutCx<'_>,
        dirty_nodes: &[NodeId],
    ) -> LayoutResult {
        let sync_dirty_nodes: HashSet<NodeId> = dirty_nodes.iter().copied().collect();
        let propagated_dirty_nodes = expand_dirty_with_ancestors(tree, dirty_nodes);
        self.dirty_layout_nodes = sync_dirty_nodes.clone();
        self.sync_incremental_tree(
            tree,
            text_system,
            viewport,
            theme,
            layout_cx,
            &sync_dirty_nodes,
        );
        let mut result = self.compute_synced_tree(tree, text_system, viewport, theme, true);
        result.debug.dirty_layout_node_count = propagated_dirty_nodes.len();
        result
    }

    pub fn build_from_tree(
        &mut self,
        tree: &crate::runtime::UiTree,
        text_system: &mut TextSystem,
        viewport: Size,
        theme: &crate::Theme,
    ) -> LayoutResult {
        self.build_from_tree_with_cx(tree, text_system, viewport, theme, &LayoutCx::empty())
    }

    pub fn build_from_tree_with_cx(
        &mut self,
        tree: &crate::runtime::UiTree,
        text_system: &mut TextSystem,
        viewport: Size,
        theme: &crate::Theme,
        layout_cx: &LayoutCx<'_>,
    ) -> LayoutResult {
        self.clear_taffy_state();
        self.sync_full_tree(tree, text_system, viewport, theme, layout_cx);
        self.compute_synced_tree(tree, text_system, viewport, theme, false)
    }

    fn clear_taffy_state(&mut self) {
        self.taffy = taffy::TaffyTree::new();
        self.taffy.disable_rounding();
        self.node_map.clear();
        self.inverse_map.clear();
        self.child_map.clear();
    }

    fn sync_full_tree(
        &mut self,
        tree: &crate::runtime::UiTree,
        text_system: &mut TextSystem,
        viewport: Size,
        theme: &crate::Theme,
        layout_cx: &LayoutCx<'_>,
    ) -> taffy::NodeId {
        let root = tree.root_node();
        self.build_node(tree, root, text_system, viewport, theme, layout_cx)
    }

    fn sync_incremental_tree(
        &mut self,
        tree: &crate::runtime::UiTree,
        text_system: &mut TextSystem,
        viewport: Size,
        theme: &crate::Theme,
        layout_cx: &LayoutCx<'_>,
        dirty_nodes: &HashSet<NodeId>,
    ) -> taffy::NodeId {
        if self.node_map.is_empty() {
            return self.sync_full_tree(tree, text_system, viewport, theme, layout_cx);
        }

        let current_nodes: HashSet<NodeId> = tree.nodes().iter().map(|node| node.id).collect();
        self.node_map.retain(|node, taffy_id| {
            let keep = current_nodes.contains(node);
            if !keep {
                self.inverse_map.remove(taffy_id);
                self.child_map.remove(node);
            }
            keep
        });

        self.sync_node_incremental(
            tree,
            tree.root_node(),
            text_system,
            viewport,
            theme,
            layout_cx,
            dirty_nodes,
        )
    }

    fn compute_synced_tree(
        &mut self,
        tree: &crate::runtime::UiTree,
        text_system: &mut TextSystem,
        viewport: Size,
        theme: &crate::Theme,
        incremental: bool,
    ) -> LayoutResult {
        let root = tree.root_node();
        let taffy_root = *self
            .node_map
            .get(&tree.root())
            .expect("root taffy node exists");

        let available = match root.kind {
            ElementKind::Widget(WidgetKind::Popover | WidgetKind::Tooltip | WidgetKind::Menu) => {
                taffy::Size {
                    width: taffy::AvailableSpace::MaxContent,
                    height: taffy::AvailableSpace::MaxContent,
                }
            }
            ElementKind::Widget(WidgetKind::Modal) => taffy::Size {
                width: taffy::AvailableSpace::Definite((viewport.width * 0.9).max(1.0)),
                height: taffy::AvailableSpace::Definite((viewport.height * 0.9).max(1.0)),
            },
            _ => taffy::Size {
                width: taffy::AvailableSpace::Definite(viewport.width),
                height: taffy::AvailableSpace::Definite(viewport.height),
            },
        };

        let layout_status = self.taffy.compute_layout_with_measure(
            taffy_root,
            available,
            |known: taffy::Size<Option<f32>>,
             space: taffy::Size<taffy::AvailableSpace>,
             _node_id: taffy::NodeId,
             ctx: Option<&mut MeasureContext>,
             _style: &taffy::Style| {
                if let Some(ctx) = ctx {
                    measure_callback(ctx, known, space, text_system, theme)
                } else {
                    taffy::Size {
                        width: 0.0,
                        height: 0.0,
                    }
                }
            },
        );

        let mut result = LayoutResult::default();
        if let Err(error) = layout_status {
            result.diagnostics.layout_errors.push(format!("{error:?}"));
            result.debug.engine = "taffy_first".to_string();
            result.debug.taffy_node_count = self.node_map.len();
            result.debug.dirty_layout_node_count = 0;
            result.debug.layout_error_count = result.diagnostics.layout_errors.len();
            result.debug.layout_warning_count = result.diagnostics.layout_warnings.len();
            if incremental {
                result.debug.incremental_layout_count = 1;
            } else {
                result.debug.full_rebuild_count = 1;
            }
            return result;
        }
        for node in tree.nodes() {
            let rgui_id = node.id;
            let Some(&taffy_id) = self.node_map.get(&rgui_id) else {
                continue;
            };
            let Ok(layout) = self.taffy.layout(taffy_id) else {
                continue;
            };
            let rect = Rect::new(
                Point::new(layout.location.x, layout.location.y),
                Size::new(layout.size.width, layout.size.height),
            );
            let content_rect = Rect::new(
                Point::new(
                    rect.origin.x + layout.border.left + layout.padding.left,
                    rect.origin.y + layout.border.top + layout.padding.top,
                ),
                Size::new(
                    layout.content_box_width().max(0.0),
                    layout.content_box_height().max(0.0),
                ),
            );
            let padding_rect = Rect::new(
                Point::new(
                    rect.origin.x + layout.border.left,
                    rect.origin.y + layout.border.top,
                ),
                Size::new(
                    (rect.size.width - layout.border.left - layout.border.right).max(0.0),
                    (rect.size.height - layout.border.top - layout.border.bottom).max(0.0),
                ),
            );
            let content_size = content_rect.size;
            let mut layout_box = LayoutBox::new(rgui_id, rect).with_content_size(content_size);
            layout_box.padding_rect = padding_rect;
            layout_box.content_rect = content_rect;
            if let Some(node) = tree.get(rgui_id) {
                if let Some(key) = node.key.as_ref() {
                    layout_box = layout_box.with_key(key.as_str());
                }
                if clips_overflow_node(node) {
                    layout_box = layout_box.with_clip(rect);
                }
            }
            result.push(layout_box);
        }

        for node in tree.nodes() {
            if matches!(node.style.position, Some(crate::core::Position::Fixed)) {
                result
                    .diagnostics
                    .layout_warnings
                    .push("position=fixed currently behaves like absolute".to_string());
            }
        }
        let enriched = enrich_world_rects(tree, &result, tree.root());
        let mut enriched = apply_scroll_content_sizes(tree, &enriched);
        enriched.debug.engine = "taffy_first".to_string();
        enriched.debug.taffy_node_count = self.node_map.len();
        enriched.debug.dirty_layout_node_count = 0;
        enriched.debug.layout_error_count = enriched.diagnostics.layout_errors.len();
        enriched.debug.layout_warning_count = enriched.diagnostics.layout_warnings.len();
        if incremental {
            enriched.debug.incremental_layout_count = 1;
        } else {
            enriched.debug.full_rebuild_count = 1;
        }
        enriched
    }

    fn sync_node_incremental(
        &mut self,
        tree: &crate::runtime::UiTree,
        node: &crate::runtime::UiNode,
        text_system: &mut TextSystem,
        viewport: Size,
        theme: &crate::Theme,
        layout_cx: &LayoutCx<'_>,
        dirty_nodes: &HashSet<NodeId>,
    ) -> taffy::NodeId {
        let dirty = dirty_nodes.contains(&node.id);
        let child_taffy_nodes = self.sync_taffy_children(
            tree,
            node,
            text_system,
            viewport,
            theme,
            layout_cx,
            dirty_nodes,
        );
        let children_changed = self.child_map.get(&node.id).map(|old| old.as_slice())
            != Some(child_taffy_nodes.as_slice());

        if let Some(existing) = self.node_map.get(&node.id).copied() {
            if should_taffy_layout_children(node) {
                if dirty || children_changed {
                    let style = compute_taffy_style_for_node(tree, node, viewport, theme);
                    self.taffy
                        .set_style(existing, style)
                        .expect("update taffy style");
                    self.taffy
                        .set_children(existing, &child_taffy_nodes)
                        .expect("update taffy children");
                    self.child_map.insert(node.id, child_taffy_nodes);
                }
                return existing;
            }
            if dirty {
                let style = compute_taffy_style_for_node(tree, node, viewport, theme);
                let ctx = measure_context_for_node(tree, node, theme);
                let replacement = self
                    .taffy
                    .new_leaf_with_context(style, ctx)
                    .expect("replace dirty taffy leaf");
                self.node_map.insert(node.id, replacement);
                self.inverse_map.remove(&existing);
                self.inverse_map.insert(replacement, node.id);
                return replacement;
            }
            existing
        } else if should_taffy_layout_children(node) {
            let style = compute_taffy_style_for_node(tree, node, viewport, theme);
            let taffy_id = self
                .taffy
                .new_with_children(style, &child_taffy_nodes)
                .expect("create taffy container");
            self.node_map.insert(node.id, taffy_id);
            self.inverse_map.insert(taffy_id, node.id);
            self.child_map.insert(node.id, child_taffy_nodes);
            taffy_id
        } else {
            let style = compute_taffy_style_for_node(tree, node, viewport, theme);
            let ctx = measure_context_for_node(tree, node, theme);
            let taffy_id = self
                .taffy
                .new_leaf_with_context(style, ctx)
                .expect("create taffy leaf");
            self.node_map.insert(node.id, taffy_id);
            self.inverse_map.insert(taffy_id, node.id);
            taffy_id
        }
    }

    fn sync_taffy_children(
        &mut self,
        tree: &crate::runtime::UiTree,
        node: &crate::runtime::UiNode,
        text_system: &mut TextSystem,
        viewport: Size,
        theme: &crate::Theme,
        layout_cx: &LayoutCx<'_>,
        dirty_nodes: &HashSet<NodeId>,
    ) -> Vec<taffy::NodeId> {
        layout_child_ids_for_node(node, layout_cx)
            .iter()
            .map(|child_id| {
                let child = tree.get(*child_id).expect("child exists");
                self.sync_node_incremental(
                    tree,
                    child,
                    text_system,
                    viewport,
                    theme,
                    layout_cx,
                    dirty_nodes,
                )
            })
            .collect()
    }

    fn build_node(
        &mut self,
        tree: &crate::runtime::UiTree,
        node: &crate::runtime::UiNode,
        _text_system: &mut TextSystem,
        _viewport: Size,
        theme: &crate::Theme,
        layout_cx: &LayoutCx<'_>,
    ) -> taffy::NodeId {
        let measure_ctx = measure_context_for_node(tree, node, theme);

        let taffy_style = compute_taffy_style_for_node(tree, node, _viewport, theme);

        let taffy_id = if should_taffy_layout_children(node) {
            let children: Vec<taffy::NodeId> = layout_child_ids_for_node(node, layout_cx)
                .iter()
                .map(|child_id| {
                    let child = tree.get(*child_id).expect("child exists");
                    self.build_node(tree, child, _text_system, _viewport, theme, layout_cx)
                })
                .collect();

            self.taffy
                .new_with_children(taffy_style, &children)
                .unwrap()
        } else {
            self.taffy
                .new_leaf_with_context(taffy_style, measure_ctx)
                .unwrap()
        };

        self.node_map.insert(node.id, taffy_id);
        self.inverse_map.insert(taffy_id, node.id);
        if should_taffy_layout_children(node) {
            let children = self.taffy.children(taffy_id).unwrap_or_default();
            self.child_map.insert(node.id, children);
        }
        taffy_id
    }
}

fn layout_child_ids_for_node(
    node: &crate::runtime::UiNode,
    layout_cx: &LayoutCx<'_>,
) -> Vec<NodeId> {
    if !should_taffy_layout_children(node) {
        return Vec::new();
    }

    if matches!(node.kind, ElementKind::Widget(WidgetKind::Tabs)) {
        let active_idx = node
            .key
            .as_ref()
            .and_then(|key| {
                layout_cx
                    .active_tab_by_key
                    .and_then(|tabs| tabs.get(key.as_str()).copied())
            })
            .unwrap_or_else(|| {
                if let Some(crate::widgets::WidgetSpec::Tabs(ref spec)) = node.widget_spec {
                    spec.active_index.unwrap_or(0)
                } else {
                    0
                }
            });
        return node.children.get(active_idx).copied().into_iter().collect();
    }

    node.children.clone()
}

fn expand_dirty_with_ancestors(tree: &crate::runtime::UiTree, dirty: &[NodeId]) -> HashSet<NodeId> {
    let mut expanded = HashSet::new();
    for mut id in dirty.iter().copied() {
        loop {
            if !expanded.insert(id) {
                break;
            }
            let Some(node) = tree.get(id) else {
                break;
            };
            let Some(parent) = node.parent else {
                break;
            };
            id = parent;
        }
    }
    expanded
}

fn enrich_world_rects(
    tree: &crate::runtime::UiTree,
    raw: &LayoutResult,
    root: NodeId,
) -> LayoutResult {
    let mut result = LayoutResult {
        diagnostics: raw.diagnostics.clone(),
        debug: raw.debug.clone(),
        ..LayoutResult::default()
    };

    fn visit(
        tree: &crate::runtime::UiTree,
        raw: &LayoutResult,
        result: &mut LayoutResult,
        node_id: NodeId,
        parent_world_origin: Point,
        parent_clip: Option<Rect>,
    ) {
        let Some(local) = raw.box_for_node(node_id).cloned() else {
            return;
        };
        let delta = crate::Vec2::new(parent_world_origin.x, parent_world_origin.y);
        let world_rect = local.local_rect.translate(delta);

        let mut out = local;
        out.world_rect = world_rect;
        if let Some(local_clip) = out.clip_rect {
            let world_clip = local_clip.translate(delta);
            out.clip_rect = parent_clip
                .and_then(|parent| parent.intersect(world_clip))
                .or(Some(world_clip));
        } else {
            out.clip_rect = parent_clip;
        }

        let next_origin = world_rect.origin;
        let next_clip = out.clip_rect;
        result.push(out);

        if let Some(node) = tree.get(node_id) {
            for child_id in &node.children {
                visit(tree, raw, result, *child_id, next_origin, next_clip);
            }
        }
    }

    visit(tree, raw, &mut result, root, Point::new(0.0, 0.0), None);
    result
}

fn apply_scroll_content_sizes(
    tree: &crate::runtime::UiTree,
    result: &LayoutResult,
) -> LayoutResult {
    let mut out = LayoutResult {
        diagnostics: result.diagnostics.clone(),
        debug: result.debug.clone(),
        ..LayoutResult::default()
    };
    for layout_box in &result.boxes {
        let mut layout_box = layout_box.clone();
        if let Some(node) = tree.get(layout_box.node) {
            layout_box.content_size = scrollable_content_size(tree, node, &layout_box, result);
        }
        out.push(layout_box);
    }
    out
}

fn taffy_style_for_node(
    node: &crate::runtime::UiNode,
    viewport: Size,
    theme: &crate::Theme,
) -> taffy::Style {
    let mut style = node.style.clone();
    match node.kind {
        ElementKind::Primitive(PrimitiveKind::Row) => {
            style.display = Some(Display::Flex);
            style.flex_direction = Some(FlexDirection::Row);
            if style.align_items.is_none() {
                style.align_items = Some(crate::core::Align::Start);
            }
        }
        ElementKind::Primitive(PrimitiveKind::Column)
        | ElementKind::Primitive(PrimitiveKind::ScrollArea) => {
            style.display = Some(Display::Flex);
            style.flex_direction = Some(FlexDirection::Column);
            if style.align_items.is_none() {
                style.align_items = Some(crate::core::Align::Start);
            }
        }
        ElementKind::Primitive(PrimitiveKind::Grid) => {
            style.display = Some(Display::Grid);
        }
        ElementKind::Primitive(PrimitiveKind::Stack) => {
            style.display = Some(Display::Grid);
            if style.grid_template_columns.is_none() {
                style.grid_template_columns = Some(vec![crate::core::GridTrack::Auto]);
            }
            if style.grid_template_rows.is_none() {
                style.grid_template_rows = Some(vec![crate::core::GridTrack::Auto]);
            }
        }
        ElementKind::Primitive(PrimitiveKind::Absolute) => {
            style.display = Some(Display::Block);
        }
        _ => {}
    }

    let mut taffy_style = to_taffy_style(&style);
    if matches!(node.kind, ElementKind::Widget(WidgetKind::Tabs)) {
        taffy_style.display = taffy::Display::Flex;
        taffy_style.flex_direction = taffy::FlexDirection::Column;
        taffy_style.padding.top = max_length_percentage(
            taffy_style.padding.top,
            theme.widgets.metrics.tabs.tab_height,
        );
    }
    if node.parent.is_none() {
        let is_overlay = matches!(
            node.kind,
            ElementKind::Widget(
                WidgetKind::Popover | WidgetKind::Modal | WidgetKind::Tooltip | WidgetKind::Menu
            )
        );
        if !is_overlay {
            if node.style.width.is_none() {
                taffy_style.size.width = taffy::Dimension::length(viewport.width);
            }
            if node.style.height.is_none() {
                taffy_style.size.height = taffy::Dimension::length(viewport.height);
            }
        } else {
            match node.kind {
                ElementKind::Widget(WidgetKind::Modal) => {
                    if node.style.max_width.is_none() {
                        taffy_style.max_size.width = taffy::Dimension::length(
                            480.0_f32.min((viewport.width - 32.0).max(1.0)),
                        );
                    }
                    if node.style.max_height.is_none() {
                        taffy_style.max_size.height =
                            taffy::Dimension::length((viewport.height - 32.0).max(1.0));
                    }
                }
                ElementKind::Widget(WidgetKind::Popover) => {
                    if node.style.max_width.is_none() {
                        taffy_style.max_size.width =
                            taffy::Dimension::length((viewport.width - 16.0).max(1.0));
                    }
                }
                ElementKind::Widget(WidgetKind::Menu) => {
                    if node.style.max_height.is_none() {
                        taffy_style.max_size.height =
                            taffy::Dimension::length((viewport.height - 16.0).max(1.0));
                    }
                }
                _ => {}
            }
        }
    }
    taffy_style
}

fn compute_taffy_style_for_node(
    tree: &crate::runtime::UiTree,
    node: &crate::runtime::UiNode,
    viewport: Size,
    theme: &crate::Theme,
) -> taffy::Style {
    let mut taffy_style = taffy_style_for_node(node, viewport, theme);

    if let Some(parent_id) = node.parent {
        if let Some(parent) = tree.get(parent_id) {
            let is_column = matches!(
                parent.kind,
                ElementKind::Primitive(PrimitiveKind::Column)
                    | ElementKind::Primitive(PrimitiveKind::ScrollArea)
            );
            let is_row = matches!(parent.kind, ElementKind::Primitive(PrimitiveKind::Row));
            let parent_scrollable = if is_column {
                matches!(
                    parent.style.overflow_y,
                    Some(crate::core::Overflow::Scroll | crate::core::Overflow::Auto)
                )
            } else if is_row {
                matches!(
                    parent.style.overflow_x,
                    Some(crate::core::Overflow::Scroll | crate::core::Overflow::Auto)
                )
            } else {
                false
            };
            if parent_scrollable && node.style.flex_shrink.is_none() {
                taffy_style.flex_shrink = 0.0;
            }

            if matches!(parent.kind, ElementKind::Primitive(PrimitiveKind::Stack)) {
                let is_absolute = matches!(
                    node.style.position,
                    Some(crate::core::Position::Absolute | crate::core::Position::Fixed)
                );
                if !is_absolute {
                    taffy_style.grid_row = taffy::Line {
                        start: taffy::GridPlacement::from_line_index(1),
                        end: taffy::GridPlacement::Auto,
                    };
                    taffy_style.grid_column = taffy::Line {
                        start: taffy::GridPlacement::from_line_index(1),
                        end: taffy::GridPlacement::Auto,
                    };
                }
            }
        }
    }

    taffy_style
}

fn max_length_percentage(
    current: taffy::LengthPercentage,
    minimum: f32,
) -> taffy::LengthPercentage {
    let current_auto = taffy::LengthPercentageAuto::from(current);
    let current_is_percent = current_auto
        .resolve_to_option(100.0, |_, _| 0.0)
        .zip(current_auto.resolve_to_option(200.0, |_, _| 0.0))
        .is_some_and(|(a, b)| (a - b).abs() > f32::EPSILON);

    if current_is_percent {
        return current;
    }

    let current_value = current_auto
        .resolve_to_option(0.0, |_, _| 0.0)
        .unwrap_or(0.0);
    taffy::LengthPercentage::length(current_value.max(minimum))
}

fn measure_context_for_node(
    tree: &crate::runtime::UiTree,
    node: &crate::runtime::UiNode,
    theme: &crate::Theme,
) -> MeasureContext {
    match &node.kind {
        ElementKind::Text(spec) => MeasureContext::Text {
            text: spec.text.clone(),
            font_size: node
                .style
                .text
                .as_ref()
                .and_then(|t| t.size.resolve(theme.typography.body_size))
                .filter(|s| *s > 0.0)
                .unwrap_or(theme.typography.body_size),
            font_weight: node
                .style
                .text
                .as_ref()
                .map(|text| text.weight)
                .unwrap_or(FontWeight::Normal),
            font_style: node
                .style
                .text
                .as_ref()
                .map(|text| text.style)
                .unwrap_or(FontStyle::Normal),
        },
        ElementKind::Widget(kind) => MeasureContext::Widget {
            widget_kind: *kind,
            label: label_text_for_node(tree, node).map(str::to_string),
            font_size: node
                .style
                .text
                .as_ref()
                .and_then(|t| t.size.resolve(theme.typography.body_size))
                .filter(|s| *s > 0.0)
                .unwrap_or(theme.typography.body_size),
            font_weight: node
                .style
                .text
                .as_ref()
                .map(|text| text.weight)
                .unwrap_or(FontWeight::Normal),
            font_style: node
                .style
                .text
                .as_ref()
                .map(|text| text.style)
                .unwrap_or(FontStyle::Normal),
        },
        ElementKind::Primitive(_) => MeasureContext::Leaf,
        _ => MeasureContext::Leaf,
    }
}

fn clips_overflow_node(node: &crate::runtime::UiNode) -> bool {
    matches!(
        (node.style.overflow_x, node.style.overflow_y),
        (
            Some(
                crate::core::Overflow::Hidden
                    | crate::core::Overflow::Clip
                    | crate::core::Overflow::Scroll
                    | crate::core::Overflow::Auto
            ),
            _
        ) | (
            _,
            Some(
                crate::core::Overflow::Hidden
                    | crate::core::Overflow::Clip
                    | crate::core::Overflow::Scroll
                    | crate::core::Overflow::Auto
            )
        )
    )
}

fn scrollable_content_size(
    _tree: &crate::runtime::UiTree,
    node: &crate::runtime::UiNode,
    layout_box: &LayoutBox,
    result: &LayoutResult,
) -> Size {
    let mut bounds: Option<Rect> = None;
    for child_id in &node.children {
        if let Some(child) = result.box_for_node(*child_id) {
            bounds = Some(match bounds {
                Some(current) => current.union(child.local_rect),
                None => child.local_rect,
            });
        }
    }
    let Some(bounds) = bounds else {
        return layout_box.content_size;
    };
    Size::new(
        bounds.max_x().max(layout_box.content_size.width).max(0.0),
        bounds.max_y().max(layout_box.content_size.height).max(0.0),
    )
}

fn measure_callback(
    ctx: &mut MeasureContext,
    known: taffy::Size<Option<f32>>,
    space: taffy::Size<taffy::AvailableSpace>,
    text_system: &mut TextSystem,
    theme: &crate::Theme,
) -> taffy::Size<f32> {
    match ctx {
        MeasureContext::Text {
            text,
            font_size,
            font_weight,
            font_style,
        } => {
            let resolved_font_size = (*font_size).max(1.0);
            let layout = if let Some(width) = known.width {
                text_system.measure_wrapped(
                    text,
                    resolved_font_size,
                    *font_weight,
                    *font_style,
                    width.max(resolved_font_size),
                )
            } else {
                match space.width {
                    taffy::AvailableSpace::Definite(w) => text_system.measure_wrapped(
                        text,
                        resolved_font_size,
                        *font_weight,
                        *font_style,
                        w.max(resolved_font_size),
                    ),
                    taffy::AvailableSpace::MinContent => text_system.measure_wrapped(
                        text,
                        resolved_font_size,
                        *font_weight,
                        *font_style,
                        resolved_font_size,
                    ),
                    taffy::AvailableSpace::MaxContent => text_system.measure_intrinsic(
                        text,
                        resolved_font_size,
                        *font_weight,
                        *font_style,
                    ),
                }
            };
            let resolved_width = known.width.unwrap_or(layout.width);
            taffy::Size {
                width: resolved_width.max(resolved_font_size),
                height: known
                    .height
                    .unwrap_or(layout.height.max(resolved_font_size)),
            }
        }
        MeasureContext::Widget {
            widget_kind,
            label,
            font_size,
            font_weight,
            font_style,
        } => {
            let label_width = label.as_deref().map(|l| {
                text_system
                    .measure_intrinsic(l, *font_size, *font_weight, *font_style)
                    .width
            });
            let size = intrinsic_widget_size(
                WidgetIntrinsicInput {
                    widget_kind: *widget_kind,
                    label_width,
                    known_width: known.width,
                    known_height: known.height,
                },
                &theme.widgets.metrics,
            );
            taffy::Size {
                width: size.width,
                height: size.height,
            }
        }
        MeasureContext::Leaf => taffy::Size {
            width: known.width.unwrap_or(0.0),
            height: known.height.unwrap_or(0.0),
        },
    }
}

fn should_taffy_layout_children(node: &crate::runtime::UiNode) -> bool {
    !node.children.is_empty()
        && !matches!(
            node.kind,
            ElementKind::Widget(
                WidgetKind::Button
                    | WidgetKind::Input
                    | WidgetKind::Checkbox
                    | WidgetKind::Radio
                    | WidgetKind::Select
                    | WidgetKind::Textarea
            )
        )
}

fn label_text_for_node<'a>(
    tree: &'a crate::runtime::UiTree,
    node: &'a crate::runtime::UiNode,
) -> Option<&'a str> {
    if let Some(ref spec) = node.widget_spec {
        match spec {
            crate::widgets::WidgetSpec::Button(bs) => {
                if let Some(ref l) = bs.label {
                    return Some(l.as_str());
                }
            }
            crate::widgets::WidgetSpec::Checkbox(cs) => {
                if let Some(ref l) = cs.label {
                    return Some(l.as_str());
                }
            }
            crate::widgets::WidgetSpec::Radio(rs) => {
                if let Some(ref l) = rs.label {
                    return Some(l.as_str());
                }
            }
            crate::widgets::WidgetSpec::Tooltip(ts) => {
                if let Some(ref l) = ts.text {
                    return Some(l.as_str());
                }
            }
            crate::widgets::WidgetSpec::Icon(is) => {
                return Some(is.name.as_str());
            }
            crate::widgets::WidgetSpec::Modal(ms) => {
                if let Some(ref l) = ms.title {
                    return Some(l.as_str());
                }
            }
            _ => {}
        }
    }
    if let Some(ref label) = node.semantic.label {
        return Some(label.as_str());
    }
    // Fallback is only for intrinsic sizing of legacy builder patterns. Prefer explicit widget labels.
    node.children
        .iter()
        .filter_map(|id| tree.get(*id))
        .find_map(|child| {
            if let ElementKind::Text(spec) = &child.kind {
                Some(spec.text.as_str())
            } else {
                None
            }
        })
}
