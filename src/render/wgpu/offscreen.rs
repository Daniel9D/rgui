use crate::core::SizeU32;

use super::{RendererResult, WgpuContext, read_rgba8_texture};

pub struct OffscreenTarget {
    size: SizeU32,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
}

impl OffscreenTarget {
    pub fn new(context: &WgpuContext, size: SizeU32) -> Self {
        let size = SizeU32::new(size.width.max(1), size.height.max(1));
        let texture = context.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("rgui-offscreen-target"),
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: context.format(),
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            size,
            texture,
            view,
        }
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub fn size(&self) -> SizeU32 {
        self.size
    }

    pub async fn read_rgba8(&self, context: &WgpuContext) -> RendererResult<Vec<u8>> {
        read_rgba8_texture(context, &self.texture, self.size).await
    }
}
