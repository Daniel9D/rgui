pub mod atlas;
pub mod batch;
#[cfg(feature = "bitmap-text-fallback")]
mod bitmap_text;
pub mod context;
pub mod debug;
pub mod error;
mod glyphon_text;
pub mod item;
pub mod offscreen;
pub mod options;
pub mod pipeline;
pub mod readback;
pub mod shaders;
pub mod surface;
pub mod text;

pub use atlas::{AtlasAllocation, GpuAtlas, TextureAtlas};
pub use batch::build_batches_from_items;
pub use context::WgpuContext;
pub use error::{RendererError, RendererResult};
pub use glyphon_text::{GlyphonTextBridge, GlyphonTextStats};
pub use item::{MAX_RENDER_ITEMS_PER_FRAME, RenderItem, build_render_items};
pub use offscreen::OffscreenTarget;
pub use options::RendererOptions;
pub use pipeline::{InstanceRaw, PipelineCache, PipelineKind};
pub use readback::read_rgba8_texture;
pub use shaders::SHADER_SOURCE;
pub use surface::SurfaceRenderer;

use crate::core::{
    AtlasEntryKind, DisplayList, ImageId, RenderStats, RendererBackend, ResourceStore, SizeU32,
};

pub struct WgpuRenderer {
    context: WgpuContext,
    pipelines: PipelineCache,
    atlas: GpuAtlas,
    text_bridge: GlyphonTextBridge,
    instance_buffer: wgpu::Buffer,
    instance_capacity: usize,
}

impl WgpuRenderer {
    pub async fn new_headless(options: RendererOptions) -> RendererResult<Self> {
        let context = WgpuContext::headless(options).await?;
        Self::from_context(context)
    }

    pub fn from_context(context: WgpuContext) -> RendererResult<Self> {
        let pipelines = PipelineCache::new(context.device(), context.format());
        let atlas = GpuAtlas::new(
            context.device(),
            SizeU32::new(1, 1),
            pipelines.bind_group_layout(),
        );
        let text_bridge =
            GlyphonTextBridge::new(context.device(), context.queue(), context.format());
        let instance_buffer = context.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("rgui-instance-buffer"),
            size: std::mem::size_of::<InstanceRaw>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Ok(Self {
            context,
            pipelines,
            atlas,
            text_bridge,
            instance_buffer,
            instance_capacity: 1,
        })
    }

    pub fn new_headless_for_tests() -> Self {
        pollster::block_on(Self::new_headless(RendererOptions::default()))
            .expect("headless renderer initializes")
    }

    pub fn context(&self) -> &WgpuContext {
        &self.context
    }

    pub fn context_mut(&mut self) -> &mut WgpuContext {
        &mut self.context
    }

    pub fn atlas(&self) -> &GpuAtlas {
        &self.atlas
    }

    pub fn atlas_mut(&mut self) -> &mut GpuAtlas {
        &mut self.atlas
    }

    pub fn upload_atlas_rgba8(
        &mut self,
        id: ImageId,
        size: SizeU32,
        rgba: &[u8],
    ) -> RendererResult<()> {
        self.atlas
            .upload_rgba8(self.context.queue(), AtlasEntryKind::Image(id), size, rgba)
            .ok_or_else(|| RendererError::InvalidDisplayList("atlas allocation failed".into()))?;
        Ok(())
    }

    pub fn render_to_target(
        &mut self,
        display_list: &DisplayList,
        resources: &ResourceStore,
        target: &wgpu::TextureView,
    ) -> RendererResult<RenderStats> {
        // Step 3a: Prepare resources (upload images/SVGs) before lowering render items
        // TODO: implement prepare_resources() - currently a no-op placeholder
        let items = build_render_items(display_list, resources, &mut self.atlas)?;
        let batches = build_batches_from_items(&items);
        let text_stats = self.text_bridge.prepare(
            self.context.device(),
            self.context.queue(),
            display_list,
            self.context.size(),
        )?;
        if std::env::var_os("RGUI_DEBUG_RENDER_ITEMS").is_some() {
            eprintln!("{}", debug::format_render_items(&items));
        }
        if std::env::var_os("RGUI_DEBUG_BATCHES").is_some() {
            eprintln!("{}", debug::format_render_batches(&batches));
        }
        let instances = self.instances_for_items(&items);
        self.ensure_instance_capacity(instances.len().max(1));
        if !instances.is_empty() {
            self.context.queue().write_buffer(
                &self.instance_buffer,
                0,
                bytemuck::cast_slice(&instances),
            );
        }

        let mut encoder =
            self.context
                .device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("rgui-render-encoder"),
                });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("rgui-render-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_bind_group(0, self.atlas.bind_group(), &[]);
            pass.set_vertex_buffer(0, self.instance_buffer.slice(..));
            for batch in &batches {
                let Some(scissor) = scissor_rect(batch.key.clip_rect, self.context.size()) else {
                    continue;
                };
                pass.set_scissor_rect(scissor.0, scissor.1, scissor.2, scissor.3);
                pass.set_pipeline(self.pipelines.pipeline(batch.key.pipeline));
                pass.draw(
                    0..6,
                    batch.first_item as u32..(batch.first_item + batch.command_count) as u32,
                );
            }

            // Glyphon clips individual text areas through TextBounds; do not let the
            // last shape batch's render-pass scissor clip all text.
            let viewport = self.context.size();
            pass.set_scissor_rect(0, 0, viewport.width, viewport.height);
            self.text_bridge.render(&mut pass)?;
        }
        self.context.queue().submit(Some(encoder.finish()));
        Ok(RenderStats {
            command_count: display_list.commands().len(),
            batch_count: batches.len(),
            atlas_upload_bytes: 0,
            render_item_count: items.len(),
            text_item_count: display_list
                .commands()
                .iter()
                .filter(|command| matches!(command, crate::core::PaintCommand::DrawText(_)))
                .count(),
            clip_batch_count: batches
                .iter()
                .filter(|batch| batch.key.clip_rect.is_some())
                .count(),
            glyphon_enabled: text_stats.glyphon_enabled,
            text_area_count: text_stats.text_area_count,
            clipped_text_area_count: text_stats.clipped_text_area_count,
            skipped_text_area_count: text_stats.skipped_text_area_count,
            glyph_count: text_stats.glyph_count,
            fallback_used: text_stats.fallback_used,
        })
    }

    fn instances_for_items(&self, items: &[RenderItem]) -> Vec<InstanceRaw> {
        let viewport = [
            self.context.size().width as f32,
            self.context.size().height as f32,
            0.0,
            0.0,
        ];
        items
            .iter()
            .map(|item| InstanceRaw {
                rect: [
                    item.rect.origin.x,
                    item.rect.origin.y,
                    item.rect.size.width,
                    item.rect.size.height,
                ],
                color: item.color,
                uv_rect: item.uv_rect,
                viewport,
                flags: [item.radius, 0.0, 0.0, 0.0],
            })
            .collect()
    }

    fn ensure_instance_capacity(&mut self, required: usize) {
        if required <= self.instance_capacity {
            return;
        }
        self.instance_capacity = required.next_power_of_two();
        self.instance_buffer = self
            .context
            .device()
            .create_buffer(&wgpu::BufferDescriptor {
                label: Some("rgui-instance-buffer"),
                size: (self.instance_capacity * std::mem::size_of::<InstanceRaw>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
    }
}

fn scissor_rect(clip_rect: Option<crate::Rect>, viewport: SizeU32) -> Option<(u32, u32, u32, u32)> {
    let Some(rect) = clip_rect else {
        return Some((0, 0, viewport.width, viewport.height));
    };

    let x0 = rect.origin.x.max(0.0).floor() as u32;
    let y0 = rect.origin.y.max(0.0).floor() as u32;
    let x1 = rect.max_x().min(viewport.width as f32).ceil().max(0.0) as u32;
    let y1 = rect.max_y().min(viewport.height as f32).ceil().max(0.0) as u32;
    let width = x1.saturating_sub(x0);
    let height = y1.saturating_sub(y0);
    if width == 0 || height == 0 {
        return None;
    }
    Some((x0, y0, width, height))
}

impl RendererBackend for WgpuRenderer {
    fn resize(&mut self, size: SizeU32) {
        self.context.resize(size);
    }

    fn render(&mut self, display_list: &DisplayList, resources: &ResourceStore) -> RenderStats {
        let target_size = self.context.size();
        let target = OffscreenTarget::new(&self.context, target_size);
        self.render_to_target(display_list, resources, target.view())
            .expect("offscreen render succeeds")
    }
}
