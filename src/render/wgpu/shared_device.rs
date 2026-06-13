//! Phase 4 / Plan 04-03: `SharedWgpuDevice` (D-06, D-07, D-09).
//!
//! A `SharedWgpuDevice` bundles the per-process wgpu state that
//! every `WgpuRenderer` (one per window) shares:
//!
//! - `Arc<wgpu::Adapter>` — the discovered adapter.
//! - `Arc<wgpu::Device>` — the device handle.
//! - `Arc<wgpu::Queue>` — the queue handle.
//! - `Arc<std::sync::Mutex<GpuAtlas>>` — the shared GPU atlas.
//!
//! Construction is async (`SharedWgpuDevice::new` is `async`) because
//! `wgpu::request_adapter` and `wgpu::Adapter::request_device` are
//! async by design. Production hosts call it from inside their async
//! runtime; tests and examples use `pollster::block_on` (the same
//! pattern as `SurfaceRenderer::new` in `examples/widgets.rs`).
//!
//! The atlas is shared, not per-window. Glyphs uploaded by any window
//! are visible to all windows — the common case (the system font's
//! ASCII glyphs) is uploaded once and reused. New glyphs (CJK,
//! custom fonts) are uploaded lazily on first use in any window.
//!
//! # Lock pattern (D-09)
//!
//! Atlas mutations lock the mutex:
//!
//! - `GpuAtlas::upload_rgba8` takes `&Mutex<GpuAtlas>`, locks, and
//!   uploads. The lock is held for the duration of the upload (a
//!   single `queue.write_texture` call).
//! - Per-frame bind group construction reads under the same lock.
//!   The lock is brief; contention with uploads is rare (most
//!   frames don't upload new glyphs).
//!
//! `Mutex` is the conservative choice for v1.x. A future v1.x
//! release can swap to `RwLock` or a lock-free structure if
//! profiling shows contention.

use std::sync::{Arc, Mutex};

use crate::core::SizeU32;

use super::{GpuAtlas, PipelineCache, RendererError, RendererOptions, RendererResult, context};

/// Per-process wgpu state shared by every `WgpuRenderer`. Construct
/// once via `SharedWgpuDevice::new().await`; clone the handle to pass
/// to each `WgpuRenderer::with_shared_device` or
/// `SurfaceRenderer::with_shared_device` call.
pub struct SharedWgpuDevice {
    adapter: Arc<wgpu::Adapter>,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    atlas: Arc<Mutex<GpuAtlas>>,
}

impl SharedWgpuDevice {
    /// Acquire a wgpu adapter + device + queue and construct a fresh
    /// shared atlas. The atlas uses the default `Rgba8UnormSrgb`
    /// format regardless of any specific window's surface format
    /// (texture bind groups are format-agnostic for the sampling
    /// stage).
    ///
    /// The adapter is requested with `compatible_surface: None`,
    /// meaning the device is *not* tied to any specific surface. The
    /// host can still request a `compatible_surface: Some(&surface)`
    /// adapter per window for the `SurfaceRenderer`; both windows
    /// use the same `SharedWgpuDevice` (the device is surface-agnostic
    /// once acquired).
    pub async fn new(options: RendererOptions) -> RendererResult<Self> {
        let instance = wgpu::Instance::new(context::instance_descriptor(options.backends));
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
                label: Some("rgui-shared-device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;
        let device = Arc::new(device);
        let queue = Arc::new(queue);
        let adapter = Arc::new(adapter);

        // Build a temporary PipelineCache just to get a bind group
        // layout for the atlas. The atlas's bind group is layout-only
        // (texture + sampler); the per-window pipelines use the same
        // layout shape, so the atlas's bind group is reusable across
        // windows regardless of their color-attachment format.
        let temp_pipelines = PipelineCache::new(&device, options.format);
        let atlas = GpuAtlas::new(
            &device,
            SizeU32::new(1, 1),
            temp_pipelines.bind_group_layout(),
        );
        Ok(Self {
            adapter,
            device,
            queue,
            atlas: Arc::new(Mutex::new(atlas)),
        })
    }

    pub fn adapter(&self) -> &Arc<wgpu::Adapter> {
        &self.adapter
    }

    pub fn device(&self) -> &Arc<wgpu::Device> {
        &self.device
    }

    pub fn queue(&self) -> &Arc<wgpu::Queue> {
        &self.queue
    }

    /// Handle to the shared atlas. Lock the returned `Mutex` to
    /// read or write atlas state. The handle is the same one every
    /// `WgpuRenderer` built from this `SharedWgpuDevice` holds; all
    /// renderers see the same atlas contents.
    pub fn atlas(&self) -> &Arc<Mutex<GpuAtlas>> {
        &self.atlas
    }
}

impl Clone for SharedWgpuDevice {
    fn clone(&self) -> Self {
        Self {
            adapter: Arc::clone(&self.adapter),
            device: Arc::clone(&self.device),
            queue: Arc::clone(&self.queue),
            atlas: Arc::clone(&self.atlas),
        }
    }
}
