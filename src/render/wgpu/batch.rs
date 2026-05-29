use crate::core::{LayerKind, PaintCommand, Rect};

use super::{PipelineKind, RenderItem};

#[derive(Clone, Debug, PartialEq)]
pub struct BatchKey {
    pub layer: LayerKind,
    pub clip_rect: Option<Rect>,
    pub pipeline: PipelineKind,
    pub z_index: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderBatch {
    pub key: BatchKey,
    pub first_item: usize,
    pub command_count: usize,
}

pub fn build_batches_from_items(items: &[RenderItem]) -> Vec<RenderBatch> {
    let mut batches: Vec<RenderBatch> = Vec::new();
    for (index, item) in items.iter().enumerate() {
        let key = BatchKey {
            layer: item.layer,
            clip_rect: item.clip_rect,
            pipeline: item.pipeline,
            z_index: item.z_index,
        };
        if let Some(last) = batches.last_mut() {
            if last.key == key {
                last.command_count += 1;
                continue;
            }
        }
        batches.push(RenderBatch {
            key,
            first_item: index,
            command_count: 1,
        });
    }
    batches
}

#[deprecated(
    note = "Use build_render_items followed by build_batches_from_items so layer and clip state are preserved."
)]
pub fn build_batches(commands: &[PaintCommand]) -> Vec<RenderBatch> {
    commands
        .iter()
        .filter_map(|command| match command {
            PaintCommand::DrawRect(cmd) => Some((PipelineKind::SolidRect, cmd.z_index)),
            PaintCommand::DrawBorder(cmd) => Some((PipelineKind::Border, cmd.z_index)),
            PaintCommand::DrawText(cmd) => Some((PipelineKind::TextGlyph, cmd.z_index)),
            PaintCommand::DrawImage(cmd) => Some((PipelineKind::Image, cmd.z_index)),
            PaintCommand::DrawSvg(cmd) => Some((PipelineKind::Svg, cmd.z_index)),
            PaintCommand::DrawPath(cmd) => Some((PipelineKind::Path, cmd.z_index)),
            PaintCommand::DrawShadow(cmd) => Some((PipelineKind::SolidRect, cmd.z_index)),
            _ => None,
        })
        .enumerate()
        .map(|(first_item, (pipeline, z_index))| RenderBatch {
            key: BatchKey {
                layer: LayerKind::Document,
                clip_rect: None,
                pipeline,
                z_index,
            },
            first_item,
            command_count: 1,
        })
        .collect()
}
