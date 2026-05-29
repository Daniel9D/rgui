use crate::core::SizeU32;

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
