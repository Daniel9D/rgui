use std::collections::HashMap;

use crate::core::{HitTestEntry, NodeId, UiEvent, WidgetKind};
use crate::runtime::{CommandQueue, UiCommand, UiNode, UiTree};

pub struct EventPath {
    nodes: Vec<NodeId>,
    target_index: usize,
}

impl EventPath {
    pub fn build(hit: &HitTestEntry, tree: &UiTree) -> Self {
        let mut nodes = Vec::new();
        let target = hit.node;

        // Walk from target to root
        let mut current = Some(target);
        while let Some(id) = current {
            nodes.push(id);
            current = tree.get(id).and_then(|n| n.parent);
        }
        nodes.reverse(); // Now root first, target last

        let target_index = nodes
            .iter()
            .position(|&id| id == target)
            .unwrap_or(nodes.len() - 1);

        Self {
            nodes,
            target_index,
        }
    }

    pub fn capture_phase(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.nodes[..self.target_index].iter().copied()
    }

    pub fn target(&self) -> NodeId {
        self.nodes[self.target_index]
    }

    pub fn target_phase(&self) -> NodeId {
        self.target()
    }

    pub fn bubble_phase(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.nodes[..self.target_index].iter().rev().copied()
    }

    pub fn capture_path(&self) -> &[NodeId] {
        &self.nodes[..self.target_index]
    }

    pub fn bubble_path(&self) -> Vec<NodeId> {
        self.bubble_phase().collect()
    }
}

pub struct EventDispatchContext<'a> {
    pub tree: &'a UiTree,
    pub widget_kinds: &'a HashMap<String, WidgetKind>,
    pub focused_key: Option<&'a str>,
    pub commands: CommandQueue,
    pub result: crate::core::EventResult,
    pub hit_node: NodeId,
    pub hit_key: Option<String>,
}

impl<'a> EventDispatchContext<'a> {
    pub fn new(tree: &'a UiTree, widget_kinds: &'a HashMap<String, WidgetKind>) -> Self {
        Self {
            tree,
            widget_kinds,
            focused_key: None,
            commands: CommandQueue::default(),
            result: crate::core::EventResult::default(),
            hit_node: NodeId::from_raw(0),
            hit_key: None,
        }
    }

    pub fn with_focus(mut self, focused_key: Option<&'a str>) -> Self {
        self.focused_key = focused_key;
        self
    }
}

pub fn dispatch_event(
    event: &UiEvent,
    hit: &HitTestEntry,
    ctx: &mut EventDispatchContext<'_>,
) -> Vec<UiCommand> {
    ctx.hit_node = hit.node;
    ctx.hit_key = hit.key.clone();
    ctx.result = crate::core::EventResult::default();

    let path = EventPath::build(hit, ctx.tree);

    // Capture phase
    for node_id in path.capture_phase() {
        if let Some(node) = ctx.tree.get(node_id) {
            handle_event_on_node(event, node, crate::core::EventPhase::Capture, ctx);
            if ctx.result.stop_propagation {
                break;
            }
        }
    }

    // Target phase
    if !ctx.result.stop_propagation {
        if let Some(node) = ctx.tree.get(path.target()) {
            handle_event_on_node(event, node, crate::core::EventPhase::Target, ctx);
        }
    }

    // Bubble phase
    if !ctx.result.stop_propagation {
        for node_id in path.bubble_phase() {
            if let Some(node) = ctx.tree.get(node_id) {
                handle_event_on_node(event, node, crate::core::EventPhase::Bubble, ctx);
                if ctx.result.stop_propagation {
                    break;
                }
            }
        }
    }

    // Default actions
    if !ctx.result.prevent_default {
        default_actions(event, ctx);
    }

    ctx.commands.drain()
}

fn handle_event_on_node(
    event: &UiEvent,
    node: &UiNode,
    phase: crate::core::EventPhase,
    ctx: &mut EventDispatchContext<'_>,
) {
    let widget_kind = node
        .key
        .as_ref()
        .and_then(|key| ctx.widget_kinds.get(key.as_str()))
        .copied();

    match event {
        UiEvent::PointerDown(_) => {
            ctx.result.handled = true;
        }
        UiEvent::KeyDown(key_event) => {
            if let Some(kind) = widget_kind {
                if phase == crate::core::EventPhase::Bubble {
                    match kind {
                        WidgetKind::Button if key_event.key == "Enter" || key_event.key == " " => {
                            ctx.result.handled = true;
                        }
                        WidgetKind::Checkbox if key_event.key == " " => {
                            ctx.result.handled = true;
                        }
                        _ => {}
                    }
                }
            }
            if key_event.key == "Escape" {
                ctx.result.handled = true;
            }
        }
        _ => {}
    }
}

fn default_actions(event: &UiEvent, ctx: &mut EventDispatchContext<'_>) {
    match event {
        UiEvent::PointerUp(_pointer) => {
            let hit_key = ctx.hit_key.clone();

            if let Some(key) = hit_key {
                let widget_kind = ctx.widget_kinds.get(&key).copied();

                match widget_kind {
                    Some(WidgetKind::Button) => {
                        ctx.commands.push(UiCommand::Click {
                            key: Some(key.clone()),
                            action: None,
                        });
                    }
                    Some(WidgetKind::Checkbox) => {
                        ctx.commands.push(UiCommand::SetBool {
                            key: key.clone(),
                            value: true, // toggle handled by runtime state layer
                        });
                    }
                    Some(WidgetKind::Input | WidgetKind::Textarea) => {
                        ctx.commands.push(UiCommand::Focus { key: key.clone() });
                    }
                    _ => {}
                }
            }
        }
        UiEvent::KeyDown(key_event) => {
            if key_event.key == "Escape" {
                ctx.commands
                    .push(UiCommand::CloseOverlay { key: String::new() });
            }
        }
        _ => {}
    }
}

pub struct FocusScope {
    pub id: FocusScopeId,
    pub parent: Option<FocusScopeId>,
    pub entries: Vec<FocusEntry>,
    pub current: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FocusScopeId(u64);

impl FocusScopeId {
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }
}

#[derive(Clone, Debug)]
pub struct FocusEntry {
    pub node: NodeId,
    pub key: Option<String>,
    pub tabindex: TabIndex,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabIndex {
    Auto,
    Explicit(i32),
    None,
}

impl FocusScope {
    pub fn new(id: FocusScopeId) -> Self {
        Self {
            id,
            parent: None,
            entries: Vec::new(),
            current: None,
        }
    }

    pub fn push_entry(&mut self, node: NodeId, key: Option<String>, tabindex: TabIndex) {
        if !matches!(tabindex, TabIndex::None) {
            self.entries.push(FocusEntry {
                node,
                key,
                tabindex,
            });
        }
    }

    pub fn advance(&mut self) -> Option<(NodeId, Option<String>)> {
        if self.entries.is_empty() {
            return None;
        }
        let next = match self.current {
            Some(i) => (i + 1) % self.entries.len(),
            None => 0,
        };
        self.current = Some(next);
        let entry = &self.entries[next];
        Some((entry.node, entry.key.clone()))
    }

    pub fn advance_prev(&mut self) -> Option<(NodeId, Option<String>)> {
        if self.entries.is_empty() {
            return None;
        }
        let next = match self.current {
            Some(i) if i > 0 => i - 1,
            _ => self.entries.len() - 1,
        };
        self.current = Some(next);
        let entry = &self.entries[next];
        Some((entry.node, entry.key.clone()))
    }

    pub fn focused_node(&self) -> Option<NodeId> {
        self.current
            .and_then(|i| self.entries.get(i))
            .map(|e| e.node)
    }

    pub fn focused_key(&self) -> Option<&str> {
        self.current
            .and_then(|i| self.entries.get(i))
            .and_then(|e| e.key.as_deref())
    }
}

pub struct FocusSystem {
    scopes: HashMap<FocusScopeId, FocusScope>,
    next_id: u64,
    active_scope: Option<FocusScopeId>,
    document_scope: Option<FocusScopeId>,
    overlay_scope: Option<FocusScopeId>,
}

impl Default for FocusSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl FocusSystem {
    pub fn new() -> Self {
        Self {
            scopes: HashMap::new(),
            next_id: 0,
            active_scope: None,
            document_scope: None,
            overlay_scope: None,
        }
    }

    pub fn create_scope(&mut self) -> FocusScopeId {
        self.next_id += 1;
        let id = FocusScopeId::from_raw(self.next_id);
        self.scopes.insert(id, FocusScope::new(id));
        if self.document_scope.is_none() {
            self.document_scope = Some(id);
        }
        id
    }

    pub fn create_document_scope(&mut self) -> FocusScopeId {
        let id = self.create_scope();
        self.document_scope = Some(id);
        id
    }

    pub fn scope_mut(&mut self, id: FocusScopeId) -> Option<&mut FocusScope> {
        self.scopes.get_mut(&id)
    }

    pub fn set_active(&mut self, scope: FocusScopeId) {
        self.active_scope = Some(scope);
    }

    pub fn activate_document_scope(&mut self) {
        if let Some(scope) = self.document_scope {
            self.set_active(scope);
        }
    }

    pub fn replace_overlay_scope(&mut self) -> FocusScopeId {
        if let Some(existing) = self.overlay_scope.take() {
            self.scopes.remove(&existing);
        }
        let id = self.create_scope();
        self.overlay_scope = Some(id);
        id
    }

    pub fn active_scope(&self) -> Option<FocusScopeId> {
        self.active_scope
    }

    pub fn tab_forward(&mut self) -> Option<(NodeId, Option<String>)> {
        self.active_scope
            .and_then(|id| self.scopes.get_mut(&id))
            .and_then(|scope| scope.advance())
    }

    pub fn tab_backward(&mut self) -> Option<(NodeId, Option<String>)> {
        self.active_scope
            .and_then(|id| self.scopes.get_mut(&id))
            .and_then(|scope| scope.advance_prev())
    }
}
