# Overflow Utilities And Clipping Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Expose Tailwind/CSS overflow utilities and make `overflow-hidden` / clipped overflow clip an element's own paint as well as its descendants.

**Architecture:** Keep the existing `Overflow` enum and per-axis `Style::overflow_x` / `Style::overflow_y` fields. Add adapter mappings on top of the existing style model, preserve Taffy-driven auto-size behavior for unconstrained in-flow children, and move runtime `PushClip` emission so an overflow-clipping node's own paint is inside its clip stack. Existing `Scroll` and `Auto` remain equivalent for scroll targeting in this phase.

**Tech Stack:** Rust 2024, `taffy` layout, retained `Element` tree, `DisplayList` paint commands, WGPU offscreen tests, existing adapter tests.

---

## Requirements From Brainstorming

- Support these utility class names in the Tailwind adapter:
  - `overflow-auto`
  - `overflow-hidden`
  - `overflow-visible`
  - `overflow-scroll`
  - `overflow-x-auto`
  - `overflow-y-auto`
  - `overflow-x-hidden`
  - `overflow-y-hidden`
  - `overflow-x-visible`
  - `overflow-y-visible`
  - `overflow-x-scroll`
  - `overflow-y-scroll`
- Keep current Rust API support:
  - `Element::overflow(Overflow::*)`
  - `Element::overflow_x(Overflow::*)`
  - `Element::overflow_y(Overflow::*)`
  - `Style::overflow_x`
  - `Style::overflow_y`
- Keep current RML support:
  - `overflow="hidden"`
  - `overflow-x="auto"`
  - `overflow-y="scroll"`
- Add CSS adapter support for:
  - `overflow: hidden`
  - `overflow-x: auto`
  - `overflow-y: scroll`
  - the same values `visible`, `hidden`, `clip`, `scroll`, `auto`
- Preserve HTML-like layout behavior:
  - unconstrained, in-flow children should size their parent through Taffy;
  - constrained parents should not expand after layout; overflow is handled by clipping/scroll/content size.
- Change paint semantics:
  - `overflow-hidden`, `overflow-clip`, `overflow-scroll`, and `overflow-auto` clip the element's own paint and descendant paint;
  - `overflow-visible` and unset overflow do not clip.
- Keep hit-testing consistent with paint clipping.
- Do not introduce a separate scrollbar visibility distinction between `Auto` and `Scroll` in this phase.

## File Structure

- Modify: `src/adapters/minimal_tailwind.rs`
  - Map the 12 requested overflow utility classes to `Style::overflow_x` and `Style::overflow_y`.
- Modify: `src/adapters/minimal_css.rs`
  - Parse `overflow`, `overflow-x`, and `overflow-y`.
- Modify: `src/runtime/runtime.rs`
  - Move overflow clip stack emission before `push_paint` so self paint is clipped.
- Modify: `tests/adapters.rs`
  - Add adapter coverage for Tailwind utility classes and CSS overflow properties.
- Modify: `tests/taffy_layout_parity.rs`
  - Add layout contract tests for auto-growth and constrained overflow content size.
- Modify: `tests/event_dispatch.rs`
  - Add hit-test contrast coverage for visible overflow vs hidden overflow.
- Modify: `tests/render_wgpu_offscreen_render.rs`
  - Add paint-pixel coverage for descendant clipping, visible overflow, and self-paint clipping.
- Modify: `docs/public-api.md`
  - Add a short overflow utility/support note and adapter example.

## Current Behavior Summary

- `Overflow::{Visible, Hidden, Clip, Scroll, Auto}` already exists.
- `Hidden`, `Clip`, `Scroll`, and `Auto` already clip descendants in layout, paint, and hit testing.
- `Visible` / unset overflow allows child paint and hit geometry to escape.
- `Scroll` and `Auto` both map to Taffy scroll overflow and both are scroll targets.
- Tailwind utilities are missing.
- CSS overflow properties are missing.
- A node's own painter output is not clipped by its own overflow because runtime currently emits node paint before `PushClip`.

---

### Task 1: Add Adapter Tests For Overflow Utilities

**Files:**
- Modify: `tests/adapters.rs`

- [ ] **Step 1: Write failing tests for Tailwind overflow classes**

Add `Overflow` to the existing imports:

```rust
use rgui::{Display, ElementKind, Length, Overflow, PrimitiveKind};
```

Add this test after `tailwind_adapter_maps_layout_classes_to_style_values`:

```rust
#[test]
fn tailwind_adapter_maps_overflow_utilities_to_style_values() {
    let style = tailwind::classes_to_style(
        "overflow-hidden overflow-x-auto overflow-y-scroll",
    )
    .unwrap();

    assert_eq!(style.overflow_x, Some(Overflow::Auto));
    assert_eq!(style.overflow_y, Some(Overflow::Scroll));

    let visible = tailwind::classes_to_style("overflow-visible").unwrap();
    assert_eq!(visible.overflow_x, Some(Overflow::Visible));
    assert_eq!(visible.overflow_y, Some(Overflow::Visible));

    let scroll = tailwind::classes_to_style("overflow-scroll").unwrap();
    assert_eq!(scroll.overflow_x, Some(Overflow::Scroll));
    assert_eq!(scroll.overflow_y, Some(Overflow::Scroll));
}
```

Add this test after `minimal_tailwind_adapter_keeps_existing_layout_smoke_test`:

```rust
#[test]
fn minimal_tailwind_adapter_maps_every_overflow_utility() {
    let cases = [
        ("overflow-auto", Some(Overflow::Auto), Some(Overflow::Auto)),
        ("overflow-hidden", Some(Overflow::Hidden), Some(Overflow::Hidden)),
        ("overflow-visible", Some(Overflow::Visible), Some(Overflow::Visible)),
        ("overflow-scroll", Some(Overflow::Scroll), Some(Overflow::Scroll)),
        ("overflow-x-auto", Some(Overflow::Auto), None),
        ("overflow-y-auto", None, Some(Overflow::Auto)),
        ("overflow-x-hidden", Some(Overflow::Hidden), None),
        ("overflow-y-hidden", None, Some(Overflow::Hidden)),
        ("overflow-x-visible", Some(Overflow::Visible), None),
        ("overflow-y-visible", None, Some(Overflow::Visible)),
        ("overflow-x-scroll", Some(Overflow::Scroll), None),
        ("overflow-y-scroll", None, Some(Overflow::Scroll)),
    ];

    for (class, expected_x, expected_y) in cases {
        let style = minimal_tailwind::classes_to_style(class).unwrap();
        assert_eq!(style.overflow_x, expected_x, "{class} overflow-x");
        assert_eq!(style.overflow_y, expected_y, "{class} overflow-y");
    }
}
```

- [ ] **Step 2: Write failing tests for CSS overflow properties**

Add this test after `css_adapter_maps_simple_properties_to_style_values`:

```rust
#[test]
fn css_adapter_maps_overflow_properties_to_style_values() {
    let style =
        css::css_to_style("overflow: hidden; overflow-x: auto; overflow-y: scroll").unwrap();

    assert_eq!(style.overflow_x, Some(Overflow::Auto));
    assert_eq!(style.overflow_y, Some(Overflow::Scroll));

    let visible = css::css_to_style("overflow: visible").unwrap();
    assert_eq!(visible.overflow_x, Some(Overflow::Visible));
    assert_eq!(visible.overflow_y, Some(Overflow::Visible));
}
```

Add this test after `minimal_css_adapter_keeps_existing_style_smoke_test`:

```rust
#[test]
fn minimal_css_adapter_maps_overflow_properties_to_style_values() {
    let style =
        minimal_css::css_to_style("overflow: clip; overflow-x: hidden; overflow-y: auto")
            .unwrap();

    assert_eq!(style.overflow_x, Some(Overflow::Hidden));
    assert_eq!(style.overflow_y, Some(Overflow::Auto));
}
```

- [ ] **Step 3: Run adapter tests and verify they fail**

Run:

```powershell
cargo test --test adapters overflow
```

Expected: compile or assertion failures because `minimal_tailwind` and `minimal_css` do not map overflow utilities/properties yet.

- [ ] **Step 4: Commit failing adapter tests**

```powershell
git add tests/adapters.rs
git commit -m "test(adapters): cover overflow utility parsing"
```

---

### Task 2: Implement Tailwind And CSS Overflow Parsing

**Files:**
- Modify: `src/adapters/minimal_tailwind.rs`
- Modify: `src/adapters/minimal_css.rs`

- [ ] **Step 1: Import `Overflow` in the Tailwind adapter**

Change the import in `src/adapters/minimal_tailwind.rs` to:

```rust
use crate::core::{Display, Edge, Length, Overflow, Style};
```

- [ ] **Step 2: Add overflow utility match arms**

Inside the `match class` in `classes_to_style`, add these arms before `_ => {}`:

```rust
"overflow-auto" => {
    style.overflow_x = Some(Overflow::Auto);
    style.overflow_y = Some(Overflow::Auto);
}
"overflow-hidden" => {
    style.overflow_x = Some(Overflow::Hidden);
    style.overflow_y = Some(Overflow::Hidden);
}
"overflow-visible" => {
    style.overflow_x = Some(Overflow::Visible);
    style.overflow_y = Some(Overflow::Visible);
}
"overflow-scroll" => {
    style.overflow_x = Some(Overflow::Scroll);
    style.overflow_y = Some(Overflow::Scroll);
}
"overflow-x-auto" => style.overflow_x = Some(Overflow::Auto),
"overflow-y-auto" => style.overflow_y = Some(Overflow::Auto),
"overflow-x-hidden" => style.overflow_x = Some(Overflow::Hidden),
"overflow-y-hidden" => style.overflow_y = Some(Overflow::Hidden),
"overflow-x-visible" => style.overflow_x = Some(Overflow::Visible),
"overflow-y-visible" => style.overflow_y = Some(Overflow::Visible),
"overflow-x-scroll" => style.overflow_x = Some(Overflow::Scroll),
"overflow-y-scroll" => style.overflow_y = Some(Overflow::Scroll),
```

- [ ] **Step 3: Import `Overflow` in the CSS adapter**

Change the import in `src/adapters/minimal_css.rs` to:

```rust
use crate::core::{Edge, Length, Overflow, Style};
```

- [ ] **Step 4: Add CSS overflow property parsing**

Inside the `match property` in `css_to_style`, add these arms before `_ => {}`:

```rust
"overflow" => {
    if let Some(overflow) = parse_overflow(value) {
        style.overflow_x = Some(overflow);
        style.overflow_y = Some(overflow);
    }
}
"overflow-x" => {
    if let Some(overflow) = parse_overflow(value) {
        style.overflow_x = Some(overflow);
    }
}
"overflow-y" => {
    if let Some(overflow) = parse_overflow(value) {
        style.overflow_y = Some(overflow);
    }
}
```

Add this helper below `parse_px`:

```rust
fn parse_overflow(value: &str) -> Option<Overflow> {
    match value.trim() {
        "visible" => Some(Overflow::Visible),
        "hidden" => Some(Overflow::Hidden),
        "clip" => Some(Overflow::Clip),
        "scroll" => Some(Overflow::Scroll),
        "auto" => Some(Overflow::Auto),
        _ => None,
    }
}
```

- [ ] **Step 5: Run adapter tests**

Run:

```powershell
cargo test --test adapters overflow
```

Expected: all overflow adapter tests pass.

- [ ] **Step 6: Commit adapter implementation**

```powershell
git add src/adapters/minimal_tailwind.rs src/adapters/minimal_css.rs tests/adapters.rs
git commit -m "feat(adapters): map overflow utilities"
```

---

### Task 3: Add Layout Contract Tests

**Files:**
- Modify: `tests/taffy_layout_parity.rs`

- [ ] **Step 1: Add a test proving unconstrained parents grow to in-flow children**

Add this test after `taffy_overflow_hidden_sets_clip_rect`:

```rust
#[test]
fn unconstrained_parent_expands_to_fit_in_flow_child() {
    let root = Element::column()
        .child(
            Element::column()
                .key("parent")
                .child(Element::text("Tall").height(120.0).key("child")),
        );

    let mut reconciler = Reconciler::default();
    let tree = reconciler.reconcile(root);

    let mut backend = TaffyLayoutBackend::new();
    let mut text = TextSystem::default();
    let result = backend.build_from_tree(
        &tree,
        &mut text,
        rgui::core::Size::new(400.0, 600.0),
        &rgui::Theme::light(),
    );

    let parent = box_for(&result, node_for_key(&tree, "parent"));
    let child = box_for(&result, node_for_key(&tree, "child"));

    assert!(parent.local_rect.size.height >= child.local_rect.size.height);
    assert_eq!(parent.clip_rect, None);
}
```

- [ ] **Step 2: Add a test proving constrained overflow does not resize parent**

Add this test after the previous test:

```rust
#[test]
fn constrained_overflow_hidden_keeps_parent_size_and_reports_content_size() {
    let root = Element::column()
        .height(40.0)
        .overflow(rgui::Overflow::Hidden)
        .key("parent")
        .child(Element::text("Tall").height(120.0).key("child"));

    let mut reconciler = Reconciler::default();
    let tree = reconciler.reconcile(root);

    let mut backend = TaffyLayoutBackend::new();
    let mut text = TextSystem::default();
    let result = backend.build_from_tree(
        &tree,
        &mut text,
        rgui::core::Size::new(400.0, 600.0),
        &rgui::Theme::light(),
    );

    let parent = box_for(&result, node_for_key(&tree, "parent"));

    assert_eq!(parent.local_rect.size.height, 40.0);
    assert_eq!(parent.clip_rect, Some(parent.local_rect));
    assert!(parent.content_size.height >= 120.0);
}
```

- [ ] **Step 3: Run layout contract tests**

Run:

```powershell
cargo test --test taffy_layout_parity overflow
```

Expected: existing and new overflow layout tests pass. If the auto-growth test fails, inspect whether `Element::text("Tall").height(120.0)` is treated as expected by Taffy; fix the test fixture before changing layout.

- [ ] **Step 4: Commit layout contract tests**

```powershell
git add tests/taffy_layout_parity.rs
git commit -m "test(layout): document overflow sizing contracts"
```

---

### Task 4: Add Paint And Hit-Test Overflow Contract Tests

**Files:**
- Modify: `tests/render_wgpu_offscreen_render.rs`
- Modify: `tests/event_dispatch.rs`

- [ ] **Step 1: Add offscreen descendant clipping tests**

In `tests/render_wgpu_offscreen_render.rs`, add this test near `push_clip_prevents_pixels_outside_clip_rect`:

```rust
#[test]
fn overflow_hidden_clips_descendant_paint() {
    let mut runtime = rgui::runtime::UiRuntime::default();
    let root = rgui::Element::column()
        .width(20.0)
        .height(20.0)
        .overflow(rgui::Overflow::Hidden)
        .child(
            rgui::Element::column()
                .width(40.0)
                .height(20.0)
                .style(rgui::Style::default().background(rgui::Color::rgb(255, 0, 0))),
        );

    let output = runtime.update(rgui::runtime::FrameInput {
        root,
        viewport: rgui::Size::new(64.0, 32.0),
        ..Default::default()
    });

    let mut renderer = pollster::block_on(WgpuRenderer::new_headless(RendererOptions {
        initial_size: SizeU32::new(64, 32),
        ..RendererOptions::default()
    }))
    .expect("renderer initializes");
    let target = OffscreenTarget::new(renderer.context(), SizeU32::new(64, 32));
    renderer
        .render_to_target(&output.display_list, &output.resources, target.view())
        .expect("runtime frame renders");
    let pixels = pollster::block_on(target.read_rgba8(renderer.context())).expect("readback works");

    assert_eq!(sample_pixel(&pixels, 64, 10, 10), [255, 0, 0, 255]);
    assert_eq!(sample_pixel(&pixels, 64, 30, 10), [0, 0, 0, 0]);
}
```

- [ ] **Step 2: Add offscreen visible overflow contrast test**

Add this test after the hidden descendant test:

```rust
#[test]
fn overflow_visible_allows_descendant_paint_outside_parent() {
    let mut runtime = rgui::runtime::UiRuntime::default();
    let root = rgui::Element::column()
        .width(20.0)
        .height(20.0)
        .overflow(rgui::Overflow::Visible)
        .child(
            rgui::Element::column()
                .width(40.0)
                .height(20.0)
                .style(rgui::Style::default().background(rgui::Color::rgb(255, 0, 0))),
        );

    let output = runtime.update(rgui::runtime::FrameInput {
        root,
        viewport: rgui::Size::new(64.0, 32.0),
        ..Default::default()
    });

    let mut renderer = pollster::block_on(WgpuRenderer::new_headless(RendererOptions {
        initial_size: SizeU32::new(64, 32),
        ..RendererOptions::default()
    }))
    .expect("renderer initializes");
    let target = OffscreenTarget::new(renderer.context(), SizeU32::new(64, 32));
    renderer
        .render_to_target(&output.display_list, &output.resources, target.view())
        .expect("runtime frame renders");
    let pixels = pollster::block_on(target.read_rgba8(renderer.context())).expect("readback works");

    assert_eq!(sample_pixel(&pixels, 64, 10, 10), [255, 0, 0, 255]);
    assert_eq!(sample_pixel(&pixels, 64, 30, 10), [255, 0, 0, 255]);
}
```

- [ ] **Step 3: Add hit-test visible overflow contrast test**

In `tests/event_dispatch.rs`, add this test after `clipped_scrolled_child_does_not_receive_pointer_outside_viewport`:

```rust
#[test]
fn overflow_visible_child_receives_pointer_outside_parent_bounds() {
    let mut runtime = UiRuntime::default();
    runtime.update(FrameInput {
        root: Element::column()
            .key("viewport")
            .width(40.0)
            .height(40.0)
            .overflow(Overflow::Visible)
            .child(button("Wide").width(120.0).height(40.0).key("wide")),
        viewport: Size::new(200.0, 120.0),
        ..Default::default()
    });

    runtime.dispatch(UiEvent::PointerDown(PointerEvent {
        position: Point::new(90.0, 20.0),
        button: Some(PointerButton::Primary),
        modifiers: 0,
    }));
    runtime.dispatch(UiEvent::PointerUp(PointerEvent {
        position: Point::new(90.0, 20.0),
        button: Some(PointerButton::Primary),
        modifiers: 0,
    }));

    assert!(runtime.command_count() > 0);
}
```

- [ ] **Step 4: Run paint and hit-test contract tests**

Run:

```powershell
cargo test --test render_wgpu_offscreen_render overflow_
cargo test --test event_dispatch overflow_visible_child_receives_pointer_outside_parent_bounds
```

Expected: descendant hidden and visible contrast tests pass with current runtime behavior. If `overflow_visible_child_receives_pointer_outside_parent_bounds` fails, inspect hit-test visible rect propagation before changing runtime.

- [ ] **Step 5: Commit paint and hit-test contract tests**

```powershell
git add tests/render_wgpu_offscreen_render.rs tests/event_dispatch.rs
git commit -m "test(runtime): cover overflow paint and hit contracts"
```

---

### Task 5: Add Failing Self-Paint Clipping Test

**Files:**
- Modify: `tests/runtime_pipeline.rs`

- [ ] **Step 1: Add imports for the self-paint test**

Ensure `tests/runtime_pipeline.rs` imports the runtime paint registry and paint command types. Add these imports if they are missing:

```rust
use rgui::core::{Color, PaintCommand, RectCmd, WidgetKind};
use rgui::runtime::paint::{
    PaintCtx, PaintedCommand, WidgetPainter, register_widget_painter, unregister_widget_painter,
};
```

If the file already imports some of these names, merge the imports instead of duplicating them.

- [ ] **Step 2: Add a custom painter that paints outside its own rect**

Add this near the top of `tests/runtime_pipeline.rs`, below existing imports:

```rust
struct OversizedBadgePainter;

impl WidgetPainter for OversizedBadgePainter {
    fn background_color(&self, _ctx: &PaintCtx<'_>) -> Color {
        Color::rgba(0, 0, 0, 0)
    }

    fn paint_content(&self, ctx: &mut PaintCtx<'_>, cmds: &mut Vec<PaintedCommand>) {
        let oversized = Rect::new(
            ctx.rect.origin,
            rgui::Size::new(ctx.rect.size.width * 2.0, ctx.rect.size.height),
        );
        cmds.push(ctx.draw_rect(oversized, Color::rgb(255, 0, 0), 0.0, 0));
    }
}

static OVERSIZED_BADGE_PAINTER: OversizedBadgePainter = OversizedBadgePainter;
```

- [ ] **Step 3: Add the failing DisplayList bracketing test**

Add this test near the other runtime/display-list paint tests:

```rust
#[test]
fn overflow_hidden_clips_node_own_paint_commands() {
    let _old = unregister_widget_painter(WidgetKind::Badge);
    register_widget_painter(WidgetKind::Badge, &OVERSIZED_BADGE_PAINTER);

    let mut runtime = rgui::runtime::UiRuntime::default();
    let output = runtime.update(rgui::runtime::FrameInput {
        root: rgui::widgets::badge("wide")
            .width(20.0)
            .height(20.0)
            .overflow(rgui::Overflow::Hidden),
        viewport: rgui::Size::new(64.0, 32.0),
        ..Default::default()
    });

    let commands = output.display_list.commands();
    let push_index = commands
        .iter()
        .position(|cmd| matches!(cmd, PaintCommand::PushClip(_)))
        .expect("overflow-hidden should push a clip");
    let oversized_rect_index = commands
        .iter()
        .position(|cmd| {
            matches!(
                cmd,
                PaintCommand::DrawRect(RectCmd { rect, .. })
                    if rect.size.width > 20.0
            )
        })
        .expect("custom oversized painter should emit a wide rect");
    let pop_index = commands
        .iter()
        .position(|cmd| matches!(cmd, PaintCommand::PopClip))
        .expect("overflow-hidden should pop the clip");

    unregister_widget_painter(WidgetKind::Badge);

    assert!(
        push_index < oversized_rect_index && oversized_rect_index < pop_index,
        "own paint must be bracketed by overflow clip"
    );
}
```

- [ ] **Step 4: Run the self-paint clipping test and verify it fails**

Run:

```powershell
cargo test --test runtime_pipeline overflow_hidden_clips_node_own_paint_commands
```

Expected: failure because `DrawRect` appears before `PushClip` in the current runtime.

- [ ] **Step 5: Commit failing self-paint test**

```powershell
git add tests/runtime_pipeline.rs
git commit -m "test(runtime): cover self paint overflow clipping"
```

---

### Task 6: Implement Self-Paint Overflow Clipping

**Files:**
- Modify: `src/runtime/runtime.rs`

- [ ] **Step 1: Move clip emission before node paint**

In `RuntimeFrameBuilder::push_node` in `src/runtime/runtime.rs`, locate this sequence:

```rust
self.push_semantics(tree, node, rect);
// Paint node background/content
self.push_paint(
    tree,
    node,
    rect,
    z_index,
    layout.content_size,
    layout.scroll_offset,
    layout.clip_rect,
);
// Collect overlay for deferred painting outside document clip stack
self.collect_overlay(tree, node, rect);

let pushes_new_clip = clip_rect.is_some() && clip_rect != inherited_clip;
if let Some(clip) = clip_rect.filter(|_| pushes_new_clip) {
    self.display_list
        .push(PaintCommand::PushClip(ClipSpec::rect(clip)));
}
```

Replace it with:

```rust
self.push_semantics(tree, node, rect);

let pushes_new_clip = clip_rect.is_some() && clip_rect != inherited_clip;
if let Some(clip) = clip_rect.filter(|_| pushes_new_clip) {
    self.display_list
        .push(PaintCommand::PushClip(ClipSpec::rect(clip)));
}

// Paint node background/content inside its own overflow clip so custom
// painters and canvas-like content follow CSS overflow-hidden semantics.
self.push_paint(
    tree,
    node,
    rect,
    z_index,
    layout.content_size,
    layout.scroll_offset,
    layout.clip_rect,
);
// Collect overlay for deferred painting outside document clip stack.
self.collect_overlay(tree, node, rect);
```

Do not move the existing `if pushes_new_clip { PopClip }` at the end of the function; it should continue to close the clip after descendants.

- [ ] **Step 2: Run the self-paint clipping test**

Run:

```powershell
cargo test --test runtime_pipeline overflow_hidden_clips_node_own_paint_commands
```

Expected: pass.

- [ ] **Step 3: Run existing clipping and event tests**

Run:

```powershell
cargo test --test event_dispatch clipped_scrolled_child_does_not_receive_pointer_outside_viewport
cargo test --test render_wgpu_render_items batches_split_when_layer_or_clip_changes
```

Expected: both pass; the display-list clip bracketing still lowers to clipped render-item batches correctly.

- [ ] **Step 4: Commit runtime clipping implementation**

```powershell
git add src/runtime/runtime.rs tests/runtime_pipeline.rs
git commit -m "fix(runtime): clip overflow node self paint"
```

---

### Task 7: Document Overflow Semantics

**Files:**
- Modify: `docs/public-api.md`

- [ ] **Step 1: Add an overflow utility note**

In the section that introduces `Overflow` or adapter helpers, add:

```markdown
Overflow is available through the Rust style API, RML attributes, CSS adapter
properties, and Tailwind-style utility classes. `overflow-hidden`,
`overflow-clip`, `overflow-scroll`, and `overflow-auto` clip both an element's
own paint and descendant paint. `overflow-visible` leaves paint and hit testing
unclipped unless an ancestor applies a clip.

Unconstrained in-flow children contribute to parent size through Taffy layout.
Once a parent has an explicit size, max size, root viewport constraint, or
scroll/clip overflow, overflow no longer resizes that parent after layout;
scroll containers instead report content size for scrolling and scrollbars.
```

- [ ] **Step 2: Add a Tailwind utility example**

Near the existing `minimal_tailwind::classes_to_style` example, add:

```rust
use rgui::{Overflow, adapters::minimal_tailwind::classes_to_style};

let style = classes_to_style("overflow-hidden overflow-y-auto")?;
assert_eq!(style.overflow_x, Some(Overflow::Hidden));
assert_eq!(style.overflow_y, Some(Overflow::Auto));
```

- [ ] **Step 3: Run doctests**

Run:

```powershell
cargo test --doc
```

Expected: doctests pass.

- [ ] **Step 4: Commit docs**

```powershell
git add docs/public-api.md
git commit -m "docs(style): document overflow utilities"
```

---

### Task 8: Full Verification

**Files:**
- No source edits expected.

- [ ] **Step 1: Run adapter tests**

Run:

```powershell
cargo test --test adapters
```

Expected: pass.

- [ ] **Step 2: Run layout overflow tests**

Run:

```powershell
cargo test --test taffy_layout_parity overflow
cargo test --test scroll_layout_contract
```

Expected: pass.

- [ ] **Step 3: Run runtime/event tests**

Run:

```powershell
cargo test --test event_dispatch
cargo test --test runtime_pipeline overflow_hidden_clips_node_own_paint_commands
```

Expected: pass.

- [ ] **Step 4: Run render tests affected by clipping**

Run:

```powershell
cargo test --test render_wgpu_offscreen_render overflow_
cargo test --test render_wgpu_render_items
```

Expected: pass.

- [ ] **Step 5: Run visual goldens**

Run:

```powershell
cargo test --test visual_goldens -j1
```

Expected: pass. If the new self-clipping behavior intentionally changes a golden, inspect the `target/rgui-goldens/diff` image before updating baselines.

- [ ] **Step 6: Inspect final diff**

Run:

```powershell
git diff --stat
git diff --check
```

Expected: `git diff --check` prints no errors.

- [ ] **Step 7: Commit verification-only docs if needed**

If Task 8 required only docs or golden-baseline updates, commit them:

```powershell
git add docs/public-api.md tests/goldens
git commit -m "docs(style): finalize overflow utility notes"
```

If no files changed during Task 8, do not create an empty commit.

---

## Final Acceptance Criteria

- All 12 requested Tailwind overflow utilities map to the expected `Style::overflow_x` / `Style::overflow_y` fields.
- CSS adapter parses `overflow`, `overflow-x`, and `overflow-y`.
- RML and Rust overflow APIs remain compatible.
- Unconstrained in-flow children still expand auto-sized parents.
- Constrained overflow containers do not resize after layout; they clip or scroll and report content size.
- `overflow-hidden` clips an element's own paint and descendant paint.
- `overflow-visible` allows descendant paint and hit testing outside parent bounds when no ancestor clips it.
- Existing scroll, hit-test, offscreen render, and visual golden tests pass.

## Explicitly Out Of Scope

- Distinguishing scrollbar visibility between `Overflow::Auto` and `Overflow::Scroll`.
- Adding new `Overflow` enum variants.
- Replacing the existing `ScrollArea` widget.
- Implementing CSS shorthand beyond the listed overflow properties.
