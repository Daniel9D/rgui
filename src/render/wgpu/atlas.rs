use std::collections::HashMap;

use crate::core::{AtlasEntryKind, GlyphKey, ImageId, Rect, SizeU32, SvgId};

#[derive(Clone, Debug, PartialEq)]
pub struct AtlasAllocation {
    pub rect: Rect,
    pub generation: u64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlyphAtlasEntry {
    pub uv_rect: [f32; 4],
    pub generation: u64,
}

#[derive(Clone, Debug)]
pub struct TextureAtlas {
    size: SizeU32,
    cursor_x: u32,
    cursor_y: u32,
    row_height: u32,
    generation: u64,
    entries: Vec<(AtlasEntryKind, AtlasAllocation)>,
}

impl TextureAtlas {
    pub fn new(size: SizeU32) -> Self {
        Self {
            size,
            cursor_x: 0,
            cursor_y: 0,
            row_height: 0,
            generation: 0,
            entries: Vec::new(),
        }
    }

    pub fn allocate(&mut self, kind: AtlasEntryKind, size: SizeU32) -> Option<AtlasAllocation> {
        if size.width > self.size.width || size.height > self.size.height {
            return None;
        }
        if self.cursor_x + size.width > self.size.width {
            self.cursor_x = 0;
            self.cursor_y += self.row_height;
            self.row_height = 0;
        }
        if self.cursor_y + size.height > self.size.height {
            self.evict_all();
        }
        if self.cursor_y + size.height > self.size.height {
            return None;
        }
        self.generation += 1;
        let allocation = AtlasAllocation {
            rect: Rect::new(
                crate::core::Point::new(self.cursor_x as f32, self.cursor_y as f32),
                crate::core::Size::new(size.width as f32, size.height as f32),
            ),
            generation: self.generation,
        };
        self.cursor_x += size.width;
        self.row_height = self.row_height.max(size.height);
        self.entries.push((kind, allocation.clone()));
        Some(allocation)
    }

    pub fn occupancy_count(&self) -> usize {
        self.entries.len()
    }

    fn evict_all(&mut self) {
        self.entries.clear();
        self.cursor_x = 0;
        self.cursor_y = 0;
        self.row_height = 0;
    }
}

pub struct GpuAtlas {
    size: SizeU32,
    cpu: TextureAtlas,
    entries: HashMap<GpuAtlasKey, AtlasAllocation>,
    texture: wgpu::Texture,
    _view: wgpu::TextureView,
    _sampler: wgpu::Sampler,
    bind_group: wgpu::BindGroup,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum GpuAtlasKey {
    Image(ImageId),
    Svg(SvgId),
    Glyph(GlyphKey),
}

impl From<AtlasEntryKind> for GpuAtlasKey {
    fn from(value: AtlasEntryKind) -> Self {
        match value {
            AtlasEntryKind::Image(id) => Self::Image(id),
            AtlasEntryKind::Svg(id) => Self::Svg(id),
            AtlasEntryKind::Glyph(key) => Self::Glyph(key),
        }
    }
}

impl GpuAtlas {
    pub fn new(
        device: &wgpu::Device,
        size: SizeU32,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let size = SizeU32::new(size.width.max(1024), size.height.max(1024));
        let cpu = TextureAtlas::new(size);
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("rgui-gpu-atlas"),
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("rgui-atlas-sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("rgui-atlas-bind-group"),
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });
        Self {
            size,
            cpu,
            entries: HashMap::new(),
            texture,
            _view: view,
            _sampler: sampler,
            bind_group,
        }
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn upload_rgba8(
        &mut self,
        queue: &wgpu::Queue,
        kind: AtlasEntryKind,
        size: SizeU32,
        rgba: &[u8],
    ) -> Option<AtlasAllocation> {
        let allocation = self.cpu.allocate(kind.clone(), size)?;
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: allocation.rect.origin.x as u32,
                    y: allocation.rect.origin.y as u32,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(size.width * 4),
                rows_per_image: Some(size.height),
            },
            wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
        );
        self.entries
            .insert(GpuAtlasKey::from(kind), allocation.clone());
        Some(allocation)
    }

    pub fn uv_for(&self, kind: &AtlasEntryKind) -> Option<[f32; 4]> {
        let allocation = self.entries.get(&GpuAtlasKey::from(kind.clone()))?;
        Some(self.uv_for_allocation(allocation))
    }

    pub fn glyph_entry(&self, key: GlyphKey) -> Option<GlyphAtlasEntry> {
        let allocation = self.entries.get(&GpuAtlasKey::Glyph(key))?;
        Some(GlyphAtlasEntry {
            uv_rect: self.uv_for_allocation(allocation),
            generation: allocation.generation,
        })
    }

    pub fn reserve_glyph(&mut self, key: GlyphKey, size: SizeU32) -> Option<GlyphAtlasEntry> {
        let allocation = self.cpu.allocate(
            AtlasEntryKind::Glyph(key.clone()),
            SizeU32::new(size.width.max(1), size.height.max(1)),
        )?;
        self.entries
            .insert(GpuAtlasKey::Glyph(key), allocation.clone());
        Some(GlyphAtlasEntry {
            uv_rect: self.uv_for_allocation(&allocation),
            generation: allocation.generation,
        })
    }

    fn uv_for_allocation(&self, allocation: &AtlasAllocation) -> [f32; 4] {
        let x0 = allocation.rect.origin.x / self.size.width as f32;
        let y0 = allocation.rect.origin.y / self.size.height as f32;
        let x1 = (allocation.rect.origin.x + allocation.rect.size.width) / self.size.width as f32;
        let y1 = (allocation.rect.origin.y + allocation.rect.size.height) / self.size.height as f32;
        [x0, y0, x1, y1]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{GlyphKey, SizeU32};

    #[test]
    fn shelf_allocator_wraps_rows_and_tracks_generation() {
        let mut atlas = TextureAtlas::new(SizeU32::new(16, 16));
        let first = atlas
            .allocate(
                AtlasEntryKind::Glyph(GlyphKey {
                    font_id: 1,
                    glyph_id: 1,
                    size_bits: 12,
                }),
                SizeU32::new(10, 8),
            )
            .expect("first glyph fits");
        let second = atlas
            .allocate(
                AtlasEntryKind::Glyph(GlyphKey {
                    font_id: 1,
                    glyph_id: 2,
                    size_bits: 12,
                }),
                SizeU32::new(10, 8),
            )
            .expect("second glyph fits on next row");

        assert_eq!(first.rect.origin.y, 0.0);
        assert_eq!(second.rect.origin.y, 8.0);
        assert!(second.generation > first.generation);
    }
}
