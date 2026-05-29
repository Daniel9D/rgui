use crate::core::SizeU32;

use super::{RendererError, RendererOptions};

pub struct WgpuContext {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    format: wgpu::TextureFormat,
    size: SizeU32,
}

impl WgpuContext {
    pub fn from_parts(
        instance: wgpu::Instance,
        adapter: wgpu::Adapter,
        device: wgpu::Device,
        queue: wgpu::Queue,
        format: wgpu::TextureFormat,
        size: SizeU32,
    ) -> Self {
        Self {
            instance,
            adapter,
            device,
            queue,
            format,
            size,
        }
    }

    pub async fn headless(options: RendererOptions) -> Result<Self, RendererError> {
        let instance = wgpu::Instance::new(instance_descriptor(options.backends));
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: options.power_preference,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .map_err(|_| RendererError::NoAdapter)?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("rgui headless device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            format: options.format,
            size: options.initial_size,
        })
    }

    pub fn instance(&self) -> &wgpu::Instance {
        &self.instance
    }

    pub fn adapter(&self) -> &wgpu::Adapter {
        &self.adapter
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    pub fn limits(&self) -> wgpu::Limits {
        self.adapter.limits()
    }

    pub fn size(&self) -> SizeU32 {
        self.size
    }

    pub fn resize(&mut self, size: SizeU32) {
        self.size = SizeU32::new(size.width.max(1), size.height.max(1));
    }
}

pub(crate) fn instance_descriptor(backends: wgpu::Backends) -> wgpu::InstanceDescriptor {
    let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
    descriptor.backends = backends;
    descriptor
}
