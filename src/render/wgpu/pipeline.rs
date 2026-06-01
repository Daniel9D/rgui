use std::collections::HashMap;

use bytemuck::{Pod, Zeroable};

use super::SHADER_SOURCE;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PipelineKind {
    SolidRect,
    RoundedRect,
    Border,
    TextGlyph,
    Image,
    Svg,
    Path,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct InstanceRaw {
    pub rect: [f32; 4],
    pub color: [f32; 4],
    pub uv_rect: [f32; 4],
    pub viewport: [f32; 4],
    pub flags: [f32; 4],
}

impl InstanceRaw {
    pub fn vertex_buffer_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
            0 => Float32x4,
            1 => Float32x4,
            2 => Float32x4,
            3 => Float32x4,
            4 => Float32x4
        ];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &ATTRIBUTES,
        }
    }
}

pub struct PipelineCache {
    pipelines: HashMap<PipelineKind, wgpu::RenderPipeline>,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl PipelineCache {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("rgui-atlas-bind-group-layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("rgui-render-shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SOURCE.into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("rgui-render-pipeline-layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let mut pipelines = HashMap::new();
        for (kind, fragment_entry) in pipeline_table() {
            pipelines.insert(
                kind,
                create_pipeline(device, &layout, &shader, format, fragment_entry),
            );
        }

        Self {
            pipelines,
            bind_group_layout,
        }
    }

    pub fn pipeline(&self, kind: PipelineKind) -> &wgpu::RenderPipeline {
        // `pipeline_table()` enumerates every `PipelineKind` variant, so a
        // missing entry would indicate a programming error (a new variant
        // added without updating the table). The table is the single source
        // of truth, so this panic is unreachable in correct code.
        match self.pipelines.get(&kind) {
            Some(p) => p,
            None => unreachable!(
                "PipelineKind::{kind:?} missing from pipeline_table(); \
                 add it to render::wgpu::pipeline::pipeline_table"
            ),
        }
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }
}

/// Single source of truth mapping each `PipelineKind` to the fragment entry
/// point in `SHADER_SOURCE` that should be used for it. Keeping this in one
/// place makes it easy to audit which shader entry every pipeline runs.
fn pipeline_table() -> [(PipelineKind, &'static str); 7] {
    [
        (PipelineKind::SolidRect, "fs_main"),
        (PipelineKind::Border, "fs_main"),
        (PipelineKind::Path, "fs_main"),
        (PipelineKind::RoundedRect, "fs_rounded"),
        (PipelineKind::TextGlyph, "fs_main"),
        (PipelineKind::Image, "fs_textured"),
        (PipelineKind::Svg, "fs_textured"),
    ]
}

fn create_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    format: wgpu::TextureFormat,
    fragment_entry: &str,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("rgui-render-pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            buffers: &[InstanceRaw::vertex_buffer_layout()],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some(fragment_entry),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}
