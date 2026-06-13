# Linear Gradients Render Fidelity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

**Goal:** Render `Paint::LinearGradient` as a real WGPU linear gradient instead of a white rectangle.

**Architecture:** Preserve the existing retained render pipeline: `DisplayList` validates commands, `item.rs` lowers commands into sorted `RenderItem`s, `batch.rs` groups compatible items, and WGPU draws instanced quads. This phase adds a `LinearGradient` pipeline and explicit per-instance gradient data, while leaving runtime paint and widget APIs unchanged.

**Tech Stack:** Rust 2024, `wgpu` 29, WGSL shaders, `bytemuck`, existing offscreen WGPU tests, existing visual golden harness.

---

## Scope Check

The approved spec is a five-phase render-fidelity roadmap. This plan implements only Phase 1: Linear Gradients, because it is the first independently testable slice and produces working software on its own. Rounded borders, soft shadows, real path strokes, and resource fidelity should each receive their own implementation plan after this phase lands.

## Current Status

Status: **complete as of 2026-06-13**.

Implementation commits on the current branch:

- `80fd0e4` - `test(render): cover linear gradient validation`
- `bd219da` - `feat(render): validate linear gradient paint`
- `d7a24ab` - `test(render): cover linear gradient lowering`
- `3af1ebd` - `feat(render): lower linear gradients to render items`
- `903d752` - `fix(render): register linear gradient pipeline kind`
- `78d9cbf` - `test(render): cover gradient shader instance layout`
- `2901ddb` - `feat(render): add linear gradient shader`
- `81e002b` - `docs(render): refresh gradient shader comments`
- `65c6204` - `test(render): verify linear gradient pixels`
- `4da7dae` - `docs(render): document linear gradient instance fields`
- `b50ffa1` - `fix(render): close linear gradient fidelity gaps`

Verification evidence from the completed phase:

- `cargo test --test render_wgpu_render_items` - passed, 20 tests.
- `cargo test --test render_wgpu_offscreen_render` - passed, 26 tests.
- `cargo test --doc` - passed, 49 doctests.
- `cargo test --test visual_goldens -j1` - passed, 11 tests.
- `cargo test --features vulkan-goldens --test visual_goldens_vulkan -j1` - passed, 9 tests.
- `git diff --check` - clean before the final implementation commit.

Task 9 is **not** the next implementation step anymore. It was completed after the rounded-gradient and gradient-layer-order follow-up concerns were addressed. The default visual golden baselines were added in `b50ffa1` so the visual-golden gate is reproducible from the branch.

## File Structure

- Modify: `src/core/render.rs`
  - Validate linear gradient start/end points and stop positions during `DisplayList::validate`.
  - Add structured `DisplayListError` variants for invalid gradient stop data.
- Modify: `src/render/wgpu/item.rs`
  - Add gradient fields to `RenderItem`.
  - Lower `Paint::LinearGradient` to `PipelineKind::LinearGradient`.
  - Normalize more-than-two stops to first and last effective stops.
- Modify: `src/render/wgpu/pipeline.rs`
  - Add `PipelineKind::LinearGradient`.
  - Extend `InstanceRaw` with per-instance gradient vector and end color data.
  - Add vertex attributes for the new fields.
  - Wire `LinearGradient` to `fs_linear_gradient`.
- Modify: `src/render/wgpu/shaders.rs`
  - Add gradient fields to `VertexOut`.
  - Pass gradient vector and end color from vertex to fragment.
  - Add `fs_linear_gradient`.
- Modify: `src/render/wgpu/mod.rs`
  - Populate the new `InstanceRaw` fields in `instances_for_items`.
- Modify: `tests/render_wgpu_render_items.rs`
  - Add lowering tests for gradient pipeline selection and normalized stops.
  - Update the public GPU layout test for the new `InstanceRaw` size.
- Modify: `tests/render_wgpu_offscreen_render.rs`
  - Add offscreen pixel tests for gradient start/end/midpoint, clipping, and z-order.
- Modify: `docs/public-api.md`
  - Update public WGPU examples that construct `RenderItem`.
  - Note the `InstanceRaw` layout change and `LinearGradient` pipeline variant.

## Implementation Decision

Use additive public fields instead of clever overloading:

```rust
pub struct RenderItem {
    pub layer: LayerKind,
    pub clip_rect: Option<Rect>,
    pub pipeline: PipelineKind,
    pub rect: Rect,
    pub color: [f32; 4],
    pub uv_rect: [f32; 4],
    pub radius: f32,
    pub z_index: i32,
    pub order: u64,
    pub gradient: [f32; 4],
    pub gradient_end_color: [f32; 4],
}
```

```rust
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct InstanceRaw {
    pub rect: [f32; 4],
    pub color: [f32; 4],
    pub uv_rect: [f32; 4],
    pub viewport: [f32; 4],
    pub flags: [f32; 4],
    pub gradient: [f32; 4],
    pub gradient_end_color: [f32; 4],
}
```

This changes a public type. Because the crate is pre-1.0 and no `CHANGELOG.md` exists, document the migration in `docs/public-api.md` in the WGPU renderer section during Task 8.

### Gradient Semantics

- `RenderItem.color` stores the first stop color after opacity is applied.
- `RenderItem.gradient_end_color` stores the last stop color after opacity is applied.
- `RenderItem.gradient` stores absolute gradient endpoints as `[start_x, start_y, end_x, end_y]`.
- More than two stops are normalized to first and last stops for this phase.
- A gradient with fewer than two stops fails validation before WGPU lowering.
- A zero-length gradient vector renders the first stop color.

---

### Task 1: Add Gradient Validation Tests

**Files:**
- Modify: `src/core/render.rs`

- [x] **Step 1: Write failing tests for gradient validation**

Add these tests inside the existing `#[cfg(test)] mod tests` in `src/core/render.rs`:

```rust
#[test]
fn validate_accepts_linear_gradient_with_two_finite_stops() {
    let mut list = DisplayList::default();
    list.push(PaintCommand::DrawRect(RectCmd {
        rect: Rect::new(Point::new(0.0, 0.0), crate::core::Size::new(10.0, 10.0)),
        paint: Paint::LinearGradient {
            start: Point::new(0.0, 0.0),
            end: Point::new(10.0, 0.0),
            stops: vec![
                (0.0, Color::rgb(255, 0, 0)),
                (1.0, Color::rgb(0, 0, 255)),
            ],
        },
        radius: 0.0,
        opacity: 1.0,
        z_index: 0,
    }));

    assert_eq!(list.validate(), Ok(()));
}

#[test]
fn validate_rejects_linear_gradient_with_too_few_stops() {
    let mut list = DisplayList::default();
    list.push(PaintCommand::DrawRect(RectCmd {
        rect: Rect::new(Point::new(0.0, 0.0), crate::core::Size::new(10.0, 10.0)),
        paint: Paint::LinearGradient {
            start: Point::new(0.0, 0.0),
            end: Point::new(10.0, 0.0),
            stops: vec![(0.0, Color::rgb(255, 0, 0))],
        },
        radius: 0.0,
        opacity: 1.0,
        z_index: 0,
    }));

    assert_eq!(list.validate().unwrap_err(), DisplayListError::GradientTooFewStops);
}

#[test]
fn validate_rejects_linear_gradient_with_non_finite_stop_position() {
    let mut list = DisplayList::default();
    list.push(PaintCommand::DrawRect(RectCmd {
        rect: Rect::new(Point::new(0.0, 0.0), crate::core::Size::new(10.0, 10.0)),
        paint: Paint::LinearGradient {
            start: Point::new(0.0, 0.0),
            end: Point::new(10.0, 0.0),
            stops: vec![
                (0.0, Color::rgb(255, 0, 0)),
                (f32::NAN, Color::rgb(0, 0, 255)),
            ],
        },
        radius: 0.0,
        opacity: 1.0,
        z_index: 0,
    }));

    assert_eq!(
        list.validate().unwrap_err(),
        DisplayListError::NonFiniteGradientStop { index: 1 }
    );
}
```

- [x] **Step 2: Run tests and verify they fail**

Run:

```powershell
cargo test validate_
```

Expected: compile failure mentioning missing `DisplayListError::GradientTooFewStops` and `DisplayListError::NonFiniteGradientStop`.

- [x] **Step 3: Commit the failing tests**

```powershell
git add src/core/render.rs
git commit -m "test(render): cover linear gradient validation"
```

---

### Task 2: Implement Gradient Validation

**Files:**
- Modify: `src/core/render.rs`

- [x] **Step 1: Add new error variants**

Add these variants to `DisplayListError` after `PathTooShort`:

```rust
/// A linear gradient had fewer than two color stops.
GradientTooFewStops,
/// A linear gradient stop position was not finite.
NonFiniteGradientStop {
    /// Index of the invalid stop.
    index: usize,
},
```

- [x] **Step 2: Add display messages**

Add these match arms in `impl std::fmt::Display for DisplayListError`:

```rust
Self::GradientTooFewStops => {
    write!(f, "linear gradient must contain at least two stops")
}
Self::NonFiniteGradientStop { index } => {
    write!(f, "linear gradient stop {index} position must be finite")
}
```

- [x] **Step 3: Add gradient paint validation helper**

Add this helper near the existing validation helpers:

```rust
fn validate_paint(paint: &Paint) -> Result<(), DisplayListError> {
    match paint {
        Paint::Solid(_) | Paint::Image(_) => Ok(()),
        Paint::LinearGradient { start, end, stops } => {
            validate_point(*start, "gradient start")?;
            validate_point(*end, "gradient end")?;
            if stops.len() < 2 {
                return Err(DisplayListError::GradientTooFewStops);
            }
            for (index, (position, _color)) in stops.iter().enumerate() {
                if !position.is_finite() {
                    return Err(DisplayListError::NonFiniteGradientStop { index });
                }
            }
            Ok(())
        }
    }
}
```

- [x] **Step 4: Call the helper from rect validation**

In the `PaintCommand::DrawRect(cmd)` branch of `DisplayList::validate`, change it to:

```rust
PaintCommand::DrawRect(cmd) => {
    validate_rect(cmd.rect)?;
    validate_paint(&cmd.paint)?;
    validate_non_negative(cmd.radius, "rect radius")?;
    validate_non_negative(cmd.opacity, "rect opacity")?;
}
```

- [x] **Step 5: Update the display-message unit test**

In `display_list_error_display_renders_readable_message`, add these cases:

```rust
(
    DisplayListError::GradientTooFewStops,
    "linear gradient must contain at least two stops",
),
(
    DisplayListError::NonFiniteGradientStop { index: 2 },
    "linear gradient stop 2 position must be finite",
),
```

- [x] **Step 6: Run validation tests**

Run:

```powershell
cargo test validate_
cargo test display_list_error_display_renders_readable_message
```

Expected: all four tests pass.

- [x] **Step 7: Commit validation implementation**

```powershell
git add src/core/render.rs
git commit -m "feat(render): validate linear gradient paint"
```

---

### Task 3: Add Render-Item Lowering Tests

**Files:**
- Modify: `tests/render_wgpu_render_items.rs`

- [x] **Step 1: Write failing lowering tests**

Add these tests after `lowers_rect_commands_to_solid_items_with_order_and_z_index`:

```rust
#[test]
fn lowers_linear_gradient_rect_to_gradient_render_item() {
    let mut list = DisplayList::default();
    list.push(PaintCommand::DrawRect(RectCmd {
        rect: Rect::new(Point::new(4.0, 8.0), Size::new(20.0, 10.0)),
        paint: Paint::LinearGradient {
            start: Point::new(4.0, 8.0),
            end: Point::new(24.0, 8.0),
            stops: vec![
                (0.0, Color::rgba(255, 0, 0, 200)),
                (1.0, Color::rgba(0, 0, 255, 128)),
            ],
        },
        radius: 0.0,
        opacity: 0.5,
        z_index: 7,
    }));

    let renderer = WgpuRenderer::new_headless_for_tests();
    let items = build_render_items(&list, &ResourceStore::default(), &mut *renderer.atlas_mut())
        .expect("valid gradient display list lowers");

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].pipeline, PipelineKind::LinearGradient);
    assert_eq!(items[0].gradient, [4.0, 8.0, 24.0, 8.0]);
    assert_eq!(items[0].color, [1.0, 0.0, 0.0, 200.0 / 255.0 * 0.5]);
    assert_eq!(
        items[0].gradient_end_color,
        [0.0, 0.0, 1.0, 128.0 / 255.0 * 0.5]
    );
    assert_eq!(items[0].z_index, 7);
}

#[test]
fn linear_gradient_lowering_uses_first_and_last_stop() {
    let mut list = DisplayList::default();
    list.push(PaintCommand::DrawRect(RectCmd {
        rect: Rect::new(Point::new(0.0, 0.0), Size::new(30.0, 10.0)),
        paint: Paint::LinearGradient {
            start: Point::new(0.0, 0.0),
            end: Point::new(30.0, 0.0),
            stops: vec![
                (0.0, Color::rgb(255, 0, 0)),
                (0.5, Color::rgb(0, 255, 0)),
                (1.0, Color::rgb(0, 0, 255)),
            ],
        },
        radius: 0.0,
        opacity: 1.0,
        z_index: 0,
    }));

    let renderer = WgpuRenderer::new_headless_for_tests();
    let items = build_render_items(&list, &ResourceStore::default(), &mut *renderer.atlas_mut())
        .expect("valid gradient display list lowers");

    assert_eq!(items[0].color, [1.0, 0.0, 0.0, 1.0]);
    assert_eq!(items[0].gradient_end_color, [0.0, 0.0, 1.0, 1.0]);
}
```

- [x] **Step 2: Run tests and verify they fail**

Run:

```powershell
cargo test --test render_wgpu_render_items linear_gradient
```

Expected: compile failure because `PipelineKind::LinearGradient`, `RenderItem::gradient`, and `RenderItem::gradient_end_color` do not exist.

- [x] **Step 3: Commit failing lowering tests**

```powershell
git add tests/render_wgpu_render_items.rs
git commit -m "test(render): cover linear gradient lowering"
```

---

### Task 4: Implement Render-Item Gradient Lowering

**Files:**
- Modify: `src/render/wgpu/pipeline.rs`
- Modify: `src/render/wgpu/item.rs`
- Modify: `tests/render_wgpu_render_items.rs`

- [x] **Step 1: Add the pipeline variant**

In `PipelineKind`, add `LinearGradient` after `RoundedRect`:

```rust
pub enum PipelineKind {
    SolidRect,
    RoundedRect,
    LinearGradient,
    Border,
    TextGlyph,
    Image,
    Svg,
    Path,
}
```

- [x] **Step 2: Add gradient fields to `RenderItem`**

In `src/render/wgpu/item.rs`, change `RenderItem` to:

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderItem {
    pub layer: LayerKind,
    pub clip_rect: Option<Rect>,
    pub pipeline: PipelineKind,
    pub rect: Rect,
    pub color: [f32; 4],
    pub uv_rect: [f32; 4],
    pub radius: f32,
    pub z_index: i32,
    pub order: u64,
    pub gradient: [f32; 4],
    pub gradient_end_color: [f32; 4],
}
```

- [x] **Step 3: Add a default helper for non-gradient items**

Add this helper near `missing_resource_item`:

```rust
fn no_gradient(color: [f32; 4]) -> ([f32; 4], [f32; 4]) {
    ([0.0, 0.0, 0.0, 0.0], color)
}
```

- [x] **Step 4: Add a gradient normalizer**

Add this helper near `push_rect`:

```rust
fn linear_gradient_parts(
    start: Point,
    end: Point,
    stops: &[(f32, crate::core::Color)],
    opacity: f32,
) -> ([f32; 4], [f32; 4], [f32; 4]) {
    let first = stops[0].1;
    let last = stops[stops.len() - 1].1;
    (
        color_to_linear(first, opacity),
        color_to_linear(last, opacity),
        [start.x, start.y, end.x, end.y],
    )
}
```

- [x] **Step 5: Update `push_rect` to lower gradients**

Replace the current `(pipeline, color)` match in `push_rect` with:

```rust
let (pipeline, color, gradient, gradient_end_color) = match &cmd.paint {
    Paint::Solid(color) => {
        let color = color_to_linear(*color, cmd.opacity);
        let (gradient, gradient_end_color) = no_gradient(color);
        (base_pipeline, color, gradient, gradient_end_color)
    }
    Paint::LinearGradient { start, end, stops } => {
        let (color, gradient_end_color, gradient) =
            linear_gradient_parts(*start, *end, stops, cmd.opacity);
        (
            PipelineKind::LinearGradient,
            color,
            gradient,
            gradient_end_color,
        )
    }
    Paint::Image(_) => {
        let color = [1.0, 1.0, 1.0, cmd.opacity];
        let (gradient, gradient_end_color) = no_gradient(color);
        (PipelineKind::Image, color, gradient, gradient_end_color)
    }
};
```

Then include the new fields in the `RenderItem` literal:

```rust
gradient,
gradient_end_color,
```

- [x] **Step 6: Add default gradient fields to every other `RenderItem` literal**

For every non-gradient `RenderItem` literal in `src/render/wgpu/item.rs`, add:

```rust
gradient: [0.0, 0.0, 0.0, 0.0],
gradient_end_color: color,
```

When the literal does not have a local `color` variable, use the literal's color value. For example, `missing_resource_item` becomes:

```rust
fn missing_resource_item(
    layer: LayerKind,
    clip_rect: Option<Rect>,
    rect: Rect,
    z_index: i32,
    order: u64,
) -> RenderItem {
    let color = [1.0, 0.0, 1.0, 1.0];
    RenderItem {
        layer,
        clip_rect,
        pipeline: PipelineKind::SolidRect,
        rect,
        color,
        uv_rect: [0.0, 0.0, 1.0, 1.0],
        radius: 0.0,
        z_index,
        order,
        gradient: [0.0, 0.0, 0.0, 0.0],
        gradient_end_color: color,
    }
}
```

- [x] **Step 7: Add default gradient fields outside `item.rs`**

Update every `RenderItem` literal in:

- `src/render/wgpu/text.rs`
- `src/render/wgpu/bitmap_text.rs`

Use this pair for non-gradient items:

```rust
gradient: [0.0, 0.0, 0.0, 0.0],
gradient_end_color: color,
```

For examples with inline color arrays, bind the color first:

```rust
let color = [1.0, 0.0, 0.0, 1.0];
let item = RenderItem {
    layer,
    clip_rect,
    pipeline: PipelineKind::SolidRect,
    rect,
    color,
    uv_rect: [0.0, 0.0, 1.0, 1.0],
    radius: 0.0,
    z_index,
    order,
    gradient: [0.0, 0.0, 0.0, 0.0],
    gradient_end_color: color,
};
```

- [x] **Step 8: Run lowering tests**

Run:

```powershell
cargo test --test render_wgpu_render_items linear_gradient
```

Expected: tests compile and pass.

- [x] **Step 9: Commit lowering implementation**

```powershell
git add src/render/wgpu/pipeline.rs src/render/wgpu/item.rs src/render/wgpu/text.rs src/render/wgpu/bitmap_text.rs tests/render_wgpu_render_items.rs
git commit -m "feat(render): lower linear gradients to render items"
```

---

### Task 5: Add Shader And Instance Layout Tests

**Files:**
- Modify: `tests/render_wgpu_render_items.rs`

- [x] **Step 1: Update the GPU layout test**

Replace `instance_raw_has_stable_gpu_layout` with:

```rust
#[test]
fn instance_raw_has_stable_gpu_layout() {
    assert_eq!(std::mem::size_of::<InstanceRaw>(), 112);
    assert_eq!(std::mem::align_of::<InstanceRaw>(), 4);
    assert!(InstanceRaw::vertex_buffer_layout().array_stride >= 112);
}
```

- [x] **Step 2: Expand shader entry-point coverage**

Replace `shader_source_contains_expected_entry_points` with:

```rust
#[test]
fn shader_source_contains_expected_entry_points() {
    let _ = std::any::type_name::<PipelineCache>();
    assert!(SHADER_SOURCE.contains("fn vs_main"));
    assert!(SHADER_SOURCE.contains("fn fs_main"));
    assert!(SHADER_SOURCE.contains("fn fs_rounded"));
    assert!(SHADER_SOURCE.contains("fn fs_textured"));
    assert!(SHADER_SOURCE.contains("fn fs_linear_gradient"));
}
```

- [x] **Step 3: Run tests and verify they fail**

Run:

```powershell
cargo test --test render_wgpu_render_items instance_raw_has_stable_gpu_layout
cargo test --test render_wgpu_render_items shader_source_contains_expected_entry_points
```

Expected: layout test fails with current size `80`; shader test fails because `fs_linear_gradient` is missing.

- [x] **Step 4: Commit failing shader/layout tests**

```powershell
git add tests/render_wgpu_render_items.rs
git commit -m "test(render): cover gradient shader instance layout"
```

---

### Task 6: Implement Instance Layout And Gradient Shader

**Files:**
- Modify: `src/render/wgpu/pipeline.rs`
- Modify: `src/render/wgpu/shaders.rs`
- Modify: `src/render/wgpu/mod.rs`

- [x] **Step 1: Extend `InstanceRaw`**

In `src/render/wgpu/pipeline.rs`, replace `InstanceRaw` with:

```rust
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct InstanceRaw {
    pub rect: [f32; 4],
    pub color: [f32; 4],
    pub uv_rect: [f32; 4],
    pub viewport: [f32; 4],
    pub flags: [f32; 4],
    pub gradient: [f32; 4],
    pub gradient_end_color: [f32; 4],
}
```

- [x] **Step 2: Extend vertex attributes**

In `InstanceRaw::vertex_buffer_layout`, replace the attributes with:

```rust
const ATTRIBUTES: [wgpu::VertexAttribute; 7] = wgpu::vertex_attr_array![
    0 => Float32x4,
    1 => Float32x4,
    2 => Float32x4,
    3 => Float32x4,
    4 => Float32x4,
    5 => Float32x4,
    6 => Float32x4
];
```

- [x] **Step 3: Wire the gradient pipeline**

Change `pipeline_table` to return eight entries:

```rust
fn pipeline_table() -> [(PipelineKind, &'static str); 8] {
    [
        (PipelineKind::SolidRect, "fs_main"),
        (PipelineKind::Border, "fs_main"),
        (PipelineKind::Path, "fs_main"),
        (PipelineKind::RoundedRect, "fs_rounded"),
        (PipelineKind::LinearGradient, "fs_linear_gradient"),
        (PipelineKind::TextGlyph, "fs_main"),
        (PipelineKind::Image, "fs_textured"),
        (PipelineKind::Svg, "fs_textured"),
    ]
}
```

- [x] **Step 4: Populate instance data**

In `src/render/wgpu/mod.rs`, update the `InstanceRaw` literal in `instances_for_items`:

```rust
InstanceRaw {
    rect: [
        item.rect.origin.x,
        item.rect.origin.y,
        item.rect.size.width,
        item.rect.size.height,
    ],
    color: item.color,
    uv_rect: item.uv_rect,
    viewport,
    flags: [item.radius, 0.0, 0.0, 0.0],
    gradient: item.gradient,
    gradient_end_color: item.gradient_end_color,
}
```

- [x] **Step 5: Extend WGSL vertex output**

In `src/render/wgpu/shaders.rs`, replace the `VertexOut` struct in `SHADER_SOURCE` with:

```wgsl
struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) size: vec2<f32>,
    @location(3) radius: f32,
    @location(4) world_pos: vec2<f32>,
    @location(5) gradient: vec4<f32>,
    @location(6) gradient_end_color: vec4<f32>,
};
```

- [x] **Step 6: Extend WGSL vertex inputs and assignments**

Change `vs_main` signature to include the new attributes:

```wgsl
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @location(0) rect: vec4<f32>,
    @location(1) color: vec4<f32>,
    @location(2) uv_rect: vec4<f32>,
    @location(3) viewport: vec4<f32>,
    @location(4) flags: vec4<f32>,
    @location(5) gradient: vec4<f32>,
    @location(6) gradient_end_color: vec4<f32>
) -> VertexOut {
```

Add these assignments before `return out;`:

```wgsl
out.world_pos = px;
out.gradient = gradient;
out.gradient_end_color = gradient_end_color;
```

- [x] **Step 7: Add gradient fragment shader**

Add this WGSL function after `fs_rounded` and before `fs_textured`:

```wgsl
@fragment
fn fs_linear_gradient(in: VertexOut) -> @location(0) vec4<f32> {
    let start = in.gradient.xy;
    let end = in.gradient.zw;
    let axis = end - start;
    let denom = dot(axis, axis);
    if denom <= 0.0001 {
        return in.color;
    }
    let t = clamp(dot(in.world_pos - start, axis) / denom, 0.0, 1.0);
    return mix(in.color, in.gradient_end_color, t);
}
```

- [x] **Step 8: Update shader docs at the top of `shaders.rs`**

Add this bullet to the entry summary:

```rust
//! - `fs_linear_gradient` - two-stop linear gradient output (LinearGradient).
```

- [x] **Step 9: Run shader/layout tests**

Run:

```powershell
cargo test --test render_wgpu_render_items instance_raw_has_stable_gpu_layout
cargo test --test render_wgpu_render_items shader_source_contains_expected_entry_points
```

Expected: both tests pass.

- [x] **Step 10: Commit shader implementation**

```powershell
git add src/render/wgpu/pipeline.rs src/render/wgpu/shaders.rs src/render/wgpu/mod.rs tests/render_wgpu_render_items.rs
git commit -m "feat(render): add linear gradient shader"
```

---

### Task 7: Add Offscreen Gradient Pixel Tests

**Files:**
- Modify: `tests/render_wgpu_offscreen_render.rs`

- [x] **Step 1: Add direct gradient pixel test**

Add this test after `renders_solid_rect_into_offscreen_texture`:

```rust
#[test]
fn renders_linear_gradient_into_offscreen_texture() {
    let mut renderer = pollster::block_on(WgpuRenderer::new_headless(RendererOptions {
        initial_size: SizeU32::new(64, 16),
        ..RendererOptions::default()
    }))
    .expect("renderer initializes");

    let mut display_list = DisplayList::default();
    display_list.push(PaintCommand::DrawRect(RectCmd {
        rect: Rect::new(Point::new(0.0, 0.0), Size::new(64.0, 16.0)),
        paint: Paint::LinearGradient {
            start: Point::new(0.0, 0.0),
            end: Point::new(64.0, 0.0),
            stops: vec![
                (0.0, Color::rgb(255, 0, 0)),
                (1.0, Color::rgb(0, 0, 255)),
            ],
        },
        radius: 0.0,
        opacity: 1.0,
        z_index: 0,
    }));

    let target = OffscreenTarget::new(renderer.context(), SizeU32::new(64, 16));
    let stats = renderer
        .render_to_target(&display_list, &ResourceStore::default(), target.view())
        .expect("offscreen render succeeds");
    let pixels = pollster::block_on(target.read_rgba8(renderer.context())).expect("readback works");

    let left = sample_pixel(&pixels, 64, 2, 8);
    let middle = sample_pixel(&pixels, 64, 32, 8);
    let right = sample_pixel(&pixels, 64, 62, 8);

    assert_eq!(stats.command_count, 1);
    assert_eq!(stats.batch_count, 1);
    assert!(left[0] > left[2], "left side should be red-dominant: {left:?}");
    assert!(
        (middle[0] as i16 - middle[2] as i16).abs() <= 20,
        "middle should blend red and blue: {middle:?}"
    );
    assert!(right[2] > right[0], "right side should be blue-dominant: {right:?}");
}
```

- [x] **Step 2: Add clipping test**

Add this test after the direct gradient test:

```rust
#[test]
fn clipped_linear_gradient_does_not_render_outside_clip() {
    let mut renderer = pollster::block_on(WgpuRenderer::new_headless(RendererOptions {
        initial_size: SizeU32::new(32, 16),
        ..RendererOptions::default()
    }))
    .expect("renderer initializes");

    let mut display_list = DisplayList::default();
    display_list.push(PaintCommand::PushClip(ClipSpec::rect(Rect::new(
        Point::new(0.0, 0.0),
        Size::new(16.0, 16.0),
    ))));
    display_list.push(PaintCommand::DrawRect(RectCmd {
        rect: Rect::new(Point::new(0.0, 0.0), Size::new(32.0, 16.0)),
        paint: Paint::LinearGradient {
            start: Point::new(0.0, 0.0),
            end: Point::new(32.0, 0.0),
            stops: vec![
                (0.0, Color::rgb(255, 0, 0)),
                (1.0, Color::rgb(0, 0, 255)),
            ],
        },
        radius: 0.0,
        opacity: 1.0,
        z_index: 0,
    }));
    display_list.push(PaintCommand::PopClip);

    let target = OffscreenTarget::new(renderer.context(), SizeU32::new(32, 16));
    renderer
        .render_to_target(&display_list, &ResourceStore::default(), target.view())
        .expect("offscreen render succeeds");
    let pixels = pollster::block_on(target.read_rgba8(renderer.context())).expect("readback works");

    assert!(sample_pixel(&pixels, 32, 8, 8)[3] > 0);
    assert_eq!(sample_pixel(&pixels, 32, 24, 8), [0, 0, 0, 0]);
}
```

- [x] **Step 3: Add z-order test**

Add this test after the clipping test:

```rust
#[test]
fn linear_gradient_respects_z_order() {
    let mut renderer = pollster::block_on(WgpuRenderer::new_headless(RendererOptions {
        initial_size: SizeU32::new(32, 16),
        ..RendererOptions::default()
    }))
    .expect("renderer initializes");

    let mut display_list = DisplayList::default();
    display_list.push(PaintCommand::DrawRect(RectCmd {
        rect: Rect::new(Point::new(0.0, 0.0), Size::new(32.0, 16.0)),
        paint: Paint::LinearGradient {
            start: Point::new(0.0, 0.0),
            end: Point::new(32.0, 0.0),
            stops: vec![
                (0.0, Color::rgb(255, 0, 0)),
                (1.0, Color::rgb(0, 0, 255)),
            ],
        },
        radius: 0.0,
        opacity: 1.0,
        z_index: 0,
    }));
    display_list.push(PaintCommand::DrawRect(RectCmd {
        rect: Rect::new(Point::new(8.0, 4.0), Size::new(16.0, 8.0)),
        paint: Paint::Solid(Color::rgb(0, 255, 0)),
        radius: 0.0,
        opacity: 1.0,
        z_index: 1,
    }));

    let target = OffscreenTarget::new(renderer.context(), SizeU32::new(32, 16));
    renderer
        .render_to_target(&display_list, &ResourceStore::default(), target.view())
        .expect("offscreen render succeeds");
    let pixels = pollster::block_on(target.read_rgba8(renderer.context())).expect("readback works");

    assert_eq!(sample_pixel(&pixels, 32, 16, 8), [0, 255, 0, 255]);
}
```

- [x] **Step 4: Run offscreen tests**

Run:

```powershell
cargo test --test render_wgpu_offscreen_render linear_gradient
```

Expected: all three tests pass.

- [x] **Step 5: Commit offscreen tests**

```powershell
git add tests/render_wgpu_offscreen_render.rs
git commit -m "test(render): verify linear gradient pixels"
```

---

### Task 8: Update Public WGPU Documentation

**Files:**
- Modify: `docs/public-api.md`

- [x] **Step 1: Update the `PipelineKind` example**

In the WGPU renderer API section, change:

```rust
let pipeline_kind = PipelineKind::SolidRect;
```

to:

```rust
let pipeline_kind = PipelineKind::LinearGradient;
```

- [x] **Step 2: Update `RenderItem` examples**

Every `RenderItem` literal in `docs/public-api.md` must include:

```rust
gradient: [0.0, 0.0, 0.0, 0.0],
gradient_end_color: color,
```

Use a local `color` binding before the example item:

```rust
let color = [1.0, 0.0, 0.0, 1.0];
let item = RenderItem {
    layer: LayerKind::Document,
    clip_rect: None,
    pipeline: PipelineKind::SolidRect,
    rect: rgui::Rect::new(rgui::Point::new(0.0, 0.0), rgui::Size::new(10.0, 10.0)),
    color,
    uv_rect: [0.0, 0.0, 1.0, 1.0],
    radius: 0.0,
    z_index: 0,
    order: 0,
    gradient: [0.0, 0.0, 0.0, 0.0],
    gradient_end_color: color,
};
```

- [x] **Step 3: Add a migration note**

Add this paragraph near the WGPU renderer API section:

```markdown
Linear gradient fidelity adds two public fields to `RenderItem` and
`InstanceRaw`: `gradient` and `gradient_end_color`. Existing custom
low-level integrations that construct render items should set
`gradient` to `[0.0, 0.0, 0.0, 0.0]` and `gradient_end_color` to the
same value as `color` for non-gradient items.
```

- [x] **Step 4: Run docs check**

Run:

```powershell
cargo test --doc
```

Expected: doctests pass. If pre-existing unrelated doctest failures appear, capture the failing names and continue with the targeted compile/test commands in Task 9.

- [x] **Step 5: Commit docs**

```powershell
git add docs/public-api.md
git commit -m "docs(render): document linear gradient instance fields"
```

---

### Task 9: Run Full Phase Verification

**Files:**
- No source edits expected.

- [x] **Step 1: Run render-item tests**

Run:

```powershell
cargo test --test render_wgpu_render_items
```

Expected: test binary passes.

- [x] **Step 2: Run offscreen render tests**

Run:

```powershell
cargo test --test render_wgpu_offscreen_render
```

Expected: test binary passes.

- [x] **Step 3: Run visual goldens**

Run:

```powershell
cargo test --test visual_goldens -j1
```

Expected: test binary passes. If a golden changes only because a test scene starts using gradients in a later change, update goldens with `RGUI_UPDATE_GOLDENS=1` in a separate intentional commit. This Phase 1 plan does not add gradient widgets to visual goldens, so no golden image change is expected.

- [x] **Step 4: Run optional Vulkan golden mirror when the backend is available**

Run:

```powershell
cargo test --features vulkan-goldens --test visual_goldens_vulkan -j1
```

Expected: pass on machines with a working Vulkan backend. If the machine lacks Vulkan support, record the backend error in the final implementation notes.

- [x] **Step 5: Inspect final diff**

Run:

```powershell
git diff --stat
git diff --check
```

Expected: `git diff --check` prints no output.

- [x] **Step 6: Commit verification note if docs changed during fixes**

If Task 9 required small docs-only corrections, commit them:

```powershell
git add docs/public-api.md
git commit -m "docs(render): tighten gradient fidelity docs"
```

If no files changed during Task 9, do not create an empty commit.

---

## Final Acceptance Criteria

- `Paint::LinearGradient` lowers to `PipelineKind::LinearGradient`.
- The first and last gradient stops are preserved as start and end colors.
- More-than-two-stop gradients use the first and last stops for this phase.
- WGPU output blends between start and end colors based on fragment position along the gradient vector.
- Gradients respect opacity, clipping, z-order, and layer order.
- Non-gradient rendering still passes existing render-item, offscreen, and visual golden tests.
- Public docs explain the new `RenderItem` and `InstanceRaw` fields.

## Out-Of-Scope Follow-Up Plans

- Rounded border fidelity.
- Soft shadow falloff.
- Real polyline path strokes.
- Image/SVG resource upload fidelity.
