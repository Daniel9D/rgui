//! Centralized render-side magic numbers.
//!
//! These constants are the single source of truth for the render/wgpu pipeline.
//! CPU-side mirrors in `runtime/paint.rs` use the same values; if you change
//! a value here, update the comment in `paint.rs` and any cross-references.
//!
//! The constants are grouped by concern:
//! - Atlas: GPU texture atlas sizing
//! - Pipeline: limits, growth policy
//! - Geometry: thresholds and ratios used in clip / shadow / SDF math
//! - Text: ratios and strides used by text lowering
//! - Surface: wgpu surface configuration

// Atlas ---------------------------------------------------------------------

/// Minimum GPU texture atlas dimensions. Smaller requests are clamped up.
pub const ATLAS_MIN_SIZE: u32 = 1024;

/// Bytes per pixel for `Rgba8UnormSrgb` (the only format currently used by
/// the atlas and offscreen target). Used in `bytes_per_row` calculations.
pub const ATLAS_BYTES_PER_PIXEL: u32 = 4;

// Pipeline -----------------------------------------------------------------

/// Hard upper bound on render items emitted per frame; exceeding this raises
/// `RendererError::InvalidDisplayList`. Sized for moderately complex GUIs.
pub const MAX_RENDER_ITEMS_PER_FRAME: usize = 100_000;

/// Initial instance buffer capacity; the first non-empty frame will resize
/// to the next power of two of the actual item count.
pub const INITIAL_INSTANCE_CAPACITY: usize = 1;

/// Target maximum number of frames in flight for surface presentation.
pub const DESIRED_MAXIMUM_FRAME_LATENCY: u32 = 2;

// Geometry -----------------------------------------------------------------

/// Radius (px) above which a rect is rendered via the rounded-rect SDF
/// pipeline instead of the solid-rect pipeline. Mirrored in
/// `shaders.rs::fs_rounded` as the cutoff.
pub const ROUNDED_RECT_RADIUS_THRESHOLD: f32 = 0.5;

/// Default opacity applied to a `DrawShadow` rect when the GPU lowers it.
/// This is a render-side policy value because the CPU `paint.rs` already
/// writes the shadow as a full alpha into the display list.
pub const SHADOW_OPACITY: f32 = 0.35;

// Text ---------------------------------------------------------------------

/// Ratio of ascender size to font size used to compute the baseline offset
/// when lowering text commands. Mirrors `runtime/paint.rs:1494`.
pub const TEXT_BASELINE_RATIO: f32 = 0.8;

/// Average advance-width-to-font-size ratio for ASCII glyphs at normal
/// weight. Used by both the CPU-side `runtime/text_metrics.rs`
/// `measure_text` heuristic and the GPU-side
/// `text_engine/system.rs` `measure_estimated` fast path. Bug fix
/// 5.6: the value was duplicated in two files; both now read the
/// constant. The real shape cache is the source of truth for layout.
pub const TEXT_WIDTH_HEURISTIC: f32 = 0.58;

/// Average advance-width-to-font-size ratio for bold ASCII glyphs.
/// Mirrors `text_engine/system.rs` and used wherever the bold path
/// is special-cased.
pub const TEXT_WIDTH_HEURISTIC_BOLD: f32 = 0.64;

/// Default line-height ratio when a text command does not specify one.
/// Mirrors `runtime/paint.rs:1489` and `glyphon_text.rs:169`.
pub const TEXT_LINE_HEIGHT_RATIO: f32 = 1.2;

/// Glyph atlas byte budget per visible glyph when computing the sub-order
/// inside `paint_order`. Picked so that up to 64 sub-items can be packed
/// into the lower 32 bits of the `order: u64`.
pub const GLYPH_SUB_ORDER_STRIDE: usize = 64;

/// Per-row sub-order stride for the bitmap glyph path; combines with
/// `GLYPH_SUB_ORDER_STRIDE` to give a stable, in-order paint order.
pub const GLYPH_ROW_STRIDE: usize = 8;

/// Multiplier used when allocating the glyphon shaping buffer height; sized
/// to comfortably hold the largest plausible wrapping layout.
pub const SHAPING_BUFFER_LINE_MULTIPLIER: f32 = 50.0;
