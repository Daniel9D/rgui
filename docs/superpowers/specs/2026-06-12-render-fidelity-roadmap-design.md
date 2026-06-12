# Render Fidelity Roadmap Design

Date: 2026-06-12

## Summary

`rsgui` already has the right high-level rendering shape: the runtime emits a retained `DisplayList`, the WGPU backend lowers commands into sorted render items, batches compatible items, and draws with deterministic clipping and z-order. The next rendering-fidelity work should keep that architecture and make the backend honor more of the visual meaning already present in `PaintCommand`.

This roadmap prioritizes visible correctness over a renderer rewrite. Each phase improves one primitive family, adds focused WGPU lowering/shader support, and pins behavior with offscreen pixel tests plus visual goldens when real widgets are affected.

## Current State

Strengths:

- `DisplayList` validation catches stack, geometry, text, path, opacity, and radius issues before lowering.
- `build_render_items` preserves layer, clip, z-index, and command order.
- `build_batches_from_items` batches adjacent items by layer, clip, pipeline, and z-index.
- WGPU rendering covers solid rects, rounded rects, borders as edge quads, clipped rects, atlas images, and glyphon text.
- The test suite includes render-item tests, offscreen pixel tests, glyphon clipping tests, visual goldens, and an optional Vulkan golden mirror.

Known fidelity gaps:

- `Paint::LinearGradient` currently lowers as a white rect instead of a gradient.
- `DrawBorder` lowers to four square rects and does not preserve rounded border geometry.
- `DrawShadow` lowers to a single expanded translucent rect rather than a soft shadow.
- `DrawPath` lowers each segment to a bounding box, so diagonal and complex strokes are visually approximate.
- SVG/image resource handling is still placeholder-heavy; missing resources render magenta, and `ResourceStore` is not the real lookup source in WGPU lowering.
- Some public documentation examples appear stale relative to current APIs, especially around atlas locking and `RenderStats` fields.

## Goals

- Make the WGPU backend faithfully render existing `PaintCommand` semantics where those semantics are already expressible.
- Preserve the current retained-mode pipeline and sorted `DisplayList` contract.
- Keep API churn low, especially for public WGPU types.
- Improve fidelity incrementally so regressions are easy to isolate.
- Add verification per primitive before broadening scope.

## Non-Goals

- Do not replace `wgpu`, `taffy`, or `glyphon`.
- Do not rewrite the runtime paint system.
- Do not introduce a new immediate-mode renderer.
- Do not add new runtime dependencies for the hot path.
- Do not solve full SVG rasterization or arbitrary filled vector paths in the first phase.

## Architecture

The pipeline remains:

```text
Element tree -> runtime paint -> DisplayList -> WGPU lowering -> batches -> draw
```

The fidelity work happens mostly inside `src/render/wgpu`:

- `item.rs` remains the semantic lowering layer.
- `pipeline.rs` remains the pipeline registry and keeps `pipeline_table()` as the single source of truth.
- `shaders.rs` gains focused WGSL entry points for new primitive families.
- `batch.rs` keeps batching purely semantic-preserving: values that differ per item must live in instance data, or the batch key must include them.
- `runtime::paint` changes only when a visual cannot be expressed by existing commands.

The first milestone is real linear gradients because `Paint::LinearGradient` already exists in the public command model and currently renders incorrectly.

## Components

### Render Items

`RenderItem` should gain only the per-item data needed by each phase. Because `RenderItem`, `PipelineKind`, and `InstanceRaw` are public, changes need a compatibility story:

- Prefer additive fields or sibling internal structs when possible.
- If a public struct must change, add a CHANGELOG note and migration docs in the same release.
- Keep tests that assert GPU layout updated intentionally.

### Pipelines And Shaders

Add specific `PipelineKind` variants rather than overloading `SolidRect`:

- `LinearGradient`
- `RoundedBorder`
- `SoftShadow`
- `StrokePath`

Each variant maps to a WGSL fragment entry in `pipeline_table()`.

### Lowering

`item.rs` should define the exact GPU meaning for each command:

- `push_rect` handles solid, image-backed, and gradient paint.
- `push_border` evolves from edge quads to rounded-border rendering.
- `push_shadow` evolves from expanded rects to soft falloff items.
- `push_path` evolves from bounding boxes to true strokes.

### Runtime Paint

Runtime paint should stay stable during early phases. It already emits enough data for gradients, borders, shadows, and basic paths. New paint commands should wait until the backend cannot preserve fidelity with existing commands.

### Tests

Each phase needs:

- render-item lowering tests for pipeline choice and ordering;
- offscreen pixel tests that sample representative pixels;
- visual goldens only when actual widget output changes;
- docs updates when public APIs or behavior change.

## Data Flow And Validation

Every phase keeps the deterministic flow:

```text
validate DisplayList
lower commands into RenderItem values
sort by layer, z_index, and order
batch adjacent compatible items
draw batches with the active clip/scissor
draw glyphon text with text bounds
```

Validation should expand only as needed:

- `LinearGradient`: finite start/end points, finite stop positions, at least two effective stops.
- `Border`: finite non-negative width and radius, skip zero-width borders.
- `Shadow`: finite blur radius and offset, skip zero-alpha or zero-size shadows.
- `Path`: finite points and non-negative width; later distinguish polyline strokes from filled paths if fills are added.

Batching must not alter paint semantics. If a visual parameter differs per item, it belongs in instance data. If it is pipeline-global or bind-group-global, it belongs in the batch key.

## Phases

### Phase 1: Linear Gradients

Implement real rendering for `Paint::LinearGradient`.

Initial scope:

- Support two effective stops from the first and last gradient stops.
- Normalize more-than-two stops to first/last until a multi-stop ramp design is justified.
- Add `PipelineKind::LinearGradient`.
- Extend GPU instance data or introduce an internal gradient instance path.
- Add WGSL that projects fragment position along the gradient vector.
- Add offscreen tests that sample start, middle, and end pixels.

Acceptance:

- A `DrawRect` with `Paint::LinearGradient` no longer renders white.
- Gradient output respects rect, opacity, clipping, z-index, and layer order.
- Existing solid rect, image, text, and golden tests still pass.

### Phase 2: Rounded Borders

Improve `DrawBorder` fidelity.

Initial scope:

- Skip zero-width borders.
- Preserve rounded corners for borders.
- Prefer a rounded-border SDF shader or paired rounded-rect approach over four square edge rects.

Acceptance:

- Rounded border corners remain transparent outside the radius.
- Border width is visually stable across small and medium radii.
- Existing square-border tests remain valid or are intentionally updated.

### Phase 3: Soft Shadows

Replace expanded solid shadow rectangles with soft falloff.

Initial scope:

- Add a `SoftShadow` pipeline.
- Render shadow alpha based on distance from a rounded rect and blur radius.
- Keep `ShadowCmd` as the public command input.

Acceptance:

- Shadow pixels show alpha falloff.
- Pixels outside the blur extent remain transparent.
- Shadows respect clipping and z-order.

### Phase 4: Real Path Strokes

Replace path bounding boxes with visible strokes.

Initial scope:

- Start with polyline strokes.
- Support horizontal, vertical, and diagonal segments.
- Choose either CPU-generated segment quads or a shader-driven segment primitive after a small prototype.

Acceptance:

- Diagonal paths render as diagonal strokes, not filled bounding boxes.
- Stroke width is respected.
- Segment ordering remains stable relative to other commands.

### Phase 5: Resource Fidelity

Tighten image and SVG behavior.

Initial scope:

- Clarify whether `ResourceStore` drives WGPU lookup or is debug/runtime metadata only.
- Document host-driven atlas uploads.
- Reduce normal-flow magenta placeholders by improving upload examples and diagnostics.
- Plan SVG rasterization or atlas upload explicitly.

Acceptance:

- Public docs match actual WGPU resource behavior.
- Missing resources remain visible during development, but normal examples upload resources before rendering.

## Verification

Run these after each implementation phase:

```powershell
cargo test --test render_wgpu_render_items
cargo test --test render_wgpu_offscreen_render
cargo test --test visual_goldens -j1
```

Optional cross-backend verification:

```powershell
cargo test --features vulkan-goldens --test visual_goldens_vulkan -j1
```

When public docs or doctests change:

```powershell
cargo test --doc
cargo doc --document-private-items
```

## Risks

- `InstanceRaw` is public and layout-tested, so adding shader inputs may be a public API change.
- Multi-stop gradients may require a texture ramp, storage buffer, or fixed stop array; the first phase intentionally avoids that.
- Shader fidelity can increase batch fragmentation if parameters are not represented as instance data.
- Visual goldens may need intentional updates, so each phase should keep output changes small.
- Surface behavior and atlas lifetime are robustness concerns, but they should not block primitive-fidelity work unless a phase directly touches them.

## Open Decisions For Implementation Planning

- Whether Phase 1 extends `InstanceRaw` or introduces an internal gradient-specific instance buffer.
- Whether rounded borders use a single SDF shader or draw an outer rounded rect with an inner cutout approximation.
- Whether path strokes start with CPU tessellation or shader-side segment distance.
- Whether `ResourceStore` should become authoritative for WGPU atlas entries or remain separate metadata.

## Recommendation

Start with Phase 1: Linear Gradients.

It is the best first fidelity milestone because the public command already exists, the current output is visibly wrong, the implementation can stay localized to WGPU lowering/shaders, and the tests can be precise without changing widget APIs.
