use crate::core::SizeU32;

use super::{RendererError, RendererResult, WgpuContext};

pub async fn read_rgba8_texture(
    context: &WgpuContext,
    texture: &wgpu::Texture,
    size: SizeU32,
) -> RendererResult<Vec<u8>> {
    let bytes_per_pixel = 4u32;
    let unpadded_bytes_per_row = size.width * bytes_per_pixel;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(align) * align;
    let output_size = padded_bytes_per_row as u64 * size.height as u64;
    let buffer = context.device().create_buffer(&wgpu::BufferDescriptor {
        label: Some("rgui-readback-buffer"),
        size: output_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = context
        .device()
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("rgui-readback-encoder"),
        });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: Some(size.height),
            },
        },
        wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        },
    );
    context.queue().submit(Some(encoder.finish()));

    let slice = buffer.slice(..);
    let (sender, receiver) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });
    let _ = context.device().poll(wgpu::PollType::wait_indefinitely());
    receiver
        .recv()
        .map_err(|error| RendererError::Readback(error.to_string()))??;

    let mapped = slice.get_mapped_range();
    let mut pixels = vec![0u8; (size.width * size.height * bytes_per_pixel) as usize];
    for y in 0..size.height as usize {
        let src_offset = y * padded_bytes_per_row as usize;
        let dst_offset = y * unpadded_bytes_per_row as usize;
        pixels[dst_offset..dst_offset + unpadded_bytes_per_row as usize]
            .copy_from_slice(&mapped[src_offset..src_offset + unpadded_bytes_per_row as usize]);
    }
    drop(mapped);
    buffer.unmap();
    Ok(pixels)
}
