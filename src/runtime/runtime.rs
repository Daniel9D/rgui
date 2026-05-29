use std::collections::{HashMap, HashSet};

use crate::core::{
    ClipSpec, DisplayList, ElementKind, HitTestEntry, HitTestTree, LayerKind, LayoutBoxSnapshot,
    MeasureSnapshot, NodeId, Overflow, OverlaySnapshot, PaintCommand, PerformanceMetrics, Point,
    Rect, RenderStats, ResolvedStyleSnapshot, ResourceStore, Role, SemanticNode, SemanticSnapshot,
    SemanticStates, SemanticTree, SemanticValue, Size, Theme, UiEvent, UiSnapshot, Vec2,
    WidgetKind,
};
use crate::layout::{LAYOUT_ENGINE_NAME, LayoutCx, TaffyLayoutBackend};
use crate::state::{ButtonState, CheckboxState, InputState, StateArena};
use crate::text_engine::TextSystem;

use super::{
    CommandQueue, FocusSystem, FrameInput, FrameOutput, Reconciler, UiCommand, UiNode, UiTree,
    paint, stable_portal_child_id,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResolvedLayout {
    pub rect: Rect,
    pub clip_rect: Option<Rect>,
    pub scroll_offset: Vec2,
    pub content_size: Size,
}

fn clips_overflow_node(node: &UiNode) -> bool {
    matches!(
        (node.style.overflow_x, node.style.overflow_y),
        (
            Some(Overflow::Hidden | Overflow::Clip | Overflow::Scroll | Overflow::Auto),
            _
        ) | (
            _,
            Some(Overflow::Hidden | Overflow::Clip | Overflow::Scroll | Overflow::Auto)
        )
    )
}

fn scroll_offset_for_node(
    node: &UiNode,
    rect: Rect,
    content_size: Size,
    scroll_offsets_by_key: &HashMap<String, Vec2>,
) -> Vec2 {
    if !clips_overflow_node(node) {
        return Vec2::default();
    }

    let requested = node
        .key
        .as_ref()
        .and_then(|key| scroll_offsets_by_key.get(key.as_str()))
        .copied()
        .unwrap_or_default();
    Vec2::new(
        requested
            .x
            .clamp(0.0, (content_size.width - rect.size.width).max(0.0)),
        requested
            .y
            .clamp(0.0, (content_size.height - rect.size.height).max(0.0)),
    )
}

fn text_hit_geometry_for_widget(
    kind: WidgetKind,
    rect: Rect,
    theme: &Theme,
) -> (Point, f32, crate::core::ResolvedWidgetStyle) {
    let style = theme.resolve_widget_style(kind, None, &crate::ResolvedStateFlags::default());
    let metrics = &theme.widgets.metrics;
    let (horizontal_padding, vertical_padding) = match kind {
        WidgetKind::Textarea => (
            metrics.textarea.horizontal_padding,
            metrics.textarea.vertical_padding,
        ),
        _ => (
            metrics.input.horizontal_padding,
            metrics.input.vertical_padding,
        ),
    };
    (
        Point::new(
            rect.origin.x + horizontal_padding,
            rect.origin.y + vertical_padding,
        ),
        (rect.size.width - horizontal_padding * 2.0).max(0.0),
        style,
    )
}

#[derive(Default)]
pub struct UiRuntime {
    reconciler: Reconciler,
    text_system: TextSystem,
    tree: Option<UiTree>,
    key_to_node: HashMap<String, NodeId>,
    node_to_key: HashMap<NodeId, String>,
    scroll_offsets_by_key: HashMap<String, crate::core::Vec2>,
    scroll_bounds_by_key: HashMap<String, crate::core::Vec2>,
    scroll_rects_by_key: HashMap<String, Rect>,
    viewport: Size,
    last_hit_test: HitTestTree,
    active_key: Option<String>,
    focused_key: Option<String>,
    focusable_keys: Vec<String>,
    overlay_focusable_keys: Vec<String>,
    widget_kind_by_key: HashMap<String, WidgetKind>,
    click_action_by_key: HashMap<String, String>,
    selected_index_by_key: HashMap<String, usize>,
    selected_value_by_key: HashMap<String, String>,
    active_index_by_key: HashMap<String, usize>,
    tree_expanded_by_key: HashMap<String, HashMap<usize, bool>>,
    table_selected_row_by_key: HashMap<String, usize>,
    list_selected_index_by_key: HashMap<String, usize>,
    widget_rect_by_key: HashMap<String, Rect>,
    open_select_key: Option<String>,
    disabled_select_options_by_key: HashMap<String, HashSet<usize>>,
    command_queue: CommandQueue,
    command_handlers: HashMap<String, Box<dyn Fn(&str) + Send + Sync>>,
    bool_state_by_key: HashMap<String, bool>,
    bool_state_by_node: HashMap<NodeId, bool>,
    focused_node: Option<NodeId>,
    active_node: Option<NodeId>,
    hovered_key: Option<String>,
    hovered_node: Option<NodeId>,
    pointer_capture: Option<NodeId>,
    pointer_capture_key: Option<String>,
    open_context_menu_key: Option<String>,
    context_menu_anchor: Option<Point>,
    open_overlay_keys: Vec<String>,
    open_overlay_rects: Vec<(String, Rect)>,
    has_open_modal: bool,
    dismissed_overlay_keys: HashSet<String>,
    opened_overlay_keys: HashSet<String>,
    theme: Theme,
    layout_backend: TaffyLayoutBackend,
    focus_system: FocusSystem,
    state_arena: StateArena,
    pub a11y_backend: Option<Box<dyn crate::core::AccessibilityBackend>>,
    a11y_update_count: usize,
    drag_source_key: Option<String>,
    drag_source_node: Option<NodeId>,
    drag_payload: Option<String>,
    drag_origin: Option<Point>,
    drag_started: bool,
}

impl UiRuntime {
    pub fn set_scroll_offset_for_key(&mut self, key: impl Into<String>, offset: crate::core::Vec2) {
        self.scroll_offsets_by_key.insert(key.into(), offset);
    }

    pub fn scroll_offset(&self, key: &str) -> Option<crate::core::Vec2> {
        self.scroll_offsets_by_key.get(key).copied()
    }

    pub fn capture_pointer(&mut self, key: &str) {
        self.pointer_capture_key = Some(key.to_string());
        self.pointer_capture = self.node_for_key(key);
    }

    pub fn release_pointer(&mut self) {
        self.pointer_capture = None;
        self.pointer_capture_key = None;
    }

    pub fn pointer_capture_key(&self) -> Option<String> {
        self.pointer_capture_key.clone()
    }

    pub fn node_for_key(&self, key: &str) -> Option<NodeId> {
        self.key_to_node.get(key).copied()
    }

    pub fn key_for_node(&self, node: NodeId) -> Option<&str> {
        self.node_to_key.get(&node).map(String::as_str)
    }

    pub fn tree(&self) -> Option<&UiTree> {
        self.tree.as_ref()
    }

    pub fn active_key(&self) -> Option<String> {
        self.active_key.clone()
    }

    pub fn focused_key(&self) -> Option<String> {
        self.focused_key.clone()
    }

    pub fn hovered_key(&self) -> Option<String> {
        self.hovered_key.clone()
    }

    pub fn command_count(&self) -> usize {
        self.command_queue.count()
    }

    pub fn selected_index(&self, key: &str) -> Option<usize> {
        self.selected_index_by_key.get(key).copied()
    }

    pub fn selected_value(&self, key: &str) -> Option<String> {
        self.selected_value_by_key.get(key).cloned()
    }

    pub fn active_index(&self, key: &str) -> Option<usize> {
        self.active_index_by_key.get(key).copied()
    }

    pub fn tree_item_expanded(&self, key: &str, index: usize) -> Option<bool> {
        self.tree_expanded_by_key
            .get(key)
            .and_then(|items| items.get(&index))
            .copied()
            .or_else(|| {
                let node = self.node_for_key(key)?;
                let node = self.tree.as_ref()?.get(node)?;
                let crate::widgets::WidgetSpec::Tree(spec) = node.widget_spec.as_ref()? else {
                    return None;
                };
                spec.items.get(index).map(|item| item.expanded)
            })
    }

    pub fn table_selected_row(&self, key: &str) -> Option<usize> {
        self.table_selected_row_by_key
            .get(key)
            .copied()
            .or_else(|| {
                let node = self.node_for_key(key)?;
                let node = self.tree.as_ref()?.get(node)?;
                let crate::widgets::WidgetSpec::Table(spec) = node.widget_spec.as_ref()? else {
                    return None;
                };
                spec.selected_row
            })
    }

    pub fn list_selected_index(&self, key: &str) -> Option<usize> {
        self.list_selected_index_by_key
            .get(key)
            .copied()
            .or_else(|| {
                let node = self.node_for_key(key)?;
                let node = self.tree.as_ref()?.get(node)?;
                let crate::widgets::WidgetSpec::List(spec) = node.widget_spec.as_ref()? else {
                    return None;
                };
                spec.selected_index
            })
    }

    pub fn drain_commands(&mut self) -> Vec<UiCommand> {
        self.command_queue.drain()
    }

    pub fn on(
        &mut self,
        action: impl Into<String>,
        handler: impl Fn(&str) + Send + Sync + 'static,
    ) {
        self.command_handlers
            .insert(action.into(), Box::new(handler));
    }

    pub fn flush_command_handlers(&self) {
        for command in self.command_queue.commands() {
            if let UiCommand::Click {
                key,
                action: Some(action),
            } = command
            {
                if let Some(handler) = self.command_handlers.get(action) {
                    handler(key.as_deref().unwrap_or(""));
                }
            }
        }
    }

    pub fn bool_state(&self, key: &str) -> Option<bool> {
        self.node_for_key(key)
            .and_then(|node| self.bool_state_by_node.get(&node).copied())
            .or_else(|| {
                self.node_for_key(key)
                    .and_then(|node| self.state_arena.get::<CheckboxState>(node))
                    .map(|state| state.checked)
            })
            .or_else(|| self.bool_state_by_key.get(key).copied())
    }

    pub fn text_state(&self, key: &str) -> Option<String> {
        self.node_for_key(key).and_then(|node| {
            self.state_arena.get::<InputState>(node).and_then(|state| {
                if state.text.is_empty() {
                    None
                } else {
                    Some(state.text.clone())
                }
            })
        })
    }

    pub fn text_cursor(&self, key: &str) -> Option<usize> {
        self.node_for_key(key)
            .and_then(|node| self.state_arena.get::<InputState>(node))
            .map(|state| state.cursor)
    }

    pub fn set_text_selection_for_key(&mut self, key: &str, range: std::ops::Range<usize>) {
        if let Some(node) = self.node_for_key(key) {
            if let Some(state) = self.state_arena.get_mut::<InputState>(node) {
                let start = range.start.min(state.text.len());
                let end = range.end.min(state.text.len());
                state.cursor = end;
                state.selection = crate::core::TextSelection {
                    anchor: crate::core::TextPosition::new(start),
                    head: crate::core::TextPosition::new(end),
                };
            }
        }
    }

    pub fn debug_legacy_text_state_count(&self) -> usize {
        0
    }

    pub fn dispatch(&mut self, event: UiEvent) {
        match event {
            UiEvent::PointerDown(pointer) => self.handle_pointer_down(pointer),
            UiEvent::PointerUp(pointer) => self.handle_pointer_up(pointer),
            UiEvent::PointerMove(pointer) => self.handle_pointer_move(pointer),
            UiEvent::TextInput(text) | UiEvent::ImeCommit(text) => self.handle_text_input(text),
            UiEvent::ImePreedit(preedit) => self.handle_ime_preedit(preedit),
            UiEvent::KeyDown(key) => self.handle_key_down(key),
            UiEvent::Wheel(wheel) => self.handle_wheel(wheel),
            other => self.handle_misc_event(other),
        }
    }

    fn handle_pointer_down(&mut self, pointer: crate::core::PointerEvent) {
        if let Some(hit) = self.last_hit_test.hit(pointer.position) {
            if self
                .tree
                .as_ref()
                .and_then(|tree| tree.get(hit.node))
                .is_some_and(node_is_disabled)
            {
                self.active_node = None;
                self.active_key = None;
                return;
            }
            self.active_node = Some(hit.node);
            self.active_key = hit.key.clone();
            if pointer.button == Some(crate::core::PointerButton::Secondary) {
                if self.node_has_context_menu(hit.node) {
                    self.open_context_menu_key = hit
                        .key
                        .clone()
                        .or_else(|| self.key_for_node(hit.node).map(str::to_string));
                    self.context_menu_anchor = Some(pointer.position);
                    return;
                }
            } else if pointer.button == Some(crate::core::PointerButton::Primary)
                && !self.open_overlay_rects.is_empty()
                && self
                    .open_overlay_rects
                    .iter()
                    .all(|(_, rect)| !rect.contains(pointer.position))
            {
                self.open_context_menu_key = None;
                self.context_menu_anchor = None;
                self.open_select_key = None;
            }
            if let Some(key) = hit.key.clone() {
                if key.ends_with("::__scrollbar_thumb") {
                    self.capture_pointer(&key);
                    return;
                }
            }
            // Click-to-focus for focusable widgets
            let hit_key = hit
                .key
                .clone()
                .or_else(|| self.key_for_node(hit.node).map(str::to_string));
            if let Some(key) = hit_key {
                let kind = self.widget_kind_by_key.get(&key).copied();
                if matches!(
                    kind,
                    Some(
                        WidgetKind::Input
                            | WidgetKind::Textarea
                            | WidgetKind::Button
                            | WidgetKind::Checkbox
                            | WidgetKind::Radio
                            | WidgetKind::Select
                    )
                ) {
                    self.focused_key = Some(key.clone());
                    self.focused_node = Some(hit.node);
                    self.key_to_node.insert(key.clone(), hit.node);
                    self.node_to_key.insert(hit.node, key.clone());

                    if matches!(kind, Some(WidgetKind::Input | WidgetKind::Textarea)) {
                        if !self.state_arena.contains::<InputState>(hit.node) {
                            self.state_arena.insert(hit.node, InputState::new(None));
                        }
                        if let Some(state) = self.state_arena.get_mut::<InputState>(hit.node) {
                            let (text_top_left, measure_width, style) =
                                text_hit_geometry_for_widget(kind.unwrap(), hit.rect, &self.theme);
                            let layout = self.text_system.measure(
                                &state.text,
                                style.font_size,
                                style.font_weight,
                                crate::core::FontStyle::Normal,
                                measure_width,
                            );
                            let new_cursor =
                                layout.caret_index_for_point(pointer.position, text_top_left);
                            state.cursor = new_cursor;
                            state.selection = crate::core::TextSelection::caret(
                                crate::core::TextPosition::new(new_cursor),
                            );
                        }
                    }
                }
            }
        } else {
            self.active_node = None;
            self.active_key = None;
        }
        // Initialize drag state if the hit node is draggable
        if let Some(hit) = self.last_hit_test.hit(pointer.position) {
            if let Some(node) = self.tree.as_ref().and_then(|tree| tree.get(hit.node)) {
                if node.handlers.draggable_payload.is_some() {
                    self.drag_source_key = hit.key.clone();
                    self.drag_source_node = Some(hit.node);
                    self.drag_payload = node.handlers.draggable_payload.clone();
                    self.drag_origin = Some(pointer.position);
                    self.drag_started = false;
                }
            }
        }
        if !self.open_overlay_rects.is_empty()
            && self
                .open_overlay_rects
                .iter()
                .all(|(_, rect)| !rect.contains(pointer.position))
        {
            for (key, _) in &self.open_overlay_rects {
                let can_close = self
                    .modal_policy_for_key(key)
                    .map(|(_, close_on_outside_click)| close_on_outside_click)
                    .unwrap_or(true);
                if can_close {
                    self.opened_overlay_keys.remove(key);
                    self.dismissed_overlay_keys.insert(key.clone());
                }
            }
            self.open_select_key = None;
        }
    }

    fn handle_pointer_up(&mut self, pointer: crate::core::PointerEvent) {
        if self.pointer_capture_key.is_some() {
            self.release_pointer();
            return;
        }
        let hit = self
            .last_hit_test
            .hit(pointer.position)
            .map(|entry| (entry.node, entry.key.clone()));
        if let Some((hit_node, hit_key)) = hit {
            if Some(hit_node) == self.active_node {
                if self
                    .tree
                    .as_ref()
                    .and_then(|tree| tree.get(hit_node))
                    .is_some_and(node_is_disabled)
                {
                    self.active_node = None;
                    self.active_key = None;
                    return;
                }
                let key = hit_key
                    .or_else(|| self.key_for_node(hit_node).map(str::to_string))
                    .or_else(|| self.interactive_ancestor_key(hit_node));
                if let Some(key) = key {
                    if let Some((select_key, option_index)) = parse_select_option_key(&key) {
                        self.select_option(select_key, option_index);
                        return;
                    }
                    // Check for explicit on_click action
                    if let Some(action) = self.click_action_by_key.get(&key) {
                        self.command_queue.push(UiCommand::Click {
                            key: Some(key.clone()),
                            action: Some(action.clone()),
                        });
                    } else {
                        match self.widget_kind_by_key.get(&key).copied() {
                            Some(WidgetKind::Select) => {
                                self.toggle_select_options(&key);
                            }
                            Some(WidgetKind::Tabs) => {
                                self.update_tabs_from_hit(&key, pointer.position);
                            }
                            Some(WidgetKind::Tree) => {
                                self.update_tree_from_hit(&key, pointer.position);
                            }
                            Some(WidgetKind::Table) => {
                                self.update_table_from_hit(&key, pointer.position);
                            }
                            Some(WidgetKind::List) => {
                                self.update_list_from_hit(&key, pointer.position);
                            }
                            Some(WidgetKind::Button) => {
                                self.toggle_button_popover(hit_node);
                                self.command_queue.push(UiCommand::Click {
                                    key: Some(key.clone()),
                                    action: None,
                                });
                            }
                            Some(WidgetKind::Checkbox) => {
                                let current = self
                                    .bool_state_by_node
                                    .get(&hit_node)
                                    .copied()
                                    .or_else(|| self.bool_state_by_key.get(&key).copied())
                                    .unwrap_or(false);
                                self.bool_state_by_node.insert(hit_node, !current);
                                self.bool_state_by_key.insert(key.clone(), !current);

                                // Also update arena state
                                if let Some(state) =
                                    self.state_arena.get_mut::<CheckboxState>(hit_node)
                                {
                                    state.checked = !current;
                                }
                                self.command_queue.push(UiCommand::SetBool {
                                    key: key.clone(),
                                    value: !current,
                                });
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        self.active_key = None;
        self.active_node = None;
        // Emit drag end and clear drag state
        if self.drag_origin.is_some() {
            self.command_queue.push(UiCommand::DragEnd {
                key: self.drag_source_key.clone(),
                position: pointer.position,
            });
            self.drag_source_key = None;
            self.drag_source_node = None;
            self.drag_payload = None;
            self.drag_origin = None;
            self.drag_started = false;
        }
    }

    fn handle_pointer_move(&mut self, pointer: crate::core::PointerEvent) {
        if let Some(key) = self.pointer_capture_key.clone() {
            if let Some(scroll_key) = key.strip_suffix("::__scrollbar_thumb") {
                self.drag_scrollbar_to(scroll_key, pointer.position);
                return;
            }
        }
        // Emit drag events when pointer moves far enough from drag origin
        if let Some(origin) = self.drag_origin {
            let dx = pointer.position.x - origin.x;
            let dy = pointer.position.y - origin.y;
            if dx.abs() > 4.0 || dy.abs() > 4.0 {
                if !self.drag_started {
                    self.command_queue.push(UiCommand::DragStart {
                        key: self.drag_source_key.clone(),
                        payload: self.drag_payload.clone(),
                    });
                    self.drag_started = true;
                }
                self.command_queue.push(UiCommand::DragMove {
                    key: self.drag_source_key.clone(),
                    position: pointer.position,
                });
                return;
            }
        }
        let hit = self
            .last_hit_test
            .hit(pointer.position)
            .map(|entry| (entry.node, entry.key.clone()));
        if let Some((node, key)) = hit {
            self.hovered_node = Some(node);
            self.hovered_key = key.or_else(|| self.key_for_node(node).map(str::to_string));
        } else {
            self.hovered_node = None;
            self.hovered_key = None;
        }
    }

    fn handle_text_input(&mut self, text: String) {
        if let Some(key) = self.focused_key.clone() {
            if matches!(
                self.widget_kind_by_key.get(&key),
                Some(WidgetKind::Input | WidgetKind::Textarea)
            ) {
                if let Some(node) = self.node_for_key(&key) {
                    if let Some(state) = self.state_arena.get_mut::<InputState>(node) {
                        state.commit_text(&text);
                        state.preedit = None;
                    }
                }
            }
        }
    }

    fn handle_ime_preedit(&mut self, preedit: crate::core::ImePreedit) {
        if let Some(key) = self.focused_key.clone() {
            if matches!(
                self.widget_kind_by_key.get(&key),
                Some(WidgetKind::Input | WidgetKind::Textarea)
            ) {
                if let Some(node) = self.node_for_key(&key) {
                    if let Some(state) = self.state_arena.get_mut::<InputState>(node) {
                        if preedit.text.is_empty() {
                            state.preedit = None;
                        } else {
                            state.preedit = Some(preedit.clone());
                        }
                    }
                }
            }
        }
    }

    fn handle_key_down(&mut self, key_event: crate::core::KeyEvent) {
        if let Some(focused_node) = self.focused_node {
            if self
                .tree
                .as_ref()
                .and_then(|tree| tree.get(focused_node))
                .is_some_and(node_is_disabled)
            {
                return;
            }
        }
        let key_str = key_event.key.as_str();
        let modifiers = key_event.modifiers;
        if (modifiers & 2 != 0 || modifiers & 8 != 0)
            && (key_str.eq_ignore_ascii_case("c")
                || key_str.eq_ignore_ascii_case("x")
                || key_str.eq_ignore_ascii_case("v"))
        {
            if let Some(focused_key) = self.focused_key.clone() {
                if matches!(
                    self.widget_kind_by_key.get(&focused_key),
                    Some(WidgetKind::Input | WidgetKind::Textarea)
                ) {
                    if let Some(node) = self.node_for_key(&focused_key) {
                        if let Some(state) = self.state_arena.get_mut::<InputState>(node) {
                            let key_lower = key_str.to_lowercase();
                            if key_lower == "c" || key_lower == "x" {
                                let range = state.selection.range();
                                let selected_text = state.text[range.start..range.end].to_string();
                                if !selected_text.is_empty() {
                                    if let Ok(mut ctx) = arboard::Clipboard::new() {
                                        let _ = ctx.set_text(selected_text);
                                    }
                                }
                                if key_lower == "x" {
                                    state.text.replace_range(range.start..range.end, "");
                                    state.cursor = range.start;
                                    state.selection = crate::core::TextSelection::caret(
                                        crate::core::TextPosition::new(range.start),
                                    );
                                }
                            } else if key_lower == "v" {
                                if let Ok(mut ctx) = arboard::Clipboard::new() {
                                    if let Ok(pasted_text) = ctx.get_text() {
                                        state.commit_text(&pasted_text);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            return;
        }
        match key_str {
            "Tab" => {
                let next = self.focus_system.tab_forward();
                if let Some((_node, key_opt)) = next {
                    self.focused_key = key_opt;
                    self.focused_node = self
                        .focused_key
                        .as_deref()
                        .and_then(|key| self.node_for_key(key));
                }
            }
            "Escape" => {
                // Dismiss only the topmost overlay (last painted)
                if let Some(topmost) = self.open_overlay_keys.last().cloned() {
                    let can_close = self
                        .modal_policy_for_key(&topmost)
                        .map(|(close_on_escape, _)| close_on_escape)
                        .unwrap_or(true);
                    if can_close {
                        self.opened_overlay_keys.remove(&topmost);
                        self.dismissed_overlay_keys.insert(topmost);
                    }
                }
            }
            "Backspace" | "Delete" => {
                if let Some(key) = self.focused_key.clone() {
                    if matches!(
                        self.widget_kind_by_key.get(&key),
                        Some(WidgetKind::Input | WidgetKind::Textarea)
                    ) {
                        if let Some(node) = self.node_for_key(&key) {
                            if let Some(state) = self.state_arena.get_mut::<InputState>(node) {
                                if key_str == "Backspace" {
                                    state.delete_before();
                                } else {
                                    state.delete_after();
                                }
                            }
                        }
                    }
                }
            }
            "ArrowLeft" | "ArrowRight" => {
                if let Some(key) = self.focused_key.clone() {
                    if matches!(
                        self.widget_kind_by_key.get(&key),
                        Some(WidgetKind::Input | WidgetKind::Textarea)
                    ) {
                        if let Some(node) = self.node_for_key(&key) {
                            if let Some(state) = self.state_arena.get_mut::<InputState>(node) {
                                if key_str == "ArrowLeft" {
                                    state.move_cursor_left();
                                } else {
                                    state.move_cursor_right();
                                }
                            }
                        }
                    }
                }
            }
            "Home" | "End" => {
                if let Some(key) = self.focused_key.clone() {
                    if matches!(
                        self.widget_kind_by_key.get(&key),
                        Some(WidgetKind::Input | WidgetKind::Textarea)
                    ) {
                        if let Some(node) = self.node_for_key(&key) {
                            if let Some(state) = self.state_arena.get_mut::<InputState>(node) {
                                if key_str == "Home" {
                                    state.move_cursor_home();
                                } else {
                                    state.move_cursor_end();
                                }
                            }
                        }
                    }
                }
            }
            " " => {
                if let Some(key) = self.focused_key.clone() {
                    if let Some(kind) = self.widget_kind_by_key.get(&key).copied() {
                        match kind {
                            WidgetKind::Checkbox => {
                                if let Some(node) = self.node_for_key(&key) {
                                    let current = self
                                        .bool_state_by_node
                                        .get(&node)
                                        .copied()
                                        .unwrap_or(false);
                                    self.bool_state_by_node.insert(node, !current);
                                    self.bool_state_by_key.insert(key.clone(), !current);
                                    if let Some(state) =
                                        self.state_arena.get_mut::<CheckboxState>(node)
                                    {
                                        state.checked = !current;
                                    }
                                    self.command_queue.push(UiCommand::SetBool {
                                        key: key.clone(),
                                        value: !current,
                                    });
                                }
                            }
                            WidgetKind::Button => {
                                self.command_queue.push(UiCommand::Click {
                                    key: Some(key.clone()),
                                    action: None,
                                });
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_wheel(&mut self, wheel: crate::core::WheelEvent) {
        if let Some(hit_node) = self.last_hit_test.hit_test(wheel.position) {
            if let Some(scrollable) = self.find_scrollable_ancestor(hit_node) {
                if let Some(ref tree) = self.tree {
                    if let Some(key) = tree.get(scrollable).and_then(|n| n.key.as_ref()) {
                        let normalized = match wheel.mode {
                            crate::core::WheelDeltaMode::Pixels => wheel.delta,
                            crate::core::WheelDeltaMode::Lines => {
                                Vec2::new(wheel.delta.x * 20.0, wheel.delta.y * 20.0)
                            }
                            crate::core::WheelDeltaMode::Pages => Vec2::new(
                                wheel.delta.x * self.viewport.width * 0.85,
                                wheel.delta.y * self.viewport.height * 0.85,
                            ),
                        };
                        let max_scroll = self
                            .scroll_bounds_by_key
                            .get(key.as_str())
                            .copied()
                            .unwrap_or_default();
                        let entry = self
                            .scroll_offsets_by_key
                            .entry(key.as_str().to_string())
                            .or_default();
                        entry.x = (entry.x + normalized.x).clamp(0.0, max_scroll.x);
                        entry.y = (entry.y + normalized.y).clamp(0.0, max_scroll.y);
                    }
                }
            }
        }
    }

    fn handle_misc_event(&mut self, _event: UiEvent) {}

    pub fn update(&mut self, input: FrameInput) -> FrameOutput {
        self.viewport = input.viewport;
        self.theme = input.theme.clone();
        let mut root = input.root;
        apply_widget_part_layout_styles(&mut root, &input.theme);
        let reconciled = self.reconciler.reconcile_with_dirty(root);
        let layout_recompute_count = reconciled
            .dirty_entries()
            .iter()
            .filter(|(_, dirty)| dirty.contains(crate::runtime::DirtyFlags::LAYOUT))
            .count();
        let tree = reconciled.tree.clone();
        let mut dirty_layout_nodes = Vec::new();
        for (key, dirty) in reconciled.dirty_entries() {
            if dirty.contains(crate::runtime::DirtyFlags::LAYOUT) {
                if let Some(node) = tree.node_for_key(key.as_str()) {
                    dirty_layout_nodes.extend(tree.ancestors_inclusive(node));
                }
            }
        }
        dirty_layout_nodes.sort_by_key(|node| node.raw());
        dirty_layout_nodes.dedup();
        self.rebuild_key_maps(&tree);
        self.focused_node = self
            .focused_key
            .as_deref()
            .and_then(|key| self.node_for_key(key));
        self.active_node = self
            .active_key
            .as_deref()
            .and_then(|key| self.node_for_key(key));
        self.hovered_node = self
            .hovered_key
            .as_deref()
            .and_then(|key| self.node_for_key(key));
        self.seed_checked_state(&tree);
        self.seed_select_state(&tree);
        self.seed_disabled_select_options(&tree);
        self.seed_collection_state(&tree);
        self.tree = Some(tree.clone());

        let layout_cx = LayoutCx {
            active_tab_by_key: Some(&self.active_index_by_key),
            scroll_offsets_by_key: Some(&self.scroll_offsets_by_key),
        };
        let taffy_layout = Some(self.layout_backend.compute_incremental_with_cx(
            &tree,
            &mut self.text_system,
            input.viewport,
            &input.theme,
            &layout_cx,
            &dirty_layout_nodes,
        ));

        let mut builder = FrameBuilder::new(
            &mut self.text_system,
            input.viewport,
            taffy_layout.as_ref(),
            &self.scroll_offsets_by_key,
            &self.dismissed_overlay_keys,
            &self.opened_overlay_keys,
            &self.bool_state_by_key,
            &self.bool_state_by_node,
            &self.selected_index_by_key,
            &self.tree_expanded_by_key,
            &self.table_selected_row_by_key,
            &self.list_selected_index_by_key,
            &self.active_index_by_key,
            self.focused_node,
            self.active_node,
            self.hovered_node,
            &self.state_arena,
            &input.theme,
            self.open_context_menu_key.as_deref(),
            self.context_menu_anchor,
            self.open_select_key.as_deref(),
        );
        builder.push_tree(&tree);
        let estimated_render_item_count = estimate_render_item_count(&builder.display_list);
        let stats = RenderStats {
            command_count: builder.display_list.commands().len(),
            batch_count: 0,
            atlas_upload_bytes: 0,
            render_item_count: estimated_render_item_count,
            text_item_count: builder
                .display_list
                .commands()
                .iter()
                .filter(|command| matches!(command, PaintCommand::DrawText(_)))
                .count(),
            clip_batch_count: 0,
            glyphon_enabled: false,
            text_area_count: 0,
            clipped_text_area_count: 0,
            skipped_text_area_count: 0,
            glyph_count: 0,
            fallback_used: false,
        };
        if let Some(backend) = self.a11y_backend.as_mut() {
            backend.update(&builder.semantics);
            self.a11y_update_count += 1;
        }

        if let Some(layout) = taffy_layout.as_ref() {
            builder.snapshot.diagnostics.layout_errors = layout.diagnostics.layout_errors.clone();
            builder.snapshot.diagnostics.layout_warnings =
                layout.diagnostics.layout_warnings.clone();
            builder.snapshot.layout_debug = layout.debug.clone();
        }

        builder.snapshot.performance = PerformanceMetrics {
            node_count: builder.snapshot.tree_nodes.len(),
            layout_recompute_count,
            display_command_count: stats.command_count,
            accessibility: crate::core::AccessibilityMetrics {
                semantic_node_count: builder.semantics.nodes().len(),
                accesskit_update_count: self.a11y_update_count,
            },
            ..PerformanceMetrics::default()
        };
        self.last_hit_test = builder.hit_test.clone();
        self.focusable_keys = builder.focusable_keys.clone();
        self.overlay_focusable_keys = builder.overlay_focusable_keys.clone();
        self.widget_kind_by_key = builder.widget_kind_by_key.clone();
        self.click_action_by_key = builder.click_action_by_key.clone();
        self.widget_rect_by_key = builder.widget_rect_by_key.clone();
        self.has_open_modal = builder.has_open_modal;
        self.open_overlay_keys = builder.open_overlay_keys.clone();
        self.open_overlay_rects = builder.open_overlay_rects.clone();
        self.scroll_bounds_by_key = builder.scroll_bounds_by_key.clone();
        self.scroll_rects_by_key = builder.scroll_rects_by_key.clone();
        for semantic_node in builder.semantics.nodes() {
            if let Some(key) = semantic_node.key.as_ref() {
                self.key_to_node
                    .entry(key.clone())
                    .or_insert(semantic_node.node);
                self.node_to_key
                    .entry(semantic_node.node)
                    .or_insert_with(|| key.clone());
            }
        }
        if builder.overlay_focusable_keys.is_empty() {
            self.focus_system.activate_document_scope();
        } else {
            let overlay_scope = self.focus_system.replace_overlay_scope();
            if let Some(scope) = self.focus_system.scope_mut(overlay_scope) {
                for key in &builder.overlay_focusable_keys {
                    if let Some(node) = self.key_to_node.get(key).copied() {
                        scope.push_entry(node, Some(key.clone()), super::TabIndex::Auto);
                    }
                }
            }
            self.focus_system.set_active(overlay_scope);
        }

        let mut frame_commands = CommandQueue::default();
        std::mem::swap(&mut self.command_queue, &mut frame_commands);

        let output = FrameOutput {
            display_list: builder.display_list,
            resources: ResourceStore::default(),
            semantics: builder.semantics,
            hit_test: builder.hit_test,
            stats,
            layout_engine: LAYOUT_ENGINE_NAME,
            commands: frame_commands,
            snapshot: Some(builder.snapshot),
        };
        let open_now: std::collections::HashSet<String> =
            self.open_overlay_keys.iter().cloned().collect();
        self.dismissed_overlay_keys
            .retain(|key| open_now.contains(key));
        if std::env::var_os("RGUI_DUMP_FRAME").is_some() {
            eprintln!(
                "{}",
                crate::runtime::debug::format_frame_dump(&output, true)
            );
        }
        output
    }

    fn find_scrollable_ancestor(&self, node_id: NodeId) -> Option<NodeId> {
        let tree = self.tree.as_ref()?;
        let mut current = Some(node_id);
        while let Some(id) = current {
            let node = tree.get(id)?;
            if matches!(
                (node.style.overflow_x, node.style.overflow_y),
                (
                    Some(crate::core::Overflow::Scroll | crate::core::Overflow::Auto),
                    _,
                ) | (
                    _,
                    Some(crate::core::Overflow::Scroll | crate::core::Overflow::Auto),
                )
            ) {
                return Some(id);
            }
            current = node.parent;
        }
        None
    }

    fn node_has_context_menu(&self, node_id: NodeId) -> bool {
        self.tree
            .as_ref()
            .and_then(|tree| tree.get(node_id))
            .and_then(|node| node.overlay.as_ref())
            .is_some_and(|overlay| matches!(overlay.kind, ElementKind::Widget(WidgetKind::Menu)))
    }

    fn toggle_button_popover(&mut self, node_id: NodeId) {
        let Some(node) = self.tree.as_ref().and_then(|tree| tree.get(node_id)) else {
            return;
        };
        let Some(overlay) = node.overlay.as_ref() else {
            return;
        };
        if !matches!(
            overlay.kind,
            ElementKind::Widget(WidgetKind::Popover | WidgetKind::Tooltip)
        ) {
            return;
        }
        let key = overlay
            .key
            .as_ref()
            .map(|key| key.as_str().to_string())
            .or_else(|| node.key.as_ref().map(|key| key.as_str().to_string()));
        let Some(key) = key else {
            return;
        };
        if self.opened_overlay_keys.remove(&key) {
            self.dismissed_overlay_keys.insert(key);
        } else {
            self.dismissed_overlay_keys.remove(&key);
            self.opened_overlay_keys.insert(key);
        }
    }

    fn interactive_ancestor_key(&self, node_id: NodeId) -> Option<String> {
        let tree = self.tree.as_ref()?;
        let mut current = tree.get(node_id).and_then(|node| node.parent);
        while let Some(id) = current {
            let node = tree.get(id)?;
            if node.key.is_some()
                && matches!(
                    node.kind,
                    ElementKind::Widget(
                        WidgetKind::Select
                            | WidgetKind::Tabs
                            | WidgetKind::Tree
                            | WidgetKind::Table
                            | WidgetKind::List
                            | WidgetKind::Menu
                    )
                )
            {
                return node.key.as_ref().map(|key| key.as_str().to_string());
            }
            current = node.parent;
        }
        None
    }

    fn toggle_select_options(&mut self, key: &str) {
        if self.open_select_key.as_deref() == Some(key) {
            self.open_select_key = None;
        } else {
            self.open_select_key = Some(key.to_string());
        }
    }

    fn select_option(&mut self, key: &str, selected_index: usize) {
        if self
            .disabled_select_options_by_key
            .get(key)
            .is_some_and(|disabled| disabled.contains(&selected_index))
        {
            self.open_select_key = None;
            return;
        }
        self.selected_index_by_key
            .insert(key.to_string(), selected_index);
        if let Some(value) = self
            .node_for_key(key)
            .and_then(|node| self.tree.as_ref().and_then(|tree| tree.get(node)))
            .and_then(|node| match node.widget_spec.as_ref()? {
                crate::widgets::WidgetSpec::Select(spec) => spec.options.get(selected_index),
                _ => None,
            })
            .map(|option| option.value.clone())
        {
            self.selected_value_by_key.insert(key.to_string(), value);
        }
        self.open_select_key = None;
    }

    fn modal_policy_for_key(&self, key: &str) -> Option<(bool, bool)> {
        let node = self.node_for_key(key)?;
        let node = self.tree.as_ref()?.get(node)?;
        let crate::widgets::WidgetSpec::Modal(spec) = node.widget_spec.as_ref()? else {
            return None;
        };
        Some((spec.close_on_escape, spec.close_on_outside_click))
    }

    fn update_tabs_from_hit(&mut self, key: &str, position: Point) {
        let Some(rect) = self.widget_rect_by_key.get(key).copied() else {
            return;
        };
        let tabs = match self
            .node_for_key(key)
            .and_then(|node| self.tree.as_ref().and_then(|tree| tree.get(node)))
            .and_then(|node| node.widget_spec.as_ref())
        {
            Some(crate::widgets::WidgetSpec::Tabs(spec)) if !spec.tabs.is_empty() => {
                spec.tabs.clone()
            }
            _ => vec!["Tab1".to_string(), "Tab2".to_string(), "Tab3".to_string()],
        };

        let rects = super::paint::calculate_tab_rects_with_metrics(
            &tabs,
            rect.origin,
            &mut self.text_system,
            &self.theme.widgets.metrics,
        );
        for (i, tab_rect) in rects.iter().enumerate() {
            let hit_rect = Rect::new(
                Point::new(tab_rect.origin.x, rect.origin.y),
                Size::new(
                    tab_rect.size.width,
                    self.theme.widgets.metrics.tabs.min_size.height,
                ),
            );
            if hit_rect.contains(position) {
                self.active_index_by_key.insert(key.to_string(), i);
                break;
            }
        }
    }

    fn update_tree_from_hit(&mut self, key: &str, position: Point) {
        let Some(rect) = self.widget_rect_by_key.get(key).copied() else {
            return;
        };
        let tree_metrics = self.theme.widgets.metrics.tree;
        let index = ((position.y - rect.origin.y - tree_metrics.indent * 0.5)
            / tree_metrics.row_height)
            .floor()
            .max(0.0) as usize;
        let current = self.tree_item_expanded(key, index).unwrap_or(false);
        self.tree_expanded_by_key
            .entry(key.to_string())
            .or_default()
            .insert(index, !current);
    }

    fn update_table_from_hit(&mut self, key: &str, position: Point) {
        let Some(rect) = self.widget_rect_by_key.get(key).copied() else {
            return;
        };
        let row_count = self
            .node_for_key(key)
            .and_then(|node| self.tree.as_ref().and_then(|tree| tree.get(node)))
            .and_then(|node| match node.widget_spec.as_ref()? {
                crate::widgets::WidgetSpec::Table(spec) => Some(spec.rows.len()),
                _ => None,
            })
            .unwrap_or(0);
        if row_count == 0 {
            return;
        }
        let table_metrics = self.theme.widgets.metrics.table;
        let row = ((position.y - rect.origin.y - table_metrics.cell_padding)
            / table_metrics.row_height)
            .floor() as isize
            - 1;
        if row >= 0 {
            self.table_selected_row_by_key
                .insert(key.to_string(), (row as usize).min(row_count - 1));
        }
    }

    fn update_list_from_hit(&mut self, key: &str, position: Point) {
        let Some(rect) = self.widget_rect_by_key.get(key).copied() else {
            return;
        };
        let item_count = self
            .node_for_key(key)
            .and_then(|node| self.tree.as_ref().and_then(|tree| tree.get(node)))
            .and_then(|node| match node.widget_spec.as_ref()? {
                crate::widgets::WidgetSpec::List(spec) => Some(spec.items.len().max(1)),
                _ => None,
            })
            .unwrap_or(1);
        let list_metrics = self.theme.widgets.metrics.list;
        let index = ((position.y - rect.origin.y - list_metrics.item_padding)
            / list_metrics.row_height)
            .floor()
            .max(0.0) as usize;
        self.list_selected_index_by_key
            .insert(key.to_string(), index.min(item_count - 1));
    }

    fn drag_scrollbar_to(&mut self, scroll_key: &str, position: Point) {
        let Some(rect) = self.scroll_rects_by_key.get(scroll_key).copied() else {
            return;
        };
        let max_scroll = self
            .scroll_bounds_by_key
            .get(scroll_key)
            .copied()
            .unwrap_or_default();
        let track = self.theme.widgets.metrics.scrollbar.track_rect(rect);
        let track_y = track.origin.y;
        let track_h = track.size.height;
        let ratio = ((position.y - track_y) / track_h).clamp(0.0, 1.0);
        self.scroll_offsets_by_key
            .insert(scroll_key.to_string(), Vec2::new(0.0, max_scroll.y * ratio));
    }

    fn rebuild_key_maps(&mut self, tree: &UiTree) {
        self.key_to_node.clear();
        self.node_to_key.clear();

        let doc_scope = self.focus_system.create_document_scope();
        self.focus_system.set_active(doc_scope);

        if let Some(scope) = self.focus_system.scope_mut(doc_scope) {
            for node in tree.nodes() {
                if let Some(key) = node.key.as_ref() {
                    let key = key.as_str().to_string();
                    self.key_to_node.insert(key.clone(), node.id);
                    self.node_to_key.insert(node.id, key);
                }
                if matches!(
                    node.kind,
                    ElementKind::Widget(
                        WidgetKind::Button
                            | WidgetKind::Input
                            | WidgetKind::Checkbox
                            | WidgetKind::Radio
                            | WidgetKind::Select
                            | WidgetKind::Textarea
                    )
                ) && !node_is_disabled(node)
                {
                    scope.push_entry(
                        node.id,
                        node.key.as_ref().map(|k| k.as_str().to_string()),
                        super::TabIndex::Auto,
                    );
                }
            }
        }
    }

    fn seed_checked_state(&mut self, tree: &UiTree) {
        for node in tree.nodes() {
            // Use `default_checked` as the uncontrolled seed value.
            // `checked` is the controlled override and is applied at render time.
            if let Some(seed) = node.default_checked.or(node.checked) {
                self.bool_state_by_node.entry(node.id).or_insert(seed);
                if let Some(key) = node.key.as_ref() {
                    self.bool_state_by_key
                        .entry(key.as_str().to_string())
                        .or_insert(seed);
                }
            }


            // Seed typed state in arena for supported widgets
            let widget_kind = match &node.kind {
                ElementKind::Widget(kind) => Some(*kind),
                _ => None,
            };
            match widget_kind {
                Some(WidgetKind::Checkbox) => {
                    if !self.state_arena.contains::<CheckboxState>(node.id) {
                        self.state_arena.insert(
                            node.id,
                            CheckboxState {
                                checked: node.checked.unwrap_or(false),
                                indeterminate: false,
                            },
                        );
                    }
                }
                Some(WidgetKind::Input | WidgetKind::Textarea) => {
                    if !self.state_arena.contains::<InputState>(node.id) {
                        let default = key_default_value(tree, node);
                        self.state_arena
                            .insert(node.id, InputState::new(default.as_deref()));
                    }
                }
                Some(WidgetKind::Button) => {
                    if !self.state_arena.contains::<ButtonState>(node.id) {
                        self.state_arena.insert(node.id, ButtonState::default());
                    }
                }
                _ => {}
            }
        }
    }

    fn seed_select_state(&mut self, tree: &UiTree) {
        for node in tree.nodes() {
            let Some(key) = node.key.as_ref().map(|key| key.as_str().to_string()) else {
                continue;
            };
            let Some(crate::widgets::WidgetSpec::Select(spec)) = node.widget_spec.as_ref() else {
                continue;
            };

            if !self.selected_value_by_key.contains_key(&key) {
                if let Some(value) = spec.default_value.clone() {
                    self.selected_value_by_key.insert(key.clone(), value);
                } else if let Some(index) = spec.selected_index {
                    if let Some(option) = spec.options.get(index) {
                        self.selected_value_by_key
                            .insert(key.clone(), option.value.clone());
                    }
                } else if let Some(option) = spec.options.first() {
                    self.selected_value_by_key
                        .insert(key.clone(), option.value.clone());
                }
            }

            if let Some(value) = self.selected_value_by_key.get(&key) {
                if let Some(index) = spec
                    .options
                    .iter()
                    .position(|option| &option.value == value)
                {
                    self.selected_index_by_key.insert(key.clone(), index);
                }
            } else if let Some(index) = spec.selected_index {
                self.selected_index_by_key.entry(key).or_insert(index);
            }
        }
    }

    fn seed_disabled_select_options(&mut self, tree: &UiTree) {
        self.disabled_select_options_by_key.clear();
        for node in tree.nodes() {
            let Some(key) = node.key.as_ref().map(|key| key.as_str().to_string()) else {
                continue;
            };
            let Some(crate::widgets::WidgetSpec::Select(spec)) = node.widget_spec.as_ref() else {
                continue;
            };
            let disabled = spec
                .options
                .iter()
                .enumerate()
                .filter_map(|(index, option)| option.disabled.then_some(index))
                .collect::<HashSet<_>>();
            if !disabled.is_empty() {
                self.disabled_select_options_by_key.insert(key, disabled);
            }
        }
    }

    fn seed_collection_state(&mut self, tree: &UiTree) {
        for node in tree.nodes() {
            let Some(key) = node.key.as_ref().map(|key| key.as_str().to_string()) else {
                continue;
            };
            match node.widget_spec.as_ref() {
                Some(crate::widgets::WidgetSpec::Tabs(spec)) => {
                    if let Some(index) = spec.active_index {
                        self.active_index_by_key.entry(key).or_insert(index);
                    }
                }
                Some(crate::widgets::WidgetSpec::Table(spec)) => {
                    if let Some(index) = spec.selected_row {
                        self.table_selected_row_by_key.entry(key).or_insert(index);
                    }
                }
                Some(crate::widgets::WidgetSpec::List(spec)) => {
                    if let Some(index) = spec.selected_index {
                        self.list_selected_index_by_key.entry(key).or_insert(index);
                    }
                }
                _ => {}
            }
        }
    }
}

struct FrameBuilder<'a> {
    text_system: &'a mut TextSystem,
    viewport: Size,
    taffy_layout: Option<&'a crate::core::LayoutResult>,
    display_list: DisplayList,
    semantics: SemanticTree,
    hit_test: HitTestTree,
    snapshot: UiSnapshot,
    paint_order: u64,
    portal_tree: super::PortalTree,
    scroll_offsets_by_key: &'a HashMap<String, crate::core::Vec2>,
    scroll_bounds_by_key: HashMap<String, crate::core::Vec2>,
    scroll_rects_by_key: HashMap<String, Rect>,
    dismissed_overlay_keys: &'a HashSet<String>,
    opened_overlay_keys: &'a HashSet<String>,
    bool_state_by_key: &'a HashMap<String, bool>,
    bool_state_by_node: &'a HashMap<NodeId, bool>,
    selected_index_by_key: &'a HashMap<String, usize>,
    tree_expanded_by_key: &'a HashMap<String, HashMap<usize, bool>>,
    table_selected_row_by_key: &'a HashMap<String, usize>,
    list_selected_index_by_key: &'a HashMap<String, usize>,
    active_index_by_key: &'a HashMap<String, usize>,
    focused_node: Option<NodeId>,
    active_node: Option<NodeId>,
    hovered_node: Option<NodeId>,
    state_arena: &'a StateArena,
    theme: &'a Theme,
    open_context_menu_key: Option<&'a str>,
    context_menu_anchor: Option<Point>,
    open_select_key: Option<&'a str>,
    focusable_keys: Vec<String>,
    overlay_focusable_keys: Vec<String>,
    widget_kind_by_key: HashMap<String, WidgetKind>,
    click_action_by_key: HashMap<String, String>,
    widget_rect_by_key: HashMap<String, Rect>,
    open_overlay_keys: Vec<String>,
    open_overlay_rects: Vec<(String, Rect)>,
    has_open_modal: bool,
}

impl<'a> FrameBuilder<'a> {
    fn new(
        text_system: &'a mut TextSystem,
        viewport: Size,
        taffy_layout: Option<&'a crate::core::LayoutResult>,
        scroll_offsets_by_key: &'a HashMap<String, crate::core::Vec2>,
        dismissed_overlay_keys: &'a HashSet<String>,
        opened_overlay_keys: &'a HashSet<String>,
        bool_state_by_key: &'a HashMap<String, bool>,
        bool_state_by_node: &'a HashMap<NodeId, bool>,
        selected_index_by_key: &'a HashMap<String, usize>,
        tree_expanded_by_key: &'a HashMap<String, HashMap<usize, bool>>,
        table_selected_row_by_key: &'a HashMap<String, usize>,
        list_selected_index_by_key: &'a HashMap<String, usize>,
        active_index_by_key: &'a HashMap<String, usize>,
        focused_node: Option<NodeId>,
        active_node: Option<NodeId>,
        hovered_node: Option<NodeId>,
        state_arena: &'a StateArena,
        theme: &'a Theme,
        open_context_menu_key: Option<&'a str>,
        context_menu_anchor: Option<Point>,
        open_select_key: Option<&'a str>,
    ) -> Self {
        Self {
            text_system,
            viewport,
            taffy_layout,
            display_list: DisplayList::default(),
            semantics: SemanticTree::default(),
            hit_test: HitTestTree::default(),
            snapshot: UiSnapshot::default(),
            paint_order: 0,
            portal_tree: super::PortalTree::default(),
            scroll_offsets_by_key,
            scroll_bounds_by_key: HashMap::new(),
            scroll_rects_by_key: HashMap::new(),
            dismissed_overlay_keys,
            opened_overlay_keys,
            bool_state_by_key,
            bool_state_by_node,
            selected_index_by_key,
            tree_expanded_by_key,
            table_selected_row_by_key,
            list_selected_index_by_key,
            active_index_by_key,
            focused_node,
            active_node,
            hovered_node,
            state_arena,
            theme,
            open_context_menu_key,
            context_menu_anchor,
            open_select_key,
            focusable_keys: Vec::new(),
            overlay_focusable_keys: Vec::new(),
            widget_kind_by_key: HashMap::new(),
            click_action_by_key: HashMap::new(),
            widget_rect_by_key: HashMap::new(),
            open_overlay_keys: Vec::new(),
            open_overlay_rects: Vec::new(),
            has_open_modal: false,
        }
    }

    fn push_tree(&mut self, tree: &UiTree) {
        let root = tree.root_node();
        self.push_node(
            tree,
            root.id,
            None,
            None,
            None,
            crate::core::Vec2::default(),
            0,
            0,
        );
        self.paint_collected_overlays();
    }

    fn next_order(&mut self) -> u64 {
        let order = self.paint_order;
        self.paint_order += 1;
        order
    }

    fn push_node(
        &mut self,
        tree: &UiTree,
        node_id: NodeId,
        parent_id: Option<NodeId>,
        parent_rect: Option<Rect>,
        inherited_clip: Option<Rect>,
        parent_scroll_offset: crate::core::Vec2,
        sibling_index: usize,
        inherited_z: i32,
    ) {
        let node = tree.get(node_id).expect("runtime node exists in tree");
        let layout = self.resolve_layout(
            tree,
            node_id,
            parent_id,
            parent_rect,
            inherited_clip,
            parent_scroll_offset,
            sibling_index,
        );
        let rect = layout.rect;
        let clip_rect = layout.clip_rect;

        self.snapshot
            .tree_nodes
            .push(kind_name(&node.kind).to_string());
        let local_z = node.style.z_index.unwrap_or(0);
        let z_index = inherited_z + local_z;
        self.snapshot.styles.push(ResolvedStyleSnapshot {
            node: node.id,
            z_index,
        });
        self.snapshot.measure.push(MeasureSnapshot {
            node: node.id,
            key: node.key.as_ref().map(|key| key.as_str().to_string()),
            preferred_width: rect.size.width,
            preferred_height: rect.size.height,
            content_width: layout.content_size.width,
            content_height: layout.content_size.height,
        });
        self.snapshot.layout.push(LayoutBoxSnapshot {
            node: node.id,
            key: node.key.as_ref().map(|key| key.as_str().to_string()),
            x: rect.origin.x,
            y: rect.origin.y,
            width: rect.size.width,
            height: rect.size.height,
            content_width: layout.content_size.width,
            content_height: layout.content_size.height,
            clip_rect,
        });
        let order = self.next_order();
        let hit_rect = match clip_rect {
            Some(clip) => rect.intersect(clip),
            None => Some(rect),
        };
        if let Some(hit_rect) = hit_rect {
            let visible_rect = (hit_rect != rect).then_some(hit_rect);
            let hit_entry = HitTestEntry::new(node.id, rect, z_index, LayerKind::Document)
                .with_key(node.key.as_ref().map(|key| key.as_str().to_string()))
                .with_visible_rect(visible_rect)
                .with_order(order as usize);
            self.hit_test.push(hit_entry);
            if let Some(key) = node.key.as_ref() {
                if layout.content_size.height > rect.size.height {
                    let thumb_rect = self.theme.widgets.metrics.scrollbar.thumb_rect(
                        rect,
                        layout.content_size,
                        layout.scroll_offset,
                    );
                    self.hit_test.push(
                        HitTestEntry::new(node.id, thumb_rect, z_index + 3, LayerKind::Document)
                            .with_key(Some(format!("{}::__scrollbar_thumb", key.as_str())))
                            .with_order(order as usize + 1),
                    );
                }
            }
        }

        self.push_semantics(tree, node, rect);
        // Paint node background/content
        self.push_paint(
            tree,
            node,
            rect,
            z_index,
            layout.content_size,
            layout.scroll_offset,
        );
        // Collect overlay for deferred painting outside document clip stack
        self.collect_overlay(tree, node, rect);

        let pushes_new_clip = clip_rect.is_some() && clip_rect != inherited_clip;
        if let Some(clip) = clip_rect.filter(|_| pushes_new_clip) {
            self.display_list
                .push(PaintCommand::PushClip(ClipSpec::rect(clip)));
        }

        if should_layout_children(node) {
            let active_idx = if matches!(node.kind, ElementKind::Widget(WidgetKind::Tabs)) {
                let key = node.key.as_ref().map(|k| k.as_str());
                Some(
                    key.and_then(|k| self.active_index_by_key.get(k).copied())
                        .unwrap_or_else(|| {
                            if let Some(crate::widgets::WidgetSpec::Tabs(ref spec)) =
                                node.widget_spec
                            {
                                spec.active_index.unwrap_or(0)
                            } else {
                                0
                            }
                        }),
                )
            } else {
                None
            };

            for (index, child_id) in node.children.iter().copied().enumerate() {
                if let Some(active) = active_idx {
                    if index != active {
                        continue;
                    }
                }
                self.push_node(
                    tree,
                    child_id,
                    Some(node.id),
                    Some(rect),
                    clip_rect,
                    layout.scroll_offset,
                    index,
                    z_index,
                );
            }
        }

        if pushes_new_clip {
            self.display_list.push(PaintCommand::PopClip);
        }
    }

    fn resolve_layout(
        &mut self,
        tree: &UiTree,
        node_id: NodeId,
        _parent_id: Option<NodeId>,
        parent_rect: Option<Rect>,
        inherited_clip: Option<Rect>,
        parent_scroll_offset: crate::core::Vec2,
        _sibling_index: usize,
    ) -> ResolvedLayout {
        let layout = self
            .taffy_layout
            .as_ref()
            .and_then(|result| result.boxes.iter().find(|layout| layout.node == node_id))
            .unwrap_or_else(|| panic!("taffy layout result missing node {node_id:?}"));

        let mut rect = layout.local_rect;
        if let Some(parent_rect) = parent_rect {
            rect.origin.x += parent_rect.origin.x - parent_scroll_offset.x;
            rect.origin.y += parent_rect.origin.y - parent_scroll_offset.y;
        }
        let content_size = layout.content_size;
        let scroll_offset = scroll_offset_for_node(
            tree.get(node_id).expect("runtime node exists in tree"),
            rect,
            content_size,
            self.scroll_offsets_by_key,
        );
        if let Some(key) = tree.get(node_id).and_then(|node| node.key.as_ref()) {
            if clips_overflow_node(tree.get(node_id).expect("runtime node exists in tree")) {
                self.scroll_bounds_by_key.insert(
                    key.as_str().to_string(),
                    Vec2::new(
                        (content_size.width - rect.size.width).max(0.0),
                        (content_size.height - rect.size.height).max(0.0),
                    ),
                );
                self.scroll_rects_by_key
                    .insert(key.as_str().to_string(), rect);
            }
        }
        let node = tree.get(node_id).expect("runtime node exists in tree");
        let clip_rect = if clips_overflow_node(node) {
            inherited_clip
                .and_then(|clip| clip.intersect(rect))
                .or(Some(rect))
        } else {
            inherited_clip
        };
        ResolvedLayout {
            rect,
            clip_rect,
            scroll_offset,
            content_size,
        }
    }

    fn push_semantics(&mut self, tree: &UiTree, node: &UiNode, rect: Rect) {
        let role = role_for_node(node);
        let label = label_for_node(node, tree);
        let value = self.semantic_value_for_node(tree, node);
        let focusable = matches!(
            node.kind,
            ElementKind::Widget(
                WidgetKind::Button
                    | WidgetKind::Input
                    | WidgetKind::Checkbox
                    | WidgetKind::Radio
                    | WidgetKind::Select
                    | WidgetKind::Textarea
            )
        ) && !node_is_disabled(node);
        if focusable {
            if let Some(key) = node.key.as_ref() {
                self.focusable_keys.push(key.as_str().to_string());
            }
        }
        if let (Some(key), ElementKind::Widget(kind)) = (node.key.as_ref(), &node.kind) {
            self.widget_kind_by_key
                .insert(key.as_str().to_string(), *kind);
        }
        if let Some(key) = node.key.as_ref() {
            self.widget_rect_by_key
                .insert(key.as_str().to_string(), rect);
            if let Some(action) = node.handlers.on_click_action.as_ref() {
                self.click_action_by_key
                    .insert(key.as_str().to_string(), action.clone());
            }
        }
        self.semantics.push(SemanticNode {
            node: node.id,
            key: node.key.as_ref().map(|key| key.as_str().to_string()),
            role,
            label: label.clone(),
            description: None,
            value,
            states: SemanticStates {
                focused: Some(node.id) == self.focused_node,
                disabled: node_is_disabled(node),
                checked: matches!(
                    node.kind,
                    ElementKind::Widget(WidgetKind::Checkbox | WidgetKind::Radio)
                ) && self
                    .widget_bool_state(
                        node,
                        node.key.as_ref().map(|key| key.as_str()).unwrap_or(""),
                    )
                    .or(node.checked)
                    .unwrap_or(false),
                expanded: None,
            },
            actions: Vec::new(),
            focusable,
            focus_order: None,
            keyboard_navigation: crate::core::KeyboardNav::None,
            bounds: rect,
        });
        self.snapshot.semantics.push(SemanticSnapshot {
            node: node.id,
            role: format!("{role:?}"),
            label,
        });
    }

    fn push_paint(
        &mut self,
        tree: &UiTree,
        node: &UiNode,
        rect: Rect,
        z_index: i32,
        content_size: Size,
        scroll_offset: Vec2,
    ) {
        let mut state = self.visual_state_for_node(tree, node);
        state.content_size = content_size;
        state.scroll_offset = scroll_offset;
        for painted in paint::paint_node_themed(
            node,
            rect,
            z_index,
            &state,
            self.text_system,
            Some(self.theme),
        ) {
            self.display_list.push(painted.command);
            self.snapshot.display_list.push(painted.snapshot);
        }
    }

    fn visual_state_for_node(&self, tree: &UiTree, node: &UiNode) -> paint::VisualState {
        let key = node.key.as_ref().map(|key| key.as_str());
        let checked = if let Some(state) = self.state_arena.get::<CheckboxState>(node.id) {
            state.checked
        } else {
            key.and_then(|key| self.widget_bool_state(node, key))
                .or(node.checked)
                .unwrap_or(false)
        };
        let (text, cursor, preedit) =
            if let Some(state) = self.state_arena.get::<InputState>(node.id) {
                (
                    Some(state.text.clone()),
                    Some(state.cursor),
                    state.preedit.clone(),
                )
            } else {
                (key_default_value(tree, node), None, None)
            };
        let label =
            if let Some(crate::widgets::WidgetSpec::Select(spec)) = node.widget_spec.as_ref() {
                key.and_then(|key| self.selected_index_by_key.get(key).copied())
                    .and_then(|index| spec.options.get(index))
                    .map(|option| option.label.clone())
                    .or_else(|| spec.placeholder.clone())
            } else {
                label_for_node(node, tree).or_else(|| node.semantic.label.clone())
            };

        paint::VisualState {
            checked,
            text,
            label,
            focused: Some(node.id) == self.focused_node,
            active: Some(node.id) == self.active_node,
            hovered: Some(node.id) == self.hovered_node,
            disabled: node_is_disabled(node),
            open: node.open,
            primary: node
                .variant
                .as_ref()
                .is_some_and(|variant| variant.as_str() == "primary"),
            cursor,
            preedit,
            content_size: Size::default(),
            scroll_offset: Vec2::default(),
            tree_expanded: key
                .and_then(|key| self.tree_expanded_by_key.get(key))
                .cloned()
                .unwrap_or_default(),
            table_selected_row: key.and_then(|key| {
                self.table_selected_row_by_key
                    .get(key)
                    .copied()
                    .or_else(|| match node.widget_spec.as_ref()? {
                        crate::widgets::WidgetSpec::Table(spec) => spec.selected_row,
                        _ => None,
                    })
            }),
            list_selected_index: key.and_then(|key| {
                self.list_selected_index_by_key
                    .get(key)
                    .copied()
                    .or_else(|| match node.widget_spec.as_ref()? {
                        crate::widgets::WidgetSpec::List(spec) => spec.selected_index,
                        _ => None,
                    })
            }),
            tabs_active_index: key.and_then(|key| {
                self.active_index_by_key.get(key).copied().or_else(|| {
                    match node.widget_spec.as_ref()? {
                        crate::widgets::WidgetSpec::Tabs(spec) => spec.active_index,
                        _ => None,
                    }
                })
            }),
        }
    }

    fn widget_bool_state(&self, node: &UiNode, key: &str) -> Option<bool> {
        self.bool_state_by_node
            .get(&node.id)
            .copied()
            .or_else(|| self.bool_state_by_key.get(key).copied())
    }

    fn semantic_value_for_node(&self, tree: &UiTree, node: &UiNode) -> Option<SemanticValue> {
        match node.kind {
            ElementKind::Widget(WidgetKind::Input | WidgetKind::Textarea) => {
                if let Some(state) = self.state_arena.get::<InputState>(node.id) {
                    if !state.text.is_empty() {
                        return Some(SemanticValue::Text(state.text.clone()));
                    }
                }
                key_default_value(tree, node)
                    .filter(|value| !value.is_empty())
                    .map(SemanticValue::Text)
            }
            _ => None,
        }
    }

    fn collect_overlay(&mut self, tree: &UiTree, node: &UiNode, rect: Rect) {
        let prev_count = self.portal_tree.roots.len();
        self.portal_tree.collect_from_overlay_pass(
            node,
            rect,
            self.viewport,
            self.dismissed_overlay_keys,
            self.opened_overlay_keys,
            tree,
            self.open_context_menu_key.as_deref(),
            self.context_menu_anchor,
        );
        self.collect_select_overlay(node, rect);
        // Track open overlay keys/rects and snapshot only new roots
        for i in prev_count..self.portal_tree.roots.len() {
            let root = &mut self.portal_tree.roots[i];
            if root.key.as_deref() == Some("__modal_backdrop") {
                if root.modal {
                    self.has_open_modal = true;
                }
                continue;
            }

            let max_size = overlay_layout_max_size(root, self.viewport, self.theme);
            let root_element = overlay_root_element_for_layout(root, self.theme, max_size);
            let subtree = UiTree::from_portal_element(root_element, root.owner);
            let mut backend = TaffyLayoutBackend::new();
            let layout_res = backend.build_from_tree_with_cx(
                &subtree,
                self.text_system,
                max_size,
                self.theme,
                &LayoutCx::empty(),
            );

            if !layout_res.diagnostics.layout_errors.is_empty() {
                self.snapshot.diagnostics.layout_errors.extend(
                    layout_res
                        .diagnostics
                        .layout_errors
                        .iter()
                        .map(|error| format!("overlay {:?}: {error}", root.key)),
                );
            }

            if let Some(root_box) = layout_res.box_for_node(root.owner) {
                let computed_size = root_box.local_rect.size;
                let panel_rect = portal_panel_rect_for_layout(root, computed_size, self.viewport);
                let content_rect = Rect::new(panel_rect.origin, computed_size);
                root.rect = panel_rect;
                append_overlay_layout_snapshots(
                    &mut self.snapshot,
                    &subtree,
                    &layout_res,
                    subtree.root(),
                    panel_rect.origin,
                );
                root.computed = Some(super::portal_pass::PortalRootLayout {
                    panel_rect,
                    content_rect,
                    layout: layout_res,
                });
                root.tree = Some(subtree);
            }

            if root.modal {
                self.has_open_modal = true;
            }
            if let Some(key) = &root.key {
                if key != "__modal_backdrop" {
                    self.open_overlay_keys.push(key.clone());
                    self.open_overlay_rects.push((key.clone(), root.rect));
                }
            }
            self.snapshot.overlays.push(OverlaySnapshot {
                key: root.key.clone(),
                layer: root.layer,
                rect: root.rect,
                modal: root.modal,
            });
        }
    }

    fn collect_select_overlay(&mut self, node: &UiNode, rect: Rect) {
        let Some(key) = node.key.as_ref().map(|key| key.as_str()) else {
            return;
        };
        if self.open_select_key != Some(key) {
            return;
        }
        let Some(crate::widgets::WidgetSpec::Select(spec)) = node.widget_spec.as_ref() else {
            return;
        };
        if spec.options.is_empty() {
            return;
        }

        let overlay_key = format!("{key}::__options");
        let mut style = crate::core::Style::default();
        style.min_width = Some(crate::core::Length::Px(rect.size.width));
        style.padding = Some(crate::core::Edge::all(crate::core::Length::Px(4.0)));
        style.gap = Some(crate::core::Length::Px(2.0));

        let children = spec
            .options
            .iter()
            .enumerate()
            .map(|(index, option)| {
                let option_key = select_option_key(key, index);
                let mut element = crate::widgets::button(option.label.clone())
                    .key(option_key)
                    .height(self.theme.widgets.metrics.menu.item_height);
                if option.disabled {
                    if let Some(crate::widgets::WidgetSpec::Button(spec)) =
                        element.widget_spec.as_mut()
                    {
                        spec.disabled = true;
                    }
                }
                super::portal_pass::PortalItem {
                    node: stable_portal_child_id(node.id, None, index),
                    element,
                }
            })
            .collect();

        self.portal_tree.roots.push(super::portal_pass::PortalRoot {
            owner: node.id,
            key: Some(overlay_key),
            layer: LayerKind::Popover,
            rect: Rect::new(Point::new(0.0, 0.0), Size::new(0.0, 0.0)),
            modal: false,
            children: super::portal_pass::PortalChildren::Items(children),
            computed: None,
            style,
            kind: ElementKind::Widget(WidgetKind::Popover),
            anchor_rect: rect,
            context_menu_anchor: None,
            tree: None,
        });
    }

    fn paint_collected_overlays(&mut self) {
        self.portal_tree.paint_all(
            self.viewport,
            self.text_system,
            &mut self.display_list,
            &mut self.hit_test,
            &mut self.semantics,
            &mut self.focusable_keys,
            &mut self.overlay_focusable_keys,
            &mut self.widget_kind_by_key,
            self.theme,
        );
    }
}

fn should_layout_children(node: &UiNode) -> bool {
    !matches!(
        node.kind,
        ElementKind::Widget(
            WidgetKind::Button
                | WidgetKind::Input
                | WidgetKind::Checkbox
                | WidgetKind::Radio
                | WidgetKind::Select
                | WidgetKind::Textarea
                | WidgetKind::Modal
                | WidgetKind::Popover
                | WidgetKind::Tooltip
        )
    )
}

fn overlay_root_element_for_layout(
    root: &super::portal_pass::PortalRoot,
    theme: &Theme,
    max_size: Size,
) -> crate::core::Element {
    let items = match &root.children {
        super::portal_pass::PortalChildren::Items(items) => items,
    };

    let mut root_style = root.style.clone();
    let metrics = &theme.widgets.metrics.overlay;

    let min_width = root_style
        .min_width
        .and_then(|width| width.resolve(metrics.min_width))
        .unwrap_or(metrics.min_width);
    root_style.min_width = Some(crate::core::Length::Px(min_width));

    let min_height = root_style
        .min_height
        .and_then(|height| height.resolve(metrics.min_height))
        .unwrap_or(metrics.min_height);
    root_style.min_height = Some(crate::core::Length::Px(min_height));

    let max_width = root_style
        .max_width
        .and_then(|width| width.resolve(max_size.width))
        .unwrap_or(max_size.width);
    root_style.max_width = Some(crate::core::Length::Px(max_width));

    let max_height = root_style
        .max_height
        .and_then(|height| height.resolve(max_size.height))
        .unwrap_or(max_size.height);
    root_style.max_height = Some(crate::core::Length::Px(max_height));

    if root_style.display.is_none() {
        root_style.display = Some(crate::core::Display::Flex);
    }
    if root_style.flex_direction.is_none() {
        root_style.flex_direction = Some(crate::core::FlexDirection::Column);
    }
    if root_style.padding.is_none() {
        root_style.padding = Some(crate::core::Edge::all(crate::core::Length::Px(
            metrics.padding,
        )));
    }
    if root_style.gap.is_none() {
        root_style.gap = Some(crate::core::Length::Px(metrics.gap));
    }

    crate::core::Element {
        key: root.key.clone().map(crate::core::ElementKey::new),
        kind: root.kind.clone(),
        widget_spec: None,
        children: items.iter().map(|item| item.element.clone()).collect(),
        style: root_style,
        variant: None,
        checked: None,
        default_checked: None,
        semantic: crate::core::Semantic::default(),
        event_handlers: crate::core::EventHandlers::default(),
        overlay: None,
        open: true,
    }
}

fn overlay_layout_max_size(
    root: &super::portal_pass::PortalRoot,
    viewport: Size,
    theme: &Theme,
) -> Size {
    if root.modal {
        return viewport;
    }

    let below = (viewport.height - root.anchor_rect.max_y()).max(0.0);
    let above = root.anchor_rect.origin.y.max(0.0);
    Size::new(
        viewport.width.max(theme.widgets.metrics.overlay.min_width),
        below
            .max(above)
            .max(theme.widgets.metrics.overlay.min_height),
    )
}

fn portal_panel_rect_for_layout(
    root: &super::portal_pass::PortalRoot,
    layout_size: Size,
    viewport: Size,
) -> Rect {
    if root.modal {
        Rect::new(
            Point::new(
                (viewport.width - layout_size.width).max(0.0) * 0.5,
                (viewport.height - layout_size.height).max(0.0) * 0.5,
            ),
            layout_size,
        )
    } else {
        super::overlay_pass::place_anchored_overlay(
            layout_size,
            root.anchor_rect,
            root.context_menu_anchor,
            viewport,
        )
    }
}

fn append_overlay_layout_snapshots(
    snapshot: &mut UiSnapshot,
    tree: &UiTree,
    layout: &crate::core::LayoutResult,
    node_id: NodeId,
    parent_origin: Point,
) {
    let Some(node) = tree.get(node_id) else {
        return;
    };
    let Some(layout_box) = layout.box_for_node(node_id) else {
        return;
    };
    let world_origin = Point::new(
        parent_origin.x + layout_box.local_rect.origin.x,
        parent_origin.y + layout_box.local_rect.origin.y,
    );

    snapshot.layout.push(LayoutBoxSnapshot {
        node: layout_box.node,
        key: layout_box.key.clone(),
        x: world_origin.x,
        y: world_origin.y,
        width: layout_box.local_rect.size.width,
        height: layout_box.local_rect.size.height,
        content_width: layout_box.content_size.width,
        content_height: layout_box.content_size.height,
        clip_rect: layout_box.clip_rect.map(|clip| {
            Rect::new(
                Point::new(
                    world_origin.x + clip.origin.x,
                    world_origin.y + clip.origin.y,
                ),
                clip.size,
            )
        }),
    });

    for child in &node.children {
        append_overlay_layout_snapshots(snapshot, tree, layout, *child, world_origin);
    }
}

fn role_for_node(node: &UiNode) -> Role {
    match &node.kind {
        ElementKind::Text(_) => Role::Text,
        ElementKind::Widget(WidgetKind::Button) => Role::Button,
        ElementKind::Widget(WidgetKind::Input | WidgetKind::Textarea) => Role::TextInput,
        ElementKind::Widget(WidgetKind::Checkbox) => Role::Checkbox,
        ElementKind::Widget(WidgetKind::Radio) => Role::Radio,
        ElementKind::Widget(WidgetKind::List) => Role::List,
        ElementKind::Widget(WidgetKind::Table) => Role::Table,
        ElementKind::Widget(WidgetKind::Modal) => Role::Dialog,
        ElementKind::Widget(WidgetKind::Tooltip) => Role::Tooltip,
        ElementKind::Widget(WidgetKind::ScrollArea) => Role::ScrollArea,
        ElementKind::Widget(WidgetKind::Image) => Role::Image,
        ElementKind::Widget(WidgetKind::Switch) => Role::Switch,
        ElementKind::Widget(WidgetKind::Slider) => Role::Slider,
        ElementKind::Widget(WidgetKind::ProgressBar) => Role::ProgressBar,
        ElementKind::Widget(WidgetKind::Spinner) => Role::Spinner,
        ElementKind::Widget(WidgetKind::Badge) => Role::Badge,
        ElementKind::Widget(WidgetKind::Avatar) => Role::Avatar,
        ElementKind::Widget(WidgetKind::Link) => Role::Link,
        ElementKind::Widget(WidgetKind::Alert) => Role::Alert,
        ElementKind::Widget(WidgetKind::Card) => Role::Card,
        _ => Role::Group,
    }
}

fn label_for_node(node: &UiNode, tree: &UiTree) -> Option<String> {
    // Prioritize widget_spec labels (clean separation from semantic.label)
    if let Some(ref spec) = node.widget_spec {
        match spec {
            crate::widgets::WidgetSpec::Button(bs) if bs.label.is_some() => {
                return bs.label.clone();
            }
            crate::widgets::WidgetSpec::Checkbox(cs) if cs.label.is_some() => {
                return cs.label.clone();
            }
            crate::widgets::WidgetSpec::Radio(rs) if rs.label.is_some() => return rs.label.clone(),
            crate::widgets::WidgetSpec::Input(is) if is.aria_label.is_some() => {
                return is.aria_label.clone();
            }
            crate::widgets::WidgetSpec::Tooltip(ts) if ts.text.is_some() => return ts.text.clone(),
            crate::widgets::WidgetSpec::Icon(is) => return Some(is.name.clone()),
            crate::widgets::WidgetSpec::Modal(ms) if ms.title.is_some() => return ms.title.clone(),
            _ => {}
        }
    }

    match &node.kind {
        ElementKind::Text(spec) => Some(spec.text.clone()),
        ElementKind::Widget(WidgetKind::Button) => node
            .children
            .iter()
            .filter_map(|child| tree.get(*child))
            .find_map(|child| label_for_node(child, tree)),
        _ => node.semantic.label.clone(),
    }
}

fn kind_name(kind: &ElementKind) -> &'static str {
    match kind {
        ElementKind::Primitive(_) => "Primitive",
        ElementKind::Widget(_) => "Widget",
        ElementKind::Component(_) => "Component",
        ElementKind::Canvas(_) => "Canvas",
        ElementKind::Text(_) => "Text",
    }
}

fn key_default_value(_tree: &UiTree, node: &UiNode) -> Option<String> {
    if let Some(ref spec) = node.widget_spec {
        use crate::WidgetSpec;
        match spec {
            WidgetSpec::Input(s) => s.default_value.clone(),
            WidgetSpec::Textarea(s) => s.default_value.clone(),
            _ => None,
        }
    } else {
        None
    }
}

fn node_is_disabled(node: &UiNode) -> bool {
    match node.widget_spec.as_ref() {
        Some(crate::widgets::WidgetSpec::Button(spec)) => spec.disabled,
        Some(crate::widgets::WidgetSpec::Input(spec)) => spec.disabled,
        Some(crate::widgets::WidgetSpec::Textarea(spec)) => spec.disabled,
        Some(crate::widgets::WidgetSpec::Checkbox(spec)) => spec.disabled,
        Some(crate::widgets::WidgetSpec::Radio(spec)) => spec.disabled,
        Some(crate::widgets::WidgetSpec::Select(spec)) => spec.disabled,
        _ => false,
    }
}

fn select_option_key(select_key: &str, index: usize) -> String {
    format!("{select_key}::__option::{index}")
}

fn parse_select_option_key(key: &str) -> Option<(&str, usize)> {
    let (select_key, index) = key.split_once("::__option::")?;
    let index = index.parse().ok()?;
    Some((select_key, index))
}

fn apply_widget_part_layout_styles(root: &mut crate::Element, theme: &Theme) {
    if let Some(crate::widgets::WidgetSpec::Select(spec)) = root.widget_spec.as_ref() {
        let mut trigger = theme.widgets.select.base.trigger.clone();
        if let Some(variant) = root.variant.as_ref() {
            if let Some(variant_styles) = theme.widgets.select.variants.get(variant) {
                trigger = merge_optional_style(trigger, variant_styles.trigger.clone());
            }
        }
        trigger = merge_optional_style(trigger, spec.styles.trigger.clone());
        if let Some(trigger) = trigger {
            root.style = root.style.clone().merge_over(trigger);
        }
    }
    for child in &mut root.children {
        apply_widget_part_layout_styles(child, theme);
    }
}

fn merge_optional_style(
    base: Option<crate::Style>,
    next: Option<crate::Style>,
) -> Option<crate::Style> {
    match (base, next) {
        (Some(base), Some(next)) => Some(base.merge_over(next)),
        (None, Some(next)) => Some(next),
        (Some(base), None) => Some(base),
        (None, None) => None,
    }
}

fn estimate_render_item_count(display_list: &DisplayList) -> usize {
    display_list
        .commands()
        .iter()
        .map(|command| match command {
            PaintCommand::DrawText(cmd) => cmd
                .text
                .chars()
                .filter(|ch| !ch.is_whitespace())
                .count()
                .max(1),
            PaintCommand::DrawBorder(_) => 4,
            PaintCommand::DrawRect(_)
            | PaintCommand::DrawImage(_)
            | PaintCommand::DrawSvg(_)
            | PaintCommand::DrawPath(_)
            | PaintCommand::DrawShadow(_) => 1,
            PaintCommand::PushLayer(_)
            | PaintCommand::PopLayer
            | PaintCommand::PushClip(_)
            | PaintCommand::PopClip => 1,
        })
        .sum()
}
