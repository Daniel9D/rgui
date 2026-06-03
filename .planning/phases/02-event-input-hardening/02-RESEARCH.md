# Phase 2: Event & Input Hardening — Research

**Phase:** 2 of 8
**Plan count:** 4 (Focus traversal / Shortcut suppression / Wheel 2D / IME)

## Research question

What does it take to make the runtime's event path robust enough
that a real desktop application (form with Tab + modal + Cmd+K
palette + horizontal trackpad + CJK IME) works without falling
through to the host's native behavior?

## Existing code to build on

### FocusManager (`src/core/event.rs:120-137`)

```rust
pub struct FocusManager { focused: Option<NodeId> }
impl FocusManager {
    pub fn request_focus(&mut self, node: NodeId) { ... }
    pub fn clear(&mut self) { ... }
    pub const fn focused(&self) -> Option<NodeId> { ... }
}
```

The struct is a single `Option<NodeId>`. No tab_next, no focus
order, no focus traps. Phase 2 extends it with:
- `tab_next(&mut self, tree: &UiTree) -> Option<NodeId>`
- `tab_prev(&mut self, tree: &UiTree) -> Option<NodeId>`
- `is_focusable(node: &UiNode) -> bool`
- The `ModalSpec.trap_focus: bool` field

### ShortcutRegistry (`src/core/event.rs:164-196`)

```rust
pub struct ShortcutRegistry { shortcuts: Vec<Shortcut> }
impl ShortcutRegistry {
    pub fn resolve(&self, chord: &str, focused: Option<NodeId>) -> Option<&str> { ... }
}
```

The current `resolve` doesn't check whether the focused node is a
text input. Phase 2 adds a `focused_is_text_input: bool` argument
and suppresses non-modifier shortcuts when in a text field.

### WheelEvent (`src/core/event.rs:31-36`)

```rust
pub struct WheelEvent {
    pub delta: Vec2,
    pub position: Point,
    pub mode: WheelDeltaMode,
}
```

The struct already carries `delta: Vec2`. The dispatch in
`runtime/events.rs` only reads `delta.y`. Phase 2 also reads
`delta.x` and routes it to the matching scroll container.

### Dispatch path (`src/runtime/events.rs`)

The dispatch walks the event → hit-test → handler tree. The four
plans each patch one branch:
- 02-01: `KeyDown(Tab)` → focus traversal
- 02-02: `KeyDown` + shortcut check (suppression)
- 02-03: `Wheel` → 2D scroll
- 02-04: `ImePreedit` / `ImeCommit` → input state

### Widget specs that need new fields

- `InputSpec` / `TextareaSpec`: `ime_enabled: bool` (default false)
- `ModalSpec`: `trap_focus: bool` (default false)
- `ScrollArea` primitive: `scroll_x: bool` + `scroll_y: bool` (default `true, true`)

These are all `#[non_exhaustive]` so adding a field with `Default` is non-breaking.

## Pitfalls (from PITFALLS.md)

- **#3 (Focus loss to host)**: The browser / winit will consume Tab by default (it moves focus to the next browser-control element). The runtime must call `prevent_default()` on the Tab key event when it handles it.
- **#5 (Pointer-capture leak)**: Phase 1 fixed this for unmount. Phase 2 must preserve the fix when focus moves (a focused node that's a button captures pointer on Down; the capture must release when focus moves elsewhere).
- **#9 (Shortcuts fire inside text fields)**: This is the entire motivation for Plan 02-02.

## Reference: how other toolkits solve this

- **React**: A separate `focus-trap-react` library handles modal traps. The pattern is "if a modal is open, intercept Tab and cycle inside".
- **egui**: Has `Memory::focus` and `Memory::set_focus`; tab traversal walks focusable widgets in iteration order. No focus traps by default.
- **Flutter**: `FocusScope` + `FocusScopeNode` for traps. The widget tree builds a focus tree parallel to the widget tree; the runtime walks the focus tree on Tab.
- **SwiftUI**: `@FocusState` + `.focused($focused, equals: .someValue)` for individual focus; `.focusSection()` for traps.

For v1, the rsgui approach is closest to Flutter's: a `FocusManager` (single `Option<NodeId>`) + a per-element `trap_focus` boolean. No focus tree (the dispatch walks the UiTree directly).

## Specific implementation choices

### Plan 02-01 (Focus traversal)

```rust
impl FocusManager {
    /// Walk to the next focusable node in DOM order from `current`.
    /// If `current` is `None`, return the first focusable node.
    /// If `current` is the last focusable, return the first (wrap).
    pub fn tab_next(&mut self, tree: &UiTree) -> Option<NodeId>;

    pub fn tab_prev(&mut self, tree: &UiTree) -> Option<NodeId>;

    /// True if the node is a focusable widget. Currently:
    /// Button, Input, Checkbox, Radio, Select, Textarea, Switch,
    /// Slider, Link, Tabs (active tab), MenuItem.
    pub fn is_focusable(node: &UiNode) -> bool;
}
```

Focus traps: when the modal's `trap_focus` is true, `tab_next`
filters focusable candidates to those *inside* the modal's
subtree. The modal's root is in `tree.ancestor_ids(focused)` — we
restrict to nodes that are descendants of the modal.

### Plan 02-02 (Shortcut suppression)

```rust
impl ShortcutRegistry {
    /// `chord` is the normalized key chord (e.g. "Cmd+K", "A",
    /// "Enter"). `focused` is the currently focused node (if any).
    /// `focused_is_text_input` is `true` if the focused node is an
    /// Input, Textarea, or Select.
    ///
    /// When `focused_is_text_input` is true, only shortcuts
    /// containing a `Cmd`/`Ctrl`/`Alt` modifier are resolved. Plain
    /// letter / digit / punctuation shortcuts (e.g. "A", "?", "1")
    /// are suppressed so the user can type them.
    pub fn resolve(
        &self,
        chord: &str,
        focused: Option<NodeId>,
        focused_is_text_input: bool,
    ) -> Option<&str>;
}
```

### Plan 02-03 (Wheel 2D + nested scroll)

The wheel dispatch already does:
```rust
scroll_offset_for(scroll_area) += wheel_event.delta.y;
```

Phase 2 extends to:
```rust
let hit = hit_test.wheel_target(point); // deepest ScrollArea under the pointer
if hit.kind == ScrollArea { 
    if hit.style.scroll_x { offset.x += wheel.delta.x; }
    if hit.style.scroll_y { offset.y += wheel.delta.y; }
    clamp(offset, scroll_bounds);
}
```

For nested scroll areas (e.g. a vertical inside a horizontal):
- The deepest one consumes the wheel that matches its axis.
- If the deepest one can't scroll further on its axis, the wheel
  bubbles to the parent.

### Plan 02-04 (IME)

```rust
// In `InputSpec`:
pub struct InputSpec {
    // ... existing fields ...
    pub ime_enabled: bool,  // default false
}

// In `InputState` (runtime state for an Input):
pub struct InputState {
    pub value: String,
    pub preedit: String,        // current IME preedit text
    pub preedit_cursor: Option<(usize, usize)>,
    pub cursor_byte: usize,
    // ...
}
```

Dispatch:
- `ImePreedit(text, range)`: if the focused Input has `ime_enabled`,
  set `state.preedit = text`.
- `ImeCommit(text)`: if the focused Input has `ime_enabled`, append
  `text` to `state.value` and clear `preedit`.

The display path renders `value` with `preedit` underlined (or
highlighted). The dispatch already knows about the `preedit`
field; this is a paint change too, captured in 02-04.

## Sources

- PITFALLS.md #3, #5, #9 (focus loss, capture leak, shortcuts in text fields)
- `src/core/event.rs` (the existing event types)
- `src/runtime/events.rs` (the dispatch)
- `src/widgets/spec.rs` (the spec types that get new fields)
- `feedback.md` 5.5 (events module docs)
- ROADMAP.md Phase 2 entry
- REQUIREMENTS.md EVNT-01..06

## What ships in v1

After this phase:
- Forms with Tab cycling (EVNT-04)
- Modals with focus traps (EVNT-04 sub-feature)
- Shortcuts that respect text-input context (EVNT-05)
- Trackpad horizontal scrolling (EVNT-03)
- CJK + other IME-using languages (EVNT-06)
- Robust keyboard event routing (EVNT-02)

What does **not** ship:
- Custom tabindex order (deferred; users use `request_focus` directly)
- Focus ring (the OS draws it)
- Momentum scrolling (v1.x)
- Touch / pen / gamepad (v1.x)
- South-East Asian complex-script IME state machine (v1.x)
