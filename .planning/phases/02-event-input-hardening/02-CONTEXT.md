# Phase 2: Event & Input Hardening

**Phase:** 2 of 8
**Depends on:** Phase 1 (Incremental Reconciliation) — pointer-capture release, focus keyed by NodeId, state preserved across patches

## Phase Boundary

Phase 1 made the runtime efficient: it can diff two `Element` trees in microseconds, release captures on unmount, and lay out only the dirty subtree. But the *event path* still has known gaps:

- **Focus traversal is broken**: `FocusManager` exists but has no Tab/Shift+Tab logic. Pressing Tab in a 5-field form does nothing.
- **Shortcuts fire inside text inputs**: `ShortcutRegistry::resolve` doesn't check whether the focused node is a text field, so `Cmd+A` (select all) in an `Input` element both fires the global "select all" shortcut *and* the native browser behavior. v1 needs to suppress shortcuts when typing.
- **Wheel events only do vertical scroll**: `WheelEvent` carries `Vec2` (so the field exists), but the dispatch only handles `delta.y`. Horizontal scrolling (e.g. on a trackpad) is silently dropped.
- **IME composition is half-wired**: `ImePreedit` / `ImeCommit` events exist, but the `Input` element doesn't accumulate preedit text. East-Asian users can't type.

This phase fixes all four.

## Implementation Decisions (locked in this context)

1. **Focus order = DOM order.** No `tabindex` attribute. v1 ships with the simplest correct behavior; users who need custom focus order can call `FocusManager::request_focus` directly. (Custom `tabindex` is a v1.x follow-up.)
2. **Focus traps are explicit, not automatic.** A modal with `trap_focus: true` (set on the `ModalSpec`) loops Tab inside itself. This is the only "non-default" focus behavior; everything else is plain DOM order.
3. **Shortcuts are suppressed when the focused node is a text input.** Detection uses `widget_spec.is_some_and(|s| matches!(s, WidgetSpec::Input(_) | WidgetSpec::Textarea(_) | WidgetSpec::Select(_)))`. Modifier-only shortcuts (e.g. `Cmd+K`) still fire because the user explicitly pressed a modifier.
4. **Wheel = Vec2, both axes always delivered.** Taffy-style scroll containers accept both axes. Single-axis wheels work as today; trackpad-style 2D wheels scroll both. No momentum in v1 (per the prd.md non-goals).
5. **IME is opt-in via `Input::ime_enabled(true)`.** The default `Input` element processes key events directly (good for Latin keyboards). Apps targeting CJK users set `ime_enabled(true)` and the runtime routes `ImePreedit` / `ImeCommit` to the focused `Input`.

## Canonical References

- **`src/core/event.rs:120-137`**: existing `FocusManager` (request_focus / clear / focused only).
- **`src/core/event.rs:164-196`**: existing `ShortcutRegistry::resolve` (no text-input suppression).
- **`src/core/event.rs:31-36`**: existing `WheelEvent { delta: Vec2, ... }` (delta is 2D, but dispatch handles only Y).
- **`src/runtime/events.rs`**: the dispatch path that routes events to handlers. Phase 2 patches this in 4 places (one per plan).
- **`src/widgets/spec.rs:78-93`**: `InputSpec` / `TextareaSpec` / `SelectSpec` (the "is text input" detector in Decision 3).
- **`src/widgets/spec.rs:218-222`**: `ModalSpec` (where `trap_focus` will live, Decision 2).
- **`feedback.md` 5.5** (events module docs) — Phase 2 expands those docs with the four behaviors.

## Specific Ideas

- **Plan 02-01 (Focus traversal)**: Add `FocusManager::tab_next(&mut self, tree: &UiTree)` and `tab_prev` that walk focusable elements in DOM order. Focusable = `widget_spec.is_some_and(is_focusable_widget)`. A modal with `trap_focus: true` loops inside its own subtree. Tests: a 5-button row with Tab cycles 1→2→3→4→5→1; Shift+Tab goes backward; modal traps loop.
- **Plan 02-02 (Shortcut suppression)**: Modify `ShortcutRegistry::resolve` to take a `focused_is_text_input: bool` argument; if `true`, skip shortcuts that don't have a `Modifier+NonMod` pattern (i.e. `Cmd+K` fires, `A` doesn't). New `ShortcutsAlwaysFire` list: `Cmd+Anything`, `Ctrl+Anything`, `Alt+Anything`. Everything else is suppressed inside text fields. Tests: pressing `A` in an `Input` doesn't fire a global "a" shortcut; pressing `Cmd+A` in an `Input` does.
- **Plan 02-03 (Wheel 2D + scroll)**: The dispatch already does `wheel_event.delta.y` → scroll. Add `delta.x` → horizontal scroll. Nested `ScrollArea` elements route the wheel to the deepest one under the pointer. Tests: trackpad pan on a horizontal scroll area scrolls horizontally; wheel on a vertical-only area scrolls vertically; nested scroll areas (vertical inside horizontal) consume the wheel that matches their axis.
- **Plan 02-04 (IME)**: New `InputSpec::ime_enabled: bool` (default false). The runtime, when an `Input` with `ime_enabled` is focused, routes `ImePreedit` events to a new `InputState.preedit: String` field; `ImeCommit` appends the committed text. The display shows the preedit text underlined. Tests: typing a Japanese character (e.g. あ + preedit) appears as preedit until commit; the committed text replaces the preedit.

## Deferred Ideas (out of scope for v1)

- **Custom `tabindex`**: users can call `request_focus` directly. v1.x follow-up adds `Element::tab_index(i32)`.
- **Focus visible / focus ring**: v1 doesn't draw a focus ring; the underlying OS / browser draws it (where applicable). v1.x adds an `OutlineKind::Focus` style.
- **Momentum scrolling for wheels**: requires physics simulation; deferred to v1.x.
- **IME per-character preedit composition (Hindi, Thai, Khmer)**: v1 covers the CJK preedit-then-commit model. South-East Asian complex scripts need a different state machine (clusters). Deferred to v1.x.
- **Gamepad / touch / pen events**: v1 is mouse + keyboard + wheel + IME only. Touch is a v1.x follow-up.

## How to know this phase is done

- A 5-button form with Tab/Shift+Tab cycles focus correctly (no focus loss to the browser).
- A modal opened with a button inside traps focus; Tab loops inside until Escape.
- `Cmd+A` works in a text input; pressing plain `A` does not fire any global shortcut.
- A horizontal trackpad pan on a horizontal-only scroll area scrolls horizontally.
- An `Input` with `ime_enabled(true)` accepts Japanese IME preedit text and commits it correctly.
- All four scenarios are covered by integration tests under `tests/event_input_hardening.rs`.
- `cargo test --features rml,bitmap-text-fallback` shows ≥ 10 new tests, all green.

## Requirements covered

This phase closes the v1 requirements: **EVNT-01**, **EVNT-02**, **EVNT-03**, **EVNT-04**, **EVNT-05**, **EVNT-06**. See `REQUIREMENTS.md` for the full text.

## Risks

- **Focus traversal is order-sensitive**: if the DOM order changes (e.g. a new child is inserted), the focus index changes. The integration test must cover this case.
- **Shortcut suppression can be surprising**: a user expects `Cmd+K` to open a palette even when an Input is focused. The "modifiers always fire" rule handles this; tests assert it.
- **Wheel 2D can break scroll containers that assume single-axis**: existing scroll containers read `delta.y` only. Phase 2 patches them; tests on the existing 8+ scroll-area visual goldens must still pass.
- **IME is platform-specific**: `ImePreedit` is delivered by the host (winit on desktop, browser on web). v1 only tests the *runtime's* response to the event, not the host's behavior. The integration test simulates the events directly.
