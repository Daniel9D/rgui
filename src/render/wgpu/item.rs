use crate::core::{
    AtlasEntryKind, BorderCmd, ClipSpec, Color, DisplayList, LayerKind, Paint, PaintCommand,
    PathCmd, Point, Rect, ResourceStore, ShadowCmd, Size, effective_clip,
};

use super::{GpuAtlas, PipelineKind, RendererError, RendererResult};

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
}

pub const MAX_RENDER_ITEMS_PER_FRAME: usize = 100_000;

pub fn build_render_items(
    display_list: &DisplayList,
    resources: &ResourceStore,
    atlas: &mut GpuAtlas,
) -> RendererResult<Vec<RenderItem>> {
    let _ = resources;
    display_list
        .validate()
        .map_err(RendererError::InvalidDisplayList)?;

    let mut items = Vec::new();
    let mut layer_stack = vec![LayerKind::Document];
    let mut clip_stack: Vec<ClipSpec> = Vec::new();
    for (order, command) in display_list.commands().iter().enumerate() {
        let layer = *layer_stack.last().unwrap_or(&LayerKind::Document);
        let clip_rect = effective_clip_rect(&clip_stack);
        match command {
            PaintCommand::PushLayer(spec) => layer_stack.push(spec.kind),
            PaintCommand::PopLayer => {
                layer_stack.pop();
            }
            PaintCommand::PushClip(spec) => clip_stack.push(spec.clone()),
            PaintCommand::PopClip => {
                clip_stack.pop();
            }
            PaintCommand::DrawRect(cmd) => push_rect(&mut items, cmd, order, layer, clip_rect)?,
            PaintCommand::DrawBorder(cmd) => push_border(&mut items, cmd, order, layer, clip_rect)?,
            PaintCommand::DrawPath(cmd) => push_path(&mut items, cmd, order, layer, clip_rect)?,
            PaintCommand::DrawShadow(cmd) => push_shadow(&mut items, cmd, order, layer, clip_rect)?,
            PaintCommand::DrawImage(cmd) => {
                let kind = AtlasEntryKind::Image(cmd.id);
                let item = if let Some(uv_rect) = atlas.uv_for(&kind) {
                    RenderItem {
                        layer,
                        clip_rect,
                        pipeline: PipelineKind::Image,
                        rect: cmd.rect,
                        color: [1.0, 1.0, 1.0, cmd.opacity],
                        uv_rect,
                        radius: 0.0,
                        z_index: cmd.z_index,
                        order: paint_order(order, 0),
                    }
                } else {
                    missing_resource_item(
                        layer,
                        clip_rect,
                        cmd.rect,
                        cmd.z_index,
                        paint_order(order, 0),
                    )
                };
                push_item(&mut items, item)?;
            }
            PaintCommand::DrawSvg(cmd) => {
                let kind = AtlasEntryKind::Svg(cmd.id);
                let item = if let Some(uv_rect) = atlas.uv_for(&kind) {
                    RenderItem {
                        layer,
                        clip_rect,
                        pipeline: PipelineKind::Svg,
                        rect: cmd.rect,
                        color: [1.0, 1.0, 1.0, cmd.opacity],
                        uv_rect,
                        radius: 0.0,
                        z_index: cmd.z_index,
                        order: paint_order(order, 0),
                    }
                } else {
                    missing_resource_item(
                        layer,
                        clip_rect,
                        cmd.rect,
                        cmd.z_index,
                        paint_order(order, 0),
                    )
                };
                push_item(&mut items, item)?;
            }
            PaintCommand::DrawText(cmd) => {
                push_text_items(&mut items, cmd, order, layer, clip_rect, atlas)?
            }
        }
    }
    debug_assert!(items.len() <= MAX_RENDER_ITEMS_PER_FRAME);
    items.sort_by_key(|item| (layer_order(item.layer), item.z_index, item.order));
    Ok(items)
}

pub(crate) fn paint_order(command_order: usize, sub_order: usize) -> u64 {
    ((command_order as u64) << 32) | sub_order as u64
}

fn layer_order(layer: LayerKind) -> i32 {
    match layer {
        LayerKind::Document => 0,
        LayerKind::Floating => 1,
        LayerKind::Popover => 2,
        LayerKind::Tooltip => 3,
        LayerKind::ContextMenu => 4,
        LayerKind::Modal => 5,
        LayerKind::Debug => 6,
    }
}

fn push_rect(
    items: &mut Vec<RenderItem>,
    cmd: &crate::core::RectCmd,
    order: usize,
    layer: LayerKind,
    clip_rect: Option<Rect>,
) -> RendererResult<()> {
    let base_pipeline = if cmd.radius > 0.5 {
        PipelineKind::RoundedRect
    } else {
        PipelineKind::SolidRect
    };
    let (pipeline, color) = match cmd.paint {
        Paint::Solid(color) => (base_pipeline, color_to_linear(color, cmd.opacity)),
        Paint::LinearGradient { .. } => (base_pipeline, [1.0, 1.0, 1.0, cmd.opacity]),
        Paint::Image(_) => (PipelineKind::Image, [1.0, 1.0, 1.0, cmd.opacity]),
    };
    push_item(
        items,
        RenderItem {
            layer,
            clip_rect,
            pipeline,
            rect: cmd.rect,
            color,
            uv_rect: [0.0, 0.0, 1.0, 1.0],
            radius: cmd.radius,
            z_index: cmd.z_index,
            order: paint_order(order, 0),
        },
    )
}

fn missing_resource_item(
    layer: LayerKind,
    clip_rect: Option<Rect>,
    rect: Rect,
    z_index: i32,
    order: u64,
) -> RenderItem {
    RenderItem {
        layer,
        clip_rect,
        pipeline: PipelineKind::SolidRect,
        rect,
        color: [1.0, 0.0, 1.0, 1.0],
        uv_rect: [0.0, 0.0, 1.0, 1.0],
        radius: 0.0,
        z_index,
        order,
    }
}

fn push_border(
    items: &mut Vec<RenderItem>,
    cmd: &BorderCmd,
    order: usize,
    layer: LayerKind,
    clip_rect: Option<Rect>,
) -> RendererResult<()> {
    let color = color_to_linear(cmd.color, 1.0);
    let x = cmd.rect.origin.x;
    let y = cmd.rect.origin.y;
    let w = cmd.rect.size.width;
    let h = cmd.rect.size.height;
    let b = cmd.width.max(0.0);
    let rects = [
        Rect::new(Point::new(x, y), Size::new(w, b)),
        Rect::new(Point::new(x, y + h - b), Size::new(w, b)),
        Rect::new(Point::new(x, y), Size::new(b, h)),
        Rect::new(Point::new(x + w - b, y), Size::new(b, h)),
    ];
    for (offset, rect) in rects.into_iter().enumerate() {
        push_item(
            items,
            RenderItem {
                layer,
                clip_rect,
                pipeline: PipelineKind::Border,
                rect,
                color,
                uv_rect: [0.0, 0.0, 1.0, 1.0],
                radius: cmd.radius,
                z_index: cmd.z_index,
                order: paint_order(order, offset),
            },
        )?;
    }
    Ok(())
}

fn push_path(
    items: &mut Vec<RenderItem>,
    cmd: &PathCmd,
    order: usize,
    layer: LayerKind,
    clip_rect: Option<Rect>,
) -> RendererResult<()> {
    for (offset, pair) in cmd.points.windows(2).enumerate() {
        let a = pair[0];
        let b = pair[1];
        let min_x = a.x.min(b.x);
        let min_y = a.y.min(b.y);
        let width = (a.x - b.x).abs().max(cmd.width);
        let height = (a.y - b.y).abs().max(cmd.width);
        push_item(
            items,
            RenderItem {
                layer,
                clip_rect,
                pipeline: PipelineKind::Path,
                rect: Rect::new(Point::new(min_x, min_y), Size::new(width, height)),
                color: color_to_linear(cmd.color, 1.0),
                uv_rect: [0.0, 0.0, 1.0, 1.0],
                radius: 0.0,
                z_index: cmd.z_index,
                order: paint_order(order, offset),
            },
        )?;
    }
    Ok(())
}

fn push_shadow(
    items: &mut Vec<RenderItem>,
    cmd: &ShadowCmd,
    order: usize,
    layer: LayerKind,
    clip_rect: Option<Rect>,
) -> RendererResult<()> {
    let expand = cmd.blur_radius + cmd.offset.x.abs().max(cmd.offset.y.abs());
    push_item(
        items,
        RenderItem {
            layer,
            clip_rect,
            pipeline: PipelineKind::SolidRect,
            rect: Rect::new(
                Point::new(cmd.rect.origin.x - expand, cmd.rect.origin.y - expand),
                Size::new(
                    cmd.rect.size.width + expand * 2.0,
                    cmd.rect.size.height + expand * 2.0,
                ),
            ),
            color: color_to_linear(cmd.color, 0.35),
            uv_rect: [0.0, 0.0, 1.0, 1.0],
            radius: 0.0,
            z_index: cmd.z_index,
            order: paint_order(order, 0),
        },
    )
}

pub(crate) fn push_text_items(
    items: &mut Vec<RenderItem>,
    cmd: &crate::core::TextCmd,
    order: usize,
    layer: LayerKind,
    clip_rect: Option<Rect>,
    atlas: &mut GpuAtlas,
) -> RendererResult<()> {
    super::text::lower_text_glyph_atlas(items, cmd, order, layer, clip_rect, atlas).map(|_| ())
}

pub(crate) fn push_item(items: &mut Vec<RenderItem>, item: RenderItem) -> RendererResult<()> {
    if is_zero_size(item.rect) {
        return Ok(());
    }
    if items.len() >= MAX_RENDER_ITEMS_PER_FRAME {
        return Err(RendererError::InvalidDisplayList(format!(
            "render item limit exceeded: {} >= {}",
            items.len(),
            MAX_RENDER_ITEMS_PER_FRAME
        )));
    }
    items.push(item);
    Ok(())
}

fn is_zero_size(rect: Rect) -> bool {
    rect.size.width == 0.0 || rect.size.height == 0.0
}

fn effective_clip_rect(clip_stack: &[ClipSpec]) -> Option<Rect> {
    let first = clip_stack.first()?.rect;
    let rest: Vec<Rect> = clip_stack.iter().skip(1).map(|clip| clip.rect).collect();
    effective_clip(&rest, first)
}

fn color_to_linear(color: Color, opacity: f32) -> [f32; 4] {
    [
        color.r as f32 / 255.0,
        color.g as f32 / 255.0,
        color.b as f32 / 255.0,
        color.a as f32 / 255.0 * opacity,
    ]
}
