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
        for kind in [
            PipelineKind::SolidRect,
            PipelineKind::Border,
            PipelineKind::Path,
        ] {
            pipelines.insert(
                kind,
                create_pipeline(device, &layout, &shader, format, "fs_main"),
            );
        }
        pipelines.insert(
            PipelineKind::RoundedRect,
            create_pipeline(device, &layout, &shader, format, "fs_rounded"),
        );
        pipelines.insert(
            PipelineKind::TextGlyph,
            create_pipeline(device, &layout, &shader, format, "fs_main"),
        );
        for kind in [PipelineKind::Image, PipelineKind::Svg] {
            pipelines.insert(
                kind,
                create_pipeline(device, &layout, &shader, format, "fs_textured"),
            );
        }

        Self {
            pipelines,
            bind_group_layout,
        }
    }

    pub fn pipeline(&self, kind: PipelineKind) -> &wgpu::RenderPipeline {
        self.pipelines
            .get(&kind)
            .expect("pipeline is created for every PipelineKind")
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }
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
