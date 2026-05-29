use super::batch::RenderBatch;
use super::item::RenderItem;

pub fn format_render_items(items: &[RenderItem]) -> String {
    let mut dump = String::from("=== RENDER ITEMS ===\n");
    for (index, item) in items.iter().enumerate() {
        dump.push_str(&format!(
            "[{index:03}] layer={:?} clip={:?} pipeline={:?} rect={:?} z={} order={}\n",
            item.layer, item.clip_rect, item.pipeline, item.rect, item.z_index, item.order
        ));
    }
    dump
}

pub fn format_render_batches(batches: &[RenderBatch]) -> String {
    let mut dump = String::from("=== RENDER BATCHES ===\n");
    for (index, batch) in batches.iter().enumerate() {
        dump.push_str(&format!(
            "[{index:03}] key={:?} first_item={} command_count={}\n",
            batch.key, batch.first_item, batch.command_count
        ));
    }
    dump
}
