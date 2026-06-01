//! Color conversion helpers for the render pipeline.
//!
//! Currently we only have a single `Color` representation in `core::Color`
//! (sRGB-encoded 8-bit channels). All render-side code that lowers colors
//! to `[f32; 4]` uses this function to keep the math consistent.

use crate::core::Color;

/// Convert an sRGB-encoded 8-bit color into a linear `[f32; 4]` suitable for
/// the GPU shaders, applying the given `opacity` multiplier to alpha.
///
/// The output is *not* gamma-corrected; the shaders treat it as linear even
/// though the input was sRGB. This is a known trade-off in the current
/// pipeline (see `shaders.rs`).
pub fn color_to_linear(color: Color, opacity: f32) -> [f32; 4] {
    [
        color.r as f32 / 255.0,
        color.g as f32 / 255.0,
        color.b as f32 / 255.0,
        color.a as f32 / 255.0 * opacity,
    ]
}
