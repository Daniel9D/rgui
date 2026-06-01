use crate::core::SizeU32;

use super::constants;

#[derive(Clone, Debug)]
pub struct RendererOptions {
    pub initial_size: SizeU32,
    pub format: wgpu::TextureFormat,
    pub power_preference: wgpu::PowerPreference,
    pub backends: wgpu::Backends,
}

impl Default for RendererOptions {
    fn default() -> Self {
        Self {
            initial_size: SizeU32::new(1, 1),
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            power_preference: wgpu::PowerPreference::default(),
            backends: wgpu::Backends::PRIMARY,
        }
    }
}

/// High-level rendering knobs that the host application can tune without
/// touching the wgpu device adapter details covered by `RendererOptions`.
///
/// The defaults mirror the values in `render::wgpu::constants`; if you change
/// a value here, update the corresponding constant so the two stay in sync.
#[derive(Clone, Copy, Debug)]
pub struct RenderConfig {
    /// Hard cap on render items per frame. Exceeding this raises
    /// `RendererError::InvalidDisplayList` during lowering.
    pub max_render_items_per_frame: usize,
    /// Initial GPU atlas side length, in pixels. Smaller requests are
    /// clamped up; the atlas grows automatically as needed.
    pub atlas_min_size: u32,
    /// Maximum number of frames the surface will keep in flight.
    pub desired_maximum_frame_latency: u32,
    /// Default line-height ratio when a text command omits one.
    pub text_line_height_ratio: f32,
    /// Default baseline ratio when a text command omits one.
    pub text_baseline_ratio: f32,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            max_render_items_per_frame: constants::MAX_RENDER_ITEMS_PER_FRAME,
            atlas_min_size: constants::ATLAS_MIN_SIZE,
            desired_maximum_frame_latency: constants::DESIRED_MAXIMUM_FRAME_LATENCY,
            text_line_height_ratio: constants::TEXT_LINE_HEIGHT_RATIO,
            text_baseline_ratio: constants::TEXT_BASELINE_RATIO,
        }
    }
}
