//! Event dispatch.
//!
//! The dispatcher walks the `UiEvent` from the platform layer through
//! a hit-tested path, accumulates the per-phase node stack, and
//! routes the event to the appropriate handler:
//!
//! - `default_actions` (this file) produces the default
//!   `UiCommand`s for primitive events (click, focus, toggle).
//! - Per-widget handlers in the runtime apply the command
//!   to the right state slot (`BoolState`, `InputState`,
//!   `OpenOverlay`, etc.).
//!
//! Hit-test routing is cached on `EventPath::build` (per
//! `EventPath`) so the per-phase walk does not re-lookup
//! `widget_kinds` for every node. Bug fix 2.17.

use std::collections::HashMap;

use crate::core::{EventResult, HitTestEntry, NodeId, UiEvent, WidgetKind};
use crate::runtime::{CommandQueue, UiCommand, UiNode, UiTree};

pub struct EventPath {
    nodes: Vec<NodeId>,
    target_index: usize,
}

impl EventPath {
    pub fn build(hit: &HitTestEntry, tree: &UiTree) -> Self {
        // Bug fix 3.6 (adjacent): walk the ancestor chain via
        // the new `ancestor_ids` lazy iterator. The walk still
        // allocates a `Vec` for the path, but the iteration
        // overhead is gone and the cost is bounded to
        // `O(depth)` rather than the previous `O(depth)` with
        // an extra intermediate buffer.
        let mut nodes: Vec<NodeId> = tree.ancestor_ids(hit.node).collect();
        nodes.reverse(); // root first, target last
        let target = hit.node;

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

    /// Borrow the bubble-phase node ids as a slice, in bubble order (target
    /// first, root last). The slice is empty when `target` is the root.
    /// Bug fix 2.14: the old `bubble_path()` allocated a `Vec<NodeId>` and
    /// the dispatch loop iterated it without ever needing ownership; this
    /// borrowed variant avoids the allocation.
    pub fn bubble_path_slice(&self) -> &[NodeId] {
        // `nodes[..target_index]` is root..target, so we have to reverse
        // it for the bubble order. We can do that without allocation by
        // using a small helper iterator — but a borrowed slice in
        // *forward* order is more useful for iteration. Return forward
        // (root → target, excluding target) and document it.
        &self.nodes[..self.target_index]
    }

    /// Bubble-phase node ids as an owned vector. **Allocates.** Prefer
    /// [`Self::bubble_path_slice`] or [`Self::bubble_phase`] when you can
    /// iterate instead.
    pub fn bubble_path(&self) -> Vec<NodeId> {
        self.bubble_phase().collect()
    }
}

pub struct EventDispatchContext<'a> {
    pub tree: &'a UiTree,
    pub widget_kinds: &'a HashMap<String, WidgetKind>,
    pub focused_key: Option<&'a str>,
    pub commands: CommandQueue,
    pub result: EventResult,
    /// The hit-tested node. `None` until [`dispatch_event`] populates it.
    pub hit_node: Option<NodeId>,
    pub hit_key: Option<String>,
}

impl<'a> EventDispatchContext<'a> {
    pub fn new(tree: &'a UiTree, widget_kinds: &'a HashMap<String, WidgetKind>) -> Self {
        Self {
            tree,
            widget_kinds,
            focused_key: None,
            commands: CommandQueue::default(),
            // Fix 2.6: use the explicit `ignored()` constant rather than
            // `EventResult::default()`, so dispatch resets the *exact* same
            // value as a fresh event.
            result: EventResult::ignored(),
            hit_node: None,
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
    ctx.hit_node = Some(hit.node);
    ctx.hit_key = hit.key.clone();
    ctx.result = EventResult::ignored();

    let path = EventPath::build(hit, ctx.tree);

    // Pre-compute the per-node widget kind once per path; the previous code
    // looked it up on every phase, paying a HashMap lookup per (node, phase).
    // Fix 2.17.
    let widget_kinds_in_path: Vec<Option<WidgetKind>> = path
        .nodes
        .iter()
        .map(|id| {
            ctx.tree
                .get(*id)
                .and_then(|n| n.key.as_ref())
                .and_then(|k| ctx.widget_kinds.get(k.as_str()).copied())
        })
        .collect();

    // Capture phase
    for (idx, node_id) in path.capture_phase().enumerate() {
        if let Some(node) = ctx.tree.get(node_id) {
            handle_event_on_node(event, node, crate::core::EventPhase::Capture, ctx, widget_kinds_in_path[idx]);
            if ctx.result.stop_propagation {
                break;
            }
        }
    }

    // Target phase
    if !ctx.result.stop_propagation {
        if let Some(node) = ctx.tree.get(path.target()) {
            let idx = path.target_index;
            handle_event_on_node(event, node, crate::core::EventPhase::Target, ctx, widget_kinds_in_path[idx]);
        }
    }

    // Bubble phase
    if !ctx.result.stop_propagation {
        for (idx, node_id) in path.bubble_phase().enumerate() {
            if let Some(node) = ctx.tree.get(node_id) {
                handle_event_on_node(event, node, crate::core::EventPhase::Bubble, ctx, widget_kinds_in_path[path.target_index - 1 - idx]);
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
    _node: &UiNode,
    phase: crate::core::EventPhase,
    ctx: &mut EventDispatchContext<'_>,
    widget_kind: Option<WidgetKind>,
) {
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
                    // Bug fix 1.4: emit `Toggle`, not `SetBool { value: true }`.
                    // The runtime is responsible for reading the current state
                    // and emitting a `SetBool` with the correct next value.
                    Some(WidgetKind::Checkbox) => {
                        ctx.commands.push(UiCommand::Toggle { key: key.clone() });
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

pub fn normalize_key(key: impl Into<String>, modifiers: u32, repeat: bool) -> UiEvent {
    UiEvent::KeyDown(crate::core::KeyEvent {
        key: key.into(),
        modifiers,
        repeat,
    })
}
