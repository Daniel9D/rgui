//! Centralized read of render-side debug environment variables.
//!
//! The render path supports a handful of opt-in debug dumps behind env vars.
//! All of them are presence-based (any non-empty value enables the dump) and
//! are read on every frame; the cost of `var_os` is negligible compared to a
//! render frame, so we deliberately avoid caching.
//!
//! Current flags:
//! - `RGUI_DEBUG_RENDER_ITEMS` — dump the lowered `RenderItem` list.
//! - `RGUI_DEBUG_BATCHES` — dump the GPU command batches built from the items.
//! - `RGUI_DEBUG_TEXT` — dump glyphon text layout stats.
//!
//! Add new flags here (and only here) so the list of knobs is auditable from
//! one place.

/// `true` when the `RGUI_DEBUG_RENDER_ITEMS` env var is set.
pub fn dump_render_items() -> bool {
    flag("RGUI_DEBUG_RENDER_ITEMS")
}

/// `true` when the `RGUI_DEBUG_BATCHES` env var is set.
pub fn dump_batches() -> bool {
    flag("RGUI_DEBUG_BATCHES")
}

/// `true` when the `RGUI_DEBUG_TEXT` env var is set.
pub fn dump_text() -> bool {
    flag("RGUI_DEBUG_TEXT")
}

fn flag(name: &str) -> bool {
    std::env::var_os(name).is_some()
}
