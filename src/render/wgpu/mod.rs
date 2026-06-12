pub mod atlas;
pub mod batch;
#[cfg(feature = "bitmap-text-fallback")]
mod bitmap_text;
pub mod color;
pub mod constants;
pub mod context;
pub mod debug;
pub mod debug_env;
pub mod error;
mod glyphon_text;
pub mod item;
pub mod offscreen;
pub mod options;
pub mod pipeline;
pub mod readback;
pub mod shaders;
pub mod shared_device;
pub mod surface;
pub mod text;

pub use atlas::{AtlasAllocation, GpuAtlas, TextureAtlas};
pub use batch::build_batches_from_items;
pub use context::WgpuContext;
pub use error::{RendererError, RendererResult};
pub use glyphon_text::{GlyphonTextBridge, GlyphonTextStats};
pub use item::{MAX_RENDER_ITEMS_PER_FRAME, RenderItem, build_render_items};
pub use offscreen::OffscreenTarget;
pub use options::{RenderConfig, RendererOptions};
pub use pipeline::{InstanceRaw, PipelineCache, PipelineKind};
pub use readback::read_rgba8_texture;
pub use shaders::SHADER_SOURCE;
pub use shared_device::SharedWgpuDevice;
pub use surface::SurfaceRenderer;

use std::sync::{Arc, Mutex};

use crate::core::{
    AtlasEntryKind, DisplayList, ImageId, RenderStats, RendererBackend, ResourceStore, SizeU32,
};

/// The `wgpu`-backed renderer. Lowers a [`DisplayList`]
/// into GPU draw calls and submits them to a `wgpu::Surface` or an
/// offscreen target.
///
/// ```rust,no_run
/// use rgui::render::wgpu::WgpuRenderer;
/// // WgpuRenderer requires a real GPU device to construct. The doctest
/// // only verifies the type name + import path resolve.
/// let _ = std::marker::PhantomData::<WgpuRenderer>;
/// ```
pub struct WgpuRenderer {
    context: WgpuContext,
    pipelines: PipelineCache,
    /// Phase 4 / Plan 04-03: the atlas is wrapped in
    /// `Arc<Mutex<>>` so multiple `WgpuRenderer` instances can share
    /// one atlas (D-07, D-09). The single-window path wraps a fresh
    /// atlas; the multi-window path clones the `SharedWgpuDevice`'s
    /// `Arc`.
    atlas: Arc<Mutex<GpuAtlas>>,
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
        let atlas = Arc::new(Mutex::new(GpuAtlas::new(
            context.device(),
            SizeU32::new(1, 1),
            pipelines.bind_group_layout(),
        )));
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
            instance_capacity: constants::INITIAL_INSTANCE_CAPACITY,
        })
    }

    /// Phase 4 / Plan 04-03: build a per-window renderer that shares
    /// the atlas (and adapter / device / queue) with other renderers
    /// built from the same [`SharedWgpuDevice`].
    ///
    /// The `surface` is borrowed for capability detection (format);
    /// the host owns the `wgpu::Surface` per the wgpu 29 + winit 0.30
    /// surface model. Each window's `SurfaceRenderer` calls
    /// `surface.configure(...)` with the format returned here.
    ///
    /// The shared atlas is wrapped in `Arc<Mutex<GpuAtlas>>` (D-09):
    /// per-frame bind group construction locks the mutex briefly,
    /// and glyph uploads also lock the mutex. Contention is rare in
    /// practice (most frames don't upload new glyphs). Profiling
    /// can guide a future swap to `RwLock` if contention becomes a
    /// bottleneck.
    pub fn with_shared_device(
        shared: &SharedWgpuDevice,
        surface: &wgpu::Surface<'static>,
        opts: RendererOptions,
    ) -> RendererResult<Self> {
        let capabilities = surface.get_capabilities(shared.adapter());
        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .unwrap_or(opts.format);
        let size = opts.initial_size;

        let instance = wgpu::Instance::new(context::instance_descriptor(opts.backends));
        let context = WgpuContext::from_parts(
            instance,
            (**shared.adapter()).clone(),
            (**shared.device()).clone(),
            (**shared.queue()).clone(),
            format,
            size,
        );
        let pipelines = PipelineCache::new(shared.device(), format);
        let atlas = Arc::clone(shared.atlas());
        let text_bridge = GlyphonTextBridge::new(shared.device(), shared.queue(), format);
        let instance_buffer =
            shared
                .device()
                .create_buffer(&wgpu::BufferDescriptor {
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
            instance_capacity: constants::INITIAL_INSTANCE_CAPACITY,
        })
    }

    pub fn new_headless_for_tests() -> Self {
        let defaults = RendererOptions::default();
        Self::new_headless_for_tests_with_backends(
            defaults.initial_size,
            defaults.format,
            defaults.backends,
        )
    }

    /// Headless test seam that lets a test pick the GPU backend, surface
    /// size, and texture format. Used by `tests/visual_goldens_vulkan.rs`
    /// to render the existing visual goldens against a non-primary
    /// backend (e.g. `wgpu::Backends::VULKAN`) and confirm that the
    /// wgpu render path is stable across backends (REND-01).
    ///
    /// All other `RendererOptions` fields (e.g. `power_preference`) fall
    /// back to `RendererOptions::default()`. Pass
    /// `RendererOptions::default().backends` to reproduce the behavior
    /// of `[\`new_headless_for_tests\`]`.
    pub fn new_headless_for_tests_with_backends(
        size: SizeU32,
        format: wgpu::TextureFormat,
        backends: wgpu::Backends,
    ) -> Self {
        pollster::block_on(Self::new_headless(RendererOptions {
            initial_size: size,
            format,
            backends,
            ..RendererOptions::default()
        }))
        .expect("headless renderer initializes")
    }

    pub fn context(&self) -> &WgpuContext {
        &self.context
    }

    pub fn context_mut(&mut self) -> &mut WgpuContext {
        &mut self.context
    }

    /// Handle to the shared atlas. The handle is the same one every
    /// `WgpuRenderer` built from the same `SharedWgpuDevice` holds;
    /// all renderers see the same atlas contents. The single-window
    /// path returns the `Arc<Mutex<GpuAtlas>>` for the local atlas
    /// (the only renderer referencing it is this one).
    pub fn atlas(&self) -> &Arc<Mutex<GpuAtlas>> {
        &self.atlas
    }

    /// Acquire a `MutexGuard` on the shared atlas. Callers that
    /// need `&mut GpuAtlas` (e.g. the render-item builder in the
    /// test suite) use this; the guard derefs to `&mut GpuAtlas`
    /// and is held for the lifetime of the borrow. The mutex is
    /// the same one every `WgpuRenderer` built from the same
    /// `SharedWgpuDevice` locks; all renders serialize through it
    /// (D-09).
    pub fn atlas_mut(&self) -> std::sync::MutexGuard<'_, GpuAtlas> {
        self.atlas
            .lock()
            .expect("WgpuRenderer atlas mutex poisoned")
    }

    /// Upload an RGBA8 image to the shared atlas. The atlas mutex
    /// is locked for the duration of the upload (a single
    /// `queue.write_texture` call).
    pub fn upload_atlas_rgba8(
        &mut self,
        id: ImageId,
        size: SizeU32,
        rgba: &[u8],
    ) -> RendererResult<()> {
        let mut atlas = self
            .atlas
            .lock()
            .map_err(|_| RendererError::InvalidDisplayList("atlas mutex poisoned".into()))?;
        atlas
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
        // Resource uploads are host-driven: the host is expected to call
        // `WgpuRenderer::upload_atlas_rgba8` (and the SVG/glyph equivalents)
        // before issuing frames. Items whose atlas entry is missing are
        // rendered via `missing_resource_item` (a magenta placeholder) so a
        // missing upload is visible during development rather than a silent
        // black hole.
        //
        // We do not auto-scan `display_list` here for missing images, because
        // (a) the ResourceStore does not always own the raw bytes (they may
        // live in the host's own cache), and (b) the host usually wants to
        // own the upload timing to avoid uploading every frame.
        let items = {
            let mut atlas = self
                .atlas
                .lock()
                .map_err(|_| RendererError::InvalidDisplayList("atlas mutex poisoned".into()))?;
            build_render_items(display_list, resources, &mut atlas)?
        };
        let batches = build_batches_from_items(&items);
        let text_stats = self.text_bridge.prepare(
            self.context.device(),
            self.context.queue(),
            display_list,
            self.context.size(),
        )?;
        if debug_env::dump_render_items() {
            eprintln!("{}", debug::format_render_items(&items));
        }
        if debug_env::dump_batches() {
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
            // Lock the atlas briefly to read the bind group handle.
            // The lock is held for the `set_bind_group` call only.
            let atlas = self
                .atlas
                .lock()
                .map_err(|_| RendererError::InvalidDisplayList("atlas mutex poisoned".into()))?;
            pass.set_bind_group(0, atlas.bind_group(), &[]);
            drop(atlas);
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
            text_cache: crate::text_engine::TextCacheStats::default(),
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
                gradient: item.gradient,
                gradient_end_color: item.gradient_end_color,
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

    // Reject non-finite geometry: a NaN/inf rect would otherwise produce a
    // bogus scissor (e.g. u32::MAX from `inf as u32`) that wgpu would either
    // reject outright or render as garbage. Returning `None` causes the
    // entire batch to be skipped, which matches the existing empty-clip path.
    if !rect.origin.x.is_finite()
        || !rect.origin.y.is_finite()
        || !rect.size.width.is_finite()
        || !rect.size.height.is_finite()
    {
        return None;
    }

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

impl WgpuRenderer {
    /// Like `RendererBackend::render` but surfaces render-time errors instead
    /// of panicking. Prefer this in long-running hosts that can recover.
    pub fn try_render(
        &mut self,
        display_list: &DisplayList,
        resources: &ResourceStore,
    ) -> RendererResult<RenderStats> {
        let target_size = self.context.size();
        let target = OffscreenTarget::new(&self.context, target_size);
        self.render_to_target(display_list, resources, target.view())
    }
}

impl RendererBackend for WgpuRenderer {
    fn resize(&mut self, size: SizeU32) {
        self.context.resize(size);
    }

    fn render(&mut self, display_list: &DisplayList, resources: &ResourceStore) -> RenderStats {
        // Trait contract returns `RenderStats`, not `Result`. Log the error
        // and return zeroed stats so the host can keep running. Callers that
        // care about the error should use `WgpuRenderer::try_render` instead.
        match self.try_render(display_list, resources) {
            Ok(stats) => stats,
            Err(err) => {
                eprintln!("WgpuRenderer::render failed: {err}");
                RenderStats::default()
            }
        }
    }
}
