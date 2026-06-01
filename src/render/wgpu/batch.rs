use crate::core::{LayerKind, Rect};

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
