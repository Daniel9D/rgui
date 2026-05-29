use crate::core::{DisplayList, RenderStats, ResourceStore, SizeU32};
use winit::window::Window;

use super::{RendererError, RendererOptions, RendererResult, WgpuContext, WgpuRenderer};

pub struct SurfaceRenderer {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    renderer: WgpuRenderer,
}

impl SurfaceRenderer {
    pub async fn new(window: &Window, options: RendererOptions) -> RendererResult<Self> {
        let mut desc = wgpu::InstanceDescriptor::new_without_display_handle();
        desc.backends = options.backends;
        let instance = wgpu::Instance::new(desc);
        // SAFETY: The window outlives the surface renderer in all examples and usage patterns
        let surface = unsafe {
            std::mem::transmute::<wgpu::Surface<'_>, wgpu::Surface<'static>>(
                instance
                    .create_surface(window)
                    .map_err(|_| RendererError::Surface("surface creation failed".to_string()))?,
            )
        };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: options.power_preference,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .map_err(|_| RendererError::NoAdapter)?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("rgui-surface-device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;
        let size = SizeU32::new(
            window.inner_size().width.max(1),
            window.inner_size().height.max(1),
        );
        let capabilities = surface.get_capabilities(&adapter);
        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .unwrap_or(options.format);
        let present_mode = capabilities
            .present_modes
            .iter()
            .copied()
            .find(|mode| *mode == wgpu::PresentMode::Fifo)
            .unwrap_or(wgpu::PresentMode::Fifo);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode,
            alpha_mode: capabilities
                .alpha_modes
                .first()
                .copied()
                .unwrap_or(wgpu::CompositeAlphaMode::Auto),
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let context = WgpuContext::from_parts(instance, adapter, device, queue, format, size);
        let renderer = WgpuRenderer::from_context(context)?;
        Ok(Self {
            surface,
            config,
            renderer,
        })
    }

    pub fn resize(&mut self, size: SizeU32) {
        let size = SizeU32::new(size.width.max(1), size.height.max(1));
        self.config.width = size.width;
        self.config.height = size.height;
        self.renderer.context_mut().resize(size);
        self.surface
            .configure(self.renderer.context().device(), &self.config);
    }

    pub fn render(
        &mut self,
        display_list: &DisplayList,
        resources: &ResourceStore,
    ) -> RendererResult<RenderStats> {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(RenderStats::default());
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface
                    .configure(self.renderer.context().device(), &self.config);
                return Ok(RenderStats::default());
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err(RendererError::Surface(
                    "surface texture validation failed".to_string(),
                ));
            }
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let stats = self
            .renderer
            .render_to_target(display_list, resources, &view)?;
        frame.present();
        Ok(stats)
    }

    pub fn renderer(&self) -> &WgpuRenderer {
        &self.renderer
    }
}
